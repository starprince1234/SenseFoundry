package com.sensefoundry

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.*
import androidx.compose.ui.platform.LocalContext
import com.sensefoundry.data.db.SenseDatabase
import com.sensefoundry.ui.detail.SenseDetail
import com.sensefoundry.ui.detail.SenseDetailScreen
import com.sensefoundry.ui.search.SearchScreen
import com.sensefoundry.ui.search.SearchResult
import com.sensefoundry.ui.settings.SettingsScreen
import com.sensefoundry.ui.submit.SubmitScreen
import kotlinx.coroutines.launch

private enum class Destination(val label: String) {
    Search("Search"),
    Submit("Submit"),
    Settings("Settings"),
}

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent { MaterialTheme { SenseFoundryApp() } }
    }
}

@Composable
private fun SenseFoundryApp() {
    val context = LocalContext.current
    val database = remember { SenseDatabase.get(context) }
    val scope = rememberCoroutineScope()
    var destination by remember { mutableStateOf(Destination.Search) }
    var selectedSense by remember { mutableStateOf<SenseDetail?>(null) }

    Scaffold(bottomBar = {
        NavigationBar {
            Destination.entries.forEach { item ->
                NavigationBarItem(
                    selected = destination == item,
                    onClick = { destination = item; selectedSense = null },
                    icon = { Text(item.label.take(1)) },
                    label = { Text(item.label) },
                )
            }
        }
    }) { _ ->
        when {
            selectedSense != null -> SenseDetailScreen(selectedSense!!)
            destination == Destination.Search -> SearchScreen(
                onSearch = { query ->
                    database.senseDao().search(query).map {
                        SearchResult(it.id, it.headword, it.definition)
                    }
                },
                onSelect = { senseId ->
                    scope.launch {
                        val sense = database.senseDao().getSense(senseId) ?: return@launch
                        val examples = database.senseDao().getExamples(senseId).map { it.sentence }
                        selectedSense = SenseDetail(
                            sense.headword,
                            sense.partOfSpeech,
                            sense.definition,
                            examples,
                        )
                    }
                },
            )
            destination == Destination.Submit -> SubmitScreen(BuildConfig.API_BASE_URL)
            else -> SettingsScreen(context, BuildConfig.API_BASE_URL)
        }
    }
}
