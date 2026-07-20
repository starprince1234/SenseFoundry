package com.sensefoundry.ui.settings

import android.content.Context
import androidx.compose.foundation.layout.Column
import androidx.compose.material3.Button
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.work.*
import com.sensefoundry.data.sync.SyncWorker
import java.text.DateFormat
import java.util.Date
import java.util.concurrent.TimeUnit

@Composable
fun SettingsScreen(context: Context, apiUrl: String) {
    val workManager = remember { WorkManager.getInstance(context) }; var status by remember { mutableStateOf("Idle") }
    val lastSync = context.getSharedPreferences("sync", Context.MODE_PRIVATE).getLong("lastSync", 0)
    Column { Text("Sync status: $status"); Text("Last sync: ${if (lastSync == 0L) "Never" else DateFormat.getDateTimeInstance().format(Date(lastSync))}")
        Button(onClick = { val request = OneTimeWorkRequestBuilder<SyncWorker>().setInputData(workDataOf("apiUrl" to apiUrl)).setBackoffCriteria(BackoffPolicy.EXPONENTIAL, 10, TimeUnit.SECONDS).build(); workManager.enqueueUniqueWork("manual-sync", ExistingWorkPolicy.REPLACE, request); status = "Queued" }, modifier = Modifier.semantics { contentDescription = "Start manual dictionary synchronization" }) { Text("Sync now") }
    }
}
