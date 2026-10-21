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
    // The same need Sleep reads: Garmin's seconds field when there is no adjusted need.
    f.store.put("sleep", "2025-11-18", "now", &json!({"dailySleepDTO": {"sleepTimeSeconds": 300 * 60, "sleepNeed": {"totalSleepNeedInSeconds": 450 * 60}}})).unwrap();
    let s = view(&f, Period::Week, "2025-11-18", at(11, 18)).sleep.unwrap();
    assert_eq!((s.nights, s.below_hour), (2, 2));
}

fn sets(store: &Store, id: u64, date: &str, weight_g: f64, reps: u32) {
    act(store, id, date, "strength_training", 40., None);
    store
        .put("exercise_sets", &id.to_string(), "now", &json!({"exerciseSets": [
            {"setType": "ACTIVE", "repetitionCount": reps, "weight": weight_g, "exercises": [{"category": "SQUAT", "name": "BARBELL_BACK_SQUAT", "probability": 0.9}]},
            {"setType": "ACTIVE", "repetitionCount": reps, "weight": weight_g, "exercises": [{"category": "SQUAT", "name": "BARBELL_BACK_SQUAT", "probability": 0.9}]}]}))
        .unwrap();
}
fn bodyweight(store: &Store, id: u64, date: &str, reps: u32) {
    act(store, id, date, "strength_training", 20., None);
    store
        .put("exercise_sets", &id.to_string(), "now", &json!({"exerciseSets": [
            {"setType": "ACTIVE", "repetitionCount": reps, "weight": 0, "exercises": [{"category": "CORE", "name": "BICYCLE_CRUNCH", "probability": 0.9}]},
            {"setType": "ACTIVE", "repetitionCount": reps - 2, "weight": 0, "exercises": [{"category": "CORE", "name": "BICYCLE_CRUNCH", "probability": 0.9}]}]}))
        .unwrap();
}

#[test]
fn bests_are_awarded_once_and_a_garmin_edit_corrects_without_a_new_award() {
    let f = fx("milestones");
    sets(&f.store, 1, "2025-11-03", 90_000., 5);
    sets(&f.store, 2, "2025-11-10", 100_000., 5);
    let m = view(&f, Period::Month, "2025-11-18", at(11, 18));
    let squat = |m: &ProgressView| m.milestones.iter().find(|x| x.kind == MilestoneKind::Best).cloned().unwrap();
    assert_eq!(m.milestones.iter().filter(|x| x.kind == MilestoneKind::Best).count(), 1, "one row per set, not one per rep count");
    let best = squat(&m);
    assert_eq!((best.value.as_str(), best.date.as_str(), best.new, best.corrected), ("100 kg × 5 · heaviest for 1–5 reps", "2025-11-10", false, false));
    assert_eq!(best.title, "Barbell Back Squat");
    f.core.local_action(&LocalAction::MilestonesSeen, at(11, 18)).unwrap();
    let again = view(&f, Period::Month, "2025-11-18", at(11, 18));
    assert!(again.milestones.iter().all(|x| !x.new), "seen once, never replayed");
    // A Garmin edit removes the heavier set: the record moves back, corrected once.
    sets(&f.store, 2, "2025-11-10", 80_000., 5);
    let edited = view(&f, Period::Month, "2025-11-18", at(11, 18));
    let best = squat(&edited);
    assert_eq!((best.value.as_str(), best.date.as_str(), best.new, best.corrected), ("90 kg × 5 · heaviest for 1–5 reps", "2025-11-03", false, true));
    // A genuinely heavier set later is a new award.
    sets(&f.store, 3, "2025-11-18", 105_000., 5);
    let best = squat(&view(&f, Period::Month, "2025-11-18", at(11, 18)));
    assert_eq!((best.value.as_str(), best.new, best.corrected), ("105 kg × 5 · heaviest for 1–5 reps", true, false));
    // A lighter set with more reps holds the higher rep counts on its own row.
    sets(&f.store, 4, "2025-11-18", 60_000., 10);
    let rows: Vec<String> = view(&f, Period::Month, "2025-11-18", at(11, 18)).milestones.into_iter().filter(|x| x.kind == MilestoneKind::Best).map(|x| x.value).collect();
    assert_eq!(rows, ["105 kg × 5 · heaviest for 1–5 reps", "60 kg × 10 · heaviest for 8–10 reps"]);
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
    let marks: Vec<(bool, Option<DayMark>)> = m.goal_weeks.iter().map(|w| (w.current, w.mark)).collect();
    assert_eq!(marks, [(false, None), (false, None), (true, None)], "the week in progress is current, not unknown");
    let first = m.milestones.iter().find(|x| x.kind == MilestoneKind::GoalWeek).unwrap();
    assert_eq!((first.date.as_str(), first.route.as_deref()), ("2025-11-09", Some("review/2025-11-03")));
}

#[test]
fn a_week_that_did_not_count_says_why_on_its_rings() {
    let f = fx("goalweek-marks");
    f.core.local_action(&LocalAction::SetGoals { targets: GoalTargets { strength_sessions: 1, ..Default::default() } }, at(11, 3)).unwrap();
    f.core.local_action(&LocalAction::Pause { reason: PauseReason::Ill, review: None }, at(11, 11)).unwrap();
    f.core.local_action(&LocalAction::Resume, at(11, 13)).unwrap();
    f.store.set_cursor("listed:lo", "2025-11-05").unwrap();
    let m = view(&f, Period::Month, "2025-11-18", at(11, 18));
    let marks: Vec<(bool, Option<DayMark>)> = m.goal_weeks.iter().map(|w| (w.counted, w.mark)).collect();
    assert_eq!(marks, [(false, Some(DayMark::Unknown)), (false, Some(DayMark::Ill)), (false, None)]);
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
    assert!(changed.milestones.iter().all(|m|m.corrected && !m.new && m.route.is_none() && m.value.starts_with("Evidence no longer available")));
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

#[test]
fn ring_days_cover_whole_plan_weeks_of_week_and_month_periods() {
    let f = fx("ringdays");
    let m = view(&f, Period::Month, "2025-11-18", at(11, 18));
    assert_eq!(m.ring_days.len() % 7, 0, "whole weeks");
    assert!(m.ring_days.first().unwrap().date.as_str() <= "2025-11-01" && m.ring_days.last().unwrap().date.as_str() >= "2025-11-30");
    assert!(m.ring_days.iter().any(|d| d.today && d.date == "2025-11-18"));
    assert_eq!(view(&f, Period::Week, "2025-11-18", at(11, 18)).ring_days.len(), 7);
    assert!(view(&f, Period::Quarter, "2025-11-18", at(11, 18)).ring_days.is_empty());
}

#[test]
fn domains_report_weeks_and_months_without_inventing_zeroes() {
    let f = fx("domains");
    sets(&f.store, 1, "2025-11-03", 90_000., 5);
    sets(&f.store, 2, "2025-11-10", 100_000., 3);
    act(&f.store, 3, "2025-11-11", "running", 30., Some(5000.));
    f.store.put("zones", "3", "now", &json!([{"zoneNumber": 2, "secsInZone": 1200}, {"zoneNumber": 4, "secsInZone": 300}])).unwrap();
    f.store.put("max_metrics", "2025-11-12", "now", &json!([{"generic": {"vo2MaxValue": 48}}])).unwrap();
    let d = f.core.progress_domains(&DomainQuery::default(), at(11, 18)).unwrap();
    let starts: Vec<&str> = d.weeks.iter().map(|w| w.start.as_str()).collect();
    assert_eq!(starts.first(), Some(&"2025-09-01"), "weeks start with covered history");
    assert_eq!(starts.last(), Some(&"2025-11-17"));
    assert!(d.weeks.last().unwrap().current);
    let wk = d.weeks.iter().find(|w| w.start == "2025-11-10").unwrap();
    // The strength activities have no zones, so zone minutes are unknown that week.
    assert_eq!((wk.strength, wk.zone2_min, wk.vo2max, wk.mark), (Some(1.), None, Some(48.), None));
    let earlier = d.weeks.iter().find(|w| w.start == "2025-10-06").unwrap();
    assert_eq!((earlier.strength, earlier.zone2_min), (Some(0.), Some(0.)), "a covered empty week is zero");
    assert_eq!(d.months.len(), 12);
    let nov = d.months.last().unwrap();
    assert_eq!((nov.start.as_str(), nov.minutes, nov.vo2max), ("2025-11-01", Some(110.), Some(48.)));
    assert_eq!(d.months[0].minutes, None, "before history");
}

#[test]
fn domains_show_the_chosen_weeks_and_months() {
    let f = fx("domain-ranges");
    f.store.put("max_metrics", "2025-06-12", "now", &json!([{"generic": {"vo2MaxValue": 46}}])).unwrap();
    // Six months of weeks ending with an earlier week, two years of months ending in June.
    let q = DomainQuery { weeks: 26, weeks_end: Some("2025-11-05".into()), months: 24, months_end: Some("2025-06-20".into()) };
    let d = f.core.progress_domains(&q, at(11, 18)).unwrap();
    assert_eq!((d.weeks_from.as_str(), d.weeks_to.as_str()), ("2025-05-12", "2025-11-09"));
    assert_eq!(d.weeks.last().map(|w| (w.start.as_str(), w.current)), Some(("2025-11-03", false)));
    assert_eq!(d.weeks.first().map(|w| w.start.as_str()), Some("2025-09-01"), "weeks before downloaded history are left out");
    assert_eq!((d.months_from.as_str(), d.months_to.as_str()), ("2023-07-01", "2025-06-30"));
    assert_eq!(d.months.len(), 24);
    assert_eq!(d.months.last().map(|m| (m.start.as_str(), m.vo2max)), Some(("2025-06-01", Some(46.))));
    assert!(d.months.iter().all(|m| m.minutes.is_none()), "no minutes before activity history");
    // The default is twelve of each, ending now.
    let now = f.core.progress_domains(&DomainQuery::default(), at(11, 18)).unwrap();
    assert_eq!((now.weeks_to.as_str(), now.months.len(), now.months_to.as_str()), ("2025-11-23", 12, "2025-11-30"));
    for end in ["2025-11-19", "1989-12-31", "2025-13-01"] {
        let q = DomainQuery { weeks_end: Some(end.into()), ..Default::default() };
        assert!(f.core.progress_domains(&q, at(11, 18)).is_err(), "{end}");
    }
}

#[test]
fn each_domain_week_carries_its_own_targets() {
    let f = fx("domain-goals");
    let targets = |strength| GoalTargets { strength_sessions: strength, intensity_garmin_goal: true, ..Default::default() };
    f.core.local_action(&LocalAction::SetGoals { targets: targets(1) }, at(11, 4)).unwrap();
    f.core.local_action(&LocalAction::SetGoals { targets: targets(2) }, at(11, 12)).unwrap();
    for d in 10..=18 {
        f.store.put("summary", &format!("2025-11-{d}"), "now", &json!({"moderateIntensityMinutes": 10, "vigorousIntensityMinutes": 5, "intensityMinutesGoal": 150})).unwrap();
    }
    let d = f.core.progress_domains(&DomainQuery::default(), at(11, 18)).unwrap();
    let goals = |start: &str| d.weeks.iter().find(|w| w.start == start).map(|w| (w.strength_goal, w.intensity_goal)).unwrap();
    assert_eq!(goals("2025-10-27"), (None, None), "no goal before goals began");
    assert_eq!(goals("2025-11-03").0, Some(1.));
    assert_eq!(goals("2025-11-10"), (Some(2.), Some(150.)), "a week ends on its latest target; intensity follows Garmin's goal");
    assert_eq!(d.weeks.last().unwrap().zone2_goal, None, "a goal that is off has no line");
}

#[test]
fn a_bodyweight_best_is_the_most_reps_and_corrects_like_a_weighted_best() {
    let f = fx("reps-milestones");
    bodyweight(&f.store, 1, "2025-11-03", 20);
    bodyweight(&f.store, 2, "2025-11-10", 30);
    let m = view(&f, Period::Month, "2025-11-18", at(11, 18));
    let best = m.milestones.iter().find(|x| x.id == "reps:CORE/BICYCLE_CRUNCH").unwrap();
    assert_eq!((best.kind, best.value.as_str(), best.date.as_str()), (MilestoneKind::Reps, "30 reps", "2025-11-10"));
    assert_eq!(best.title, "Bicycle Crunch · most reps in a set");
    assert_eq!(best.route.as_deref(), Some("workout/2"));
    // A later Garmin edit lowers the set: the same award moves back, corrected.
    bodyweight(&f.store, 2, "2025-11-10", 15);
    let edited = view(&f, Period::Month, "2025-11-18", at(11, 18));
    let best = edited.milestones.iter().find(|x| x.id == "reps:CORE/BICYCLE_CRUNCH").unwrap();
    assert_eq!((best.value.as_str(), best.date.as_str(), best.corrected), ("20 reps", "2025-11-03", true));
    assert_eq!(edited.milestones.iter().filter(|x| x.id == "reps:CORE/BICYCLE_CRUNCH").count(), 1, "one award per exercise");
}

#[test]
fn the_archive_keeps_every_milestone_dated_and_awards_nothing_twice() {
    let f = fx("archive-all");
    // Fourteen exercises, each done twice and heavier the second time: more
    // milestones than Progress lists.
    for i in 0..14u64 {
        for (n, date, kg) in [(0u64, "2025-09-10", 50_000.), (1, "2025-10-10", 55_000.)] {
            let id = 200 + i * 2 + n;
            act(&f.store, id, date, "strength_training", 40., None);
            f.store.put("exercise_sets", &id.to_string(), "now", &json!({"exerciseSets": [
                {"setType": "ACTIVE", "repetitionCount": 5, "weight": kg + 1000. * i as f64, "exercises": [{"category": "SQUAT", "name": format!("VARIANT_{i}"), "probability": 0.9}]}]})).unwrap();
        }
    }
    let m = view(&f, Period::Month, "2025-11-18", at(11, 18));
    assert_eq!(m.milestones.len(), 12.min(m.milestones_total as usize));
    assert!(m.milestones_total > 12, "{} milestones", m.milestones_total);
    let all = f.core.milestone_archive(at(11, 18)).unwrap();
    assert_eq!(all.len() as u32, m.milestones_total, "the archive has every milestone Progress counted");
    assert!(all.windows(2).all(|w| w[0].date >= w[1].date), "newest first");
    let again = f.core.milestone_archive(at(11, 18)).unwrap();
    assert_eq!(all.iter().map(|m| &m.id).collect::<Vec<_>>(), again.iter().map(|m| &m.id).collect::<Vec<_>>(), "reading again awards nothing new");
}

#[test]
fn a_detail_without_distance_takes_it_from_the_activity_list() {
    let f = fx("brief");
    f.store.put("activity", "77", "now", &json!({"activityId": 77, "activityName": "Run", "activityTypeDTO": {"typeKey": "running"},
        "summaryDTO": {"startTimeLocal": "2025-11-17 07:00:00", "duration": 1800}})).unwrap();
    f.store.put("activities", "0", "now", &json!([{"activityId": 77, "distance": 5000.0, "duration": 1800.0, "activityType": {"typeKey": "running"}}])).unwrap();
    let w = view(&f, Period::Week, "2025-11-18", at(11, 18));
    assert_eq!((w.totals.activities, w.totals.distance_km), (1, 5.), "the list page fills the missing distance");
}
