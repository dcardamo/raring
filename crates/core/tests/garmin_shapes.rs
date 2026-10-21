use raring_core::{
    screens,
    store::Store,
    view::{OvernightKind, Stage, Tone, VitalKind},
};
use serde_json::{json, Value};

const DAY: &str = "2025-01-16";
fn put(store: &Store, kind: &str, day: &str, value: Value) {
    store.put(kind, day, "now", &value).unwrap();
}

#[test]
fn sleep_gmt_stages_and_root_samples_keep_local_clock_separate() {
    let store = Store::open(":memory:").unwrap();
    put(
        &store,
        "sleep",
        DAY,
        json!({
            "dailySleepDTO": {"sleepStartTimestampLocal":"2025-01-15T23:00:00", "sleepEndTimestampLocal":"2025-01-16T07:00:00", "avgHeartRate":55, "averageSpO2Value":96, "averageRespirationValue":14},
            "sleepLevels": [{"activityLevel":0,"startGMT":"2025-01-16T04:00:00.000","endGMT":"2025-01-16T04:15:00.000"}, {"activityLevel":2,"startGMT":"2025-01-16T04:15:00.000","endGMT":"2025-01-16T04:30:00.000"}],
            "hrvData":[{"value":51},{"value":52}], "sleepHeartRate":[{"value":55},{"value":-1},{"value":56}],
            "wellnessEpochRespirationDataDTOList":[{"respirationValue":14},{"respirationValue":-1},{"respirationValue":15}],
            "wellnessEpochSPO2DataDTOList":[{"spo2Reading":96},{"spo2Reading":-1},{"spo2Reading":97}],
            "avgSkinTempDeviationC":-0.4
        }),
    );
    put(
        &store,
        "hrv",
        DAY,
        json!({"hrvSummary":{"lastNightAvg":52}}),
    );
    let sleep = screens::sleep_view(&store, DAY);
    assert_eq!(sleep.start_local.as_deref(), Some("2025-01-15T23:00:00"));
    assert_eq!(sleep.stages[0].stage, Stage::Deep);
    assert_eq!(sleep.stages[0].minutes, 15.);
    assert_eq!(sleep.stages[1].start_min, 15.);
    assert_eq!(
        sleep
            .overnight
            .iter()
            .find(|t| t.kind == OvernightKind::Hrv)
            .unwrap()
            .series,
        vec![Some(51.), Some(52.)]
    );
    for (kind, summary, samples) in [
        (
            OvernightKind::HeartRate,
            55.,
            vec![Some(55.), None, Some(56.)],
        ),
        (
            OvernightKind::Respiration,
            14.,
            vec![Some(14.), None, Some(15.)],
        ),
        (OvernightKind::Spo2, 96., vec![Some(96.), None, Some(97.)]),
    ] {
        let tile = sleep.overnight.iter().find(|t| t.kind == kind).unwrap();
        assert_eq!(tile.value, Some(summary));
        assert_eq!(tile.series, samples);
    }
    assert_eq!(
        screens::vitals(&store, DAY)
            .readings
            .iter()
            .find(|r| r.kind == VitalKind::SkinTemp)
            .unwrap()
            .value,
        Some(-0.4)
    );
}

#[test]
fn training_uses_feedback_ratio_load_focus_and_vo2_without_rolling_load_as_daily() {
    let store = Store::open(":memory:").unwrap();
    put(
        &store,
        "training",
        DAY,
        json!({"mostRecentTrainingStatus":{"latestTrainingStatusData":{"device-a":{"trainingStatusFeedbackPhrase":"UNPRODUCTIVE_5"},"device-b":{"primaryTrainingDevice":true,"trainingStatus":6,"trainingStatusFeedbackPhrase":"PRODUCTIVE_6","acuteTrainingLoadDTO":{"dailyTrainingLoadAcute":289,"dailyAcuteChronicWorkloadRatio":1.2,"acwrPercent":57}}}},"acwrPercent":120,"mostRecentVO2Max":{"generic":{"vo2MaxValue":48}}}),
    );
    put(&store, "max_metrics", DAY, json!([]));
    put(
        &store,
        "load_focus",
        DAY,
        json!({"metricsTrainingLoadBalanceDTOMap":{"device-a":{"monthlyLoadAerobicLow":999},"device-b":{"primaryTrainingDevice":true,"monthlyLoadAerobicLow":123,"monthlyLoadAerobicLowTargetMin":150,"monthlyLoadAerobicLowTargetMax":300,"monthlyLoadAerobicHigh":200,"monthlyLoadAerobicHighTargetMin":100,"monthlyLoadAerobicHighTargetMax":250,"monthlyLoadAnaerobic":25,"monthlyLoadAnaerobicTargetMin":50,"monthlyLoadAnaerobicTargetMax":100}}}),
    );
    let view = screens::training(&store, DAY);
    assert_eq!(view.status.as_deref(), Some("Productive"));
    assert_eq!(view.status_tone, Tone::Good);
    assert_eq!(view.chips[0].value.as_deref(), Some("1.2"));
    assert_eq!(view.acute.unwrap().band, None);
    assert!(view.week.is_empty());
    assert_eq!(view.vo2max, Some(48.));
    assert_eq!(view.vo2max_trend[4], Some(48.));
    assert_eq!(view.load_focus.len(), 3);
    assert_eq!(view.load_focus[0].target, Some([150., 300.]));
    assert!(view.load_focus[0].shortage);
    put(
        &store,
        "training_daily",
        DAY,
        json!({"dailyTrainingLoad":73}),
    );
    assert_eq!(
        screens::training(&store, DAY).week.last().unwrap().load,
        73.
    );
    assert_eq!(screens::today(&store, DAY).load.unwrap().band, None);
    let mut numeric_metric = store.get("training", DAY).unwrap();
    numeric_metric["mostRecentTrainingStatus"]["latestTrainingStatusData"]["device-b"]
        ["trainingStatusFeedbackPhrase"] = json!("IMPROVING_VO2_MAX_15");
    put(&store, "training", DAY, numeric_metric);
    assert_eq!(
        screens::training(&store, DAY).status.as_deref(),
        Some("Improving Vo2 Max")
    );
    let no_ratio = Store::open(":memory:").unwrap();
    put(&no_ratio, "training", DAY, json!({"acwrPercent":120}));
    assert!(screens::training(&no_ratio, DAY).chips.is_empty());
}

#[test]
fn fitness_age_components_and_legacy_drivers_only_offer_unmet_levers() {
    let store = Store::open(":memory:").unwrap();
    put(
        &store,
        "fitness_age",
        DAY,
        json!({"fitnessAge":36,"chronologicalAge":40,"components":{"bmi":{"value":31,"targetValue":25,"potentialAge":34},"rhr":{"value":68,"targetValue":60,"potentialAge":35},"vigorousDaysAvg":{"value":1,"targetValue":4,"potentialAge":33},"vigorousMinutesAvg":{"value":40,"targetValue":100,"potentialAge":32}}}),
    );
    let health = screens::health(&store, DAY);
    assert_eq!(health.fitness_age.fitness_age, Some(36.));
    assert_eq!(health.drivers.len(), 4);
    assert_eq!(
        health
            .drivers
            .iter()
            .find(|d| d.key == "vigorousDays")
            .unwrap()
            .unit,
        "days/week"
    );
    assert_eq!(
        health.drivers.iter().find(|d| d.key == "bmi").unwrap().tone,
        Tone::Fair
    );
    assert_eq!(health.lever.unwrap().age_if_met, Some(33.));
    put(
        &store,
        "fitness_age",
        DAY,
        json!({"fitnessAgeData":{"restingHr":{"value":65,"target":60,"ageIfMet":34},"vigorousMinutes":{"value":50,"target":100,"ageIfMet":32}}}),
    );
    let health = screens::health(&store, DAY);
    assert_eq!(health.drivers.len(), 2);
    assert_eq!(health.lever.unwrap().age_if_met, Some(32.));
    put(
        &store,
        "fitness_age",
        DAY,
        json!({"components":{"bmi":{"value":23,"targetValue":25},"rhr":{"value":55,"targetValue":60},"vigorousDaysAvg":{"value":4,"targetValue":4}}}),
    );
    let health = screens::health(&store, DAY);
    assert!(health.lever.is_none());
    assert!(health.drivers.iter().all(|driver| driver.met));
}

#[test]
fn daily_running_tolerance_arrays_are_converted_from_metres() {
    let store = Store::open(":memory:").unwrap();
    for (payload, expected) in [
        (
            json!([{ "calendarDate": DAY, "acuteTolerance": 25300, "acuteDistance": 0 }]),
            Some(25.3),
        ),
        (
            json!([{ "calendarDate": DAY, "acuteTolerance": 0 }]),
            Some(0.),
        ),
        (json!([]), None),
        (json!({ "runningTolerance": 42 }), Some(42.)),
    ] {
        put(&store, "tolerance", DAY, payload);
        assert_eq!(
            screens::training(&store, DAY).running_tolerance_km,
            expected
        );
    }
}

#[test]
fn invalid_overnight_summaries_are_absent_and_legacy_series_still_works() {
    let store = Store::open(":memory:").unwrap();
    put(
        &store,
        "sleep",
        DAY,
        json!({"dailySleepDTO":{"avgHeartRate":-1,"averageSpO2Value":-1,"averageRespirationValue":-1},"overnightSeries":{"heartRate":[50,51]}}),
    );
    assert!(screens::sleep_view(&store, DAY).overnight.is_empty());
    put(
        &store,
        "sleep",
        DAY,
        json!({"dailySleepDTO":{"averageHeartRate":52},"overnightSeries":{"heartRate":[50,51]}}),
    );
    assert_eq!(
        screens::sleep_view(&store, DAY).overnight[0].series,
        vec![Some(50.), Some(51.)]
    );
}

#[test]
fn acute_load_is_rated_against_garmins_chronic_bounds_on_today_and_training() {
    let store = Store::open(":memory:").unwrap();
    for (amount, status, label, tone) in [
        (150, "LOW", "Low", Tone::Fair),
        (250, "OPTIMAL", "Optimal", Tone::Good),
        (360, "HIGH", "High", Tone::Poor),
    ] {
        put(
            &store,
            "training",
            DAY,
            json!({"mostRecentTrainingStatus":{"latestTrainingStatusData":{"device-b":{"primaryTrainingDevice":true,"trainingStatus":4,"acuteTrainingLoadDTO":{"dailyTrainingLoadAcute":amount,"dailyTrainingLoadChronic":226,"minTrainingLoadChronic":180.8,"maxTrainingLoadChronic":339.0,"acwrStatus":status}}}}}),
        );
        let acute = screens::training(&store, DAY).acute.unwrap();
        assert_eq!((acute.value, acute.band, acute.label.as_str(), acute.tone), (amount as f64, Some([180.8, 339.]), label, tone), "{status}");
        assert_eq!(acute.max % 100., 0., "a round scale end");
        assert_eq!(screens::today(&store, DAY).load, Some(acute), "Today draws the same bar");
    }
}
