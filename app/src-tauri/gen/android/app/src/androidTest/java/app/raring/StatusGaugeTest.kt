package app.raring

import androidx.test.ext.junit.runners.AndroidJUnit4
import org.junit.Assert.*
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class StatusGaugeTest {
    @Test fun settingsCanClearWhileSyncIsLocked() {
        val context = androidx.test.core.app.ApplicationProvider.getApplicationContext<android.content.Context>()
        // No vault is installed on the emulator. An opt-out/signed-out render
        // must return immediately even while a sync owns the network lock.
        assertTrue(SyncGate.mutex.tryLock())
        try { StatusSurfaces.applySettings(context) } finally { SyncGate.mutex.unlock() }
        val manager = context.getSystemService(android.app.NotificationManager::class.java)
        assertFalse(manager.activeNotifications.any { it.id == 7002 })
    }
    @Test fun missingReadingDrawsOnlyTrack() {
        val missing = StatusSurfaces.gauge(StatusMetric(null, "neutral"))
        val present = StatusSurfaces.gauge(StatusMetric(82.0, "good"))
        // Top of the arc is colored only with a real reading; its track remains for null.
        assertNotEquals(0, missing.getPixel(56, 8))
        assertNotEquals(missing.getPixel(56, 8), present.getPixel(56, 8))
        assertEquals(0, missing.getPixel(56, 52))
    }
}
