package app.raring

import androidx.work.WorkInfo
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test
import java.util.UUID

class SyncCompletionTest {
    private fun info(state: WorkInfo.State, attempts: Int = 0) =
        WorkInfo(UUID.randomUUID(), state, emptySet(), runAttemptCount = attempts)

    @Test fun waitsForQueuedAndBlockedWork() {
        assertFalse(info(WorkInfo.State.ENQUEUED).isSyncAttemptComplete())
        assertFalse(info(WorkInfo.State.BLOCKED).isSyncAttemptComplete())
    }

    @Test fun runningIsNotARetryEvenOnTheFirstAttempt() {
        // WorkManager increments the count when work starts, not when it fails.
        assertFalse(info(WorkInfo.State.RUNNING, 1).isSyncAttemptComplete())
        assertFalse(info(WorkInfo.State.RUNNING, 2).isSyncAttemptComplete())
    }

    @Test fun stopsWaitingWhenAnAttemptIsRescheduled() {
        assertTrue(info(WorkInfo.State.ENQUEUED, 1).isSyncAttemptComplete())
        assertTrue(info(WorkInfo.State.ENQUEUED, 2).isSyncAttemptComplete())
    }

    @Test fun stopsWaitingForEveryTerminalState() {
        for (state in listOf(WorkInfo.State.SUCCEEDED, WorkInfo.State.FAILED, WorkInfo.State.CANCELLED)) {
            assertTrue(info(state, 1).isSyncAttemptComplete())
        }
    }
}
