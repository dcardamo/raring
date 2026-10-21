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
    // Garmin's words for the stress calendar: mostly balanced, a stressful run of days,
    // a day the watch was barely worn and one night it wasn't.
    let qualifier = match days_ago {
        0 => "UNKNOWN",
        9 | 10 => "STRESSFUL",
        11 => "VERY_STRESSFUL",
        d if d % 13 == 5 => "UNKNOWN",
        17 => "BALANCED_AWAKE",
        _ if average < 26 => "CALM",
        _ => "BALANCED",
    };
    let summary = json!({
        "averageStressLevel": average,
        "stressQualifier": qualifier,
        "restStressDuration": seconds(0..=25),
        "lowStressDuration": seconds(26..=50),
        "mediumStressDuration": seconds(51..=75),
        "highStressDuration": seconds(76..=100),
        "activityStressDuration": seconds(-2..=-2),
    });
    (stress, summary)
}

/// One synthetic Garmin day `days_ago` before `day`, in Garmin's shapes.
fn seed_day(store: &Store, fixture: &Value, day: NaiveDate, days_ago: i64) -> rusqlite::Result<()> {
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
    sleep["dailySleepDTO"]["sleepStartTimestampGMT"] = json!(bed.and_utc().timestamp_millis() + 5 * 3_600_000);
    sleep["dailySleepDTO"]["sleepEndTimestampGMT"] = json!(wake.and_utc().timestamp_millis() + 5 * 3_600_000);
    sleep["dailySleepDTO"]["sleepTimeSeconds"] = json!(if days_ago == 0 {
        28080
    } else {
        26400 + ((days_ago * 317) % 4200)
    });
    sleep["dailySleepDTO"]["sleepNeed"]["totalSleepNeedInSeconds"] = json!(27000);
    sleep["dailySleepDTO"]["sleepAlignment"] = json!({
        "status": if shift > 18 { "BEHIND" } else { "ALIGNED" },
        "optimalSleepWindowStartMins": -75,
        "optimalSleepWindowEndMins": 375,
    });
    sleep["dailySleepDTO"]["nextSleepNeed"] = json!({
        "calendarDate": (date + Duration::days(1)).to_string(),
        "actual": 470,
        "baseline": 450,
        "recommendedBedtimeStartMins": 1350,
        "recommendedBedtimeEndMins": 1386,
        "trainingFeedback": "DAILY_ACTIVITY_AND_TODAYS_LOAD",
    });
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
    values["summary"]["moderateIntensityMinutes"] =
        json!([12, 30, 8, 0, 25, 18, 0][days_ago as usize % 7]);
    values["summary"]["vigorousIntensityMinutes"] =
        json!([20, 14, 0, 0, 22, 6, 0][days_ago as usize % 7]);
    values["summary"]["intensityMinutesGoal"] = json!(150);
    values["summary"]["bodyBatteryAtWakeTime"] = json!(if days_ago == 0 {
        87
    } else {
        70 + days_ago % 25
    });
    if days_ago == 0 {
        values["summary"]["bodyBatteryActivityEventList"] = json!([
            {"eventType":"SLEEP","bodyBatteryImpact":62,"timezoneOffset":0,
             "eventStartTimeGmt":bed.format("%Y-%m-%dT%H:%M:%S").to_string(),
             "durationInMilliseconds":(wake - bed).num_milliseconds(),"shortFeedback":"NONE"},
            {"eventType":"ACTIVITY","activityName":"Morning Run","bodyBatteryImpact":-9,"timezoneOffset":0,
             "eventStartTimeGmt":format!("{date}T06:12:00"),"durationInMilliseconds":3_134_000,
             "shortFeedback":"IMPROVING_TEMPO"}
        ]);
    }
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
    // Garmin's readiness carries the values its factors used.
    let (sleep_score, acute) = (
        values["sleep"]["dailySleepDTO"]["sleepScores"]["overall"]["value"].clone(),
        values["training"]["acuteTrainingLoad"].clone(),
    );
    let r = &mut values["readiness"][0];
    r["sleepScore"] = sleep_score;
    r["hrvWeeklyAverage"] = json!(56);
    r["acuteLoad"] = acute;
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
        (40. + (days_ago - 8) as f64 * 0.02).min(46.)
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
    if days_ago < 2 {
        let start = date
            .and_hms_opt(5, 0, 0)
            .unwrap()
            .and_utc()
            .timestamp_millis();
        let samples: Vec<Value> = (0..if days_ago == 0 { 400 } else { 720 })
            .map(|i| {
                json!([
                    start + i * 120_000,
                    if (210..235).contains(&i) {
                        Value::Null
                    } else {
                        json!(
                            60 + (i as f64 / 15.).sin().round() as i64 * 8
                                + if (260..290).contains(&i) { 45 } else { 0 }
                        )
                    }
                ])
            })
            .collect();
        store.put("daily_hr", &key, "synthetic", &json!({
            "startTimestampGMT":format!("{date}T05:00:00.0"),
            "startTimestampLocal":format!("{date}T00:00:00.0"),
            "endTimestampGMT":format!("{}T05:00:00.0",date + Duration::days(1)),
            "endTimestampLocal":format!("{}T00:00:00.0",date + Duration::days(1)),
            "restingHeartRate":54,"minHeartRate":49,"maxHeartRate":132,"heartRateValues":samples
        }))?;
    }
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
    store.put("max_metrics", &key, "synthetic", &json!([{"generic":{"vo2MaxValue":if days_ago < 8 { 52. } else if days_ago < 15 { 50.5 } else { (49. + (42 - days_ago) as f64 / 30.).max(44.) }}}]))?;
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
    Ok(())
}

/// The demo plus `years` of earlier synthetic history: every day's Garmin rows
/// and about six activities a week (runs, rides, walks and strength with sets),
/// for long-history timings. Synthetic only; never used by sign-in or sync.
pub fn seed_long_history(store: &Store, day: NaiveDate, years: i64) -> rusqlite::Result<u32> {
    seed(store, day)?;
    let fixture: Value = serde_json::from_str(include_str!("../tests/fixtures/day.json")).expect("synthetic fixture must be valid JSON");
    let days = 365 * years;
    for days_ago in 42..days {
        seed_day(store, &fixture, day, days_ago)?;
    }
    let first = day - Duration::days(days - 1);
    store.set_cursor("listed:lo", &first.to_string())?;
    let mut count = 0;
    let mut pages: Vec<Value> = vec![];
    for days_ago in 2..days {
        let date = day - Duration::days(days_ago);
        // Six of seven days, one kind each by weekday.
        let slot = (days_ago % 7) as usize;
        let kind = ["running", "strength_training", "cycling", "", "running", "strength_training", "walking"][slot];
        if kind.is_empty() {
            continue;
        }
        let id = 900_000_000 + days_ago as u64;
        let minutes = 30 + (days_ago * 7) % 50;
        let distance = match kind { "running" => 5000 + (days_ago * 131) % 9000, "cycling" => 20000 + (days_ago * 377) % 40000, "walking" => 3000, _ => 0 };
        let mut summary = json!({"startTimeLocal": format!("{date} 07:30:00"), "duration": minutes * 60, "averageHR": 120 + days_ago % 40,
            "activityTrainingLoad": 40 + days_ago % 160, "moderateIntensityMinutes": minutes / 3, "vigorousIntensityMinutes": minutes / 5});
        if distance > 0 {
            summary["distance"] = json!(distance);
        }
        store.put("activity", &id.to_string(), "synthetic", &json!({"activityId": id, "activityName": format!("History {kind}"),
            "activityTypeDTO": {"typeKey": kind}, "summaryDTO": summary}))?;
        // As sync keeps them: every listed activity by id, the newest also in list pages.
        let listed = json!({"activityId": id, "activityName": format!("History {kind}"), "activityType": {"typeKey": kind},
            "startTimeLocal": format!("{date} 07:30:00"), "duration": minutes * 60, "distance": distance});
        store.put("listed", &id.to_string(), "synthetic", &listed)?;
        if count < 400 {
            pages.push(listed);
        }
        let z = minutes * 60;
        store.put("zones", &id.to_string(), "synthetic", &json!([{"zoneNumber": 1, "secsInZone": z / 5}, {"zoneNumber": 2, "secsInZone": z / 2},
            {"zoneNumber": 3, "secsInZone": z / 5}, {"zoneNumber": 4, "secsInZone": z / 10}, {"zoneNumber": 5, "secsInZone": 0}]))?;
        if kind == "strength_training" {
            // Weights drift up over the years, so bests and milestones accrue.
            let progress = (days - days_ago) as f64 / days as f64;
            let set = |category: &str, name: &str, reps: i64, kg: f64| json!({"setType": "ACTIVE", "repetitionCount": reps,
                "weight": ((kg * (0.8 + 0.4 * progress)) * 1000.).round(), "duration": 40.0, "exercises": [{"category": category, "name": name, "probability": 95.0}]});
            store.put("exercise_sets", &id.to_string(), "synthetic", &json!({"activityId": id, "exerciseSets": [
                set("SQUAT", "GOBLET_SQUAT", 8, 32.), set("SQUAT", "GOBLET_SQUAT", 8, 32.),
                set("BENCH_PRESS", "BARBELL_BENCH_PRESS", 5, 70.), set("BENCH_PRESS", "BARBELL_BENCH_PRESS", 5, 70.),
                set("DEADLIFT", "ROMANIAN_DEADLIFT", 8, 80.)]}))?;
        }
        count += 1;
    }
    for (n, page) in pages.chunks(20).enumerate() {
        store.put("activities", &(n * 20).to_string(), "synthetic", &json!(page))?;
    }
    Ok(count)
}

pub fn seed(store: &Store, day: NaiveDate) -> rusqlite::Result<()> {
    seed_variant(store, day, true)
}

/// `recorded_today: false` moves the morning run to the day before, so a
/// planned session can lead today (the debug emulator's Keep / Choose check).
pub fn seed_variant(store: &Store, day: NaiveDate, recorded_today: bool) -> rusqlite::Result<()> {
    let fixture: Value = serde_json::from_str(include_str!("../tests/fixtures/day.json"))
        .expect("synthetic fixture must be valid JSON");
    for days_ago in (0..42).rev() {
        seed_day(store, &fixture, day, days_ago)?;
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
    store.put(
        "devices",
        "me",
        "synthetic",
        &json!([
            {"deviceId": 1001, "primaryActivityTrackerIndicator": true,
             "sleepSkinTemperatureCapable": true, "hrvStatusCapable": true,
             "trainingReadinessCapable": true, "sleepScoreCapable": true}
        ]),
    )?;
    store.put(
        "device_detail",
        "selected",
        "synthetic",
        &json!({
            "deviceId": 1001,
            "lastUploadTimestamp": day.and_hms_opt(9, 0, 0).unwrap().and_utc().timestamp_millis()
        }),
    )?;
    for (id, name, kind, when, distance, seconds, load, zones) in [
        (
            WORKOUT_ID,
            "Morning Run",
            "running",
            if recorded_today { day } else { day - Duration::days(1) },
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
        activity["startTimeGMT"] = json!(format!("{when}T11:12:00"));
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
        if id == WORKOUT_ID {
            activity["moderateIntensityMinutes"] = json!(14);
            activity["vigorousIntensityMinutes"] = json!(24);
            activity["differenceBodyBattery"] = json!(-9);
            activity["movingDuration"] = json!(3101);
        }
        store.put("activity", id, &format!("{when}T09:00:00Z"), &activity)?;
        store.put("zones", id, "synthetic", &json!(zones.iter().enumerate().map(|(i, s)| json!({"zoneNumber":i+1,"secsInZone":s,"minHeartRate":([0,120,138,154,167][i])})).collect::<Vec<_>>()))?;
    }
    // A synthetic adaptive Garmin Coach week around today, read just now.
    let task = |offset: i64, name: &str, sport: Option<&str>, seconds: u32, label: &str| {
        json!({"calendarDate": (day + Duration::days(offset)).to_string(), "workoutOrder": 1, "taskWorkout": {
            "workoutName": name, "workoutId": null, "estimatedDurationInSecs": seconds, "restDay": sport.is_none(),
            "trainingEffectLabel": label, "sportType": sport.map(|s| json!({"sportTypeKey": s})), "adaptiveCoachingWorkoutStatus": "NOT_COMPLETE"}})
    };
    let fetched = chrono::Utc::now().to_rfc3339();
    store.put("coach_plans", "current", &fetched, &json!({"trainingPlanList": [{"trainingPlanId": 4242, "trainingPlanCategory": "FBT_ADAPTIVE",
        "trainingStatus": {"statusKey": "Scheduled"}}]}))?;
    store.put("coach_adaptive", "current", &fetched, &json!({"trainingPlanId": 4242, "trainingPlanCategory": "FBT_ADAPTIVE", "name": "Sample Coach",
        "taskList": [task(-1, "Strength", Some("strength_training"), 2400, "UNKNOWN"), task(0, "Base", Some("other"), 1800, "AEROBIC_BASE"),
            task(1, "Rest", None, 0, "INVALID"), task(2, "Threshold", Some("other"), 2700, "LACTATE_THRESHOLD")]}))?;
    // The synthetic activity list covers the seeded days, so coverage rules apply.
    store.set_cursor("listed:lo", &(day - Duration::days(41)).to_string())?;
    store.set_cursor("listed:hi", &day.to_string())?;
    // Yesterday's strength session has the synthetic sets fixture, so Strength,
    // lifts and muscles have something to show; units and a weigh-in fill Profile.
    let mut sets: Value = serde_json::from_str(include_str!("../tests/fixtures/exercise_sets.json")).expect("valid synthetic sets fixture");
    sets["activityId"] = json!(123457);
    store.put("exercise_sets", "123457", "synthetic", &sets)?;
    let catalog: Value = serde_json::from_str(include_str!("../tests/fixtures/exercise_catalog.json")).expect("valid synthetic catalog fixture");
    if let Some(reduced) = crate::strength::reduce_catalog(&catalog) {
        store.put("exercise_catalog", "garmin", "synthetic", &reduced)?;
    }
    store.put("profile_settings", "me", "synthetic", &json!({"measurementSystem": "metric"}))?;
    let weighed = day - Duration::days(3);
    store.put("weight", &weighed.to_string(), "synthetic", &json!({"weight": 72400.0,
        "timestampGMT": weighed.and_hms_opt(7, 0, 0).unwrap().and_utc().timestamp_millis()}))?;
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
        // Speed in m/s and elevation in metres, as Garmin's unit keys say.
        let speed = 3.3 + 0.25 * (angle * 3.).sin() - (hr - 150.) / 200.;
        let elevation = 60. + 8. * (angle * 2.).sin();
        json!({"metrics":[t * 3134., hr, 45.501 + (0.5 - y) * 0.008, -73.601 + (x - 0.5) * 0.012, speed, elevation]})
    }).collect();
    store.put("details", WORKOUT_ID, "synthetic", &json!({
        "metricDescriptors":[{"key":"directTimestamp"},{"key":"directHeartRate"},{"key":"directLatitude"},{"key":"directLongitude"},
            {"key":"directSpeed","metricsIndex":4,"unit":{"key":"mps","factor":0.1}},{"key":"directElevation","metricsIndex":5,"unit":{"key":"meter","factor":100.0}}],
        "activityDetailMetrics":points
    }))?;
    let times = [321, 308, 302, 298, 305, 295, 301, 297, 310, 318];
    store.put("splits", WORKOUT_ID, "synthetic", &json!({"lapDTOs":times.iter().enumerate().map(|(i, t)| json!({"distance":1000,"duration":t,"averageHR":145+i})).collect::<Vec<_>>()}))?;
    Ok(())
}
