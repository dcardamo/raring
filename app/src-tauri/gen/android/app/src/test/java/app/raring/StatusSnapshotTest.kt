package app.raring

import org.junit.Assert.*
import org.junit.Test

class StatusSnapshotTest {
    private val snapshot = """{
        "version":1,"enabled":true,"generatedAt":"2025-10-16T14:20:00Z",
        "asOf":"2025-10-16T14:05:00Z","date":"2025-10-16","phase":"day",
        "focusLine":"Push","focusReason":null,"focusTarget":"readiness",
        "readiness":{"value":82,"tone":"good"},"sleep":{"value":null,"tone":"neutral"},
        "bodyBattery":{"value":54,"tone":"fair"},"stress":{"value":31,"tone":"neutral"},
        "heartRate":{"value":null,"tone":"neutral"}
    }"""
    @Test fun valuesAndNulls() {
        val parsed = StatusSnapshot.parse(snapshot)!!
        assertEquals("82", parsed.readiness.display)
        assertEquals("", parsed.sleep.display)
        assertEquals("", parsed.heartRate.display)
        assertTrue(parsed.asOf.startsWith("As of "))
        assertEquals("", parsed.reason)
    }
    @Test fun unsupportedAndMalformedSnapshotsBlank() {
        assertNull(StatusSnapshot.parse(snapshot.replace("\"version\":1", "\"version\":2")))
        assertNull(StatusSnapshot.parse("{"))
        assertNull(StatusSnapshot.parse(snapshot.replace("\"value\":null", "\"value\":-1")))
        assertNull(StatusSnapshot.parse(snapshot.replace("\"focusTarget\":\"readiness\"", "\"focusTarget\":\"external\"")))
        assertEquals("As of unavailable", StatusSnapshot.parse(snapshot.replace("2025-10-16T14:05:00Z", "invalid"))!!.asOf)
    }
    @Test fun zeroIsNotMissingAndHeartRateIsNotAScore() {
        val zero = StatusSnapshot.parse(snapshot.replace("\"value\":82", "\"value\":0"))!!
        assertEquals("0", zero.readiness.display)
        val heart = StatusSnapshot.parse(snapshot.replace("\"heartRate\":{\"value\":null", "\"heartRate\":{\"value\":134"))!!
        assertEquals("134", heart.heartRate.display)
        assertNull(StatusSnapshot.parse(snapshot.replace("\"value\":82", "\"value\":182")))
    }
    @Test fun routesAndPermissionGate() {
        assertEquals("metric/bodyBattery", safeStatusTarget("metric/bodyBattery"))
        assertEquals("metric/stress", safeStatusTarget("metric/stress"))
        assertEquals("sleep", safeStatusTarget("sleep"))
        assertEquals("sleep/last", safeStatusTarget("sleep/last"))
        assertEquals("today", safeStatusTarget("https://example.com"))
        assertFalse(shouldPostStatus(false, true))
        assertFalse(shouldPostStatus(true, false))
        assertTrue(shouldPostStatus(true, true))
    }
}
