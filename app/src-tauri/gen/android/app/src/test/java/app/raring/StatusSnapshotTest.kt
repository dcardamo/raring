package app.raring

import org.junit.Assert.*
import org.junit.Test

class StatusSnapshotTest {
    private val snapshot = """{
        "version":2,"generatedAt":"2025-10-16T14:20:00Z",
        "asOf":"2025-10-16T14:05:00Z","date":"2025-10-16","phase":"day",
        "focusLine":"Push","focusTarget":"readiness",
        "readiness":{"value":82,"tone":"good"},"sleep":{"value":null,"tone":"neutral"},
        "hrv":{"value":58,"status":"Balanced","tone":"good"}
    }"""
    @Test fun valuesAndNulls() {
        val parsed = StatusSnapshot.parse(snapshot)!!
        assertEquals("82", parsed.readiness.display)
        assertEquals("", parsed.sleep.display)
        assertEquals("58", parsed.hrv.display)
        assertEquals("Balanced", parsed.hrvStatus)
        assertTrue(parsed.asOf.startsWith("As of "))
        val noHrv = StatusSnapshot.parse(snapshot.replace("{\"value\":58,\"status\":\"Balanced\",\"tone\":\"good\"}",
            "{\"value\":null,\"status\":null,\"tone\":\"neutral\"}"))!!
        assertEquals("", noHrv.hrv.display)
        assertEquals("", noHrv.hrvStatus)
    }
    @Test fun unsupportedAndMalformedSnapshotsBlank() {
        // Version 1 carried Body Battery and stress instead of HRV.
        assertNull(StatusSnapshot.parse(snapshot.replace("\"version\":2", "\"version\":1")))
        assertNull(StatusSnapshot.parse("{"))
        assertNull(StatusSnapshot.parse(snapshot.replace("\"value\":null", "\"value\":-1")))
        assertNull(StatusSnapshot.parse(snapshot.replace("\"focusTarget\":\"readiness\"", "\"focusTarget\":\"external\"")))
        assertEquals("As of unavailable", StatusSnapshot.parse(snapshot.replace("2025-10-16T14:05:00Z", "invalid"))!!.asOf)
    }
    @Test fun zeroIsNotMissingAndHrvIsNotAScore() {
        val zero = StatusSnapshot.parse(snapshot.replace("\"value\":82", "\"value\":0"))!!
        assertEquals("0", zero.readiness.display)
        assertNull(StatusSnapshot.parse(snapshot.replace("\"value\":82", "\"value\":182")))
        assertEquals("112", StatusSnapshot.parse(snapshot.replace("\"value\":58", "\"value\":112"))!!.hrv.display)
    }
    @Test fun routes() {
        assertEquals("metric/hrv", safeStatusTarget("metric/hrv"))
        assertEquals("sleep", safeStatusTarget("sleep"))
        assertEquals("sleep/last", safeStatusTarget("sleep/last"))
        assertEquals("today", safeStatusTarget("metric/stress"))
        assertEquals("today", safeStatusTarget("https://example.com"))
    }
}
