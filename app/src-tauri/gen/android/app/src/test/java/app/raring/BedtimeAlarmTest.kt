package app.raring

import org.junit.Assert.assertEquals
import org.junit.Test
import java.time.LocalDateTime
import java.time.OffsetDateTime
import java.time.ZoneId

class BedtimeAlarmTest {
    private val zone = ZoneId.of("America/Toronto")
    private fun at(local: String) = LocalDateTime.parse(local)
    private fun instant(offset: String) = OffsetDateTime.parse(offset).toInstant()

    @Test fun ordinaryNightsMapDirectly() {
        assertEquals(instant("2026-10-17T22:40-04:00"), BedtimeScheduler.alarmInstant(at("2026-10-17T22:40"), zone, instant("2026-10-17T14:00-04:00")))
    }

    @Test fun fallBackOverlapTakesTheFirstOccurrenceStillAhead() {
        val local = at("2026-11-01T01:30")
        // Before the clocks go back, the daylight-time 01:30 comes first.
        assertEquals(instant("2026-11-01T01:30-04:00"), BedtimeScheduler.alarmInstant(local, zone, instant("2026-11-01T01:10-04:00")))
        // After they go back, that one has passed; the standard-time 01:30 is next.
        assertEquals(instant("2026-11-01T01:30-05:00"), BedtimeScheduler.alarmInstant(local, zone, instant("2026-11-01T01:10-05:00")))
    }

    @Test fun springForwardGapMovesLater() {
        assertEquals(instant("2026-03-08T03:30-04:00"), BedtimeScheduler.alarmInstant(at("2026-03-08T02:30"), zone, instant("2026-03-08T01:00-05:00")))
    }

    @Test fun aPastTimeIsRecheckedInAMinuteRatherThanLooping() {
        val now = instant("2026-10-17T23:00-04:00")
        assertEquals(now.plusSeconds(60), BedtimeScheduler.alarmInstant(at("2026-10-17T22:40"), zone, now))
    }
}
