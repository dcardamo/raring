package app.raring

import org.json.JSONObject
import java.time.Instant
import java.time.ZoneId
import java.time.format.DateTimeFormatter

internal data class StatusMetric(val value: Double?, val tone: String) {
    val display: String get() = value?.toInt()?.toString() ?: ""
}

internal data class StatusSnapshot(
    val enabled: Boolean, val asOf: String, val focus: String, val reason: String,
    val target: String, val readiness: StatusMetric, val sleep: StatusMetric,
    val battery: StatusMetric, val stress: StatusMetric, val heartRate: StatusMetric
) {
    companion object {
        private val tones = setOf("good", "fair", "poor", "neutral")
        private val targets = setOf("today", "readiness", "sleep")
        private val stamp = DateTimeFormatter.ofPattern("d MMM, HH:mm")
        private fun metric(json: JSONObject, key: String): StatusMetric {
            val item = json.getJSONObject(key)
            val tone = item.getString("tone").lowercase()
            require(tone in tones)
            require(item.has("value"))
            val rawValue = item.get("value")
            require(rawValue == JSONObject.NULL || rawValue is Number)
            val value = if (rawValue == JSONObject.NULL) null else (rawValue as Number).toDouble()
            require(value == null || (value.isFinite() && value >= 0 && (key == "heartRate" || value <= 100)))
            return StatusMetric(value, tone)
        }
        fun parse(raw: String): StatusSnapshot? = try {
            val json = JSONObject(raw)
            require(json.get("version") == 1)
            require(json.get("enabled") is Boolean)
            require(json.getString("phase") in setOf("overview", "wake", "day", "evening", "windDown", "lateNight"))
            // An absent upload time is not a fresh reading. Never substitute generatedAt.
            val uploaded = runCatching { json.optString("asOf").takeIf { it.isNotBlank() && it != "null" }
                ?.let { Instant.parse(it).atZone(ZoneId.systemDefault()).format(stamp) } }.getOrNull() ?: "unavailable"
            Instant.parse(json.getString("generatedAt"))
            java.time.LocalDate.parse(json.getString("date"))
            val target = json.getString("focusTarget")
            require(target in targets)
            StatusSnapshot(json.getBoolean("enabled"), "As of $uploaded", json.getString("focusLine"),
                json.optString("focusReason").takeUnless { it == "null" } ?: "", target,
                metric(json, "readiness"), metric(json, "sleep"), metric(json, "bodyBattery"),
                metric(json, "stress"), metric(json, "heartRate"))
        } catch (_: Exception) { null }
    }
}
