package app.raring

import android.Manifest
import android.app.Activity
import android.app.NotificationManager
import android.content.ActivityNotFoundException
import android.content.Intent
import android.os.Build
import android.webkit.WebView
import android.webkit.CookieManager
import android.webkit.WebStorage
import androidx.activity.result.ActivityResult
import org.json.JSONObject
import app.tauri.PermissionState
import app.tauri.annotation.ActivityCallback
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
import kotlinx.coroutines.withContext
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
        val target = intent?.getStringExtra("target")?.takeIf { it in setOf("today", "checkin", "sleep", "sleep/last", "sleep/tonight", "training", "health", "readiness", "metric/bodyBattery", "metric/stress") } ?: return
        webview?.post { webview?.loadUrl("http://tauri.localhost/#$target") }
        intent.removeExtra("target")
    }
    @Command fun loadTokens(invoke: Invoke) {
        CoroutineScope(Dispatchers.IO).launch {
            try {
                SyncGate.mutex.withLock {
                    invoke.resolve(JSObject().apply { put("value", if (AccountSession.active(activity)) AccountSession.loadLocked(activity) else null) })
                }
            } catch (e: Exception) { invoke.reject("Local session is suspended", e) }
        }
    }
    @Command fun exchangeTicket(invoke: Invoke) {
        CoroutineScope(Dispatchers.IO).launch {
            try {
                val args = invoke.getArgs()
                val saved = AccountSession.exchangeSignIn(activity, args.getString("ticket"), args.getLong("generation"))
                if (saved) SyncScheduler.start(activity)
                invoke.resolve(JSObject().apply { put("saved", saved) })
            } catch (e: Exception) { invoke.reject("Sign-in could not be completed. No unverified account data is shown.", e) }
        }
    }
    @Command fun suspendSession(invoke: Invoke) {
        CoroutineScope(Dispatchers.IO).launch {
            try {
                SyncScheduler.cancel(activity)
                val generation = AccountSession.beginSignIn(activity)
                invoke.resolve(JSObject().apply { put("generation", generation) })
            } catch (e: Exception) { invoke.reject("Could not suspend the previous session", e) }
        }
    }
    @Command fun sessionState(invoke: Invoke) {
        CoroutineScope(Dispatchers.IO).launch {
            invoke.resolve(JSObject().apply { put("active", AccountSession.active(activity)) })
        }
    }
    @Command fun demoSession(invoke: Invoke) {
        CoroutineScope(Dispatchers.IO).launch {
            try {
                SyncGate.mutex.withLock { AccountSession.useDemoLocked(activity) }
                invoke.resolve(JSObject())
            } catch (e: Exception) { invoke.reject("Demo requires a credential-free debug emulator", e) }
        }
    }
    @Command fun applyStatusSettings(invoke: Invoke) {
        CoroutineScope(Dispatchers.IO).launch {
            StatusSurfaces.applySettings(activity)
            invoke.resolve(JSObject())
        }
    }
    @Command fun clearTokens(invoke: Invoke) {
        CoroutineScope(Dispatchers.IO).launch {
            try {
                SyncScheduler.cancel(activity)
                BedtimeScheduler.cancel(activity)
                AccountSession.signOut(activity)
                CookieManager.getInstance().removeAllCookies(null)
                WebStorage.getInstance().deleteAllData()
                invoke.resolve(JSObject())
            } catch (e: Exception) { invoke.reject("token vault unavailable", e) }
        }
    }
    @Command fun schedule(invoke: Invoke) {
        CoroutineScope(Dispatchers.IO).launch {
            try {
                StatusSurfaces.applySettings(activity)
                if (AccountSession.isDemo(activity)) {
                    StatusSurfaces.refresh(activity)
                    invoke.resolve(JSObject())
                    return@launch
                }
                SyncGate.mutex.withLock {
                    if (AccountSession.loadLocked(activity) != null && (AccountSession.active(activity) || AccountSession.migrationAllowed(activity))) {
                        SyncScheduler.start(activity)
                        if (!AccountSession.active(activity)) SyncScheduler.immediate(activity)
                    } else {
                        SyncScheduler.cancel(activity)
                        AccountSession.suspendLocked(activity)
                    }
                }
                BedtimeScheduler.run(activity)
                StatusSurfaces.refresh(activity)
                invoke.resolve(JSObject())
            } catch (e: Exception) {
                // Suspension remains authoritative even when storage cannot be repaired.
                StatusSurfaces.clear(activity)
                invoke.reject("Local session is suspended", e)
            }
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
    @Command fun heartRateDay(invoke: Invoke) {
        CoroutineScope(Dispatchers.IO).launch {
            try {
                val date = invoke.getArgs().getString("date")
                val result = SyncGate.mutex.withLock {
                    check(AccountSession.active(activity)) { "sign in again" }
                    val tokens = AccountSession.loadLocked(activity) ?: ""
                    val db = java.io.File(activity.filesDir, "raring.db").absolutePath
                    val response = JSONObject(NativeBridge.heartRateDay(db, tokens, date))
                    if (response.optBoolean("authFailed")) {
                        AccountSession.suspendLocked(activity)
                        SyncScheduler.cancel(activity)
                        throw IllegalStateException("sign in again")
                    }
                    response.optString("tokensJson").takeIf { it.isNotEmpty() }?.let { TokenVault.save(activity, it) }
                    if (!response.isNull("error")) throw IllegalStateException(response.getString("error"))
                    response.getJSONObject("day")
                }
                invoke.resolve(JSObject().apply { put("day", result) })
            } catch (e: Exception) { invoke.reject("Heart rate unavailable — try again later", e) }
            finally { StatusSurfaces.refresh(activity) }
        }
    }
    // One explicitly opened past day or older workout; never navigation prefetch.
    @Command fun fetchDetail(invoke: Invoke) {
        CoroutineScope(Dispatchers.IO).launch {
            try {
                val date = invoke.getArgs().optString("date", "")
                val id = invoke.getArgs().optString("activityId", "")
                val scope = invoke.getArgs().optString("scope", "day")
                check(date.isNotEmpty() != id.isNotEmpty()) { "one day or workout" }
                SyncGate.mutex.withLock {
                    check(AccountSession.active(activity)) { "sign in again" }
                    // A demo session has no Garmin credentials; a detail read must not suspend it.
                    val tokens = AccountSession.loadLocked(activity) ?: ""
                    check(tokens.isNotEmpty()) { "no Garmin session for a detail read" }
                    val db = java.io.File(activity.filesDir, "raring.db").absolutePath
                    val response = JSONObject(NativeBridge.fetchDetail(db, tokens, date, id, scope))
                    if (response.optBoolean("authFailed")) {
                        AccountSession.suspendLocked(activity)
                        SyncScheduler.cancel(activity)
                        throw IllegalStateException("sign in again")
                    }
                    response.optString("tokensJson").takeIf { it.isNotEmpty() }?.let { TokenVault.save(activity, it) }
                    if (!response.isNull("error")) throw IllegalStateException(response.getString("error"))
                }
                invoke.resolve(JSObject())
            } catch (e: Exception) { invoke.reject("This detail is unavailable — try again later", e) }
            finally { StatusSurfaces.refresh(activity) }
        }
    }
    @Command fun refreshStrength(invoke: Invoke) {
        CoroutineScope(Dispatchers.IO).launch {
            try {
                val id = invoke.getArgs().getString("id")
                val result = SyncGate.mutex.withLock {
                    check(AccountSession.active(activity)) { "sign in again" }
                    val tokens = AccountSession.loadLocked(activity) ?: ""
                    val db = java.io.File(activity.filesDir, "raring.db").absolutePath
                    val response = JSONObject(NativeBridge.refreshStrength(db, tokens, id))
                    if (response.optBoolean("authFailed")) {
                        AccountSession.suspendLocked(activity)
                        SyncScheduler.cancel(activity)
                        throw IllegalStateException("sign in again")
                    }
                    response.optString("tokensJson").takeIf { it.isNotEmpty() }?.let { TokenVault.save(activity, it) }
                    if (!response.isNull("error")) throw IllegalStateException(response.getString("error"))
                    response.opt("session")
                }
                invoke.resolve(JSObject().apply { put("session", result) })
            } catch (e: Exception) { invoke.reject("Sets could not be refreshed — try again later", e) }
            finally { StatusSurfaces.refresh(activity) }
        }
    }
    @Command fun prepareExport(invoke: Invoke) {
        CoroutineScope(Dispatchers.IO).launch {
            try {
                ExportShare.prepare(activity, invoke.getArgs().getString("filename"))
                invoke.resolve(JSObject())
            } catch (e: Exception) {
                invoke.reject("could not prepare export", e)
            }
        }
    }
    @Command fun shareExport(invoke: Invoke) {
        CoroutineScope(Dispatchers.IO).launch {
            try {
                val name = invoke.getArgs().getString("filename")
                val expected = invoke.getArgs().getString("accountId")
                SyncGate.mutex.withLock {
                    val identity = org.json.JSONObject(NativeBridge.accountSession(AccountSession.db(activity), "state", "", "")).optString("identity")
                    if (!AccountSession.active(activity) || identity != expected) {
                        ExportShare.cleanPrepared(activity.cacheDir, name)
                        error("Account changed during export")
                    }
                    withContext(Dispatchers.Main) { ExportShare.share(activity, name) }
                }
                invoke.resolve(JSObject())
            } catch (e: Exception) {
                invoke.reject("could not share export", e)
            }
        }
    }
    // The system picker reads the chosen export; only a private cache copy is
    // handed to the core, which validates it before anything is written.
    @Command fun pickImport(invoke: Invoke) {
        CoroutineScope(Dispatchers.IO).launch {
            try {
                ImportPicker.sweep(activity.cacheDir)
                val intent = Intent(Intent.ACTION_OPEN_DOCUMENT).apply {
                    addCategory(Intent.CATEGORY_OPENABLE)
                    type = "*/*"
                    // Show zips whose provider labels them generically, hide the rest.
                    putExtra(Intent.EXTRA_MIME_TYPES, arrayOf(
                        "application/zip", "application/x-zip-compressed", "application/octet-stream"
                    ))
                    addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
                }
                withContext(Dispatchers.Main) { startActivityForResult(invoke, intent, "importPicked") }
            } catch (e: Exception) {
                invoke.reject("Could not open the file chooser", e)
            }
        }
    }
    @ActivityCallback
    private fun importPicked(invoke: Invoke, result: ActivityResult) {
        CoroutineScope(Dispatchers.IO).launch {
            try {
                if (result.resultCode != Activity.RESULT_OK) {
                    invoke.reject("No file chosen")
                    return@launch
                }
                val uri = result.data?.data ?: run {
                    invoke.reject("No file chosen")
                    return@launch
                }
                val filename = ImportPicker.copy(activity, uri)
                invoke.resolve(JSObject().apply { put("filename", filename) })
            } catch (e: Exception) {
                invoke.reject(e.message ?: "Could not read the chosen file", e)
            }
        }
    }
    @Command fun discardImport(invoke: Invoke) {
        CoroutineScope(Dispatchers.IO).launch {
            try {
                ImportPicker.delete(activity.cacheDir, invoke.getArgs().getString("filename"))
                invoke.resolve(JSObject())
            } catch (e: Exception) {
                invoke.reject("Could not remove the chosen file", e)
            }
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
    // Android can block notifications in its settings even after the permission was granted.
    @Command fun notificationPermission(invoke: Invoke) {
        val manager = activity.getSystemService(NotificationManager::class.java)
        invoke.resolve(JSObject().apply {
            put("granted", manager.areNotificationsEnabled() && (Build.VERSION.SDK_INT < 33 || getPermissionState("notifications") == PermissionState.GRANTED))
        })
    }
    @Command fun requestNotificationPermission(invoke: Invoke) {
        if (Build.VERSION.SDK_INT < 33 || getPermissionState("notifications") == PermissionState.GRANTED) {
            permissionResult(invoke)
        } else requestPermissionForAlias("notifications", invoke, "permissionResult")
    }
    @PermissionCallback fun permissionResult(invoke: Invoke) {
        CoroutineScope(Dispatchers.IO).launch { StatusSurfaces.refresh(activity) }
        invoke.resolve(JSObject().apply {
            put("granted", Build.VERSION.SDK_INT < 33 || getPermissionState("notifications") == PermissionState.GRANTED)
        })
    }
}
