package app.raring

import android.app.NotificationManager
import android.app.PendingIntent
import android.appwidget.AppWidgetManager
import android.appwidget.AppWidgetProvider
import android.content.BroadcastReceiver
import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.graphics.Bitmap
import android.graphics.Canvas
import android.graphics.Color
import android.graphics.Paint
import android.util.Log
import android.widget.RemoteViews
import org.json.JSONObject
import java.io.File
import java.time.OffsetDateTime
import java.time.format.DateTimeFormatter

internal fun safeStatusTarget(route: String) = route.takeIf { it in setOf("today", "readiness", "sleep", "sleep/last", "metric/bodyBattery", "metric/stress") } ?: "today"

internal object StatusSurfaces {
    private const val RETIRED_CHANNEL = "raring-status"
    private const val RETIRED_ID = 7002
    private val track = Color.rgb(58, 68, 81)
    private fun tone(tone: String) = when (tone) {
        "good" -> Color.rgb(105, 215, 157)
        "fair" -> Color.rgb(244, 189, 91)
        "poor" -> Color.rgb(245, 120, 115)
        else -> Color.rgb(169, 182, 198)
    }
    private fun pending(context: Context, route: String): PendingIntent {
        val target = safeStatusTarget(route)
        val intent = Intent(context, MainActivity::class.java).putExtra("target", target)
            .addFlags(Intent.FLAG_ACTIVITY_SINGLE_TOP or Intent.FLAG_ACTIVITY_CLEAR_TOP)
        return PendingIntent.getActivity(context, target.hashCode(), intent,
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE)
    }
    // Draw only the track for absent readings. Never turn null into 0.
    internal fun gauge(metric: StatusMetric, density: Float = 1f): Bitmap {
        val bitmap = Bitmap.createBitmap((112 * density).toInt(), (92 * density).toInt(), Bitmap.Config.ARGB_8888)
        val canvas = Canvas(bitmap).apply { scale(density, density) }
        val paint = Paint(Paint.ANTI_ALIAS_FLAG).apply { style = Paint.Style.STROKE; strokeWidth = 9f; strokeCap = Paint.Cap.ROUND }
        // Open at the bottom, like the app's gauges.
        val bounds = android.graphics.RectF(12f, 8f, 100f, 96f)
        paint.color = track
        canvas.drawArc(bounds, 135f, 270f, false, paint)
        metric.value?.let {
            paint.color = tone(metric.tone)
            canvas.drawArc(bounds, 135f, (270f * (it / 100)).toFloat(), false, paint)
        }
        return bitmap
    }
    private fun widget(context: Context, medium: Boolean, status: StatusSnapshot?): RemoteViews {
        val view = RemoteViews(context.packageName, if (medium) R.layout.status_widget_medium else R.layout.status_widget_small)
        val readiness = status?.readiness ?: StatusMetric(null, "neutral")
        view.setImageViewBitmap(R.id.status_gauge, gauge(readiness, context.resources.displayMetrics.density))
        view.setTextViewText(R.id.status_readiness, readiness.display)
        view.setTextViewText(R.id.status_asof, status?.asOf ?: "As of unavailable")
        if (medium) {
            view.setTextViewText(R.id.status_sleep, status?.sleep?.display ?: "")
            view.setTextViewText(R.id.status_battery, status?.battery?.display ?: "")
            view.setTextViewText(R.id.status_stress, status?.stress?.display ?: "")
            view.setOnClickPendingIntent(R.id.status_tile_readiness, pending(context, "readiness"))
            view.setOnClickPendingIntent(R.id.status_tile_sleep, pending(context, "sleep/last"))
            view.setOnClickPendingIntent(R.id.status_tile_battery, pending(context, "metric/bodyBattery"))
            view.setOnClickPendingIntent(R.id.status_tile_stress, pending(context, "metric/stress"))
        } else {
            view.setTextViewText(R.id.status_focus, status?.focus ?: "")
            view.setOnClickPendingIntent(R.id.status_widget_root, pending(context, status?.target ?: "today"))
            view.setOnClickPendingIntent(R.id.status_gauge, pending(context, "readiness"))
            view.setOnClickPendingIntent(R.id.status_readiness, pending(context, "readiness"))
        }
        return view
    }
    // Android keeps an app's notifications across updates, so installed copies may
    // still show the retired ongoing status notification until it is removed.
    private fun retireNotification(context: Context) {
        val manager = context.getSystemService(NotificationManager::class.java)
        manager.cancel(RETIRED_ID)
        manager.deleteNotificationChannel(RETIRED_CHANNEL)
    }
    private val renderLock = Any()
    fun suspendSession(context: Context) = synchronized(renderLock) {
        context.getSystemService(NotificationManager::class.java).cancelAll()
        drawWidgets(context, null)
    }
    fun resumeSession(context: Context) = synchronized(renderLock) {
        render(context)
    }
    fun clear(context: Context) = synchronized(renderLock) {
        retireNotification(context)
        drawWidgets(context, null)
    }
    // Settings already wrote the new snapshot; redrawing must not wait behind
    // minutes of network work.
    fun applySettings(context: Context) = render(context)
    private fun render(context: Context) = synchronized(renderLock) {
        if (!AccountSession.active(context)) {
            clear(context)
        } else {
            retireNotification(context)
            // Rust checks the verified account binding.
            val response = JSONObject(NativeBridge.accountSession(AccountSession.db(context), "snapshot", "", ""))
            val status = response.optJSONObject("snapshot")?.toString()?.let { StatusSnapshot.parse(it) }
            drawWidgets(context, status)
        }
    }
    private fun drawWidgets(context: Context, status: StatusSnapshot?) {
        val manager = AppWidgetManager.getInstance(context)
        for ((provider, medium) in listOf(StatusSmallWidget::class.java to false, StatusMediumWidget::class.java to true)) {
            val ids = manager.getAppWidgetIds(ComponentName(context, provider))
            if (ids.isNotEmpty()) manager.updateAppWidget(ids, widget(context, medium, status))
        }
    }
    // All producers serialize with token-vault writes and sign-out. No network work.
    fun refresh(context: Context) = refreshWhenIdle(context)
    // Only an explicitly seeded synthetic demo may render without credentials.
    internal fun refreshOfflineForDebug(context: Context) = refresh(context)
    private fun refreshWhenIdle(context: Context) {
        // A network sync can take minutes, longer than a receiver's lifetime.
        // Its finally block refreshes these surfaces, so never wait for its lock.
        if (!SyncGate.mutex.tryLock()) return
        try { refreshLocked(context) } finally { SyncGate.mutex.unlock() }
    }
    private fun refreshLocked(context: Context) {
        try {
            if (!AccountSession.active(context)) { clear(context); return }
            val db = File(context.filesDir, "raring.db")
            if (!db.exists()) { clear(context); return }
            val now = OffsetDateTime.now().format(DateTimeFormatter.ISO_OFFSET_DATE_TIME)
            if (JSONObject(NativeBridge.refreshStatus(db.absolutePath, now)).has("error")) {
                Log.w("RaringStatus", "offline snapshot refresh failed")
                clear(context)
                return
            }
            render(context)
        } catch (e: Exception) {
            Log.w("RaringStatus", "offline status unavailable", e)
            clear(context)
        }
    }
}

abstract class StatusWidgetProvider : AppWidgetProvider() {
    override fun onUpdate(context: Context, manager: AppWidgetManager, ids: IntArray) { asyncRefresh(context) }
    override fun onReceive(context: Context, intent: Intent) {
        super.onReceive(context, intent)
        if (intent.action in setOf(Intent.ACTION_TIMEZONE_CHANGED, Intent.ACTION_TIME_CHANGED,
                Intent.ACTION_BOOT_COMPLETED, Intent.ACTION_MY_PACKAGE_REPLACED)) asyncRefresh(context)
    }
    private fun asyncRefresh(context: Context) {
        val pending = goAsync()
        Thread { try { StatusSurfaces.refresh(context.applicationContext) } finally { pending.finish() } }.start()
    }
}
class StatusSmallWidget : StatusWidgetProvider()
class StatusMediumWidget : StatusWidgetProvider()

class StatusClockReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        if (intent.action !in setOf(Intent.ACTION_TIMEZONE_CHANGED, Intent.ACTION_TIME_CHANGED,
                Intent.ACTION_BOOT_COMPLETED, Intent.ACTION_MY_PACKAGE_REPLACED)) return
        val pending = goAsync()
        Thread { try { StatusSurfaces.refresh(context.applicationContext) } finally { pending.finish() } }.start()
    }
}
