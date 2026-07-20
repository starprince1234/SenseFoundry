package com.sensefoundry.ui.submit

import androidx.compose.foundation.layout.Column
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.RequestBody.Companion.toRequestBody
import java.util.UUID

@Composable
fun SubmitScreen(apiUrl: String) {
    var text by remember { mutableStateOf("") }; var status by remember { mutableStateOf("") }; var sending by remember { mutableStateOf(false) }
    val scope = rememberCoroutineScope()
    Column { OutlinedTextField(value = text, onValueChange = { text = it }, label = { Text("Contribution") }, modifier = Modifier.semantics { contentDescription = "Contribution text field" })
        Button(enabled = text.isNotBlank() && !sending, onClick = { scope.launch { sending = true; status = runCatching { postSubmission(apiUrl, text) }.getOrElse { "Submission failed" }; sending = false } }, modifier = Modifier.semantics { contentDescription = "Submit contribution" }) { Text(if (sending) "Submitting" else "Submit") }
        Text(status, modifier = Modifier.semantics { contentDescription = "Submission status: $status" })
    }
}

private suspend fun postSubmission(apiUrl: String, text: String): String = withContext(Dispatchers.IO) {
    val escaped = text.replace("\\", "\\\\").replace("\"", "\\\"")
    val request = Request.Builder()
        .url("$apiUrl/submissions")
        .header("Idempotency-Key", UUID.randomUUID().toString())
        .header("x-submitter-id", UUID.randomUUID().toString())
        .post("{\"text\":\"$escaped\",\"submission_type\":\"text\"}".toRequestBody("application/json".toMediaType()))
        .build()
    OkHttpClient().newCall(request).execute().use { response -> if (!response.isSuccessful) error("HTTP ${response.code}"); "Submission received" }
}
