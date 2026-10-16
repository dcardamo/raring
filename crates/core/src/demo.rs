//! Synthetic QA dataset. No Garmin identity, credentials or network access.
use crate::store::Store;
use chrono::{Duration, NaiveDate};
use serde_json::{json, Value};

pub const WORKOUT_ID: &str = "123456";

pub fn seed(store: &Store, day: NaiveDate) -> rusqlite::Result<()> {
    let fixture: Value = serde_json::from_str(include_str!("../tests/fixtures/day.json"))
        .expect("synthetic fixture must be valid JSON");
    for days_ago in (0..30).rev() {
        let date = day - Duration::days(days_ago);
        let key = date.to_string();
        let mut values = fixture.clone();
        values["sleep"]["dailySleepDTO"]["sleepStartTimestampLocal"] =
            format!("{}T23:15:00", (date - Duration::days(1))).into();
        values["sleep"]["dailySleepDTO"]["sleepEndTimestampLocal"] =
            format!("{key}T07:25:00").into();
        values["summary"]["bodyBatteryHighestValue"] = json!(78);
        values["summary"]["bodyBatteryChargedValue"] = json!(62);
        values["training"]["acuteTrainingLoad"] = json!(612 - days_ago % 7 * 35);
        values["readiness"][0]["score"] = json!(82 - days_ago % 5 * 3);
        values["fitness_age"]["fitnessAgeData"] = json!({
            "vo2max":{"value":52,"target":48},
            "restingHr":{"value":52,"target":55},
            "bmi":{"value":23.8,"target":25},
            "vigorousMinutes":{"value":112,"target":150,"ageIfMet":38}
        });
        for kind in [
            "summary",
            "readiness",
            "sleep",
            "hrv",
            "training",
            "fitness_age",
        ] {
            store.put(kind, &key, &format!("{key}T09:00:00Z"), &values[kind])?;
        }
        store.put(
            "max_metrics",
            &key,
            "synthetic",
            &json!([{"generic":{"vo2MaxValue":52.0 - days_ago as f64 / 30.0}}]),
        )?;
        store.put(
            "load_focus",
            &key,
            "synthetic",
            &json!({
                "lowAerobic":{"value":450,"optimalMin":400,"optimalMax":500},
                "highAerobic":{"value":610,"optimalMin":500,"optimalMax":700},
                "anaerobic":{"value":60,"optimalMin":90,"optimalMax":150}
            }),
        )?;
    }
    store.put(
        "profile",
        "me",
        &format!("{day}T09:00:00Z"),
        &fixture["profile"],
    )?;
    let mut activity = fixture["activity"].clone();
    activity["startTimeLocal"] = format!("{day}T06:45:00").into();
    activity["recoveryTime"] = json!(64800);
    activity["averageHR"] = json!(152);
    activity["calories"] = json!(712);
    activity["elevationGain"] = json!(80);
    store.put(
        "activity",
        WORKOUT_ID,
        &format!("{day}T09:00:00Z"),
        &activity,
    )?;
    store.put(
        "races",
        &day.to_string(),
        "synthetic",
        &json!({"time5K":1200,"time10K":2550,"timeHalfMarathon":5700,"timeMarathon":12000}),
    )?;
    store.put(
        "endurance",
        &day.to_string(),
        "synthetic",
        &json!({"overallScore":6400}),
    )?;
    store.put(
        "hill",
        &day.to_string(),
        "synthetic",
        &json!({"overallScore":72}),
    )?;
    store.put(
        "tolerance",
        &day.to_string(),
        "synthetic",
        &json!({"runningTolerance":45}),
    )?;
    let points: Vec<Value> = (0..80).map(|i| {
        json!({"metrics":[i as f64 * 40.0, 130.0 + (i % 30) as f64, 0.01 + (i as f64 / 50000.0), 0.01 + (i as f64 / 65000.0)]})
    }).collect();
    store.put("details", WORKOUT_ID, "synthetic", &json!({
        "metricDescriptors":[{"key":"directTimestamp"},{"key":"directHeartRate"},{"key":"directLatitude"},{"key":"directLongitude"}],
        "activityDetailMetrics":points
    }))?;
    let laps: Vec<Value> = (0..10)
        .map(|i| json!({"distance":1000,"duration":300 + (i * 7) % 35,"averageHR":145 + i}))
        .collect();
    store.put("splits", WORKOUT_ID, "synthetic", &json!({"lapDTOs":laps}))?;
    store.put(
        "zones",
        WORKOUT_ID,
        &format!("{day}T09:00:00Z"),
        &fixture["zones"],
    )?;
    Ok(())
}
