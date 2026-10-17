use raring_core::{
    screens::vitals,
    store::Store,
    view::{Tone, VitalKind, VitalRangeSource},
};
use serde_json::{json, Value};

fn put(store: &Store, date: &str, value: &Value) {
    store.put("health_status", date, "now", value).unwrap();
}

#[test]
fn garmin_status_and_bounds_override_local_percentiles() {
    let store = Store::open(":memory:").unwrap();
    for i in 0..30 {
        let date = (chrono::NaiveDate::from_ymd_opt(2025, 10, 16).unwrap()
            - chrono::Duration::days(i))
        .to_string();
        store
            .put("summary", &date, "now", &json!({"restingHeartRate": 40}))
            .unwrap();
    }
    let data: Value = serde_json::from_str(include_str!("fixtures/health_status.json")).unwrap();
    put(&store, "2025-10-16", &data);
    let card = vitals(&store, "2025-10-16");
    assert_eq!(card.off_count, 2);
    let hr = card
        .readings
        .iter()
        .find(|r| r.kind == VitalKind::RestingHr)
        .unwrap();
    assert_eq!(hr.value, Some(58.)); // Overnight average, not the resting HR in summary.
    assert_eq!(hr.normal, Some([52., 62.]));
    assert_eq!(hr.tone, Tone::Good);
    assert_eq!(hr.range_source, VitalRangeSource::Garmin);
    assert_eq!(hr.trend[0], None); // Never join resting-HR history to overnight HR.
    let spo2 = card
        .readings
        .iter()
        .find(|r| r.kind == VitalKind::Spo2)
        .unwrap();
    assert_eq!(spo2.tone, Tone::Poor);
    assert_eq!(spo2.status.as_deref(), Some("BELOW_RANGE"));
    let temp = card
        .readings
        .iter()
        .find(|r| r.kind == VitalKind::SkinTemp)
        .unwrap();
    assert_eq!(temp.value, Some(0.8)); // Celsius, not the duplicate Fahrenheit metric.
    assert_eq!(temp.tone, Tone::Poor);
}

#[test]
fn onboarding_and_missing_days_are_not_all_clear() {
    let store = Store::open(":memory:").unwrap();
    let data: Value =
        serde_json::from_str(include_str!("fixtures/health_status_onboarding.json")).unwrap();
    put(&store, "2025-10-02", &data);
    let card = vitals(&store, "2025-10-02");
    assert_eq!(card.off_count, 0);
    assert!(!card.baseline_ready);
    assert!(card
        .readings
        .iter()
        .all(|r| r.tone == Tone::Neutral && r.normal.is_none()));
    put(&store, "2025-10-03", &Value::Null);
    let missing = vitals(&store, "2025-10-03");
    assert_eq!(missing.off_count, 0);
    assert!(missing.readings.iter().all(|r| r.value.is_none()));
}

#[test]
fn fallback_only_where_garmin_has_no_metric() {
    let store = Store::open(":memory:").unwrap();
    let end = chrono::NaiveDate::from_ymd_opt(2025, 10, 16).unwrap();
    for i in 0..30 {
        let day = (end - chrono::Duration::days(i)).to_string();
        store
            .put("summary", &day, "now", &json!({"restingHeartRate": 48}))
            .unwrap();
    }
    put(
        &store,
        "2025-10-16",
        &json!({"calendarDate":"2025-10-16", "metrics":[]}),
    );
    let hr = vitals(&store, "2025-10-16").readings.remove(0);
    assert_eq!(hr.range_source, VitalRangeSource::Raring);
    assert_eq!(hr.normal, Some([48., 48.]));
    assert_eq!(hr.tone, Tone::Good);
    let mut partial: Value =
        serde_json::from_str(include_str!("fixtures/health_status.json")).unwrap();
    partial["metrics"][0]["status"] = json!("SOMETHING_NEW");
    put(&store, "2025-10-16", &partial);
    let hrv = &vitals(&store, "2025-10-16").readings[1];
    assert_eq!(hrv.tone, Tone::Neutral);
}
