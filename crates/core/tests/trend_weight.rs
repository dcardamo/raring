use chrono::{Duration, NaiveDate};
use raring_core::{store::Store, view::*, weight};
use serde_json::json;
fn day(s: &str) -> NaiveDate {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
}
fn store() -> Store {
    let s = Store::open(":memory:").unwrap();
    s.put(
        "profile_settings",
        "me",
        "synthetic",
        &json!({"measurementSystem":"metric"}),
    )
    .unwrap();
    s
}
fn put(s: &Store, d: &str, kg: f64) {
    s.put("weight",d,"synthetic",&json!({"weight":kg*1000.,"timestampGMT":day(d).and_hms_opt(7,0,0).unwrap().and_utc().timestamp_millis()})).unwrap()
}
fn view(s: &Store, range: TrendRange, d: &str) -> WeightView {
    weight::view(s, range, day(d), day(d)).unwrap()
}
fn near(a: f64, b: f64) {
    assert!((a - b).abs() < 1e-8, "{a} != {b}")
}
#[test]
fn daily_smoothing_holds_on_gaps_and_stops_at_the_last_weigh_in() {
    let s = store();
    put(&s, "2025-03-01", 70.);
    put(&s, "2025-03-02", 71.);
    put(&s, "2025-03-05", 69.);
    let v = view(&s, TrendRange::Days7, "2025-03-10");
    near(v.points[1].trend.unwrap(), 70.1);
    near(v.points[2].trend.unwrap(), 70.1);
    near(v.points[3].trend.unwrap(), 70.1);
    near(
        v.trend.unwrap(),
        70.1 + (1. - 0.9_f64.powi(3)) * (69. - 70.1),
    );
    assert_eq!(v.to.as_deref(), Some("2025-03-05"));
    assert_eq!(v.points.len(), 5);
    assert!(
        v.points[4].break_scale,
        "three-day scale gap breaks the scale line"
    );
    assert_eq!(v.freshness.age_days, Some(5));
    assert!(!v.freshness.stale);
    assert!(v
        .changes
        .iter()
        .find(|c| c.days == 7)
        .unwrap()
        .change
        .is_none());
}
#[test]
fn scale_and_trend_are_distinct_and_small_moves_are_steady() {
    let s = store();
    put(&s, "2025-03-01", 70.);
    put(&s, "2025-03-02", 70.7);
    let v = view(&s, TrendRange::Days30, "2025-03-02");
    near(v.scale.unwrap(), 70.7);
    near(v.trend.unwrap(), 70.07);
    let one = &v.changes[0];
    near(one.change.unwrap(), 0.07);
    assert_eq!(one.direction, Some(WeightDirection::Steady));
    assert_eq!(one.per_week, None);
    near(weight::steady_threshold(7), 0.2);
    near(weight::steady_threshold(30), 0.05 * 30. / 7.);
    near(weight::steady_threshold(90), 0.05 * 90. / 7.);
}
#[test]
fn steady_decision_is_unrounded_and_independent_of_display_units() {
    let s = store();
    put(&s, "2025-03-01", 70.);
    put(&s, "2025-03-02", 73.);
    let kg = view(&s, TrendRange::All, "2025-03-02");
    s.put(
        "profile_settings",
        "me",
        "synthetic",
        &json!({"measurementSystem":"statute_us"}),
    )
    .unwrap();
    let lb = view(&s, TrendRange::All, "2025-03-02");
    assert_eq!(lb.unit, Some(WeightUnit::Lb));
    near(lb.trend.unwrap(), kg.trend.unwrap() / 0.45359237);
    assert_eq!(lb.changes[0].direction, Some(WeightDirection::Gaining));
    assert_eq!(lb.changes[0].direction, kg.changes[0].direction);
    near(
        lb.changes[0].change.unwrap(),
        kg.changes[0].change.unwrap() / 0.45359237,
    );
}
#[test]
fn freshness_uses_reading_dates_and_strict_two_or_seven_day_thresholds() {
    for threshold in [2, 7] {
        let d = day("2025-03-01");
        assert!(!weight::freshness(Some(d), d + Duration::days(threshold.into()), threshold).stale);
        assert!(
            weight::freshness(
                Some(d),
                d + Duration::days(i64::from(threshold) + 1),
                threshold
            )
            .stale
        );
        assert_eq!(weight::freshness(None, d, threshold).age_days, None);
        assert_eq!(
            weight::freshness(Some(d + Duration::days(1)), d, threshold).recorded_on,
            None
        );
    }
    let s = store();
    put(&s, "2025-03-01", 70.);
    let v = view(&s, TrendRange::All, "2025-03-09");
    assert!(v.freshness.stale);
    assert_eq!(v.freshness.recorded_on.as_deref(), Some("2025-03-01"));
}
#[test]
fn newer_same_day_latest_wins_but_invalid_measurements_do_not_replace_valid_ones() {
    let s = store();
    put(&s, "2025-03-01", 70.);
    let date = day("2025-03-01")
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc()
        .timestamp_millis();
    let latest = |w: f64, t: i64| {
        s.put(
            "weight_latest",
            "me",
            "synthetic",
            &json!({"date":date,"timestampGMT":date+t*3600000,"weight":w}),
        )
        .unwrap()
    };
    latest(72000., 8);
    assert_eq!(view(&s, TrendRange::All, "2025-03-01").scale, Some(72.));
    latest(72000., 6);
    assert_eq!(view(&s, TrendRange::All, "2025-03-01").scale, Some(70.));
    latest(0., 9);
    assert_eq!(view(&s, TrendRange::All, "2025-03-01").scale, Some(70.));
}
#[test]
fn missing_units_empty_history_and_future_readings_are_not_invented() {
    let s = Store::open(":memory:").unwrap();
    put(&s, "2025-03-01", 70.);
    let v = view(&s, TrendRange::Days30, "2025-03-01");
    assert_eq!(v.unit, None);
    assert_eq!(v.scale, None);
    assert!(v.points.is_empty());
    let s = store();
    put(&s, "2025-03-20", 70.);
    put(&s, "2025-03-01", 0.);
    let v = view(&s, TrendRange::Days14, "2025-03-19");
    assert_eq!(v.freshness.recorded_on, None);
    assert_eq!(v.trend, None);
    assert!(weight::view(&s, TrendRange::Custom, day("2025-03-01"), day("2025-03-01")).is_err());
    assert!(weight::view(&s, TrendRange::Days7, day("2025-03-02"), day("2025-03-01")).is_err());
}
#[test]
fn long_range_averages_and_changes_use_daily_values_not_plot_buckets() {
    let s = store();
    let start = day("2023-01-01");
    for i in 0..500 {
        if i % 5 != 3 {
            put(
                &s,
                &(start + Duration::days(i)).to_string(),
                70. + (i as f64 / 30.).sin(),
            );
        }
    }
    let end = start + Duration::days(499);
    let all = weight::view(&s, TrendRange::All, end, end).unwrap();
    let year = weight::view(&s, TrendRange::Year, end, end).unwrap();
    assert!(all.points.len() < 40);
    assert!(year.points.len() < 60);
    let mut sum = 0.;
    let mut count = 0.;
    for d in (end - Duration::days(364)).iter_days().take(365) {
        let v = weight::view(&s, TrendRange::Days7, d, d).unwrap();
        sum += v.trend.unwrap();
        count += 1.;
    }
    near(year.average.unwrap(), sum / count);
    let change = year.changes.iter().find(|c| c.days == 7).unwrap();
    near(change.per_week.unwrap(), change.change.unwrap());
    assert_eq!(
        all.changes, year.changes,
        "change rows are anchored at last weigh-in, independent of chart range"
    );
}
#[test]
fn correction_recomputes_without_writing_records_or_settings() {
    let s = store();
    put(&s, "2025-03-01", 70.);
    put(&s, "2025-03-02", 72.);
    let before = view(&s, TrendRange::All, "2025-03-02").trend.unwrap();
    put(&s, "2025-03-02", 69.);
    let after = view(&s, TrendRange::All, "2025-03-02").trend.unwrap();
    assert!(after < before);
    assert!(s.setting("local:milestones").is_none());
    assert!(s.setting("local:schema").is_none());
}
