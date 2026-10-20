//! Garmin Coach sessions as read-only planned work: rest days, counting,
//! matching and Garmin's own status (docs/spec.md, Rules: Garmin Coach).
use chrono::{Local, TimeZone};
use raring_core::{store::Store, view::*, Raring};
use serde_json::{json, Value};

fn at(d: u32, h: u32) -> chrono::DateTime<Local> {
    Local.with_ymd_and_hms(2025, 3, d, h, 0, 0).unwrap()
}
fn fixture() -> Value {
    serde_json::from_str(include_str!("fixtures/coach_adaptive.json")).unwrap()
}
struct Fx {
    dir: std::path::PathBuf,
    store: Store,
    core: Raring,
}
impl Drop for Fx {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}
fn fx(name: &str, plan: Value) -> Fx {
    let dir = std::env::temp_dir().join(format!("raring-coach-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let store = Store::open(dir.join("raring.db")).unwrap();
    let fetched = at(13, 8).to_rfc3339();
    store.put("coach_plans", "current", &fetched, &json!({"trainingPlanList": [plan]})).unwrap();
    store.put("coach_adaptive", "current", &fetched, &fixture()).unwrap();
    store.set_cursor("listed:lo", "2025-03-01").unwrap();
    store.set_cursor("listed:hi", "2025-03-13").unwrap();
    let core = Raring::open(dir.join("raring.db")).unwrap();
    Fx { dir, store, core }
}
fn adaptive() -> Value {
    json!({"trainingPlanId": 42, "trainingPlanCategory": "FBT_ADAPTIVE", "trainingStatus": {"statusKey": "Scheduled"}})
}
fn act(store: &Store, id: u64, date: &str, kind: &str, minutes: f64) {
    store.put("activity", &id.to_string(), "now", &json!({"activityId": id, "activityName": kind, "activityTypeDTO": {"typeKey": kind},
        "summaryDTO": {"startTimeLocal": format!("{date} 07:00:00"), "duration": minutes * 60.}})).unwrap();
}
fn board(f: &Fx) -> PlanBoard {
    f.core.plan_board("2025-03-13", at(13, 18)).unwrap()
}
fn item<'a>(b: &'a PlanBoard, date: &str) -> &'a PlanItem {
    b.items.iter().find(|p| p.intention.date == date && p.coach.is_some()).unwrap()
}

#[test]
fn coach_sessions_are_read_only_planned_work_and_rest_days_are_not_items() {
    let f = fx("items", adaptive());
    let b = board(&f);
    let coach = b.coach.as_ref().unwrap();
    assert_eq!((coach.name.as_str(), coach.sessions.len()), ("Synthetic Coach", 5));
    let rest = coach.sessions.iter().find(|s| s.date == "2025-03-12").unwrap();
    assert!(rest.rest_day && rest.kind.is_none());
    assert_eq!(b.items.iter().filter(|p| p.coach.is_some()).count(), 4, "the rest day is not a session");
    let strength = item(&b, "2025-03-11");
    assert_eq!((strength.intention.sport, strength.intention.allocation.strength_sessions), (Sport::Strength, 1));
    let cardio = item(&b, "2025-03-13");
    let c = cardio.coach.as_ref().unwrap();
    assert_eq!((c.kind, c.duration_min, c.effect.as_deref(), c.garmin_status.as_deref()), (Some(CoachKind::Cardio), Some(45), Some("Lactate threshold"), Some("Not complete")));
    let a = &cardio.intention.allocation;
    assert_eq!((a.zone2_min, a.high_intensity_min, a.intensity_min), (0, 0, 0), "no zone or intensity minutes inferred");
    assert_eq!(item(&b, "2025-03-11").coach.as_ref().unwrap().effect, None, "an unknown label isn't shown");
}

#[test]
fn garmins_status_is_not_completion_and_recordings_match_by_kind() {
    let f = fx("match", adaptive());
    // Monday: Garmin says not complete and nothing was recorded.
    assert_eq!(item(&board(&f), "2025-03-10").state, ItemState::NotRecorded);
    act(&f.store, 1, "2025-03-10", "cycling", 32.);
    act(&f.store, 2, "2025-03-11", "strength_training", 40.);
    let b = board(&f);
    for (date, rec) in [("2025-03-10", "1"), ("2025-03-11", "2")] {
        let p = item(&b, date);
        assert_eq!((p.state, p.recording.as_deref(), p.matched_by), (ItemState::Done, Some(rec), Some(MatchKind::Automatic)), "{date}");
    }
    let strength = b.ledger.iter().find(|l| l.kind == GoalKind::Strength);
    assert!(strength.is_none(), "no goals set, no ledger");
    // Two cardio recordings on Thursday: Raring asks.
    act(&f.store, 3, "2025-03-13", "running", 30.);
    act(&f.store, 4, "2025-03-13", "walking", 20.);
    let p = item(&board(&f), "2025-03-13").clone();
    assert_eq!((p.state, p.candidates.len()), (ItemState::Review, 2));
    f.core.local_action(&LocalAction::Link { plan_id: p.intention.id.clone(), activity_id: "3".into() }, at(13, 19)).unwrap();
    assert_eq!(item(&board(&f), "2025-03-13").recording.as_deref(), Some("3"), "a link settles it");
    f.core.local_action(&LocalAction::Unlink { plan_id: "coach:2025-03-10:1".into(), activity_id: "1".into() }, at(13, 19)).unwrap();
    assert_eq!(item(&board(&f), "2025-03-10").state, ItemState::Review, "an unlinked pair stays apart until linked, as for your sessions");
}

#[test]
fn a_coach_strength_session_counts_as_planned_and_can_be_kept_today() {
    let f = fx("lead", adaptive());
    // Read on Monday morning: fetched just before.
    f.store.put("coach_plans", "current", &at(10, 7).to_rfc3339(), &json!({"trainingPlanList": [adaptive()]})).unwrap();
    f.store.put("coach_adaptive", "current", &at(10, 7).to_rfc3339(), &fixture()).unwrap();
    let t = GoalTargets { strength_sessions: 2, zone2_min: 60, high_intensity_min: 0, intensity_min: 0, intensity_garmin_goal: false,
        sleep_nights: 0, sleep_time: None, sleep_kind: SleepTimingKind::AsleepBy, sleep_score: 0, rest_days: vec![] };
    f.core.local_action(&LocalAction::SetGoals { targets: t }, at(10, 8)).unwrap();
    let b = f.core.plan_board("2025-03-10", at(10, 8)).unwrap();
    let strength = b.ledger.iter().find(|l| l.kind == GoalKind::Strength).unwrap();
    let zone2 = b.ledger.iter().find(|l| l.kind == GoalKind::Zone2).unwrap();
    assert_eq!((strength.planned, zone2.planned), (1., 0.), "Coach strength plans a session; cardio plans no Zone 2");
    assert!(f.core.local_action(&LocalAction::KeepToday { plan_id: "coach:2025-03-10:1".into() }, at(10, 9)).is_ok());
    assert!(f.core.local_action(&LocalAction::KeepToday { plan_id: "coach:2025-03-11:1".into() }, at(10, 9)).is_err(), "only today's");
}

#[test]
fn only_an_active_adaptive_plan_in_the_plan_list_shows_sessions() {
    for (plan, shown) in [
        (adaptive(), true),
        (json!({"trainingPlanId": 42, "trainingPlanCategory": "FBT_ADAPTIVE", "trainingStatus": {"statusKey": "Completed"}}), false),
        (json!({"trainingPlanId": 42, "trainingPlanCategory": "RUNNING", "trainingStatus": {"statusKey": "Scheduled"}}), false),
        (json!({"trainingPlanId": 7, "trainingPlanCategory": "FBT_ADAPTIVE", "trainingStatus": {"statusKey": "Scheduled"}}), false),
    ] {
        let f = fx("list", plan.clone());
        assert_eq!(board(&f).coach.is_some(), shown, "{plan}");
    }
}

#[test]
fn a_stale_plan_is_named_but_plans_nothing() {
    let f = fx("stale", adaptive());
    let old = at(12, 18).to_rfc3339();
    f.store.put("coach_adaptive", "current", &old, &fixture()).unwrap();
    let b = board(&f);
    let coach = b.coach.unwrap();
    assert_eq!((coach.state, coach.sessions.len()), (PlannedState::Stale, 0));
    assert!(b.items.iter().all(|p| p.coach.is_none()));
}

#[test]
fn a_workout_says_how_it_counted_toward_the_week() {
    let f = fx("counted", json!({"trainingPlanCategory": "NONE"}));
    let t = GoalTargets { strength_sessions: 2, zone2_min: 60, high_intensity_min: 0, intensity_min: 0, intensity_garmin_goal: false,
        sleep_nights: 0, sleep_time: None, sleep_kind: SleepTimingKind::AsleepBy, sleep_score: 0, rest_days: vec![] };
    f.core.local_action(&LocalAction::SetGoals { targets: t }, at(10, 8)).unwrap();
    act(&f.store, 5, "2025-03-11", "strength_training", 40.);
    act(&f.store, 6, "2025-03-12", "strength_training", 30.);
    let c = f.core.workout_live("6", at(13, 18)).unwrap().counted.unwrap();
    let s = c.lines.iter().find(|l| l.kind == GoalKind::Strength).unwrap();
    assert_eq!((c.week.as_str(), s.amount, s.position, s.recorded, s.target), ("2025-03-10", 1., Some(2), Some(2.), 2.), "the second of two strength sessions");
    assert!(c.lines.iter().all(|l| l.kind != GoalKind::Zone2), "no zones, no Zone 2 share");
    assert!(c.plan_item.is_none() && c.bests.is_empty());
    assert!(c.goals);
    assert!(f.core.workout("6").unwrap().counted.is_none(), "only live reads say how it counted");
}

#[test]
fn progress_marks_coach_days_exactly_when_plan_shows_them() {
    let f = fx("progress-gate", adaptive());
    let garmin = |days: &[DayCell]| days.iter().filter(|d| d.date.as_str() >= "2025-03-10" && d.date.as_str() <= "2025-03-16" && d.garmin > 0).count();
    for (when, fresh) in [(at(13, 18), true), (at(14, 9), false)] {
        let plan = f.core.plan_board("2025-03-13", when).unwrap();
        let progress = f.core.progress(Period::Month, "2025-03-13", when).unwrap();
        assert_eq!(garmin(&progress.ring_days), garmin(&plan.days), "fresh {fresh}");
        assert_eq!(garmin(&plan.days) > 0, fresh, "a snapshot older than 12 hours plans nothing");
    }
}

#[test]
fn a_workout_without_weekly_goals_says_so() {
    let f = fx("no-goals", json!({"trainingPlanCategory": "NONE"}));
    act(&f.store, 6, "2025-03-12", "running", 30.);
    let c = f.core.workout_live("6", at(13, 18)).unwrap().counted.unwrap();
    assert!(!c.goals && c.lines.is_empty(), "no goal on, so no goal rejected it");
}
