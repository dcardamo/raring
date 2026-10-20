//! Importing local settings from a Raring export: what is refused, what is
//! skipped, the merge rules, atomicity and the export round trip.
use chrono::{Local, TimeZone};
use raring_core::{import::ImportError, local, plan, store::Store, view::*, Raring};
use serde_json::{json, Value};
use std::{
    io::{Read, Write},
    path::PathBuf,
};

// Thursday 13 March 2025; the plan week starts Monday 10 March.
fn now() -> chrono::DateTime<Local> {
    Local.with_ymd_and_hms(2025, 3, 13, 10, 0, 0).unwrap()
}

struct Dir(PathBuf);
impl Dir {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!("raring-import-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }
    fn db(&self) -> PathBuf {
        self.0.join("raring.db")
    }
    fn path(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}
impl Drop for Dir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn bind(store: &Store, id: u64) {
    store
        .put("profile", "me", "now", &json!({"profileId": id, "displayName": "synthetic"}))
        .unwrap();
    store
        .set_setting("account_session", &json!({"identity": id.to_string(), "active": true}).to_string())
        .unwrap();
}

/// A signed-in phone for account 11.
fn open(name: &str) -> (Dir, Store, Raring) {
    let dir = Dir::new(name);
    let store = Store::open(dir.db()).unwrap();
    bind(&store, 11);
    let core = Raring::open(dir.db()).unwrap();
    (dir, store, core)
}

fn signed_out(name: &str) -> (Dir, Store, Raring) {
    let dir = Dir::new(name);
    let store = Store::open(dir.db()).unwrap();
    let core = Raring::open(dir.db()).unwrap();
    (dir, store, core)
}

fn manifest(account: Value) -> Value {
    json!({
        "archive": {"format": "raring-export", "version": 1, "account": account},
        "exported_at": "2025-03-13T09:00:00+00:00",
        "app_version": "test"
    })
}

/// A ZIP holding just the two entries an import reads.
fn archive(dir: &Dir, name: &str, settings: &Value, manifest: &Value) -> PathBuf {
    archive_entries(dir, name, &[
        ("manifest.json", manifest.to_string().as_bytes()),
        ("settings.json", settings.to_string().as_bytes()),
    ])
}

fn archive_entries(dir: &Dir, name: &str, entries: &[(&str, &[u8])]) -> PathBuf {
    let path = dir.path(name);
    let mut zip = zip::ZipWriter::new(std::fs::File::create(&path).unwrap());
    for (entry, body) in entries {
        zip.start_file(*entry, zip::write::FileOptions::default()).unwrap();
        zip.write_all(body).unwrap();
    }
    zip.finish().unwrap();
    path
}

fn export_settings(core: &Raring, dir: &Dir, name: &str) -> Value {
    let path = dir.path(&format!("{name}-export.zip"));
    core.export(&path, "test").unwrap();
    let mut zip = zip::ZipArchive::new(std::fs::File::open(&path).unwrap()).unwrap();
    let mut body = String::new();
    zip.by_name("settings.json").unwrap().read_to_string(&mut body).unwrap();
    serde_json::from_str(&body).unwrap()
}

/// A captured prospective sleep plan, as the app writes it: a synthetic store
/// with demo data and tonight's plan, then one capture. The night and its row
/// are returned so tests import a plausible key.
fn saved_sleep_plan() -> (String, Value) {
    let dir = Dir::new("saved-night");
    let store = Store::open(dir.db()).unwrap();
    let date = chrono::NaiveDate::from_ymd_opt(2025, 1, 2).unwrap();
    let at = Local.from_local_datetime(&date.and_hms_opt(12, 0, 0).unwrap()).single().unwrap();
    raring_core::demo::seed(&store, date).unwrap();
    plan::set(&store, date, &PlanUpdate { targets: PlanTargets::preset(PlanPreset::Strength), paused: false }).unwrap();
    plan::capture_sleep_plan(&store, at).unwrap();
    let (key, raw) = store
        .setting("sleep_plan:2025-01-03")
        .map(|raw| ("sleep_plan:2025-01-03".to_owned(), raw))
        .expect("the synthetic plan was not captured");
    (key, serde_json::from_str(&raw).unwrap())
}

fn goal_version(from: &str, saved_at: &str, strength: u32) -> Value {
    json!({"from": from, "targets": {"strengthSessions": strength, "zone2Min": 90},
        "source": "revision", "savedAt": saved_at})
}

fn goals_row(versions: Vec<Value>) -> Value {
    json!({"schema": 1, "versions": versions, "observationExplained": false})
}

fn act(core: &Raring, action: LocalAction) -> raring_core::Result<()> {
    core.local_action(&action, now())
}

fn targets(strength: u32) -> GoalTargets {
    GoalTargets { strength_sessions: strength, zone2_min: 90, ..Default::default() }
}

fn intention(id: &str, date: &str, last_at: &str, name: &str) -> Value {
    json!({"id": id, "date": date, "time": null, "sport": "strength", "typeKey": null,
        "name": name, "durationMin": 45, "allocation": {"strengthSessions": 1}, "kind": "workout",
        "garminId": null, "status": "open",
        "history": [{"at": last_at, "change": "added", "from": null, "to": date}]})
}

fn sessions_row(items: Vec<Value>) -> Value {
    json!({"schema": 1, "items": items})
}

fn link(plan_id: &str, activity_id: &str, at: &str) -> Value {
    json!({"planId": plan_id, "activityId": activity_id, "at": at})
}

fn assert_unreadable(result: Result<ImportPreview, ImportError>) {
    assert!(matches!(result, Err(ImportError::Unreadable(_))), "unexpected result");
}

#[test]
fn refuses_files_that_are_not_this_export_format() {
    let (dir, _store, core) = open("format");
    let text = dir.path("notes.zip");
    std::fs::write(&text, b"not a zip").unwrap();
    assert!(matches!(core.import_preview(&text, None), Err(ImportError::NotAnArchive)));

    let no_manifest = archive_entries(&dir, "no-manifest.zip", &[("settings.json", b"{}")]);
    assert!(matches!(core.import_preview(&no_manifest, None), Err(ImportError::NotAnArchive)));

    // An export from a version that does not declare the archive format.
    let older = archive(&dir, "older.zip", &json!({}), &json!({"exported_at": "2025-03-01T00:00:00Z", "app_version": "0.1.29"}));
    assert!(matches!(core.import_preview(&older, None), Err(ImportError::NotAnArchive)));

    let newer = archive(&dir, "newer.zip", &json!({}), &json!({"archive": {"format": "raring-export", "version": 2, "account": "11"}}));
    assert!(matches!(core.import_preview(&newer, None), Err(ImportError::NewerExport)));

    let wrong_format = archive(&dir, "other.zip", &json!({}), &json!({"archive": {"format": "other-app", "version": 1, "account": "11"}}));
    assert!(matches!(core.import_preview(&wrong_format, None), Err(ImportError::NotAnArchive)));

    let not_object = archive_entries(&dir, "not-object.zip", &[
        ("manifest.json", manifest(json!("11")).to_string().as_bytes()),
        ("settings.json", b"[]"),
    ]);
    assert_unreadable(core.import_preview(&not_object, None));
}

#[test]
fn refuses_a_missing_another_or_changed_account() {
    let (dir, _store, core) = open("accounts");
    let unbound = archive(&dir, "unbound.zip", &json!({}), &manifest(Value::Null));
    assert!(matches!(core.import_preview(&unbound, None), Err(ImportError::MissingAccount)));
    let invalid = archive(&dir, "invalid.zip", &json!({}), &manifest(json!("not-an-id")));
    assert!(matches!(core.import_preview(&invalid, None), Err(ImportError::MissingAccount)));

    // Another Garmin account is never mixed in.
    let other = archive(&dir, "other-account.zip", &json!({"settings": {"planPrompt": false}}), &manifest(json!("22")));
    assert!(matches!(core.import_preview(&other, None), Err(ImportError::AccountMismatch)));
    assert!(matches!(core.import_settings(&other, None), Err(ImportError::AccountMismatch)));
    assert_eq!(core.local_records(), local::LocalRecords::default(), "nothing may be written");

    // A queued import is refused if the account changed since it was chosen.
    let mine = archive(&dir, "mine.zip", &json!({"settings": {"planPrompt": false}}), &manifest(json!("11")));
    assert!(matches!(core.import_settings(&mine, Some("22".into())), Err(ImportError::AccountChanged)));
    assert!(core.settings().plan_prompt, "nothing may be written");

    let (_dir, _store, out) = signed_out("signed-out");
    assert!(matches!(out.import_preview(&mine, None), Err(ImportError::NotSignedIn)));
}

#[test]
fn refuses_credentials_newer_rows_and_unreadable_shapes() {
    let (dir, _store, core) = open("shapes");
    let credential = |result: Result<ImportPreview, ImportError>| {
        assert!(matches!(result, Err(ImportError::Credentials)), "unexpected result");
    };
    let newer = |result: Result<ImportPreview, ImportError>| {
        assert!(matches!(result, Err(ImportError::NewerRow(_))), "unexpected result");
    };
    let refused = |name: &str, settings: Value, check: fn(Result<ImportPreview, ImportError>)| {
        let path = archive(&dir, name, &settings, &manifest(json!("11")));
        check(core.import_preview(&path, None));
    };

    refused("token.zip", json!({"access_token": "secret"}), credential);
    refused("nested-token.zip", json!({"local:goals": {"schema": 1, "versions": [], "refreshToken": "secret"}}), credential);
    refused("newer-local.zip", json!({"local:goals": {"schema": 2, "versions": []}}), newer);
    refused("newer-unknown.zip", json!({"local:future": {"schema": 1}}), newer);
    refused("newer-export-marker.zip", json!({"local:schema": 2}), |result| {
        assert!(matches!(result, Err(ImportError::NewerExport)), "unexpected result");
    });

    for (name, settings) in [
        ("no-schema.zip", json!({"local:goals": {"versions": []}})),
        ("unknown-field.zip", json!({"local:goals": {"schema": 1, "versions": [], "extra": 1}})),
        ("bad-date.zip", json!({"local:goals": goals_row(vec![goal_version("2025-02-30", "2025-02-28T10:00:00", 3)])})),
        ("duplicate-date.zip", json!({"local:goals": goals_row(vec![
            goal_version("2025-03-10", "2025-03-10T10:00:00", 3),
            goal_version("2025-03-10", "2025-03-11T10:00:00", 2),
        ])})),
        ("out-of-range.zip", json!({"local:goals": goals_row(vec![goal_version("2025-03-10", "2025-03-10T10:00:00", 40)])})),
        ("duplicate-session.zip", json!({"local:intentions": sessions_row(vec![
            intention("i1", "2025-03-14", "2025-03-13T10:00:00", "Tempo"),
            intention("i1", "2025-03-15", "2025-03-13T11:00:00", "Easy"),
        ])})),
        ("bad-session-time.zip", json!({"local:intentions": sessions_row(vec![json!({
            "id": "i1", "date": "2025-03-14", "time": "25:00", "sport": "strength", "typeKey": null,
            "name": null, "durationMin": 45, "allocation": {}, "kind": "workout", "garminId": null,
            "status": "open", "history": []
        })])})),
        ("allocation-out-of-range.zip", json!({"local:intentions": sessions_row(vec![json!({
            "id": "i1", "date": "2025-03-14", "time": null, "sport": "strength", "typeKey": null,
            "name": null, "durationMin": 45, "allocation": {"zone2Min": 900}, "kind": "workout",
            "garminId": null, "status": "open", "history": []
        })])})),
        ("bad-pause.zip", json!({"local:circumstances": {"schema": 1, "rest": [], "limits": {},
            "dismissals": [], "windDowns": [], "todayChoice": null,
            "pauses": [{"id": "p1", "since": "2025-03-10", "until": "2025-03-01", "reason": "pause", "review": null}]}})),
        ("duplicate-link.zip", json!({"local:links": {"schema": 1,
            "links": [link("i1", "7001", "2025-03-13T10:00:00"), link("i1", "7002", "2025-03-13T11:00:00")],
            "unlinks": []}})),
        ("orphan-report.zip", json!({"local:routines": {"schema": 1, "items": [], "reports": [
            {"id": "r1", "date": "2025-03-13", "at": "2025-03-13T20:00:00"}]}})),
        ("unreadable-plan.zip", json!({"plan": {"versions": {"not-a-date": {}}, "paused": []}})),
        ("bad-plan-row.zip", json!({"plan": "not an object"})),
        ("unreadable-sleep-plan.zip", json!({"sleep_plan:2025-03-14": {"plan": {}}})),
        ("bad-sleep-plan-key.zip", json!({"sleep_plan:tomorrow": {}})),
    ] {
        refused(name, settings, assert_unreadable);
    }
}

#[test]
fn export_round_trip_is_identical_and_idempotent() {
    let (source_dir, source_store, source) = open("round-trip-source");
    source_store
        .set_setting("plan", &json!({"versions": {"2024-12-23": {"preset": "strength", "strengthSessions": 3,
            "intensityMin": null, "zone2Min": 60, "asleepNights": 2, "restDays": [6]}}, "paused": ["2024-12-30"]}).to_string())
        .unwrap();
    act(&source, LocalAction::SetGoals { targets: targets(2) }).unwrap();
    source_store
        .put("activity", "7001", "now", &json!({"activityId": 7001, "activityTypeDTO": {"typeKey": "strength_training"},
            "summaryDTO": {"startTimeLocal": "2025-03-13 07:10:00", "duration": 1800}}))
        .unwrap();
    act(&source, LocalAction::AddIntention {
        date: "2025-03-14".into(), time: Some("18:30".into()), sport: Sport::Strength, type_key: None,
        name: Some("Tempo".into()), duration_min: Some(45), allocation: Allocation { strength_sessions: 1, ..Default::default() },
        kind: IntentionKind::Workout, garmin_id: None,
    })
    .unwrap();
    let intention_id = source.local_records().intentions.items[0].id.clone();
    act(&source, LocalAction::Link { plan_id: intention_id, activity_id: "7001".into() }).unwrap();
    act(&source, LocalAction::RestToday).unwrap();
    act(&source, LocalAction::Pause { reason: PauseReason::Ill, review: Some("2025-03-20".into()) }).unwrap();
    act(&source, LocalAction::SetLimits { limits: Limits { excluded: vec![Sport::Running], note: "knee".into() } }).unwrap();
    act(&source, LocalAction::SetPromptWeek { week: "2025-03-10".into(), choice: PromptChoice::Planned }).unwrap();
    act(&source, LocalAction::AddEvent { name: "10K".into(), date: "2025-04-01".into(), sport: Some(Sport::Running) }).unwrap();
    act(&source, LocalAction::AddRoutine { name: "Wall sit".into(), how: Some("3 × 45 s".into()) }).unwrap();
    let routine_id = source.local_records().routines.items[0].id.clone();
    act(&source, LocalAction::ReportRoutine { id: routine_id, done: true }).unwrap();
    act(&source, LocalAction::Dismiss { id: "suggestion-1".into() }).unwrap();
    act(&source, LocalAction::ReportWindDown).unwrap();
    act(&source, LocalAction::DecideReview { week: "2025-03-03".into(), accepted: vec![] }).unwrap();
    let session_id = source.local_records().intentions.items[0].id.clone();
    act(&source, LocalAction::SetIntentionStatus { id: session_id, status: IntentionStatus::LetGo }).unwrap();
    let settings = export_settings(&source, &source_dir, "round-trip");

    let path = archive(&source_dir, "round-trip.zip", &settings, &manifest(json!("11")));
    let (_target_dir, target_store, target) = open("round-trip-target");
    let preview = target.import_preview(&path, None).unwrap();
    assert_eq!(preview.app_version.as_deref(), Some("test"));
    let (from, to) = (preview.from.clone().unwrap(), preview.to.clone().unwrap());
    assert!(from <= to, "{from}..{to}");
    assert!(preview.rows.iter().any(|row| row.kind == "Your sessions" && row.added == 1));
    assert!(preview.skipped.iter().any(|skip| skip.what == "Garmin data and journal logs"));

    // Preview alone writes nothing.
    assert_eq!(target.local_records(), local::LocalRecords::default());
    target.import_settings(&path, None).unwrap();

    assert_eq!(target.local_records(), source.local_records());
    let plan = |store: &Store| serde_json::from_str::<Value>(&store.setting("plan").unwrap()).unwrap();
    assert_eq!(plan(&target_store), plan(&source_store));
    assert_eq!(target_store.setting("local:schema"), source_store.setting("local:schema"));

    // Re-importing the same file changes nothing.
    let before = target.local_records();
    let again = target.import_preview(&path, None).unwrap();
    assert!(again.rows.iter().all(|row| row.added == 0 && row.replaced == 0), "{:?}", again.rows);
    target.import_settings(&path, None).unwrap();
    assert_eq!(target.local_records(), before);
}

#[test]
fn goals_merge_by_effective_date_and_newer_save() {
    let (dir, _store, core) = open("goals");
    act(&core, LocalAction::SetGoals { targets: targets(2) }).unwrap();
    let start = core.local_records().goals.versions[0].from.clone();
    assert_eq!(start, "2025-03-10");

    let later = archive(&dir, "later.zip", &json!({
        "local:goals": goals_row(vec![
            goal_version(&start, "2025-03-13T11:00:00", 4),
            goal_version("2025-03-17", "2025-03-13T09:00:00", 3),
        ])
    }), &manifest(json!("11")));
    let preview = core.import_preview(&later, None).unwrap();
    let goals = preview.rows.iter().find(|row| row.kind == "Goals").cloned().unwrap();
    assert_eq!((goals.added, goals.replaced, goals.kept), (1, 1, 0));
    core.import_settings(&later, None).unwrap();
    let versions = core.local_records().goals.versions;
    assert_eq!(versions.len(), 2);
    assert_eq!(versions[0].targets.strength_sessions, 4, "a later save on the same day replaces");
    assert_eq!(versions[1].from, "2025-03-17");
    assert_eq!(versions[1].targets.strength_sessions, 3);

    // An older copy of a date this phone already has is kept.
    let earlier = archive(&dir, "earlier.zip", &json!({
        "local:goals": goals_row(vec![goal_version("2025-03-17", "2025-03-01T09:00:00", 1)])
    }), &manifest(json!("11")));
    let preview = core.import_preview(&earlier, None).unwrap();
    let goals = preview.rows.iter().find(|row| row.kind == "Goals").cloned().unwrap();
    assert_eq!((goals.added, goals.replaced, goals.kept), (0, 0, 1));
    core.import_settings(&earlier, None).unwrap();
    assert_eq!(core.local_records().goals.versions[1].targets.strength_sessions, 3);
}

#[test]
fn sessions_links_and_decisions_take_the_later_copy() {
    let (dir, store, core) = open("sessions");
    for (id, start) in [("7001", "2025-03-13 07:00:00"), ("7002", "2025-03-13 08:00:00")] {
        store
            .put("activity", id, "now", &json!({"activityId": id.parse::<u64>().unwrap(),
                "activityTypeDTO": {"typeKey": "strength_training"},
                "summaryDTO": {"startTimeLocal": start, "duration": 1800}}))
            .unwrap();
    }
    act(&core, LocalAction::AddIntention {
        date: "2025-03-14".into(), time: None, sport: Sport::Strength, type_key: None, name: Some("Easy".into()),
        duration_min: Some(45), allocation: Allocation { strength_sessions: 1, ..Default::default() },
        kind: IntentionKind::Workout, garmin_id: None,
    })
    .unwrap();
    let id = core.local_records().intentions.items[0].id.clone();
    act(&core, LocalAction::Link { plan_id: id.clone(), activity_id: "7001".into() }).unwrap();
    act(&core, LocalAction::Unlink { plan_id: "garmin:2025-03-13:1".into(), activity_id: "7002".into() }).unwrap();
    act(&core, LocalAction::DecideReview { week: "2025-03-03".into(), accepted: vec![] }).unwrap();

    let later = archive(&dir, "later.zip", &json!({
        "local:intentions": sessions_row(vec![
            intention(&id, "2025-03-14", "2025-03-13T11:00:00", "Tempo"),
            intention("i999", "2025-03-16", "2025-03-13T11:00:00", "Long"),
        ]),
        "local:links": {"schema": 1,
            "links": [link(&id, "7002", "2025-03-13T11:00:00")],
            "unlinks": [link("garmin:2025-03-13:1", "7002", "2025-03-13T09:00:00")]},
        "local:reviews": {"schema": 1, "prompts": {}, "decisions": [
            {"week": "2025-03-03", "at": "2025-03-13T11:00:00", "proposals": [], "accepted": []}]}
    }), &manifest(json!("11")));
    let preview = core.import_preview(&later, None).unwrap();
    let row = |kind: &str| preview.rows.iter().find(|row| row.kind == kind).cloned().unwrap();
    assert_eq!((row("Your sessions").added, row("Your sessions").replaced, row("Your sessions").kept), (1, 1, 0));
    assert_eq!((row("Plan links").added, row("Plan links").replaced, row("Plan links").kept), (0, 1, 1));
    assert_eq!((row("Plan-week prompts").added, row("Plan-week prompts").replaced), (0, 1));
    core.import_settings(&later, None).unwrap();

    let records = core.local_records();
    let session = records.intentions.items.iter().find(|item| item.id == id).unwrap();
    assert_eq!(session.name.as_deref(), Some("Tempo"));
    // The later link takes the session from 7001 to 7002, and the older unlink stands.
    assert_eq!(records.links.links.len(), 1);
    assert_eq!(records.links.links[0].activity_id, "7002");
    assert_eq!(records.links.unlinks.len(), 1);
    assert_eq!(records.reviews.decisions[0].at, "2025-03-13T11:00:00");

    // An earlier copy of the same session is kept.
    let earlier = archive(&dir, "earlier.zip", &json!({
        "local:intentions": sessions_row(vec![intention(&id, "2025-03-14", "2025-03-12T08:00:00", "Old name")])
    }), &manifest(json!("11")));
    core.import_settings(&earlier, None).unwrap();
    let session = core.local_records().intentions.items.iter().find(|item| item.id == id).cloned().unwrap();
    assert_eq!(session.name.as_deref(), Some("Tempo"));

    // A later link re-pairs a pair this phone unlinked.
    let relink = archive(&dir, "relink.zip", &json!({
        "local:links": {"schema": 1,
            "links": [link("garmin:2025-03-13:1", "7002", "2025-03-13T12:00:00")],
            "unlinks": []}
    }), &manifest(json!("11")));
    core.import_settings(&relink, None).unwrap();
    let records = core.local_records();
    assert_eq!(records.links.links.len(), 2);
    assert!(records.links.unlinks.is_empty());
}

#[test]
fn circumstances_only_fill_what_is_missing() {
    let (dir, _store, core) = open("circumstances");
    act(&core, LocalAction::RestToday).unwrap();
    act(&core, LocalAction::Pause { reason: PauseReason::Ill, review: None }).unwrap();
    act(&core, LocalAction::Dismiss { id: "suggestion-1".into() }).unwrap();
    act(&core, LocalAction::ReportWindDown).unwrap();

    let pause_id = core.local_records().circumstances.pauses[0].id.clone();
    let incoming = archive(&dir, "circumstances.zip", &json!({
        "local:circumstances": {
            "schema": 1,
            "rest": [
                {"date": "2025-03-13", "at": "2025-03-01T08:00:00"},
                {"date": "2025-03-05", "at": "2025-03-05T08:00:00"}
            ],
            "pauses": [
                {"id": pause_id, "since": "2025-03-01", "until": "2025-03-10", "reason": "ill", "review": null},
                {"id": "p2", "since": "2025-02-01", "until": null, "reason": "pause", "review": null}
            ],
            "limits": {"excluded": ["running"], "note": "knee"},
            "dismissals": [
                {"id": "suggestion-1", "date": "2025-03-13"},
                {"id": "suggestion-2", "date": "2025-03-12"}
            ],
            "windDowns": [
                {"date": "2025-03-12", "at": "2025-03-01T20:00:00"},
                {"date": "2025-03-11", "at": "2025-03-11T20:00:00"}
            ],
            "todayChoice": {"date": "2025-03-13", "planId": "garmin:2025-03-13:1"}
        }
    }), &manifest(json!("11")));
    core.import_settings(&incoming, None).unwrap();
    let c = core.local_records().circumstances;
    assert_eq!(c.rest.len(), 2, "a day this phone already marked rest is kept");
    assert_eq!(c.pauses.len(), 2);
    let p1 = c.pauses.iter().find(|p| p.id == pause_id).unwrap();
    assert_eq!(p1.until.as_deref(), Some("2025-03-10"), "a closed pause ends an open one");
    assert_eq!(c.limits.note, "knee");
    assert_eq!(c.dismissals.len(), 2);
    assert_eq!(c.wind_downs.len(), 2);
    assert_eq!(c.today_choice.unwrap().plan_id, "garmin:2025-03-13:1");

    // Limits and today's choice this phone already has are kept, not overwritten.
    let other = archive(&dir, "other-limits.zip", &json!({
        "local:circumstances": {
            "schema": 1, "rest": [], "pauses": [], "dismissals": [], "windDowns": [],
            "limits": {"excluded": ["cycling"], "note": "shoulder"},
            "todayChoice": {"date": "2025-03-13", "planId": "garmin:2025-03-13:2"}
        }
    }), &manifest(json!("11")));
    let preview = core.import_preview(&other, None).unwrap();
    let row = preview.rows.iter().find(|row| row.kind == "Rest, pauses and limits").cloned().unwrap();
    assert_eq!((row.added, row.replaced, row.kept), (0, 0, 2));
    core.import_settings(&other, None).unwrap();
    let c = core.local_records().circumstances;
    assert_eq!(c.limits.note, "knee");
    assert_eq!(c.today_choice.unwrap().plan_id, "garmin:2025-03-13:1");

    // An open incoming pause never reopens a pause this phone closed.
    let resume = archive(&dir, "resume.zip", &json!({
        "local:circumstances": {"schema": 1, "rest": [], "pauses": [
            {"id": pause_id, "since": "2025-03-01", "until": null, "reason": "ill", "review": null}
        ], "limits": {}, "dismissals": [], "windDowns": [], "todayChoice": null}
    }), &manifest(json!("11")));
    core.import_settings(&resume, None).unwrap();
    let c = core.local_records().circumstances;
    assert_eq!(c.pauses.iter().find(|p| p.id == pause_id).unwrap().until.as_deref(), Some("2025-03-10"));
}

#[test]
fn events_routines_plan_snapshots_and_the_prompt_fill_missing_only() {
    let (dir, store, core) = open("fill-missing");
    act(&core, LocalAction::AddEvent { name: "10K".into(), date: "2025-04-01".into(), sport: None }).unwrap();
    let event_id = core.local_records().events.events[0].id.clone();
    act(&core, LocalAction::AddRoutine { name: "Wall sit".into(), how: None }).unwrap();
    let routine_id = core.local_records().routines.items[0].id.clone();
    act(&core, LocalAction::ReportRoutine { id: routine_id.clone(), done: true }).unwrap();
    store
        .set_setting("plan", &json!({"versions": {"2024-12-23": {"preset": "strength", "strengthSessions": 3,
            "intensityMin": null, "zone2Min": 0, "asleepNights": 0, "restDays": []}}, "paused": []}).to_string())
        .unwrap();
    // A phone that has saved its own settings keeps its prompt choice.
    store.set_settings(&Settings { plan_prompt: true, ..Default::default() }).unwrap();
    let (night, saved_night) = saved_sleep_plan();

    let incoming = archive(&dir, "fill-missing.zip", &json!({
        "local:events": {"schema": 1, "events": [
            {"id": event_id, "name": "Renamed", "date": "2025-04-02", "sport": null},
            {"id": "e2", "name": "Half", "date": "2025-05-01", "sport": "running"}
        ]},
        "local:routines": {"schema": 1,
            "items": [
                {"id": routine_id, "name": "Renamed", "how": null},
                {"id": "r2", "name": "Calf raise", "how": null}
            ],
            "reports": [
                {"id": routine_id, "date": "2025-03-13", "at": "2025-03-13T09:00:00"},
                {"id": routine_id, "date": "2025-03-12", "at": "2025-03-12T21:00:00"}
            ]},
        "plan": {"versions": {
            "2024-12-23": {"preset": "strength", "strengthSessions": 1, "intensityMin": null, "zone2Min": 0, "asleepNights": 0, "restDays": []},
            "2025-01-06": {"preset": "custom", "strengthSessions": 2, "intensityMin": null, "zone2Min": 30, "asleepNights": 0, "restDays": []}
        }, "paused": ["2025-01-06"]},
        night.clone(): saved_night,
        "settings": {"planPrompt": false}
    }), &manifest(json!("11")));
    let preview = core.import_preview(&incoming, None).unwrap();
    let row = |kind: &str| preview.rows.iter().find(|row| row.kind == kind).cloned().unwrap();
    assert_eq!((row("Events").added, row("Events").kept), (1, 1));
    assert_eq!((row("Routines").added, row("Routines").kept), (2, 2));
    assert_eq!((row("Weekly plan history").added, row("Weekly plan history").kept), (2, 1));
    let snapshots = row("Sleep plan snapshots");
    assert_eq!((snapshots.added, snapshots.replaced, snapshots.kept), (1, 0, 0));
    let prompt = row("Plan prompt preference");
    assert_eq!((prompt.added, prompt.kept), (0, 1));
    core.import_settings(&incoming, None).unwrap();

    let records = core.local_records();
    assert_eq!(records.events.events.len(), 2);
    assert_eq!(records.events.events.iter().find(|e| e.id == event_id).unwrap().name, "10K");
    assert_eq!(records.routines.items.len(), 2);
    assert_eq!(records.routines.items.iter().find(|i| i.id == routine_id).unwrap().name, "Wall sit");
    assert_eq!(records.routines.reports.len(), 2);
    let plan: Value = serde_json::from_str(&store.setting("plan").unwrap()).unwrap();
    assert_eq!(plan["versions"]["2024-12-23"]["strengthSessions"], 3, "an existing week is kept");
    assert_eq!(plan["versions"]["2025-01-06"]["strengthSessions"], 2, "a missing week is added");
    assert_eq!(plan["paused"], json!(["2025-01-06"]));
    assert!(store.setting(&night).is_some());
    assert!(core.settings().plan_prompt, "this phone's own preference is kept");

    // A phone that never saved settings takes the exported prompt preference.
    let (fresh_dir, fresh_store, fresh) = open("fill-missing-fresh");
    let only_prompt = archive(&fresh_dir, "prompt.zip", &json!({"settings": {"planPrompt": false}}), &manifest(json!("11")));
    fresh.import_settings(&only_prompt, None).unwrap();
    assert!(fresh_store.setting("settings").is_some());
    assert!(!fresh.settings().plan_prompt);
}

#[test]
fn a_newer_record_on_this_phone_is_left_alone_and_milestones_stay_out() {
    let (dir, store, core) = open("read-only");
    let newer = json!({"schema": 99, "versions": [{"from": "2025-03-10"}]});
    store.set_setting("local:goals", &newer.to_string()).unwrap();

    let path = archive(&dir, "read-only.zip", &json!({
        "local:goals": goals_row(vec![goal_version("2025-03-10", "2025-03-10T10:00:00", 4)]),
        "local:events": {"schema": 1, "events": [{"id": "e1", "name": "10K", "date": "2025-04-01", "sport": null}]},
        "local:milestones": {"schema": 1, "items": [{"id": "m1", "value": 1.0, "date": "2025-03-01",
            "activity": null, "seen": false, "corrected": false, "kind": null, "title": "Best", "route": null,
            "withdrawn": false}]}
    }), &manifest(json!("11")));
    let preview = core.import_preview(&path, None).unwrap();
    assert!(preview.skipped.iter().any(|skip| skip.what == "Goals"));
    assert!(preview.skipped.iter().any(|skip| skip.what == "Milestones and records"));
    assert!(preview.rows.iter().all(|row| row.kind != "Goals"));
    core.import_settings(&path, None).unwrap();

    assert_eq!(store.setting("local:goals").as_deref(), Some(newer.to_string().as_str()));
    assert_eq!(core.local_records().events.events.len(), 1);
    assert!(store.setting("local:milestones").is_none(), "milestones are rebuilt from Garmin");
}

#[test]
fn recent_garmin_changes_are_account_history_and_never_imported() {
    let (dir, store, core) = open("garmin-changes");
    let log = |schema: u64| json!({"schema": schema, "entries": [{"at": "2025-03-12T08:00:00Z",
        "resource": "Watch alarms", "path": "/device-service/deviceservice/device-info/settings/3001",
        "fields": [{"field": "alarm:7.alarmTime", "label": "Alarm time", "before": "06:30", "after": "07:00",
            "beforeValue": 390, "afterValue": 420}],
        "outcome": "applied", "message": null, "mismatches": [], "device": "3001", "downloadBaseline": 1}]});
    let events = json!({"schema": 1, "events": [{"id": "e1", "name": "10K", "date": "2025-04-01", "sport": null}]});
    // A newer log schema is skipped too: it is not a row the import would write.
    for schema in [1, 99] {
        let path = archive(&dir, &format!("changes-{schema}.zip"),
            &json!({"garmin:changes": log(schema), "local:events": events}), &manifest(json!("11")));
        let preview = core.import_preview(&path, None).unwrap();
        let skip = preview.skipped.iter().find(|skip| skip.what == "Recent Garmin changes").expect("the preview names the log");
        assert!(skip.reason.starts_with("Not imported"));
        assert!(preview.rows.iter().all(|row| !row.kind.contains("Garmin")));
        core.import_settings(&path, None).unwrap();
        assert!(store.setting("garmin:changes").is_none(), "the log never crosses phones");
        assert!(core.garmin_changes().is_empty());
    }
    assert_eq!(core.local_records().events.events.len(), 1);

    // Without a log in the file, the preview doesn't mention one.
    let path = archive(&dir, "no-changes.zip", &json!({"local:events": events}), &manifest(json!("11")));
    assert!(core.import_preview(&path, None).unwrap().skipped.iter().all(|skip| skip.what != "Recent Garmin changes"));
}

#[test]
fn a_saved_sleep_plan_is_prospective_evidence_and_is_never_replaced() {
    let (dir, store, core) = open("snapshots");
    let (night, saved) = saved_sleep_plan();
    let one = archive(&dir, "one.zip", &json!({night.clone(): saved.clone()}), &manifest(json!("11")));
    let preview = core.import_preview(&one, None).unwrap();
    assert!(preview.rows.iter().any(|row| row.kind == "Sleep plan snapshots" && row.added == 1));
    core.import_settings(&one, None).unwrap();
    let stored = store.setting(&night).unwrap();

    // A second import of a row this phone already has is kept, not replaced.
    let mut changed = saved;
    changed["capturedAt"] = json!("2025-03-14T06:00:00+00:00");
    let two = archive(&dir, "two.zip", &json!({night.clone(): changed}), &manifest(json!("11")));
    let preview = core.import_preview(&two, None).unwrap();
    assert!(preview.rows.iter().all(|row| row.added == 0 && row.replaced == 0));
    core.import_settings(&two, None).unwrap();
    assert_eq!(store.setting(&night).as_deref(), Some(stored.as_str()));
}

#[test]
fn the_manifest_names_the_account_that_made_the_export() {
    let (dir, _store, core) = open("manifest");
    let path = dir.path("archive.zip");
    core.export(&path, "test").unwrap();
    let mut zip = zip::ZipArchive::new(std::fs::File::open(&path).unwrap()).unwrap();
    let mut body = String::new();
    zip.by_name("manifest.json").unwrap().read_to_string(&mut body).unwrap();
    let manifest: Value = serde_json::from_str(&body).unwrap();
    assert_eq!(manifest["archive"]["format"], "raring-export");
    assert_eq!(manifest["archive"]["version"], 1);
    assert_eq!(manifest["archive"]["account"], "11");
    assert!(!body.contains("account_session") && !body.contains("account_pending"));

    // A store with no verified account names none, and cannot be imported.
    let (out_dir, _store, out) = signed_out("manifest-unbound");
    let path = out_dir.path("archive.zip");
    out.export(&path, "test").unwrap();
    let mut zip = zip::ZipArchive::new(std::fs::File::open(&path).unwrap()).unwrap();
    let mut body = String::new();
    zip.by_name("manifest.json").unwrap().read_to_string(&mut body).unwrap();
    let manifest: Value = serde_json::from_str(&body).unwrap();
    assert!(manifest["archive"]["account"].is_null());
    let (dir, _store, reader) = open("manifest-unbound-reader");
    let copy = archive(&dir, "unbound.zip", &export_settings(&out, &out_dir, "unbound"), &manifest);
    assert!(matches!(reader.import_preview(&copy, None), Err(ImportError::MissingAccount)));
}

#[test]
fn plan_row_and_settings_are_part_of_the_round_trip() {
    let (source_dir, source_store, source) = open("plan-source");
    source_store
        .set_setting("plan", &json!({"versions": {"2024-12-23": {"preset": "strength", "strengthSessions": 3,
            "intensityMin": 120, "zone2Min": 90, "asleepNights": 2, "restDays": [2, 6]}}, "paused": ["2024-12-30", "2025-01-06"]}).to_string())
        .unwrap();
    let settings = export_settings(&source, &source_dir, "plan");
    let path = archive(&source_dir, "plan.zip", &settings, &manifest(json!("11")));
    let (_target_dir, target_store, target) = open("plan-target");
    target.import_settings(&path, None).unwrap();
    let plan = |store: &Store| serde_json::from_str::<Value>(&store.setting("plan").unwrap()).unwrap();
    assert_eq!(plan(&target_store), plan(&source_store));
    assert_eq!(target.local_records().goals, source.local_records().goals);
    assert_eq!(target.local_records().circumstances, source.local_records().circumstances);
    // The row is read and migrated by the first local read, as on the source.
    assert_eq!(target.local_records(), source.local_records());
}

#[test]
fn a_refused_file_writes_nothing() {
    let (dir, store, core) = open("atomic");
    act(&core, LocalAction::RestToday).unwrap();
    let before = core.local_records();
    let path = archive(&dir, "bad-row.zip", &json!({
        "local:events": {"schema": 1, "events": [{"id": "e1", "name": "10K", "date": "2025-04-01", "sport": null}]},
        "local:routines": {"schema": 1, "items": [],
            "reports": [{"id": "missing", "date": "2025-03-13", "at": "2025-03-13T09:00:00"}]}
    }), &manifest(json!("11")));
    assert!(matches!(core.import_preview(&path, None), Err(ImportError::Unreadable(_))));
    assert!(matches!(core.import_settings(&path, None), Err(ImportError::Unreadable(_))));
    assert!(store.setting("local:events").is_none(), "no row may be written");
    assert_eq!(core.local_records(), before);
}
