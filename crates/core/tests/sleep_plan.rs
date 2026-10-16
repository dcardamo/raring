use chrono::{Duration, NaiveDate};
use pepometer_core::{notifications, screens, store::Store, view::VitalKind};
use serde_json::json;

const DATE: &str = "2025-01-16";

fn put_sleep(store: &Store, date: &str, wake: Option<&str>, need: serde_json::Value) {
    let mut night = json!({"sleepNeed": need});
    if let Some(wake) = wake {
        night["sleepEndTimestampLocal"] = json!(format!("{date}T{wake}:00"));
    }
    store
        .put("sleep", date, "now", &json!({"dailySleepDTO": night}))
        .unwrap();
}

#[test]
fn today_and_sleep_share_current_garmin_plan_across_midnight() {
    let store = Store::open(":memory:").unwrap();
    put_sleep(
        &store,
        "2025-01-15",
        Some("07:15"),
        json!({"totalSleepNeedSeconds": 30000}),
    );
    put_sleep(
        &store,
        DATE,
        Some("07:15"),
        json!({"totalSleepNeedInSeconds": 28800}),
    );
    let today = screens::today(&store, DATE);
    let sleep = screens::sleep_view(&store, DATE);
    assert_eq!(today.sleep_plan, sleep.sleep_plan);
    let plan = today.sleep_plan.unwrap();
    assert_eq!(plan.bedtime, "23:15");
    assert_eq!(plan.wake_time, "07:15");
    assert_eq!(plan.need_min, 480.);
    assert!(!plan.uses_average);

    // Today's alternate alias is preferred even when the primary alias is invalid.
    put_sleep(
        &store,
        DATE,
        Some("07:15"),
        json!({"totalSleepNeedInSeconds": "NaN", "totalSleepNeedSeconds": 27000}),
    );
    let plan = screens::sleep_view(&store, DATE).sleep_plan.unwrap();
    assert_eq!(plan.need_min, 450.);
    assert_eq!(plan.bedtime, "23:45");
    assert!(!plan.uses_average);
}

#[test]
fn bedtime_notification_uses_shared_plan() {
    let store = Store::open(":memory:").unwrap();
    put_sleep(
        &store,
        DATE,
        Some("07:15"),
        json!({"totalSleepNeedSeconds": 28800}),
    );
    let mut settings = store.settings();
    settings.notify_bedtime = true;
    store.set_settings(&settings).unwrap();
    let when = NaiveDate::parse_from_str(DATE, "%Y-%m-%d")
        .unwrap()
        .and_hms_opt(23, 0, 0)
        .unwrap()
        .and_local_timezone(chrono::Local)
        .single()
        .unwrap();
    let bedtime = notifications::notifications(&store, when)
        .into_iter()
        .find(|item| item.kind == "bedtime")
        .unwrap();
    assert_eq!(bedtime.body, "Aim to be asleep by 23:15.");
}

#[test]
fn historical_fallback_averages_both_aliases_and_ignores_invalid_needs() {
    let store = Store::open(":memory:").unwrap();
    put_sleep(
        &store,
        "2025-01-15",
        Some("07:30"),
        json!({"totalSleepNeedInSeconds": 25200}),
    );
    put_sleep(
        &store,
        "2025-01-14",
        Some("07:30"),
        json!({"totalSleepNeedSeconds": 32400}),
    );
    put_sleep(
        &store,
        "2025-01-13",
        Some("07:30"),
        json!({"totalSleepNeedInSeconds": -10}),
    );
    put_sleep(
        &store,
        "2025-01-12",
        Some("07:30"),
        json!({"totalSleepNeedSeconds": "NaN"}),
    );
    put_sleep(
        &store,
        DATE,
        Some("07:30"),
        json!({"totalSleepNeedInSeconds": 0}),
    );
    let plan = screens::today(&store, DATE).sleep_plan.unwrap();
    assert_eq!(plan, screens::sleep_view(&store, DATE).sleep_plan.unwrap());
    assert_eq!(plan.need_min, 480.);
    assert_eq!(plan.bedtime, "23:30");
    assert_eq!(plan.wake_time, "07:30");
    assert!(plan.uses_average);
}

#[test]
fn plans_require_both_garmin_need_and_observed_wake() {
    let store = Store::open(":memory:").unwrap();
    assert!(screens::today(&store, DATE).sleep_plan.is_none());
    put_sleep(&store, DATE, Some("07:00"), json!({}));
    assert!(screens::sleep_view(&store, DATE).sleep_plan.is_none());
    put_sleep(&store, DATE, None, json!({"totalSleepNeedSeconds": 28800}));
    assert!(screens::today(&store, DATE).sleep_plan.is_none());
    assert!(screens::sleep_view(&store, DATE).sleep_plan.is_none());
}

#[test]
fn health_date_and_extreme_vitals_stay_on_scale() {
    let store = Store::open(":memory:").unwrap();
    let date = NaiveDate::parse_from_str(DATE, "%Y-%m-%d").unwrap();
    for offset in 0..14 {
        let day = (date - Duration::days(offset)).to_string();
        store
            .put(
                "summary",
                &day,
                "now",
                &json!({"restingHeartRate": if offset == 0 { 200 } else { 60 }}),
            )
            .unwrap();
    }
    store.put("hrv", DATE, "now", &json!({"hrvSummary": {"lastNightAvg": 200, "baseline": {"balancedLow": 50, "balancedUpper": 70}}})).unwrap();
    let view = screens::health(&store, DATE);
    assert_eq!(view.date, DATE);
    for kind in [VitalKind::RestingHr, VitalKind::Hrv] {
        let reading = view
            .vitals
            .readings
            .iter()
            .find(|r| r.kind == kind)
            .unwrap();
        let [low, high] = reading.normal.unwrap();
        assert!(reading.scale[0] < low && reading.scale[1] > high);
        assert!(
            reading.scale[0] < reading.value.unwrap() && reading.scale[1] > reading.value.unwrap()
        );
    }
    // A normal range of zero width still receives padding when the value is identical.
    let constant = Store::open(":memory:").unwrap();
    for offset in 0..14 {
        let day = (date - Duration::days(offset)).to_string();
        constant
            .put("summary", &day, "now", &json!({"restingHeartRate": 60}))
            .unwrap();
    }
    let reading = screens::vitals(&constant, DATE)
        .readings
        .into_iter()
        .find(|r| r.kind == VitalKind::RestingHr)
        .unwrap();
    assert_eq!(reading.normal, Some([60., 60.]));
    assert!(reading.scale[0] < 60. && reading.scale[1] > 60.);
}
