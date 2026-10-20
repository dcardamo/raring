use raring_core::{screens, store::Store, view::Stage};
use serde_json::{json, Value};

fn put(store: &Store, day: &str, stages: Value) {
    store.put("sleep", day, "now", &json!({"dailySleepDTO": stages})).unwrap();
}

#[test]
fn stage_means_use_the_inclusive_calendar_window_and_per_stage_denominators() {
    let store = Store::open(":memory:").unwrap();
    put(&store, "2025-01-01", json!({"deepSleepSeconds":600,"remSleepSeconds":0}));
    put(&store, "2025-01-15", json!({"deepSleepSeconds":null,"remSleepSeconds":1200}));
    put(&store, "2025-01-20", json!({"deepSleepSeconds":-1,"remSleepSeconds":"unknown"}));
    put(&store, "2025-01-30", json!({"deepSleepSeconds":1800,"remSleepSeconds":2400,"awakeSleepSeconds":0}));
    // Neither the day before the inclusive window nor a future night contributes.
    for day in ["2024-12-31", "2025-01-31"] {
        put(&store, day, json!({"deepSleepSeconds":99999,"remSleepSeconds":99999}));
    }
    let view = screens::sleep_view(&store, "2025-01-30");
    for (stage, minutes, mean, count, delta) in [
        (Stage::Deep, 30., 20., 2, 10.),
        (Stage::Rem, 40., 20., 3, 20.),
        (Stage::Awake, 0., 0., 1, 0.),
    ] {
        let row = view.totals.iter().find(|r| r.stage == stage).unwrap();
        assert_eq!(row.minutes, minutes);
        assert_eq!(row.comparison_min, Some(mean));
        assert_eq!(row.comparison_nights, count);
        assert_eq!(row.usual_delta_min, Some(delta));
    }
    assert_eq!(view.totals.len(), 3, "a missing stage is not a zero row");
}

#[test]
fn absent_and_invalid_selected_stages_have_no_comparison_or_zero_fill() {
    let store = Store::open(":memory:").unwrap();
    assert!(screens::sleep_view(&store, "2025-01-30").totals.is_empty());
    put(&store, "2025-01-29", json!({"deepSleepSeconds":600,"lightSleepSeconds":1200}));
    put(&store, "2025-01-30", json!({"deepSleepSeconds":null,"lightSleepSeconds":-60,"remSleepSeconds":"NaN"}));
    assert!(screens::sleep_view(&store, "2025-01-30").totals.is_empty());
}

#[test]
fn stage_comparison_preserves_fractional_minutes_without_rounding_the_mean() {
    let store = Store::open(":memory:").unwrap();
    put(&store, "2025-01-29", json!({"deepSleepSeconds":61}));
    put(&store, "2025-01-30", json!({"deepSleepSeconds":62}));
    let view = screens::sleep_view(&store, "2025-01-30");
    assert_eq!(view.totals[0].comparison_nights, 2);
    assert_eq!(view.totals[0].comparison_min, Some(61.5 / 60.));
    assert_eq!(view.totals[0].usual_delta_min, Some(0.5 / 60.));
}
