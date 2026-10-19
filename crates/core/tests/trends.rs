use chrono::{Duration, NaiveDate};
use raring_core::{store::Store, trends::trend as read_trend, view::*};
use serde_json::{json, Value};

fn trend(store: &Store, query: &TrendQuery, today: &str) -> TrendView {
    read_trend(store, query, today).unwrap()
}

const TODAY: &str = "2026-03-18"; // a Wednesday

fn day(n: i64) -> String {
    (NaiveDate::parse_from_str(TODAY, "%Y-%m-%d").unwrap() - Duration::days(n)).to_string()
}
fn put(store: &Store, endpoint: &str, key: &str, value: Value) {
    store
        .put(endpoint, key, "2026-03-18T08:00:00Z", &value)
        .unwrap();
}
fn query(metric: TrendMetric, range: TrendRange) -> TrendQuery {
    TrendQuery {
        metric,
        range,
        end: None,
        overlay: None,
        previous: false,
        start: None,
        comparison_start: None,
    }
}
fn values(view: &TrendView) -> Vec<Option<f64>> {
    view.series.points.iter().map(|p| p.value).collect()
}

#[test]
fn daily_ranges_read_recent_and_history_rows_and_leave_gaps() {
    let store = Store::open(":memory:").unwrap();
    // Recent per-day rows and an older history row, side by side.
    put(
        &store,
        "hrv",
        &day(0),
        json!({"hrvSummary": {"lastNightAvg": 41}}),
    );
    put(
        &store,
        "hrv",
        &day(2),
        json!({"hrvSummary": {"lastNightAvg": 47}}),
    );
    put(
        &store,
        "hrv_summary",
        &day(5),
        json!({"calendarDate": day(5), "lastNightAvg": 44}),
    );
    let view = trend(&store, &query(TrendMetric::Hrv, TrendRange::Days7), TODAY);
    assert_eq!(view.bucket, TrendBucket::Day);
    assert_eq!(view.from, day(6));
    assert_eq!(
        values(&view),
        [None, Some(44.), None, None, Some(47.), None, Some(41.)]
    );
    assert_eq!(view.series.days, 3);
    assert_eq!(view.series.average, Some(44.));
}

#[test]
fn health_status_wins_and_placeholders_are_missing() {
    let store = Store::open(":memory:").unwrap();
    put(
        &store,
        "sleep",
        &day(0),
        json!({"dailySleepDTO": {"avgHeartRate": 52}}),
    );
    put(
        &store,
        "health_status",
        &day(0),
        json!({"metrics": [{"type": "HR", "value": 51}]}),
    );
    // Garmin's -1 means too few readings, not a calm day.
    put(&store, "stress", &day(0), json!({"avgStressLevel": -1}));
    put(
        &store,
        "summary",
        &day(0),
        json!({"averageStressLevel": 33}),
    );
    put(&store, "stress", &day(1), json!({"avgStressLevel": -1}));
    let hr = trend(
        &store,
        &query(TrendMetric::SleepHr, TrendRange::Days7),
        TODAY,
    );
    assert_eq!(hr.series.points[6].value, Some(51.));
    let stress = trend(
        &store,
        &query(TrendMetric::Stress, TrendRange::Days7),
        TODAY,
    );
    assert_eq!(stress.series.points[6].value, Some(33.));
    assert_eq!(stress.series.points[5].value, None);
}

#[test]
fn a_year_averages_recorded_days_in_monday_weeks() {
    let store = Store::open(":memory:").unwrap();
    // This week so far: Monday and Wednesday recorded, Tuesday missing.
    put(&store, "summary", &day(2), json!({"restingHeartRate": 50}));
    put(&store, "summary", &day(0), json!({"restingHeartRate": 53}));
    // An older week from the history rows.
    put(
        &store,
        "daily_summary",
        &day(100),
        json!({"heartRate": {"restingValue": 60}}),
    );
    let view = trend(
        &store,
        &query(TrendMetric::RestingHr, TrendRange::Year),
        TODAY,
    );
    assert_eq!(view.bucket, TrendBucket::Week);
    assert_eq!(view.series.points.len(), 52);
    let last = view.series.points.last().unwrap();
    assert_eq!(
        (last.start.as_str(), last.end.as_str()),
        (day(2).as_str(), TODAY)
    );
    assert_eq!((last.value, last.days), (Some(51.5), 2));
    assert_eq!(values(&view).iter().flatten().count(), 2);
    assert_eq!(view.series.days, 3);
}

#[test]
fn all_starts_at_the_first_recorded_day_and_months_cover_long_history() {
    let store = Store::open(":memory:").unwrap();
    put(
        &store,
        "max_metrics",
        "2021-07-26",
        json!([{"generic": {"vo2MaxValue": 48}}]),
    );
    put(
        &store,
        "max_metrics",
        "2021-07-28",
        json!([{"generic": {"vo2MaxValue": 50}}]),
    );
    put(
        &store,
        "max_metrics",
        &day(0),
        json!([{"generic": {"vo2MaxValue": 52}}]),
    );
    let view = trend(&store, &query(TrendMetric::Vo2max, TrendRange::All), TODAY);
    assert_eq!(view.bucket, TrendBucket::Month);
    assert_eq!(view.from, "2021-07-01");
    let first = &view.series.points[0];
    assert_eq!((first.value, first.days), (Some(49.), 2));
    assert_eq!(view.series.points.last().unwrap().value, Some(52.));
    assert!(view.previous.is_none());
    assert!(view.available.contains(&TrendMetric::Vo2max));
}

#[test]
fn previous_period_and_overlay_share_the_points() {
    let store = Store::open(":memory:").unwrap();
    for n in 0..60 {
        let score = if n < 30 { 70 } else { 60 };
        put(
            &store,
            "readiness",
            &day(n),
            json!([{"score": score, "inputContext": "AFTER_WAKEUP_RESET", "acuteLoad": 300}]),
        );
    }
    put(
        &store,
        "sleep",
        &day(3),
        json!({"dailySleepDTO": {"sleepTimeSeconds": 27000}}),
    );
    let view = trend(
        &store,
        &TrendQuery {
            overlay: Some(TrendMetric::SleepTime),
            previous: true,
            ..query(TrendMetric::Readiness, TrendRange::Days30)
        },
        TODAY,
    );
    let previous = view.previous.as_ref().unwrap();
    assert_eq!(previous.points.len(), 30);
    assert_eq!(previous.points[29].start, day(30));
    assert_eq!(previous.average, Some(60.));
    assert_eq!(view.change, Some(10.));
    let overlay = view.overlay.as_ref().unwrap();
    assert_eq!(overlay.points.len(), 30);
    assert_eq!(overlay.points[26].value, Some(450.));
    assert_eq!(overlay.unit, "min");
}

#[test]
fn chosen_blocks_align_by_elapsed_day_and_weight_recorded_days() {
    let store = Store::open(":memory:").unwrap();
    for (date, value) in [("2024-02-28", 40), ("2024-02-29", 60), ("2025-01-03", 20)] {
        put(
            &store,
            "daily_summary",
            date,
            json!({"stress":{"avgLevel":value}}),
        );
    }
    let q = TrendQuery {
        start: Some("2024-02-28".into()),
        end: Some("2024-03-02".into()),
        comparison_start: Some("2025-01-01".into()),
        previous: true,
        ..query(TrendMetric::Stress, TrendRange::Custom)
    };
    let view = trend(&store, &q, TODAY);
    assert_eq!(view.series.points.len(), 4);
    assert_eq!(view.series.average, Some(50.));
    let before = view.previous.unwrap();
    assert_eq!(before.points[2].start, "2025-01-03");
    assert_eq!(before.points[2].value, Some(20.));
    assert_eq!(before.points[3].end, "2025-01-04");
    assert_eq!(view.change, Some(30.));
}

#[test]
fn rejects_invalid_custom_periods_and_dates() {
    let store = Store::open(":memory:").unwrap();
    let q = TrendQuery {
        start: Some("2026-03-20".into()),
        ..query(TrendMetric::Hrv, TrendRange::Custom)
    };
    assert!(read_trend(&store, &q, TODAY).is_err());
    let q = TrendQuery {
        end: Some("not-a-date".into()),
        ..query(TrendMetric::Hrv, TrendRange::All)
    };
    assert!(read_trend(&store, &q, TODAY).is_err());
}

#[test]
fn latest_load_and_morning_readiness_are_distinct_without_double_counting() {
    let store = Store::open(":memory:").unwrap();
    put(
        &store,
        "readiness",
        TODAY,
        json!([
            {"score":22,"acuteLoad":420,"timestamp":"2026-03-18T19:00:00Z","inputContext":"POST_EXERCISE"},
            {"score":78,"acuteLoad":210,"timestamp":"2026-03-18T07:00:00Z","inputContext":"AFTER_WAKEUP_RESET"}
        ]),
    );
    assert_eq!(
        trend(
            &store,
            &query(TrendMetric::Readiness, TrendRange::Days7),
            TODAY
        )
        .series
        .average,
        Some(78.)
    );
    assert_eq!(
        trend(
            &store,
            &query(TrendMetric::AcuteLoad, TrendRange::Days7),
            TODAY
        )
        .series
        .average,
        Some(420.)
    );
}

#[test]
fn source_precedence_counts_each_day_once_and_keeps_sleep_and_resting_hr_separate() {
    let store = Store::open(":memory:").unwrap();
    put(
        &store,
        "daily_summary",
        TODAY,
        json!({"bodyBattery":{"maxValue":91},"heartRate":{"restingValue":42}}),
    );
    put(
        &store,
        "summary",
        TODAY,
        json!({"bodyBatteryHighestValue":93,"restingHeartRate":43}),
    );
    put(
        &store,
        "sleep_summary",
        TODAY,
        json!({"avgHeartRate":60,"averageRespirationValue":15.2,"averageSpO2Value":95}),
    );
    let bb = trend(
        &store,
        &query(TrendMetric::BodyBattery, TrendRange::All),
        TODAY,
    );
    assert_eq!((bb.series.average, bb.series.days), (Some(93.), 1));
    assert_eq!(
        trend(
            &store,
            &query(TrendMetric::RestingHr, TrendRange::Days7),
            TODAY
        )
        .series
        .average,
        Some(43.)
    );
    assert_eq!(
        trend(
            &store,
            &query(TrendMetric::SleepHr, TrendRange::Days7),
            TODAY
        )
        .series
        .average,
        Some(60.)
    );
}

#[test]
fn decades_of_daily_rows_leave_the_core_as_months() {
    let store = Store::open(":memory:").unwrap();
    let from = NaiveDate::from_ymd_opt(2001, 1, 1).unwrap();
    for i in 0..9000 {
        put(
            &store,
            "daily_summary",
            &(from + Duration::days(i)).to_string(),
            json!({"stress":{"avgLevel":25}}),
        );
    }
    let view = trend(&store, &query(TrendMetric::Stress, TrendRange::All), TODAY);
    assert!(view.series.points.len() < 310);
    assert_eq!(view.series.days, 9000);
    assert_eq!(view.series.average, Some(25.));
}

#[test]
fn nightly_fallbacks_match_the_screens_without_health_status() {
    let store = Store::open(":memory:").unwrap();
    // A recent per-day row using Garmin's alternative field names.
    put(
        &store,
        "sleep",
        &day(0),
        json!({"dailySleepDTO": {"averageHeartRate": 51, "avgSpO2": 95, "skinTempDeviation": -0.3}}),
    );
    // An older history item, flat like the range response.
    put(
        &store,
        "sleep_summary",
        &day(40),
        json!({"calendarDate": day(40), "averageHeartRate": 55, "avgSpO2": 94, "skinTempDeviation": 0.2}),
    );
    let read = |metric| {
        trend(&store, &query(metric, TrendRange::Days90), TODAY)
            .series
            .points
    };
    for (metric, recent, older) in [
        (TrendMetric::SleepHr, 51., 55.),
        (TrendMetric::Spo2, 95., 94.),
        (TrendMetric::SkinTemp, -0.3, 0.2),
    ] {
        let points = read(metric);
        assert_eq!(points[89].value, Some(recent), "{metric:?} recent row");
        assert_eq!(points[49].value, Some(older), "{metric:?} history row");
    }
    // Sleep HR is the overnight average; resting HR has its own source.
    assert!(read(TrendMetric::RestingHr).iter().all(|p| p.value.is_none()));
    // Screens read the same fields as the trend.
    let sleep = raring_core::screens::sleep_view(&store, TODAY);
    let tile = |kind| sleep.overnight.iter().find(|t| t.kind == kind).and_then(|t| t.value);
    assert_eq!(tile(OvernightKind::HeartRate), Some(51.));
    assert_eq!(tile(OvernightKind::Spo2), Some(95.));
    let vitals = raring_core::screens::vitals(&store, TODAY);
    let skin = vitals.readings.iter().find(|r| r.kind == VitalKind::SkinTemp).unwrap();
    assert_eq!(skin.value, Some(-0.3));
}

#[test]
fn row_skin_temperature_wins_over_the_record() {
    let store = Store::open(":memory:").unwrap();
    put(
        &store,
        "sleep",
        TODAY,
        json!({"avgSkinTempDeviationC": 0.4, "dailySleepDTO": {"skinTempDeviation": -0.1}}),
    );
    let view = trend(&store, &query(TrendMetric::SkinTemp, TrendRange::Days7), TODAY);
    assert_eq!(view.series.points[6].value, Some(0.4));
}

#[test]
fn every_supplied_date_lies_between_the_first_day_and_today() {
    let store = Store::open(":memory:").unwrap();
    let err = |q: TrendQuery| read_trend(&store, &q, TODAY).unwrap_err();
    let tomorrow = "2026-03-19".to_string();
    // A future end.
    let e = err(TrendQuery {
        end: Some(tomorrow.clone()),
        ..query(TrendMetric::Hrv, TrendRange::Days30)
    });
    assert!(e.contains("no later than today"), "{e}");
    // A future comparison start.
    let e = err(TrendQuery {
        previous: true,
        comparison_start: Some(tomorrow),
        ..query(TrendMetric::Hrv, TrendRange::Days30)
    });
    assert!(e.contains("no later than today"), "{e}");
    // A comparison block that starts in the past but would end after today.
    let e = err(TrendQuery {
        previous: true,
        comparison_start: Some(day(10)),
        ..query(TrendMetric::Hrv, TrendRange::Days30)
    });
    assert!(e.contains("runs past today"), "{e}");
    // A custom start before the first possible day.
    let e = err(TrendQuery {
        start: Some("1989-12-31".into()),
        ..query(TrendMetric::Hrv, TrendRange::Custom)
    });
    assert!(e.contains("from 1990-01-01"), "{e}");
    // The bounds themselves are fine.
    let ok = trend(
        &store,
        &TrendQuery {
            start: Some("1990-01-01".into()),
            end: Some(TODAY.into()),
            ..query(TrendMetric::Hrv, TrendRange::Custom)
        },
        TODAY,
    );
    assert_eq!(ok.from, "1990-01-01");
    assert_eq!(ok.to, TODAY);
}

#[test]
fn freshness_flags_a_daily_metric_older_than_two_days_on_todays_chart() {
    let store = Store::open(":memory:").unwrap();
    let fresh = |store: &Store, metric, end: Option<String>| {
        let mut q = query(metric, TrendRange::Days30);
        q.end = end;
        trend(store, &q, TODAY).freshness
    };
    let none = fresh(&store, TrendMetric::Hrv, None).unwrap();
    assert_eq!((none.recorded_on, none.stale), (None, false), "no reading is unknown, not stale");
    put(&store, "hrv", &day(2), json!({"hrvSummary": {"lastNightAvg": 47}}));
    let f = fresh(&store, TrendMetric::Hrv, None).unwrap();
    assert_eq!((f.recorded_on.as_deref(), f.age_days, f.stale, f.threshold_days), (Some(day(2).as_str()), Some(2), false, 2));
    put(&store, "hrv", &day(2), json!({"hrvSummary": {"lastNightAvg": null}}));
    put(&store, "hrv", &day(3), json!({"hrvSummary": {"lastNightAvg": 45}}));
    assert!(fresh(&store, TrendMetric::Hrv, None).unwrap().stale, "three days old");
    assert!(fresh(&store, TrendMetric::Hrv, Some(day(10))).is_none(), "a past chart isn't judged");
    assert!(fresh(&store, TrendMetric::Vo2max, None).is_none(), "sparse estimates are dated, not flagged");
}
