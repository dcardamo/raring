use chrono::{Duration, Local, NaiveDate, TimeZone};
use raring_core::{plan, screens, store::Store, view::*};
use serde_json::{json, Value};

const TODAY: &str = "2025-04-10";
fn day(s: &str) -> NaiveDate {
    s.parse().unwrap()
}
fn put(s: &Store, kind: &str, key: &str, v: Value) {
    s.put(kind, key, "synthetic", &v).unwrap();
}
fn setup(s: &Store, date: &str, targets: PlanTargets) {
    plan::set(
        s,
        day(date),
        &PlanUpdate {
            targets,
            paused: false,
        },
    )
    .unwrap();
}
fn targets() -> PlanTargets {
    PlanTargets::preset(PlanPreset::Strength)
}
fn summary(s: &Store, d: &str) {
    put(
        s,
        "summary",
        d,
        json!({"moderateIntensityMinutes":0,"vigorousIntensityMinutes":0,"intensityMinutesGoal":170}),
    );
}
fn listed(s: &Store) {
    s.set_cursor("listed:lo", "2025-03-31").unwrap();
    s.set_cursor("listed:hi", TODAY).unwrap();
}
fn target(s: &Store, kind: PlanTargetKind) -> PlanProgress {
    plan::view(s, TODAY)
        .week
        .items
        .into_iter()
        .find(|i| i.kind == kind)
        .unwrap()
}
fn activity(s: &Store) {
    // Genuine timer-only strength activity: deliberately no set or lap data.
    put(
        s,
        "activity",
        "201",
        json!({"activityId":201,"startTimeLocal":"2025-04-08T12:00:00","activityType":{"typeKey":"strength_training"},"duration":1200}),
    );
}

#[test]
fn fresh_and_partially_backfilled_databases_never_count_missing_as_failure() {
    let s = Store::open(":memory:").unwrap();
    setup(&s, TODAY, targets());
    let w = plan::view(&s, TODAY).week;
    assert_eq!(w.completion, None);
    assert_eq!(w.total, 0);
    assert_eq!(w.headline, "Not enough evidence");
    assert!(w
        .items
        .iter()
        .all(|i| i.unavailable.is_some() && i.state != PlanState::Behind));
    summary(&s, TODAY);
    activity(&s);
    let w = plan::view(&s, TODAY).week;
    assert!(w.items.iter().all(|i| i.unavailable.is_some()));
    assert_eq!(w.items[0].value, 1.); // known partial count retained, not scored
    assert_eq!(w.completion, None);
    assert!(screens::health(&s, TODAY).intensity.is_none());
}

#[test]
fn indexed_activity_without_payload_and_missing_zones_are_unknown() {
    let s = Store::open(":memory:").unwrap();
    setup(&s, TODAY, targets());
    listed(&s);
    // Pagination later overwrote the page before activity detail arrived.
    put(
        &s,
        "activities",
        "0",
        json!([{"activityId":201,"startTimeLocal":"2025-04-08T12:00:00"}]),
    );
    put(&s, "activities", "0", json!([]));
    assert!(target(&s, PlanTargetKind::Strength).unavailable.is_some());
    assert!(target(&s, PlanTargetKind::Zone2).unavailable.is_some());
    activity(&s);
    let strength = target(&s, PlanTargetKind::Strength);
    assert_eq!(strength.value, 1.);
    assert!(
        strength.unavailable.is_none(),
        "timer-only strength counts without sets"
    );
    assert!(target(&s, PlanTargetKind::Zone2).unavailable.is_some());
    put(&s, "zones", "201", Value::Null);
    assert!(target(&s, PlanTargetKind::Zone2).unavailable.is_some());
    for zones in [json!([]), json!([{"zoneNumber":2,"secsInZone":0}])] {
        put(&s, "zones", "201", zones);
        let z = target(&s, PlanTargetKind::Zone2);
        assert_eq!(z.value, 0.);
        assert!(z.unavailable.is_none());
        assert_eq!(z.state, PlanState::Behind);
    }
    for malformed in [
        json!([{"zoneNumber":2}]),
        json!([{"zoneNumber":2,"secsInZone":-5}]),
        json!({}),
    ] {
        put(&s, "zones", "201", malformed);
        assert!(target(&s, PlanTargetKind::Zone2).unavailable.is_some());
    }
}

#[test]
fn intensity_needs_all_days_or_authoritative_weekly_row_not_future_days() {
    let s = Store::open(":memory:").unwrap();
    setup(&s, "2025-03-31", targets());
    summary(&s, "2025-04-03");
    assert!(plan::intensity(&s, day("2025-03-31"), day(TODAY)).is_none());
    put(
        &s,
        "intensity_weeks",
        "me",
        json!([{"calendarDate":"2025-03-31","moderateValue":21,"vigorousValue":13,"weeklyGoal":170}]),
    );
    assert_eq!(
        plan::intensity(&s, day("2025-03-31"), day(TODAY))
            .unwrap()
            .total,
        47.
    );
    for date in ["2025-04-07", "2025-04-08", "2025-04-09"] {
        summary(&s, date);
    }
    assert!(plan::intensity(&s, day("2025-04-07"), day(TODAY)).is_none());
    summary(&s, TODAY);
    assert_eq!(
        plan::intensity(&s, day("2025-04-07"), day(TODAY))
            .unwrap()
            .total,
        0.
    );
    // No Fri/Sat/Sun summaries required on Thursday.
    assert!(s.get("summary", "2025-04-11").is_none());
}

#[test]
fn complete_zero_week_is_zero_and_headlines_never_claim_unknown_targets_reached() {
    let s = Store::open(":memory:").unwrap();
    listed(&s);
    let mut t = targets();
    t.asleep_nights = 0;
    setup(&s, "2025-03-31", t.clone());
    for d in day("2025-03-31").iter_days().take(7) {
        summary(&s, &d.to_string());
    }
    let check = plan::checkin(&s, day("2025-04-07")).unwrap();
    assert_eq!(check.week.completion, Some(0.));
    assert_eq!(check.week.headline, "No targets reached");
    assert!(check.week.items.iter().all(|i| i.unavailable.is_none()));
    // One complete goal, other targets unknown, must not say "All reached".
    t.strength_sessions = 0;
    t.zone2_min = 0;
    t.asleep_nights = 5;
    t.intensity_min = Some(1);
    setup(&s, "2025-03-31", t);
    put(
        &s,
        "summary",
        "2025-04-01",
        json!({"moderateIntensityMinutes":2,"vigorousIntensityMinutes":0}),
    );
    let check = plan::checkin(&s, day("2025-04-07")).unwrap();
    assert_eq!(check.week.reached, 1);
    assert_eq!(check.week.headline, "Partial evidence");
    assert!(!check.week.detail.contains("Every target"));
    assert_eq!(check.week.completion, Some(1.)); // only known goal, not unknown sleep as zero
                                                 // With only the fully measured target enabled, all reached is honest.
    let mut t = targets();
    t.strength_sessions = 0;
    t.zone2_min = 0;
    t.asleep_nights = 0;
    t.intensity_min = Some(1);
    setup(&s, "2025-03-31", t.clone());
    assert_eq!(
        plan::checkin(&s, day("2025-04-07")).unwrap().week.headline,
        "All reached"
    );
    t.strength_sessions = 2;
    setup(&s, "2025-03-31", t.clone());
    assert_eq!(
        plan::checkin(&s, day("2025-04-07")).unwrap().week.headline,
        "Partly reached"
    );
    plan::set(
        &s,
        day("2025-03-31"),
        &PlanUpdate {
            targets: t,
            paused: true,
        },
    )
    .unwrap();
    let w = plan::checkin(&s, day("2025-04-07")).unwrap().week;
    assert_eq!(w.headline, "Paused");
    assert_eq!(w.completion, None);
}

fn night(s: &Store, date: &str, start: &str, end_minutes: i32) {
    put(
        s,
        "sleep",
        date,
        json!({"dailySleepDTO":{"sleepStartTimestampLocal":start,"sleepTimeSeconds":26000,"nextSleepNeed":{"actual":480,"recommendedBedtimeStartMins":1320,"recommendedBedtimeEndMins":end_minutes}}}),
    );
}
fn capture(s: &Store, date: &str, hour: u32) {
    let now = Local
        .from_local_datetime(&day(date).and_hms_opt(hour, 0, 0).unwrap())
        .single()
        .unwrap();
    plan::capture_sleep_plan(s, now).unwrap();
}
#[test]
fn backfill_and_chart_reads_cannot_invent_sleep_adherence() {
    let s = Store::open(":memory:").unwrap();
    setup(&s, TODAY, targets());
    night(&s, "2025-04-08", "2025-04-07T22:00:00", 1350);
    night(&s, "2025-04-09", "2025-04-08T22:15:00", 1350);
    let _ = screens::sleep_view(&s, "2025-04-08");
    let _ = screens::today(&s, "2025-04-08");
    let _ = plan::checkin(&s, day("2025-04-14"));
    assert!(s.setting("sleep_plan:2025-04-09").is_none());
    let n = target(&s, PlanTargetKind::AsleepOnTime);
    assert!(n.unavailable.is_some());
    assert_eq!(n.value, 0.);
    // A live capture after the outcome arrived still cannot backdate a target.
    capture(&s, "2025-04-08", 18);
    assert!(s.setting("sleep_plan:2025-04-09").is_none());
}

#[test]
fn captured_before_night_is_scored_and_cannot_be_rewritten() {
    let s = Store::open(":memory:").unwrap();
    setup(&s, TODAY, targets());
    night(&s, "2025-04-08", "2025-04-07T22:00:00", 1350);
    capture(&s, "2025-04-08", 18);
    let saved = s.setting("sleep_plan:2025-04-09").unwrap();
    let v: Value = serde_json::from_str(&saved).unwrap();
    assert_eq!(v["plan"]["night"], "2025-04-09");
    assert_eq!(v["plan"]["bedtimeSource"], "garmin");
    assert!(v["capturedAt"]
        .as_str()
        .unwrap()
        .starts_with("2025-04-08T18:00:00"));
    // First prospective capture is immutable, even before sleep arrives.
    night(&s, "2025-04-08", "2025-04-07T22:00:00", 1400);
    capture(&s, "2025-04-08", 19);
    assert_eq!(s.setting("sleep_plan:2025-04-09").unwrap(), saved);
    night(&s, "2025-04-09", "2025-04-08T22:40:00", 1350);
    capture(&s, "2025-04-08", 20);
    assert_eq!(s.setting("sleep_plan:2025-04-09").unwrap(), saved);
    let n = target(&s, PlanTargetKind::AsleepOnTime);
    assert!(n.unavailable.is_none());
    assert_eq!(n.value, 1.);
    // A later upload revealing sleep before capture invalidates the evidence.
    night(&s, "2025-04-09", "2025-04-08T17:00:00", 1350);
    assert!(target(&s, PlanTargetKind::AsleepOnTime)
        .unavailable
        .is_some());
}

#[test]
fn first_setup_never_fabricates_history_but_setup_preview_remains_retrospective() {
    let s = Store::open(":memory:").unwrap();
    listed(&s);
    for d in day("2025-03-31").iter_days().take(7) {
        summary(&s, &d.to_string());
    }
    setup(&s, TODAY, targets());
    let v = plan::view(&s, TODAY);
    assert!(v.history[..7].iter().all(|w| w.completion.is_none()));
    assert!(
        v.last_week.completion.is_some(),
        "explicitly labelled comparison still works"
    );
    assert!(plan::checkin(&s, day("2025-04-07")).is_none());
    let prior = v.week.items[0].target;
    let mut t = targets();
    t.strength_sessions = 6;
    setup(&s, "2025-04-15", t);
    assert_eq!(plan::view(&s, TODAY).week.items[0].target, prior);
    assert!(plan::view(&s, "2025-04-15").history[..6]
        .iter()
        .all(|w| w.completion.is_none()));
}

#[test]
fn late_capture_and_future_night_never_create_targets() {
    let s = Store::open(":memory:").unwrap();
    night(&s, "2025-04-08", "2025-04-07T22:00:00", 1350);
    capture(&s, "2025-04-08", 23);
    assert!(s.setting("sleep_plan:2025-04-09").is_none());
    let mut v = s.get("sleep", "2025-04-08").unwrap();
    v["dailySleepDTO"]["nextSleepNeed"]["calendarDate"] =
        json!((day("2025-04-08") + Duration::days(3)).to_string());
    put(&s, "sleep", "2025-04-08", v);
    capture(&s, "2025-04-08", 18);
    assert!(s.setting("sleep_plan:2025-04-11").is_none());
}
