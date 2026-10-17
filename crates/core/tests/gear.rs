use chrono::Utc;
use raring_core::{
    garmin::{Client, Tokens},
    screens,
    store::Store,
    sync,
};
use serde_json::{json, Value};
use std::{
    io::{Read, Write},
    sync::{Arc, Mutex},
};

fn gear(id: &str, status: &str, distance: Value, threshold: Value) -> Value {
    json!({"uuid":id,"name":format!("Sample {id}"),"gearType":"SHOES","status":status,
        "usageType":"DISTANCE","distanceUsedMeters":distance,"maxUsageDistanceMeters":threshold})
}
#[test]
fn gear_distance_units_missing_zero_retired_and_complete_replacement() {
    let s = Store::open(":memory:").unwrap();
    assert!(screens::training(&s, "2025-01-02").gear.is_none());
    let rows = json!([
        gear("a", "ACTIVE", json!(0), json!(650000)),
        gear("b", "RETIRED", Value::Null, Value::Null),
        gear("c", "ACTIVE", json!(5000), json!(0))
    ]);
    s.put("gear", "me", "2025-01-02T09:00:00Z", &rows).unwrap();
    let view = screens::training(&s, "2025-01-02").gear.unwrap();
    assert_eq!(view.unit, "m");
    assert_eq!(view.fetched_at, "2025-01-02T09:00:00Z");
    assert_eq!(view.items[0].distance, Some(0.));
    assert_eq!(view.items[1].status, "Retired");
    assert_eq!(view.items[1].distance, None);
    assert_eq!(view.items[1].threshold, None);
    assert_eq!(view.items[2].threshold, None);
    s.put(
        "profile_settings",
        "me",
        "now",
        &json!({"measurementSystem":"metric"}),
    )
    .unwrap();
    let view = screens::training(&s, "2025-01-02").gear.unwrap();
    assert_eq!(view.unit, "km");
    assert_eq!(view.items[0].threshold, Some(650.));
    s.put(
        "profile_settings",
        "me",
        "now",
        &json!({"measurementSystem":"statute_us"}),
    )
    .unwrap();
    let view = screens::training(&s, "2025-01-02").gear.unwrap();
    assert_eq!(view.unit, "mi");
    assert!((view.items[2].distance.unwrap() - 3.10685596).abs() < 0.00001);
    s.put(
        "gear",
        "me",
        "2025-01-03T09:00:00Z",
        &json!([gear("b", "RETIRED", json!(12000), Value::Null)]),
    )
    .unwrap();
    let view = screens::training(&s, "2025-01-02").gear.unwrap();
    assert_eq!(view.items.len(), 1);
    assert_eq!(view.items[0].id, "b");
    s.put("gear", "me", "2025-01-04T09:00:00Z", &json!([]))
        .unwrap();
    assert!(screens::training(&s, "2025-01-02").gear.is_none());
}

// Network responses are scripted locally: never put a real Garmin response in fixtures.
#[tokio::test]
async fn gear_sync_is_budgeted_daily_and_does_not_erase_on_failure_or_404() {
    let store = Store::open(":memory:").unwrap();
    let today = "2025-01-02";
    store
        .put(
            "gear",
            "me",
            "2025-01-01T09:00:00Z",
            &json!([gear("old", "RETIRED", json!(0), Value::Null)]),
        )
        .unwrap();
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let seen = Arc::new(Mutex::new(vec![]));
    let log = seen.clone();
    let replies = [
        (500, "{}".to_owned()),
        (404, "{}".to_owned()),
        (200, "{}".to_owned()),
        (
            200,
            json!([gear("new", "ACTIVE", json!(1200), Value::Null)]).to_string(),
        ),
        (200, "[]".to_owned()),
    ];
    let server = std::thread::spawn(move || {
        for (code, body) in replies {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = [0; 8192];
            let n = stream.read(&mut buf).unwrap();
            log.lock()
                .unwrap()
                .push(String::from_utf8_lossy(&buf[..n]).into_owned());
            let reply=format!("HTTP/1.1 {code} OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len());
            stream.write_all(reply.as_bytes()).unwrap();
        }
    });
    let client = Client::with_urls(
        Tokens {
            access_token: "synthetic".into(),
            refresh_token: "synthetic".into(),
            client_id: "synthetic".into(),
            expires_at: Utc::now() + chrono::Duration::days(1),
        },
        &url,
        &url,
        std::time::Duration::ZERO,
    )
    .unwrap();
    let mut report = sync::SyncReport::default();
    sync::sync_gear(&store, &client, &mut report, 0, Utc::now()).await;
    assert_eq!(report.requests, 0);
    let mut report = sync::SyncReport::default();
    sync::sync_gear(&store, &client, &mut report, 1, Utc::now()).await;
    assert_eq!(report.requests, 1);
    assert_eq!(report.data_failures, 1);
    assert_eq!(
        screens::training(&store, today).gear.unwrap().items[0].id,
        "old"
    );
    let mut report = sync::SyncReport::default();
    sync::sync_gear(&store, &client, &mut report, 1, Utc::now()).await;
    assert_eq!(report.requests, 0); // retry hour
    store.set_cursor("retry:gear:me", "0").unwrap();
    let mut report = sync::SyncReport::default();
    sync::sync_gear(&store, &client, &mut report, 1, Utc::now()).await;
    assert_eq!(report.requests, 1); // 404, still old
    assert_eq!(
        screens::training(&store, today).gear.unwrap().items[0].id,
        "old"
    );
    let mut report = sync::SyncReport::default();
    sync::sync_gear(&store, &client, &mut report, 1, Utc::now()).await;
    assert_eq!(report.requests, 0);
    store.set_cursor("gear_checked", "0").unwrap();
    let mut report = sync::SyncReport::default();
    sync::sync_gear(&store, &client, &mut report, 1, Utc::now()).await;
    assert_eq!(report.requests, 1);
    assert_eq!(report.data_failures, 1);
    assert_eq!(
        screens::training(&store, today).gear.unwrap().items[0].id,
        "old"
    );
    let mut report = sync::SyncReport::default();
    sync::sync_gear(&store, &client, &mut report, 1, Utc::now()).await;
    assert_eq!(report.requests, 0);
    store.set_cursor("gear_checked", "0").unwrap();
    let mut report = sync::SyncReport::default();
    sync::sync_gear(&store, &client, &mut report, 1, Utc::now()).await;
    assert_eq!(report.requests, 1);
    assert_eq!(
        screens::training(&store, today).gear.unwrap().items[0].id,
        "new"
    );
    store.set_cursor("gear_checked", "0").unwrap();
    let mut report = sync::SyncReport::default();
    sync::sync_gear(&store, &client, &mut report, 1, Utc::now()).await;
    assert_eq!(report.requests, 1);
    assert!(screens::training(&store, today).gear.is_none());
    server.join().unwrap();
    assert!(seen
        .lock()
        .unwrap()
        .iter()
        .all(|s| s.starts_with("GET /gear-service/gear/v2/list HTTP/")));
}
