package app.raring

import android.Manifest
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.appwidget.AppWidgetManager
import android.appwidget.AppWidgetProvider
import android.content.BroadcastReceiver
import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.graphics.Bitmap
import android.graphics.Canvas
import android.graphics.Color
import android.graphics.Paint
import android.os.Build
import android.util.Log
import android.widget.RemoteViews
import androidx.core.app.NotificationCompat
import androidx.core.content.ContextCompat
import org.json.JSONObject
import java.io.File
import java.time.OffsetDateTime
import java.time.format.DateTimeFormatter

internal fun safeStatusTarget(route: String) = route.takeIf { it in setOf("today", "readiness", "sleep", "metric/bodyBattery", "metric/stress") } ?: "today"
internal fun shouldPostStatus(enabled: Boolean, granted: Boolean) = enabled && granted

internal object StatusSurfaces {
    private const val CHANNEL = "raring-status"
    private const val ID = 7002
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
            view.setOnClickPendingIntent(R.id.status_tile_sleep, pending(context, "sleep"))
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
    private fun notification(context: Context, status: StatusSnapshot) {
        val manager = context.getSystemService(NotificationManager::class.java)
        val granted = manager.areNotificationsEnabled() && (Build.VERSION.SDK_INT < 33 ||
            ContextCompat.checkSelfPermission(context, Manifest.permission.POST_NOTIFICATIONS) == PackageManager.PERMISSION_GRANTED)
        if (!shouldPostStatus(status.enabled, granted)) {
            manager.cancel(ID)
            return
        }
        manager.createNotificationChannel(NotificationChannel(CHANNEL, "Status at a glance", NotificationManager.IMPORTANCE_LOW).apply {
            setSound(null, null)
            enableVibration(false)
            setShowBadge(false)
            lockscreenVisibility = android.app.Notification.VISIBILITY_PRIVATE
        })
        val compact = RemoteViews(context.packageName, R.layout.status_notification_compact)
        compact.setTextViewText(R.id.status_asof, status.asOf)
        compact.setTextViewText(R.id.status_battery, status.battery.display)
        compact.setTextViewText(R.id.status_stress, status.stress.display)
        compact.setTextViewText(R.id.status_hr, status.heartRate.display)
        val expanded = RemoteViews(context.packageName, R.layout.status_notification_expanded)
        expanded.setTextViewText(R.id.status_asof, status.asOf)
        expanded.setTextViewText(R.id.status_battery, status.battery.display)
        expanded.setTextViewText(R.id.status_stress, status.stress.display)
        expanded.setTextViewText(R.id.status_hr, status.heartRate.display)
        expanded.setTextViewText(R.id.status_focus, status.focus)
        expanded.setTextViewText(R.id.status_reason, status.reason)
        expanded.setProgressBar(R.id.status_battery_bar, 100, status.battery.value?.toInt() ?: 0, false)
        expanded.setProgressBar(R.id.status_stress_bar, 100, status.stress.value?.toInt() ?: 0, false)
        expanded.setViewVisibility(R.id.status_battery_bar, if (status.battery.value == null) android.view.View.INVISIBLE else android.view.View.VISIBLE)
        expanded.setViewVisibility(R.id.status_stress_bar, if (status.stress.value == null) android.view.View.INVISIBLE else android.view.View.VISIBLE)
        // HR is a reading, not a 0–100 gauge: no invented scale.
        val notice = NotificationCompat.Builder(context, CHANNEL)
            .setSmallIcon(R.drawable.ic_notification).setContentTitle("Raring")
            .setContentText("${status.asOf} · ${status.focus}")
            .setCustomContentView(compact).setCustomBigContentView(expanded)
            .setStyle(NotificationCompat.DecoratedCustomViewStyle())
            .setContentIntent(pending(context, status.target))
            .setOngoing(true).setSilent(true).setOnlyAlertOnce(true)
            .setVisibility(NotificationCompat.VISIBILITY_PRIVATE)
            .setPriority(NotificationCompat.PRIORITY_LOW).build()
        manager.notify(ID, notice)
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
        context.getSystemService(NotificationManager::class.java).cancel(ID)
        drawWidgets(context, null)
    }
    // Settings already wrote the new snapshot. In particular, opt-out must not
    // wait behind minutes of network work. Re-read inside the rendering lock so
    // an older renderer cannot repost the notification after cancellation.
    fun applySettings(context: Context) = render(context)
    private fun render(context: Context) = synchronized(renderLock) {
        if (!AccountSession.active(context)) {
            clear(context)
        } else {
            // Rust checks the verified account binding and authoritative opt-out.
            // An old file's enabled bit can never authorize a notification.
            val response = JSONObject(NativeBridge.accountSession(AccountSession.db(context), "snapshot", "", ""))
            val status = response.optJSONObject("snapshot")?.toString()?.let { StatusSnapshot.parse(it) }
            drawWidgets(context, status)
            if (status == null) context.getSystemService(NotificationManager::class.java).cancel(ID)
            else notification(context, status)
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
