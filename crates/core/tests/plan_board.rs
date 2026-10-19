//! The plan board: matching, the goal ledger, rings and backlog
//! (docs/spec.md, Rules). Synthetic week Mon 10 – Sun 16 March 2025.
use chrono::{DateTime, Local, TimeZone};
use raring_core::{store::Store, view::*, Raring};
use serde_json::{json, Value};

fn at(day: u32, hour: u32) -> DateTime<Local> {
    Local.with_ymd_and_hms(2025, 3, day, hour, 0, 0).unwrap()
}
const THU: u32 = 13;

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
fn fx(name: &str) -> Fx {
    let dir = std::env::temp_dir().join(format!("raring-board-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let store = Store::open(dir.join("raring.db")).unwrap();
    let core = Raring::open(dir.join("raring.db")).unwrap();
    // The activity list is complete through Thursday.
    listed(&store, "2025-03-01", "2025-03-13");
    Fx { dir, store, core }
}
fn listed(store: &Store, lo: &str, hi: &str) {
    store.set_cursor("listed:lo", lo).unwrap();
    store.set_cursor("listed:hi", hi).unwrap();
}

/// A recording; zones are seconds in zones 1–5, None for no zone data.
fn rec(store: &Store, id: u64, start: &str, kind: &str, minutes: f64, zones: Option<[u32; 5]>) {
    store
        .put("activity", &id.to_string(), "now", &json!({"activityId": id, "activityName": kind,
            "activityTypeDTO": {"typeKey": kind},
            "summaryDTO": {"startTimeLocal": start, "duration": minutes * 60.,
                "moderateIntensityMinutes": 10, "vigorousIntensityMinutes": 5}}))
        .unwrap();
    if let Some(z) = zones {
        let list: Vec<Value> = z.iter().enumerate().map(|(i, s)| json!({"zoneNumber": i + 1, "secsInZone": s})).collect();
        store.put("zones", &id.to_string(), "now", &json!(list)).unwrap();
    }
}
fn summary(store: &Store, date: &str, moderate: u32, vigorous: u32) {
    store
        .put("summary", date, "now", &json!({"moderateIntensityMinutes": moderate, "vigorousIntensityMinutes": vigorous, "intensityMinutesGoal": 150}))
        .unwrap();
}

fn goals(f: &Fx, t: GoalTargets) {
    f.core.local_action(&LocalAction::SetGoals { targets: t }, at(10, 8)).unwrap();
}
fn strength(n: u32) -> GoalTargets {
    GoalTargets { strength_sessions: n, ..Default::default() }
}
fn plan(f: &Fx, date: &str, sport: Sport, alloc: Allocation, minutes: Option<u32>, type_key: Option<&str>) -> String {
    f.core
        .local_action(&LocalAction::AddIntention {
            date: date.into(), time: None, sport, type_key: type_key.map(str::to_owned), name: None,
            duration_min: minutes, allocation: alloc, kind: IntentionKind::Workout, garmin_id: None,
        }, at(10, 7))
        .unwrap();
    f.core.local_records().intentions.items.last().unwrap().id.clone()
}
fn lift(f: &Fx, date: &str) -> String {
    plan(f, date, Sport::Strength, Allocation { strength_sessions: 1, ..Default::default() }, None, None)
}
fn board(f: &Fx) -> PlanBoard {
    f.core.plan_board("2025-03-13", at(THU, 12)).unwrap()
}
fn item<'a>(b: &'a PlanBoard, id: &str) -> &'a PlanItem {
    b.items.iter().find(|p| p.intention.id == id).unwrap()
}
fn row(b: &PlanBoard, kind: GoalKind) -> &GoalLedger {
    b.ledger.iter().find(|l| l.kind == kind).unwrap()
}
fn day<'a>(b: &'a PlanBoard, date: &str) -> &'a DayCell {
    b.days.iter().find(|d| d.date == date).unwrap()
}

#[test]
fn without_goals_the_week_still_shows_its_recordings() {
    let f = fx("nogoals");
    rec(&f.store, 1, "2025-03-11 07:00:00", "walking", 30., Some([600, 600, 0, 0, 0]));
    let b = board(&f);
    assert!(b.goals.is_none() && b.ledger.is_empty() && b.week_rings.is_empty() && b.backlog.is_empty());
    assert_eq!(b.recordings.len(), 1);
    assert!(b.days.iter().all(|d| d.rings.is_empty()));
    assert_eq!((b.start.as_str(), b.end.as_str()), ("2025-03-10", "2025-03-16"));
}

#[test]
fn an_unambiguous_same_day_same_sport_pair_matches_and_counts_once() {
    let f = fx("auto");
    goals(&f, strength(2));
    let p = lift(&f, "2025-03-11");
    rec(&f.store, 1, "2025-03-11 18:00:00", "strength_training", 45., Some([0; 5]));
    let b = board(&f);
    let i = item(&b, &p);
    assert_eq!((i.state, i.recording.as_deref(), i.matched_by), (ItemState::Done, Some("1"), Some(MatchKind::Automatic)));
    let l = row(&b, GoalKind::Strength);
    assert_eq!((l.recorded, l.planned, l.remaining), (Some(1.), 0., Some(1.)));
    assert!(day(&b, "2025-03-11").done);
}

#[test]
fn a_link_holds_and_a_recording_counts_once_however_it_is_matched() {
    let f = fx("linked");
    goals(&f, strength(2));
    let p = lift(&f, "2025-03-11");
    rec(&f.store, 1, "2025-03-11 07:00:00", "strength_training", 45., Some([0; 5]));
    rec(&f.store, 2, "2025-03-11 18:00:00", "strength_training", 45., Some([0; 5]));
    f.core.local_action(&LocalAction::Link { plan_id: p.clone(), activity_id: "2".into() }, at(THU, 9)).unwrap();
    let b = board(&f);
    let i = item(&b, &p);
    assert_eq!((i.state, i.recording.as_deref(), i.matched_by), (ItemState::Done, Some("2"), Some(MatchKind::Linked)));
    assert_eq!(row(&b, GoalKind::Strength).recorded, Some(2.), "goals count recordings, never plans");
}

#[test]
fn raring_asks_only_when_it_cannot_tell() {
    let f = fx("review");
    goals(&f, strength(3));
    let a = lift(&f, "2025-03-11");
    let b2 = lift(&f, "2025-03-11");
    rec(&f.store, 1, "2025-03-11 18:00:00", "strength_training", 45., Some([0; 5]));
    let b = board(&f);
    assert_eq!(item(&b, &a).state, ItemState::Review);
    assert_eq!(item(&b, &b2).state, ItemState::Review);
    assert_eq!(item(&b, &a).candidates, ["1"]);
    let l = row(&b, GoalKind::Strength);
    assert_eq!((l.recorded, l.planned, l.review, l.remaining), (Some(1.), 0., 2, Some(2.)), "review is neither planned nor a second count");

    f.core.local_action(&LocalAction::RemoveIntention { id: b2 }, at(10, 9)).unwrap();
    assert_eq!(item(&board(&f), &a).state, ItemState::Done);
    f.core.local_action(&LocalAction::Unlink { plan_id: a.clone(), activity_id: "1".into() }, at(THU, 9)).unwrap();
    assert_eq!(item(&board(&f), &a).state, ItemState::Review, "an unlinked pair stays apart until linked");
}

#[test]
fn mixed_types_match_only_their_own_garmin_type() {
    let f = fx("other");
    let loose = plan(&f, "2025-03-11", Sport::Other, Allocation::default(), None, None);
    let typed = plan(&f, "2025-03-12", Sport::Other, Allocation::default(), None, Some("hiit"));
    rec(&f.store, 1, "2025-03-11 18:00:00", "hiit", 30., Some([0; 5]));
    rec(&f.store, 2, "2025-03-12 18:00:00", "hiit", 30., Some([0; 5]));
    let b = board(&f);
    assert_eq!(item(&b, &loose).state, ItemState::NotRecorded, "an untyped mixed session never guesses");
    assert_eq!(item(&b, &typed).state, ItemState::Done);
}

#[test]
fn a_different_sport_is_asked_about_only_for_one_similar_endurance_recording_on_a_passed_day() {
    let f = fx("swap");
    let ride = plan(&f, "2025-03-11", Sport::Cycling, Allocation::default(), Some(60), None);
    rec(&f.store, 1, "2025-03-11 18:00:00", "running", 50., Some([0; 5]));
    let lifted = lift(&f, "2025-03-12");
    rec(&f.store, 2, "2025-03-12 18:00:00", "running", 40., Some([0; 5]));
    let today_ride = plan(&f, "2025-03-13", Sport::Cycling, Allocation::default(), Some(60), None);
    rec(&f.store, 3, "2025-03-13 07:00:00", "running", 50., Some([0; 5]));
    let b = board(&f);
    assert_eq!((item(&b, &ride).state, item(&b, &ride).candidates.clone()), (ItemState::Review, vec!["1".to_owned()]));
    assert_eq!(item(&b, &lifted).state, ItemState::NotRecorded);
    assert_eq!(item(&b, &today_ride).state, ItemState::Planned, "today isn't over");
}

#[test]
fn zone_goals_count_recorded_zone_time_never_planned_duration() {
    let f = fx("zones");
    goals(&f, GoalTargets { zone2_min: 90, high_intensity_min: 30, ..Default::default() });
    plan(&f, "2025-03-15", Sport::Cycling, Allocation::default(), Some(90), None);
    plan(&f, "2025-03-16", Sport::Cycling, Allocation { zone2_min: 40, ..Default::default() }, Some(60), None);
    rec(&f.store, 1, "2025-03-11 18:00:00", "cycling", 60., Some([300, 1250, 900, 400, 200]));
    rec(&f.store, 2, "2025-03-12 18:00:00", "running", 30., Some([0, 1255, 0, 30, 0]));
    let b = board(&f);
    let z2 = row(&b, GoalKind::Zone2);
    assert_eq!((z2.recorded, z2.planned, z2.remaining), (Some(41.), 40., Some(9.)), "seconds sum before rounding down");
    let hi = row(&b, GoalKind::HighIntensity);
    assert_eq!((hi.recorded, hi.planned), (Some(10.), 0.));
    assert_eq!(b.recordings[0].zone2_min, Some(20.));
}

#[test]
fn unknown_evidence_is_never_zero() {
    let f = fx("unknown");
    listed(&f.store, "2025-03-12", "2025-03-13");
    goals(&f, GoalTargets { strength_sessions: 2, zone2_min: 60, ..Default::default() });
    rec(&f.store, 1, "2025-03-12 18:00:00", "cycling", 60., None);
    let b = board(&f);
    let s = row(&b, GoalKind::Strength);
    assert_eq!((s.recorded, s.remaining, s.unavailable.as_deref()), (None, None, Some("Workout list incomplete")));
    assert!(!b.coverage_complete);
    assert_eq!(day(&b, "2025-03-10").mark, Some(DayMark::Unknown));
    assert_eq!(day(&b, "2025-03-12").mark, None);
    assert_eq!(day(&b, "2025-03-15").mark, None, "days to come are blank, not unknown");

    listed(&f.store, "2025-03-01", "2025-03-13");
    let z = board(&f);
    assert_eq!(row(&z, GoalKind::Zone2).unavailable.as_deref(), Some("Heart-rate zones incomplete"));
    assert_eq!(row(&z, GoalKind::Strength).recorded, Some(0.), "a fully listed empty week is a measured zero");
}

#[test]
fn a_passed_session_is_not_recorded_until_its_person_says_otherwise() {
    let f = fx("passed");
    let p = lift(&f, "2025-03-11");
    let q = lift(&f, "2025-03-12");
    let gone = lift(&f, "2025-03-15");
    f.core.local_action(&LocalAction::SetIntentionStatus { id: q.clone(), status: IntentionStatus::Missed }, at(THU, 9)).unwrap();
    f.core.local_action(&LocalAction::SetIntentionStatus { id: gone.clone(), status: IntentionStatus::LetGo }, at(THU, 9)).unwrap();
    let b = board(&f);
    assert_eq!(item(&b, &p).state, ItemState::NotRecorded);
    assert_eq!(item(&b, &q).state, ItemState::Missed);
    assert_eq!(item(&b, &gone).state, ItemState::LetGo);
    assert!(day(&b, "2025-03-11").not_recorded);
    listed(&f.store, "2025-03-12", "2025-03-13");
    assert_eq!(item(&board(&f), &p).state, ItemState::Awaiting, "an incomplete day awaits data");
}

#[test]
fn rings_draw_only_recorded_work_in_one_order() {
    let f = fx("rings");
    goals(&f, GoalTargets { strength_sessions: 3, zone2_min: 90, ..Default::default() });
    lift(&f, "2025-03-11");
    lift(&f, "2025-03-11");
    lift(&f, "2025-03-14");
    rec(&f.store, 1, "2025-03-11 07:00:00", "strength_training", 45., Some([0; 5]));
    rec(&f.store, 2, "2025-03-12 07:00:00", "strength_training", 45., Some([0; 5]));
    let b = board(&f);
    let kinds: Vec<GoalKind> = day(&b, "2025-03-11").rings.iter().map(|r| r.kind).collect();
    assert_eq!(kinds, [GoalKind::Strength, GoalKind::Zone2]);
    let tue = &day(&b, "2025-03-11").rings[0];
    assert_eq!((tue.recorded, tue.planned, tue.fill), (1., 2., 0.5));
    let wed = &day(&b, "2025-03-12").rings[0];
    assert_eq!((wed.recorded, wed.planned, wed.fill), (1., 0., 1.), "recorded work with nothing planned fills the ring");
    let fri = &day(&b, "2025-03-14").rings[0];
    assert_eq!((fri.planned, fri.fill), (1., 0.), "planned work is never drawn");
    let week = &b.week_rings[0];
    assert_eq!((week.recorded, week.fill), (2., 2. / 3.));
}

#[test]
fn rest_illness_and_pauses_mark_their_days_and_stop_the_backlog() {
    let f = fx("marks");
    goals(&f, strength(3));
    f.core.local_action(&LocalAction::RestToday, at(11, 9)).unwrap();
    f.core.local_action(&LocalAction::Pause { reason: PauseReason::Ill, review: None }, at(12, 9)).unwrap();
    let b = board(&f);
    assert_eq!(day(&b, "2025-03-11").mark, Some(DayMark::Rest));
    assert_eq!(day(&b, "2025-03-12").mark, Some(DayMark::Ill));
    assert_eq!(day(&b, "2025-03-16").mark, Some(DayMark::Ill), "an open pause runs until resumed");
    assert!(b.backlog_paused && b.backlog.is_empty(), "nothing to catch up while ill");
    assert_eq!(row(&b, GoalKind::Strength).remaining, None);
    assert!(b.pause.as_ref().is_some_and(|p| p.reason == PauseReason::Ill && !p.whole_week));
    f.core.local_action(&LocalAction::Resume, at(THU, 9)).unwrap();
    let after = board(&f);
    assert!(!after.backlog_paused);
    assert!(after.backlog.is_empty(), "a week with a pause still adds no catch-up");
}

#[test]
fn the_backlog_lists_what_is_left_after_recordings_and_placed_sessions() {
    let f = fx("backlog");
    goals(&f, GoalTargets { strength_sessions: 3, zone2_min: 90, ..Default::default() });
    rec(&f.store, 1, "2025-03-11 07:00:00", "strength_training", 45., Some([0; 5]));
    rec(&f.store, 2, "2025-03-12 07:00:00", "cycling", 60., Some([0, 1800, 0, 0, 0]));
    rec(&f.store, 3, "2025-03-05 07:00:00", "cycling", 60., Some([0, 1800, 0, 0, 0]));
    lift(&f, "2025-03-15");
    let b = board(&f);
    let kinds: Vec<(GoalKind, f64, Option<Sport>)> = b.backlog.iter().map(|i| (i.kind, i.amount, i.sport)).collect();
    assert_eq!(kinds, [(GoalKind::Strength, 1., Some(Sport::Strength)), (GoalKind::Zone2, 60., Some(Sport::Cycling))]);
}

#[test]
fn a_mid_week_change_keeps_the_original_target() {
    let f = fx("revision");
    goals(&f, strength(3));
    rec(&f.store, 1, "2025-03-11 07:00:00", "strength_training", 45., Some([0; 5]));
    f.core.local_action(&LocalAction::SetGoals { targets: strength(1) }, at(12, 9)).unwrap();
    let l = row(&board(&f), GoalKind::Strength).clone();
    assert_eq!((l.target, l.original, l.revised_on.as_deref(), l.reached), (1., Some(3.), Some("2025-03-12"), true));
}

#[test]
fn intensity_is_garmins_weekly_total_against_its_goal_unless_chosen() {
    let f = fx("intensity");
    goals(&f, GoalTargets { intensity_garmin_goal: true, ..Default::default() });
    for (d, m, v) in [("2025-03-10", 20, 5), ("2025-03-11", 0, 0), ("2025-03-12", 30, 10), ("2025-03-13", 10, 0)] {
        summary(&f.store, d, m, v);
    }
    let b = board(&f);
    let l = row(&b, GoalKind::Intensity);
    assert_eq!((l.target, l.recorded), (150., Some(90.)));
    assert_eq!(day(&b, "2025-03-12").rings[0].recorded, 50.);
    goals(&f, GoalTargets { intensity_min: 60, ..Default::default() });
    let l = board(&f).ledger[0].clone();
    assert_eq!((l.target, l.reached), (60., true));
}

#[test]
fn sleep_timing_counts_nights_asleep_by_the_chosen_time_and_sleep_score_is_a_reading() {
    let f = fx("sleep");
    goals(&f, GoalTargets { sleep_nights: 4, sleep_time: Some("23:00".into()), sleep_score: 80, ..Default::default() });
    let night = |morning: &str, start: &str, score: u32| {
        f.store.put("sleep", morning, "now", &json!({"dailySleepDTO": {"sleepStartTimestampLocal": start,
            "sleepTimeSeconds": 25000, "sleepScores": {"overall": {"value": score}}}})).unwrap();
    };
    night("2025-03-11", "2025-03-10T23:10:00.0", 70);
    night("2025-03-12", "2025-03-11T23:20:00.0", 90);
    night("2025-03-13", "2025-03-13T00:30:00.0", 85);
    let b = board(&f);
    let l = row(&b, GoalKind::SleepTiming);
    assert_eq!((l.recorded, l.remaining), (Some(1.), Some(3.)));
    assert_eq!(day(&b, "2025-03-10").rings[0].fill, 1.);
    assert_eq!(day(&b, "2025-03-11").rings[0].fill, 0.);
    let s = b.sleep_score.unwrap();
    assert_eq!((s.target, s.average, s.nights), (80., Some(82.), 3));
    assert!(!b.week_rings.iter().any(|r| r.kind == GoalKind::SleepScore), "a Sleep Score is never a ring");
}

#[test]
fn a_migrated_sleep_target_is_an_observation_without_a_verdict() {
    let f = fx("observation");
    f.store.set_setting("plan", &json!({"versions": {"2025-03-10": {"preset": "sleep", "strengthSessions": 0,
        "intensityMin": 0, "zone2Min": 0, "asleepNights": 5, "restDays": []}}, "paused": []}).to_string()).unwrap();
    let b = board(&f);
    let l = row(&b, GoalKind::SleepTiming);
    assert!(l.observation && !l.reached && l.remaining.is_none());
    assert!(b.week_rings.is_empty() && b.days.iter().all(|d| d.rings.is_empty()));
    assert!(b.explain_observation);
    f.core.local_action(&LocalAction::ObservationExplained, at(THU, 9)).unwrap();
    assert!(!board(&f).explain_observation);
}

#[test]
fn garmin_sessions_are_listed_by_name_and_date_but_not_counted() {
    let f = fx("garmin");
    goals(&f, strength(2));
    f.store.put("scheduled_workouts", "current", &at(THU, 11).to_rfc3339(), &json!({"startDate": "2025-03-10", "endDate": "2025-03-23",
        "entries": [{"calendarDate": "2025-03-14", "workoutName": "Intervals"}]})).unwrap();
    let b = board(&f);
    assert_eq!(b.garmin.len(), 1);
    assert_eq!(b.garmin[0].name, "Intervals");
    assert_eq!(day(&b, "2025-03-14").garmin, 1);
    assert_eq!(row(&b, GoalKind::Strength).planned, 0.);
}

#[test]
fn a_future_week_shows_its_plan_and_nothing_recorded() {
    let f = fx("future");
    goals(&f, strength(2));
    let b = f.core.plan_board("2025-03-18", at(THU, 12)).unwrap();
    assert_eq!(b.start, "2025-03-17");
    assert_eq!(row(&b, GoalKind::Strength).recorded, Some(0.));
    assert!(b.backlog.is_empty(), "only this week has a backlog");
    assert!(b.days.iter().all(|d| d.future && d.rings.iter().all(|r| r.fill == 0.)));
}
