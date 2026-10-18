package app.raring

import android.content.Context
import java.io.File
import org.json.JSONObject
import kotlinx.coroutines.sync.withLock

/** Platform boundary only: Rust verifies identity; SyncGate owns every credential change. */
internal object AccountSession {
    @Volatile private var suspended = false
    private fun policy(context: Context) = context.getSharedPreferences("credential-session", Context.MODE_PRIVATE)
    fun db(context: Context) = File(context.filesDir, "raring.db").absolutePath
    private fun blocked(context: Context) = suspended || policy(context).getBoolean("suspended", false)
    fun active(context: Context): Boolean = try {
        val state = JSONObject(NativeBridge.accountSession(db(context), "state", "", ""))
        val fixture = BuildConfig.DEBUG && policy(context).getBoolean("demo", false) && state.optBoolean("demo")
        !blocked(context) && (fixture || TokenVault.exists(context)) && state.optBoolean("active")
    } catch (_: Exception) { false }
    fun isDemo(context: Context): Boolean = BuildConfig.DEBUG && policy(context).getBoolean("demo", false) && active(context)
    fun loadLocked(context: Context): String? {
        if (blocked(context)) return null
        return try { TokenVault.load(context) } catch (_: Exception) {
            suspendLocked(context)
            null
        }
    }
    fun migrationAllowed(context: Context): Boolean = try {
        !blocked(context) && TokenVault.exists(context) &&
            JSONObject(NativeBridge.accountSession(db(context), "state", "", "")).optBoolean("migrationAllowed")
    } catch (_: Exception) { false }

    // Called with SyncGate held. This tombstone contains no credentials; it must
    // survive a failed vault deletion or SQLite suspension, including restart.
    fun suspendLocked(context: Context) {
        suspended = true
        StatusSurfaces.suspendSession(context)
        BedtimeScheduler.cancel(context)
        var failure: Exception? = null
        try { check(policy(context).edit().putBoolean("suspended", true).putBoolean("demo", false).commit()) { "Could not persist session suspension" } }
        catch (error: Exception) { failure = error }
        try { checkResult(NativeBridge.accountSession(db(context), "suspend", "", "")) }
        catch (error: Exception) { failure = error }
        try { TokenVault.clear(context) }
        catch (error: Exception) { failure = error }
        failure?.let { throw it }
    }
    // Existing synthetic UI fixtures remain explicit and emulator-only, with no token saved.
    fun useDemoLocked(context: Context) {
        check(BuildConfig.DEBUG && (android.os.Build.HARDWARE == "ranchu" || android.os.Build.HARDWARE == "goldfish"))
        check(!TokenVault.exists(context))
        check(JSONObject(NativeBridge.accountSession(db(context), "state", "", "")).optBoolean("demo"))
        check(policy(context).edit().putBoolean("demo", true).putBoolean("suspended", false).commit())
        suspended = false
    }

    suspend fun beginSignIn(context: Context): Long = SyncGate.mutex.withLock {
        val generation = SyncGate.invalidateSignInLocked()
        suspendLocked(context)
        generation
    }

    suspend fun signOut(context: Context) = SyncGate.mutex.withLock {
        SyncGate.invalidateSignInLocked()
        forgetLocked(context)
    }

    // Ticket exchange intentionally runs without SyncGate. Carry the captured
    // attempt, then check it under the gate before even suspending a session.
    suspend fun exchangeSignIn(context: Context, ticket: String, generation: Long,
        testTokenApi: String = "", testProfileApi: String = ""): Boolean {
        val response = try { checkResult(NativeBridge.exchangeTicket(ticket, testTokenApi)) }
        catch (error: Exception) {
            return SyncGate.mutex.withLock {
                if (!SyncGate.isCurrentSignInLocked(generation)) false else throw error
            }
        }
        return SyncGate.mutex.withLock {
            if (!SyncGate.isCurrentSignInLocked(generation)) return@withLock false
            verifyAndSaveLocked(context, response.getString("tokensJson"), testProfileApi)
            true
        }
    }

    fun verifyAndSaveLocked(context: Context, candidate: String, testApi: String = "") {
        suspendLocked(context)
        // Candidate credentials live only in this call until the fresh profile is verified.
        val verified = checkResult(NativeBridge.accountSession(db(context), "verify", candidate, testApi))
        try {
            TokenVault.save(context, verified.getString("tokensJson"))
            checkResult(NativeBridge.accountSession(db(context), "activate", "", ""))
            check(policy(context).edit().putBoolean("suspended", false).commit()) { "Could not activate the session" }
            suspended = false
            StatusSurfaces.resumeSession(context)
        } catch (error: Exception) {
            suspendLocked(context)
            throw error
        }
    }

    fun forgetLocked(context: Context) {
        suspendLocked(context)
        checkResult(NativeBridge.accountSession(db(context), "forget", "", ""))
    }

    private fun checkResult(raw: String): JSONObject {
        val result = JSONObject(raw)
        check(!result.has("error")) { result.optString("error", "Account verification failed") }
        return result
    }
}
