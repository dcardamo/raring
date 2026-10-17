package app.raring

import android.Manifest
import android.app.Activity
import android.content.ActivityNotFoundException
import android.os.Build
import android.content.Intent
import android.webkit.WebView
import android.webkit.CookieManager
import android.webkit.WebStorage
import app.tauri.PermissionState
import app.tauri.annotation.Command
import app.tauri.annotation.Permission
import app.tauri.annotation.PermissionCallback
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import androidx.work.WorkManager
import androidx.work.WorkInfo
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.launch
import kotlinx.coroutines.withTimeoutOrNull
import kotlinx.coroutines.sync.withLock

// WorkManager increments the count on RUNNING; only ENQUEUED again means a retry.
internal fun WorkInfo.isSyncAttemptComplete(): Boolean =
    state.isFinished || (state == WorkInfo.State.ENQUEUED && runAttemptCount > 0)

@TauriPlugin(permissions = [Permission(strings = [Manifest.permission.POST_NOTIFICATIONS], alias = "notifications")])
class VaultPlugin(private val activity: Activity): Plugin(activity) {
    private var webview: WebView? = null
    override fun load(webView: WebView) {
        this.webview = webView
        openTarget(activity.intent)
    }
    override fun onNewIntent(intent: Intent) { openTarget(intent) }
    private fun openTarget(intent: Intent?) {
        val target = intent?.getStringExtra("target")?.takeIf { it in setOf("today", "checkin", "sleep", "training", "health") } ?: return
        webview?.post { webview?.loadUrl("http://tauri.localhost/#$target") }
        intent.removeExtra("target")
    }
    @Command fun loadTokens(invoke: Invoke) {
        CoroutineScope(Dispatchers.IO).launch {
            invoke.resolve(JSObject().apply { put("value", TokenVault.loadOrClear(activity)) })
        }
    }
    @Command fun saveTokens(invoke: Invoke) {
        CoroutineScope(Dispatchers.IO).launch {
            try {
                SyncGate.mutex.withLock { TokenVault.save(activity, invoke.getArgs().getString("value")) }
                SyncScheduler.start(activity)
                invoke.resolve(JSObject())
            } catch (e: Exception) { invoke.reject("token vault unavailable", e) }
        }
    }
    @Command fun clearTokens(invoke: Invoke) {
        CoroutineScope(Dispatchers.IO).launch {
            try {
                SyncScheduler.cancel(activity)
                SyncGate.mutex.withLock { TokenVault.clear(activity) }
                CookieManager.getInstance().removeAllCookies(null)
                WebStorage.getInstance().deleteAllData()
                invoke.resolve(JSObject())
            } catch (e: Exception) { invoke.reject("token vault unavailable", e) }
        }
    }
    @Command fun schedule(invoke: Invoke) {
        CoroutineScope(Dispatchers.IO).launch {
            if (TokenVault.loadOrClear(activity) != null) SyncScheduler.start(activity)
            invoke.resolve(JSObject())
        }
    }
    @Command fun syncNow(invoke: Invoke) {
        CoroutineScope(Dispatchers.IO).launch {
            try {
                val id = SyncScheduler.immediate(activity)
                val completed = withTimeoutOrNull(60_000) {
                    WorkManager.getInstance(activity).getWorkInfoByIdFlow(id)
                        .first { info -> info?.isSyncAttemptComplete() == true }
                }
                if (completed?.state == WorkInfo.State.SUCCEEDED) invoke.resolve(JSObject())
                else invoke.reject("sync pending — try again later")
            } catch (e: Exception) { invoke.reject("sync failed — try again", e) }
        }
    }
    @Command fun openGarminConnect(invoke: Invoke) {
        val intent = activity.packageManager.getLaunchIntentForPackage("com.garmin.android.apps.connectmobile")
        val launched = if (intent == null) false else try {
            activity.startActivity(intent)
            true
        } catch (_: ActivityNotFoundException) {
            false
        }
        invoke.resolve(JSObject().apply { put("launched", launched) })
    }
    @Command fun requestNotificationPermission(invoke: Invoke) {
        if (Build.VERSION.SDK_INT < 33 || getPermissionState("notifications") == PermissionState.GRANTED) {
            permissionResult(invoke)
        } else requestPermissionForAlias("notifications", invoke, "permissionResult")
    }
    @PermissionCallback fun permissionResult(invoke: Invoke) {
        invoke.resolve(JSObject().apply {
            put("granted", Build.VERSION.SDK_INT < 33 || getPermissionState("notifications") == PermissionState.GRANTED)
        })
    }
}
