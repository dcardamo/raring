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
import android.os.Build
import android.util.Log
import android.view.View
import android.widget.RemoteViews
import androidx.core.content.ContextCompat
import org.json.JSONObject
import java.io.File
import java.time.OffsetDateTime
import java.time.format.DateTimeFormatter

internal fun safeStatusTarget(route: String) = route.takeIf { it in setOf("today", "readiness", "sleep", "sleep/last", "metric/hrv") } ?: "today"

internal object StatusSurfaces {
    private const val RETIRED_CHANNEL = "raring-status"
    private const val RETIRED_ID = 7002
    private val track = Color.rgb(52, 58, 67)
    private val surface = Color.rgb(27, 30, 36)
    private fun tone(tone: String) = when (tone) {
        "good" -> Color.rgb(96, 211, 160)
        "fair" -> Color.rgb(242, 184, 94)
        "poor" -> Color.rgb(241, 126, 120)
        else -> Color.rgb(194, 200, 209)
    }
    private fun pending(context: Context, route: String): PendingIntent {
        val target = safeStatusTarget(route)
        val intent = Intent(context, MainActivity::class.java).putExtra("target", target)
            .addFlags(Intent.FLAG_ACTIVITY_SINGLE_TOP or Intent.FLAG_ACTIVITY_CLEAR_TOP)
        return PendingIntent.getActivity(context, target.hashCode(), intent,
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE)
    }
    // The app's gauge: a 270° arc open at the bottom, with a marker at the reading.
    // Absent readings draw only the track. Never turn null into 0.
    internal fun gauge(metric: StatusMetric, density: Float = 1f, sizeDp: Float = 80f): Bitmap {
        val px = (sizeDp * density).toInt()
        val bitmap = Bitmap.createBitmap(px, px, Bitmap.Config.ARGB_8888)
        val canvas = Canvas(bitmap).apply { scale(density, density) }
        val stroke = sizeDp * 0.08f
        val inset = stroke * 1.1f
        val bounds = android.graphics.RectF(inset, inset, sizeDp - inset, sizeDp - inset)
        val paint = Paint(Paint.ANTI_ALIAS_FLAG).apply { style = Paint.Style.STROKE; strokeWidth = stroke; strokeCap = Paint.Cap.ROUND }
        paint.color = track
        canvas.drawArc(bounds, 135f, 270f, false, paint)
        metric.value?.let {
            val sweep = 270f * (it / 100).toFloat().coerceIn(0f, 1f)
            paint.color = tone(metric.tone)
            if (sweep > 0) canvas.drawArc(bounds, 135f, sweep, false, paint)
            val angle = Math.toRadians((135f + sweep).toDouble())
            val r = bounds.width() / 2
            val x = bounds.centerX() + r * Math.cos(angle).toFloat()
            val y = bounds.centerY() + r * Math.sin(angle).toFloat()
            val dot = Paint(Paint.ANTI_ALIAS_FLAG)
            dot.color = surface
            canvas.drawCircle(x, y, stroke * 0.62f + 1.5f, dot)
            dot.color = Color.WHITE
            canvas.drawCircle(x, y, stroke * 0.62f, dot)
        }
        return bitmap
    }
    private const val DASH = "\u2014"
    // A dash, not a blank or a zero, says a reading is missing.
    private fun reading(view: RemoteViews, id: Int, metric: StatusMetric?, context: Context) {
        val value = metric?.value?.let { metric.display }
        view.setTextViewText(id, value ?: DASH)
        view.setTextColor(id, ContextCompat.getColor(context, if (value == null) R.color.widget_text3 else R.color.widget_ink))
    }
    internal fun widgetViews(context: Context, medium: Boolean, status: StatusSnapshot?): RemoteViews {
        val view = RemoteViews(context.packageName, if (medium) R.layout.status_widget_medium else R.layout.status_widget_small)
        val readiness = status?.readiness ?: StatusMetric(null, "neutral")
        // Matches the layouts' gauge frames.
        view.setImageViewBitmap(R.id.status_gauge, gauge(readiness, context.resources.displayMetrics.density, if (medium) 80f else 72f))
        reading(view, R.id.status_readiness, readiness, context)
        view.setTextViewText(R.id.status_asof, status?.asOf ?: "As of unavailable")
        view.setOnClickPendingIntent(R.id.status_tile_readiness, pending(context, "readiness"))
        if (medium) {
            reading(view, R.id.status_sleep, status?.sleep, context)
            reading(view, R.id.status_hrv, status?.hrv, context)
            // Garmin's status word, in its rating colour, beside the number.
            val hrvStatus = status?.hrvStatus.orEmpty().takeIf { status?.hrv?.value != null }.orEmpty()
            view.setTextViewText(R.id.status_hrv_status, hrvStatus)
            view.setTextColor(R.id.status_hrv_status, tone(status?.hrv?.tone ?: "neutral"))
            view.setViewVisibility(R.id.status_hrv_status, if (hrvStatus.isEmpty()) View.GONE else View.VISIBLE)
            view.setViewVisibility(R.id.status_hrv_unit, if (status?.hrv?.value == null) View.GONE else View.VISIBLE)
            view.setOnClickPendingIntent(android.R.id.background, pending(context, "today"))
            view.setOnClickPendingIntent(R.id.status_tile_sleep, pending(context, "sleep/last"))
            view.setOnClickPendingIntent(R.id.status_tile_hrv, pending(context, "metric/hrv"))
        } else {
            view.setTextViewText(R.id.status_focus, status?.focus ?: "")
            view.setOnClickPendingIntent(android.R.id.background, pending(context, status?.target ?: "today"))
        }
        return view
    }
    // Android keeps an app's notifications across updates, so installed copies may
    // still show the retired ongoing status notification until it is removed.
    private fun retireNotification(context: Context) {
        val manager = context.getSystemService(NotificationManager::class.java)
        manager.cancel(RETIRED_ID)
        if (Build.VERSION.SDK_INT >= 26) manager.deleteNotificationChannel(RETIRED_CHANNEL)
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
            if (ids.isNotEmpty()) manager.updateAppWidget(ids, widgetViews(context, medium, status))
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
