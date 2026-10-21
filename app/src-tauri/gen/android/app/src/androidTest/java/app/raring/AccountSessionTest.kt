package app.raring

import android.Manifest
import android.content.Context
import android.database.sqlite.SQLiteDatabase
import android.os.Build
import androidx.test.core.app.ApplicationProvider
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import java.io.File
import java.net.ServerSocket
import java.time.Instant
import java.util.concurrent.atomic.AtomicInteger
import java.util.concurrent.CountDownLatch
import java.util.concurrent.Executors
import java.util.concurrent.TimeUnit
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.sync.withLock
import org.json.JSONObject
import org.junit.After
import org.junit.Assert.*
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith

/** Synthetic accounts only, through the production SyncGate and the one encrypted vault. */
@RunWith(AndroidJUnit4::class)
class AccountSessionTest {
    private val context: Context = ApplicationProvider.getApplicationContext()
    private var isolated = false
    private val dbPath get() = AccountSession.db(context)
    private val snapshot get() = File(context.filesDir, "raring.status.json")
    private val settings = JSONObject("""{"notifyMorningReadiness":false,"notifyWeeklyCheckin":false,"notifyMetricChanges":false,"notifyVitals":false,"notifyTrainingStatus":false,"notifyBedtime":false,"bedtimeLeadMin":30,"adaptiveToday":true}""")

    private fun tokens(account: String) = JSONObject().put("access_token", "synthetic-$account")
        .put("refresh_token", "synthetic-refresh-$account").put("client_id", "synthetic-client")
        .put("expires_at", Instant.now().plusSeconds(86400).toString()).toString()
    private fun call(action: String, value: String = "") = JSONObject(NativeBridge.accountSession(dbPath, action, value, ""))
    private fun sql(block: (SQLiteDatabase) -> Unit) { SQLiteDatabase.openDatabase(dbPath, null, SQLiteDatabase.OPEN_READWRITE).use(block) }
    private fun setting(key: String): String? {
        var value: String? = null
        sql { db -> db.rawQuery("SELECT value FROM settings WHERE key=?", arrayOf(key)).use { if (it.moveToFirst()) value = it.getString(0) } }
        return value
    }
    private fun seedHistory() = sql { db ->
        db.execSQL("INSERT OR REPLACE INTO raw VALUES ('summary','2025-03-12','synthetic',?)", arrayOf("""{"bodyBatteryMostRecentValue":88}"""))
        db.execSQL("INSERT OR REPLACE INTO settings VALUES ('plan','synthetic-targets-A')")
        db.execSQL("INSERT OR REPLACE INTO settings VALUES ('sleep_plan:2025-03-12','synthetic-plan-A')")
        db.execSQL("INSERT OR REPLACE INTO cursors VALUES ('synthetic-A','cursor-A')")
        db.execSQL("INSERT OR REPLACE INTO notifications VALUES ('morning','synthetic-A','synthetic')")
    }
    private fun bind(id: Int, account: String) {
        ProfileServer(200, """{"profileId":$id,"displayName":"fake-$account"}""").use { server ->
            runBlocking { SyncGate.mutex.withLock { AccountSession.verifyAndSaveLocked(context, tokens(account), server.url) } }
            assertEquals(1, server.requests.get())
            assertEquals("Bearer synthetic-$account", server.authorization)
        }
    }

    @Before fun isolate() {
        check(Build.HARDWARE == "ranchu" || Build.HARDWARE == "goldfish") { "Use an isolated emulator" }
        check(context.packageName == "app.raring.debug")
        check(!TokenVault.exists(context)) { "Refusing to replace installed credentials" }
        isolated = true
        androidx.work.WorkManager.getInstance(context).cancelAllWork().result.get()
        InstrumentationRegistry.getInstrumentation().uiAutomation.executeShellCommand("pm grant ${context.packageName} ${Manifest.permission.POST_NOTIFICATIONS}").close()
        runBlocking { SyncGate.mutex.withLock { AccountSession.forgetLocked(context) } }
    }
    @After fun clean() {
        if (!isolated) return
        snapshot.takeIf { it.isDirectory }?.delete()
        File(context.filesDir, "status-test-old.json").delete()
        sql { it.execSQL("DROP TRIGGER IF EXISTS fail_session_suspend") }
        runBlocking { SyncGate.mutex.withLock { AccountSession.forgetLocked(context) } }
    }

    @Test fun lateTicketExchangeCannotReplaceTheNewerAccount() {
        TicketServer("A", blocked = true).use { ticketA ->
            TicketServer("B").use { ticketB ->
                ProfileServer(200, """{"profileId":101,"displayName":"fake-A"}""").use { profileA ->
                    ProfileServer(200, """{"profileId":202,"displayName":"fake-B"}""").use { profileB ->
                        val executor = Executors.newSingleThreadExecutor()
                        try {
                            val generationA = runBlocking { AccountSession.beginSignIn(context) }
                            val lateA = executor.submit<Boolean> { runBlocking {
                                AccountSession.exchangeSignIn(context, "ST-synthetic-A", generationA, ticketA.url, profileA.url)
                            } }
                            assertTrue("A must be paused inside the actual HTTP ticket exchange", ticketA.requested.await(10, TimeUnit.SECONDS))
                            val generationB = runBlocking { AccountSession.beginSignIn(context) }
                            assertTrue(runBlocking { AccountSession.exchangeSignIn(context, "ST-synthetic-B", generationB, ticketB.url, profileB.url) })
                            sql { db ->
                                db.execSQL("INSERT OR REPLACE INTO settings VALUES ('plan','targets-B')")
                                db.execSQL("INSERT OR REPLACE INTO cursors VALUES ('synthetic-B','cursor-B')")
                                db.execSQL("INSERT OR REPLACE INTO raw VALUES ('summary','2025-03-12','synthetic',?)", arrayOf("""{"owner":"B","bodyBatteryMostRecentValue":77}"""))
                            }
                            assertFalse(call("testSettings", settings.toString()).has("error"))
                            val vaultBefore = File(context.noBackupFilesDir, "tokens.enc").readBytes()
                            val snapshotBefore = snapshot.readBytes()
                            val flagsBefore = setting("settings")
                            ticketA.release.countDown()
                            assertFalse("stale ticket must be discarded", lateA.get(10, TimeUnit.SECONDS))
                            assertEquals("no stale profile verification", 0, profileA.requests.get())
                            assertEquals(1, profileB.requests.get())
                            assertEquals("ST-synthetic-A", ticketA.ticket)
                            assertEquals("ST-synthetic-B", ticketB.ticket)
                            assertTrue(AccountSession.active(context))
                            assertEquals("202", call("state").getString("identity"))
                            assertArrayEquals(vaultBefore, File(context.noBackupFilesDir, "tokens.enc").readBytes())
                            assertArrayEquals(snapshotBefore, snapshot.readBytes())
                            assertEquals(flagsBefore, setting("settings"))
                            assertEquals("targets-B", setting("plan"))
                            sql { db ->
                                db.rawQuery("SELECT value FROM cursors WHERE key='synthetic-B'", null).use { assertTrue(it.moveToFirst()); assertEquals("cursor-B", it.getString(0)) }
                                db.rawQuery("SELECT json FROM raw WHERE endpoint='summary'", null).use { assertTrue(it.moveToFirst()); assertEquals("B", JSONObject(it.getString(0)).getString("owner")) }
                            }
                        } finally { ticketA.release.countDown(); executor.shutdownNow() }
                    }
                }
            }
        }
    }

    @Test fun signOutInvalidatesAnExchangeAlreadyInFlight() {
        TicketServer("A", blocked = true).use { ticket ->
            ProfileServer(200, """{"profileId":101,"displayName":"fake-A"}""").use { profile ->
                val executor = Executors.newSingleThreadExecutor()
                try {
                    val generation = runBlocking { AccountSession.beginSignIn(context) }
                    val late = executor.submit<Boolean> { runBlocking {
                        AccountSession.exchangeSignIn(context, "ST-synthetic-A", generation, ticket.url, profile.url)
                    } }
                    assertTrue(ticket.requested.await(10, TimeUnit.SECONDS))
                    runBlocking { AccountSession.signOut(context) }
                    ticket.release.countDown()
                    assertFalse(late.get(10, TimeUnit.SECONDS))
                    assertEquals(0, profile.requests.get())
                    assertFalse(TokenVault.exists(context))
                    assertFalse(AccountSession.active(context))
                    assertNull(setting("account_session"))
                } finally { ticket.release.countDown(); executor.shutdownNow() }
            }
        }
    }

    @Test fun databaseSuspensionFailureBlocksEveryPlatformConsumer() {
        bind(101, "A")
        seedHistory()
        sql { it.execSQL("CREATE TRIGGER fail_session_suspend BEFORE UPDATE ON settings WHEN NEW.key='account_session' AND json_extract(NEW.value, '$.active')=0 BEGIN SELECT RAISE(ABORT, 'synthetic storage failure'); END") }
        try {
            runBlocking { SyncGate.mutex.withLock { AccountSession.suspendLocked(context) } }
            fail("Suspension failure must be reported")
        } catch (_: IllegalStateException) { }
        assertTrue(call("state").getBoolean("active")) // SQLite deliberately still contains the old active state.
        assertFalse(TokenVault.exists(context))
        assertFalse(AccountSession.active(context)) // Used by Tauri sessionState, views, bedtime and surfaces.
        assertFalse(AccountSession.migrationAllowed(context))
        assertTrue(context.getSharedPreferences("credential-session", Context.MODE_PRIVATE).getBoolean("suspended", false))
        StatusSurfaces.applySettings(context)
        BedtimeScheduler.run(context)
        assertEquals("synthetic-targets-A", setting("plan"))
    }

    @Test fun failedVaultDeletionCannotReactivateRetainedCredentials() {
        bind(101, "A")
        seedHistory()
        android.system.Os.chmod(context.noBackupFilesDir.absolutePath, 320) // 0500: retained file cannot be unlinked.
        try {
            try {
                runBlocking { SyncGate.mutex.withLock { AccountSession.suspendLocked(context) } }
                fail("Vault deletion failure must be reported")
            } catch (_: IllegalStateException) { }
            assertTrue(TokenVault.exists(context))
            assertFalse(AccountSession.active(context))
            assertFalse(AccountSession.migrationAllowed(context))
            assertTrue(context.getSharedPreferences("credential-session", Context.MODE_PRIVATE).getBoolean("suspended", false))
            StatusSurfaces.applySettings(context)
            BedtimeScheduler.run(context)
            assertEquals("synthetic-targets-A", setting("plan"))
        } finally { android.system.Os.chmod(context.noBackupFilesDir.absolutePath, 448) } // 0700
        bind(101, "A-reauth")
        assertEquals("synthetic-targets-A", setting("plan"))
    }

    @Test fun differentAccountNeverReceivesOldHistoryOrSnapshot() {
        bind(101, "A")
        seedHistory()
        assertFalse(call("testSettings", settings.toString()).has("error"))
        val old = snapshot.readText()
        assertFalse(call("snapshot").isNull("snapshot"))
        runBlocking { SyncGate.mutex.withLock { AccountSession.suspendLocked(context) } }
        assertFalse(AccountSession.active(context))
        assertTrue(call("snapshot").isNull("snapshot"))
        bind(202, "B")
        assertTrue(AccountSession.active(context))
        val saved = runBlocking { SyncGate.mutex.withLock { TokenVault.load(context) } }
        assertTrue(saved!!.contains("synthetic-B"))
        assertNull(setting("plan"))
        assertNull(setting("sleep_plan:2025-03-12"))
        sql { db ->
            for (query in listOf("SELECT count(*) FROM raw WHERE endpoint!='profile'", "SELECT count(*) FROM activities", "SELECT count(*) FROM cursors", "SELECT count(*) FROM notifications")) {
                db.rawQuery(query, null).use { assertTrue(it.moveToFirst()); assertEquals(0, it.getInt(0)) }
            }
        }
        // Even if cache deletion/publication had failed, A's old file cannot render for B.
        snapshot.writeText(old)
        assertTrue(call("snapshot").isNull("snapshot"))
    }

    @Test fun sameAccountKeepsPlansAndFailedVerificationKeepsHistorySuspended() {
        bind(101, "A")
        seedHistory()
        bind(101, "A-reauth")
        assertEquals("synthetic-targets-A", setting("plan"))
        assertEquals("synthetic-plan-A", setting("sleep_plan:2025-03-12"))
        ProfileServer(503, "{}").use { server ->
            try {
                runBlocking { SyncGate.mutex.withLock { AccountSession.verifyAndSaveLocked(context, tokens("B"), server.url) } }
                fail("Unverified credentials must not be promoted")
            } catch (_: IllegalStateException) { }
            assertEquals(1, server.requests.get())
        }
        assertFalse(TokenVault.exists(context))
        assertFalse(AccountSession.active(context))
        assertEquals("synthetic-targets-A", setting("plan"))
        assertTrue(call("snapshot").isNull("snapshot"))
    }

    @Test fun journalRevocationSurvivesSnapshotFailureAndDelayedUnrelatedPatch() {
        bind(101, "A")
        assertFalse(call("testSettings", settings.toString()).has("error"))
        assertFalse(call("testSettingsPatch", """{"journalEnabled":true}""").has("error"))
        sql { db -> db.execSQL("INSERT OR REPLACE INTO raw VALUES ('journal',?,?,?)", arrayOf(
            java.time.LocalDate.now().toString(), Instant.now().toString(),
            """{"dailyLogsReport":[{"name":"PRIVATE_SYNTHETIC_JOURNAL","logStatus":"YES"}]}"""
        )) }
        StatusSurfaces.refresh(context)
        assertFalse(snapshot.readText().contains("PRIVATE_SYNTHETIC_JOURNAL"))
        val old = File(context.filesDir, "status-test-old.json")
        assertTrue(snapshot.renameTo(old)); assertTrue(snapshot.mkdir())
        val queued = CountDownLatch(1); val release = CountDownLatch(1)
        val executor = Executors.newSingleThreadExecutor()
        try {
            val older = executor.submit<JSONObject> {
                val patch = """{"notifyVitals":true}"""
                queued.countDown(); check(release.await(10, TimeUnit.SECONDS))
                call("testSettingsPatch", patch)
            }
            assertTrue(queued.await(10, TimeUnit.SECONDS))
            assertFalse(call("testSettingsPatch", """{"journalEnabled":false}""").has("error"))
            assertFalse(JSONObject(setting("settings")!!).getBoolean("journalEnabled"))
            release.countDown(); assertFalse(older.get(10, TimeUnit.SECONDS).has("error"))
            val saved = JSONObject(setting("settings")!!)
            assertFalse(saved.getBoolean("journalEnabled")); assertTrue(saved.getBoolean("notifyVitals"))
            assertTrue(snapshot.delete()); assertTrue(old.renameTo(snapshot))
            StatusSurfaces.refresh(context)
            val native = call("snapshot").getJSONObject("snapshot")
            assertFalse(native.has("journal")); assertFalse(native.has("journalEnabled"))
            assertFalse(native.toString().contains("PRIVATE_SYNTHETIC_JOURNAL"))
        } finally { release.countDown(); executor.shutdownNow() }
    }

    private class TicketServer(account: String, blocked: Boolean = false) : AutoCloseable {
        private val socket = ServerSocket(0, 1, java.net.InetAddress.getByName("127.0.0.1"))
        val url = "http://127.0.0.1:${socket.localPort}/token"
        val requested = CountDownLatch(1)
        val release = CountDownLatch(if (blocked) 1 else 0)
        @Volatile var ticket = ""
        private val thread = Thread {
            socket.accept().use { client ->
                client.soTimeout = 10000
                val input = client.getInputStream().bufferedReader()
                check(input.readLine().startsWith("POST /token "))
                var length = 0
                while (true) {
                    val header = input.readLine() ?: break
                    if (header.isEmpty()) break
                    if (header.startsWith("content-length:", true)) length = header.substringAfter(':').trim().toInt()
                }
                val body = CharArray(length)
                var count = 0
                while (count < length) { val n = input.read(body, count, length-count); check(n > 0); count += n }
                ticket = String(body).split('&').single { it.startsWith("service_ticket=") }.substringAfter('=')
                requested.countDown()
                check(release.await(10, TimeUnit.SECONDS))
                val reply = JSONObject().put("access_token", "synthetic-$account").put("refresh_token", "synthetic-refresh-$account").put("expires_in", 3600).toString()
                client.getOutputStream().write("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: ${reply.length}\r\nConnection: close\r\n\r\n$reply".toByteArray())
            }
        }.apply { start() }
        override fun close() { release.countDown(); socket.close(); thread.join(2000) }
    }

    private class ProfileServer(private val status: Int, private val body: String) : AutoCloseable {
        private val socket = ServerSocket(0, 1, java.net.InetAddress.getByName("127.0.0.1"))
        val url = "http://127.0.0.1:${socket.localPort}"
        val requests = AtomicInteger()
        @Volatile var authorization: String = ""
        private val thread = Thread {
            val accepted = try { socket.accept() } catch (e: java.net.SocketException) {
                if (socket.isClosed) return@Thread else throw e
            }
            accepted.use { client ->
                val input = client.getInputStream().bufferedReader()
                check(input.readLine().startsWith("GET /userprofile-service/socialProfile "))
                while (true) {
                    val header = input.readLine() ?: break
                    if (header.isEmpty()) break
                    if (header.startsWith("authorization:", ignoreCase = true)) authorization = header.substringAfter(':').trim()
                }
                requests.incrementAndGet()
                val bytes = body.toByteArray()
                client.getOutputStream().write("HTTP/1.1 $status Synthetic\r\nContent-Type: application/json\r\nContent-Length: ${bytes.size}\r\nConnection: close\r\n\r\n".toByteArray() + bytes)
            }
        }.apply { start() }
        override fun close() { socket.close(); thread.join(2000) }
    }
}
