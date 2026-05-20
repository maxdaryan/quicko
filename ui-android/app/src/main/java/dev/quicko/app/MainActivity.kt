package dev.quicko.app

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.viewmodel.compose.viewModel
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.unit.dp
import dev.quicko.app.viewmodel.QuickoViewModel

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            MaterialTheme {
                Surface(
                    modifier = Modifier.fillMaxSize(),
                    color = MaterialTheme.colorScheme.background
                ) {
                    QuickoAppScreen()
                }
            }
        }
    }
}

@Composable
fun QuickoAppScreen(viewModel: QuickoViewModel = viewModel()) {
    val isConnected by viewModel.isConnected.collectAsState()
    val events by viewModel.events.collectAsState()
    val sessionInfo by viewModel.sessionInfo.collectAsState()
    val keyInfo by viewModel.keyInfo.collectAsState()

    LaunchedEffect(Unit) {
        viewModel.initClient("ws://10.0.2.2:9900")
    }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(16.dp)
            .verticalScroll(rememberScrollState()),
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        Text(
            text = """
                /\_____/\
               /  o   o  \
              ( ==  ^  == )
               )         (
              (           )
             ( (  )   (  ) )
            (__(__)___(__)__)
            """.trimIndent(),
            style = MaterialTheme.typography.bodySmall.copy(
                fontFamily = FontFamily.Monospace,
                color = MaterialTheme.colorScheme.secondary
            )
        )
        
        Spacer(modifier = Modifier.height(8.dp))

        Text(
            text = "Quicko2 Android",
            style = MaterialTheme.typography.headlineMedium
        )
        
        Spacer(modifier = Modifier.height(16.dp))
        
        StatusCard(isConnected, events?.eventType)
        
        Spacer(modifier = Modifier.height(16.dp))
        
        ActionButtons(viewModel, isConnected)
        
        Spacer(modifier = Modifier.height(16.dp))
        
        sessionInfo?.let {
            InfoCard(title = "Session Info", items = listOf(
                "ID: ${it.sessionId}",
                "Name: ${it.displayName}",
                "Invite: ${it.inviteCode}"
            ))
        }
        
        Spacer(modifier = Modifier.height(16.dp))
        
        keyInfo?.let {
            InfoCard(title = "QuickoKey", items = listOf(
                "Key: ${it.formattedKey}",
                "Display: ${it.displayName}",
                "Seed: ${it.seedPhrase.joinToString(" ")}"
            ))
        }
    }
}

@Composable
fun StatusCard(isConnected: Boolean, lastEvent: String?) {
    Card(
        modifier = Modifier.fillMaxWidth(),
        colors = CardDefaults.cardColors(
            containerColor = if (isConnected) 
                MaterialTheme.colorScheme.primaryContainer 
            else 
                MaterialTheme.colorScheme.errorContainer
        )
    ) {
        Column(modifier = Modifier.padding(16.dp)) {
            Text(
                text = if (isConnected) "Connected" else "Disconnected",
                style = MaterialTheme.typography.titleMedium
            )
            lastEvent?.let {
                Text(text = "Last Event: $it", style = MaterialTheme.typography.bodySmall)
            }
        }
    }
}

@Composable
fun ActionButtons(viewModel: QuickoViewModel, isConnected: Boolean) {
    Column(horizontalAlignment = Alignment.CenterHorizontally) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceEvenly
        ) {
            Button(onClick = { viewModel.connect() }, enabled = !isConnected) {
                Text("Connect")
            }
            Button(onClick = { viewModel.disconnect() }, enabled = isConnected) {
                Text("Disconnect")
            }
        }
        
        Spacer(modifier = Modifier.height(8.dp))
        
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceEvenly
        ) {
            Button(onClick = { viewModel.generateKey() }) {
                Text("Gen Key")
            }
            Button(onClick = { viewModel.createSession() }) {
                Text("New Session")
            }
        }
    }
}

@Composable
fun InfoCard(title: String, items: List<String>) {
    Card(
        modifier = Modifier.fillMaxWidth(),
        elevation = CardDefaults.cardElevation(defaultElevation = 2.dp)
    ) {
        Column(modifier = Modifier.padding(16.dp)) {
            Text(text = title, style = MaterialTheme.typography.titleSmall)
            Divider(modifier = Modifier.padding(vertical = 8.dp))
            items.forEach { item ->
                Text(text = item, style = MaterialTheme.typography.bodyMedium)
            }
        }
    }
}
