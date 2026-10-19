package app.raring

import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class DemoSyncTest {
    private fun skips(debug: Boolean = true, policy: Boolean = true, state: Boolean = true, active: Boolean = true, tokens: Boolean = false) =
        AccountSession.syncSkipsDemo(debug, policy, state, active, tokens)

    @Test fun onlyTheActiveDebugDemoIsSkipped() {
        assertTrue(skips())
    }

    @Test fun everyOtherSessionStillReachesTheCredentialCheck() {
        // Each of these falls through to loadLocked, which suspends a session without tokens.
        assertFalse("release build", skips(debug = false))
        assertFalse("demo policy off", skips(policy = false))
        assertFalse("not a demo account", skips(state = false))
        assertFalse("suspended or inactive", skips(active = false))
        assertFalse("a real signed-in session", skips(tokens = true))
        assertFalse("release real session", skips(debug = false, policy = false, state = false, tokens = true))
    }
}
