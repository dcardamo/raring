//! Synthetic status snapshot contract; no account data or network.
use chrono::{Duration, Local, NaiveDate, TimeZone, Utc};
use raring_core::{store::Store, view::Settings, Raring};
use serde_json::{json, Value};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(0);
fn db_path() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "raring-status-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&dir).unwrap();
    dir.join("raring.db")
}
fn now(hour: u32) -> chrono::DateTime<Local> {
    Local
        .with_ymd_and_hms(2025, 3, 12, hour, 0, 0)
        .single()
        .unwrap()
}
fn seed(store: &Store) {
    let day = NaiveDate::from_ymd_opt(2025, 3, 12).unwrap();
    for i in 0..8 {
        let date = day - Duration::days(i);
        let wake = date
            .and_hms_opt(7, 0, 0)
            .unwrap()
            .and_utc()
            .timestamp_millis();
        store.put("sleep", &date.to_string(), "fetch", &json!({"dailySleepDTO": {
            "sleepScore": 0, "sleepTimeSeconds": 27000,
            "sleepStartTimestampLocal": wake - 27_000_000,
            "sleepEndTimestampLocal": wake,
            "sleepEndTimestampGMT": wake - now(12).offset().local_minus_utc() as i64 * 1000,
            "sleepNeed": {"actual": 470},
            "nextSleepNeed": {"actual": 470, "baseline": 430, "recommendedBedtimeStartMins": 1350, "recommendedBedtimeEndMins": 1390}
        }})).unwrap();
    }
    store.put("readiness", "2025-03-12", "fetch", &json!({"score": 0, "level": "LOW", "sleepScoreFactorPercent": 21, "sleepScoreFactorFeedback": "POOR"})).unwrap();
    store
        .put(
            "summary",
            "2025-03-12",
            "fetch",
            &json!({"bodyBatteryMostRecentValue": 0, "averageStressLevel": 95}),
        )
        .unwrap();
    store.put("stress", "2025-03-12", "fetch", &json!({"startTimestampGMT": "2025-03-12T00:00:00", "stressValuesArray": [[1741770000000_i64, 0], [1741770180000_i64, -2]]})).unwrap();
    store
        .put(
            "devices",
            "me",
            "fetch",
            &json!([{"deviceId": 44, "primaryActivityTrackerIndicator": true}]),
        )
        .unwrap();
    store
        .put(
            "device_detail",
            "selected",
            "fetch",
            &json!({"deviceId": 44, "lastUploadTimestamp": 1741755600000_i64}),
        )
        .unwrap();
}
#[test]
fn schema_uses_verified_zero_watch_upload_and_phase_focus() {
    let path = db_path();
    let store = Store::open(&path).unwrap();
    seed(&store);
    let core = Raring::open(&path).unwrap();
    core.set_setting("status", true).unwrap();
    core.refresh_status_snapshot(now(12)).unwrap();
    let path = path.with_extension("status.json");
    let v: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(v["version"], 1);
    assert_eq!(v["date"], "2025-03-12");
    assert_eq!(v["enabled"], true);
    assert_eq!(v["phase"], "day");
    assert_eq!(v["focusLine"], "Easy");
    assert_eq!(v["focusTarget"], "readiness");
    assert!(v["focusReason"].as_str().unwrap().contains("Sleep"));
    assert_eq!(v["readiness"], json!({"value": 0.0, "tone": "poor"}));
    assert_eq!(v["sleep"]["value"], 0.0);
    assert_eq!(v["bodyBattery"], json!({"value": 0.0, "tone": "poor"}));
    assert_eq!(v["stress"], json!({"value": 0.0, "tone": "neutral"}));
    assert_eq!(v["heartRate"], json!({"value": null, "tone": "neutral"}));
    assert_eq!(
        v["asOf"],
        chrono::DateTime::<Utc>::from_timestamp_millis(1741755600000)
            .unwrap()
            .to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
    );
    assert_ne!(v["asOf"], "fetch");
    assert_eq!(
        chrono::DateTime::parse_from_rfc3339(v["generatedAt"].as_str().unwrap()).unwrap(),
        now(12).with_timezone(&Utc)
    );
    core.refresh_status_snapshot(now(22)).unwrap();
    let v: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(v["phase"], "windDown");
    assert_eq!(v["focusLine"], "Time to wind down");
    assert_eq!(v["focusTarget"], "sleep");
    assert_eq!(
        v["focusReason"],
        raring_core::screens::today_at(&store, "2025-03-12", now(22).fixed_offset())
            .sleep_plan
            .unwrap()
            .reason
            .unwrap()
    );
    let next_morning = now(8) + Duration::days(1);
    core.refresh_status_snapshot(next_morning).unwrap();
    let next: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(next["date"], "2025-03-12");
    assert_eq!(next["readiness"]["value"], 0.0);
    assert!(next["bodyBattery"]["value"].is_null());
    assert!(next["stress"]["value"].is_null());
    // A daily stress average is not a last measured sample, and yesterday's
    // battery reading is never carried into a missing day.
    store.put("stress", "2025-03-12", "fetch", &json!({"startTimestampGMT": "2025-03-12T00:00:00", "stressValuesArray": [[1741770000000_i64, -2]]})).unwrap();
    store
        .put(
            "summary",
            "2025-03-12",
            "fetch",
            &json!({"averageStressLevel": 95}),
        )
        .unwrap();
    core.refresh_status_snapshot(now(12)).unwrap();
    let v: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(v["stress"], json!({"value": null, "tone": "neutral"}));
    assert_eq!(v["bodyBattery"], json!({"value": null, "tone": "neutral"}));
    let tomorrow = Local
        .with_ymd_and_hms(2025, 3, 13, 8, 0, 0)
        .single()
        .unwrap();
    core.refresh_status_snapshot(tomorrow).unwrap();
    let v: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(v["date"], "2025-03-12");
    assert_eq!(v["focusLine"], "Waiting for last night's sleep");
    assert_eq!(v["focusTarget"], "today");
    assert!(v["focusReason"].is_null());
    core.refresh_status_snapshot(tomorrow + Duration::hours(3))
        .unwrap();
    let v: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(v["date"], "2025-03-13");
    assert!(v["bodyBattery"]["value"].is_null());
    assert!(v["stress"]["value"].is_null());
}
#[test]
fn latest_hr_uses_the_shared_calendar_day_samples_not_resting_hr() {
    let store = Store::open(":memory:").unwrap();
    let start = 1741737600000_i64;
    store.put("daily_hr", "2025-03-12", "synthetic", &json!({
        "startTimestampGMT":"2025-03-12T00:00:00", "restingHeartRate":53,
        "heartRateValues":[[start,81],[start+120000,134],[start+240000,-1]]
    })).unwrap();
    let snapshot = raring_core::status::snapshot(&store, now(12));
    assert_eq!(snapshot.heart_rate.value, Some(134.));
    assert_eq!(raring_core::status::snapshot(&store, now(12)+Duration::days(1)).heart_rate.value, None);
    store.put("daily_hr", "2025-03-12", "synthetic", &json!({
        "startTimestampGMT":"2025-03-12T00:00:00", "restingHeartRate":53,
        "heartRateValues":[[start,-1],[start+120000,null]]
    })).unwrap();
    assert_eq!(raring_core::status::snapshot(&store, now(12)).heart_rate.value, None);
}

#[test]
fn missing_data_migration_atomic_replace_and_forget() {
    let path = db_path();
    let store = Store::open(&path).unwrap();
    store.set_setting("settings", r#"{"notifyMorningReadiness":true,"notifyWeeklyCheckin":true,"notifyMetricChanges":true,"notifyVitals":true,"notifyTrainingStatus":false,"notifyBedtime":false}"#).unwrap();
    assert!(!store.settings().notify_status);
    assert!(!store.settings().status_offer_seen);
    assert!(!Settings::default().notify_status);
    let core = Raring::open(&path).unwrap();
    core.refresh_status_snapshot(now(12)).unwrap();
    let snapshot = path.with_extension("status.json");
    let first = fs::read(&snapshot).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(&snapshot).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
    let v: Value = serde_json::from_slice(&first).unwrap();
    assert!(v["asOf"].is_null());
    for metric in ["readiness", "sleep", "bodyBattery", "stress", "heartRate"] {
        assert_eq!(v[metric], json!({"value": null, "tone": "neutral"}));
    }
    // Nothing synced: Today's lead waits for the night, and so does the surface.
    assert_eq!(v["focusLine"], "Waiting for last night's sleep");
    assert_eq!(v["focusTarget"], "today");
    assert_eq!(v["phase"], "overview");
    let mut settings = core.settings();
    settings.notify_status = true;
    settings.status_offer_seen = true;
    core.set_settings(settings).unwrap();
    let second: Value = serde_json::from_slice(&fs::read(&snapshot).unwrap()).unwrap();
    assert_eq!(second["enabled"], true);
    assert_ne!(fs::read(&snapshot).unwrap(), first);
    assert_eq!(
        fs::read_dir(snapshot.parent().unwrap())
            .unwrap()
            .filter_map(Result::ok)
            .filter(|e| e.file_name().to_string_lossy().contains("tmp"))
            .count(),
        0
    );
    let orphan = snapshot.with_file_name("raring.status.json.999.0.tmp");
    fs::write(&orphan, &first).unwrap();
    core.forget_account().unwrap();
    assert!(!orphan.exists());
    assert!(!snapshot.exists());
    core.refresh_status_snapshot(now(12)).unwrap();
    assert!(!snapshot.exists());
    assert_eq!(core.settings(), Settings::default());
}
#[test]
fn replacement_is_always_complete_and_signout_wins_concurrent_refresh() {
    let path = db_path();
    let core = std::sync::Arc::new(Raring::open(&path).unwrap());
    core.refresh_status_snapshot(now(12)).unwrap();
    let snapshot = path.with_extension("status.json");
    let writer = {
        let core = std::sync::Arc::clone(&core);
        std::thread::spawn(move || {
            for _ in 0..30 {
                core.refresh_status_snapshot(now(12)).unwrap();
            }
        })
    };
    for _ in 0..30 {
        let bytes = fs::read(&snapshot).unwrap();
        let value: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(value["version"], 1);
    }
    core.forget_account().unwrap();
    writer.join().unwrap();
    assert!(!snapshot.exists());
}
#[test]
fn memory_db_skips_snapshot_and_filesystem_errors_propagate() {
    Raring::open(":memory:")
        .unwrap()
        .refresh_status_snapshot(now(12))
        .unwrap();
    let path = db_path();
    let core = Raring::open(&path).unwrap();
    fs::create_dir(path.with_extension("status.json")).unwrap();
    assert!(core.refresh_status_snapshot(now(12)).is_err());
    assert!(core.set_setting("status", true).is_err());
    assert!(!core.setting("status")); // a failed opt-in must not enable a later refresh
    fs::remove_dir(path.with_extension("status.json")).unwrap();
    core.set_setting("status", true).unwrap();
    fs::remove_file(path.with_extension("status.json")).unwrap();
    fs::create_dir(path.with_extension("status.json")).unwrap();
    core.set_setting("status", false).unwrap();
    assert!(!core.setting("status")); // explicit opt-out survives failed cache publication
}

#[test]
fn surfaces_respect_rest_illness_pauses_limits_and_not_today_without_private_text() {
    use raring_core::view::{Limits, LocalAction, PauseReason, Sport};
    let path = db_path();
    let store = Store::open(&path).unwrap();
    seed(&store);
    store.put("readiness", "2025-03-12", "fetch", &json!({"score": 88, "level": "HIGH", "sleepScoreFactorPercent": 90, "sleepScoreFactorFeedback": "GOOD"})).unwrap();
    let core = Raring::open(&path).unwrap();
    core.set_setting("status", true).unwrap();
    let file = path.with_extension("status.json");
    let read = || -> Value { serde_json::from_slice(&fs::read(&file).unwrap()).unwrap() };
    core.refresh_status_snapshot(now(12)).unwrap();
    assert_eq!(read()["focusLine"], "Push");

    // A local change republishes the snapshot offline, without a sync.
    core.local_action(&LocalAction::Dismiss { id: "suggest:2025-03-12".into() }, now(12)).unwrap();
    assert_eq!(read()["focusLine"], "No training suggestion today");
    core.local_action(&LocalAction::RestToday, now(12)).unwrap();
    let v = read();
    assert_eq!((v["focusLine"].as_str(), v["focusTarget"].as_str(), &v["focusReason"]), (Some("Rest day"), Some("today"), &Value::Null));
    core.local_action(&LocalAction::CancelRest, now(12)).unwrap();
    core.local_action(&LocalAction::Pause { reason: PauseReason::Ill, review: None }, now(12)).unwrap();
    assert_eq!(read()["focusLine"], "Ill · goals paused");
    core.local_action(&LocalAction::Resume, now(13)).unwrap();
    core.local_action(&LocalAction::SetLimits { limits: Limits { excluded: vec![Sport::Running], note: "left knee, no jumping".into() } }, now(13)).unwrap();
    let text = fs::read_to_string(&file).unwrap();
    assert!(text.contains("Training suggestions limited"));
    assert!(!text.contains("knee") && !text.contains("jumping"), "limits text never reaches a lock-screen surface");
    // Evening still leads with sleep.
    core.refresh_status_snapshot(now(22)).unwrap();
    assert_eq!(read()["focusLine"], "Time to wind down");

    // The morning notification stays a factual update without a training call.
    core.local_action(&LocalAction::SetLimits { limits: Limits::default() }, now(13)).unwrap();
    core.local_action(&LocalAction::RestToday, now(13)).unwrap();
    let morning = core.notifications(now(9)).into_iter().find(|n| n.kind == "morning").unwrap();
    assert_eq!(morning.title, "Readiness 88");
    assert!(morning.body.ends_with("Rest day · no training call"), "{}", morning.body);
    core.local_action(&LocalAction::CancelRest, now(13)).unwrap();
    let morning = core.notifications(now(9)).into_iter().find(|n| n.kind == "morning").unwrap();
    assert_eq!(morning.title, "Readiness 88", "cancelling rest must not undo Not today");
}

#[test]
fn the_morning_update_drops_garmins_advice_while_resting() {
    use raring_core::view::LocalAction;
    let path = db_path();
    let store = Store::open(&path).unwrap();
    seed(&store);
    store.put("readiness", "2025-03-12", "fetch", &json!({"score": 88, "level": "HIGH", "feedbackShort": "READY_FOR_HIGH_INTENSITY"})).unwrap();
    store.put("activity", "9", "fetch", &json!({"activityId": 9, "activityName": "Early Run", "activityTypeDTO": {"typeKey": "running"},
        "summaryDTO": {"startTimeLocal": "2025-03-12 06:00:00", "duration": 1800, "activityTrainingLoad": 80}})).unwrap();
    let core = Raring::open(&path).unwrap();
    let body = |core: &Raring| core.notifications(now(9)).into_iter().find(|n| n.kind == "morning").unwrap().body;
    let before = body(&core);
    core.local_action(&LocalAction::RestToday, now(8)).unwrap();
    let rested = body(&core);
    assert!(rested.ends_with("Rest day · no training call"), "{rested}");
    let advice: Vec<&str> = before.lines().filter(|l| !rested.contains(*l)).collect();
    assert!(!rested.contains("done ·"), "no workout or Garmin call note beside the rest line: {rested} (before: {advice:?})");
}

#[test]
fn surfaces_follow_todays_lead_through_the_day() {
    use raring_core::view::LeadKind;
    let path = db_path();
    let store = Store::open(&path).unwrap();
    seed(&store);
    let core = Raring::open(&path).unwrap();
    core.set_setting("status", true).unwrap();
    let file = path.with_extension("status.json");
    let tomorrow = |h: u32| Local.with_ymd_and_hms(2025, 3, 13, h, 0, 0).single().unwrap();
    for (when, line, target) in [
        (now(12), "Easy", "readiness"),
        // Today's evening lead, not the phase headline ("Wind down at 22:00").
        (now(21), "Make room for sleep", "sleep"),
        (now(22), "Time to wind down", "sleep"),
        (tomorrow(1), "Rest when you're ready", "sleep"),
        // Past the usual wake with the night unsynced: yesterday's readiness
        // is still displayed, but neither Today nor the surface gives its verdict.
        (tomorrow(8), "Waiting for last night's sleep", "today"),
    ] {
        core.refresh_status_snapshot(when).unwrap();
        let v: Value = serde_json::from_slice(&fs::read(&file).unwrap()).unwrap();
        let today = core.today_live_at(&core.current_date(when), when);
        let lead = today.lead.unwrap();
        assert_eq!((v["focusLine"].as_str(), v["focusTarget"].as_str()), (Some(line), Some(target)), "{when}");
        match lead.kind {
            LeadKind::Sleep => assert_eq!(lead.headline, line),
            LeadKind::Waiting => assert!(v["focusReason"].is_null()),
            // The same reason line as Today, from Garmin's inputs.
            LeadKind::Easy => assert_eq!(v["focusReason"].as_str(), today.readiness_why.as_ref().map(|w| w.summary.as_str())),
            other => panic!("{when}: unexpected lead {other:?}"),
        }
    }
}
