use super::*;
use serde_json::json;
use std::{io::{Read, Write}, sync::{Arc, Mutex}};

fn mock(replies: Vec<(u16, &'static str)>) -> (Client, Arc<Mutex<Vec<String>>>, std::thread::JoinHandle<()>) {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let seen = Arc::new(Mutex::new(Vec::new()));
    let log = seen.clone();
    let thread = std::thread::spawn(move || {
        for (code, body) in replies {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = [0; 4096];
            let n = stream.read(&mut buf).unwrap();
            log.lock().unwrap().push(String::from_utf8_lossy(&buf[..n]).into_owned());
            let response = format!("HTTP/1.1 {code} OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len());
            stream.write_all(response.as_bytes()).unwrap();
        }
    });
    let client = Client::with_urls(
        Tokens { access_token: "test".into(), refresh_token: "test".into(), client_id: "TEST".into(), expires_at: Utc::now() + Duration::days(30) },
        &url, &url, std::time::Duration::ZERO,
    ).unwrap();
    (client, seen, thread)
}

#[tokio::test]
async fn absence_is_cached_six_hours_and_new_schedule_replaces_old() {
    let store = Store::open(":memory:").unwrap();
    let now = Utc::now();
    let day = now.date_naive();
    let (client, seen, thread) = mock(vec![(404,"{}"),(200,"[]"),(404,"{}")]);
    let mut report = SyncReport::default();
    sync_planned(&store, &client, &mut report, day, 3, now).await;
    thread.join().unwrap();
    assert_eq!(report.requests, 3);
    assert_eq!(seen.lock().unwrap().len(), 3);
    assert!(store.get("coach_plans", "current").unwrap().is_null());
    assert_eq!(store.get("scheduled_workouts", "current").unwrap()["entries"], json!([]));
    assert!(store.get("upcoming_events", "current").unwrap().is_null());
    let mut again = SyncReport::default();
    sync_planned(&store, &client, &mut again, day, 3, now + Duration::hours(1)).await;
    assert_eq!(again.requests, 0);
    let (client, _, thread) = mock(vec![(200,"{\"trainingPlanList\":[]}"),(200,"[{\"calendarDate\":\"2026-10-21\",\"workoutName\":\"Easy Run\"}]"),(200,"[]")]);
    let mut changed = SyncReport::default();
    sync_planned(&store, &client, &mut changed, day, 3, now + Duration::hours(7)).await;
    thread.join().unwrap();
    assert_eq!(changed.requests, 3);
    assert_eq!(store.get("scheduled_workouts", "current").unwrap()["entries"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn coach_tasks_are_deferred_even_when_a_plan_id_exists() {
    let store = Store::open(":memory:").unwrap();
    let now = Utc::now();
    let (client, seen, thread) = mock(vec![(200,"{\"trainingPlanList\":[{\"trainingPlanId\":\"42\"}]}"),(200,"[]"),(200,"[]")]);
    let mut report = SyncReport::default();
    sync_planned(&store, &client, &mut report, now.date_naive(), 3, now).await;
    thread.join().unwrap();
    assert_eq!(report.requests, 3);
    let requests = seen.lock().unwrap();
    assert!(requests.iter().all(|r| r.starts_with("GET ") && !r.contains("/tasks/")));
}

#[tokio::test]
async fn rate_limit_stops_planned_fetch_and_sets_shared_backoff() {
    let store = Store::open(":memory:").unwrap();
    let now = Utc::now();
    let (client, seen, thread) = mock(vec![(429,"{}")]);
    let mut report = SyncReport::default();
    sync_planned(&store, &client, &mut report, now.date_naive(), 20, now).await;
    thread.join().unwrap();
    assert_eq!(seen.lock().unwrap().len(), 1);
    assert!(report.rate_limited);
    assert!(store.cursor("backoff_until").is_some());
}

const ADAPTIVE_PLANS: &str = r#"{"trainingPlanList":[{"trainingPlanId":42,"trainingPlanCategory":"FBT_ADAPTIVE","trainingStatus":{"statusKey":"Scheduled"}}]}"#;
const ADAPTIVE: &str = r#"{"trainingPlanId":42,"trainingPlanCategory":"FBT_ADAPTIVE","taskList":[{"calendarDate":"2026-10-20","workoutOrder":1,"taskWorkout":{"workoutName":"Base","restDay":false,"sportType":{"sportTypeKey":"other"}}}]}"#;

#[tokio::test]
async fn an_adaptive_coach_plan_is_read_from_its_own_endpoint_within_the_budget() {
    let store = Store::open(":memory:").unwrap();
    let now = Utc::now();
    let (client, seen, thread) = mock(vec![(200, ADAPTIVE_PLANS), (200, "[]"), (200, "[]"), (200, ADAPTIVE)]);
    let mut report = SyncReport::default();
    sync_planned(&store, &client, &mut report, now.date_naive(), 4, now).await;
    thread.join().unwrap();
    assert_eq!(report.requests, 4);
    assert!(seen.lock().unwrap()[3].starts_with("GET /trainingplan-service/trainingplan/fbt-adaptive/42 "));
    assert_eq!(store.get("coach_adaptive", "current").unwrap()["taskList"].as_array().unwrap().len(), 1);
    // Within six hours nothing is read again.
    let mut again = SyncReport::default();
    sync_planned(&store, &client, &mut again, now.date_naive(), 4, now + Duration::hours(1)).await;
    assert_eq!(again.requests, 0);
}

#[tokio::test]
async fn the_budget_holds_and_a_bad_answer_keeps_the_last_good_plan() {
    let store = Store::open(":memory:").unwrap();
    let now = Utc::now();
    let (client, seen, thread) = mock(vec![(200, ADAPTIVE_PLANS), (200, "[]"), (200, "[]")]);
    let mut report = SyncReport::default();
    sync_planned(&store, &client, &mut report, now.date_naive(), 3, now).await;
    thread.join().unwrap();
    assert_eq!((report.requests, seen.lock().unwrap().len()), (3, 3), "the budget is spent: no fourth request");
    assert!(store.get("coach_adaptive", "current").is_none());
    store.put("coach_adaptive", "current", &now.to_rfc3339(), &serde_json::from_str::<Value>(ADAPTIVE).unwrap()).unwrap();
    for bad in [r#"{"trainingPlanId":42,"trainingPlanCategory":"FBT_ADAPTIVE","taskList":[]}"#, r#"{"trainingPlanId":43,"trainingPlanCategory":"FBT_ADAPTIVE","taskList":[{"calendarDate":"2026-10-20","taskWorkout":{}}]}"#, r#"{"taskList":null}"#] {
        let (client, _, thread) = mock(vec![(200, bad)]);
        let mut r = SyncReport::default();
        sync_coach_adaptive(&store, &client, &mut r, 1, now).await;
        thread.join().unwrap();
        assert_eq!(r.data_failures, 1, "{bad}");
        assert_eq!(store.get("coach_adaptive", "current").unwrap()["taskList"].as_array().unwrap().len(), 1, "the last good plan stays");
        let _ = store.set_cursor("retry:coach_adaptive:42", "0");
    }
}
