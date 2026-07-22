package com.sensefoundry.ui.settings

import android.content.Context
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.CloudDone
import androidx.compose.material.icons.rounded.CloudSync
import androidx.compose.material.icons.rounded.Lock
import androidx.compose.material.icons.rounded.Storage
import androidx.compose.material3.Button
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.livedata.observeAsState
import androidx.compose.runtime.mutableLongStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.dp
import androidx.work.BackoffPolicy
import androidx.work.ExistingWorkPolicy
import androidx.work.OneTimeWorkRequestBuilder
import androidx.work.WorkInfo
import androidx.work.WorkManager
import androidx.work.workDataOf
import com.sensefoundry.data.db.OfflineStats
import com.sensefoundry.data.sync.SyncWorker
import com.sensefoundry.ui.components.Metric
import com.sensefoundry.ui.components.MetricRow
import com.sensefoundry.ui.components.ScreenHeader
import com.sensefoundry.ui.components.SectionCard
import com.sensefoundry.ui.components.StatusPill
import java.text.DateFormat
import java.util.Date
import java.util.concurrent.TimeUnit
import kotlinx.coroutines.launch

@Composable
fun LibraryScreen(
    context: Context,
    apiUrl: String,
    loadStats: suspend () -> OfflineStats,
    modifier: Modifier = Modifier,
) {
    val workManager = remember { WorkManager.getInstance(context) }
    val workInfos by workManager
        .getWorkInfosForUniqueWorkLiveData("manual-sync")
        .observeAsState(initial = emptyList())
    val latestWork = workInfos.firstOrNull()
    val preferences = remember {
        context.getSharedPreferences("sync", Context.MODE_PRIVATE)
    }
    var lastSync by remember { mutableLongStateOf(preferences.getLong("lastSync", 0)) }
    var syncToken by remember { mutableLongStateOf(preferences.getLong("token", 0)) }
    var stats by remember { mutableStateOf(OfflineStats(0, 0, 0)) }
    var statsError by remember { mutableStateOf(false) }
    val scope = rememberCoroutineScope()

    fun refreshLocalState() {
        scope.launch {
            lastSync = preferences.getLong("lastSync", 0)
            syncToken = preferences.getLong("token", 0)
            runCatching { loadStats() }
                .onSuccess {
                    stats = it
                    statsError = false
                }
                .onFailure { statsError = true }
        }
    }

    LaunchedEffect(latestWork?.state) {
        refreshLocalState()
    }

    Column(
        modifier = modifier
            .fillMaxSize()
            .verticalScroll(rememberScrollState())
            .padding(horizontal = 20.dp, vertical = 24.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp),
    ) {
        ScreenHeader(
            eyebrow = "离线书库",
            title = "可信版本，随身可查",
            description = "下载正式发布的增量包。应用会先校验 SHA-256 摘要和 P-256 签名，再用 Room 事务导入。",
        )

        SectionCard {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Row(
                    horizontalArrangement = Arrangement.spacedBy(10.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Icon(
                        Icons.Rounded.Storage,
                        contentDescription = null,
                        tint = MaterialTheme.colorScheme.primary,
                    )
                    Text("本地内容", style = MaterialTheme.typography.titleLarge)
                }
                StatusPill(text = if (statsError) "读取失败" else "同步令牌 $syncToken")
            }
            MetricRow {
                Metric("版本", stats.editionCount.toString())
                Metric("义项", stats.senseCount.toString())
                Metric("例证", stats.exampleCount.toString())
            }
        }

        SectionCard {
            Row(
                horizontalArrangement = Arrangement.spacedBy(10.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Icon(
                    if (latestWork?.state == WorkInfo.State.SUCCEEDED) {
                        Icons.Rounded.CloudDone
                    } else {
                        Icons.Rounded.CloudSync
                    },
                    contentDescription = null,
                    tint = MaterialTheme.colorScheme.primary,
                )
                Text("同步状态", style = MaterialTheme.typography.titleLarge)
            }
            StatusPill(text = workStateLabel(latestWork?.state))
            Text(
                text = if (lastSync == 0L) {
                    "尚未成功同步"
                } else {
                    "上次成功：${DateFormat.getDateTimeInstance().format(Date(lastSync))}"
                },
                style = MaterialTheme.typography.bodyLarge,
            )
            latestWork?.outputData?.getString("reason")?.let { reason ->
                Text(
                    text = when (reason) {
                        "integrity" -> "同步包完整性验证失败，已拒绝导入。"
                        "version_regression" -> "服务端版本低于本地可信版本，已拒绝回退。"
                        else -> "同步未完成，请检查服务状态后重试。"
                    },
                    color = MaterialTheme.colorScheme.error,
                    style = MaterialTheme.typography.bodyMedium,
                )
            }
            Button(
                onClick = {
                    val request = OneTimeWorkRequestBuilder<SyncWorker>()
                        .setInputData(workDataOf("apiUrl" to apiUrl))
                        .setBackoffCriteria(
                            BackoffPolicy.EXPONENTIAL,
                            10,
                            TimeUnit.SECONDS,
                        )
                        .build()
                    workManager.enqueueUniqueWork(
                        "manual-sync",
                        ExistingWorkPolicy.REPLACE,
                        request,
                    )
                },
                enabled = latestWork?.state !in setOf(
                    WorkInfo.State.ENQUEUED,
                    WorkInfo.State.RUNNING,
                    WorkInfo.State.BLOCKED,
                ),
                modifier = Modifier
                    .fillMaxWidth()
                    .semantics { contentDescription = "同步最新签名辞典版本" },
                contentPadding = PaddingValues(vertical = 14.dp),
            ) {
                if (latestWork?.state == WorkInfo.State.RUNNING) {
                    CircularProgressIndicator(
                        modifier = Modifier.size(20.dp),
                        strokeWidth = 2.dp,
                    )
                    Text("  正在校验与导入")
                } else {
                    Icon(Icons.Rounded.CloudSync, contentDescription = null)
                    Text("  同步最新版本")
                }
            }
        }

        SectionCard {
            Row(
                horizontalArrangement = Arrangement.spacedBy(10.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Icon(
                    Icons.Rounded.Lock,
                    contentDescription = null,
                    tint = MaterialTheme.colorScheme.primary,
                )
                Text("安全与隐私", style = MaterialTheme.typography.titleLarge)
            }
            Text("· 签名或摘要不匹配时拒绝导入，不使用旧数据伪装同步成功。")
            Text("· 网络失败时保留已有离线辞典与投稿草稿，并由 WorkManager 指数退避。")
            Text("· 投稿草稿只保存在本机；提交后进入审核队列，不会自动发布。")
            Text(
                "服务地址：${apiUrl.removeSuffix("/api/v1")}",
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                style = MaterialTheme.typography.bodyMedium,
            )
        }
    }
}

private fun workStateLabel(state: WorkInfo.State?): String = when (state) {
    WorkInfo.State.ENQUEUED -> "等待网络与调度"
    WorkInfo.State.RUNNING -> "正在同步"
    WorkInfo.State.SUCCEEDED -> "同步成功"
    WorkInfo.State.FAILED -> "同步失败"
    WorkInfo.State.BLOCKED -> "等待前置任务"
    WorkInfo.State.CANCELLED -> "已取消"
    null -> "尚未发起"
}
