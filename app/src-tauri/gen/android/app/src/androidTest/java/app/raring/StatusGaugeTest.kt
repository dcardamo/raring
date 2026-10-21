package app.raring

import androidx.test.ext.junit.runners.AndroidJUnit4
import org.junit.Assert.*
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class StatusGaugeTest {
    @Test fun settingsRenderRetiresTheStatusNotificationWhileSyncIsLocked() {
        val context = androidx.test.core.app.ApplicationProvider.getApplicationContext<android.content.Context>()
        androidx.test.platform.app.InstrumentationRegistry.getInstrumentation().uiAutomation
            .executeShellCommand("pm grant ${context.packageName} ${android.Manifest.permission.POST_NOTIFICATIONS}").close()
        val manager = context.getSystemService(android.app.NotificationManager::class.java)
        // What an older version left behind: an ongoing notification on its own channel.
        manager.createNotificationChannel(android.app.NotificationChannel("raring-status", "Status", android.app.NotificationManager.IMPORTANCE_LOW))
        manager.notify(7002, androidx.core.app.NotificationCompat.Builder(context, "raring-status")
            .setSmallIcon(R.drawable.ic_notification).setOngoing(true).build())
        awaitStatusNotification(manager, true)
        // No vault is installed on the emulator. A signed-out render must
        // return immediately even while a sync owns the network lock.
        assertTrue(SyncGate.mutex.tryLock())
        try { StatusSurfaces.applySettings(context) } finally { SyncGate.mutex.unlock() }
        awaitStatusNotification(manager, false)
        assertNull(manager.getNotificationChannel("raring-status"))
    }
    private fun awaitStatusNotification(manager: android.app.NotificationManager, present: Boolean) {
        val deadline = android.os.SystemClock.elapsedRealtime() + 3000
        do {
            android.os.SystemClock.sleep(25) // NotificationManager processes posts/cancels asynchronously.
            if (manager.activeNotifications.any { it.id == 7002 } == present) return
        } while (android.os.SystemClock.elapsedRealtime() < deadline)
        assertEquals(present, manager.activeNotifications.any { it.id == 7002 })
    }
    @Test fun missingReadingDrawsOnlyTrack() {
        val missing = StatusSurfaces.gauge(StatusMetric(null, "neutral"))
        val present = StatusSurfaces.gauge(StatusMetric(82.0, "good"))
        // Top of the arc is colored only with a real reading; its track remains for null.
        assertNotEquals(0, missing.getPixel(40, 8))
        assertNotEquals(missing.getPixel(40, 8), present.getPixel(40, 8))
        assertEquals(0, missing.getPixel(40, 40))
    }
}
