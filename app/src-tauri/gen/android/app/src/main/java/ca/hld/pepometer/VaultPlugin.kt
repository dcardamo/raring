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
            TokenVault.save(activity, invoke.getArgs().getString("value"))
            SyncScheduler.start(activity)
            invoke.resolve(JSObject())
        } catch (e: Exception) { invoke.reject("token vault unavailable", e) }
    }
    @Command fun clearTokens(invoke: Invoke) {
        try { TokenVault.clear(activity); invoke.resolve(JSObject()) }
        catch (e: Exception) { invoke.reject("token vault unavailable", e) }
    }
    @Command fun schedule(invoke: Invoke) {
        SyncScheduler.start(activity)
        invoke.resolve(JSObject())
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
