//! Your day on one axis, nap Body Battery change and keeping today's session.
use chrono::{Local, TimeZone};
use raring_core::{day, store::Store, view::*, Raring};
use serde_json::{json, Value};

const DAY: &str = "2025-03-13";

fn at(d: u32, h: u32, m: u32) -> chrono::DateTime<Local> {
    Local.with_ymd_and_hms(2025, 3, d, h, m, 0).unwrap()
}
fn fx(name: &str) -> (std::path::PathBuf, Store, Raring) {
    let dir = std::env::temp_dir().join(format!("raring-yourday-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let store = Store::open(dir.join("raring.db")).unwrap();
    let core = Raring::open(dir.join("raring.db")).unwrap();
    (dir, store, core)
}
fn midnight_ms() -> i64 {
    chrono::NaiveDate::from_ymd_opt(2025, 3, 13).unwrap().and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp_millis()
}
/// Garmin's intraday Body Battery: (three-minute slot, status, level).
fn battery(store: &Store, samples: &[(i64, &str, f64)]) {
    let bb: Vec<Value> = samples.iter().map(|(slot, status, v)| json!([midnight_ms() + slot * 180_000, status, v, 3.0])).collect();
    store.put("stress", DAY, "now", &json!({"startTimestampGMT": "2025-03-13T00:00:00.0", "bodyBatteryValuesArray": bb})).unwrap();
}
fn heart(store: &Store, minutes: std::ops::Range<i64>) {
    let hr: Vec<Value> = minutes.step_by(2).map(|m| json!([midnight_ms() + m * 60_000, 60])).collect();
    store.put("daily_hr", DAY, "now", &json!({"startTimestampGMT": "2025-03-13T00:00:00.0", "heartRateValues": hr})).unwrap();
}
fn night(store: &Store, start: &str, end: &str, levels: &[(&str, &str, f64)]) {
    let levels: Vec<Value> = levels.iter().map(|(a, b, l)| json!({"startGMT": a, "endGMT": b, "activityLevel": l})).collect();
    store.put("sleep", DAY, "now", &json!({"dailySleepDTO": {"sleepStartTimestampLocal": start, "sleepEndTimestampLocal": end, "sleepTimeSeconds": 27000},
        "sleepLevels": levels})).unwrap();
}
fn nap(store: &Store, start: &str, end: &str) {
    store.put("naps", DAY, "now", &json!([{"napStartTimestampGMT": start, "napStartTimeOffset": 0,
        "napEndTimestampGMT": end, "napEndTimeOffset": 0, "napTimeSec": 1800}])).unwrap();
}
/// The timeline at `now` with the watch's upload at `upload` (local time).
fn timeline(store: &Store, core: &Raring, now: chrono::DateTime<Local>, upload: Option<chrono::DateTime<Local>>, day_minutes: f64) -> DayTimeline {
    let mut view = core.today_at(DAY, now);
    view.watch_last_upload = upload.map(|u| u.to_rfc3339());
    day::timeline(store, &view, now.date_naive(), now.fixed_offset(), day_minutes)
}
type Levels<'a> = &'a [(&'a str, &'a str, f64)];
type Samples<'a> = &'a [(i64, &'a str, f64)];
type Clock = (u32, u32);
type Spans<'a> = &'a [(f64, f64)];
type SleepCase<'a> = (Levels<'a>, Clock, Option<Clock>, Spans<'a>, &'a str);

fn last_drawn(slots: &[Option<f64>]) -> Option<usize> {
    slots.iter().rposition(Option::is_some)
}

#[test]
fn nap_change_uses_measured_levels_either_side() {
    let mut slots = vec![None; 100];
    slots[40] = Some(30.);
    slots[45] = Some(31.);
    slots[60] = Some(42.);
    // (start, end) in minutes → change.
    for (start, end, want, why) in [
        (138., 171., Some(11.), "last before the start, first after the end"),
        (135., 180., Some(11.), "readings exactly at the start and end count"),
        (300., 400., None, "nothing measured after the nap"),
        (90., 171., None, "nothing measured before the nap"),
        (100., 90., None, "an end before the start"),
        (-3., 90., None, "a start before midnight"),
    ] {
        assert_eq!(day::nap_change(&slots, start, end), want, "{why}");
    }
    assert_eq!(day::nap_change(&[None, None], 0., 3.), None);
}

#[test]
fn the_day_places_sleep_activities_and_naps_on_the_calendar_axis() {
    let (dir, store, core) = fx("axis");
    night(&store, "2025-03-12T23:00:00.0", "2025-03-13T07:00:00.0",
        &[("2025-03-12T23:00:00.0", "2025-03-13T03:00:00.0", 1.0), ("2025-03-13T03:00:00.0", "2025-03-13T07:00:00.0", 0.0)]);
    store.put("activity", "5", "now", &json!({"activityId": 5, "activityName": "Morning Run", "activityTypeDTO": {"typeKey": "running"},
        "summaryDTO": {"startTimeLocal": "2025-03-13 08:30:00", "duration": 1800}})).unwrap();
    nap(&store, "2025-03-13T13:00:00", "2025-03-13T13:30:00");
    battery(&store, &[(255, "MEASURED", 40.), (260, "MEASURED", 41.), (275, "MEASURED", 52.)]);
    let view = core.today_live_at(DAY, at(13, 15, 0));
    let d = view.day.unwrap();
    assert_eq!((d.date.as_str(), d.now_min), (DAY, 900.));
    let run = d.events.iter().find(|e| e.kind == DayEventKind::Activity).unwrap();
    assert_eq!((run.start_min, run.end_min, run.id.as_deref()), (Some(510.), Some(540.), Some("5")));
    let n = d.events.iter().find(|e| e.kind == DayEventKind::Nap).unwrap();
    assert_eq!((n.start_min, n.end_min, n.body_battery_change), (Some(780.), Some(810.), Some(11.)));
    assert!(core.today_at(DAY, at(13, 15, 0)).day.is_none(), "historical reads have no live timeline");
    drop(core);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn sleep_is_clipped_at_midnight_and_at_the_drawing_limit() {
    let (dir, store, core) = fx("sleep");
    // (levels, now, upload, expected spans in minutes from midnight)
    let cases: [SleepCase; 3] = [
        (&[("2025-03-12T23:00:00.0", "2025-03-13T03:00:00.0", 1.0), ("2025-03-13T03:00:00.0", "2025-03-13T07:00:00.0", 0.0)],
            (15, 0), Some((14, 0)), &[(0., 180.), (180., 420.)], "a stage across midnight starts at 00:00"),
        (&[("2025-03-12T22:00:00.0", "2025-03-12T23:30:00.0", 1.0), ("2025-03-12T23:30:00.0", "2025-03-13T07:00:00.0", 0.0)],
            (15, 0), Some((14, 0)), &[(0., 420.)], "a stage before midnight isn't drawn"),
        (&[("2025-03-12T23:00:00.0", "2025-03-13T03:00:00.0", 1.0), ("2025-03-13T03:00:00.0", "2025-03-13T07:00:00.0", 0.0)],
            (5, 0), Some((4, 30)), &[(0., 180.), (180., 270.)], "nothing after the upload"),
    ];
    for (levels, (h, m), upload, want, why) in cases {
        let start = levels[0].0;
        night(&store, start, "2025-03-13T07:00:00.0", levels);
        let d = timeline(&store, &core, at(13, h, m), upload.map(|(h, m)| at(13, h, m)), 1440.);
        let spans: Vec<(f64, f64)> = d.sleep.iter().map(|s| (s.start_min, s.end_min)).collect();
        assert_eq!(spans, want, "{why}");
    }
    drop(core);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn nothing_is_drawn_after_the_upload_or_now() {
    let (dir, store, core) = fx("limit");
    // Readings every three minutes to 20:00 and heart rate to 20:00.
    let samples: Vec<(i64, &str, f64)> = (0..400).map(|s| (s, "MEASURED", 50.)).collect();
    battery(&store, &samples);
    heart(&store, 0..1200);
    // (now, upload, last Body Battery slot, last heart-rate slot)
    for (now, upload, bb, hr, why) in [
        ((15, 0), Some((13, 30)), Some(270), Some(405), "cut at the upload"),
        ((15, 0), None, Some(300), Some(450), "without an upload time, cut at now"),
        ((12, 0), Some((13, 30)), Some(240), Some(360), "an upload later than now still stops at now"),
    ] {
        let d = timeline(&store, &core, at(13, now.0, now.1), upload.map(|(h, m)| at(13, h, m)), 1440.);
        assert_eq!((last_drawn(&d.body_battery), last_drawn(&d.heart_rate)), (bb, hr), "{why}");
    }
    let yesterday = timeline(&store, &core, at(13, 15, 0), Some(at(12, 23, 0)), 1440.);
    assert_eq!((yesterday.upload_min, last_drawn(&yesterday.body_battery)), (None, Some(0)), "an upload before midnight draws only 00:00");
    drop(core);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn a_nap_change_needs_measured_levels_before_the_upload() {
    let (dir, store, core) = fx("nap");
    nap(&store, "2025-03-13T13:00:00", "2025-03-13T13:30:00");
    // The nap runs 780–810 (slots 260–270).
    let cases: [(Samples, Clock, Option<f64>, &str); 5] = [
        (&[(255, "MEASURED", 40.), (275, "MEASURED", 52.)], (14, 30), Some(12.), "measured either side"),
        (&[(255, "MEASURED", 40.), (270, "MODELED", 45.), (275, "MEASURED", 52.)], (14, 30), Some(12.), "a modelled level is skipped"),
        (&[(255, "MODELED", 40.), (275, "MEASURED", 52.)], (14, 30), None, "only a modelled level before"),
        (&[(255, "MEASURED", 40.)], (14, 30), None, "nothing after the nap"),
        (&[(255, "MEASURED", 40.), (275, "MEASURED", 52.)], (13, 20), None, "the nap hadn't ended by the upload"),
    ];
    for (samples, upload, want, why) in cases {
        battery(&store, samples);
        let d = timeline(&store, &core, at(13, 15, 0), Some(at(13, upload.0, upload.1)), 1440.);
        let n = d.events.iter().find(|e| e.kind == DayEventKind::Nap).unwrap();
        assert_eq!(n.body_battery_change, want, "{why}");
    }
    drop(core);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn a_clock_change_day_leaves_intraday_series_to_their_details() {
    let (dir, store, core) = fx("dst");
    battery(&store, &[(10, "MEASURED", 50.), (20, "MEASURED", 55.)]);
    heart(&store, 0..120);
    for (minutes, shown) in [(1380., false), (1440., true), (1500., false)] {
        let d = timeline(&store, &core, at(13, 15, 0), Some(at(13, 14, 0)), minutes);
        assert_eq!(d.clock_change, !shown, "{minutes}-minute day");
        assert_eq!((!d.heart_rate.is_empty(), !d.body_battery.is_empty()), (shown, shown), "{minutes}-minute day");
    }
    drop(core);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn keep_lasts_the_local_day_and_respects_limits() {
    let (dir, store, core) = fx("keep");
    night(&store, "2025-03-12T23:00:00.0", "2025-03-13T07:00:00.0", &[("2025-03-12T23:00:00.0", "2025-03-13T07:00:00.0", 1.0)]);
    let add = |date: &str, sport| {
        core.local_action(&LocalAction::AddIntention { date: date.into(), time: Some("18:00".into()), sport, type_key: None, name: None,
            duration_min: None, allocation: Allocation::default(), kind: IntentionKind::Workout, garmin_id: None }, at(13, 8, 0)).unwrap();
        core.local_records().intentions.items.last().unwrap().id.clone()
    };
    let run = add(DAY, Sport::Running);
    let lift = add(DAY, Sport::Strength);
    let tomorrow = add("2025-03-14", Sport::Strength);
    core.local_action(&LocalAction::SetLimits { limits: Limits { excluded: vec![Sport::Running], note: String::new() } }, at(13, 8, 1)).unwrap();
    let keep = |id: &str, when| core.local_action(&LocalAction::KeepToday { plan_id: id.into() }, when);
    // (session, when, accepted, why)
    for (id, when, ok, why) in [
        (run.as_str(), at(13, 9, 0), false, "an excluded sport can't be kept"),
        (tomorrow.as_str(), at(13, 9, 0), false, "only today's sessions"),
        ("garmin:2025-03-14:Tempo", at(13, 9, 0), false, "only today's Garmin schedule"),
        (lift.as_str(), at(13, 9, 0), true, "today's open session"),
        (lift.as_str(), at(14, 9, 0), false, "yesterday's session can't be kept today"),
    ] {
        assert_eq!(keep(id, when).is_ok(), ok, "{why}");
    }
    // Which session leads, and that it stops leading the next day, is
    // lead.rs's a_kept_session_leads_and_only_todays_sessions_can_be_kept.
    let choice = core.local_records().circumstances.today_choice.unwrap();
    assert_eq!((choice.date.as_str(), choice.plan_id.as_str()), (DAY, lift.as_str()));
    drop(core);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn the_planned_demo_day_lets_a_session_lead() {
    let (dir, store, core) = fx("demo");
    let day = at(13, 10, 0);
    raring_core::demo::seed_variant(&store, day.date_naive(), false).unwrap();
    core.local_action(&LocalAction::AddIntention { date: DAY.into(), time: Some("18:00".into()), sport: Sport::Running, type_key: None, name: None,
        duration_min: None, allocation: Allocation::default(), kind: IntentionKind::Workout, garmin_id: None }, day).unwrap();
    let lead = core.today_live_at(DAY, day).lead.unwrap();
    assert_eq!(lead.kind, LeadKind::Session, "{lead:?}");
    raring_core::demo::seed(&store, day.date_naive()).unwrap();
    assert_eq!(core.today_live_at(DAY, day).lead.unwrap().kind, LeadKind::Done, "the standard demo records a run today");
    let run = raring_core::screens::workout(&store, raring_core::demo::WORKOUT_ID).unwrap();
    let kinds: Vec<_> = run.along.unwrap().series.iter().map(|s| s.kind).collect();
    assert_eq!(kinds, vec![StreamKind::Speed, StreamKind::Elevation], "the demo run has streams with Garmin's unit keys");
    assert!(run.more.iter().any(|m| m.key == "vigorousIntensityMinutes"));
    drop(core);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn a_routine_is_self_reported_per_day_and_stays_off_status_surfaces() {
    let (dir, _store, core) = fx("routine");
    let at13 = at(13, 9, 0);
    core.local_action(&LocalAction::AddRoutine { name: "  Wall sit 3 × 30 s ".into(), how: Some("From my physio".into()) }, at13).unwrap();
    assert!(core.local_action(&LocalAction::AddRoutine { name: "   ".into(), how: None }, at13).is_err(), "a name is required");
    assert!(core.local_action(&LocalAction::ReportRoutine { id: "r-none".into(), done: true }, at13).is_err());
    let routines = || core.local_records().routines;
    let id = routines().items[0].id.clone();
    assert_eq!(routines().items[0].name, "Wall sit 3 × 30 s");
    let done = |when| core.today_live_at(&when_date(when), when).adapt.unwrap().routines[0].done;
    assert!(!done(at13));
    core.local_action(&LocalAction::ReportRoutine { id: id.clone(), done: true }, at13).unwrap();
    core.local_action(&LocalAction::ReportRoutine { id: id.clone(), done: true }, at(13, 10, 0)).unwrap();
    assert_eq!(routines().reports.len(), 1, "one report per day");
    assert!(done(at13));
    assert!(!done(at(14, 9, 0)), "each day starts unreported");
    let snapshot = serde_json::to_string(&raring_core::status::snapshot(&Store::open(dir.join("raring.db")).unwrap(), at13)).unwrap();
    assert!(!snapshot.contains("Wall sit") && !snapshot.contains("physio"), "never on status surfaces");
    // Self-reports never become Garmin evidence.
    assert!(core.plan_board(DAY, at13).unwrap().recordings.is_empty());
    core.local_action(&LocalAction::ReportRoutine { id: id.clone(), done: false }, at(13, 11, 0)).unwrap();
    assert!(!done(at13));
    core.local_action(&LocalAction::RemoveRoutine { id }, at13).unwrap();
    assert_eq!(routines(), Default::default());
    drop(core);
    let _ = std::fs::remove_dir_all(dir);
}
fn when_date(t: chrono::DateTime<Local>) -> String {
    t.date_naive().to_string()
}
