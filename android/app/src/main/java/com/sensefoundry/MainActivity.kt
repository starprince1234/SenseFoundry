package com.sensefoundry

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.CloudSync
import androidx.compose.material.icons.rounded.EditNote
import androidx.compose.material.icons.rounded.Search
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.platform.LocalContext
import com.sensefoundry.data.db.SenseDatabase
import com.sensefoundry.ui.detail.SenseDetail
import com.sensefoundry.ui.detail.SenseDetailScreen
import com.sensefoundry.ui.search.SearchResult
import com.sensefoundry.ui.search.SearchScreen
import com.sensefoundry.ui.settings.LibraryScreen
import com.sensefoundry.ui.submit.SubmitScreen
import com.sensefoundry.ui.theme.SenseFoundryTheme
import kotlinx.coroutines.launch

private enum class Destination(val label: String, val icon: ImageVector) {
    Dictionary("辞典", Icons.Rounded.Search),
    Contribution("投稿", Icons.Rounded.EditNote),
    Library("书库", Icons.Rounded.CloudSync),
}

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        setContent {
            SenseFoundryTheme {
                SenseFoundryApp()
            }
        }
    }
}

@Composable
private fun SenseFoundryApp() {
    val context = LocalContext.current
    val database = remember { SenseDatabase.get(context) }
    val scope = rememberCoroutineScope()
    var destination by remember { mutableStateOf(Destination.Dictionary) }
    var selectedSense by remember { mutableStateOf<SenseDetail?>(null) }

    Surface(
        modifier = Modifier.fillMaxSize(),
        color = MaterialTheme.colorScheme.background,
    ) {
        Scaffold(
            containerColor = MaterialTheme.colorScheme.background,
            bottomBar = {
                if (selectedSense == null) {
                    NavigationBar(
                        containerColor = MaterialTheme.colorScheme.surface,
                        tonalElevation = androidx.compose.ui.unit.Dp(10f),
                    ) {
                        Destination.entries.forEach { item ->
                            NavigationBarItem(
                                selected = destination == item,
                                onClick = { destination = item },
                                icon = {
                                    Icon(
                                        imageVector = item.icon,
                                        contentDescription = item.label,
                                    )
                                },
                                label = { Text(item.label) },
                            )
                        }
                    }
                }
            },
        ) { innerPadding ->
            val contentModifier = Modifier
                .fillMaxSize()
                .padding(innerPadding)

            when {
                selectedSense != null -> SenseDetailScreen(
                    sense = selectedSense!!,
                    modifier = contentModifier,
                    onBack = { selectedSense = null },
                    onContribute = {
                        selectedSense = null
                        destination = Destination.Contribution
                    },
                )

                destination == Destination.Dictionary -> SearchScreen(
                    modifier = contentModifier,
                    onSearch = { query ->
                        database.senseDao().search(query).map {
                            SearchResult(
                                senseId = it.id,
                                headword = it.headword,
                                partOfSpeech = it.partOfSpeech,
                                definition = it.definition,
                            )
                        }
                    },
                    onSelect = { senseId ->
                        scope.launch {
                            val sense = database.senseDao().getSense(senseId)
                                ?: return@launch
                            val examples = database.senseDao().getExamples(senseId)
                            selectedSense = SenseDetail(
                                senseId = sense.id,
                                editionId = sense.editionId,
                                headword = sense.headword,
                                partOfSpeech = sense.partOfSpeech,
                                definition = sense.definition,
                                examples = examples.map { it.sentence },
                            )
                        }
                    },
                )

                destination == Destination.Contribution -> SubmitScreen(
                    apiUrl = BuildConfig.API_BASE_URL,
                    modifier = contentModifier,
                )

                else -> LibraryScreen(
                    context = context,
                    apiUrl = BuildConfig.API_BASE_URL,
                    modifier = contentModifier,
                    loadStats = { database.senseDao().offlineStats() },
                )
            }
        }
    }
}
