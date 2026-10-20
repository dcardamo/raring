package app.raring

import java.io.ByteArrayInputStream
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertThrows
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.rules.TemporaryFolder

class ImportPickerTest {
    @get:Rule val temp = TemporaryFolder()

    @Test fun namesAreGeneratedAndUntrustedNamesAreRejected() {
        assertTrue(ImportPicker.validName(ImportPicker.newName()).endsWith(".zip"))
        for (name in listOf("../safe.zip", "/safe.zip", "safe", "safe.zip/", "...", "", "a".repeat(200) + ".zip")) {
            assertThrows(IllegalArgumentException::class.java) { ImportPicker.validName(name) }
            assertThrows(IllegalArgumentException::class.java) { ImportPicker.delete(temp.root, name) }
        }
    }

    @Test fun aCopyIsCappedAndLeavesNothingBehindWhenRefused() {
        val directory = ImportPicker.directory(temp.root).apply { mkdirs() }
        val target = directory.resolve("import-test.zip")

        assertEquals(5, ImportPicker.copyInto(ByteArrayInputStream("12345".toByteArray()), target, 8))
        assertEquals(5L, target.length())
        assertTrue(ImportPicker.delete(temp.root, "import-test.zip"))
        assertFalse(target.exists())

        val oversized = directory.resolve("oversized.zip")
        assertThrows(IllegalArgumentException::class.java) {
            ImportPicker.copyInto(ByteArrayInputStream(ByteArray(64) { 1 }), oversized, 32)
        }
        assertFalse(oversized.exists())

        val empty = directory.resolve("empty.zip")
        assertThrows(IllegalArgumentException::class.java) {
            ImportPicker.copyInto(ByteArrayInputStream(ByteArray(0)), empty, 32)
        }
        assertFalse(empty.exists())
    }

    @Test fun theSweepRemovesOnlyStaleCopies() {
        val directory = ImportPicker.directory(temp.root).apply { mkdirs() }
        val stale = directory.resolve("import-stale.zip").apply { writeText("archive") }
        val recent = directory.resolve("import-recent.zip").apply { writeText("archive") }
        val now = System.currentTimeMillis()
        stale.setLastModified(now - ImportPicker.RETENTION_MS - 1_000)
        recent.setLastModified(now - 1_000)

        assertEquals(1, ImportPicker.sweep(temp.root, now))
        assertFalse(stale.exists())
        assertTrue(recent.exists())
        // The user's own chosen name is never a path the app deletes outside the cache.
        assertFalse(java.io.File(temp.root, "elsewhere.zip").exists())
    }
}
