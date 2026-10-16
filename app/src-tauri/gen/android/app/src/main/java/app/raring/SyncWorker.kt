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
    @JvmStatic external fun sync(dbPath: String, tokensJson: String, nowIso: String): String
    @JvmStatic external fun markSent(dbPath: String, tagsJson: String, nowIso: String): String
}

// The vault read, JNI refresh and vault write must share one lock across every work request.
object SyncGate { val mutex = Mutex() }

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
                .setAutoCancel(true).setContentIntent(pending).build()
            (context.getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager).notify(id.hashCode(), notification)
            Log.i(TAG, "notification posted: ${id.substringBefore(':')}")
            return true
        }
    }
    override suspend fun doWork(): Result = withContext(Dispatchers.IO) {
        try {
            SyncGate.mutex.withLock {
                val tokens = TokenVault.loadOrClear(applicationContext)
                if (tokens == null) {
                    SyncScheduler.cancel(applicationContext)
                    notify(applicationContext, "auth-required", "Sign in to Garmin again", "Open Raring to reconnect", "today")
                    return@withLock Result.success()
                }
                val prepared = JSONObject(NativeBridge.prepareTokens(tokens))
                if (prepared.optBoolean("authFailed")) {
                    TokenVault.clear(applicationContext)
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
                // Persist a rotated token before handling partial endpoint failures.
                if (!response.optBoolean("authFailed")) TokenVault.save(applicationContext, response.getString("tokensJson"))
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
                if (response.optBoolean("authFailed")) {
                    TokenVault.clear(applicationContext)
                    SyncScheduler.cancel(applicationContext)
                    notify(applicationContext, "auth-required", "Sign in to Garmin again", "Open Raring to reconnect", "today")
                    return@withLock Result.success()
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
