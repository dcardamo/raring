use pepometer_core::{screens, store::Store};
use serde_json::{json, Value};

fn today(readiness: Value) -> pepometer_core::view::TodayView {
    let store = Store::open(":memory:").unwrap();
    store
        .put("readiness", "2025-01-02", "now", &readiness)
        .unwrap();
    screens::today(&store, "2025-01-02")
}

#[test]
fn latest_readiness_drives_score_factors_advice_and_trend_in_any_array_order() {
    let morning = json!({
        "inputContext":"AFTER_WAKEUP_RESET", "timestamp":"2025-01-02T07:00:00.0",
        "score":82, "level":"HIGH", "recoveryTimeFactorPercent":90
    });
    let after_workout = json!({
        "inputContext":"AFTER_POST_EXERCISE_RESET", "timestamp":"2025-01-02T13:00:00.0",
        "score":31, "level":"LOW", "recoveryTimeFactorPercent":20,
        "feedbackShort":"LET_YOUR_BODY_RECOVER"
    });
    let realtime = json!({
        "inputContext":"UPDATE_REALTIME_VARIABLES", "timestamp":"2025-01-02T18:00:00.0",
        "score":42, "level":"LOW", "recoveryTimeFactorPercent":35,
        "feedbackShort":"LET_YOUR_BODY_RECOVER"
    });
    for snapshots in [
        json!([morning, after_workout, realtime]),
        json!([realtime, morning, after_workout]),
        json!([after_workout, realtime, morning]),
    ] {
        let view = today(snapshots);
        assert_eq!(view.readiness.unwrap().value, 42.);
        assert_eq!(view.readiness_trend[29], Some(42.));
        assert_eq!(view.factors[0].percent, Some(35.));
        assert_eq!(view.headline, "Take it easy");
        assert_eq!(view.call_note.as_deref(), Some("Let your body recover"));
    }
}

#[test]
fn historical_readiness_and_weekly_checkins_keep_the_morning_baseline() {
    let store = Store::open(":memory:").unwrap();
    store.put("readiness", "2025-01-02", "now", &json!([
        {"inputContext":"AFTER_POST_EXERCISE_RESET", "timestamp":"2025-01-02T18:00:00", "score":31},
        {"inputContext":"AFTER_WAKEUP_RESET", "timestamp":"2025-01-02T07:00:00", "score":82}
    ])).unwrap();
    assert_eq!(
        screens::today(&store, "2025-01-02")
            .readiness
            .unwrap()
            .value,
        31.
    );
    assert_eq!(
        screens::today(&store, "2025-01-03").readiness_trend[28],
        Some(82.)
    );
    assert_eq!(
        screens::checkin(&store, "2025-01-06").cards[0].value,
        Some(82.)
    );
}

#[test]
fn readiness_handles_local_times_offsets_and_incomplete_snapshots() {
    let cases = [
        json!([
            {"timestampLocal":"2025-01-02T06:00:00", "score":82},
            {"timestampLocal":"2025-01-02T18:00:00.123", "score":42},
            {"timestampLocal":"2025-01-02T19:00:00", "score":null}
        ]),
        json!([
            {"timestamp":"2025-01-02T16:00:00Z", "score":82},
            {"timestamp":"2025-01-02T12:00:00-05:00", "score":42},
            {"timestamp":"invalid", "score":99}
        ]),
        json!([
            {"timestamp":"2025-01-01T23:59:00Z", "score":82},
            {"timestamp":"2025-01-02T00:01:00Z", "score":42}
        ]),
        json!([
            {"score":82},
            {"inputContext":"AFTER_WAKEUP_RESET", "score":42}
        ]),
        json!({"score":42}),
    ];
    for snapshots in cases {
        assert_eq!(today(snapshots).readiness.unwrap().value, 42.);
    }
    assert!(today(json!([])).readiness.is_none());
    assert!(today(json!([{"timestamp":"2025-01-02T13:00:00"}]))
        .readiness
        .is_none());
}

#[test]
fn body_battery_uses_latest_valid_sample_not_array_order_or_daily_high() {
    let store = Store::open(":memory:").unwrap();
    store
        .put(
            "summary",
            "2025-01-02",
            "now",
            &json!({"bodyBatteryHighestValue":95}),
        )
        .unwrap();
    assert_eq!(screens::today(&store, "2025-01-02").body_battery.now, None);
    store
        .put(
            "battery",
            "2025-01-02",
            "now",
            &json!([{
                "charged":52, "drained":70,
                "bodyBatteryValuesArray":[
                    [1735833600000_i64,24], [1735790400000_i64,95],
                    [1735840800000_i64,null], [1735840900000_i64,-1], [1735841000000_i64,101]
                ]
            }]),
        )
        .unwrap();
    let battery = screens::today(&store, "2025-01-02").body_battery;
    assert_eq!(battery.now, Some(24.));
    assert_eq!(battery.charged, Some(52.));
    assert_eq!(battery.drained, Some(70.));
    store
        .put(
            "summary",
            "2025-01-02",
            "now",
            &json!({
                "bodyBatteryMostRecentValue":18, "bodyBatteryHighestValue":95
            }),
        )
        .unwrap();
    assert_eq!(
        screens::today(&store, "2025-01-02").body_battery.now,
        Some(18.)
    );
}

#[test]
fn body_battery_accepts_zero_and_skips_malformed_samples() {
    let store = Store::open(":memory:").unwrap();
    store
        .put(
            "battery",
            "2025-01-02",
            "now",
            &json!({
                "bodyBatteryValuesArray":[null,[],[1],["bad",99],[2,5],[3,0]]
            }),
        )
        .unwrap();
    assert_eq!(
        screens::today(&store, "2025-01-02").body_battery.now,
        Some(0.)
    );
}
