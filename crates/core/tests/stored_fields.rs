//! Stored Garmin fields newly exposed without new requests.
use raring_core::{screens, store::Store};
use serde_json::json;

#[test]
fn intraday_body_battery_follows_garmins_columns_and_keeps_gaps() {
    let store = Store::open(":memory:").unwrap();
    let start = 1_741_579_200_000_i64; // 2025-03-10T04:00:00Z
    store.put("stress", "2025-03-10", "now", &json!({
        "startTimestampGMT": "2025-03-10T04:00:00.0", "startTimestampLocal": "2025-03-10T00:00:00.0",
        "bodyBatteryValueDescriptorsDTOList": [
            {"bodyBatteryValueDescriptorIndex": 0, "bodyBatteryValueDescriptorKey": "timestamp"},
            {"bodyBatteryValueDescriptorIndex": 1, "bodyBatteryValueDescriptorKey": "bodyBatteryStatus"},
            {"bodyBatteryValueDescriptorIndex": 2, "bodyBatteryValueDescriptorKey": "bodyBatteryLevel"},
            {"bodyBatteryValueDescriptorIndex": 3, "bodyBatteryValueDescriptorKey": "bodyBatteryVersion"}],
        "bodyBatteryValuesArray": [
            [start, "MEASURED", 20, 3.0], [start + 180_000, "MODELED", 21, 3.0],
            [start + 360_000, null, null, 3.0], [start + 900_000, "MEASURED", 25, 3.0]],
    })).unwrap();
    let today = screens::today(&store, "2025-03-10");
    assert_eq!(today.body_battery.timeline, [Some(20.), Some(21.), None, None, None, Some(25.)]);
}

#[test]
fn activity_cards_carry_average_hr_and_garmins_intensity_minutes() {
    let store = Store::open(":memory:").unwrap();
    store.put("activity", "5", "now", &json!({"activityId": 5, "activityTypeDTO": {"typeKey": "running"},
        "summaryDTO": {"startTimeLocal": "2025-03-10 07:00:00", "duration": 1800, "averageHR": 148,
            "moderateIntensityMinutes": 12, "vigorousIntensityMinutes": 9}})).unwrap();
    store.put("activity", "6", "now", &json!({"activityId": 6, "activityTypeDTO": {"typeKey": "walking"},
        "summaryDTO": {"startTimeLocal": "2025-03-10 09:00:00", "duration": 1800, "moderateIntensityMinutes": 12}})).unwrap();
    let w = screens::workout(&store, "5").unwrap().summary;
    assert_eq!((w.avg_hr, w.intensity_min), (Some(148.), Some(30.)));
    let walk = screens::workout(&store, "6").unwrap().summary;
    assert_eq!((walk.avg_hr, walk.intensity_min), (None, None), "half a figure is not Garmin's figure");
}
