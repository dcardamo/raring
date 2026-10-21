//! Opening a past day or an older workout: missing per-day detail is fetched
//! once, explicitly, and kept. Synthetic server only; no Garmin account.
use chrono::{Duration, Local, Utc};
use raring_core::{garmin::Client, screens, store::Store, sync, view::DayDetailPart, Tokens};
use serde_json::json;
use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::{Arc, Mutex},
};

fn test_client(url: &str) -> Client {
    let token = format!(
        "e30.{}.sig",
        base64::Engine::encode(
            &base64::engine::general_purpose::URL_SAFE_NO_PAD,
            json!({"exp":1900000000,"client_id":"TEST_PAST"}).to_string()
        )
    );
    Client::with_urls(
        Tokens {
            access_token: token,
            refresh_token: "refresh".into(),
            client_id: "TEST_PAST".into(),
            expires_at: Utc::now() + Duration::days(30),
        },
        url,
        url,
        std::time::Duration::ZERO,
    )
    .unwrap()
}

fn profile(store: &Store) {
    store
        .put("profile", "me", "synthetic", &json!({"displayName": "Synthetic"}))
        .unwrap();
}

fn sleep_body() -> serde_json::Value {
    json!({
        "dailySleepDTO": {
            "calendarDate": "2025-06-03",
            "sleepTimeSeconds": 27000,
            "deepSleepSeconds": 3600,
            "lightSleepSeconds": 18000,
            "remSleepSeconds": 4800,
            "awakeSleepSeconds": 600,
            "sleepStartTimestampLocal": "2025-06-02T22:30:00",
            "sleepEndTimestampLocal": "2025-06-03T06:45:00"
        },
        "sleepLevels": [
            {"activityLevel": 1, "startGMT": "2025-06-02T20:30:00.0", "endGMT": "2025-06-02T21:30:00.0"},
            {"activityLevel": 0, "startGMT": "2025-06-02T21:30:00.0", "endGMT": "2025-06-02T22:30:00.0"},
            {"activityLevel": 3, "startGMT": "2025-06-02T22:30:00.0", "endGMT": "2025-06-02T22:40:00.0"}
        ]
    })
}

fn stress_body() -> serde_json::Value {
    let start = 1748923200000_i64;
    json!({
        "startTimestampGMT": "2025-06-03T04:00:00.0",
        "avgStressLevel": 34,
        "maxStressLevel": 78,
        "stressValuesArray": [[start, 22], [start + 180_000, 41], [start + 360_000, -1], [start + 540_000, 63]]
    })
}

fn summary_body() -> serde_json::Value {
    json!({
        "calendarDate": "2025-06-03",
        "averageStressLevel": 34,
        "maxStressLevel": 78,
        "restStressDuration": 3600,
        "lowStressDuration": 5400,
        "mediumStressDuration": 1800,
        "highStressDuration": 600,
        "activityStressDuration": 1200,
        "stressQualifier": "BALANCED"
    })
}

fn hrv_body() -> serde_json::Value {
    json!({"hrvSummary": {"calendarDate": "2025-06-03", "lastNightAvg": 57, "status": "BALANCED"}})
}

fn fitness_age_body() -> serde_json::Value {
    json!({
        "chronologicalAge": 44,
        "fitnessAge": 38.5,
        "lastUpdated": "2025-06-01T00:00:00.0",
        "components": {
            "rhr": {"value": 52},
            "vigorousMinutesAvg": {"value": 40, "targetValue": 75, "potentialAge": 37.2},
            "bmi": {"value": 23.4, "targetValue": 22.0}
        }
    })
}

fn health_status_body() -> serde_json::Value {
    json!({"data": {"healthStatusSummary": {
        "calendarDate": "2025-06-03",
        "outliersCount": 0,
        "metrics": []
    }}})
}

/// Accepts `count` requests, answers each by path, and records the request lines.
fn server(count: usize, answer: impl Fn(&str) -> String + Send + 'static) -> (String, Arc<Mutex<Vec<String>>>, std::thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let seen = Arc::new(Mutex::new(Vec::new()));
    let requests = Arc::clone(&seen);
    let thread = std::thread::spawn(move || {
        for _ in 0..count {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = [0; 8192];
            let n = stream.read(&mut buf).unwrap();
            let line = String::from_utf8_lossy(&buf[..n]).into_owned();
            let request_line = line.lines().next().unwrap_or_default().to_owned();
            requests.lock().unwrap().push(request_line.clone());
            let body = answer(&request_line);
            let header = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            stream.write_all(header.as_bytes()).unwrap();
            stream.write_all(body.as_bytes()).unwrap();
        }
    });
    (url, seen, thread)
}

#[tokio::test]
async fn opened_past_day_fetches_only_the_page_it_opens_once_and_keeps_it() {
    let store = Store::open(":memory:").unwrap();
    profile(&store);
    // A past night reads the night and its HRV, never the day's stress.
    let (url, seen, thread) = server(2, |line| {
        if line.contains("dailySleepData") {
            sleep_body().to_string()
        } else {
            hrv_body().to_string()
        }
    });
    let client = test_client(&url);
    let report = sync::day_detail_on_demand(&store, &client, "2025-06-03", &[DayDetailPart::Sleep]).await;
    thread.join().unwrap();
    assert_eq!(report.requests, 2);
    assert_eq!(report.stored, 2);
    assert!(report.error.is_none());
    {
        let lines = seen.lock().unwrap();
        assert!(lines[0].starts_with("GET /wellness-service/wellness/dailySleepData/Synthetic?"));
        assert!(lines[0].contains("date=2025-06-03"));
        assert!(lines[1].starts_with("GET /hrv-service/hrv/2025-06-03"));
    }
    assert!(store.get("stress", "2025-06-03").is_none(), "a night does not read the day");
    let sleep = screens::sleep_view(&store, "2025-06-03");
    assert_eq!(sleep.stages.len(), 3);
    assert_eq!(sleep.asleep_min, Some(450.));
    assert!(sleep.detail.missing.is_empty() && sleep.detail.unavailable.is_empty());
    assert_eq!(
        sync::day_detail_on_demand(&store, &client, "2025-06-03", &[DayDetailPart::Sleep])
            .await
            .requests,
        0
    );
    // A past Health day reads its Fitness Age, stress and Health Status, never the night.
    let day = screens::day_detail_parts("day").unwrap();
    assert_eq!(day, [DayDetailPart::FitnessAge, DayDetailPart::Stress, DayDetailPart::Vitals]);
    // A day's stress detail shows no Fitness Age, so it never reads one.
    assert_eq!(screens::day_detail_parts("stress").unwrap(), [DayDetailPart::Stress, DayDetailPart::Vitals]);
    assert_eq!(screens::health(&store, "2025-06-03").detail.missing, day);
    let (url, seen, thread) = server(4, |line| {
        if line.contains("/graphql-gateway/graphql") {
            health_status_body().to_string()
        } else if line.contains("/fitnessage-service/") {
            fitness_age_body().to_string()
        } else if line.contains("dailyStress") {
            stress_body().to_string()
        } else {
            summary_body().to_string()
        }
    });
    let day_client = test_client(&url);
    let report = sync::day_detail_on_demand(&store, &day_client, "2025-06-03", day).await;
    thread.join().unwrap();
    assert_eq!(report.requests, 4);
    assert_eq!(report.stored, 4);
    {
        let lines = seen.lock().unwrap();
        assert!(lines[0].starts_with("GET /fitnessage-service/fitnessage/2025-06-03 "));
        assert!(lines[1].starts_with("GET /wellness-service/wellness/dailyStress/2025-06-03"));
        assert!(lines[2].starts_with("GET /usersummary-service/usersummary/daily/Synthetic?"));
        assert!(lines[2].contains("calendarDate=2025-06-03"));
        assert!(lines[3].starts_with("POST /graphql-gateway/graphql"));
    }
    let health = screens::health(&store, "2025-06-03");
    assert!(!health.stress.timeline.is_empty());
    assert_eq!(health.stress.average, Some(34.));
    // That day's own estimate and its drivers, never a later one.
    assert_eq!(health.fitness_age.fitness_age, Some(38.5));
    assert_eq!(health.fitness_age.chronological_age, Some(44.));
    assert_eq!(health.fitness_age.recorded_on.as_deref(), Some("2025-06-03"));
    assert_eq!(health.lever.as_ref().map(|l| l.key.as_str()), Some("vigorousMinutes"));
    assert!(health.drivers.iter().any(|d| d.key == "restingHr" && d.value == Some(52.)));
    assert!(health.detail.missing.is_empty());
    assert_eq!(
        sync::day_detail_on_demand(&store, &day_client, "2025-06-03", day)
            .await
            .requests,
        0
    );
}

#[test]
fn a_later_estimate_never_stands_in_for_an_older_day() {
    let store = Store::open(":memory:").unwrap();
    store.put("fitness_age", "2025-06-10", "synthetic", &fitness_age_body()).unwrap();
    let before = screens::health(&store, "2025-06-03");
    assert_eq!(before.fitness_age.fitness_age, None);
    assert!(before.drivers.is_empty() && before.lever.is_none());
    // A day after it shows that estimate with its date, and the same estimate's drivers.
    let after = screens::health(&store, "2025-06-12");
    assert_eq!(after.fitness_age.fitness_age, Some(38.5));
    assert_eq!(after.fitness_age.recorded_on.as_deref(), Some("2025-06-10"));
    assert!(after.drivers.iter().any(|d| d.key == "vigorousMinutes" && d.target == Some(75.)));
}

#[tokio::test]
async fn opened_past_day_keeps_garmin_empty_answers() {
    let store = Store::open(":memory:").unwrap();
    profile(&store);
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let thread = std::thread::spawn(move || {
        for _ in 0..6 {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = [0; 4096];
            let _ = stream.read(&mut buf).unwrap();
            stream
                .write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}")
                .unwrap();
        }
    });
    let client = test_client(&url);
    let report = sync::day_detail_on_demand(&store, &client, "2025-06-03", &[DayDetailPart::Sleep]).await;
    assert_eq!(report.requests, 2);
    let day = screens::day_detail_parts("day").unwrap();
    let report = sync::day_detail_on_demand(&store, &client, "2025-06-03", day).await;
    thread.join().unwrap();
    assert_eq!(report.requests, 4);
    for kind in ["sleep", "hrv", "fitness_age", "stress", "summary", "health_status"] {
        assert_eq!(store.get(kind, "2025-06-03"), Some(serde_json::Value::Null));
    }
    let sleep = screens::sleep_view(&store, "2025-06-03");
    assert_eq!(sleep.detail.unavailable, vec![DayDetailPart::Sleep]);
    let health = screens::health(&store, "2025-06-03");
    assert_eq!(health.detail.unavailable, day);
    assert_eq!(health.fitness_age.fitness_age, None);
    // Empty answers are kept, as for an opened all-day heart-rate read.
    assert_eq!(
        sync::day_detail_on_demand(&store, &client, "2025-06-03", &[DayDetailPart::Sleep])
            .await
            .requests,
        0
    );
    assert_eq!(
        sync::day_detail_on_demand(&store, &client, "2025-06-03", day)
            .await
            .requests,
        0
    );
}

#[tokio::test]
async fn only_an_opened_past_day_is_fetched() {
    let store = Store::open(":memory:").unwrap();
    profile(&store);
    let client = test_client("http://127.0.0.1:1");
    let today = Local::now().date_naive().to_string();
    let future = (Local::now().date_naive() + Duration::days(1)).to_string();
    for date in [today.as_str(), future.as_str(), "2025-06-31", "", "2025-6-3"] {
        assert_eq!(
            sync::day_detail_on_demand(&store, &client, date, &[DayDetailPart::Sleep])
                .await
                .requests,
            0,
            "{date}"
        );
    }
    // Today's unsynced reads are not on-demand fetches: they wait for sync.
    let today_view = screens::sleep_view(&store, &today);
    assert!(today_view.detail.missing.is_empty());
    let past = screens::sleep_view(&store, "2025-06-03");
    assert_eq!(past.detail.missing, vec![DayDetailPart::Sleep]);
}

#[tokio::test]
async fn rate_limit_stops_opened_day_reads_until_backoff_expires() {
    let store = Store::open(":memory:").unwrap();
    profile(&store);
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let thread = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut buf = [0; 4096];
        let _ = stream.read(&mut buf).unwrap();
        stream
            .write_all(b"HTTP/1.1 429 Too Many Requests\r\nRetry-After: 600\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}")
            .unwrap();
    });
    let client = test_client(&url);
    let first = sync::day_detail_on_demand(&store, &client, "2025-06-03", &[DayDetailPart::Sleep]).await;
    thread.join().unwrap();
    assert_eq!(first.requests, 1);
    assert!(first.rate_limited);
    let second = sync::day_detail_on_demand(&store, &client, "2025-06-04", &[DayDetailPart::Sleep]).await;
    assert_eq!(second.requests, 0);
    assert!(second.rate_limited);
}

fn listed(store: &Store) {
    store
        .put(
            "listed",
            "9000000000042",
            "synthetic",
            &json!({
                "activityId": 9000000000042_u64,
                "activityName": "Old Ride",
                "activityType": {"typeKey": "cycling"},
                "startTimeLocal": "2023-04-11T07:00:00",
                "duration": 3600,
                "distance": 24000
            }),
        )
        .unwrap();
}

fn activity_body() -> serde_json::Value {
    json!({
        "activityId": 9000000000042_u64,
        "activityName": "Old Ride",
        "activityTypeDTO": {"typeKey": "cycling"},
        "summaryDTO": {
            "startTimeLocal": "2023-04-11T07:00:00",
            "duration": 3600,
            "distance": 24000,
            "activityTrainingLoad": 90,
            "averageHR": 130,
            "maxHR": 155
        }
    })
}

fn splits_body() -> serde_json::Value {
    json!({"lapDTOs": [{"distance": 5000, "duration": 750, "averageHR": 128}]})
}

fn zones_body() -> serde_json::Value {
    json!([
        {"zoneNumber": 1, "secsInZone": 100, "zoneLowBoundary": 90},
        {"zoneNumber": 2, "secsInZone": 200, "zoneLowBoundary": 110}
    ])
}

fn details_body() -> serde_json::Value {
    let start = 1681196400000_i64;
    json!({
        "metricDescriptors": [
            {"key": "directTimestamp", "metricsIndex": 0},
            {"key": "directHeartRate", "metricsIndex": 1},
            {"key": "directLatitude", "metricsIndex": 2},
            {"key": "directLongitude", "metricsIndex": 3},
            {"key": "directSpeed", "metricsIndex": 4, "unit": {"key": "mps"}}
        ],
        "activityDetailMetrics": [
            {"metrics": [start, 120, 45.1, -75.1, 6.0]},
            {"metrics": [start + 15_000, 132, 45.2, -75.2, 6.4]},
            {"metrics": [start + 30_000, 140, 45.3, -75.3, 6.8]}
        ]
    })
}

#[tokio::test]
async fn opened_older_workout_fetches_its_detail_once_and_keeps_it() {
    let store = Store::open(":memory:").unwrap();
    listed(&store);
    let before = screens::workout(&store, "9000000000042").unwrap();
    assert_eq!(before.detail.missing, vec![DayDetailPart::Workout]);
    assert!(before.route.is_empty() && before.heart_rate.is_empty());
    let (url, seen, thread) = server(4, |line| {
        if line.contains("/splits") {
            splits_body().to_string()
        } else if line.contains("/hrTimeInZones") {
            zones_body().to_string()
        } else if line.contains("/details") {
            details_body().to_string()
        } else {
            activity_body().to_string()
        }
    });
    let client = test_client(&url);
    let report = sync::workout_detail_on_demand(&store, &client, "9000000000042").await;
    thread.join().unwrap();
    assert_eq!(report.requests, 4);
    assert_eq!(report.stored, 4);
    {
        let lines = seen.lock().unwrap();
        assert!(lines[0].starts_with("GET /activity-service/activity/9000000000042 "));
        assert!(lines[1].starts_with("GET /activity-service/activity/9000000000042/splits"));
        assert!(lines[2].starts_with("GET /activity-service/activity/9000000000042/hrTimeInZones"));
        assert!(lines[3].starts_with("GET /activity-service/activity/9000000000042/details"));
    }
    let after = screens::workout(&store, "9000000000042").unwrap();
    assert!(after.detail.missing.is_empty());
    assert_eq!(after.route.len(), 3);
    assert_eq!(after.heart_rate.len(), 3);
    assert_eq!(after.splits.len(), 1);
    assert_eq!(after.zone_floors, vec![110.]);
    assert_eq!(
        sync::workout_detail_on_demand(&store, &client, "9000000000042")
            .await
            .requests,
        0
    );
    // Unknown and malformed ids never request anything.
    for id in ["", "abc", "123", "9000000000042/invalid"] {
        assert_eq!(
            sync::workout_detail_on_demand(&store, &client, id).await.requests,
            0,
            "{id}"
        );
    }
}
