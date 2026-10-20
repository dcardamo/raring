//! The week review: outcomes against each week's own goals, counted weeks,
//! next week's proposals and their acceptance; the plan prompt and events.
use chrono::{DateTime, Local, TimeZone};
use raring_core::{store::Store, view::*, Raring};
use serde_json::{json, Value};

fn at(month: u32, day: u32, hour: u32) -> DateTime<Local> {
    Local.with_ymd_and_hms(2025, month, day, hour, 0, 0).unwrap()
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
fn fx(name: &str) -> Fx {
    let dir = std::env::temp_dir().join(format!("raring-review-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let store = Store::open(dir.join("raring.db")).unwrap();
    let core = Raring::open(dir.join("raring.db")).unwrap();
    store.set_cursor("listed:lo", "2025-02-01").unwrap();
    store.set_cursor("listed:hi", "2025-03-19").unwrap();
    Fx { dir, store, core }
}
fn ride(store: &Store, id: u64, date: &str, zone2_min: u32) {
    store
        .put("activity", &id.to_string(), "now", &json!({"activityId": id, "activityTypeDTO": {"typeKey": "cycling"},
            "summaryDTO": {"startTimeLocal": format!("{date} 07:00:00"), "duration": 3600}}))
        .unwrap();
    let zones: Vec<Value> = [0, zone2_min * 60, 0, 0, 0].iter().enumerate().map(|(i, s)| json!({"zoneNumber": i + 1, "secsInZone": s})).collect();
    store.put("zones", &id.to_string(), "now", &json!(zones)).unwrap();
}
fn lift(store: &Store, id: u64, date: &str) {
    store
        .put("activity", &id.to_string(), "now", &json!({"activityId": id, "activityTypeDTO": {"typeKey": "strength_training"},
            "summaryDTO": {"startTimeLocal": format!("{date} 18:00:00"), "duration": 2400}}))
        .unwrap();
    store.put("zones", &id.to_string(), "now", &json!([])).unwrap();
}
fn goals(f: &Fx, t: GoalTargets) {
    // Set in the week of 17 February, so it applies from that week's Monday.
    f.core.local_action(&LocalAction::SetGoals { targets: t }, at(2, 17, 8)).unwrap();
}
fn review(f: &Fx, date: &str) -> WeekReview {
    f.core.week_review(date, at(3, 17, 9)).unwrap()
}
fn proposal(r: &WeekReview, kind: GoalKind) -> GoalProposal {
    r.proposals.iter().find(|p| p.kind == kind).unwrap().clone()
}
// Weeks under review: 17 Feb, 24 Feb, 3 Mar and 10 Mar 2025; today is Monday 17 March.
const WEEKS: [&str; 4] = ["2025-02-18", "2025-02-25", "2025-03-04", "2025-03-11"];

#[test]
fn a_minute_goal_rises_only_after_two_met_weeks_with_room_to_spare() {
    let f = fx("nudge");
    goals(&f, GoalTargets { zone2_min: 100, ..Default::default() });
    for (i, (day, minutes)) in WEEKS.iter().zip([80, 90, 105, 115]).enumerate() {
        ride(&f.store, 100 + i as u64, day, minutes);
    }
    let r = review(&f, "2025-03-10");
    assert!(r.counted);
    assert_eq!(r.goals[0].state, OutcomeState::Met);
    let p = proposal(&r, GoalKind::Zone2);
    assert_eq!((p.current, p.proposed), (100., 110.));
    assert_eq!(p.reason, "Met in your last two complete weeks, 115 of 100 min last week");
    assert_eq!(r.next_start, "2025-03-17");
}

#[test]
fn nothing_rises_without_comfortable_success_and_two_short_weeks_ease_back() {
    let f = fx("ease");
    goals(&f, GoalTargets { zone2_min: 100, ..Default::default() });
    for (i, (day, minutes)) in WEEKS.iter().zip([100, 100, 60, 104]).enumerate() {
        ride(&f.store, 200 + i as u64, day, minutes);
    }
    let p = proposal(&review(&f, "2025-03-10"), GoalKind::Zone2);
    assert_eq!((p.proposed, p.reason.as_str()), (100., "Met last week; keep it steady"), "met but not by 10%");
    let g = fx("ease2");
    goals(&g, GoalTargets { zone2_min: 100, ..Default::default() });
    for (i, (day, minutes)) in WEEKS.iter().zip([100, 50, 100, 60]).enumerate() {
        ride(&g.store, 300 + i as u64, day, minutes);
    }
    let p = proposal(&review(&g, "2025-03-10"), GoalKind::Zone2);
    assert_eq!((p.proposed, p.reason.as_str()), (90., "Under target in 2 of your last 3 complete weeks"));
}

#[test]
fn sessions_rise_after_three_met_weeks_and_ease_back_after_illness() {
    let f = fx("sessions");
    goals(&f, GoalTargets { strength_sessions: 2, ..Default::default() });
    for (i, day) in WEEKS.iter().enumerate() {
        lift(&f.store, 400 + 2 * i as u64, day);
        lift(&f.store, 401 + 2 * i as u64, &day.replace("-18", "-19").replace("-25", "-26").replace("-04", "-05").replace("-11", "-12"));
    }
    let p = proposal(&review(&f, "2025-03-10"), GoalKind::Strength);
    assert_eq!((p.current, p.proposed), (2., 3.));
    // Two ill days in the reviewed week: ease back, and the week doesn't count.
    f.core.local_action(&LocalAction::Pause { reason: PauseReason::Ill, review: None }, at(3, 13, 8)).unwrap();
    f.core.local_action(&LocalAction::Resume, at(3, 15, 8)).unwrap();
    let r = review(&f, "2025-03-10");
    assert_eq!((r.ill_days, r.counted), (2, false));
    assert!(r.reasons.contains(&"2 ill days".to_owned()));
    assert_eq!(r.goals[0].state, OutcomeState::NotCounted);
    let p = proposal(&r, GoalKind::Strength);
    assert_eq!((p.proposed, p.reason.as_str()), (1., "Easing back after 2 ill days"));
}

#[test]
fn incomplete_coverage_doesnt_count_and_too_few_weeks_keep_goals() {
    let f = fx("coverage");
    f.store.set_cursor("listed:lo", "2025-03-05").unwrap();
    goals(&f, GoalTargets { zone2_min: 60, ..Default::default() });
    ride(&f.store, 500, "2025-03-11", 90);
    let r = review(&f, "2025-03-03");
    assert!(!r.counted);
    assert_eq!(r.reasons[0], "Garmin's activity list is incomplete");
    let p = proposal(&review(&f, "2025-03-10"), GoalKind::Zone2);
    assert_eq!((p.proposed, p.reason.as_str()), (60., "Not enough complete weeks to suggest a change"));
}

#[test]
fn sleep_score_and_garmins_intensity_goal_are_never_proposed() {
    let f = fx("never");
    goals(&f, GoalTargets { sleep_score: 80, intensity_garmin_goal: true, strength_sessions: 1, ..Default::default() });
    let kinds: Vec<GoalKind> = review(&f, "2025-03-10").proposals.iter().map(|p| p.kind).collect();
    assert_eq!(kinds, [GoalKind::Strength]);
}

#[test]
fn accepted_proposals_apply_from_next_week_and_nothing_changes_otherwise() {
    let f = fx("accept");
    goals(&f, GoalTargets { zone2_min: 100, strength_sessions: 2, ..Default::default() });
    for (i, (day, minutes)) in WEEKS.iter().zip([80, 90, 105, 115]).enumerate() {
        ride(&f.store, 600 + i as u64, day, minutes);
    }
    let decide = |accepted: Vec<GoalKind>| f.core.local_action(&LocalAction::DecideReview { week: "2025-03-10".into(), accepted }, at(3, 17, 9));
    assert!(decide(vec![GoalKind::HighIntensity]).is_err(), "only an offered change can be accepted");
    decide(vec![]).unwrap();
    assert_eq!(f.core.local_records().goals.versions.len(), 1, "Keep all changes nothing");
    assert!(review(&f, "2025-03-10").decision.is_some_and(|d| d.accepted.is_empty()));
    decide(vec![GoalKind::Zone2]).unwrap();
    let v = f.core.local_records().goals.versions;
    let next = v.last().unwrap();
    assert_eq!((next.from.as_str(), next.source, next.targets.zone2_min, next.targets.strength_sessions), ("2025-03-17", GoalSource::Proposal, 110, 2));
    let board = f.core.plan_board("2025-03-17", at(3, 17, 9)).unwrap();
    assert_eq!(board.ledger.iter().find(|l| l.kind == GoalKind::Zone2).unwrap().target, 110.);
    let last_week = f.core.plan_board("2025-03-12", at(3, 17, 9)).unwrap();
    assert_eq!(last_week.ledger.iter().find(|l| l.kind == GoalKind::Zone2).unwrap().target, 100., "earlier weeks keep their goals");
}

#[test]
fn the_prompt_offers_last_weeks_review_at_the_start_of_a_week_and_then_stays_quiet() {
    let f = fx("prompt");
    let board = |when| f.core.plan_board(&when_date(when), when).unwrap();
    fn when_date(d: DateTime<Local>) -> String {
        d.date_naive().to_string()
    }
    assert!(board(at(3, 17, 9)).prompt.is_none(), "no prompt for someone who never chose goals");
    goals(&f, GoalTargets { strength_sessions: 2, ..Default::default() });
    assert_eq!(board(at(3, 17, 9)).prompt.unwrap().review.as_deref(), Some("2025-03-10"));
    f.core
        .local_action(&LocalAction::AddIntention { date: "2025-03-19".into(), time: None, sport: Sport::Strength, type_key: None, name: None,
            duration_min: None, allocation: Allocation { strength_sessions: 1, ..Default::default() }, kind: IntentionKind::Workout, garmin_id: None }, at(3, 17, 9))
        .unwrap();
    assert!(board(at(3, 19, 9)).prompt.is_none(), "a planned week mid-week stays quiet");
    f.core.local_action(&LocalAction::SetPromptWeek { week: "2025-03-17".into(), choice: PromptChoice::Dismissed }, at(3, 17, 9)).unwrap();
    assert!(board(at(3, 17, 9)).prompt.is_none(), "an answered week stays quiet");
    let mut s = f.core.settings();
    s.plan_prompt = false;
    f.core.set_settings(s).unwrap();
    assert!(board(at(3, 24, 9)).prompt.is_none(), "turned off");
}

#[test]
fn local_events_and_garmins_race_count_down_together() {
    let f = fx("events");
    f.core.local_action(&LocalAction::AddEvent { name: "Club 10K".into(), date: "2025-04-05".into(), sport: Some(Sport::Running) }, at(3, 17, 9)).unwrap();
    f.store
        .put("upcoming_events", "current", &at(3, 17, 8).to_rfc3339(), &json!([{"eventType": "RACE", "eventName": "Spring Half", "calendarDate": "2025-03-29"}]))
        .unwrap();
    let events = f.core.plan_board("2025-03-17", at(3, 17, 9)).unwrap().events;
    let lines: Vec<(&str, i32, EventSource)> = events.iter().map(|e| (e.name.as_str(), e.days_until, e.source)).collect();
    assert_eq!(lines, [("Spring Half", 12, EventSource::Garmin), ("Club 10K", 19, EventSource::Local)]);
}

#[test]
fn the_review_list_covers_weeks_since_goals_began() {
    let f = fx("list");
    goals(&f, GoalTargets { zone2_min: 60, ..Default::default() });
    for (i, day) in WEEKS.iter().enumerate() {
        ride(&f.store, 700 + i as u64, day, 70);
    }
    let rows = f.core.reviews(at(3, 17, 9));
    let starts: Vec<&str> = rows.iter().map(|r| r.start.as_str()).collect();
    assert_eq!(starts, ["2025-03-10", "2025-03-03", "2025-02-24", "2025-02-17"]);
    assert!(rows.iter().all(|r| r.counted && r.met == 1 && r.goals == 1));
}

#[test]
fn an_open_week_is_not_counted_and_old_reviews_cannot_rewrite_goal_history() {
    let f=fx("review-time-boundary");
    goals(&f,GoalTargets{zone2_min:100,..Default::default()});
    for (i,day) in WEEKS.iter().enumerate(){ ride(&f.store,800+i as u64,day,120); }
    let current=f.core.week_review("2025-03-10",at(3,12,9)).unwrap();
    assert!(!current.counted);
    assert!(f.core.local_action(&LocalAction::DecideReview{week:"2025-03-10".into(),accepted:vec![GoalKind::Zone2]},at(3,12,9)).is_err());
    assert!(f.core.local_action(&LocalAction::DecideReview{week:"2025-03-10".into(),accepted:vec![GoalKind::Zone2]},at(3,31,9)).is_err());
    assert_eq!(f.core.local_records().goals.versions.len(),1);
}

#[test]
fn only_the_latest_completed_undecided_week_offers_a_decision() {
    let f = fx("decidable");
    goals(&f, GoalTargets { zone2_min: 100, ..Default::default() });
    for (i, day) in WEEKS.iter().enumerate() {
        ride(&f.store, 900 + i as u64, day, 120);
    }
    let decidable = |week: &str, when| f.core.week_review(week, when).unwrap().decidable;
    assert!(decidable("2025-03-10", at(3, 17, 9)), "the week just ended");
    assert!(decidable("2025-03-10", at(3, 20, 9)), "still the latest completed week mid-week");
    assert!(!decidable("2025-03-03", at(3, 17, 9)), "an older week is history");
    assert!(!decidable("2025-03-10", at(3, 12, 9)), "the week in progress");
    f.core.local_action(&LocalAction::DecideReview { week: "2025-03-10".into(), accepted: vec![] }, at(3, 17, 9)).unwrap();
    assert!(!decidable("2025-03-10", at(3, 17, 9)), "a decided week shows its decision");
}

#[test]
fn a_change_accepted_after_next_week_began_applies_from_today_and_keeps_the_starting_target() {
    let f = fx("late-accept");
    goals(&f, GoalTargets { zone2_min: 100, ..Default::default() });
    for (i, (day, minutes)) in WEEKS.iter().zip([80, 90, 115, 115]).enumerate() {
        ride(&f.store, 950 + i as u64, day, minutes);
    }
    let thursday = at(3, 20, 9);
    let r = f.core.week_review("2025-03-10", thursday).unwrap();
    assert_eq!((r.next_start.as_str(), r.applies_from.as_str()), ("2025-03-17", "2025-03-20"));
    assert_eq!(proposal(&r, GoalKind::Zone2).proposed, 110.);
    f.core.local_action(&LocalAction::DecideReview { week: "2025-03-10".into(), accepted: vec![GoalKind::Zone2] }, thursday).unwrap();
    let v = f.core.local_records().goals.versions;
    assert_eq!((v.last().unwrap().from.as_str(), v.last().unwrap().targets.zone2_min), ("2025-03-20", 110));
    let board = f.core.plan_board("2025-03-20", thursday).unwrap();
    let zone2 = board.ledger.iter().find(|l| l.kind == GoalKind::Zone2).unwrap();
    assert_eq!((zone2.target, zone2.original, zone2.revised_on.as_deref()), (110., Some(100.), Some("2025-03-20")));
}
