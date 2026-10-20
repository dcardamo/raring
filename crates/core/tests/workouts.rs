use raring_core::{screens, store::Store, view::Icon};
use serde_json::{json, Value};

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
    assert_eq!(view.summary.zone_seconds, vec![Some(30.), Some(20.), None, Some(10.), Some(5.)]);
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
    assert_eq!(view.summary.zone_seconds, vec![Some(3.), Some(4.)]);
    assert_eq!(view.heart_rate[1].t_s, 30.);
}

#[test]
fn missing_null_and_invalid_zone_durations_are_not_measured_zero() {
    let store = Store::open(":memory:").unwrap();
    store.put("activity", "4", "now", &json!({"activityId":4,"startTimeLocal":"2025-02-03T07:00:00"})).unwrap();
    for (zones, expected) in [
        (json!([{"zoneNumber":1,"secsInZone":0},{"zoneNumber":2,"secsInZone":null},{"zoneNumber":4,"secsInZone":-1},{"zoneNumber":5,"secsInZone":30}]),
         vec![Some(0.),None,None,None,Some(30.)]),
        (json!([{"secsInZone":0},{"secsInZone":null},{"secsInZone":-1},{"secsInZone":30}]),
         vec![Some(0.),None,None,Some(30.)]),
        (json!([]), vec![]),
        (Value::Null, vec![]),
    ] {
        store.put("zones", "4", "now", &zones).unwrap();
        assert_eq!(screens::workout(&store, "4").unwrap().summary.zone_seconds, expected);
    }
}

#[test]
fn streams_align_on_one_axis_and_only_recorded_fields_appear() {
    use raring_core::view::{EffectKind, StreamKind};
    let store = Store::open(":memory:").unwrap();
    store.put("activity", "77", "now", &json!({"activityId": 77, "activityName": "Hill Run", "activityTypeDTO": {"typeKey": "running"},
        "summaryDTO": {"startTimeLocal": "2025-03-12T07:00:00", "duration": 120, "distance": 400, "trainingEffect": 3.4,
            "anaerobicTrainingEffect": 0.8, "moderateIntensityMinutes": 2, "vigorousIntensityMinutes": 0, "differenceBodyBattery": -3,
            "normalizedPower": null}})).unwrap();
    // Timestamp, HR, speed (m/s), elevation, double cadence; the middle sample lacks speed.
    let row = |t: i64, hr: f64, speed: Option<f64>, elev: f64| json!({"metrics": [t, hr, speed, elev, 160.0]});
    store.put("details", "77", "now", &json!({
        // Garmin's factors are not scaling to apply: values are already in the unit.
        "metricDescriptors": [{"key": "directTimestamp", "metricsIndex": 0}, {"key": "directHeartRate", "metricsIndex": 1},
            {"key": "directSpeed", "metricsIndex": 2, "unit": {"key": "mps", "factor": 0.1}},
            {"key": "directElevation", "metricsIndex": 3, "unit": {"key": "meter", "factor": 100.0}},
            {"key": "directDoubleCadence", "metricsIndex": 4, "unit": {"key": "stepsPerMinute", "factor": 1.0}}],
        "activityDetailMetrics": [row(1_741_762_800_000, 120., Some(3.), 100.), row(1_741_762_860_000, 140., None, 104.), row(1_741_762_920_000, 150., Some(3.5), 108.)]})).unwrap();
    store.put("max_metrics", "2025-03-11", "now", &json!([{"generic": {"vo2MaxValue": 50}}])).unwrap();
    store.put("max_metrics", "2025-03-12", "now", &json!([{"generic": {"vo2MaxValue": 51}}])).unwrap();
    let w = screens::workout(&store, "77").unwrap();
    let along = w.along.as_ref().unwrap();
    assert_eq!(along.t_s, vec![0., 60., 120.], "the HR's time axis");
    let kinds: Vec<_> = along.series.iter().map(|s| s.kind).collect();
    assert_eq!(kinds, vec![StreamKind::Speed, StreamKind::Elevation, StreamKind::Cadence], "only recorded streams");
    assert_eq!(along.series[0].values, vec![Some(10.8), None, Some(12.6)], "km/h, and a gap stays a gap");
    assert_eq!(along.series[2].unit, "spm");
    let more: Vec<_> = w.more.iter().map(|m| (m.key.as_str(), m.value)).collect();
    assert_eq!(more, vec![("moderateIntensityMinutes", 2.), ("vigorousIntensityMinutes", 0.), ("differenceBodyBattery", -3.)], "null fields don't appear");
    let e: Vec<_> = w.effects.iter().map(|e| (e.kind, e.band.as_str(), e.line.as_str())).collect();
    assert_eq!(e, vec![(EffectKind::Aerobic, "Improving", "builds your aerobic base"), (EffectKind::Anaerobic, "No benefit", "little sprint or hard work")]);
    assert_eq!(w.vo2max_new, Some(51.), "a new estimate that day");
    store.put("max_metrics", "2025-03-11", "now", &json!([{"generic": {"vo2MaxValue": 51}}])).unwrap();
    assert_eq!(screens::workout(&store, "77").unwrap().vo2max_new, None, "an unchanged estimate isn't new");
}

#[test]
fn effect_bands_follow_garmins_scale() {
    use raring_core::view::EffectKind::*;
    for (kind, v, band) in [(Aerobic, 0.9, "No benefit"), (Aerobic, 1.0, "Minor benefit"), (Anaerobic, 2.0, "Maintaining"), (Aerobic, 3.99, "Improving"),
        (Anaerobic, 4.0, "Highly improving"), (Aerobic, 5.0, "Overreaching")] {
        assert_eq!(screens::effect_band(kind, v).0, band, "{v}");
    }
}

#[test]
fn a_stream_needs_its_expected_unit_key() {
    for (key, unit, ok) in [
        ("directSpeed", Some("mps"), true), ("directSpeed", Some("kph"), false), ("directSpeed", None, false),
        ("directElevation", Some("meter"), true), ("directElevation", Some("foot"), false),
        ("directDoubleCadence", Some("stepsPerMinute"), true), ("directBikeCadence", Some("rpm"), true), ("directBikeCadence", Some("stepsPerMinute"), false),
        ("directPower", Some("watt"), true), ("directRespirationRate", Some("breathsPerMinute"), true),
        ("directAvailableStamina", Some("dimensionless"), true), ("directPerformanceCondition", Some("dimensionless"), true),
        ("directAirTemperature", Some("celcius"), true), ("directAirTemperature", Some("fahrenheit"), false),
        ("directRunCadence", Some("stepsPerMinute"), false),
    ] {
        assert_eq!(screens::stream_unit_ok(key, unit), ok, "{key} in {unit:?}");
    }
}

#[test]
fn unexpected_units_drop_the_stream_and_factors_are_never_applied() {
    use raring_core::view::StreamKind;
    let store = Store::open(":memory:").unwrap();
    store.put("activity", "78", "now", &json!({"activityId": 78, "activityName": "Ride", "activityTypeDTO": {"typeKey": "cycling"},
        "summaryDTO": {"startTimeLocal": "2025-03-12T07:00:00", "duration": 60, "averageTemperature": 27.0}})).unwrap();
    let row = |t: i64| json!({"metrics": [t, 130.0, 5.0, 210.0, 31.0, 88.0]});
    store.put("details", "78", "now", &json!({
        "metricDescriptors": [{"key": "directTimestamp", "metricsIndex": 0}, {"key": "directHeartRate", "metricsIndex": 1},
            {"key": "directSpeed", "metricsIndex": 2, "unit": {"key": "mps", "factor": 10.0}},
            {"key": "directPower", "metricsIndex": 3, "unit": {"key": "kilowatt", "factor": 1.0}},
            {"key": "directAirTemperature", "metricsIndex": 4, "unit": {"key": "celcius", "factor": 0.1}},
            {"key": "directBikeCadence", "metricsIndex": 5}],
        "activityDetailMetrics": [row(1_741_762_800_000), row(1_741_762_860_000)]})).unwrap();
    let w = screens::workout(&store, "78").unwrap();
    let series = &w.along.as_ref().unwrap().series;
    let kinds: Vec<_> = series.iter().map(|s| s.kind).collect();
    assert_eq!(kinds, vec![StreamKind::Speed, StreamKind::Temperature], "power in an unexpected unit and cadence without one are dropped");
    assert_eq!(series[0].values, vec![Some(18.), Some(18.)], "5 m/s is 18 km/h whatever the factor");
    assert_eq!(series[1].values, vec![Some(31.), Some(31.)], "the watch's °C as recorded");
    assert!(w.more.iter().any(|m| m.key == "averageTemperature" && m.value == 27.), "the summary's watch temperature");
}
