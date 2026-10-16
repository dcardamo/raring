package ca.hld.pepometer

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent

/** Debug-only emulator probe. No real Garmin credentials or network requests. */
class DebugSyncReceiver: BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        if (intent.getBooleanExtra("seedSynthetic", false)) TokenVault.save(context, "{\"synthetic\":true}")
        SyncScheduler.testRun(context)
    }
}
