package ca.hld.pepometer

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
import java.time.OffsetDateTime
import java.util.concurrent.TimeUnit

object NativeBridge {
    init { System.loadLibrary("pepometer_app_lib") }
    @JvmStatic external fun sync(dbPath: String, tokensJson: String, nowIso: String): String
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
        WorkManager.getInstance(context).enqueueUniqueWork("readiness-followup", ExistingWorkPolicy.APPEND_OR_REPLACE, request)
    }
    fun immediate(context: Context) = WorkManager.getInstance(context).enqueueUniqueWork(
        "garmin-immediate", ExistingWorkPolicy.KEEP,
        OneTimeWorkRequestBuilder<SyncWorker>().setConstraints(network)
            .setExpedited(OutOfQuotaPolicy.RUN_AS_NON_EXPEDITED_WORK_REQUEST).build()
    )
    fun cancel(context: Context) {
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
        private const val TAG = "PepometerSync"
        private const val CHANNEL = "pepometer"
        fun channel(context: Context) {
            (context.getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager)
                .createNotificationChannel(NotificationChannel(CHANNEL, "Pepometer", NotificationManager.IMPORTANCE_DEFAULT))
        }
        fun notify(context: Context, id: String, title: String, body: String, target: String) {
            channel(context)
            if (Build.VERSION.SDK_INT >= 33 && ContextCompat.checkSelfPermission(context, Manifest.permission.POST_NOTIFICATIONS) != PackageManager.PERMISSION_GRANTED) {
                Log.i(TAG, "notification permission missing")
                return
            }
            val intent = Intent(context, MainActivity::class.java).putExtra("target", target)
                .addFlags(Intent.FLAG_ACTIVITY_SINGLE_TOP or Intent.FLAG_ACTIVITY_CLEAR_TOP)
            val pending = PendingIntent.getActivity(context, id.hashCode(), intent, PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE)
            val notification = NotificationCompat.Builder(context, CHANNEL)
                .setSmallIcon(R.mipmap.ic_launcher).setContentTitle(title).setContentText(body)
                .setAutoCancel(true).setContentIntent(pending).build()
            (context.getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager).notify(id.hashCode(), notification)
            Log.i(TAG, "notification posted: $id")
        }
    }
    override suspend fun doWork(): Result = withContext(Dispatchers.IO) {
        try {
            SyncGate.mutex.withLock {
                val tokens = TokenVault.load(applicationContext)
                if (tokens == null) {
                    notify(applicationContext, "auth-required", "Sign in to Garmin again", "Open Pepometer to reconnect", "today")
                    return@withLock Result.success()
                }
                val now = OffsetDateTime.now().toString()
                val response = JSONObject(NativeBridge.sync(File(applicationContext.filesDir, "pepometer.db").absolutePath, tokens, now))
                if (response.optBoolean("authFailed")) {
                    TokenVault.clear(applicationContext)
                    notify(applicationContext, "auth-required", "Sign in to Garmin again", "Open Pepometer to reconnect", "today")
                    return@withLock Result.success()
                }
                // A refresh may have succeeded even when a later endpoint fails. Always persist it first.
                TokenVault.save(applicationContext, response.getString("tokensJson"))
                if (!response.isNull("error")) {
                    Log.w(TAG, "background sync incomplete: ${response.getString("error")}")
                    return@withLock Result.retry()
                }
                Log.i(TAG, "background JNI sync completed")
                val alerts = response.getJSONArray("notifications")
                for (i in 0 until alerts.length()) {
                    val alert = alerts.getJSONObject(i)
                    notify(applicationContext, alert.getString("id"), alert.getString("title"), alert.getString("body"), alert.optString("target", "today"))
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
