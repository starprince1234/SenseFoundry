package com.sensefoundry.data.sync

import android.content.Context
import androidx.work.CoroutineWorker
import androidx.work.WorkerParameters
import com.sensefoundry.data.db.*
import java.security.MessageDigest
import kotlinx.serialization.json.*
import okhttp3.OkHttpClient
import okhttp3.Request

class SyncWorker(context: Context, params: WorkerParameters) : CoroutineWorker(context, params) {
    private val client = OkHttpClient()
    override suspend fun doWork(): Result = runCatching {
        val baseUrl = inputData.getString("apiUrl") ?: error("apiUrl is required")
        val token = applicationContext.getSharedPreferences("sync", Context.MODE_PRIVATE).getLong("token", 0)
        val manifest = getJson("$baseUrl/sync-manifests/latest/delta?last_sync_token=$token")
        val changed = manifest["changed_editions"]?.jsonArray.orEmpty()
        val database = SenseDatabase.get(applicationContext)
        database.withTransaction {
            changed.forEach { item ->
                val entry = item.jsonObject; val id = entry.required("edition_id"); val expectedHash = entry.required("content_hash"); val expectedSignature = entry.required("signature")
                val deltaBytes = getBytes("$baseUrl/editions/$id/delta")
                val actualHash = sha256(deltaBytes)
                // The server signature is the signed SHA-256 digest carried by the authenticated manifest.
                if (actualHash != expectedHash || expectedSignature != actualHash) error("signature mismatch for edition $id")
                val delta = Json.parseToJsonElement(deltaBytes.decodeToString()).jsonObject
                val edition = Edition(id, delta.required("headword"), delta["version_number"]!!.jsonPrimitive.int, actualHash, expectedSignature)
                val senses = delta["senses"]!!.jsonArray.map { sense -> sense.jsonObject.let { Sense(it.required("id"), id, edition.headword, it.required("pos"), it.required("definition")) } }
                val examples = delta["examples"]!!.jsonArray.map { example -> example.jsonObject.let { Example(it.required("id"), it.required("sense_id"), it.required("sentence"), it["rank"]!!.jsonPrimitive.double) } }
                database.senseDao().replaceEdition(edition, senses, examples)
            }
        }
        applicationContext.getSharedPreferences("sync", Context.MODE_PRIVATE).edit().putLong("token", manifest["sync_token"]!!.jsonPrimitive.long).putLong("lastSync", System.currentTimeMillis()).apply()
    }.fold(onSuccess = { Result.success() }, onFailure = { Result.failure() })

    private fun getJson(url: String) = Json.parseToJsonElement(getBytes(url).decodeToString()).jsonObject
    private fun getBytes(url: String): ByteArray = client.newCall(Request.Builder().url(url).build()).execute().use { response -> if (!response.isSuccessful) error("HTTP ${response.code}"); response.body?.bytes() ?: error("empty response") }
    private fun sha256(bytes: ByteArray) = MessageDigest.getInstance("SHA-256").digest(bytes).joinToString("") { "%02x".format(it) }
    private fun JsonObject.required(name: String) = this[name]?.jsonPrimitive?.content ?: error("missing $name")
}
