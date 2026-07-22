package com.sensefoundry.ui.detail

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.ArrowBack
import androidx.compose.material.icons.rounded.EditNote
import androidx.compose.material.icons.rounded.FormatQuote
import androidx.compose.material.icons.rounded.VerifiedUser
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.FilledTonalIconButton
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import com.sensefoundry.ui.components.SectionCard
import com.sensefoundry.ui.components.StatusPill

data class SenseDetail(
    val senseId: String,
    val editionId: String,
    val headword: String,
    val partOfSpeech: String,
    val definition: String,
    val examples: List<String>,
)

@Composable
fun SenseDetailScreen(
    sense: SenseDetail,
    onBack: () -> Unit,
    onContribute: () -> Unit,
    modifier: Modifier = Modifier,
) {
    LazyColumn(
        modifier = modifier.fillMaxSize(),
        contentPadding = PaddingValues(horizontal = 20.dp, vertical = 20.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp),
    ) {
        item {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                FilledTonalIconButton(onClick = onBack) {
                    Icon(Icons.AutoMirrored.Rounded.ArrowBack, contentDescription = "返回辞典")
                }
                StatusPill(text = "离线可用")
            }
        }
        item {
            Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
                Text(
                    text = sense.headword,
                    style = MaterialTheme.typography.displaySmall,
                    color = MaterialTheme.colorScheme.primary,
                )
                Row(
                    horizontalArrangement = Arrangement.spacedBy(10.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    StatusPill(text = sense.partOfSpeech)
                    Text(
                        text = "正式发布义项",
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            }
        }
        item {
            SectionCard {
                Text("释义", style = MaterialTheme.typography.titleMedium)
                Text(
                    text = sense.definition,
                    style = MaterialTheme.typography.headlineMedium,
                    fontWeight = FontWeight.Normal,
                )
            }
        }
        item {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text("真实语料例证", style = MaterialTheme.typography.titleLarge)
                StatusPill(text = "${sense.examples.size} 条")
            }
        }
        if (sense.examples.isEmpty()) {
            item {
                SectionCard {
                    Text("当前离线版本尚未提供可公开例证。")
                }
            }
        } else {
            itemsIndexed(sense.examples) { index, example ->
                Card(
                    modifier = Modifier.fillMaxWidth(),
                    colors = CardDefaults.cardColors(
                        containerColor = MaterialTheme.colorScheme.surface,
                    ),
                ) {
                    Row(
                        modifier = Modifier.padding(18.dp),
                        horizontalArrangement = Arrangement.spacedBy(14.dp),
                    ) {
                        Icon(
                            Icons.Rounded.FormatQuote,
                            contentDescription = null,
                            tint = MaterialTheme.colorScheme.secondary,
                        )
                        Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                            Text(example, style = MaterialTheme.typography.bodyLarge)
                            Text(
                                "例证 ${index + 1}",
                                style = MaterialTheme.typography.labelLarge,
                                color = MaterialTheme.colorScheme.onSurfaceVariant,
                            )
                        }
                    }
                }
            }
        }
        item {
            SectionCard {
                Row(
                    horizontalArrangement = Arrangement.spacedBy(10.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Icon(
                        Icons.Rounded.VerifiedUser,
                        contentDescription = null,
                        tint = MaterialTheme.colorScheme.primary,
                    )
                    Text("版本与来源", style = MaterialTheme.typography.titleMedium)
                }
                Text(
                    "版本 ${sense.editionId.take(8)} · 内容已通过同步包摘要与 P-256 签名校验。",
                    style = MaterialTheme.typography.bodyMedium,
                )
                Text(
                    "当前离线包未下发来源书目信息；应用不会推测或补写来源。完整血缘应在具备权限的编纂端查看。",
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    style = MaterialTheme.typography.bodyMedium,
                )
            }
        }
        item {
            Button(
                onClick = onContribute,
                modifier = Modifier
                    .fillMaxWidth()
                    .semantics { contentDescription = "为${sense.headword}补充真实语料" },
                contentPadding = PaddingValues(vertical = 14.dp),
            ) {
                Icon(Icons.Rounded.EditNote, contentDescription = null)
                Text("  补充真实语料")
            }
        }
    }
}
