//! Field patches must not replay another control's stale consent.
use chrono::Local;
use raring_core::{settings::SettingsPatch, store::Store, Raring};
use serde_json::{json, Value};
use std::{fs, path::PathBuf, sync::{mpsc, Arc}, thread};

fn patch(value: Value) -> SettingsPatch { serde_json::from_value(value).unwrap() }
fn fixture(name: &str) -> (PathBuf, Arc<Raring>) {
    let dir = std::env::temp_dir().join(format!("raring-settings-patch-{name}-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let db = dir.join("raring.db");
    let store = Store::open(&db).unwrap();
    store.put("profile", "me", "synthetic", &json!({"profileId":101,"displayName":"fake-A"})).unwrap();
    store.set_setting("account_session", &json!({"identity":"101","active":true}).to_string()).unwrap();
    (dir, Arc::new(Raring::open(db).unwrap()))
}
fn save(core: &Raring, value: Value) -> raring_core::Result<()> {
    core.patch_settings_for_account(patch(value), Some("101".into()))
}

#[test]
fn queued_unrelated_patch_cannot_restore_revoked_status() {
    let (dir, core) = fixture("delayed");
    save(&core, json!({"notifyStatus":true})).unwrap();
    let (queued, ready) = mpsc::channel();
    let (release, held) = mpsc::channel();
    let worker = {
        let core = Arc::clone(&core);
        thread::spawn(move || {
            let older = patch(json!({"notifyVitals":false}));
            let expected = core.bound_account_identity();
            queued.send(()).unwrap();
            held.recv().unwrap(); // Models a delayed spawn_blocking save, not a timing sleep.
            core.patch_settings_for_account(older, expected).unwrap();
        })
    };
    ready.recv().unwrap();
    save(&core, json!({"notifyStatus":false})).unwrap();
    assert!(!core.settings().notify_status);
    assert_eq!(core.surface_snapshot().unwrap().unwrap()["enabled"], false);
    release.send(()).unwrap(); worker.join().unwrap();
    assert!(!core.settings().notify_vitals, "the old unrelated change still applies");
    assert!(!core.settings().notify_status);
    assert_eq!(core.surface_snapshot().unwrap().unwrap()["enabled"], false);
    drop(core); fs::remove_dir_all(dir).unwrap();
}

#[test]
fn failed_opt_in_then_unrelated_patch_stays_off() {
    let (dir, core) = fixture("failed-opt-in");
    let path = dir.join("raring.status.json");
    fs::create_dir(&path).unwrap();
    let old_unrelated = patch(json!({"notifyBedtime":true}));
    assert!(save(&core, json!({"notifyStatus":true,"statusOfferSeen":true})).is_err());
    assert!(!core.settings().notify_status);
    core.patch_settings_for_account(old_unrelated, Some("101".into())).unwrap();
    assert!(core.settings().notify_bedtime);
    assert!(!core.settings().notify_status);
    fs::remove_dir(&path).unwrap();
    core.refresh_status_snapshot(Local::now()).unwrap();
    assert_eq!(core.surface_snapshot().unwrap().unwrap()["enabled"], false);
    drop(core); fs::remove_dir_all(dir).unwrap();
}

#[test]
fn cache_failure_rolls_back_only_new_status_opt_in_never_other_revocations() {
    let (dir, core) = fixture("revocations");
    let path = dir.join("raring.status.json");
    save(&core, json!({"journalEnabled":true})).unwrap();
    fs::remove_file(&path).unwrap(); fs::create_dir(&path).unwrap();
    // Mixed patches must not resurrect another consent when status opt-in fails.
    assert!(save(&core, json!({"notifyStatus":true,"notifyVitals":false,"journalEnabled":false})).is_err());
    assert!(!core.settings().notify_status);
    assert!(!core.settings().notify_vitals);
    assert!(!core.settings().journal_enabled);
    fs::remove_dir(&path).unwrap();
    save(&core, json!({"notifyStatus":true})).unwrap();
    fs::remove_file(&path).unwrap(); fs::create_dir(&path).unwrap();
    // Only a new status opt-in can roll back on cache failure.
    save(&core, json!({"notifyMorningReadiness":false})).unwrap();
    assert!(core.settings().notify_status);
    assert!(!core.settings().notify_morning_readiness);
    save(&core, json!({"notifyStatus":false})).unwrap();
    assert!(!core.settings().notify_status);
    drop(core); fs::remove_dir_all(dir).unwrap();
}

#[test]
fn journal_revocation_survives_cache_failure_and_an_older_unrelated_patch() {
    let (dir, core) = fixture("journal-off");
    let store = Store::open(dir.join("raring.db")).unwrap();
    save(&core, json!({"notifyStatus":true,"journalEnabled":true})).unwrap();
    let day = Local::now().date_naive().to_string();
    let retained = json!({"dailyLogsReport":[{"name":"PRIVATE_SYNTHETIC_JOURNAL","logStatus":"YES"}]});
    assert!(store.put_journal_if_enabled(&day, "synthetic", &retained).unwrap());
    let path = dir.join("raring.status.json");
    fs::remove_file(&path).unwrap(); fs::create_dir(&path).unwrap();
    let (queued, ready) = mpsc::channel(); let (release, held) = mpsc::channel();
    let worker = {
        let core = Arc::clone(&core);
        thread::spawn(move || {
            let older = patch(json!({"notifyMorningReadiness":false}));
            let expected = core.bound_account_identity();
            queued.send(()).unwrap(); held.recv().unwrap();
            core.patch_settings_for_account(older, expected).unwrap();
        })
    };
    ready.recv().unwrap();
    save(&core, json!({"journalEnabled":false})).unwrap();
    assert!(!core.settings().journal_enabled);
    assert!(core.settings().notify_status);
    // The unchanged guarded-write path observes this patch immediately.
    assert!(!store.put_journal_if_enabled(&day, "late-response", &json!({"dailyLogsReport":[]})).unwrap());
    release.send(()).unwrap(); worker.join().unwrap();
    assert!(!core.settings().journal_enabled);
    assert!(!core.settings().notify_morning_readiness);
    assert_eq!(store.get("journal", &day), Some(retained));
    fs::remove_dir(&path).unwrap(); core.refresh_status_snapshot(Local::now()).unwrap();
    assert!(!core.settings().journal_enabled);
    assert!(!core.surface_snapshot().unwrap().unwrap().to_string().contains("PRIVATE_SYNTHETIC_JOURNAL"));
    drop(store); drop(core); fs::remove_dir_all(dir).unwrap();
}

#[test]
fn journal_data_and_consent_never_enter_native_snapshot_or_notifications() {
    let (dir, core) = fixture("journal-allowlist");
    let store = Store::open(dir.join("raring.db")).unwrap();
    let now = Local::now(); let day = now.date_naive().to_string();
    raring_core::demo::seed(&store, now.date_naive()).unwrap();
    store.put("profile", "me", "synthetic", &json!({"profileId":101,"displayName":"fake-A"})).unwrap();
    store.put("journal", &day, &now.to_rfc3339(), &json!({"dailyLogsReport":[{"name":"PRIVATE_SYNTHETIC_JOURNAL","logStatus":"YES"}]})).unwrap();
    save(&core, json!({"notifyStatus":true})).unwrap();
    core.refresh_status_snapshot(now).unwrap();
    let before = core.surface_snapshot().unwrap().unwrap();
    save(&core, json!({"journalEnabled":true})).unwrap();
    assert!(core.health_live_at(&day, now).journal.iter().flat_map(|d| &d.entries).any(|e| e.name == "PRIVATE_SYNTHETIC_JOURNAL"));
    core.refresh_status_snapshot(now).unwrap();
    let after = core.surface_snapshot().unwrap().unwrap();
    assert_eq!(before, after, "journal consent and contents must not affect the native snapshot");
    let keys = after.as_object().unwrap().keys().map(String::as_str).collect::<std::collections::BTreeSet<_>>();
    assert_eq!(keys, ["version","accountId","generatedAt","asOf","date","enabled","phase","focusLine","focusReason","focusTarget","readiness","sleep","bodyBattery","stress","heartRate"].into_iter().collect());
    assert!(!serde_json::to_string(&core.notifications(now)).unwrap().contains("PRIVATE_SYNTHETIC_JOURNAL"));
    drop(store); drop(core); fs::remove_dir_all(dir).unwrap();
}

#[test]
fn patches_reject_unknown_names_wrong_types_and_changed_accounts_atomically() {
    let (dir, core) = fixture("validation");
    let before = core.settings();
    for value in [
        json!({"unknownSetting":false}), json!({"notifyStatus":"false"}),
        json!({"notifyStatus":null}), json!({"notifyStatus":0}),
        json!({"journalEnabled":"false"}), json!({"journalEnabled":null}),
        json!({"bedtimeLeadMin":false}), json!({"bedtimeLeadMin":-1}),
        json!({"bedtimeLeadMin":1.5}), json!({"bedtimeLeadMin":4294967296_u64}),
        json!({"notifyVitals":false,"unknownSetting":true}),
    ] {
        assert!(save(&core, value).is_err());
        assert_eq!(core.settings(), before);
    }
    for value in [Value::Null, json!([]), json!(false)] {
        assert!(serde_json::from_value::<SettingsPatch>(value).is_err());
    }
    assert!(core.patch_settings_for_account(patch(json!({"notifyStatus":true})), Some("202".into())).is_err());
    assert_eq!(core.settings(), before);
    save(&core, json!({"bedtimeLeadMin":45,"adaptiveToday":false})).unwrap();
    assert_eq!(core.settings().bedtime_lead_min, 45);
    assert!(!core.settings().adaptive_today);
    assert_eq!(core.settings().notify_status, before.notify_status);
    drop(core); fs::remove_dir_all(dir).unwrap();
}
