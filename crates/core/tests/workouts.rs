use raring_core::{screens, store::Store, view::Icon};
use serde_json::json;

#[test]
fn nested_detail_indexes_activity_and_reads_garmin_summary_and_zones() {
    let store = Store::open(":memory:").unwrap();
    store
        .put(
            "activity",
            "42",
            "now",
            &json!({
                "activityId": 42, "activityName": "Quick Ride",
                "activityTypeDTO": {"typeKey": "cycling"},
                "summaryDTO": {
                    "startTimeLocal": "2025-02-03T07:00:00", "duration": 80,
                    "distance": 1200, "activityTrainingLoad": 16,
                    "trainingEffect": 2.4, "anaerobicTrainingEffect": 0.3,
                    "aerobicTrainingEffectMessage": "MAINTAINING", "trainingEffectLabel": "BASE",
                    "averageHR": 125, "maxHR": 144, "calories": 55, "elevationGain": 12,
                    "averageSpeed": 5, "averagePower": 180, "averageBikeCadence": 80
                }
            }),
        )
        .unwrap();
    store
        .put(
            "zones",
            "42",
            "now",
            &json!([
                {"zoneNumber": 4, "secsInZone": 10, "zoneLowBoundary": 155},
                {"zoneNumber": 2, "secsInZone": 20, "zoneLowBoundary": 105},
                {"zoneNumber": 1, "secsInZone": 30, "zoneLowBoundary": 90},
                {"zoneNumber": 5, "secsInZone": 5, "zoneLowBoundary": 175}
            ]),
        )
        .unwrap();
    assert_eq!(store.activity_ids(Some("2025-02-03"), 10), vec!["42"]);
    assert_eq!(
        store.newest_activity().as_deref(),
        Some("2025-02-03T07:00:00")
    );
    let view = screens::workout(&store, "42").unwrap();
    assert_eq!(view.summary.icon, Icon::Bike);
    assert_eq!(view.summary.start_local, "2025-02-03T07:00:00");
    assert_eq!(view.summary.duration_s, 80.);
    assert_eq!(view.summary.distance_km, Some(1.2));
    assert_eq!(view.summary.load, Some(16.));
    assert_eq!(view.summary.aerobic_te, Some(2.4));
    assert_eq!(view.summary.aerobic_label.as_deref(), Some("Maintaining"));
    assert_eq!(view.summary.benefit.as_deref(), Some("Base"));
    assert_eq!(view.summary.zone_seconds, vec![30., 20., 0., 10., 5.]);
    assert_eq!(view.zone_floors, vec![105., 155., 175.]);
    for (key, value) in [
        ("distance", 1.2),
        ("duration", 80.),
        ("averageHR", 125.),
        ("maxHR", 144.),
        ("calories", 55.),
        ("elevationGain", 12.),
        ("speed", 18.),
        ("averagePower", 180.),
        ("averageBikeCadence", 80.),
    ] {
        assert_eq!(
            view.stats.iter().find(|s| s.key == key).unwrap().value,
            value
        );
    }
    assert!(
        !view.stats.iter().any(|s| s.key == "pace"),
        "cycling shows Garmin speed, not running pace"
    );
    // No optional detail metrics or splits endpoint is required to render a workout.
    assert!(view.heart_rate.is_empty());
    assert!(view.splits.is_empty());
}

#[test]
fn partial_detail_nulls_fall_back_to_flat_list_and_sparse_stats_do_not_panic() {
    let store = Store::open(":memory:").unwrap();
    store.put("activities", "2025-02-03", "now", &json!([{
        "activityId": 77, "activityName": "List Run", "activityType": {"typeKey": "running"},
        "startTimeLocal": "2025-02-03T08:00:00", "duration": 40,
        "distance": 1000, "aerobicTrainingEffect": 1.8, "averageHR": 120
    }])).unwrap();
    store
        .put(
            "activity",
            "77",
            "now",
            &json!({
                "activityId": 77, "activityName": "Detail Run", "activityTypeDTO": null,
                "summaryDTO": {"startTimeLocal": null, "duration": null, "distance": null,
                               "trainingEffect": null, "averageHR": null}
            }),
        )
        .unwrap();
    let view = screens::workout(&store, "77").unwrap();
    assert_eq!(view.summary.name, "Detail Run");
    assert_eq!(view.summary.icon, Icon::Run);
    assert_eq!(view.summary.start_local, "2025-02-03T08:00:00");
    assert_eq!(view.summary.duration_s, 40.);
    assert_eq!(view.summary.aerobic_te, Some(1.8));
    assert_eq!(
        view.stats
            .iter()
            .find(|s| s.key == "averageHR")
            .unwrap()
            .value,
        120.
    );
    assert!(view.stats.iter().any(|s| s.key == "pace"));
    // Only distance and duration, without any HR or calories, must also be safe.
    let sparse = Store::open(":memory:").unwrap();
    sparse
        .put(
            "activity",
            "1",
            "now",
            &json!({
                "activityId": 1, "summaryDTO": {"startTimeLocal": "2025-02-03T09:00:00",
                    "duration": 45, "distance": 1000}
            }),
        )
        .unwrap();
    let stats = screens::workout(&sparse, "1").unwrap().stats;
    assert_eq!(
        stats.iter().map(|s| s.key.as_str()).collect::<Vec<_>>(),
        vec!["distance", "duration", "pace"]
    );
}

#[test]
fn descriptor_indices_and_first_valid_epoch_millisecond_timestamp_are_used() {
    let store = Store::open(":memory:").unwrap();
    store
        .put(
            "activity",
            "9",
            "now",
            &json!({"activityId": 9,
                "summaryDTO": {"startTimeLocal": "2025-02-03T08:00:00", "duration": 70}
            }),
        )
        .unwrap();
    store
        .put(
            "details",
            "9",
            "now",
            &json!({
                "metricDescriptors": [
                    {"key": "directHeartRate", "metricsIndex": 2},
                    {"key": "directLongitude", "metricsIndex": 3},
                    {"key": "directTimestamp", "metricsIndex": 0, "unit": {"key": "gmt"}},
                    {"key": "directLatitude", "metricsIndex": 1}
                ],
                "activityDetailMetrics": [
                    {"metrics": [null, 45.0, 90, -70.0]},
                    {"metrics": [1738570000000_i64, 45.1, 100, -70.1]},
                    {"metrics": [1738570030000_i64, 45.2, 110, -70.2]},
                    {"metrics": [1738570060000_i64, 45.3, 120, -70.3]}
                ]
            }),
        )
        .unwrap();
    let view = screens::workout(&store, "9").unwrap();
    assert_eq!(
        view.heart_rate.iter().map(|h| h.t_s).collect::<Vec<_>>(),
        vec![0., 30., 60.]
    );
    assert_eq!(view.route.len(), 4);
    assert_eq!(view.route[1], [45.1, -70.1]);
}

#[test]
fn mixed_list_and_detail_timestamps_sort_chronologically() {
    let store = Store::open(":memory:").unwrap();
    store
        .put(
            "activities",
            "0",
            "now",
            &json!([
                {"activityId": 11, "startTimeLocal": "2025-02-03 18:00:00"},
                {"activityId": 12, "startTimeLocal": "2025-02-03 07:00:00"}
            ]),
        )
        .unwrap();
    store
        .put(
            "activity",
            "12",
            "now",
            &json!({
                "activityId": 12, "summaryDTO": {"startTimeLocal": "2025-02-03T07:00:00"}
            }),
        )
        .unwrap();
    assert_eq!(store.activity_ids(None, 10), vec!["11", "12"]);
    assert_eq!(
        store.newest_activity().as_deref(),
        Some("2025-02-03 18:00:00")
    );
}

#[test]
fn unnumbered_zones_and_positional_second_timestamps_remain_supported() {
    let store = Store::open(":memory:").unwrap();
    store
        .put(
            "activity",
            "4",
            "now",
            &json!({"activityId": 4, "startTimeLocal": "2025-02-03T07:00:00"}),
        )
        .unwrap();
    store
        .put(
            "zones",
            "4",
            "now",
            &json!([{"secsInZone": 3}, {"secsInZone": 4}]),
        )
        .unwrap();
    store
        .put(
            "details",
            "4",
            "now",
            &json!({
                "metricDescriptors": [{"key": "directTimestamp"}, {"key": "directHeartRate"}],
                "activityDetailMetrics": [{"metrics": [0, 100]}, {"metrics": [30, 110]}]
            }),
        )
        .unwrap();
    let view = screens::workout(&store, "4").unwrap();
    assert_eq!(view.summary.zone_seconds, vec![3., 4.]);
    assert_eq!(view.heart_rate[1].t_s, 30.);
}
