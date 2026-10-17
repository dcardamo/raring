use raring_core::{screens, store::Store};
use serde_json::json;

#[test]
fn weight_uses_garmin_units_and_keeps_missing_weeks_missing() {
    let s = Store::open(":memory:").unwrap();
    s.put(
        "profile_settings",
        "me",
        "now",
        &json!({"measurementSystem":"statute_us"}),
    )
    .unwrap();
    s.put(
        "weight",
        "2025-04-10",
        "now",
        &json!({"weight":68000.0,"bmi":24.1,"bodyFat":null}),
    )
    .unwrap();
    s.put(
        "weight",
        "2025-06-30",
        "now",
        &json!({"weight":70000.0,"bmi":24.8,"bodyFat":null}),
    )
    .unwrap();
    let health = screens::health(&s, "2025-06-30");
    let weight = health.body_weight.unwrap();
    assert_eq!(weight.value, 154.3);
    assert_eq!(weight.unit, raring_core::view::WeightUnit::Lb);
    assert_eq!(weight.measured_on, "2025-06-30");
    assert_eq!(weight.trend.iter().filter(|n| n.is_some()).count(), 2);
    assert!(health
        .drivers
        .iter()
        .any(|d| d.key == "bmi" && d.value == Some(24.8) && d.target.is_none()));
    assert!(!health.drivers.iter().any(|d| d.key == "bodyFat"));
    assert!(health.lever.is_none());
    assert!(screens::health(&s, "2025-04-01").body_weight.is_none());
    let old = screens::health(&s, "2025-08-30");
    assert_eq!(old.body_weight.unwrap().measured_on, "2025-06-30");
    assert!(!old.drivers.iter().any(|d| d.key == "bmi"));
}

#[test]
fn recent_weigh_in_fills_missing_or_null_fitness_age_values_without_inventing_targets() {
    for components in [json!({}), json!({"bmi":{"value":null},"bodyFat":{"value":null}})] {
        let s = Store::open(":memory:").unwrap();
        s.put("profile_settings", "me", "now", &json!({"measurementSystem":"metric"})).unwrap();
        s.put("weight", "2025-06-29", "now", &json!({"weight":71000.,"bmi":24.5,"bodyFat":21.0})).unwrap();
        s.put("fitness_age", "2025-06-30", "now", &json!({"components":components})).unwrap();
        let view = screens::health(&s, "2025-06-30");
        for (key, expected) in [("bmi", 24.5), ("bodyFat", 21.0)] {
            let driver = view.drivers.iter().find(|d| d.key == key).unwrap();
            assert_eq!(driver.value, Some(expected), "{key}");
            assert_eq!(driver.target, None);
        }
        assert!(view.lever.is_none());
    }
}

#[test]
fn garmin_fitness_age_value_and_target_take_precedence_over_weigh_in() {
    let s = Store::open(":memory:").unwrap();
    s.put("profile_settings", "me", "now", &json!({"measurementSystem":"metric"})).unwrap();
    s.put("weight", "2025-06-29", "now", &json!({"weight":71000.,"bmi":24.5,"bodyFat":21.0})).unwrap();
    s.put("fitness_age", "2025-06-30", "now", &json!({"components":{"bmi":{"value":26.0,"targetValue":25.0,"potentialAge":38.0},"bodyFat":{"value":null,"targetValue":20.0}}})).unwrap();
    let view = screens::health(&s, "2025-06-30");
    let bmi = view.drivers.iter().find(|d| d.key == "bmi").unwrap();
    assert_eq!((bmi.value, bmi.target), (Some(26.0), Some(25.0)));
    let fat = view.drivers.iter().find(|d| d.key == "bodyFat").unwrap();
    assert_eq!((fat.value, fat.target), (Some(21.0), Some(20.0)));
    assert!(view.lever.as_ref().is_some_and(|lever| lever.key == "bodyFat" && lever.age_if_met.is_none()));
}

#[test]
fn latest_weight_is_visible_before_range_backfill_without_inventing_fitness_age_targets() {
    let s = Store::open(":memory:").unwrap();
    s.put("weight_latest", "me", "now", &json!({"date":1751241600000_i64,"timestampGMT":1751241600000_i64,"weight":72000.0,"bmi":null,"bodyFat":20.5})).unwrap();
    assert!(screens::health(&s, "2025-06-30").body_weight.is_none());
    s.put("profile_settings", "me", "now", &json!({"measurementSystem":"metric"})).unwrap();
    let view = screens::health(&s, "2025-06-30");
    let weight = view.body_weight.unwrap();
    assert_eq!(weight.value, 72.0);
    assert_eq!(weight.unit, raring_core::view::WeightUnit::Kg);
    assert_eq!(weight.body_fat, Some(20.5));
    assert!(view
        .drivers
        .iter()
        .any(|d| d.key == "bodyFat" && d.target.is_none()));
    assert!(view.lever.is_none());
}
