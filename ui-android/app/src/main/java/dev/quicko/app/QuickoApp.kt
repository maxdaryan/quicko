package dev.quicko.app

import android.app.Application
import dev.quicko.core.QuickoClient

class QuickoApp : Application() {
    // Global client instance
    var client: QuickoClient? = null

    override fun onCreate() {
        super.onCreate()
    }
}
