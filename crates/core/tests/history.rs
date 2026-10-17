use chrono::{Duration, NaiveDate, Utc};
use raring_core::{
    garmin::{self, Client, Tokens},
    store::Store,
    sync::{history, history_progress, SyncReport},
};
use serde_json::{json, Value};
use std::{
    io::{Read, Write},
    sync::{Arc, Mutex},
};

// Local-only HTTP server; records request lines and returns scripted responses. No Garmin traffic.
fn mock(
    replies: Vec<(u16, String)>,
) -> (Client, Arc<Mutex<Vec<String>>>, std::thread::JoinHandle<()>) {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let seen = Arc::new(Mutex::new(vec![]));
    let log = seen.clone();
    let thread = std::thread::spawn(move || {
        for (status, body) in replies {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(std::time::Duration::from_secs(3)))
                .unwrap();
            let mut buf = [0u8; 8192];
            let n = stream.read(&mut buf).unwrap();
            let line = String::from_utf8_lossy(&buf[..n])
                .lines()
                .next()
                .unwrap_or_default()
                .to_owned();
            log.lock().unwrap().push(line);
            let text = format!("HTTP/1.1 {status} OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len());
            stream.write_all(text.as_bytes()).unwrap();
        }
    });
    let tokens = Tokens {
        access_token: "synthetic-access".into(),
        refresh_token: "synthetic-refresh".into(),
        client_id: "TEST_DI".into(),
        expires_at: Utc::now() + Duration::days(30),
    };
    let client = Client::with_urls(tokens, &url, &url, std::time::Duration::ZERO).unwrap();
    (client, seen, thread)
}
fn today() -> NaiveDate {
    NaiveDate::from_ymd_opt(2025, 6, 30).unwrap()
}
async fn run(
    store: &Store,
    replies: Vec<(u16, String)>,
    budget: usize,
) -> (SyncReport, Vec<String>) {
    let (client, seen, thread) = mock(replies);
    let mut report = SyncReport::default();
    history(store, &client, &mut report, today(), budget, Utc::now()).await;
    thread.join().unwrap();
    let seen = seen.lock().unwrap().clone();
    (report, seen)
}
fn cover(store: &Store, name: &str, lo: &str, hi: &str) {
    store.set_cursor(&format!("history:{name}:lo"), lo).unwrap();
    store.set_cursor(&format!("history:{name}:hi"), hi).unwrap();
}

#[tokio::test]
async fn summaries_walk_back_a_year_per_request_and_store_one_row_per_day() {
    let s = Store::open(":memory:").unwrap();
    let year = json!({"data":[{"calendarDate":"2025-03-01","stress":{"avgLevel":30}},{"calendarDate":"2024-05-01"}]});
    let (report, seen) = run(&s, vec![(200, year.to_string()), (204, String::new())], 2).await;
    assert_eq!(report.requests, 2);
    assert_eq!(report.stored, 2);
    assert!(
        seen[0].starts_with(
            "GET /usersummary-service/v2/dailysummaries?startDate=2024-04-02&endDate=2025-04-01 "
        ),
        "{}",
        seen[0]
    );
    assert!(
        seen[1].contains("startDate=2023-04-03&endDate=2024-04-01"),
        "{}",
        seen[1]
    );
    assert_eq!(
        s.get("daily_summary", "2025-03-01").unwrap()["stress"]["avgLevel"],
        30
    );
    assert_eq!(
        s.cursor("history:summary:lo").as_deref(),
        Some("2023-04-03")
    );
    assert_eq!(
        s.cursor("history:summary:hi").as_deref(),
        Some("2025-04-01")
    );
    assert_eq!(
        history_progress(&s, today()),
        (Some("2024-05-01".into()), false)
    );
}

#[tokio::test]
async fn days_that_left_the_recent_window_are_fetched_before_older_ones() {
    let s = Store::open(":memory:").unwrap();
    cover(&s, "summary", "2000-01-01", "2025-01-01");
    let (_, seen) = run(&s, vec![(200, "{\"data\":[]}".into())], 1).await;
    assert!(
        seen[0].contains("startDate=2025-01-02&endDate=2025-04-01"),
        "{}",
        seen[0]
    );
    assert_eq!(
        s.cursor("history:summary:hi").as_deref(),
        Some("2025-04-01")
    );
}

#[tokio::test]
async fn activity_ranges_page_until_a_short_page_and_resume_at_the_offset() {
    let s = Store::open(":memory:").unwrap();
    cover(&s, "summary", "2000-01-01", "2025-04-01");
    let page: Vec<Value> = (0..100)
        .map(|i| json!({"activityId": 1000 + i, "startTimeLocal": "2025-03-01 07:00:00"}))
        .collect();
    let (_, seen) = run(&s, vec![(200, Value::Array(page).to_string())], 1).await;
    assert!(seen[0].starts_with("GET /activitylist-service/activities/search/activities?startDate=2024-04-02&endDate=2025-04-01&start=0&limit=100 "), "{}", seen[0]);
    assert!(s.cursor("history:activities:lo").is_none());

    let rest = json!([{"activityId": 7, "startTimeLocal": "2024-06-01 07:00:00"}]);
    let (_, seen) = run(&s, vec![(200, rest.to_string())], 1).await;
    assert!(seen[0].contains("start=100&limit=100"), "{}", seen[0]);
    assert_eq!(
        s.cursor("history:activities:lo").as_deref(),
        Some("2024-04-02")
    );
    assert_eq!(
        s.get("listed", "1099").unwrap()["startTimeLocal"],
        "2025-03-01 07:00:00"
    );
    assert_eq!(s.oldest_activity().as_deref(), Some("2024-06-01 07:00:00"));
}

#[tokio::test]
async fn wellness_ranges_stop_at_the_oldest_summary_and_keep_existing_days() {
    let s = Store::open(":memory:").unwrap();
    cover(&s, "summary", "2000-01-01", "2025-04-01");
    cover(&s, "activities", "2000-01-01", "2025-04-01");
    s.put(
        "daily_summary",
        "2025-01-15",
        "now",
        &json!({"calendarDate":"2025-01-15"}),
    )
    .unwrap();
    s.put("readiness", "2025-02-01", "now", &json!([{"score": 1}]))
        .unwrap();
    let readiness = json!([
        {"calendarDate":"2025-02-01","score":50},
        {"calendarDate":"2025-03-02","score":60},
        {"calendarDate":"2025-03-02","score":70}
    ]);
    let replies = vec![
        (200, readiness.to_string()),
        (
            200,
            json!({"hrvSummaries":[{"calendarDate":"2025-03-01","lastNightAvg":40}]}).to_string(),
        ),
        (
            200,
            json!([{"generic":null,"cycling":{"calendarDate":"2025-03-03","vo2MaxValue":50}}])
                .to_string(),
        ),
        (
            200,
            json!([{"calendarDate":"2025-03-10","sleepTimeSeconds":25000}]).to_string(),
        ),
        (204, String::new()),
        (200, "[]".into()),
    ];
    let (report, seen) = run(&s, replies, 10).await;
    assert_eq!(report.requests, 6);
    assert!(
        seen[0]
            .starts_with("GET /metrics-service/metrics/trainingreadiness/2025-01-15/2025-04-01 "),
        "{}",
        seen[0]
    );
    assert!(
        seen[1].starts_with("GET /hrv-service/hrv/daily/2025-01-15/2025-04-01 "),
        "{}",
        seen[1]
    );
    assert!(
        seen[2].starts_with("GET /metrics-service/metrics/maxmet/daily/2025-01-15/2025-04-01 "),
        "{}",
        seen[2]
    );
    assert!(seen[3].starts_with("GET /sleep-service/sleep/dailySleepSummariesByDate?startDate=2025-03-05&endDate=2025-04-01 "), "{}", seen[3]);
    assert!(
        seen[4].contains("startDate=2025-02-05&endDate=2025-03-04"),
        "{}",
        seen[4]
    );
    assert!(
        seen[5].contains("startDate=2025-01-15&endDate=2025-02-04"),
        "{}",
        seen[5]
    );
    assert_eq!(
        s.get("readiness", "2025-02-01").unwrap(),
        json!([{"score": 1}])
    );
    assert_eq!(
        s.get("readiness", "2025-03-02")
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        s.get("hrv_summary", "2025-03-01").unwrap()["lastNightAvg"],
        40
    );
    assert_eq!(
        s.get("max_metrics", "2025-03-03").unwrap()[0]["cycling"]["vo2MaxValue"],
        50
    );
    assert_eq!(
        s.get("sleep_summary", "2025-03-10").unwrap()["sleepTimeSeconds"],
        25000
    );
    assert_eq!(
        history_progress(&s, today()),
        (Some("2025-01-15".into()), true)
    );
}

#[tokio::test]
async fn a_failed_range_is_retried_on_a_later_run_without_moving_the_cursor() {
    let s = Store::open(":memory:").unwrap();
    let (report, seen) = run(&s, vec![(500, "{}".into()), (204, String::new())], 2).await;
    assert_eq!(report.data_failures, 1);
    assert!(seen[0].contains("dailysummaries"));
    assert!(s.cursor("history:summary:lo").is_none());
    // Activities still advance while summaries wait for their retry hour.
    assert!(seen[1].contains("activitylist-service"), "{}", seen[1]);
    let (_, seen) = run(&s, vec![(204, String::new())], 1).await;
    assert!(seen[0].contains("activitylist-service"), "{}", seen[0]);
    assert!(s.cursor("history:summary:lo").is_none());
}

#[tokio::test]
async fn range_keys_are_validated_before_any_request() {
    let (client, seen, thread) = mock(vec![]);
    for key in [
        "2025-01-01",
        "2025-01-01:x",
        "2025-01-01:2025-02-01:",
        "2025-01-01:2025-02-01:1:2",
        "../x:2025-01-01",
    ] {
        assert!(
            matches!(
                client.endpoint("summary_range", key, None).await,
                Err(garmin::Error::Decode)
            ),
            "{key}"
        );
    }
    thread.join().unwrap();
    assert!(seen.lock().unwrap().is_empty());
}
