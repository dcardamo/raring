package ca.hld.pepometer

import android.Manifest
import android.app.Activity
import android.os.Build
import android.content.Intent
import android.webkit.WebView
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
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.sync.withLock

@TauriPlugin(permissions = [Permission(strings = [Manifest.permission.POST_NOTIFICATIONS], alias = "notifications")])
class VaultPlugin(private val activity: Activity): Plugin(activity) {
    private var webview: WebView? = null
    override fun load(webView: WebView) {
        this.webview = webView
        openTarget(activity.intent)
    }
    override fun onNewIntent(intent: Intent) { openTarget(intent) }
    private fun openTarget(intent: Intent?) {
        val target = intent?.getStringExtra("target")?.takeIf { it == "today" || it == "checkin" } ?: return
        webview?.post { webview?.loadUrl("http://tauri.localhost/#$target") }
        intent.removeExtra("target")
    }
    @Command fun loadTokens(invoke: Invoke) {
        try { invoke.resolve(JSObject().apply { put("value", TokenVault.load(activity)) }) }
        catch (e: Exception) { invoke.reject("token vault unavailable", e) }
    }
    @Command fun saveTokens(invoke: Invoke) {
        try {
            runBlocking { SyncGate.mutex.withLock { TokenVault.save(activity, invoke.getArgs().getString("value")) } }
            SyncScheduler.start(activity)
            invoke.resolve(JSObject())
        } catch (e: Exception) { invoke.reject("token vault unavailable", e) }
    }
    @Command fun clearTokens(invoke: Invoke) {
        try {
            SyncScheduler.cancel(activity)
            runBlocking { SyncGate.mutex.withLock { TokenVault.clear(activity) } }
            invoke.resolve(JSObject())
        }
        catch (e: Exception) { invoke.reject("token vault unavailable", e) }
    }
    @Command fun schedule(invoke: Invoke) {
        if (TokenVault.load(activity) != null) SyncScheduler.start(activity)
        invoke.resolve(JSObject())
    }
    @Command fun syncNow(invoke: Invoke) {
        CoroutineScope(Dispatchers.IO).launch {
            try {
                SyncScheduler.immediate(activity).result.get()
                val completed = WorkManager.getInstance(activity).getWorkInfosForUniqueWorkFlow("garmin-immediate")
                    .first { infos -> infos.any { it.state.isFinished } }
                if (completed.any { it.state == WorkInfo.State.SUCCEEDED }) invoke.resolve(JSObject())
                else invoke.reject("sync failed — try again")
            } catch (e: Exception) { invoke.reject("sync failed — try again", e) }
        }
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
