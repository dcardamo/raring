package app.raring

import android.Manifest
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.os.Build
import android.util.Log
import androidx.core.app.NotificationCompat
import androidx.core.content.ContextCompat
import androidx.work.Constraints
import androidx.work.CoroutineWorker
import androidx.work.ExistingPeriodicWorkPolicy
import androidx.work.ExistingWorkPolicy
import androidx.work.NetworkType
import androidx.work.OutOfQuotaPolicy
import androidx.work.OneTimeWorkRequestBuilder
import androidx.work.PeriodicWorkRequestBuilder
import androidx.work.WorkerParameters
import androidx.work.WorkManager
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withContext
import org.json.JSONObject
import java.io.File
import java.time.LocalDateTime
import java.time.format.DateTimeFormatter
import java.time.OffsetDateTime
import java.util.concurrent.TimeUnit

object NativeBridge {
    init { System.loadLibrary("raring_app_lib") }
    @JvmStatic external fun prepareTokens(tokensJson: String): String
    @JvmStatic external fun exchangeTicket(ticket: String, testApi: String): String
    @JvmStatic external fun sync(dbPath: String, tokensJson: String, nowIso: String): String
    @JvmStatic external fun heartRateDay(dbPath: String, tokensJson: String, date: String): String
    @JvmStatic external fun refreshStrength(dbPath: String, tokensJson: String, activityId: String): String
    @JvmStatic external fun markSent(dbPath: String, tagsJson: String, nowIso: String): String
    @JvmStatic external fun bedtime(dbPath: String, nowLocal: String): String
    @JvmStatic external fun refreshStatus(dbPath: String, nowIso: String): String
    @JvmStatic external fun accountSession(dbPath: String, action: String, tokensJson: String, testApi: String): String
}

// The vault read, JNI refresh and vault write must share one lock across every work request.
object SyncGate {
    val mutex = Mutex()
    // Same attempt-generation semantics as DesktopCredentials. All Android
    // invalidation/check/promotion runs under this existing gate, so a profile
    // request cannot be crossed by another invalidation.
    private var signInGeneration = 0L
    internal fun invalidateSignInLocked(): Long { check(mutex.isLocked); return ++signInGeneration }
    internal fun isCurrentSignInLocked(generation: Long): Boolean {
        check(mutex.isLocked)
        return generation == signInGeneration
    }
}

object SyncScheduler {
    private val network = Constraints.Builder().setRequiredNetworkType(NetworkType.CONNECTED).build()
    fun start(context: Context) {
        val request = PeriodicWorkRequestBuilder<SyncWorker>(60, TimeUnit.MINUTES).setConstraints(network).build()
        WorkManager.getInstance(context).enqueueUniquePeriodicWork("garmin-hourly", ExistingPeriodicWorkPolicy.KEEP, request)
        SyncWorker.channel(context)
    }
    fun followUp(context: Context) {
        val request = OneTimeWorkRequestBuilder<SyncWorker>().setInitialDelay(15, TimeUnit.MINUTES)
            .setConstraints(network).build()
        WorkManager.getInstance(context).enqueueUniqueWork("readiness-followup", ExistingWorkPolicy.REPLACE, request)
    }
    fun immediate(context: Context): java.util.UUID {
        val request = OneTimeWorkRequestBuilder<SyncWorker>().setConstraints(network)
            .setExpedited(OutOfQuotaPolicy.RUN_AS_NON_EXPEDITED_WORK_REQUEST).build()
        WorkManager.getInstance(context).enqueueUniqueWork("garmin-immediate", ExistingWorkPolicy.REPLACE, request)
        return request.id
    }
    fun cancel(context: Context) {
        Log.i("RaringSync", "scheduled work cancelled")
        val manager = WorkManager.getInstance(context)
        manager.cancelUniqueWork("garmin-hourly")
        manager.cancelUniqueWork("readiness-followup")
        manager.cancelUniqueWork("garmin-immediate")
    }
    fun testRun(context: Context) {
        WorkManager.getInstance(context).enqueue(OneTimeWorkRequestBuilder<SyncWorker>().build())
    }
}

class SyncWorker(context: Context, params: WorkerParameters): CoroutineWorker(context, params) {
    companion object {
        private const val TAG = "RaringSync"
        private const val CHANNEL = "raring"
        fun channel(context: Context) {
            (context.getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager)
                .createNotificationChannel(NotificationChannel(CHANNEL, "Raring", NotificationManager.IMPORTANCE_DEFAULT))
        }
        fun notify(context: Context, id: String, title: String, body: String, target: String): Boolean {
            channel(context)
            if (Build.VERSION.SDK_INT >= 33 && ContextCompat.checkSelfPermission(context, Manifest.permission.POST_NOTIFICATIONS) != PackageManager.PERMISSION_GRANTED) {
                Log.i(TAG, "notification permission missing")
                return false
            }
            val intent = Intent(context, MainActivity::class.java).putExtra("target", target)
                .addFlags(Intent.FLAG_ACTIVITY_SINGLE_TOP or Intent.FLAG_ACTIVITY_CLEAR_TOP)
            val pending = PendingIntent.getActivity(context, id.hashCode(), intent, PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE)
            val notification = NotificationCompat.Builder(context, CHANNEL)
                .setSmallIcon(R.drawable.ic_notification).setContentTitle(title).setContentText(body)
                .setStyle(NotificationCompat.BigTextStyle().bigText(body))
                .setAutoCancel(true).setContentIntent(pending).build()
            (context.getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager).notify(id.hashCode(), notification)
            Log.i(TAG, "notification posted: ${id.substringBefore(':')}")
            return true
        }
    }
    override suspend fun doWork(): Result = withContext(Dispatchers.IO) {
        try {
            syncLocked()
        } finally {
            // Each sync can move tonight's plan; the reminder follows it.
            BedtimeScheduler.run(applicationContext)
            StatusSurfaces.refresh(applicationContext)
        }
    }
    private suspend fun syncLocked(): Result {
        return try {
            SyncGate.mutex.withLock {
                // The debug emulator's synthetic session has no tokens by design; it is
                // offline data, not a lost sign-in, so there is nothing to sync or suspend.
                if (AccountSession.syncSkipsDemo(applicationContext)) return@withLock Result.success()
                var tokens = AccountSession.loadLocked(applicationContext)
                if (tokens == null) {
                    AccountSession.suspendLocked(applicationContext)
                    SyncScheduler.cancel(applicationContext)
                    notify(applicationContext, "auth-required", "Sign in to Garmin again", "Open Raring to reconnect", "today")
                    return@withLock Result.success()
                }
                if (!AccountSession.active(applicationContext)) {
                    if (!AccountSession.migrationAllowed(applicationContext)) {
                        AccountSession.suspendLocked(applicationContext)
                        SyncScheduler.cancel(applicationContext)
                        return@withLock Result.success()
                    }
                    try {
                        AccountSession.verifyAndSaveLocked(applicationContext, tokens)
                        tokens = TokenVault.load(applicationContext) ?: return@withLock Result.failure()
                    } catch (_: Exception) {
                        SyncScheduler.cancel(applicationContext)
                        notify(applicationContext, "auth-required", "Sign in to Garmin again", "Account identity could not be verified", "today")
                        return@withLock Result.success()
                    }
                }
                val prepared = JSONObject(NativeBridge.prepareTokens(tokens))
                if (prepared.optBoolean("authFailed")) {
                    AccountSession.suspendLocked(applicationContext)
                    SyncScheduler.cancel(applicationContext)
                    notify(applicationContext, "auth-required", "Sign in to Garmin again", "Open Raring to reconnect", "today")
                    return@withLock Result.success()
                }
                if (prepared.has("error")) return@withLock Result.retry()
                val readyTokens = prepared.getString("tokensJson")
                if (readyTokens != tokens) TokenVault.save(applicationContext, readyTokens)
                val now = OffsetDateTime.now().format(DateTimeFormatter.ISO_OFFSET_DATE_TIME)
                val db = File(applicationContext.filesDir, "raring.db").absolutePath
                val response = JSONObject(NativeBridge.sync(db, readyTokens, now))
                if (response.optBoolean("authFailed")) {
                    AccountSession.suspendLocked(applicationContext)
                    SyncScheduler.cancel(applicationContext)
                    notify(applicationContext, "auth-required", "Sign in to Garmin again", "Open Raring to reconnect", "today")
                    return@withLock Result.success()
                }
                // Persist a rotated token before handling partial endpoint failures.
                TokenVault.save(applicationContext, response.getString("tokensJson"))
                Log.i(TAG, "background JNI sync completed")
                val alerts = response.optJSONArray("notifications") ?: org.json.JSONArray()
                for (i in 0 until alerts.length()) {
                    val alert = alerts.getJSONObject(i)
                    if (notify(applicationContext, alert.getString("id"), alert.getString("title"), alert.getString("body"), alert.optString("target", "today"))) {
                        val tags = org.json.JSONArray().put(JSONObject().put("kind", alert.getString("kind"))
                            .put("tag", alert.getString("tag")).put("title", "").put("body", ""))
                        val marked = JSONObject(NativeBridge.markSent(db, tags.toString(), now))
                        if (marked.has("error")) Log.w(TAG, "could not record notification delivery")
                    }
                }
                if (!response.isNull("error")) {
                    Log.w(TAG, "background sync incomplete: ${response.getString("error")}")
                    return@withLock Result.retry()
                }
                val hour = LocalDateTime.now().hour
                if (response.optBoolean("readinessMissing") && hour in 4..10) SyncScheduler.followUp(applicationContext)
                Result.success()
            }
        } catch (error: Exception) {
            Log.e(TAG, "background sync failed", error)
            Result.retry()
        }
    }
}
