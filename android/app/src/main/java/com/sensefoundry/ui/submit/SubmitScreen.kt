package com.sensefoundry.ui.submit

import android.content.Context
import android.content.SharedPreferences
import androidx.core.content.edit
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
import androidx.compose.material.icons.rounded.CheckCircle
import androidx.compose.material.icons.rounded.CloudUpload
import androidx.compose.material.icons.rounded.Description
import androidx.compose.material.icons.rounded.Link
import androidx.compose.material.icons.rounded.Lock
import androidx.compose.material.icons.rounded.Refresh
import androidx.compose.material3.Button
import androidx.compose.material3.Checkbox
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.FilterChip
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.TextButton
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.dp
import com.sensefoundry.ui.components.ScreenHeader
import com.sensefoundry.ui.components.SectionCard
import com.sensefoundry.ui.components.StatusPill
import java.util.UUID
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.RequestBody.Companion.toRequestBody

private enum class ContributionType(val value: String, val label: String) {
    Text("text", "粘贴文本"),
    Url("url", "来源链接"),
}

private data class SubmissionReceipt(
    val id: String,
    val status: String,
    val createdAt: String,
)

@Composable
fun SubmitScreen(
    apiUrl: String,
    modifier: Modifier = Modifier,
) {
    val context = LocalContext.current
    val preferences = remember {
        context.getSharedPreferences("contribution", Context.MODE_PRIVATE)
    }
    var type by remember { mutableStateOf(ContributionType.Text) }
    var text by remember { mutableStateOf(preferences.getString("draft_text", "").orEmpty()) }
    var sourceUrl by remember {
        mutableStateOf(preferences.getString("draft_source_url", "").orEmpty())
    }
    var rightsConfirmed by remember { mutableStateOf(false) }
    var sending by remember { mutableStateOf(false) }
    var receipt by remember { mutableStateOf(savedReceipt(preferences)) }
    var error by remember { mutableStateOf<String?>(null) }
    val scope = rememberCoroutineScope()
    val canSubmit = when (type) {
        ContributionType.Text -> text.isNotBlank()
        ContributionType.Url -> sourceUrl.isNotBlank()
    } && rightsConfirmed && !sending

    Column(
        modifier = modifier
            .fillMaxSize()
            .verticalScroll(rememberScrollState())
            .padding(horizontal = 20.dp, vertical = 24.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp),
    ) {
        ScreenHeader(
            eyebrow = "语料共建",
            title = "提交一条可核验的真实用法",
            description = "投稿只会进入待核验队列，不会直接成为正式辞书内容。语料管理员会继续检查来源、版权与质量。",
        )

        Row(horizontalArrangement = Arrangement.spacedBy(10.dp)) {
            ContributionType.entries.forEach { item ->
                FilterChip(
                    selected = type == item,
                    onClick = {
                        type = item
                        receipt = null
                        error = null
                    },
                    label = { Text(item.label) },
                    leadingIcon = {
                        Icon(
                            if (item == ContributionType.Text) {
                                Icons.Rounded.Description
                            } else {
                                Icons.Rounded.Link
                            },
                            contentDescription = null,
                        )
                    },
                )
            }
        }

        SectionCard {
            if (type == ContributionType.Text) {
                Text("语料原文", style = MaterialTheme.typography.titleMedium)
                OutlinedTextField(
                    value = text,
                    onValueChange = {
                        text = it.take(50_000)
                        preferences.edit { putString("draft_text", text) }
                    },
                    modifier = Modifier
                        .fillMaxWidth()
                        .semantics { contentDescription = "语料原文输入框" },
                    placeholder = { Text("粘贴包含目标字词的完整句子或短段落") },
                    supportingText = { Text("${text.length} / 50,000 字符 · 草稿仅保存在本机") },
                    minLines = 6,
                    maxLines = 12,
                )
            } else {
                Text("公开来源链接", style = MaterialTheme.typography.titleMedium)
                OutlinedTextField(
                    value = sourceUrl,
                    onValueChange = {
                        sourceUrl = it.trim().take(2_048)
                        preferences.edit { putString("draft_source_url", sourceUrl) }
                    },
                    modifier = Modifier
                        .fillMaxWidth()
                        .semantics { contentDescription = "来源链接输入框" },
                    placeholder = { Text("https://example.org/source") },
                    supportingText = { Text("请提供可由审核人员访问和核验的来源") },
                    singleLine = true,
                )
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
                Text("权利与隐私确认", style = MaterialTheme.typography.titleMedium)
            }
            Row(verticalAlignment = Alignment.Top) {
                Checkbox(
                    checked = rightsConfirmed,
                    onCheckedChange = { rightsConfirmed = it },
                    modifier = Modifier.semantics {
                        contentDescription = "确认有权提交且内容不包含不必要的个人信息"
                    },
                )
                Text(
                    text = "我确认有权提交此内容，并已移除与语言研究无关的个人信息。投稿仍需通过来源、版权与内容审核。",
                    modifier = Modifier.padding(top = 10.dp),
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }

        error?.let { message ->
            SectionCard {
                StatusPill(
                    text = "提交失败",
                    containerColor = MaterialTheme.colorScheme.errorContainer,
                    contentColor = MaterialTheme.colorScheme.onErrorContainer,
                )
                Text(message, style = MaterialTheme.typography.bodyMedium)
                Text(
                    "草稿仍保存在本机，可检查网络后再次提交。",
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }

        receipt?.let { submission ->
            SectionCard {
                Row(
                    horizontalArrangement = Arrangement.spacedBy(10.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Icon(
                        Icons.Rounded.CheckCircle,
                        contentDescription = null,
                        tint = MaterialTheme.colorScheme.primary,
                    )
                    Text("已收到投稿", style = MaterialTheme.typography.titleMedium)
                }
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    StatusPill(text = statusLabel(submission.status))
                    StatusPill(text = "编号 ${submission.id.take(8)}")
                }
                Text(
                    "创建时间：${submission.createdAt.replace('T', ' ').take(19)}",
                    style = MaterialTheme.typography.bodyMedium,
                )
                Text(
                    "待核验不代表已收录；正式发布仍需专家审核、双人复核和签名发布。",
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    style = MaterialTheme.typography.bodyMedium,
                )
                TextButton(
                    enabled = !sending,
                    onClick = {
                        scope.launch {
                            sending = true
                            error = null
                            runCatching { getSubmission(apiUrl, submission.id) }
                                .onSuccess {
                                    receipt = it
                                    saveReceipt(preferences, it)
                                }
                                .onFailure {
                                    error = "无法刷新投稿状态，请检查网络后重试。"
                                }
                            sending = false
                        }
                    },
                    modifier = Modifier.semantics { contentDescription = "刷新最近投稿状态" },
                ) {
                    Icon(Icons.Rounded.Refresh, contentDescription = null)
                    Text("  刷新状态")
                }
            }
        }

        Button(
            enabled = canSubmit,
            onClick = {
                scope.launch {
                    sending = true
                    error = null
                    receipt = runCatching {
                        postSubmission(
                            context = context,
                            apiUrl = apiUrl,
                            type = type,
                            text = text,
                            sourceUrl = sourceUrl,
                        )
                    }.onSuccess {
                        saveReceipt(preferences, it)
                        if (type == ContributionType.Text) {
                            text = ""
                            preferences.edit { remove("draft_text") }
                        } else {
                            sourceUrl = ""
                            preferences.edit { remove("draft_source_url") }
                        }
                        rightsConfirmed = false
                    }.onFailure {
                        error = it.message ?: "网络请求失败。"
                    }.getOrNull()
                    sending = false
                }
            },
            modifier = Modifier
                .fillMaxWidth()
                .semantics { contentDescription = "提交语料到待核验队列" },
            contentPadding = PaddingValues(vertical = 14.dp),
        ) {
            if (sending) {
                CircularProgressIndicator(
                    modifier = Modifier
                        .padding(end = 12.dp)
                        .size(20.dp),
                    strokeWidth = 2.dp,
                )
                Text("正在安全提交")
            } else {
                Icon(Icons.Rounded.CloudUpload, contentDescription = null)
                Text("  提交审核")
            }
        }
    }
}

private fun savedReceipt(preferences: SharedPreferences): SubmissionReceipt? {
    val id = preferences.getString("last_submission_id", null) ?: return null
    return SubmissionReceipt(
        id = id,
        status = preferences.getString("last_submission_status", "pending").orEmpty(),
        createdAt = preferences.getString("last_submission_created_at", "").orEmpty(),
    )
}

private fun saveReceipt(
    preferences: SharedPreferences,
    receipt: SubmissionReceipt,
) {
    preferences.edit {
        putString("last_submission_id", receipt.id)
        putString("last_submission_status", receipt.status)
        putString("last_submission_created_at", receipt.createdAt)
    }
}

private suspend fun postSubmission(
    context: Context,
    apiUrl: String,
    type: ContributionType,
    text: String,
    sourceUrl: String,
): SubmissionReceipt = withContext(Dispatchers.IO) {
    val preferences = context.getSharedPreferences("contribution", Context.MODE_PRIVATE)
    val submitterId = preferences.getString("submitter_id", null)
        ?: UUID.randomUUID().toString().also {
            preferences.edit { putString("submitter_id", it) }
        }
    val body = if (type == ContributionType.Text) {
        """{"text":${Json.encodeToString(text)},"submission_type":"text"}"""
    } else {
        """{"source_url":${Json.encodeToString(sourceUrl)},"submission_type":"url"}"""
    }
    val request = Request.Builder()
        .url("$apiUrl/submissions")
        .header("Idempotency-Key", UUID.randomUUID().toString())
        .header("x-submitter-id", submitterId)
        .post(body.toRequestBody("application/json".toMediaType()))
        .build()

    OkHttpClient().newCall(request).execute().use { response ->
        val responseBody = response.body?.string().orEmpty()
        if (!response.isSuccessful) {
            error(if (response.code == 422) "内容格式或安全检查未通过。" else "服务器返回 HTTP ${response.code}。")
        }
        val json = Json.parseToJsonElement(responseBody).jsonObject
        SubmissionReceipt(
            id = json.getValue("id").jsonPrimitive.content,
            status = json.getValue("status").jsonPrimitive.content,
            createdAt = json.getValue("created_at").jsonPrimitive.content,
        )
    }
}

private suspend fun getSubmission(
    apiUrl: String,
    submissionId: String,
): SubmissionReceipt = withContext(Dispatchers.IO) {
    val request = Request.Builder()
        .url("$apiUrl/submissions/$submissionId")
        .get()
        .build()
    OkHttpClient().newCall(request).execute().use { response ->
        val responseBody = response.body?.string().orEmpty()
        if (!response.isSuccessful) error("服务器返回 HTTP ${response.code}。")
        val json = Json.parseToJsonElement(responseBody).jsonObject
        SubmissionReceipt(
            id = json.getValue("id").jsonPrimitive.content,
            status = json.getValue("status").jsonPrimitive.content,
            createdAt = json.getValue("created_at").jsonPrimitive.content,
        )
    }
}

private fun statusLabel(status: String): String = when (status.lowercase()) {
    "pending" -> "待核验"
    "processing" -> "处理中"
    "accepted" -> "已接受"
    "rejected" -> "未通过"
    else -> status
}
