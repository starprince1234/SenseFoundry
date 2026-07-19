package com.sensefoundry.ui.search

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics

data class SearchResult(val senseId: String, val character: String, val glossPreview: String)

@Composable
fun SearchScreen(onSearch: suspend (String) -> List<SearchResult>, onSelect: (String) -> Unit) {
    var query by remember { mutableStateOf("") }; var results by remember { mutableStateOf(emptyList<SearchResult>()) }
    LaunchedEffect(query) { results = if (query.isBlank()) emptyList() else onSearch(query) }
    Column { OutlinedTextField(value = query, onValueChange = { query = it }, label = { Text("Search") }, modifier = Modifier.semantics { contentDescription = "Dictionary search field" })
        LazyColumn { items(results, key = { it.senseId }) { result -> Column(Modifier.clickable { onSelect(result.senseId) }.semantics { contentDescription = "Open ${result.character}" }) { Text(result.character); Text(result.glossPreview) } } }
    }
}
