//! Synthetic QA dataset. No Garmin identity, credentials or network access.
use crate::store::Store;
use chrono::{Duration, NaiveDate};
use serde_json::{json, Value};

pub const WORKOUT_ID: &str = "123456";

/// A day of three-minute stress readings in Garmin's shape: calm sleep, a
/// morning workout Garmin cannot rate (-2), a short off-wrist gap (-1) and a
/// busier afternoon. Today stops at midday, as if synced then.
fn stress(date: NaiveDate, days_ago: i64) -> (Value, Value) {
    let start = date
        .and_hms_opt(5, 0, 0)
        .unwrap()
        .and_utc()
        .timestamp_millis();
    let slots = if days_ago == 0 { 240 } else { 480 };
    let wobble = |i: i64, f: f64| ((i as f64 * f + days_ago as f64).sin() * 6.).round() as i64;
    let values: Vec<i64> = (0..slots)
        .map(|i| match i * 3 {
            m if m < 372 => 14 + wobble(i, 0.7),
            m if (372..424).contains(&m) => -2,
            m if (424..445).contains(&m) => -1,
            m if m < 720 => 30 + wobble(i, 0.4) + (i % 9) * 2,
            m if m < 1020 => 42 + wobble(i, 0.3) + ((i * 7 + days_ago) % 23) * 2,
            m if m < 1350 => 24 + wobble(i, 0.5),
            _ => 16 + wobble(i, 0.8),
        })
        .collect();
    let seconds = |range: std::ops::RangeInclusive<i64>| {
        values.iter().filter(|v| range.contains(v)).count() as i64 * 180
    };
    let measured: Vec<i64> = values.iter().copied().filter(|v| *v >= 0).collect();
    let average = measured.iter().sum::<i64>() / measured.len().max(1) as i64;
    let stress = json!({
        "calendarDate": date.to_string(),
        "startTimestampGMT": format!("{date}T05:00:00.0"),
        "startTimestampLocal": format!("{date}T00:00:00.0"),
        "avgStressLevel": average,
        "maxStressLevel": measured.iter().max(),
        "stressValuesArray": values
            .iter()
            .enumerate()
            .map(|(i, v)| json!([start + i as i64 * 180_000, v]))
            .collect::<Vec<_>>(),
    });
    let summary = json!({
        "averageStressLevel": average,
        "stressQualifier": if days_ago == 0 { "UNKNOWN" } else if average < 26 { "CALM" } else { "BALANCED" },
        "restStressDuration": seconds(0..=25),
        "lowStressDuration": seconds(26..=50),
        "mediumStressDuration": seconds(51..=75),
        "highStressDuration": seconds(76..=100),
        "activityStressDuration": seconds(-2..=-2),
    });
    (stress, summary)
}

pub fn seed(store: &Store, day: NaiveDate) -> rusqlite::Result<()> {
    let fixture: Value = serde_json::from_str(include_str!("../tests/fixtures/day.json"))
        .expect("synthetic fixture must be valid JSON");
    for days_ago in (0..42).rev() {
        let date = day - Duration::days(days_ago);
        let key = date.to_string();
        let mut values = fixture.clone();
        let shift = if days_ago == 0 {
            0
        } else {
            ((days_ago * 37) % 51) - 25
        };
        let bed = date.and_hms_opt(0, 0, 0).unwrap() - Duration::minutes(89 - shift);
        let wake = date
            .and_hms_opt(6, if days_ago == 0 { 39 } else { 20 }, 0)
            .unwrap()
            + Duration::minutes(shift / 3);
        let sleep = &mut values["sleep"];
        sleep["dailySleepDTO"]["sleepStartTimestampLocal"] =
            json!(bed.format("%Y-%m-%dT%H:%M:%S").to_string());
        sleep["dailySleepDTO"]["sleepEndTimestampLocal"] =
            json!(wake.format("%Y-%m-%dT%H:%M:%S").to_string());
        sleep["dailySleepDTO"]["sleepTimeSeconds"] = json!(if days_ago == 0 {
            28080
        } else {
            26400 + ((days_ago * 317) % 4200)
        });
        sleep["dailySleepDTO"]["sleepNeed"]["totalSleepNeedInSeconds"] = json!(27000);
        sleep["dailySleepDTO"]["sleepScores"]["overall"]["value"] = json!(if days_ago == 0 {
            84
        } else {
            67 + (days_ago * 11) % 24
        });
        for (key, current, usual, spread) in [
            ("deepSleepSeconds", 5760, 5040, 700),
            ("lightSleepSeconds", 15120, 15360, 900),
            ("remSleepSeconds", 7200, 6660, 700),
            ("awakeSleepSeconds", 1200, 1560, 300),
        ] {
            sleep["dailySleepDTO"][key] = json!(if days_ago == 0 {
                current
            } else {
                usual + ((days_ago * 137) % spread) - spread / 2
            });
        }
        sleep["dailySleepDTO"]["averageHeartRate"] = json!(if days_ago == 0 {
            46.
        } else {
            46. + (days_ago as f64 * 0.67).sin() * 1.8
        });
        sleep["dailySleepDTO"]["averageRespirationValue"] = json!(if days_ago == 0 {
            14.1
        } else {
            14.1 + (days_ago as f64 * 0.43).sin() * 0.25
        });
        sleep["dailySleepDTO"]["avgSpO2"] = json!(if days_ago == 0 {
            96
        } else {
            96 + ((days_ago as f64 * 0.51).sin() * 1.2).round() as i64
        });
        sleep["dailySleepDTO"]["skinTempDeviation"] = json!(if days_ago == 0 {
            0.1
        } else {
            (days_ago as f64 * 0.38).sin() * 0.25
        });
        sleep["overnightSeries"] = json!({
            "hrv": [49,54,57,61,60,58,56,59],
            "heartRate": [52,50,48,47,46,46,47,46,48,47,49,51],
            "respiration": [14.4,14.2,14.0,13.9,14.1,14.0,14.2,14.1],
            "spo2": [97,96,95,96,96,95,96,96]
        });
        let mut timestamp = bed.and_utc().timestamp();
        let stages = [
            (1, 12),
            (0, 38),
            (1, 22),
            (2, 14),
            (1, 30),
            (0, 32),
            (1, 18),
            (3, 4),
            (2, 22),
            (1, 34),
            (0, 18),
            (1, 26),
            (2, 28),
            (3, 6),
            (1, 40),
            (0, 8),
            (1, 24),
            (2, 30),
            (1, 28),
            (3, 5),
            (2, 26),
            (1, 18),
            (3, 5),
        ];
        sleep["sleepLevels"] = json!(stages
            .iter()
            .map(|(level, minutes)| {
                let start = timestamp;
                timestamp += minutes * 60;
                json!({"activityLevel":level,"startGMT":start,"endGMT":timestamp})
            })
            .collect::<Vec<_>>());
        values["summary"]["restingHeartRate"] = json!(if days_ago == 0 {
            46
        } else {
            46 + ((days_ago as f64 * 0.54).sin() * 1.5).round() as i64
        });
        values["summary"]["bodyBatteryMostRecentValue"] = json!(if days_ago == 0 {
            78
        } else {
            62 + days_ago % 27
        });
        values["summary"]["bodyBatteryChargedValue"] = json!(if days_ago == 0 {
            62
        } else {
            45 + days_ago % 22
        });
        values["hrv"]["hrvSummary"]["lastNightAvg"] = json!(if days_ago == 0 {
            58
        } else {
            54 + ((days_ago as f64 * 0.61).sin() * 4.).round() as i64
        });
        values["training"]["acuteTrainingLoad"] = json!(if days_ago == 0 {
            612
        } else {
            460 + (days_ago * 47) % 270
        });
        values["training"]["acuteChronicWorkloadRatio"] = json!(1.1);
        values["training"]["dailyTrainingLoad"] =
            json!([186, 320, 95, 0, 210, 142, 0][days_ago as usize % 7]);
        let r = &mut values["readiness"][0];
        r["score"] = json!(if days_ago == 0 {
            82
        } else {
            62 + (days_ago * 13) % 29
        });
        r["recoveryTime"] = json!(1080);
        for (percent, feedback, value, rating) in [
            (
                "sleepScoreFactorPercent",
                "sleepScoreFactorFeedback",
                86,
                "GOOD",
            ),
            (
                "recoveryTimeFactorPercent",
                "recoveryTimeFactorFeedback",
                92,
                "EXCELLENT",
            ),
            ("hrvFactorPercent", "hrvFactorFeedback", 78, "BALANCED"),
            ("acwrFactorPercent", "acwrFactorFeedback", 58, "MODERATE"),
            (
                "sleepHistoryFactorPercent",
                "sleepHistoryFactorFeedback",
                81,
                "GOOD",
            ),
            (
                "stressHistoryFactorPercent",
                "stressHistoryFactorFeedback",
                73,
                "GOOD",
            ),
        ] {
            r[percent] = json!(if days_ago == 0 {
                value
            } else {
                value - (days_ago * 3) % 17
            });
            r[feedback] = json!(rating);
        }
        values["fitness_age"]["fitnessAge"] = json!(if days_ago < 12 {
            39.
        } else {
            40. + (days_ago - 8) as f64 * 0.02
        });
        values["fitness_age"]["chronologicalAge"] = json!(46);
        values["fitness_age"]["fitnessAgeData"] = json!({
            "vo2max":{"value":52,"target":48},
            "restingHr":{"value":46,"target":55},
            "bmi":{"value":23.8,"target":25},
            "vigorousMinutes":{"value":112,"target":150,"ageIfMet":38}
        });
        let (stress, summary) = stress(date, days_ago);
        store.put("stress", &key, "synthetic", &stress)?;
        for (field, value) in summary.as_object().unwrap() {
            values["summary"][field] = value.clone();
        }
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
        store.put("respiration", &key, "synthetic", &json!({"sleepingRespirationValue":values["sleep"]["dailySleepDTO"]["averageRespirationValue"]}))?;
        store.put(
            "spo2",
            &key,
            "synthetic",
            &json!({"averageSpO2":values["sleep"]["dailySleepDTO"]["avgSpO2"]}),
        )?;
        store.put("max_metrics", &key, "synthetic", &json!([{"generic":{"vo2MaxValue":if days_ago < 8 { 52. } else if days_ago < 15 { 50.5 } else { 49. + (42 - days_ago) as f64 / 30. }}}]))?;
        store.put(
            "load_focus",
            &key,
            "synthetic",
            &json!({
                "lowAerobic":{"value":450,"optimalMin":300,"optimalMax":600},
                "highAerobic":{"value":610,"optimalMin":400,"optimalMax":800},
                "anaerobic":{"value":60,"optimalMin":150,"optimalMax":350}
            }),
        )?;
    }
    // The age trend samples one historical record per month.
    for months_ago in 1..12 {
        if let Some(month) = day.checked_sub_months(chrono::Months::new(months_ago)) {
            let key = month.to_string();
            store.put(
                "fitness_age",
                &key,
                "synthetic",
                &json!({"fitnessAge":39. + months_ago as f64 * 0.1,"chronologicalAge":46}),
            )?;
        }
    }
    store.put(
        "profile",
        "me",
        &format!("{day}T09:00:00Z"),
        &fixture["profile"],
    )?;
    for (id, name, kind, when, distance, seconds, load, zones) in [
        (
            WORKOUT_ID,
            "Morning Run",
            "running",
            day,
            10200,
            3134,
            186,
            [190, 585, 1100, 1050, 209],
        ),
        (
            "123457",
            "Strength",
            "strength_training",
            day - Duration::days(1),
            0,
            2882,
            74,
            [840, 1320, 540, 180, 2],
        ),
        (
            "123458",
            "Evening Ride",
            "cycling",
            day - Duration::days(2),
            38400,
            5050,
            246,
            [480, 1800, 1920, 720, 120],
        ),
    ] {
        let mut activity = fixture["activity"].clone();
        activity["activityId"] = json!(id.parse::<u64>().unwrap());
        activity["activityName"] = json!(name);
        activity["activityType"]["typeKey"] = json!(kind);
        activity["startTimeLocal"] = json!(format!("{when}T06:12:00"));
        activity["distance"] = json!(distance);
        activity["duration"] = json!(seconds);
        activity["activityTrainingLoad"] = json!(load);
        activity["recoveryTime"] = json!(64800);
        activity["averageHR"] = json!(152);
        activity["maxHR"] = json!(171);
        activity["calories"] = json!(712);
        activity["elevationGain"] = Value::Null;
        activity["anaerobicTrainingEffect"] = json!(1.2);
        activity["anaerobicTrainingEffectMessage"] = json!("MINOR");
        store.put("activity", id, &format!("{when}T09:00:00Z"), &activity)?;
        store.put("zones", id, "synthetic", &json!(zones.iter().enumerate().map(|(i, s)| json!({"zoneNumber":i+1,"secsInZone":s,"minHeartRate":([0,120,138,154,167][i])})).collect::<Vec<_>>()))?;
    }
    store.put(
        "races",
        &day.to_string(),
        "synthetic",
        &json!({"time5K":1300,"time10K":2710,"timeHalfMarathon":5990,"timeMarathon":12665}),
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
    let points: Vec<Value> = (0..210).map(|i| {
        let t = i as f64 / 209.;
        let angle = t * std::f64::consts::TAU;
        let hr = 112. + 47. * (t * 8.).min(1.) + 9. * (angle * 5.).sin() + 3. * (angle * 19.).sin();
        let waypoints = [(0.10, 0.92), (0.08, 0.57), (0.18, 0.28), (0.36, 0.12), (0.59, 0.08), (0.82, 0.20), (0.94, 0.49), (0.83, 0.78), (0.64, 0.85), (0.43, 0.77), (0.22, 0.96), (0.10, 0.92)];
        let section = t * (waypoints.len() - 1) as f64;
        let index = (section as usize).min(waypoints.len() - 2);
        let fraction = section - index as f64;
        let x = waypoints[index].0 + (waypoints[index + 1].0 - waypoints[index].0) * fraction;
        let y = waypoints[index].1 + (waypoints[index + 1].1 - waypoints[index].1) * fraction;
        json!({"metrics":[t * 3134., hr, 45.501 + (0.5 - y) * 0.008, -73.601 + (x - 0.5) * 0.012]})
    }).collect();
    store.put("details", WORKOUT_ID, "synthetic", &json!({
        "metricDescriptors":[{"key":"directTimestamp"},{"key":"directHeartRate"},{"key":"directLatitude"},{"key":"directLongitude"}],
        "activityDetailMetrics":points
    }))?;
    let times = [321, 308, 302, 298, 305, 295, 301, 297, 310, 318];
    store.put("splits", WORKOUT_ID, "synthetic", &json!({"lapDTOs":times.iter().enumerate().map(|(i, t)| json!({"distance":1000,"duration":t,"averageHR":145+i})).collect::<Vec<_>>()}))?;
    Ok(())
}
