use chrono::{Duration, Local, Utc};
use raring_core::{garmin::Client, screens, store::Store, sync, Tokens};
use serde_json::json;
use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::{Arc, Mutex},
};

fn client(url: &str) -> Client {
    let token = format!(
        "e30.{}.sig",
        base64::Engine::encode(
            &base64::engine::general_purpose::URL_SAFE_NO_PAD,
            json!({"exp":1900000000,"client_id":"TEST_DI"}).to_string()
        )
    );
    Client::with_urls(
        Tokens {
            access_token: token,
            refresh_token: "refresh".into(),
            client_id: "TEST_DI".into(),
            expires_at: Utc::now() + Duration::days(30),
        },
        url,
        url,
        std::time::Duration::ZERO,
    )
    .unwrap()
}

#[test]
fn heart_rate_preserves_gaps_and_separate_resting_value() {
    let store = Store::open(":memory:").unwrap();
    store.put("daily_hr", "2025-06-03", "synthetic", &json!({
        "startTimestampGMT":"2025-06-03T04:00:00.0",
        "startTimestampLocal":"2025-06-03T00:00:00.0",
        "endTimestampGMT":"2025-06-04T04:00:00.0",
        "endTimestampLocal":"2025-06-04T00:00:00.0",
        "restingHeartRate":53,"minHeartRate":48,"maxHeartRate":131,
        "heartRateValues":[[1748923200000_i64,58],[1748923320000_i64,null],[1748923560000_i64,0],[1748923680000_i64,71]]
    })).unwrap();
    store
        .put(
            "listed",
            "1",
            "synthetic",
            &json!({"activityId":1,"startTimeLocal":"2025-06-03T08:30:00","duration":3600}),
        )
        .unwrap();
    store
        .put(
            "listed",
            "2",
            "synthetic",
            &json!({"activityId":2,"startTimeLocal":"2025-06-02T23:50:00","duration":1800}),
        )
        .unwrap();
    let day = screens::heart_rate_day(&store, "2025-06-03");
    assert_eq!(day.duration_min, 1440.);
    assert_eq!(day.activities, vec![[0., 20.], [510., 570.]]);
    assert_eq!(day.resting, Some(53.));
    assert_eq!(day.minimum, Some(48.));
    assert_eq!(day.maximum, Some(131.));
    assert_eq!(day.timeline, vec![Some(58.), None, None, None, Some(71.)]);
    assert_eq!(screens::heart_rate_day(&store, "2025-06-02").resting, None);
    store.put("daily_hr", "2025-06-04", "synthetic", &json!({
        "startTimestampGMT":"2025-06-04T04:00:00.0",
        "heartRateValueDescriptors":[{"key":"heartrate","index":0},{"key":"timestamp","index":1}],
        "heartRateValues":[[59,1749009600000_i64]]
    })).unwrap();
    assert_eq!(
        screens::heart_rate_day(&store, "2025-06-04").timeline,
        vec![Some(59.)]
    );
}

fn millis(s: &str) -> i64 {
    chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S")
        .unwrap()
        .and_utc()
        .timestamp_millis()
}

#[test]
fn adjacent_charts_clip_morning_and_next_evening_sleep_separately() {
    let store = Store::open(":memory:").unwrap();
    for (date, next) in [("2025-06-03", "2025-06-04"), ("2025-06-04", "2025-06-05")] {
        store
            .put(
                "daily_hr",
                date,
                "synthetic",
                &json!({
                    "startTimestampGMT":format!("{date}T04:00:00"),
                    "startTimestampLocal":format!("{date}T00:00:00"),
                    "endTimestampGMT":format!("{next}T04:00:00"),
                    "endTimestampLocal":format!("{next}T00:00:00"),
                    "heartRateValues":[]
                }),
            )
            .unwrap();
    }
    for (wake, start, end) in [
        ("2025-06-03", "2025-06-03T03:00:00", "2025-06-03T11:00:00"),
        ("2025-06-04", "2025-06-04T03:00:00", "2025-06-04T11:00:00"),
    ] {
        store
            .put(
                "sleep",
                wake,
                "synthetic",
                &json!({"dailySleepDTO":{
                    "sleepStartTimestampGMT":millis(start),"sleepEndTimestampGMT":millis(end)
                }}),
            )
            .unwrap();
    }
    assert_eq!(
        screens::heart_rate_day(&store, "2025-06-03").sleep,
        vec![[0., 420.], [1380., 1440.]]
    );
    assert_eq!(
        screens::heart_rate_day(&store, "2025-06-04").sleep,
        vec![[0., 420.]]
    );
}

#[test]
fn spring_fall_and_partial_days_use_elapsed_utc_bounds() {
    let store = Store::open(":memory:").unwrap();
    for (date, next, start, end, minutes) in [
        (
            "2025-03-09",
            "2025-03-10",
            "2025-03-09T05:00:00",
            "2025-03-10T04:00:00",
            1380,
        ),
        (
            "2025-11-02",
            "2025-11-03",
            "2025-11-02T04:00:00",
            "2025-11-03T05:00:00",
            1500,
        ),
    ] {
        let first = millis(start);
        store
            .put(
                "daily_hr",
                date,
                "synthetic",
                &json!({
                    "startTimestampGMT":start,"startTimestampLocal":format!("{date}T00:00:00"),
                    "endTimestampGMT":end,"endTimestampLocal":format!("{next}T00:00:00"),
                    "heartRateValues":[[first,61],[first+(minutes-2)*60_000,75]]
                }),
            )
            .unwrap();
        let day = screens::heart_rate_day(&store, date);
        assert_eq!(day.duration_min, minutes as f64);
        assert_eq!(day.timeline.len(), minutes as usize / 2);
        assert_eq!(day.timeline.last(), Some(&Some(75.)));
    }
    // The 25th hour is real elapsed time, not a second rendering of a
    // guessed 00:xx local wall clock. Both overnight edges and a late
    // activity are positioned against absolute GMT instants.
    let start = millis("2025-11-02T04:00:00");
    store
        .put(
            "sleep",
            "2025-11-02",
            "synthetic",
            &json!({"dailySleepDTO":{
                "sleepStartTimestampGMT":start-60*60_000,"sleepEndTimestampGMT":start+480*60_000
            }}),
        )
        .unwrap();
    store
        .put(
            "sleep",
            "2025-11-03",
            "synthetic",
            &json!({"dailySleepDTO":{
                "sleepStartTimestampGMT":start+1440*60_000,"sleepEndTimestampGMT":start+1800*60_000
            }}),
        )
        .unwrap();
    store
        .put(
            "listed",
            "night",
            "synthetic",
            &json!({
                "activityId":"night","startTimeLocal":"2025-11-02T23:30:00",
                "startTimeGMT":"2025-11-03T04:30:00","duration":3600
            }),
        )
        .unwrap();
    store
        .put(
            "listed",
            "local-only",
            "synthetic",
            &json!({
                "activityId":"local-only","startTimeLocal":"2025-11-02T12:00:00","duration":600
            }),
        )
        .unwrap();
    let fall = screens::heart_rate_day(&store, "2025-11-02");
    assert_eq!(fall.sleep, vec![[0., 480.], [1440., 1500.]]);
    assert_eq!(fall.activities, vec![[1470., 1500.]]); // local-only on a shifted day is unknown.

    let today = Local::now().date_naive();
    let start = today
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc()
        .timestamp_millis();
    store
        .put(
            "daily_hr",
            &today.to_string(),
            "synthetic",
            &json!({
                "startTimestampGMT":start,"startTimestampLocal":format!("{today}T00:00:00"),
                "endTimestampGMT":start+600*60_000,
                "endTimestampLocal":format!("{}T00:00:00",today+Duration::days(1)),
                "heartRateValues":[[start,52],[start+598*60_000,66]]
            }),
        )
        .unwrap();
    let partial = screens::heart_rate_day(&store, &today.to_string());
    assert_eq!(partial.duration_min, 1440.);
    assert_eq!(partial.timeline.len(), 300);
    assert_eq!(partial.timeline.last(), Some(&Some(66.)));
}

#[tokio::test]
async fn opened_old_day_is_fetched_once_and_kept_even_if_empty() {
    let store = Store::open(":memory:").unwrap();
    store
        .put(
            "profile",
            "me",
            "synthetic",
            &json!({"displayName":"Synthetic"}),
        )
        .unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let seen = Arc::new(Mutex::new(String::new()));
    let requests = Arc::clone(&seen);
    let thread = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut buf = [0; 4096];
        let n = stream.read(&mut buf).unwrap();
        *requests.lock().unwrap() = String::from_utf8_lossy(&buf[..n]).into_owned();
        stream
            .write_all(
                b"HTTP/1.1 404 Not Found\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}",
            )
            .unwrap();
    });
    let client = client(&url);
    let first = sync::heart_rate_on_demand(&store, &client, "2025-06-03").await;
    thread.join().unwrap();
    assert_eq!(first.requests, 1);
    assert!(seen
        .lock()
        .unwrap()
        .contains("/dailyHeartRate/Synthetic?date=2025-06-03"));
    assert_eq!(
        store.get("daily_hr", "2025-06-03"),
        Some(serde_json::Value::Null)
    );
    assert_eq!(
        sync::heart_rate_on_demand(&store, &client, "2025-06-03")
            .await
            .requests,
        0
    );
    assert_eq!(
        sync::heart_rate_on_demand(&store, &client, "2025-06-04/invalid")
            .await
            .requests,
        0
    );
    let future = (Local::now().date_naive() + Duration::days(1)).to_string();
    assert_eq!(
        sync::heart_rate_on_demand(&store, &client, &future)
            .await
            .requests,
        0
    );
}

#[tokio::test]
async fn rate_limit_stops_on_demand_reads_until_backoff_expires() {
    let store = Store::open(":memory:").unwrap();
    store
        .put(
            "profile",
            "me",
            "synthetic",
            &json!({"displayName":"Synthetic"}),
        )
        .unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let thread = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut buf = [0; 4096];
        let _ = stream.read(&mut buf).unwrap();
        stream.write_all(b"HTTP/1.1 429 Too Many Requests\r\nRetry-After: 600\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}").unwrap();
    });
    let client = client(&url);
    let first = sync::heart_rate_on_demand(&store, &client, "2025-06-03").await;
    thread.join().unwrap();
    assert_eq!(first.requests, 1);
    assert!(first.rate_limited);
    assert!(
        store
            .cursor("backoff_until")
            .unwrap()
            .parse::<i64>()
            .unwrap()
            > Utc::now().timestamp()
    );
    let second = sync::heart_rate_on_demand(&store, &client, "2025-06-02").await;
    assert_eq!(second.requests, 0);
    assert!(second.rate_limited);
}
