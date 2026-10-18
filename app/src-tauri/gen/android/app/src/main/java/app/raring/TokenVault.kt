package app.raring

import android.content.Context
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import java.io.File
import java.security.KeyStore
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec

/** The key never leaves AndroidKeyStore. The ciphertext is never backed up. */
object TokenVault {
    private const val ALIAS = "app.raring.tokens.v1"
    private val aad = ALIAS.toByteArray(Charsets.UTF_8)
    private fun file(context: Context) = File(context.noBackupFilesDir, "tokens.enc")
    fun exists(context: Context): Boolean = file(context).isFile
    private fun key(): SecretKey {
        val store = KeyStore.getInstance("AndroidKeyStore").apply { load(null) }
        (store.getKey(ALIAS, null) as? SecretKey)?.let { return it }
        val spec = KeyGenParameterSpec.Builder(ALIAS, KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT)
            .setKeySize(256).setBlockModes(KeyProperties.BLOCK_MODE_GCM)
            .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE).build()
        return KeyGenerator.getInstance(KeyProperties.KEY_ALGORITHM_AES, "AndroidKeyStore")
            .apply { init(spec) }.generateKey()
    }
    @Synchronized fun save(context: Context, value: String) {
        val cipher = Cipher.getInstance("AES/GCM/NoPadding")
        cipher.init(Cipher.ENCRYPT_MODE, key())
        cipher.updateAAD(aad)
        check(cipher.iv.size == 12)
        val bytes = byteArrayOf(1) + cipher.iv + cipher.doFinal(value.toByteArray(Charsets.UTF_8))
        val tmp = File(context.noBackupFilesDir, "tokens.enc.tmp")
        try {
            tmp.outputStream().use { it.write(bytes); it.fd.sync() }
            check(tmp.renameTo(file(context)))
        } finally { tmp.delete() }
    }
    @Synchronized fun load(context: Context): String? {
        val bytes = file(context).takeIf { it.isFile }?.readBytes() ?: return null
        require(bytes.size >= 30 && bytes[0] == 1.toByte())
        val cipher = Cipher.getInstance("AES/GCM/NoPadding")
        cipher.init(Cipher.DECRYPT_MODE, key(), GCMParameterSpec(128, bytes.copyOfRange(1, 13)))
        cipher.updateAAD(aad)
        return String(cipher.doFinal(bytes.copyOfRange(13, bytes.size)), Charsets.UTF_8)
    }
    @Synchronized fun clear(context: Context) {
        val stored = file(context)
        if (stored.exists() && !stored.delete()) {
            // A retained ciphertext must not become a usable session on a later
            // migration. The platform session tombstone also blocks promotion.
            KeyStore.getInstance("AndroidKeyStore").apply { load(null); deleteEntry(ALIAS) }
            error("Could not remove stored credentials")
        }
    }
}
