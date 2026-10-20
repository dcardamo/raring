package app.raring

import android.content.Context
import android.net.Uri
import java.io.File
import java.io.FileOutputStream
import java.io.InputStream
import java.util.UUID

// The system picker hands the app a content URI, which the Rust core cannot
// open. A chosen export is copied into the private cache under a generated
// name; the copy is removed after the import or by the next sweep, and the
// core only ever reads that cache file.
internal object ImportPicker {
    internal const val MAX_BYTES = 512L * 1024 * 1024
    internal const val RETENTION_MS = 24 * 60 * 60 * 1000L
    private val archiveName = Regex("[A-Za-z0-9][A-Za-z0-9._-]{0,119}\\.zip")

    internal fun validName(filename: String): String {
        require(archiveName.matches(filename)) { "invalid import filename" }
        return filename
    }

    internal fun directory(cacheDir: File): File = File(cacheDir, "imports")

    internal fun newName(): String = "import-${UUID.randomUUID().toString().replace("-", "")}.zip"

    /// Copy the chosen document into the cache and return its leaf name.
    internal fun copy(context: Context, uri: Uri): String {
        val name = newName()
        val directory = directory(context.cacheDir).apply { mkdirs() }
        val input = context.contentResolver.openInputStream(uri)
            ?: throw IllegalArgumentException("The chosen file can't be read")
        input.use { copyInto(it, File(directory, name), MAX_BYTES) }
        return name
    }

    /// Copy at most `cap` bytes; an oversized or empty document leaves nothing behind.
    internal fun copyInto(input: InputStream, target: File, cap: Long): Long {
        var copied = 0L
        val buffer = ByteArray(64 * 1024)
        try {
            FileOutputStream(target).use { output ->
                while (true) {
                    val read = input.read(buffer)
                    if (read < 0) break
                    copied += read
                    if (copied > cap) throw IllegalArgumentException("This file is too large to import")
                    output.write(buffer, 0, read)
                }
                output.flush()
            }
        } catch (e: Exception) {
            target.delete()
            throw e
        }
        if (copied == 0L) {
            target.delete()
            throw IllegalArgumentException("The chosen file is empty")
        }
        return copied
    }

    internal fun delete(cacheDir: File, filename: String): Boolean {
        val file = File(directory(cacheDir), validName(filename))
        return !file.exists() || (file.isFile && file.delete())
    }

    /// Copies older than the retention window are removed at the next pick,
    /// so an abandoned choice never takes space indefinitely.
    internal fun sweep(cacheDir: File, now: Long = System.currentTimeMillis()): Int {
        val files = directory(cacheDir).listFiles() ?: return 0
        return files.count { file ->
            file.isFile && now - file.lastModified() > RETENTION_MS && file.delete()
        }
    }
}
