//! Local records: storage, migration from the earlier weekly plan, every
//! change, export, account isolation and newer-schema protection.
use chrono::{Local, TimeZone};
use raring_core::{local, store::Store, view::*, Raring};
use serde_json::{json, Value};
use std::io::Read;

// Thursday 13 March 2025; the plan week starts Monday 10 March.
fn now() -> chrono::DateTime<Local> {
    Local.with_ymd_and_hms(2025, 3, 13, 10, 0, 0).unwrap()
}
fn evening() -> chrono::DateTime<Local> {
    Local.with_ymd_and_hms(2025, 3, 13, 21, 30, 0).unwrap()
}

struct Dir(std::path::PathBuf);
impl Dir {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!("raring-local-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }
    fn db(&self) -> std::path::PathBuf {
        self.0.join("raring.db")
    }
}
impl Drop for Dir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn open(name: &str) -> (Dir, Store, Raring) {
    let dir = Dir::new(name);
    let store = Store::open(dir.db()).unwrap();
    let core = Raring::open(dir.db()).unwrap();
    (dir, store, core)
}

fn bind(store: &Store, id: u64) {
    let profile = json!({"profileId": id, "displayName": "synthetic"});
    store.put("profile", "me", "now", &profile).unwrap();
    store
        .set_setting("account_session", &json!({"identity": id.to_string(), "active": true}).to_string())
        .unwrap();
}

fn activity(store: &Store, id: u64, start: &str, kind: &str) {
    store
        .put("activity", &id.to_string(), "now", &json!({"activityId": id, "activityTypeDTO": {"typeKey": kind},
            "summaryDTO": {"startTimeLocal": start, "duration": 1800}}))
        .unwrap();
}

fn act(core: &Raring, action: LocalAction) -> raring_core::Result<()> {
    core.local_action(&action, now())
}

fn targets(strength: u32) -> GoalTargets {
    GoalTargets { strength_sessions: strength, zone2_min: 90, ..Default::default() }
}

fn add(core: &Raring, date: &str) -> String {
    act(core, LocalAction::AddIntention {
        date: date.into(), time: Some("18:30".into()), sport: Sport::Strength, type_key: None, name: None,
        duration_min: Some(45), allocation: Allocation { strength_sessions: 1, ..Default::default() },
        kind: IntentionKind::Workout, garmin_id: None,
    })
    .unwrap();
    core.local_records().intentions.items.last().unwrap().id.clone()
}

fn legacy_plan(intensity: Value, asleep: u32) -> String {
    json!({
        "versions": {
            "2025-02-24": {"preset": "strength", "strengthSessions": 3, "intensityMin": intensity,
                "zone2Min": 90, "asleepNights": asleep, "restDays": [6, 2, 9]},
            "2025-03-10": {"preset": "custom", "strengthSessions": 2, "intensityMin": 120,
                "zone2Min": 0, "asleepNights": 0, "restDays": []}
        },
        "paused": ["2025-03-03"]
    })
    .to_string()
}

#[test]
fn an_empty_database_stays_empty_so_a_first_sign_in_can_bind() {
    let (_dir, store, core) = open("empty");
    assert_eq!(core.local_records(), local::LocalRecords::default());
    assert!(store.setting("local:schema").is_none());
    let profile = json!({"profileId": 101, "displayName": "synthetic"});
    assert!(store.bind_account("101", &profile, "now").unwrap());
}

#[test]
fn an_unconfigured_plan_migrates_to_no_goals() {
    let (_dir, store, core) = open("unconfigured");
    store.set_setting("plan", r#"{"versions":{},"paused":[]}"#).unwrap();
    let r = core.local_records();
    assert!(r.goals.versions.is_empty(), "the Strength preview preset is never adopted");
    assert!(r.circumstances.pauses.is_empty());
    assert_eq!(store.setting("local:schema").as_deref(), Some("1"));
}

#[test]
fn saved_plan_versions_become_goals_and_keep_the_old_row() {
    let (_dir, store, core) = open("migrate");
    let plan = legacy_plan(Value::Null, 5);
    store.set_setting("plan", &plan).unwrap();
    store.set_setting("sleep_plan:2025-03-01", r#"{"plan":{"night":"2025-03-01"}}"#).unwrap();
    let r = core.local_records();
    let [first, second] = &r.goals.versions[..] else { panic!("two versions") };
    assert_eq!(first.from, "2025-02-24");
    assert_eq!(first.source, GoalSource::Migrated);
    let t = &first.targets;
    assert_eq!((t.strength_sessions, t.zone2_min, t.intensity_min), (3, 90, 0));
    assert!(t.intensity_garmin_goal, "no saved intensity follows Garmin's weekly goal");
    assert_eq!((t.sleep_nights, t.sleep_kind, t.sleep_time.clone()), (5, SleepTimingKind::SavedPlan, None));
    assert_eq!(t.rest_days, vec![6, 2]);
    assert_eq!(second.from, "2025-03-10");
    assert_eq!((second.targets.intensity_min, second.targets.intensity_garmin_goal), (120, false));
    assert_eq!(second.targets.sleep_nights, 0);
    let [pause] = &r.circumstances.pauses[..] else { panic!("one pause") };
    assert_eq!((pause.since.as_str(), pause.until.as_deref()), ("2025-03-03", Some("2025-03-09")));
    assert_eq!(store.setting("plan").unwrap(), plan, "earlier weeks keep their original evaluation");
    assert!(store.setting("sleep_plan:2025-03-01").is_some());

    // Once migrated, later edits are not overwritten by the old row.
    act(&core, LocalAction::SetGoals { targets: targets(4) }).unwrap();
    let again = core.local_records();
    assert_eq!(again.goals.versions.last().unwrap().targets.strength_sessions, 4);
    assert_eq!(again.goals.versions.len(), 3);
}

#[test]
fn a_malformed_plan_row_migrates_to_nothing() {
    let (_dir, store, core) = open("malformed");
    store.set_setting("plan", "not json").unwrap();
    assert!(core.local_records().goals.versions.is_empty());
    assert_eq!(store.setting("plan").as_deref(), Some("not json"));
}

#[test]
fn the_old_setup_screen_writes_through_after_migration() {
    let (_dir, store, core) = open("mirror");
    store.set_setting("plan", &legacy_plan(json!(150), 0)).unwrap();
    let _ = core.local_records();
    let update = PlanUpdate {
        targets: PlanTargets { strength_sessions: 1, ..PlanTargets::preset(PlanPreset::Custom) },
        paused: true,
    };
    core.set_plan("2025-03-13", &update).unwrap();
    let r = core.local_records();
    let v = r.goals.versions.iter().find(|v| v.from == "2025-03-10").unwrap();
    assert_eq!(v.targets.strength_sessions, 1);
    assert_eq!(r.goals.versions.len(), 2, "the week's version is replaced, not duplicated");
    assert!(r.circumstances.pauses.iter().any(|p| p.since == "2025-03-10" && p.until.as_deref() == Some("2025-03-16")));
    core.set_plan("2025-03-13", &PlanUpdate { paused: false, ..update }).unwrap();
    assert!(!core.local_records().circumstances.pauses.iter().any(|p| p.since == "2025-03-10"));
}

#[test]
fn first_goals_apply_from_the_week_start_and_changes_from_today() {
    let (_dir, _store, core) = open("goals");
    act(&core, LocalAction::SetGoals { targets: targets(2) }).unwrap();
    let v = core.local_records().goals.versions;
    assert_eq!((v[0].from.as_str(), v[0].source), ("2025-03-10", GoalSource::Setup));
    act(&core, LocalAction::SetGoals { targets: targets(3) }).unwrap();
    let v = core.local_records().goals.versions;
    assert_eq!(v.len(), 2);
    assert_eq!((v[1].from.as_str(), v[1].source), ("2025-03-13", GoalSource::Revision));
    assert_eq!(v[0].targets.strength_sessions, 2, "the week's original targets stay");
    act(&core, LocalAction::SetGoals { targets: targets(1) }).unwrap();
    let v = core.local_records().goals.versions;
    assert_eq!(v.len(), 2, "same-day changes replace that day's change");
    assert_eq!(v[1].targets.strength_sessions, 1);

    let bad = |t: GoalTargets| act(&core, LocalAction::SetGoals { targets: t }).is_err();
    assert!(bad(GoalTargets { strength_sessions: 15, ..Default::default() }));
    assert!(bad(GoalTargets { sleep_nights: 3, ..Default::default() }), "asleep-by needs a time");
    assert!(bad(GoalTargets { sleep_nights: 3, sleep_kind: SleepTimingKind::SavedPlan, ..Default::default() }),
        "a saved-plan observation is never chosen anew");
    assert!(bad(GoalTargets { sleep_nights: 3, sleep_time: Some("25:00".into()), ..Default::default() }));
    act(&core, LocalAction::SetGoals { targets: GoalTargets { sleep_nights: 3, sleep_time: Some("22:30".into()), ..Default::default() } }).unwrap();
}

#[test]
fn a_migrated_observation_can_be_kept_but_not_restarted() {
    let (_dir, store, core) = open("observation");
    store.set_setting("plan", &json!({"versions": {"2025-03-10": {"preset": "sleep", "strengthSessions": 2,
        "intensityMin": null, "zone2Min": 60, "asleepNights": 5, "restDays": []}}, "paused": []}).to_string()).unwrap();
    let kept = core.local_records().goals.versions.last().unwrap().targets.clone();
    assert_eq!((kept.sleep_kind, kept.sleep_nights), (SleepTimingKind::SavedPlan, 5));
    act(&core, LocalAction::SetGoals { targets: GoalTargets { strength_sessions: 1, ..kept.clone() } }).unwrap();
    act(&core, LocalAction::SetGoals { targets: GoalTargets { sleep_nights: 0, ..kept.clone() } }).unwrap();
    assert!(act(&core, LocalAction::SetGoals { targets: kept }).is_err(), "once turned off it is not restarted");
}

#[test]
fn sessions_are_added_moved_let_go_and_marked_missed_by_the_person() {
    let (_dir, _store, core) = open("intentions");
    assert!(act(&core, LocalAction::AddIntention {
        date: "2025-03-12".into(), time: None, sport: Sport::Running, type_key: None, name: None, duration_min: None,
        allocation: Allocation::default(), kind: IntentionKind::Workout, garmin_id: None,
    }).is_err(), "historical days never get new sessions");
    let id = add(&core, "2025-03-13");
    let item = |core: &Raring| core.local_records().intentions.items.into_iter().find(|i| i.id == id).unwrap();
    assert_eq!(item(&core).status, IntentionStatus::Open);
    assert!(act(&core, LocalAction::SetIntentionStatus { id: id.clone(), status: IntentionStatus::Missed }).is_err(),
        "today's session isn't missed");
    assert!(act(&core, LocalAction::MoveIntention { id: id.clone(), date: "2025-03-12".into() }).is_err(), "future days only");
    act(&core, LocalAction::EditIntention { id: id.clone(), time: Some("07:00".into()), name: Some(" Legs ".into()),
        duration_min: Some(40), allocation: Allocation { strength_sessions: 1, ..Default::default() } }).unwrap();
    let edited = item(&core);
    assert_eq!((edited.date.as_str(), edited.time.as_deref(), edited.name.as_deref()), ("2025-03-13", Some("07:00"), Some("Legs")));
    act(&core, LocalAction::MoveIntention { id: id.clone(), date: "2025-03-15".into() }).unwrap();
    let moved = item(&core);
    assert_eq!(moved.date, "2025-03-15");
    let last = moved.history.last().unwrap();
    assert_eq!((last.change, last.from.as_deref(), last.to.as_deref()), (IntentionChange::Moved, Some("2025-03-13"), Some("2025-03-15")));
    act(&core, LocalAction::SetIntentionStatus { id: id.clone(), status: IntentionStatus::LetGo }).unwrap();
    assert_eq!(item(&core).status, IntentionStatus::LetGo);
    act(&core, LocalAction::SetIntentionStatus { id: id.clone(), status: IntentionStatus::Open }).unwrap();
    assert_eq!(item(&core).history.last().unwrap().change, IntentionChange::Reopened);
    act(&core, LocalAction::RemoveIntention { id: id.clone() }).unwrap();
    assert!(core.local_records().intentions.items.is_empty());
}

#[test]
fn a_passed_session_is_kept_and_only_its_person_marks_it_missed() {
    let (dir, _store, core) = open("passed");
    let id = add(&core, "2025-03-13");
    drop(core);
    let core = Raring::open(dir.db()).unwrap();
    let tomorrow = Local.with_ymd_and_hms(2025, 3, 14, 9, 0, 0).unwrap();
    assert!(core.local_action(&LocalAction::RemoveIntention { id: id.clone() }, tomorrow).is_err());
    core.local_action(&LocalAction::SetIntentionStatus { id: id.clone(), status: IntentionStatus::Missed }, tomorrow).unwrap();
    assert_eq!(core.local_records().intentions.items[0].status, IntentionStatus::Missed);
}

#[test]
fn links_pair_one_recording_with_one_plan_and_unlinks_keep_pairs_apart() {
    let (_dir, store, core) = open("links");
    activity(&store, 7001, "2025-03-13 07:10:00", "strength_training");
    let a = add(&core, "2025-03-13");
    let b = add(&core, "2025-03-13");
    assert!(act(&core, LocalAction::Link { plan_id: a.clone(), activity_id: "999".into() }).is_err());
    assert!(act(&core, LocalAction::Link { plan_id: "i-unknown".into(), activity_id: "7001".into() }).is_err());
    act(&core, LocalAction::Link { plan_id: a.clone(), activity_id: "7001".into() }).unwrap();
    act(&core, LocalAction::Link { plan_id: b.clone(), activity_id: "7001".into() }).unwrap();
    let links = core.local_records().links;
    assert_eq!(links.links.len(), 1, "one recording completes one plan item");
    assert_eq!(links.links[0].plan_id, b);
    act(&core, LocalAction::Unlink { plan_id: b.clone(), activity_id: "7001".into() }).unwrap();
    let links = core.local_records().links;
    assert!(links.links.is_empty());
    assert_eq!((links.unlinks[0].plan_id.as_str(), links.unlinks[0].activity_id.as_str()), (b.as_str(), "7001"));
    act(&core, LocalAction::Link { plan_id: b.clone(), activity_id: "7001".into() }).unwrap();
    assert!(core.local_records().links.unlinks.is_empty(), "linking again replaces the unlink");
    act(&core, LocalAction::MoveIntention { id: b.clone(), date: "2025-03-14".into() }).unwrap();
    let r = core.local_records();
    assert!(r.links.links.is_empty(), "a moved session no longer fits its recording");
    assert_eq!(r.intentions.items[1].history.last().unwrap().change, IntentionChange::LinkCleared);
    act(&core, LocalAction::Link { plan_id: "garmin:42".into(), activity_id: "7001".into() }).unwrap();
}

#[test]
fn rest_pause_limits_and_dismissals_are_explicit_and_end_only_when_asked() {
    let (_dir, _store, core) = open("circumstances");
    act(&core, LocalAction::RestToday).unwrap();
    act(&core, LocalAction::RestToday).unwrap();
    assert_eq!(core.local_records().circumstances.rest.len(), 1);
    act(&core, LocalAction::CancelRest).unwrap();
    assert!(core.local_records().circumstances.rest.is_empty());

    assert!(act(&core, LocalAction::Resume).is_err());
    assert!(act(&core, LocalAction::Pause { reason: PauseReason::Ill, review: Some("2025-03-13".into()) }).is_err());
    act(&core, LocalAction::Pause { reason: PauseReason::Pause, review: Some("2025-03-20".into()) }).unwrap();
    act(&core, LocalAction::Pause { reason: PauseReason::Ill, review: None }).unwrap();
    let pauses = core.local_records().circumstances.pauses;
    assert_eq!(pauses.len(), 1, "one open pause at a time");
    assert_eq!((pauses[0].reason, pauses[0].since.as_str(), pauses[0].until.as_deref()), (PauseReason::Ill, "2025-03-13", None));
    act(&core, LocalAction::Resume).unwrap();
    assert!(core.local_records().circumstances.pauses.is_empty(), "a pause begun and ended today leaves no interval");
    act(&core, LocalAction::Pause { reason: PauseReason::Ill, review: None }).unwrap();
    let later = Local.with_ymd_and_hms(2025, 3, 17, 8, 0, 0).unwrap();
    core.local_action(&LocalAction::Resume, later).unwrap();
    let p = &core.local_records().circumstances.pauses[0];
    assert_eq!((p.since.as_str(), p.until.as_deref()), ("2025-03-13", Some("2025-03-16")), "the interval survives resume");

    act(&core, LocalAction::SetLimits { limits: Limits { excluded: vec![Sport::Running, Sport::Cycling, Sport::Running], note: "  No overhead lifts ".into() } }).unwrap();
    let limits = core.local_records().circumstances.limits;
    assert_eq!(limits.excluded, vec![Sport::Running, Sport::Cycling]);
    assert_eq!(limits.note, "No overhead lifts");
    assert!(act(&core, LocalAction::SetLimits { limits: Limits { excluded: vec![], note: "x".repeat(501) } }).is_err());

    act(&core, LocalAction::Dismiss { id: "suggest".into() }).unwrap();
    act(&core, LocalAction::Dismiss { id: "suggest".into() }).unwrap();
    assert_eq!(core.local_records().circumstances.dismissals.len(), 1);
    let month = Local.with_ymd_and_hms(2025, 4, 20, 8, 0, 0).unwrap();
    core.local_action(&LocalAction::Dismiss { id: "other".into() }, month).unwrap();
    let d = core.local_records().circumstances.dismissals;
    assert_eq!(d.len(), 1, "day-long dismissals are pruned after 30 days");
}

#[test]
fn a_wind_down_after_midnight_belongs_to_the_evening_before() {
    let (_dir, _store, core) = open("winddown");
    core.local_action(&LocalAction::ReportWindDown, evening()).unwrap();
    let late = Local.with_ymd_and_hms(2025, 3, 15, 0, 40, 0).unwrap();
    core.local_action(&LocalAction::ReportWindDown, late).unwrap();
    let dates: Vec<String> = core.local_records().circumstances.wind_downs.into_iter().map(|w| w.date).collect();
    assert_eq!(dates, ["2025-03-13", "2025-03-14"]);
}

#[test]
fn prompts_and_events_are_kept_per_week_and_date() {
    let (_dir, _store, core) = open("events");
    act(&core, LocalAction::SetPromptWeek { week: "2025-03-10".into(), choice: PromptChoice::Dismissed }).unwrap();
    assert_eq!(core.local_records().reviews.prompts["2025-03-10"], PromptChoice::Dismissed);
    assert!(act(&core, LocalAction::AddEvent { name: "Winter 10K".into(), date: "2025-03-01".into(), sport: None }).is_err());
    assert!(act(&core, LocalAction::AddEvent { name: "  ".into(), date: "2025-04-01".into(), sport: None }).is_err());
    act(&core, LocalAction::AddEvent { name: "Winter 10K".into(), date: "2025-04-01".into(), sport: Some(Sport::Running) }).unwrap();
    let id = core.local_records().events.events[0].id.clone();
    act(&core, LocalAction::RemoveEvent { id }).unwrap();
    assert!(core.local_records().events.events.is_empty());
}

#[test]
fn a_newer_schema_is_shown_but_never_rewritten() {
    let (_dir, store, core) = open("newer");
    let row = json!({"schema": 2, "versions": [{"from": "2025-03-10", "targets": {"strengthSessions": 2},
        "source": "setup", "savedAt": "2025-03-10T08:00:00"}], "futureField": true}).to_string();
    store.set_setting("local:goals", &row).unwrap();
    let r = core.local_records();
    assert_eq!(r.goals.versions[0].targets.strength_sessions, 2);
    assert_eq!(r.read_only, vec!["local:goals"]);
    assert!(act(&core, LocalAction::SetGoals { targets: targets(3) }).is_err());
    assert_eq!(store.setting("local:goals").unwrap(), row);
    act(&core, LocalAction::RestToday).unwrap();
    store.set_setting("local:intentions", "{broken").unwrap();
    assert!(core.local_records().read_only.contains(&"local:intentions"));
    assert!(core.local_records().intentions.items.is_empty());
    assert_eq!(store.setting("local:intentions").as_deref(), Some("{broken"));
}

#[test]
fn every_record_is_exported_without_credentials() {
    let (dir, store, core) = open("export");
    activity(&store, 7001, "2025-03-13 07:10:00", "running");
    act(&core, LocalAction::SetGoals { targets: targets(2) }).unwrap();
    let id = add(&core, "2025-03-14");
    act(&core, LocalAction::Link { plan_id: id, activity_id: "7001".into() }).unwrap();
    act(&core, LocalAction::SetLimits { limits: Limits { excluded: vec![Sport::Running], note: "knee".into() } }).unwrap();
    act(&core, LocalAction::SetPromptWeek { week: "2025-03-10".into(), choice: PromptChoice::Planned }).unwrap();
    act(&core, LocalAction::AddEvent { name: "10K".into(), date: "2025-04-01".into(), sport: None }).unwrap();
    act(&core, LocalAction::AddRoutine { name: "Wall sit".into(), how: None }).unwrap();
    let path = dir.0.join("export.zip");
    core.export(&path, "test").unwrap();
    let mut archive = zip::ZipArchive::new(std::fs::File::open(&path).unwrap()).unwrap();
    let mut body = String::new();
    archive.by_name("settings.json").unwrap().read_to_string(&mut body).unwrap();
    let settings: Value = serde_json::from_str(&body).unwrap();
    for key in ["local:goals", "local:intentions", "local:links", "local:circumstances", "local:reviews", "local:events", "local:routines"] {
        assert_eq!(settings[key]["schema"], 1, "{key} is exported");
    }
    assert_eq!(settings["local:circumstances"]["limits"]["note"], "knee");
    assert_eq!(settings["local:schema"], 1);
}

#[test]
fn records_survive_reauthentication_and_are_erased_on_switch_and_sign_out() {
    let (_dir, store, core) = open("isolation");
    bind(&store, 101);
    act(&core, LocalAction::SetGoals { targets: targets(2) }).unwrap();
    act(&core, LocalAction::RestToday).unwrap();
    act(&core, LocalAction::AddRoutine { name: "Wall sit".into(), how: None }).unwrap();
    let same = json!({"profileId": 101, "displayName": "synthetic"});
    assert!(store.bind_account("101", &same, "later").unwrap());
    assert_eq!(core.local_records().goals.versions.len(), 1, "same-account sign-in keeps local records");
    let other = json!({"profileId": 202, "displayName": "synthetic"});
    assert!(store.bind_account("202", &other, "later").unwrap());
    assert_eq!(core.local_records(), local::LocalRecords::default(), "a verified switch clears them");

    act(&core, LocalAction::SetGoals { targets: targets(2) }).unwrap();
    core.forget_account().unwrap();
    assert_eq!(core.local_records(), local::LocalRecords::default(), "sign-out erases them");
}

#[test]
fn a_queued_change_for_another_account_is_refused() {
    let (_dir, store, core) = open("queued");
    bind(&store, 101);
    assert!(core.local_action_for_account(&LocalAction::RestToday, now(), Some("202".into())).is_err());
    assert!(core.local_records().circumstances.rest.is_empty());
    core.local_action_for_account(&LocalAction::RestToday, now(), Some("101".into())).unwrap();
    assert_eq!(core.local_records().circumstances.rest.len(), 1);
}

#[test]
fn sport_families_keep_mixed_types_apart() {
    assert_eq!(local::sport_of("trail_running"), Sport::Running);
    assert_eq!(local::sport_of("mountain_biking"), Sport::Cycling);
    assert_eq!(local::sport_of("virtual_ride"), Sport::Cycling);
    assert_eq!(local::sport_of("strength_training"), Sport::Strength);
    assert_eq!(local::sport_of("hiit"), Sport::Other, "HIIT is never strength");
    assert_eq!(local::sport_of("lap_swimming"), Sport::Swimming);
}

#[test]
fn a_newer_row_doesnt_stop_the_other_rows_migrating_and_the_migration_stays_unfinished() {
    let (_dir, store, core) = open("partial-migration");
    store.set_setting("plan", &legacy_plan(Value::Null, 0)).unwrap();
    let newer = json!({"schema": 2, "versions": []}).to_string();
    store.set_setting("local:goals", &newer).unwrap();
    let r = core.local_records();
    assert_eq!(r.circumstances.pauses.len(), 1, "paused weeks still migrate");
    assert_eq!(store.setting("local:goals").unwrap(), newer);
    assert!(store.setting("local:schema").is_none(), "unfinished, so a build that can will finish it");
}

#[test]
fn a_refused_mirror_leaves_the_old_plan_row_unchanged() {
    let (_dir, store, core) = open("refused-mirror");
    let plan = legacy_plan(json!(150), 0);
    store.set_setting("plan", &plan).unwrap();
    let _ = core.local_records();
    store.set_setting("local:goals", &json!({"schema": 2, "versions": []}).to_string()).unwrap();
    let update = PlanUpdate { targets: PlanTargets::preset(PlanPreset::Custom), paused: false };
    assert!(core.set_plan("2025-03-13", &update).is_err());
    assert_eq!(store.setting("plan").unwrap(), plan);
}

#[test]
fn a_lighter_week_halves_this_week_and_next_week_returns() {
    use chrono::TimeZone;
    let (_dir, _store, core) = open("lighter");
    let mon = chrono::Local.with_ymd_and_hms(2025, 3, 10, 9, 0, 0).unwrap();
    let wed = chrono::Local.with_ymd_and_hms(2025, 3, 12, 9, 0, 0).unwrap();
    assert!(core.local_action(&LocalAction::LighterWeek, wed).is_err(), "no goals to lighten");
    let mut t = targets(3);
    t.zone2_min = 90;
    core.local_action(&LocalAction::SetGoals { targets: t }, mon).unwrap();
    assert!(core.local_action(&LocalAction::LighterWeek, mon).is_err(), "first targets stay on record");
    core.local_action(&LocalAction::LighterWeek, wed).unwrap();
    let v = core.local_records().goals.versions;
    let at = |d: &str| v.iter().rev().find(|x| x.from.as_str() <= d).unwrap().targets.clone();
    assert_eq!((at("2025-03-10").strength_sessions, at("2025-03-10").zone2_min), (3, 90), "Monday's targets are kept");
    assert_eq!((at("2025-03-12").strength_sessions, at("2025-03-12").zone2_min), (1, 45), "lighter from today");
    assert_eq!((at("2025-03-17").strength_sessions, at("2025-03-17").zone2_min), (3, 90), "next week returns");
    let b = core.plan_board("2025-03-12", wed).unwrap();
    let s = b.ledger.iter().find(|l| l.kind == GoalKind::Strength).unwrap();
    assert_eq!((s.target, s.original), (1., Some(3.)), "the week shows its original target beside the lighter one");
}
