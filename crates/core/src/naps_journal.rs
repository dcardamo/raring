//! Read-only, bounded Garmin day snapshots. Unknown response shapes remain unknown.
use crate::{store::Store, view::{JournalDay, JournalEntry, Nap, ReadState}};
use chrono::{NaiveDate, NaiveDateTime, Utc};
use serde_json::Value;

fn state(store: &Store, kind: &str, day: &str, populated: bool, empty: bool) -> ReadState {
    let Some(raw) = store.get(kind, day) else { return ReadState::Unknown };
    if raw.is_null() { return ReadState::Unavailable }
    let age_days = NaiveDate::parse_from_str(day, "%Y-%m-%d").ok()
        .map(|d| (chrono::Local::now().date_naive() - d).num_days());
    let max_hours = match (kind, age_days) {
        (_, Some(0)) => Some(6),
        ("journal", Some(1..=6)) | ("naps", Some(1)) => Some(24),
        _ => None,
    };
    if max_hours.is_some_and(|hours| store.fetched_at(kind, day)
        .and_then(|t| chrono::DateTime::parse_from_rfc3339(&t).ok())
        .is_some_and(|t| Utc::now().signed_duration_since(t).num_hours() >= hours)) { return ReadState::Stale }
    if populated { ReadState::Available } else if empty { ReadState::Empty } else { ReadState::Unavailable }
}

/// Garmin's raw nap endpoint writes each zone offset in minutes (this account
/// sent -240 for UTC−4); its exported sleep summary writes the same zone in
/// seconds. Decide by magnitude: no real zone is more than 14 hours out, so a
/// raw value within that many minutes is minutes and anything larger is seconds.
fn offset_seconds(raw: i64) -> Option<i64> {
    let seconds = if raw.abs() <= 14 * 60 { raw * 60 } else { raw };
    (seconds.abs() <= 14 * 3600).then_some(seconds)
}

pub fn naps(store: &Store, day: &str) -> (Vec<Nap>, ReadState) {
    let raw = store.get("naps", day).unwrap_or(Value::Null);
    let Some(rows) = raw.as_array() else { return (vec![], state(store, "naps", day, false, false)) };
    // GMT wall-clock strings plus Garmin's offsets. Never infer nap duration
    // from the main sleep DTO's napTimeSeconds total.
    let naps: Vec<Nap> = rows.iter().filter_map(|row| {
        if !row.is_object() { return None }
        let wall = |stamp: &str, offset: &str| -> Option<String> {
            let utc = NaiveDateTime::parse_from_str(row[stamp].as_str()?, "%Y-%m-%dT%H:%M:%S").ok()?;
            let seconds = offset_seconds(row[offset].as_i64()?)?;
            Some((utc + chrono::Duration::seconds(seconds)).format("%Y-%m-%dT%H:%M:%S").to_string())
        };
        let start_local = wall("napStartTimestampGMT", "napStartTimeOffset");
        let end_local = wall("napEndTimestampGMT", "napEndTimeOffset");
        let duration_min = row["napTimeSec"].as_f64().filter(|n| n.is_finite() && *n >= 0.).map(|s| s / 60.);
        let feedback = row["napFeedback"].as_str().filter(|s| !s.is_empty() && s.len() <= 80).map(str::to_owned);
        (start_local.is_some() || end_local.is_some() || duration_min.is_some())
            .then_some(Nap { start_local, end_local, duration_min, feedback })
    }).collect();
    let read = state(store, "naps", day, !naps.is_empty(), rows.is_empty());
    (naps, read)
}

pub fn journal(store: &Store, end: &str) -> Vec<JournalDay> {
    let Ok(end) = NaiveDate::parse_from_str(end, "%Y-%m-%d") else { return vec![] };
    (0..7).map(|offset| {
        let date = (end - chrono::Duration::days(offset)).to_string();
        let raw = store.get("journal", &date).unwrap_or(Value::Null);
        let report = &raw["dailyLogsReport"];
        // A null report is not proof of zero entries. Ignore completionStats: it
        // counts tracking configuration, not the day's recorded journal values.
        let entries = report.as_array().map(|rows| rows.iter().filter_map(|row| {
            let name = row["name"].as_str().filter(|s| !s.trim().is_empty())?;
            let value = match row["logStatus"].as_str() {
                Some("YES") => Some("Yes".to_owned()),
                Some("NO") => Some("No".to_owned()),
                _ => None,
            };
            // Garmin's logged quantities have subtype names but no generic unit.
            let detail = row["details"].as_array().map(|details| details.iter().filter_map(|d| {
                let amount = d["amount"].as_f64()?;
                if !amount.is_finite() { return None }
                // Garmin's subtype keys are capitals ("COFFEE"); show them as words.
                let label = d["subTypeName"].as_str().unwrap_or("amount").replace('_', " ").to_lowercase();
                Some(format!("{amount} {label}"))
            }).collect::<Vec<_>>().join(", ")).filter(|s| !s.is_empty());
            Some(JournalEntry { name: name.to_owned(), value: detail.or(value) })
        }).collect::<Vec<_>>()).unwrap_or_default();
        let state = state(store, "journal", &date, !entries.is_empty(), report.as_array().is_some_and(|a| a.is_empty()));
        JournalDay { date, state, entries }
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn absent_empty_multiple_midnight_and_unknown() {
        let db = Store::open(":memory:").unwrap();
        let day = "2025-02-05";
        assert_eq!(naps(&db, day).1, ReadState::Unknown);
        db.put("naps", day, &Utc::now().to_rfc3339(), &serde_json::json!([])).unwrap();
        assert_eq!(naps(&db, day).1, ReadState::Empty);
        db.put("naps", day, &Utc::now().to_rfc3339(), &serde_json::json!([
            {"napStartTimestampGMT":"2025-02-05T01:45:00","napStartTimeOffset":-120,"napEndTimestampGMT":"2025-02-05T02:15:00","napEndTimeOffset":-120,"napTimeSec":1800,"napFeedback":"SYNTHETIC_FEEDBACK"},
            {"napStartTimestampGMT":"2025-02-05T02:15:00","napStartTimeOffset":-7200,"napTimeSec":0}
        ])).unwrap();
        let (rows, state) = naps(&db, day);
        assert_eq!(state, ReadState::Available);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].duration_min, Some(30.));
        assert_eq!(rows[0].feedback.as_deref(), Some("SYNTHETIC_FEEDBACK"));
        assert_eq!(rows[1].duration_min, Some(0.));
        // Garmin's raw endpoint sends minutes; an exported-summary seconds
        // value for the same UTC−2 zone lands at the same local clock.
        assert_eq!(rows[0].start_local.as_deref(), Some("2025-02-04T23:45:00"));
        assert_eq!(rows[0].end_local.as_deref(), Some("2025-02-05T00:15:00"));
        assert_eq!(rows[1].start_local.as_deref(), Some("2025-02-05T00:15:00"));
        db.put("journal", day, &Utc::now().to_rfc3339(), &serde_json::json!({"dailyLogsReport":null,"completionStats":[]})).unwrap();
        let days = journal(&db, day);
        assert_eq!(days[0].state, ReadState::Unavailable);
        assert!(days[0].entries.is_empty());
        db.put("journal", day, &Utc::now().to_rfc3339(), &serde_json::json!({"dailyLogsReport":[{"name":"Test","logStatus":"UNKNOWN","details":[{"amount":null}]}]})).unwrap();
        let days = journal(&db, day);
        assert_eq!(days[0].entries[0].value, None);
        db.put("journal", day, &Utc::now().to_rfc3339(), &serde_json::json!({"dailyLogsReport":[{"name":"Caffeine","logStatus":"YES","details":[{"amount":2,"subTypeName":"COFFEE"},{"amount":1,"subTypeName":"ENERGY_DRINK"}]}]})).unwrap();
        assert_eq!(journal(&db, day)[0].entries[0].value.as_deref(), Some("2 coffee, 1 energy drink"));
        db.put("sleep", day, &Utc::now().to_rfc3339(), &serde_json::json!({"dailySleepDTO":{"sleepTimeSeconds":25200,"sleepStartTimestampLocal":"2025-02-04T22:00:00","sleepEndTimestampLocal":"2025-02-05T05:00:00"}})).unwrap();
        let sleep = crate::screens::sleep_view(&db, day);
        assert_eq!(sleep.asleep_min, Some(420.));
        assert_eq!(sleep.naps.len(), 2); // not added to main sleep or its need
    }
    #[test]
    fn stale_and_disabled_journal_never_look_like_empty_or_a_zero() {
        let db = Store::open(":memory:").unwrap();
        let today = chrono::Local::now().date_naive().to_string();
        let old = (Utc::now() - chrono::Duration::hours(25)).to_rfc3339();
        db.put("naps", &today, &old, &serde_json::json!([])).unwrap();
        assert_eq!(naps(&db, &today).1, ReadState::Stale);
        db.put("journal", &today, &old, &serde_json::json!({"dailyLogsReport":[]})).unwrap();
        assert_eq!(journal(&db, &today)[0].state, ReadState::Stale);
        let health = crate::screens::health(&db, &today);
        assert!(!health.journal_enabled);
        assert!(health.journal.is_empty());
    }

    #[test]
    fn live_health_journal_uses_calendar_day_before_sleep_handover() {
        let db = Store::open(":memory:").unwrap();
        let mut settings = db.settings();
        settings.journal_enabled = true;
        db.set_settings(&settings).unwrap();
        db.put("sleep", "2025-10-16", "synthetic", &serde_json::json!({
            "dailySleepDTO":{"sleepTimeSeconds":25200,"sleepEndTimestampLocal":"2025-10-16T07:00:00"}
        })).unwrap();
        for day in ["2025-10-17", "2025-10-11", "2025-10-10"] {
            db.put_journal_if_enabled(day, "synthetic", &serde_json::json!({
                "dailyLogsReport":[{"name":"Synthetic entry","logStatus":"YES"}]
            })).unwrap();
        }
        let now = "2025-10-17T00:30:00".parse::<chrono::NaiveDateTime>().unwrap();
        let display = crate::screens::current_date(&db, now).to_string();
        assert_eq!(display, "2025-10-16");
        let live = crate::screens::health_at_calendar(&db, &display, &now.date().to_string());
        assert_eq!(live.date, "2025-10-16");
        assert_eq!(live.heart_rate_day.date, "2025-10-16");
        assert_eq!(live.journal.first().unwrap().date, "2025-10-17");
        assert_eq!(live.journal.last().unwrap().date, "2025-10-11");
        assert_eq!(live.journal.len(), 7);
        assert_eq!(live.journal[0].entries[0].name, "Synthetic entry");
        let historical = crate::screens::health(&db, "2025-10-16");
        assert_eq!(historical.journal.first().unwrap().date, "2025-10-16");
        assert_eq!(historical.journal.last().unwrap().date, "2025-10-10");
        assert_eq!(historical.journal[0].state, ReadState::Unknown);
    }

    #[test]
    fn consented_write_preserves_existing_logs_when_disabled() {
        let db = Store::open(":memory:").unwrap();
        let mut settings = db.settings();
        settings.journal_enabled = true;
        db.set_settings(&settings).unwrap();
        let saved = serde_json::json!({"dailyLogsReport":[{"name":"Synthetic entry","logStatus":"YES"}]});
        assert!(db.put_journal_if_enabled("2025-10-16", "initial", &saved).unwrap());
        settings.journal_enabled = false;
        db.set_settings(&settings).unwrap();
        assert!(!db.put_journal_if_enabled("2025-10-16", "later", &Value::Null).unwrap());
        assert_eq!(db.get("journal", "2025-10-16"), Some(saved));
        assert_eq!(db.fetched_at("journal", "2025-10-16").as_deref(), Some("initial"));
    }
}
