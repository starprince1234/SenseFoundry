package com.sensefoundry.ui.detail

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable

data class SenseDetail(val character: String, val partOfSpeech: String, val definition: String, val examples: List<String>)

@Composable
fun SenseDetailScreen(sense: SenseDetail) {
    Column { Text(sense.character); Text(sense.partOfSpeech); Text(sense.definition); Text("Examples")
        LazyColumn { items(sense.examples) { example -> Text(example) } }
    }
}
