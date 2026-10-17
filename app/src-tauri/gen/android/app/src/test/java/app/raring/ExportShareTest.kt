package app.raring

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertThrows
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.rules.TemporaryFolder

class ExportShareTest {
    @get:Rule val temp = TemporaryFolder()

    @Test fun cleanupIsScopedToThePreparedName() {
        val exports = temp.newFolder("exports")
        // The partial was never published because the app died before share().
        val crashed = exports.resolve("crashed.zip.partial").apply { writeText("incomplete") }
        val finished = exports.resolve("finished.zip").apply { writeText("archive") }
        val newer = exports.resolve("newer.zip").apply { writeText("other archive") }
        val newerPartial = exports.resolve("newer.zip.partial").apply { writeText("other partial") }

        assertTrue(ExportShare.cleanPrepared(temp.root, "crashed.zip"))
        assertFalse(crashed.exists())
        assertTrue(finished.exists())
        assertTrue(newer.exists())
        assertTrue(newerPartial.exists())

        assertTrue(ExportShare.cleanPrepared(temp.root, "finished.zip"))
        assertFalse(finished.exists())
        assertTrue(newer.exists())
        assertTrue(newerPartial.exists())
        assertEquals("export-cleanup-newer.zip", ExportShare.cleanupWorkName("newer.zip"))
    }

    @Test fun cleanupRejectsUntrustedNamesAndNeverDeletesOtherFiles() {
        val exports = temp.newFolder("exports")
        val unrelated = exports.resolve("safe.zip").apply { writeText("archive") }
        for (name in listOf("../safe.zip", "/safe.zip", "safe.zip.partial", "...", "")) {
            assertThrows(IllegalArgumentException::class.java) {
                ExportShare.cleanPrepared(temp.root, name)
            }
            assertThrows(IllegalArgumentException::class.java) {
                ExportShare.cleanupWorkName(name)
            }
        }
        assertTrue(unrelated.exists())
    }
}
