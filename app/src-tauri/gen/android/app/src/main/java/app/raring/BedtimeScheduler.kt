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
import java.time.Instant
import java.time.LocalDateTime
import java.time.OffsetDateTime
import java.time.ZoneId
import java.time.format.DateTimeFormatter

// The Rust core decides when tonight's wind-down reminder is due, in local
// wall-clock time; this only maps that time to an alarm. A ten-minute window
// needs no exact-alarm permission; Doze can still defer it, and every sync
// runs this check too. It is the only place a bedtime reminder is posted.
object BedtimeScheduler {
    private const val TAG = "RaringBedtime"
    private const val ACTION = "app.raring.BEDTIME"
    private fun alarm(context: Context, flags: Int): PendingIntent? =
        PendingIntent.getBroadcast(context, 7301, Intent(context, BedtimeReceiver::class.java).setAction(ACTION), flags or PendingIntent.FLAG_IMMUTABLE)

    // Posts a reminder that is due now and schedules the next one, or cancels it.
    // Synchronized so the alarm, sync and clock paths check, post and record in turn.
    @Synchronized fun run(context: Context) {
        // Account changes and alarm delivery must not cross. An active sync's
        // finally block checks the reminder again, so receivers need not wait.
        if (!SyncGate.mutex.tryLock()) return
        try {
            if (!AccountSession.active(context)) { cancel(context); return }
            runLocked(context)
        } finally { SyncGate.mutex.unlock() }
    }
    private fun runLocked(context: Context) {
        try {
            val db = File(context.filesDir, "raring.db").absolutePath
            val now = LocalDateTime.now().format(DateTimeFormatter.ISO_LOCAL_DATE_TIME)
            val result = JSONObject(NativeBridge.bedtime(db, now))
            if (result.has("error") && !result.isNull("error")) {
                Log.w(TAG, "bedtime check failed: ${result.getString("error")}")
                return
            }
            val due = result.optJSONObject("due")
            if (due != null && SyncWorker.notify(context, "bedtime", due.getString("id"), due.getString("title"), due.getString("body"), due.optString("target", "sleep/tonight"))) {
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
            val at = alarmInstant(LocalDateTime.parse(next), ZoneId.systemDefault(), Instant.now()).toEpochMilli()
            alarm(context, PendingIntent.FLAG_UPDATE_CURRENT)?.let { manager.setWindow(AlarmManager.RTC_WAKEUP, at, 10 * 60_000L, it) }
            Log.i(TAG, "wind-down reminder scheduled")
        } catch (error: Exception) {
            Log.e(TAG, "bedtime scheduling failed", error)
        }
    }

    // The next instant showing this wall-clock time. In a fall-back overlap the
    // time occurs twice, so take the first one still ahead; in a spring-forward
    // gap java.time moves it forward by the gap. A time already past (which the
    // core never returns) is checked again in a minute rather than looping.
    internal fun alarmInstant(local: LocalDateTime, zone: ZoneId, now: Instant): Instant {
        val zoned = local.atZone(zone)
        return listOf(zoned.withEarlierOffsetAtOverlap(), zoned.withLaterOffsetAtOverlap())
            .map { it.toInstant() }
            .firstOrNull { it.isAfter(now) } ?: now.plusSeconds(60)
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

// Alarms are cleared on reboot and can be lost on update, and a new clock or
// timezone moves the wall-clock time; reschedule from the cached plan, offline.
class BedtimeRescheduleReceiver : BroadcastReceiver() {
    private val actions = setOf(Intent.ACTION_TIMEZONE_CHANGED, Intent.ACTION_TIME_CHANGED, Intent.ACTION_BOOT_COMPLETED, Intent.ACTION_MY_PACKAGE_REPLACED)
    override fun onReceive(context: Context, intent: Intent) {
        if (intent.action !in actions) return
        val pending = goAsync()
        Thread {
            try { BedtimeScheduler.run(context.applicationContext) } finally { pending.finish() }
        }.start()
    }
}
