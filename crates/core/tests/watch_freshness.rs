use chrono::{Local, Utc};
use raring_core::{
    garmin::{Client, Tokens},
    screens,
    store::Store,
    sync,
};
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};

const DAY: &str = "2025-02-04";

#[test]
fn scrubbed_device_response_shapes_select_the_watch() {
    let store = Store::open(":memory:").unwrap();
    let list: Value = serde_json::from_str(include_str!("fixtures/devices.json")).unwrap();
    let details: Value =
        serde_json::from_str(include_str!("fixtures/device_details.json")).unwrap();
    put(&store, "devices", "me", list);
    put(
        &store,
        "device_detail",
        "selected",
        details[1]["detail"].clone(),
    );
    assert_eq!(
        screens::today(&store, DAY).watch_last_upload.as_deref(),
        Some("2025-02-04T09:00:00.000Z")
    );
}

fn put(store: &Store, kind: &str, key: &str, value: Value) {
    store.put(kind, key, "synthetic", &value).unwrap();
}

#[test]
fn selected_watch_capabilities_and_upload_are_not_inferred_from_accessories() {
    let store = Store::open(":memory:").unwrap();
    put(
        &store,
        "devices",
        "me",
        json!([
            {"deviceId": 1, "primaryActivityTrackerIndicator": false,
             "sleepSkinTemperatureCapable": true, "hrvStatusCapable": true},
            {"deviceId": 2, "primaryActivityTrackerIndicator": true,
             "sleepSkinTemperatureCapable": false, "hrvStatusCapable": false,
             "trainingReadinessCapable": false, "sleepScoreCapable": false},
            {"deviceId": 3, "primaryActivityTrackerIndicator": false}
        ]),
    );
    put(
        &store,
        "device_detail",
        "selected",
        json!({"deviceId":2,"lastUploadTimestamp":1738659600000_i64}),
    );
    let view = screens::today(&store, DAY);
    assert_eq!(
        view.watch_last_upload.as_deref(),
        Some("2025-02-04T09:00:00.000Z")
    );
    for key in ["skinTemp", "hrv", "readiness", "sleep"] {
        assert!(view.missing_metric_reasons.contains_key(key), "{key}");
    }
    put(
        &store,
        "hrv",
        DAY,
        json!({"hrvSummary":{"lastNightAvg":52}}),
    );
    assert!(!screens::today(&store, DAY)
        .missing_metric_reasons
        .contains_key("hrv"));
    // A newly selected watch must never display the previous watch's timestamp.
    put(
        &store,
        "devices",
        "me",
        json!([{"deviceId": 1, "primaryActivityTrackerIndicator": true}]),
    );
    let view = screens::today(&store, DAY);
    assert_eq!(view.watch_last_upload, None);
    assert!(view.missing_metric_reasons.is_empty());
    put(
        &store,
        "device_detail",
        "selected",
        json!({"deviceId":1,"lastUploadTimestamp":null}),
    );
    assert_eq!(screens::today(&store, DAY).watch_last_upload, None);
    put(
        &store,
        "devices",
        "me",
        json!([{"deviceId":3},{"deviceId":4}]),
    );
    assert!(screens::today(&store, DAY)
        .missing_metric_reasons
        .is_empty());
}

#[test]
fn only_one_clearly_capable_watch_is_a_fallback() {
    let store = Store::open(":memory:").unwrap();
    put(
        &store,
        "devices",
        "me",
        json!([
            {"deviceId": 11, "sleepScoreCapable": true}, {"deviceId": 12}
        ]),
    );
    put(
        &store,
        "device_detail",
        "selected",
        json!({"deviceId": 11, "lastUploadTimestamp": 1738659600000_i64}),
    );
    assert!(screens::today(&store, DAY).watch_last_upload.is_some());
    put(
        &store,
        "devices",
        "me",
        json!([
            {"deviceId": 11, "sleepScoreCapable": true}, {"deviceId": 12, "hrvStatusCapable": true}
        ]),
    );
    assert!(screens::today(&store, DAY).watch_last_upload.is_none());
}

fn mock(
    replies: Vec<(u16, String)>,
) -> (String, Arc<Mutex<Vec<String>>>, std::thread::JoinHandle<()>) {
    use std::io::{Read, Write};
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let seen = Arc::new(Mutex::new(Vec::new()));
    let log = Arc::clone(&seen);
    let handle = std::thread::spawn(move || {
        for (status, body) in replies {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(std::time::Duration::from_secs(4)))
                .unwrap();
            let mut bytes = [0; 8192];
            let length = stream.read(&mut bytes).unwrap();
            log.lock()
                .unwrap()
                .push(String::from_utf8_lossy(&bytes[..length]).into_owned());
            let response = format!("HTTP/1.1 {status} OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len());
            stream.write_all(response.as_bytes()).unwrap();
        }
    });
    (url, seen, handle)
}
fn client(url: &str) -> Client {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    let token = format!(
        "e30.{}.sig",
        URL_SAFE_NO_PAD.encode(json!({"exp":1900000000,"client_id":"TEST_DI"}).to_string())
    );
    Client::with_urls(
        Tokens {
            access_token: token,
            refresh_token: "refresh".into(),
            client_id: "TEST_DI".into(),
            expires_at: Utc::now() + chrono::Duration::days(30),
        },
        url,
        url,
        std::time::Duration::ZERO,
    )
    .unwrap()
}
fn profile(store: &Store) {
    put(store, "profile", "me", json!({"displayName":"Synthetic"}));
}

#[tokio::test]
async fn list_is_hourly_detail_each_run_and_budget_is_strict() {
    let store = Store::open(":memory:").unwrap();
    profile(&store);
    let list = json!([{"deviceId": 57, "primaryActivityTrackerIndicator": true}]);
    let (url, seen, handle) = mock(vec![
        (200, list.to_string()),
        (
            200,
            json!({"deviceId":57,"lastUploadTimestamp":1738659600000_i64}).to_string(),
        ),
        (
            200,
            json!({"deviceId":57,"lastUploadTimestamp":null}).to_string(),
        ),
    ]);
    let client = client(&url);
    let first = sync::sync(&store, &client, Local::now(), 2).await;
    assert_eq!(first.requests, 2);
    assert_eq!(store.get("devices", "me"), Some(list));
    assert!(store.get("device_detail", "57").is_none());
    assert!(screens::today(&store, DAY).watch_last_upload.is_some());
    let second = sync::sync(&store, &client, Local::now(), 1).await;
    assert_eq!(second.requests, 1);
    assert!(screens::today(&store, DAY).watch_last_upload.is_none());
    let none = sync::sync(&store, &client, Local::now(), 0).await;
    assert_eq!(none.requests, 0);
    handle.join().unwrap();
    let seen = seen.lock().unwrap();
    assert!(seen[0].contains("/deviceregistration/devices"));
    assert!(seen[1].contains("/user-device/57"));
    assert!(seen[2].contains("/user-device/57"));
}

#[tokio::test]
async fn rate_limit_stops_before_detail_and_retains_previous_snapshot() {
    let store = Store::open(":memory:").unwrap();
    profile(&store);
    put(
        &store,
        "devices",
        "me",
        json!([{"deviceId":57,"primaryActivityTrackerIndicator":true}]),
    );
    put(
        &store,
        "device_detail",
        "selected",
        json!({"deviceId":57,"lastUploadTimestamp":1738659600000_i64}),
    );
    let (url, seen, handle) = mock(vec![(429, "{}".into())]);
    let report = sync::sync(&store, &client(&url), Local::now(), 5).await;
    handle.join().unwrap();
    assert!(report.rate_limited);
    assert_eq!(report.requests, 1);
    assert_eq!(seen.lock().unwrap().len(), 1);
    assert!(screens::today(&store, DAY).watch_last_upload.is_some());
}
