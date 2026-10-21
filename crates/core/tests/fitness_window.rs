//! Fitness Age's vigorous window: Garmin's 42-day rule over Garmin's daily
//! figures, where Today shows the card, the push-day line, and how a recording
//! and the plan week count toward Garmin's target.
use chrono::{Duration, FixedOffset, Local, NaiveDate, TimeZone};
use raring_core::{fitness, ledger, screens, store::Store, view::*, Raring};
use serde_json::{json, Value};

/// A Thursday; its plan week starts Monday the 17th.
const END: &str = "2025-03-20";
fn end() -> NaiveDate {
    NaiveDate::parse_from_str(END, "%Y-%m-%d").unwrap()
}
fn back(days: i64) -> String {
    (end() - Duration::days(days)).to_string()
}
fn estimate(store: &Store, day: &str, age: f64, vigorous: Value) {
    store.put("fitness_age", day, "now", &json!({"fitnessAge": age, "chronologicalAge": 46,
        "components": {"vigorousMinutesAvg": vigorous, "rhr": {"value": 50}}})).unwrap();
}
fn unmet() -> Value {
    json!({"value": 10.0, "targetValue": 75, "potentialAge": 37.8})
}
/// Garmin's daily summaries for the 42 days ending END; None leaves the figure out.
fn days(store: &Store, minutes: impl Fn(i64) -> Option<f64>) {
    for b in 0..42 {
        let mut s = json!({"moderateIntensityMinutes": 0});
        if let Some(m) = minutes(b) {
            s["vigorousIntensityMinutes"] = json!(m);
        }
        store.put("summary", &back(b), "now", &s).unwrap();
    }
}
/// 14 six weeks ago, 20 ten days ago, 6 on Monday and 18 today.
fn pattern(b: i64) -> Option<f64> {
    Some(match b {
        41 => 14.,
        10 => 20.,
        3 => 6.,
        0 => 18.,
        _ => 0.,
    })
}
fn store() -> Store {
    let store = Store::open(":memory:").unwrap();
    estimate(&store, END, 39., unmet());
    days(&store, pattern);
    store
}
fn today_view(store: &Store) -> TodayView {
    let now = FixedOffset::east_opt(0).unwrap().from_local_datetime(&end().and_hms_opt(10, 0, 0).unwrap()).unwrap();
    screens::today_at(store, END, now)
}

#[test]
fn the_window_is_garmins_42_days_per_week() {
    let w = fitness::window(&store(), end()).unwrap();
    assert_eq!(w.end, END);
    // Six seven-day blocks ending today, oldest first; their sum ÷ 6 is the average.
    assert_eq!(w.weeks, vec![Some(14.), Some(0.), Some(0.), Some(0.), Some(20.), Some(24.)]);
    assert_eq!(w.today, Some(18.));
    let average = w.average.unwrap();
    assert!((average - 58. / 6.).abs() < 1e-9, "{average}");
    assert!((w.before_today.unwrap() - 40. / 6.).abs() < 1e-9);
    assert_eq!((w.target, w.age_at_target), (75., Some(37.8)));
}

#[test]
fn a_missing_day_is_unknown_never_zero() {
    let store = store();
    // A summary without Garmin's figure, and a day with no summary at all.
    store.put("summary", &back(9), "now", &json!({"moderateIntensityMinutes": 5})).unwrap();
    let w = fitness::window(&store, end()).unwrap();
    assert_eq!(w.weeks[4], None, "the block with the gap");
    assert_eq!(w.weeks[1], Some(0.), "explicit zeros are measured");
    assert_eq!((w.average, w.before_today), (None, None));
    let store = Store::open(":memory:").unwrap();
    estimate(&store, END, 39., unmet());
    days(&store, pattern);
    store.put("summary", &back(0), "now", &json!({})).unwrap();
    let w = fitness::window(&store, end()).unwrap();
    assert_eq!((w.today, w.average, w.before_today), (None, None, None), "today's figure not in yet");
}

#[test]
fn the_target_comes_from_garmins_latest_estimate_in_either_shape() {
    let store = Store::open(":memory:").unwrap();
    days(&store, pattern);
    assert_eq!(fitness::window(&store, end()), None, "no estimate");
    store.put("fitness_age", &back(2), "now", &json!({"fitnessAge": 39, "fitnessAgeData": {"vigorousMinutes": {"value": 9, "target": 75, "ageIfMet": 38}}})).unwrap();
    let w = fitness::window(&store, end()).unwrap();
    assert_eq!((w.target, w.age_at_target), (75., Some(38.)), "the latest estimate before today, legacy shape");
    estimate(&store, END, 39., json!({"value": 80.0}));
    assert_eq!(fitness::window(&store, end()), None, "no target, no window");
}

#[test]
fn the_card_goes_where_the_day_moved_it() {
    let store = store();
    let place = |store: &Store| fitness::today(store, &today_view(store), end()).unwrap().card;
    assert_eq!(place(&store), Some(FitnessPlace::AfterWeek), "vigorous minutes today");
    estimate(&store, &back(1), 44.4, unmet());
    estimate(&store, END, 43.4, unmet());
    let f = fitness::today(&store, &today_view(&store), end()).unwrap();
    assert_eq!((f.card, f.dropped_from, f.age), (Some(FitnessPlace::Hero), Some(44.4), Some(43.4)), "Garmin's estimate fell a whole year");
    estimate(&store, END, 44.2, unmet());
    assert_eq!(place(&store), Some(FitnessPlace::AfterWeek), "under a year is no new estimate");
    estimate(&store, END, 45.2, unmet());
    assert_eq!(place(&store), Some(FitnessPlace::AfterWeek), "a rise stays on Health");
    store.put("summary", END, "now", &json!({"vigorousIntensityMinutes": 0, "moderateIntensityMinutes": 0})).unwrap();
    assert_eq!(place(&store), None, "a day that moved neither");
    estimate(&store, END, 43.4, unmet());
    let mut view = today_view(&store);
    assert_eq!(fitness::today(&store, &view, end()).unwrap().card, Some(FitnessPlace::Hero));
    view.focus.phase = DayPhase::WindDown;
    assert_eq!(fitness::today(&store, &view, end()).unwrap().card, None, "tonight's plan stays alone");
}

#[test]
fn the_lever_line_needs_garmins_push_call_before_any_vigorous_minutes() {
    let store = store();
    store.put("summary", END, "now", &json!({"vigorousIntensityMinutes": 0, "moderateIntensityMinutes": 0})).unwrap();
    let mut view = today_view(&store);
    let line = |view: &TodayView| fitness::today(&store, view, end()).unwrap().lever_line;
    (view.call, view.focus.effort) = (Some(Effort::Push), true);
    assert!(line(&view), "push, vigorous minutes the biggest lever, none yet");
    view.call = Some(Effort::Steady);
    assert!(!line(&view), "only Garmin's push call");
    (view.call, view.focus.effort) = (Some(Effort::Push), false);
    assert!(!line(&view), "no effort call shown (rest, illness, limits, a recorded workout)");
    view.focus.effort = true;
    store.put("summary", END, "now", &json!({"vigorousIntensityMinutes": 12, "moderateIntensityMinutes": 0})).unwrap();
    assert!(!line(&view), "vigorous minutes today: the card says it");
    store.put("summary", END, "now", &json!({"vigorousIntensityMinutes": 0, "moderateIntensityMinutes": 0})).unwrap();
    store.put("fitness_age", END, "now", &json!({"fitnessAge": 39, "components": {
        "vigorousMinutesAvg": {"value": 80.0, "targetValue": 75}, "bmi": {"value": 26.0, "targetValue": 24}}})).unwrap();
    assert!(!line(&view), "another driver is the lever");
}

#[test]
fn a_recording_counts_while_inside_the_window() {
    let store = store();
    let monday = back(3);
    let c = fitness::count(&store, Some(18.), end() - Duration::days(3), end()).unwrap();
    assert_eq!((c.minutes, c.week, c.this_week, c.target), (18., Some(24.), true, 75.));
    assert_eq!(c.until, (end() + Duration::days(38)).to_string(), "its day and the 41 after");
    assert_eq!(fitness::count(&store, Some(0.), end(), end()), None, "no vigorous minutes, no line");
    assert_eq!(fitness::count(&store, None, end(), end()), None);
    let last = fitness::count(&store, Some(14.), end() - Duration::days(41), end()).unwrap();
    assert_eq!((last.until.as_str(), last.this_week), (END, false), "the oldest day still counts today");
    assert_eq!(fitness::count(&store, Some(14.), end() - Duration::days(42), end()), None, "out of the window");
    let week = fitness::plan_week(&store, NaiveDate::parse_from_str(&monday, "%Y-%m-%d").unwrap(), end()).unwrap();
    assert_eq!((week.vigorous, week.target), (Some(24.), 75.));
    assert_eq!(fitness::plan_week(&store, end() - Duration::days(10), end()), None, "only the current week");
}

#[test]
fn the_views_carry_the_window() {
    let store = store();
    let now = chrono::Utc.with_ymd_and_hms(2025, 3, 20, 10, 0, 0).unwrap();
    assert_eq!(screens::health(&store, END).vigorous, fitness::window(&store, end()));
    let board = ledger::board(&store, end(), end(), now);
    assert_eq!(board.fitness.map(|f| (f.vigorous, f.target)), Some((Some(24.), 75.)));
    store.put("activity", "7", "now", &json!({"activityId": 7, "activityName": "Ride", "activityTypeDTO": {"typeKey": "cycling"},
        "summaryDTO": {"startTimeLocal": "2025-03-20 07:00:00", "duration": 3600, "moderateIntensityMinutes": 20, "vigorousIntensityMinutes": 13}})).unwrap();
    let w = screens::workout(&store, "7").unwrap();
    assert_eq!((w.summary.vigorous_min, w.summary.intensity_min), (Some(13.), Some(46.)));
}

#[test]
fn a_recording_without_goals_still_says_how_it_counted_toward_fitness_age() {
    let dir = std::env::temp_dir().join(format!("raring-fitness-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let store = Store::open(dir.join("raring.db")).unwrap();
    estimate(&store, END, 39., unmet());
    days(&store, pattern);
    store.put("activity", "7", "now", &json!({"activityId": 7, "activityName": "Ride", "activityTypeDTO": {"typeKey": "cycling"},
        "summaryDTO": {"startTimeLocal": "2025-03-20 07:00:00", "duration": 3600, "moderateIntensityMinutes": 20, "vigorousIntensityMinutes": 13}})).unwrap();
    let core = Raring::open(dir.join("raring.db")).unwrap();
    let now = Local.with_ymd_and_hms(2025, 3, 20, 18, 0, 0).unwrap();
    let counted = core.workout_live("7", now).unwrap().counted.unwrap();
    assert!(!counted.goals);
    let f = counted.fitness.unwrap();
    assert_eq!((f.minutes, f.week, f.target, f.until.as_str()), (13., Some(24.), 75., "2025-04-30"));
    let today = core.today_live_at(END, now);
    assert!(today.fitness.is_some(), "the live Today carries it");
    assert!(core.today(END).fitness.is_none(), "a historical read never does");
    drop(core);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn the_lever_line_reads_the_windows_estimate_and_names_only_vigorous_minutes() {
    let store = Store::open(":memory:").unwrap();
    days(&store, pattern);
    store.put("summary", END, "now", &json!({"vigorousIntensityMinutes": 0, "moderateIntensityMinutes": 0})).unwrap();
    estimate(&store, &back(2), 39., unmet());
    let mut view = today_view(&store);
    (view.call, view.focus.effort) = (Some(Effort::Push), true);
    assert!(fitness::today(&store, &view, end()).unwrap().lever_line, "between Garmin's estimates");
    store.put("fitness_age", &back(2), "now", &json!({"fitnessAge": 39, "components": {
        "vigorousMinutesAvg": {"value": 80.0, "targetValue": 75}, "vigorousDaysAvg": {"value": 1.0, "targetValue": 3}}})).unwrap();
    assert!(!fitness::today(&store, &view, end()).unwrap().lever_line, "vigorous days as the lever: the line would misname it");
}

#[test]
fn a_weeks_vigorous_minutes_need_only_garmins_vigorous_figures() {
    let store = Store::open(":memory:").unwrap();
    estimate(&store, END, 39., unmet());
    for b in 0..42 {
        store.put("summary", &back(b), "now", &json!({"vigorousIntensityMinutes": if b == 0 { 18 } else { 2 }})).unwrap();
    }
    let monday = end() - Duration::days(3);
    assert_eq!(fitness::plan_week(&store, monday, end()).unwrap().vigorous, Some(24.), "no moderate figures needed");
    store.put("summary", &back(1), "now", &json!({"moderateIntensityMinutes": 4})).unwrap();
    assert_eq!(fitness::plan_week(&store, monday, end()).unwrap().vigorous, None, "a day without Garmin's vigorous figure");
    assert_eq!(fitness::count(&store, Some(5.), monday, end()).unwrap().week, None);
}
