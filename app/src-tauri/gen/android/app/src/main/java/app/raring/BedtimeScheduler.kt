package app.raring

import android.app.AlarmManager
import android.app.PendingIntent
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.util.Log
import org.json.JSONArray
import org.json.JSONObject
import java.io.File
import java.time.LocalDateTime
import java.time.OffsetDateTime
import java.time.ZoneId
import java.time.format.DateTimeFormatter

// The Rust core decides when tonight's wind-down reminder is due, in local
// wall-clock time; this only maps that time to an alarm. A ten-minute window
// needs no exact-alarm permission; Doze can still defer it, and the hourly
// sync posts a due reminder too.
object BedtimeScheduler {
    private const val TAG = "RaringBedtime"
    private const val ACTION = "app.raring.BEDTIME"
    private fun alarm(context: Context, flags: Int): PendingIntent? =
        PendingIntent.getBroadcast(context, 7301, Intent(context, BedtimeReceiver::class.java).setAction(ACTION), flags or PendingIntent.FLAG_IMMUTABLE)

    // Posts a reminder that is due now and schedules the next one, or cancels it.
    fun run(context: Context) {
        try {
            val db = File(context.filesDir, "raring.db").absolutePath
            val now = LocalDateTime.now().format(DateTimeFormatter.ISO_LOCAL_DATE_TIME)
            val result = JSONObject(NativeBridge.bedtime(db, now))
            if (result.has("error") && !result.isNull("error")) {
                Log.w(TAG, "bedtime check failed: ${result.getString("error")}")
                return
            }
            val due = result.optJSONObject("due")
            if (due != null && SyncWorker.notify(context, due.getString("id"), due.getString("title"), due.getString("body"), due.optString("target", "sleep"))) {
                val tags = JSONArray().put(JSONObject().put("kind", due.getString("kind"))
                    .put("tag", due.getString("tag")).put("title", "").put("body", ""))
                val sentAt = OffsetDateTime.now().format(DateTimeFormatter.ISO_OFFSET_DATE_TIME)
                if (JSONObject(NativeBridge.markSent(db, tags.toString(), sentAt)).has("error")) Log.w(TAG, "could not record reminder")
            }
            val manager = context.getSystemService(AlarmManager::class.java)
            val next = if (result.isNull("nextAt")) null else result.getString("nextAt")
            if (next == null) {
                alarm(context, PendingIntent.FLAG_NO_CREATE)?.let { manager.cancel(it) }
                return
            }
            // A wall-clock time inside a daylight-saving gap moves forward, as java.time does.
            val at = LocalDateTime.parse(next).atZone(ZoneId.systemDefault()).toInstant().toEpochMilli()
            alarm(context, PendingIntent.FLAG_UPDATE_CURRENT)?.let { manager.setWindow(AlarmManager.RTC_WAKEUP, at, 10 * 60_000L, it) }
            Log.i(TAG, "wind-down reminder scheduled")
        } catch (error: Exception) {
            Log.e(TAG, "bedtime scheduling failed", error)
        }
    }

    fun cancel(context: Context) {
        alarm(context, PendingIntent.FLAG_NO_CREATE)?.let { context.getSystemService(AlarmManager::class.java).cancel(it) }
    }
}

// Fires at the wind-down time, and when the clock or timezone changes.
class BedtimeReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        val pending = goAsync()
        Thread {
            try { BedtimeScheduler.run(context.applicationContext) } finally { pending.finish() }
        }.start()
    }
}

class BedtimeClockReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        if (intent.action != Intent.ACTION_TIMEZONE_CHANGED && intent.action != Intent.ACTION_TIME_CHANGED) return
        val pending = goAsync()
        Thread {
            try { BedtimeScheduler.run(context.applicationContext) } finally { pending.finish() }
        }.start()
    }
}
