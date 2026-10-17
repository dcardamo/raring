package app.raring

import android.app.Activity
import android.content.ClipData
import android.content.Context
import android.content.Intent
import androidx.core.content.FileProvider
import androidx.work.Data
import androidx.work.ExistingWorkPolicy
import androidx.work.OneTimeWorkRequestBuilder
import androidx.work.Worker
import androidx.work.WorkerParameters
import androidx.work.WorkManager
import java.io.File
import java.util.concurrent.TimeUnit

internal object ExportShare {
    internal const val RETENTION_MS = 24 * 60 * 60 * 1000L
    internal const val FILENAME = "filename"
    private val archiveName = Regex("[A-Za-z0-9][A-Za-z0-9._-]{0,119}\\.zip")

    private fun validName(filename: String): String {
        require(archiveName.matches(filename)) { "invalid export filename" }
        return filename
    }

    internal fun cleanupWorkName(filename: String) = "export-cleanup-${validName(filename)}"

    // Register a durable, filename-scoped deadline before Rust creates the partial ZIP.
    // Waiting for WorkManager's enqueue result prevents a crash from leaving an unscheduled file.
    fun prepare(context: Context, filename: String) {
        val name = validName(filename)
        val cleanup = OneTimeWorkRequestBuilder<ExportCleanupWorker>()
            .setInputData(Data.Builder().putString(FILENAME, name).build())
            .setInitialDelay(RETENTION_MS, TimeUnit.MILLISECONDS)
            .build()
        WorkManager.getInstance(context)
            .enqueueUniqueWork(cleanupWorkName(name), ExistingWorkPolicy.KEEP, cleanup)
            .result.get()
    }

    // The worker runs once per basename, removing whichever stage exists at the deadline.
    // No mtime check: a file published seconds after preparation must still be removed.
    internal fun cleanPrepared(cacheDir: File, filename: String): Boolean {
        val name = validName(filename)
        val directory = File(cacheDir, "exports")
        return listOf(name, "$name.partial").all { leaf ->
            val file = File(directory, leaf)
            !file.exists() || (file.isFile && file.delete())
        }
    }

    fun share(activity: Activity, filename: String) {
        val name = validName(filename)
        val directory = File(activity.cacheDir, "exports").canonicalFile
        val file = File(directory, name)
        require(file.canonicalFile == file.absoluteFile && file.isFile && file.length() > 0) {
            "export archive unavailable"
        }
        val uri = FileProvider.getUriForFile(activity, "${activity.packageName}.export", file)
        val intent = Intent(Intent.ACTION_SEND).apply {
            type = "application/zip"
            putExtra(Intent.EXTRA_STREAM, uri)
            clipData = ClipData.newRawUri(name, uri)
            addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
        }
        // Recipients may read the URI after the chooser closes or an IPC response fails.
        activity.startActivity(Intent.createChooser(intent, "Share Raring export"))
    }
}

class ExportCleanupWorker(context: Context, params: WorkerParameters): Worker(context, params) {
    override fun doWork(): Result {
        val name = inputData.getString(ExportShare.FILENAME) ?: return Result.failure()
        return try {
            if (ExportShare.cleanPrepared(applicationContext.cacheDir, name)) Result.success()
            else Result.retry()
        } catch (_: IllegalArgumentException) {
            Result.failure()
        }
    }
}
