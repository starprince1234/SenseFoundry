package com.sensefoundry.data.sync

import android.content.Context
import androidx.work.CoroutineWorker
import androidx.work.WorkerParameters
import androidx.work.workDataOf
import androidx.core.content.edit
import androidx.room.withTransaction
import com.sensefoundry.data.db.*
import com.sensefoundry.BuildConfig
import android.util.Base64
import java.nio.charset.StandardCharsets
import java.security.KeyFactory
import java.security.MessageDigest
import java.security.Signature
import java.security.spec.X509EncodedKeySpec
import kotlinx.serialization.json.*
import kotlinx.coroutines.CancellationException
import okhttp3.OkHttpClient
import okhttp3.Request
import java.io.IOException

private class SyncIntegrityException(message: String) : IllegalStateException(message)

class SyncWorker(context: Context, params: WorkerParameters) : CoroutineWorker(context, params) {
    private val client = OkHttpClient()
    override suspend fun doWork(): Result = try {
        val baseUrl = inputData.getString("apiUrl") ?: error("apiUrl is required")
        val token = applicationContext.getSharedPreferences("sync", Context.MODE_PRIVATE).getLong("token", 0)
        val manifest = getJson("$baseUrl/sync-manifests/latest/delta?last_sync_token=$token")
        val nextToken = requireMonotonicSyncToken(
            currentToken = token,
            receivedToken = manifest["sync_token"]!!.jsonPrimitive.long,
        )
        val changed = manifest["changed_editions"]?.jsonArray.orEmpty()
        val database = SenseDatabase.get(applicationContext)
        database.withTransaction {
            changed.forEach { item ->
                val entry = item.jsonObject; val id = entry.required("edition_id"); val expectedHash = entry.required("content_hash"); val expectedSignature = entry.required("signature")
                val deltaBytes = getBytes("$baseUrl/editions/$id/delta")
                val actualHash = sha256(deltaBytes)
                if (actualHash != expectedHash) {
                    throw SyncIntegrityException("content hash mismatch for edition $id")
                }
                if (!verifySignature(actualHash, expectedSignature)) {
                    throw SyncIntegrityException("signature mismatch for edition $id")
                }
                val delta = Json.parseToJsonElement(deltaBytes.decodeToString()).jsonObject
                val edition = Edition(id, delta.required("headword"), delta["version_number"]!!.jsonPrimitive.int, actualHash, expectedSignature)
                val senses = delta["senses"]!!.jsonArray.map { sense -> sense.jsonObject.let { Sense(it.required("id"), id, edition.headword, it.required("pos"), it.required("definition")) } }
                val examples = delta["examples"]!!.jsonArray.map { example -> example.jsonObject.let { Example(it.required("id"), it.required("sense_id"), it.required("sentence"), it["rank"]!!.jsonPrimitive.double) } }
                database.senseDao().replaceEdition(edition, senses, examples)
            }
        }
        applicationContext.getSharedPreferences("sync", Context.MODE_PRIVATE).edit {
            putLong("token", nextToken)
            putLong("lastSync", System.currentTimeMillis())
        }
        Result.success()
    } catch (error: CancellationException) {
        throw error
    } catch (_: SyncIntegrityException) {
        Result.failure(workDataOf("reason" to "integrity"))
    } catch (_: SyncTokenRegressionException) {
        Result.failure(workDataOf("reason" to "version_regression"))
    } catch (_: IOException) {
        Result.retry()
    } catch (_: Exception) {
        Result.failure(workDataOf("reason" to "invalid_delta"))
    }

    private fun getJson(url: String) = Json.parseToJsonElement(getBytes(url).decodeToString()).jsonObject
    private fun getBytes(url: String): ByteArray = client
        .newCall(Request.Builder().url(url).build())
        .execute()
        .use { response ->
            if (!response.isSuccessful) throw IOException("HTTP ${response.code}")
            response.body?.bytes() ?: throw IOException("empty response")
        }
    private fun sha256(bytes: ByteArray) = MessageDigest.getInstance("SHA-256").digest(bytes).joinToString("") { "%02x".format(it) }
    private fun verifySignature(contentHash: String, encodedSignature: String): Boolean {
        val publicKeyBytes = Base64.decode(
            BuildConfig.SYNC_SIGNING_PUBLIC_KEY
                .replace("-----BEGIN PUBLIC KEY-----", "")
                .replace("-----END PUBLIC KEY-----", "")
                .replace("\\s".toRegex(), ""),
            Base64.DEFAULT,
        )
        val publicKey = KeyFactory.getInstance("EC").generatePublic(X509EncodedKeySpec(publicKeyBytes))
        return Signature.getInstance("SHA256withECDSA").run {
            initVerify(publicKey)
            update(contentHash.toByteArray(StandardCharsets.UTF_8))
            verify(Base64.decode(encodedSignature, Base64.DEFAULT))
        }
    }
    private fun JsonObject.required(name: String) = this[name]?.jsonPrimitive?.content ?: error("missing $name")
}
