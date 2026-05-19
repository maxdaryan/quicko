package dev.quicko.app.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dev.quicko.core.*
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

class QuickoViewModel : ViewModel() {
    private val _events = MutableStateFlow<QuickoTransportEvent?>(null)
    val events = _events.asStateFlow()

    private val _isConnected = MutableStateFlow(false)
    val isConnected = _isConnected.asStateFlow()

    private val _sessionInfo = MutableStateFlow<QuickoSessionInfo?>(null)
    val sessionInfo = _sessionInfo.asStateFlow()

    private val _keyInfo = MutableStateFlow<QuickoKeyInfo?>(null)
    val keyInfo = _keyInfo.asStateFlow()

    private var client: QuickoClient? = null

    fun initClient(serverUrl: String) {
        if (client != null) return
        viewModelScope.launch(Dispatchers.IO) {
            try {
                client = QuickoClient(serverUrl, 3600u, 1000u)
                startPolling()
            } catch (e: Exception) {
                // Handle error
            }
        }
    }

    fun connect() {
        viewModelScope.launch(Dispatchers.IO) {
            try {
                client?.connect()
            } catch (e: Exception) {
                // Handle error
            }
        }
    }

    fun disconnect() {
        client?.disconnect()
        _isConnected.value = false
    }

    fun createSession() {
        viewModelScope.launch(Dispatchers.IO) {
            _sessionInfo.value = client?.createSession()
        }
    }

    fun generateKey() {
        viewModelScope.launch(Dispatchers.IO) {
            try {
                _keyInfo.value = client?.generateQuickokey()
            } catch (e: Exception) {
                // Handle error
            }
        }
    }

    private fun startPolling() {
        viewModelScope.launch(Dispatchers.IO) {
            while (true) {
                try {
                    val event = client?.pollEvent()
                    if (event != null) {
                        _events.value = event
                        when (event.eventType) {
                            "Connected" -> _isConnected.value = true
                            "Disconnected" -> _isConnected.value = false
                        }
                    }
                } catch (e: Exception) {
                    // Poll failed
                }
                delay(100) // Poll every 100ms
            }
        }
    }
}
