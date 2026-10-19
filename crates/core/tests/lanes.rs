//! Training's week lanes: cumulative Monday → today per goal kind, and the
//! usual-week band from counted earlier weeks.
use chrono::{Local, TimeZone};
use raring_core::{lanes, store::Store, view::*, Raring};
use serde_json::json;

fn run(store: &Store, id: u64, start: &str, minutes: u64, zones: Option<[u64; 5]>) {
    store.put("activity", &id.to_string(), "now", &json!({"activityId": id, "activityName": "Run", "activityTypeDTO": {"typeKey": "running"},
        "summaryDTO": {"startTimeLocal": start, "duration": minutes * 60}})).unwrap();
    if let Some(z) = zones {
        let zones: Vec<_> = z.iter().enumerate().map(|(i, s)| json!({"zoneNumber": i + 1, "secsInZone": s})).collect();
        store.put("zones", &id.to_string(), "now", &json!(zones)).unwrap();
    }
}

#[test]
fn percentiles_interpolate() {
    for (xs, p, want) in [(vec![10., 20., 30., 40.], 0.5, 25.), (vec![10., 20., 30., 40.], 0.25, 17.5), (vec![5.], 0.75, 5.), (vec![30., 10., 20.], 0.5, 20.)] {
        assert_eq!(lanes::percentile(&xs, p), want, "{xs:?} at {p}");
    }
}

#[test]
fn lanes_follow_goals_and_need_three_counted_weeks_for_a_band() {
    let dir = std::env::temp_dir().join(format!("raring-lanes-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let store = Store::open(dir.join("raring.db")).unwrap();
    store.set_cursor("listed:lo", "2025-03-10").unwrap();
    store.set_cursor("listed:hi", "2025-03-13").unwrap();
    // Thursday 13 March; the plan week starts Monday 10 March.
    let now = Local.with_ymd_and_hms(2025, 3, 13, 18, 0, 0).unwrap();
    run(&store, 1, "2025-03-10 07:00:00", 30, Some([0, 1200, 0, 0, 0]));
    run(&store, 2, "2025-03-12 07:00:00", 40, None);
    let core = Raring::open(dir.join("raring.db")).unwrap();
    let week = || core.training_live_at("2025-03-13", now).lanes.unwrap();
    let w = week();
    assert_eq!(w.start, "2025-03-10");
    let cardio = &w.lanes[0];
    assert_eq!(cardio.kind, LaneKind::Cardio, "no cardio goal: recorded cardio minutes");
    assert_eq!(cardio.cumulative, vec![Some(30.), Some(30.), Some(70.), Some(70.), None, None, None], "nothing after today");
    assert!(cardio.goal.is_none() && cardio.usual.is_none() && cardio.weeks == 0, "no goal, no covered history");
    assert!(!w.lanes.iter().any(|l| l.kind == LaneKind::Strength), "no strength goal or sessions");

    let mut targets = GoalTargets { strength_sessions: 0, zone2_min: 90, high_intensity_min: 0, intensity_min: 0, intensity_garmin_goal: false,
        sleep_nights: 0, sleep_time: None, sleep_kind: SleepTimingKind::AsleepBy, sleep_score: 0, rest_days: vec![] };
    core.local_action(&LocalAction::SetGoals { targets: targets.clone() }, now).unwrap();
    let z = &week().lanes[0];
    assert_eq!((z.kind, z.goal), (LaneKind::Zone2, Some(90.)));
    assert_eq!(z.cumulative[..3], [Some(20.), Some(20.), None], "a day with a zoneless activity is unknown from then on");

    // Earlier weeks: one run each Tuesday with 30, 40, 50 Zone 2 minutes.
    for (i, (day, m)) in [("2025-02-18", 30), ("2025-02-25", 40), ("2025-03-04", 50)].iter().enumerate() {
        run(&store, 10 + i as u64, &format!("{day} 07:00:00"), *m, Some([0, m * 60, 0, 0, 0]));
    }
    assert_eq!(week().lanes[0].weeks, 0, "weeks before the activity list's coverage aren't counted");
    store.set_cursor("listed:lo", "2025-02-17").unwrap();
    let z = &week().lanes[0];
    assert_eq!(z.weeks, 3, "three counted weeks");
    let band = z.usual.as_ref().expect("three weeks make a band");
    assert_eq!((band.low[0], band.median[0], band.high[0]), (0., 0., 0.), "Monday before any run");
    assert_eq!((band.low[1], band.median[1], band.high[1]), (35., 40., 45.), "Tuesday's cumulative percentiles");
    assert_eq!(band.median[6], 40.);

    // An ill week isn't counted.
    core.local_action(&LocalAction::Pause { reason: PauseReason::Ill, review: None }, Local.with_ymd_and_hms(2025, 3, 3, 9, 0, 0).unwrap()).unwrap();
    let z = &week().lanes[0];
    assert_eq!((z.weeks, z.usual.is_none()), (2, true), "the ill week drops out, leaving too few for a band");
    targets.strength_sessions = 2;
    core.local_action(&LocalAction::SetGoals { targets }, now).unwrap();
    let w = week();
    assert!(w.lanes.iter().any(|l| l.kind == LaneKind::Strength && l.goal == Some(2.) && l.cumulative[3] == Some(0.)), "a strength goal adds its lane");
    drop(core);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn a_day_outside_the_activity_list_makes_recorded_lanes_unknown() {
    let dir = std::env::temp_dir().join(format!("raring-lanes-cover-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let store = Store::open(dir.join("raring.db")).unwrap();
    // The list covers Monday and Tuesday only.
    store.set_cursor("listed:lo", "2025-03-10").unwrap();
    store.set_cursor("listed:hi", "2025-03-11").unwrap();
    run(&store, 1, "2025-03-10 07:00:00", 30, None);
    let core = Raring::open(dir.join("raring.db")).unwrap();
    let now = Local.with_ymd_and_hms(2025, 3, 13, 18, 0, 0).unwrap();
    let cardio = &core.training_live_at("2025-03-13", now).lanes.unwrap().lanes[0];
    assert_eq!(cardio.cumulative[..4], [Some(30.), Some(30.), None, None], "unknown from the first uncovered day, never zero");
    drop(core);
    let _ = std::fs::remove_dir_all(dir);
}
