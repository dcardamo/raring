//! Progress periods: bounds, known totals, honest comparisons, coverage,
//! the calendar, goals over time, sleep against need and milestones.
use chrono::{DateTime, Local, TimeZone};
use raring_core::{store::Store, view::*, Raring};
use serde_json::json;

fn at(month: u32, day: u32) -> DateTime<Local> {
    Local.with_ymd_and_hms(2025, month, day, 12, 0, 0).unwrap()
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
    let dir = std::env::temp_dir().join(format!("raring-progress-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let store = Store::open(dir.join("raring.db")).unwrap();
    let core = Raring::open(dir.join("raring.db")).unwrap();
    store.set_cursor("listed:lo", "2025-09-01").unwrap();
    store.set_cursor("listed:hi", "2025-11-18").unwrap();
    Fx { dir, store, core }
}
fn act(store: &Store, id: u64, date: &str, kind: &str, minutes: f64, metres: Option<f64>) {
    let mut summary = json!({"startTimeLocal": format!("{date} 07:00:00"), "duration": minutes * 60.});
    if let Some(m) = metres {
        summary["distance"] = json!(m);
    }
    store.put("activity", &id.to_string(), "now", &json!({"activityId": id, "activityName": kind, "activityTypeDTO": {"typeKey": kind}, "summaryDTO": summary})).unwrap();
}
fn view(f: &Fx, period: Period, date: &str, today: DateTime<Local>) -> ProgressView {
    f.core.progress(period, date, today).unwrap()
}

#[test]
fn periods_have_calendar_bounds_and_never_pass_today() {
    let f = fx("bounds");
    let m = view(&f, Period::Month, "2025-11-18", at(11, 18));
    assert_eq!((m.from.as_str(), m.to.as_str(), m.end.as_str(), m.current), ("2025-11-01", "2025-11-30", "2025-11-18", true));
    let q = view(&f, Period::Quarter, "2025-11-18", at(11, 18));
    assert_eq!((q.from.as_str(), q.to.as_str()), ("2025-09-01", "2025-11-30"));
    let y = view(&f, Period::Year, "2025-11-18", at(11, 18));
    assert_eq!(y.from, "2024-12-01");
    let w = view(&f, Period::Week, "2025-11-18", at(11, 18));
    assert_eq!((w.from.as_str(), w.to.as_str()), ("2025-11-17", "2025-11-23"));
    assert_eq!(w.buckets.len(), 7);
    assert_eq!(q.buckets.iter().filter(|b| !b.future).count(), q.buckets.len() - 1, "the last partial week is still current");
    assert_eq!(y.buckets.len(), 12);
}

#[test]
fn totals_are_known_totals_and_distance_only_adds_recorded_distance() {
    let f = fx("totals");
    act(&f.store, 1, "2025-11-03", "running", 50., Some(10_000.));
    act(&f.store, 2, "2025-11-05", "strength_training", 40., None);
    act(&f.store, 3, "2025-11-05", "cycling", 60., Some(25_400.));
    f.store.put("summary", "2025-11-03", "now", &json!({"moderateIntensityMinutes": 10, "vigorousIntensityMinutes": 20})).unwrap();
    let m = view(&f, Period::Month, "2025-11-18", at(11, 18));
    let t = &m.totals;
    assert_eq!((t.activities, t.active_min, t.distance_km, t.intensity_min), (3, 150., 35.4, Some(50.)));
    assert_eq!((t.active_days, t.covered_days, t.days), (2, 18, 18));
    let buckets: Vec<f64> = m.buckets.iter().map(|b| b.total).collect();
    assert_eq!((buckets[2], buckets[4]), (50., 100.));
    assert_eq!(m.buckets[4].minutes[0].sport, Sport::Cycling, "the day's sports, longest first");
}

#[test]
fn a_current_period_compares_the_same_elapsed_days_and_says_nothing_when_it_cant() {
    let f = fx("compare");
    act(&f.store, 1, "2025-10-10", "running", 30., None);
    act(&f.store, 2, "2025-10-25", "running", 30., None);
    let m = view(&f, Period::Month, "2025-11-18", at(11, 18));
    assert_eq!((m.previous_from.as_deref(), m.previous_to.as_deref()), (Some("2025-10-01"), Some("2025-10-18")));
    assert_eq!(m.previous.unwrap().activities, 1, "1–18 Oct, not all of October");
    let done = view(&f, Period::Month, "2025-10-15", at(11, 18));
    assert_eq!(done.previous_to.as_deref(), Some("2025-09-30"), "a completed month compares the whole previous one");
    let sept = view(&f, Period::Month, "2025-09-15", at(11, 18));
    assert!(sept.previous.is_none(), "no comparison before downloaded history");
    f.store.set_cursor("listed:lo", "2025-11-05").unwrap();
    let gap = view(&f, Period::Month, "2025-11-18", at(11, 18));
    assert!(gap.partial && gap.previous.is_none(), "incomplete coverage gives known totals and no comparison");
}

#[test]
fn a_shorter_previous_month_cant_supply_the_same_days() {
    let f = fx("short");
    f.store.set_cursor("listed:lo", "2025-01-01").unwrap();
    f.store.set_cursor("listed:hi", "2025-03-30").unwrap();
    let m = view(&f, Period::Month, "2025-03-30", at(3, 30));
    assert!(m.previous.is_none(), "1–30 Feb doesn't exist without overlapping March");
}

#[test]
fn the_calendar_keeps_rest_pauses_gaps_and_future_days_distinct() {
    let f = fx("calendar");
    // The activity list is complete through 15 November; later days are a gap.
    f.store.set_cursor("listed:hi", "2025-11-15").unwrap();
    act(&f.store, 1, "2025-11-17", "running", 30., None);
    act(&f.store, 2, "2025-11-10", "running", 30., None);
    f.core.local_action(&LocalAction::RestToday, at(11, 11)).unwrap();
    f.core.local_action(&LocalAction::Pause { reason: PauseReason::Ill, review: None }, at(11, 12)).unwrap();
    f.core.local_action(&LocalAction::Resume, at(11, 14)).unwrap();
    let m = view(&f, Period::Month, "2025-11-18", at(11, 18));
    let state = |d: &str| m.calendar.iter().find(|c| c.date == d).unwrap();
    assert_eq!((state("2025-11-17").state, state("2025-11-17").covered), (CalendarState::Active, false), "a known workout keeps its mark on an uncovered day");
    assert_eq!(state("2025-11-16").state, CalendarState::Unknown);
    assert_eq!(state("2025-11-10").state, CalendarState::Active);
    assert_eq!(state("2025-11-11").state, CalendarState::Rest);
    assert_eq!(state("2025-11-12").state, CalendarState::Ill);
    assert_eq!(state("2025-11-14").state, CalendarState::None, "resumed");
    assert_eq!(state("2025-11-19").state, CalendarState::Future);
    assert_eq!((m.totals.active_days, m.totals.covered_days), (1, 15), "uncovered days count toward neither");
    let before = view(&f, Period::Month, "2025-08-10", at(11, 18));
    assert_eq!(before.calendar[0].state, CalendarState::Before);
}

#[test]
fn sleep_against_need_counts_nights_with_both_and_no_running_total() {
    let f = fx("sleep");
    for (d, asleep, need) in [("2025-11-16", 300, 480), ("2025-11-17", 360, 480), ("2025-11-18", 450, 480), ("2025-11-19", 300, 0)] {
        f.store.put("sleep", d, "now", &json!({"dailySleepDTO": {"sleepTimeSeconds": asleep * 60, "sleepNeed": {"actual": need}}})).unwrap();
    }
    let s = view(&f, Period::Week, "2025-11-18", at(11, 18)).sleep.unwrap();
    assert_eq!((s.nights, s.below_hour), (2, 1));
}

fn sets(store: &Store, id: u64, date: &str, weight_g: f64, reps: u32) {
    act(store, id, date, "strength_training", 40., None);
    store
        .put("exercise_sets", &id.to_string(), "now", &json!({"exerciseSets": [
            {"setType": "ACTIVE", "repetitionCount": reps, "weight": weight_g, "exercises": [{"category": "SQUAT", "name": "BARBELL_BACK_SQUAT", "probability": 0.9}]},
            {"setType": "ACTIVE", "repetitionCount": reps, "weight": weight_g, "exercises": [{"category": "SQUAT", "name": "BARBELL_BACK_SQUAT", "probability": 0.9}]}]}))
        .unwrap();
}

#[test]
fn bests_are_awarded_once_and_a_garmin_edit_corrects_without_a_new_award() {
    let f = fx("milestones");
    sets(&f.store, 1, "2025-11-03", 90_000., 5);
    sets(&f.store, 2, "2025-11-10", 100_000., 5);
    let m = view(&f, Period::Month, "2025-11-18", at(11, 18));
    let best = m.milestones.iter().find(|x| x.id == "best:SQUAT/BARBELL_BACK_SQUAT:5").unwrap();
    assert_eq!((best.value.as_str(), best.date.as_str(), best.new, best.corrected), ("100 kg", "2025-11-10", false, false));
    assert_eq!(best.title, "Barbell Back Squat · best for 5 reps");
    f.core.local_action(&LocalAction::MilestonesSeen, at(11, 18)).unwrap();
    let again = view(&f, Period::Month, "2025-11-18", at(11, 18));
    assert!(again.milestones.iter().all(|x| !x.new), "seen once, never replayed");
    // A Garmin edit removes the heavier set: the record moves back, corrected once.
    sets(&f.store, 2, "2025-11-10", 80_000., 5);
    let edited = view(&f, Period::Month, "2025-11-18", at(11, 18));
    let best = edited.milestones.iter().find(|x| x.id == "best:SQUAT/BARBELL_BACK_SQUAT:5").unwrap();
    assert_eq!((best.value.as_str(), best.date.as_str(), best.new, best.corrected), ("90 kg", "2025-11-03", false, true));
    // A genuinely heavier set later is a new award.
    sets(&f.store, 3, "2025-11-18", 105_000., 5);
    let best = view(&f, Period::Month, "2025-11-18", at(11, 18)).milestones.into_iter().find(|x| x.id == "best:SQUAT/BARBELL_BACK_SQUAT:5").unwrap();
    assert_eq!((best.value.as_str(), best.new, best.corrected), ("105 kg", true, false));
}

#[test]
fn the_longest_distance_needs_a_few_recordings_of_the_sport() {
    let f = fx("longest");
    act(&f.store, 1, "2025-11-03", "running", 30., Some(5_000.));
    act(&f.store, 2, "2025-11-05", "running", 50., Some(9_800.));
    let none = view(&f, Period::Month, "2025-11-18", at(11, 18));
    assert!(none.milestones.iter().all(|m| m.kind != MilestoneKind::Longest));
    act(&f.store, 3, "2025-11-07", "running", 40., Some(7_000.));
    let m = view(&f, Period::Month, "2025-11-18", at(11, 18));
    let l = m.milestones.iter().find(|m| m.kind == MilestoneKind::Longest).unwrap();
    assert_eq!((l.title.as_str(), l.value.as_str(), l.route.as_deref()), ("Longest run", "9.8 km", Some("workout/2")));
}

#[test]
fn goals_over_time_show_each_weeks_rings_and_the_first_fully_met_week() {
    let f = fx("goalweeks");
    f.core.local_action(&LocalAction::SetGoals { targets: GoalTargets { strength_sessions: 1, ..Default::default() } }, at(11, 3)).unwrap();
    act(&f.store, 1, "2025-11-04", "strength_training", 40., None);
    let m = view(&f, Period::Month, "2025-11-18", at(11, 18));
    let starts: Vec<&str> = m.goal_weeks.iter().map(|w| w.start.as_str()).collect();
    assert_eq!(starts, ["2025-11-03", "2025-11-10", "2025-11-17"], "weeks before goals have none");
    assert_eq!((m.goal_weeks[0].met, m.goal_weeks[0].goals, m.goal_weeks[0].rings[0].fill), (1, 1, 1.));
    assert_eq!((m.goal_weeks[1].met, m.goal_weeks[1].rings[0].fill), (0, 0.));
    let first = m.milestones.iter().find(|x| x.kind == MilestoneKind::GoalWeek).unwrap();
    assert_eq!((first.date.as_str(), first.route.as_deref()), ("2025-11-09", Some("review/2025-11-03")));
}

#[test]
fn historical_progress_is_read_only_and_uses_only_evidence_available_by_its_end() {
    let f=fx("history-pure");
    sets(&f.store,1,"2025-10-20",90_000.,5);
    sets(&f.store,2,"2025-11-18",110_000.,5);
    let past=view(&f,Period::Month,"2025-10-01",at(11,18));
    assert_eq!(past.today,"2025-11-18","navigation bounds stay independent of the selected historical period");
    assert!(past.milestones.iter().all(|m|m.date.as_str() <= "2025-10-31" && !m.new));
    assert!(f.store.setting("local:milestones").is_none());
    view(&f,Period::Month,"2025-11-18",at(11,18));
    let saved=f.store.setting("local:milestones");
    view(&f,Period::Month,"2025-10-01",at(11,18));
    assert_eq!(f.store.setting("local:milestones"),saved);
}

#[test]
fn changing_display_units_never_awards_a_lift_again() {
    let f=fx("units");
    sets(&f.store,1,"2025-11-18",100_000.,5);
    view(&f,Period::Month,"2025-11-18",at(11,18));
    f.core.local_action(&LocalAction::MilestonesSeen,at(11,18)).unwrap();
    let saved=f.store.setting("local:milestones");
    f.store.put("profile_settings","me","now",&json!({"measurementSystem":"statute"})).unwrap();
    let changed=view(&f,Period::Month,"2025-11-18",at(11,18));
    assert!(changed.milestones.iter().all(|m|!m.new && !m.corrected));
    assert_eq!(f.store.setting("local:milestones"),saved);
}

#[test]
fn vanished_records_remain_visible_as_quiet_corrections() {
    let f=fx("withdrawn");
    sets(&f.store,1,"2025-11-18",100_000.,5);
    view(&f,Period::Month,"2025-11-18",at(11,18));
    f.store.put("exercise_sets","1","now",&json!({"exerciseSets":[]})).unwrap();
    let changed=view(&f,Period::Month,"2025-11-18",at(11,18));
    assert!(!changed.milestones.is_empty());
    assert!(changed.milestones.iter().all(|m|m.corrected && !m.new && m.route.is_none() && m.value=="Evidence no longer available"));
}

#[test]
fn a_revised_week_and_observations_alone_do_not_earn_an_award() {
    let f=fx("adjusted-award");
    f.core.local_action(&LocalAction::SetGoals{targets:GoalTargets{strength_sessions:3,..Default::default()}},at(11,3)).unwrap();
    act(&f.store,1,"2025-11-04","strength_training",40.,None);
    f.core.local_action(&LocalAction::SetGoals{targets:GoalTargets{strength_sessions:1,..Default::default()}},at(11,6)).unwrap();
    let report=view(&f,Period::Month,"2025-11-18",at(11,18));
    assert!(report.milestones.iter().all(|m|m.kind!=MilestoneKind::GoalWeek));
}

#[test]
fn invalid_or_future_period_anchors_are_refused() {
    let f=fx("dates");
    for date in ["0000-01-01","1989-12-31","+262142-12-31","2025-11-19"] {
        assert!(f.core.progress(Period::Year,date,at(11,18)).is_err(),"{date}");
    }
}

#[test]
fn milestone_rows_export_and_follow_the_account_boundary() {
    use std::io::Read;
    let f=fx("archive");
    let profile=json!({"profileId":101,"displayName":"synthetic"});
    f.store.put("profile","me","now",&profile).unwrap();
    f.store.set_setting("account_session",&json!({"identity":"101","active":true}).to_string()).unwrap();
    sets(&f.store,1,"2025-11-18",100_000.,5);
    view(&f,Period::Month,"2025-11-18",at(11,18));
    let archive=f.dir.join("export.zip");
    f.core.export(&archive,"test").unwrap();
    let mut zip=zip::ZipArchive::new(std::fs::File::open(archive).unwrap()).unwrap();
    let mut body=String::new();
    zip.by_name("settings.json").unwrap().read_to_string(&mut body).unwrap();
    let settings:serde_json::Value=serde_json::from_str(&body).unwrap();
    assert_eq!(settings["local:milestones"]["schema"],1);
    assert!(f.store.bind_account("101",&profile,"later").unwrap());
    assert!(f.store.setting("local:milestones").is_some());
    assert!(f.store.bind_account("202",&json!({"profileId":202}),"later").unwrap());
    assert!(f.store.setting("local:milestones").is_none());
    f.core.forget_account().unwrap();
    assert!(f.core.local_records().milestones.items.is_empty());
}
