use chrono::{Duration, Utc};
use raring_core::garmin::{self, Client, Tokens};
use std::{io::{Read, Write}, sync::{Arc, Mutex}};

#[tokio::test]
async fn read_routes_use_bounded_reference_arguments_and_404_is_absent() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let seen = Arc::new(Mutex::new(Vec::new()));
    let log = seen.clone();
    let thread = std::thread::spawn(move || {
        for (code, body) in [(200,"{\"trainingPlanList\":[]}"),(200,"[]"),(200,"[]"),(404,"{}")]
        {
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
    client.endpoint("coach_plans", "2026-10-20", None).await.unwrap();
    client.endpoint("scheduled_workouts", "2026-10-20:2026-11-03", None).await.unwrap();
    client.endpoint("upcoming_events", "all", None).await.unwrap();
    assert!(matches!(client.endpoint("coach_tasks", "42", None).await, Err(garmin::Error::NotFound)));
    assert!(matches!(client.endpoint("coach_tasks", "../42", None).await, Err(garmin::Error::Decode)));
    assert!(matches!(client.endpoint("scheduled_workouts", "2026-10-20:2026-12-03", None).await, Err(garmin::Error::Decode)));
    thread.join().unwrap();
    let requests = seen.lock().unwrap();
    assert!(requests[0].starts_with("GET /trainingplan-service/trainingplan/active-plans?startDate=2026-10-20 "));
    assert!(requests[1].starts_with("GET /workout-service/schedule/workoutScheduleByPeriod?startDate=2026-10-20&endDate=2026-11-03 "));
    assert!(requests[2].starts_with("GET /calendar-service/events/upcoming?numDaysForward=90&limit=10 "));
    assert!(requests[3].starts_with("GET /trainingplan-service/trainingplan/tasks/42?localeKey=en "));
}
