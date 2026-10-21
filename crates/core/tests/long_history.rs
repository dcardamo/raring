//! Long-history reads (U61): three years of synthetic Garmin days and about a
//! thousand activities, timed through the same core reads the app makes. The
//! budgets catch gross regressions, not small ones; timings print with
//! `--nocapture`. Real phones and real accounts are separate evidence.
use chrono::{Local, TimeZone};
use raring_core::{demo, view::*, Raring};
use std::time::{Duration, Instant};

/// Budgets are for the app's release build, about 5–10 times the measured
/// reads; debug builds (the usual test run) get four times as long.
const SCALE: u32 = if cfg!(debug_assertions) { 4 } else { 1 };

fn timed<T>(name: &str, budget_ms: u64, times: &mut Vec<(String, u128)>, f: impl FnOnce() -> T) -> T {
    let start = Instant::now();
    let out = f();
    let took = start.elapsed();
    times.push((name.to_owned(), took.as_millis()));
    let budget = Duration::from_millis(budget_ms) * SCALE;
    assert!(took <= budget, "{name} took {took:?}, over its {budget:?} budget");
    out
}

#[test]
fn three_years_of_history_reads_within_budget() {
    let dir = std::env::temp_dir().join(format!("raring-long-history-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("raring.db");
    let now = Local.with_ymd_and_hms(2026, 10, 14, 12, 0, 0).unwrap();
    let store = raring_core::store::Store::open(&path).unwrap();
    let activities = demo::seed_long_history(&store, now.date_naive(), 3).unwrap();
    assert!(activities >= 900, "{activities} activities");
    drop(store);
    let core = Raring::open(&path).unwrap();
    let t = GoalTargets { strength_sessions: 2, zone2_min: 120, high_intensity_min: 0, intensity_min: 0, intensity_garmin_goal: true,
        sleep_nights: 0, sleep_time: None, sleep_kind: SleepTimingKind::AsleepBy, sleep_score: 0, rest_days: vec![] };
    let set_at = Local.with_ymd_and_hms(2024, 1, 8, 9, 0, 0).unwrap();
    core.local_action(&LocalAction::SetGoals { targets: t }, set_at).unwrap();
    let day = now.date_naive().to_string();
    let mut times = vec![];
    let today = timed("Today", 100, &mut times, || core.today_live_at(&day, now));
    assert!(today.day.is_some());
    timed("Plan board", 100, &mut times, || core.plan_board(&day, now).unwrap());
    timed("Sleep", 100, &mut times, || core.sleep_live(now));
    timed("Training (with lanes)", 250, &mut times, || core.training_live_at(&day, now));
    timed("Health", 100, &mut times, || core.health_live_at(&day, now));
    for period in [Period::Week, Period::Month, Period::Quarter, Period::Year] {
        let p = timed(&format!("Progress {period:?}"), 500, &mut times, || core.progress(period, &day, now).unwrap());
        assert!(p.totals.activities > 0);
    }
    let domains = timed("Progress domains", 300, &mut times, || core.progress_domains(&DomainQuery::default(), now).unwrap());
    assert_eq!(domains.weeks.len(), 12);
    let longest = DomainQuery { weeks: 52, months: 60, ..Default::default() };
    let domains = timed("Progress domains (a year of weeks, five years of months)", 800, &mut times, || core.progress_domains(&longest, now).unwrap());
    assert_eq!((domains.weeks.len(), domains.months.len()), (52, 60));
    let all = timed("Milestone archive", 300, &mut times, || core.milestone_archive(now).unwrap());
    assert!(!all.is_empty(), "three years of rising lifts award milestones");
    timed("Strength", 150, &mut times, || core.strength(&day));
    timed("Lift series (Max)", 100, &mut times, || core.lift_series("BENCH_PRESS/BARBELL_BENCH_PRESS", TrendRange::All, now));
    timed("Muscles (Max)", 100, &mut times, || core.muscles(TrendRange::All, now));
    for (metric, range) in [(TrendMetric::Hrv, TrendRange::Days30), (TrendMetric::RestingHr, TrendRange::Year), (TrendMetric::SleepScore, TrendRange::All)] {
        let q = TrendQuery { metric, range, end: None, overlay: None, previous: true, start: None, comparison_start: None };
        timed(&format!("Trend {metric:?} {range:?}"), 100, &mut times, || core.trend(&q, &day).unwrap());
    }
    println!("Long-history reads ({activities} activities, 3 years, {} build):", if cfg!(debug_assertions) { "debug" } else { "release" });
    for (name, ms) in &times {
        println!("  {name}: {ms} ms");
    }
    drop(core);
    let _ = std::fs::remove_dir_all(dir);
}
