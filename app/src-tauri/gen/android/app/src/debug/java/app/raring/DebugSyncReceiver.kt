package app.raring

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.sync.withLock

/** Debug-only emulator probe. No real Garmin credentials or network requests. */
class DebugSyncReceiver: BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        if (intent.action == "app.raring.DEBUG_STATUS") {
            val pending = goAsync()
            Thread { try { StatusSurfaces.refreshOfflineForDebug(context.applicationContext) } finally { pending.finish() } }.start()
            return
        }
        val pending = goAsync()
        CoroutineScope(Dispatchers.IO).launch {
            try {
                SyncGate.mutex.withLock {
                    if (intent.getBooleanExtra("seedSynthetic", false)) TokenVault.save(context, "{\"synthetic\":true}")
                }
                SyncScheduler.testRun(context)
            } finally { pending.finish() }
        }
    }
}
