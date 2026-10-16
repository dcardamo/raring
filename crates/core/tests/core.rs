use chrono::{Datelike, Duration, Local, Utc};
use pepometer_core::{
    garmin::{self, Client, Tokens},
    notifications,
    store::Store,
    Pepometer,
};
use serde_json::{json, Value};
use std::{
    io::{Read, Write},
    sync::{Arc, Mutex},
    time::Instant,
};

fn fixture() -> Value {
    serde_json::from_str(include_str!("fixtures/day.json")).unwrap()
}
fn seed(store: &Store) {
    let v = fixture();
    for kind in [
        "profile",
        "summary",
        "readiness",
        "sleep",
        "hrv",
        "training",
        "fitness_age",
        "activity",
        "zones",
    ] {
        let key = match kind {
            "profile" => "me",
            "activity" | "zones" => "123456",
            _ => "2025-01-02",
        };
        store
            .put(kind, key, "2025-01-02T09:00:00Z", &v[kind])
            .unwrap();
    }
}
#[test]
fn demo_seeds_current_day_and_screen_details() {
    let store = Store::open(":memory:").unwrap();
    let date = chrono::NaiveDate::from_ymd_opt(2026, 10, 16).unwrap();
    pepometer_core::demo::seed(&store, date).unwrap();
    assert!(store.days() >= 42);
    let today = pepometer_core::screens::today(&store, &date.to_string());
    assert_eq!(today.readiness.as_ref().unwrap().value, 82.);
    assert_eq!(today.sleep.as_ref().unwrap().value, 84.);
    assert_eq!(today.load.as_ref().unwrap().value, 612.);
    assert_eq!(today.chips[0].value.as_deref(), Some("7:48"));
    assert_eq!(today.chips[1].value.as_deref(), Some("58"));
    assert_eq!(today.chips[1].text.as_deref(), Some("ms"));
    assert_eq!(today.chips[2].text.as_deref(), Some("On track"));
    assert_eq!(today.factors.len(), 6);
    assert_eq!(today.body_battery.now, Some(78.));
    assert_eq!(today.headline, "Ready to push");
    assert_eq!(
        today.call_note.as_deref(),
        Some("Morning Run done · 186 load")
    );
    assert_eq!(today.fitness_age.chronological_age, Some(46.));
    assert_eq!(today.vitals.readings.len(), 5);
    assert_eq!(today.vitals.off_count, 0);
    let sleep = pepometer_core::screens::sleep_view(&store, &date.to_string());
    assert_eq!(sleep.stages.len(), 23);
    assert_eq!(sleep.overnight.len(), 4);
    assert!(sleep.overnight.iter().all(|o| o.series.len() >= 8));
    assert_eq!(sleep.consistency.len(), 14);
    assert!(sleep.consistency_spread_min.unwrap() > 0.);
    let health = pepometer_core::screens::health(&store, &date.to_string());
    assert_eq!(health.lever.as_ref().unwrap().key, "vigorousMinutes");
    assert_eq!(health.fitness_age.trend.len(), 12);
    assert!(health.fitness_age.trend.iter().all(Option::is_some));
    let monday = date - Duration::days(date.weekday().num_days_from_monday() as i64);
    let checkin = pepometer_core::screens::checkin(&store, &monday.to_string());
    assert!(checkin
        .cards
        .iter()
        .all(|c| c.value.is_some() && c.previous.is_some()));
    assert!(checkin.fitness_age_change.is_some_and(|delta| delta <= -1.));
    assert!(checkin.vo2max_change.is_some_and(|delta| delta >= 1.));
    assert_eq!(
        pepometer_core::screens::training(&store, &date.to_string())
            .load_focus
            .len(),
        3
    );
    let workout =
        pepometer_core::screens::workout(&store, pepometer_core::demo::WORKOUT_ID).unwrap();
    assert_eq!(workout.splits.len(), 10);
    assert_eq!(workout.heart_rate.len(), 210);
    assert_eq!(workout.route.len(), 210);
    assert_eq!(workout.zone_floors.len(), 4);
    assert_eq!(
        pepometer_core::screens::training(&store, &date.to_string())
            .workouts
            .len(),
        3
    );
}
#[test]
fn readiness_feedback_overrides_percent_and_missing_feedback_falls_back() {
    use pepometer_core::view::Tone;
    let store = Store::open(":memory:").unwrap();
    store
        .put(
            "readiness",
            "2025-01-02",
            "now",
            &json!([{
                "score":82,"level":"HIGH",
                "sleepScoreFactorPercent":91,"sleepScoreFactorFeedback":"POOR",
                "hrvFactorPercent":12,"hrvFactorFeedback":"BALANCED",
                "acwrFactorPercent":50,"acwrFactorFeedback":"MODERATE",
                "recoveryTimeFactorPercent":85,"sleepHistoryFactorPercent":80,
                "stressHistoryFactorFeedback":"GOOD"
            }]),
        )
        .unwrap();
    let view = pepometer_core::screens::today(&store, "2025-01-02");
    assert_eq!(view.factors.len(), 6);
    let factor = |key| view.factors.iter().find(|f| f.key == key).unwrap().tone;
    assert_eq!(factor("sleep"), Tone::Poor);
    assert_eq!(factor("hrv"), Tone::Good);
    assert_eq!(factor("acuteLoad"), Tone::Fair);
    assert_eq!(factor("recovery"), Tone::Good);
    assert_eq!(factor("stressHistory"), Tone::Good);
}
#[test]
fn load_chip_uses_garmin_optimal_range() {
    let store = Store::open(":memory:").unwrap();
    for (amount, expected) in [
        (350, "Room to build"),
        (612, "On track"),
        (810, "Load high"),
    ] {
        store
            .put(
                "training",
                "2025-01-02",
                "now",
                &json!({"acuteTrainingLoad":amount,"optimalRangeMin":420,"optimalRangeMax":760}),
            )
            .unwrap();
        let view = pepometer_core::screens::today(&store, "2025-01-02");
        assert_eq!(view.chips[0].text.as_deref(), Some(expected));
    }
}
#[test]
fn views_match_synthetic_mockup() {
    let s = Store::open(":memory:").unwrap();
    seed(&s);
    let t = pepometer_core::screens::today(&s, "2025-01-02");
    assert_eq!(t.readiness.as_ref().unwrap().value, 82.);
    assert_eq!(t.readiness_trend.len(), 30);
    assert_eq!(t.readiness_trend[29], Some(82.));
    assert_eq!(t.sleep.unwrap().value, 84.);
    assert_eq!(t.load.unwrap().value, 612.);
    assert_eq!(t.hrv.last_night, Some(58.));
    assert_eq!(t.fitness_age.fitness_age, Some(39.));
    assert_eq!(t.activities[0].distance_km, Some(10.2));
    assert_eq!(t.activities[0].duration_s, 3134.);
    assert_eq!(t.activities[0].aerobic_te, Some(3.8));
    assert_eq!(t.activities[0].benefit.as_deref(), Some("Tempo"));
    let sleep = pepometer_core::screens::sleep_view(&s, "2025-01-02");
    assert_eq!(sleep.score.unwrap().value, 84.);
    assert_eq!(sleep.tonight_bedtime.as_deref(), Some("23:53"));
    assert!(pepometer_core::screens::workout(&s, "123456").is_some());
}
#[test]
fn health_lever_follows_unmet_garmin_targets() {
    let store = Store::open(":memory:").unwrap();
    store.put("fitness_age","2025-01-02","now",&json!({"fitnessAge":39,"fitnessAgeData":{"vigorousMinutes":{"value":10,"target":30,"ageIfMet":38},"restingHr":{"value":60,"target":55}}})).unwrap();
    let view = pepometer_core::screens::health(&store, "2025-01-02");
    assert_eq!(view.lever.unwrap().key, "vigorousMinutes");
    assert!(
        !view
            .drivers
            .iter()
            .find(|d| d.key == "restingHr")
            .unwrap()
            .met
    );
}
#[test]
fn sleep_accepts_numeric_garmin_timestamps() {
    let s = Store::open(":memory:").unwrap();
    s.put("sleep","2025-01-02","now",&json!({"dailySleepDTO":{"sleepStartTimestampLocal":1735772400000_i64,"sleepEndTimestampLocal":1735801200000_i64,"sleepTimeSeconds":28000}})).unwrap();
    let view = pepometer_core::screens::sleep_view(&s, "2025-01-02");
    assert!(view.start_local.unwrap().contains('T'));
    assert_eq!(view.consistency.len(), 1);
}
#[test]
fn unknown_data_degrades() {
    let s = Store::open(":memory:").unwrap();
    assert!(pepometer_core::screens::today(&s, "2025-01-02")
        .readiness
        .is_none());
    assert!(pepometer_core::screens::workout(&s, "123456").is_none());
}
#[test]
fn notification_log_and_defaults() {
    let s = Store::open(":memory:").unwrap();
    seed(&s);
    let when = chrono::NaiveDate::from_ymd_opt(2025, 1, 2)
        .unwrap()
        .and_hms_opt(9, 0, 0)
        .unwrap()
        .and_local_timezone(Local)
        .single()
        .unwrap();
    let first = notifications::notifications(&s, when);
    assert!(first.iter().any(|v| v.kind == "morning"));
    assert!(!first.iter().any(|v| v.kind == "bedtime"));
    notifications::mark_sent(&s, &first, when).unwrap();
    assert!(!notifications::notifications(&s, when)
        .iter()
        .any(|v| v.kind == "morning"));
}
#[test]
fn metric_change_notifications_obey_shared_setting() {
    let s = Store::open(":memory:").unwrap();
    seed(&s);
    s.put(
        "max_metrics",
        "2025-01-01",
        "now",
        &json!([{"generic":{"vo2MaxValue":43.}}]),
    )
    .unwrap();
    s.put(
        "max_metrics",
        "2025-01-02",
        "now",
        &json!([{"generic":{"vo2MaxValue":44.}}]),
    )
    .unwrap();
    s.put(
        "fitness_age",
        "2025-01-01",
        "now",
        &json!({"fitnessAge":40}),
    )
    .unwrap();
    let when = chrono::NaiveDate::from_ymd_opt(2025, 1, 2)
        .unwrap()
        .and_hms_opt(9, 0, 0)
        .unwrap()
        .and_local_timezone(Local)
        .single()
        .unwrap();
    let items = notifications::notifications(&s, when);
    assert!(items.iter().any(|n| n.kind == "vo2max"));
    assert!(items.iter().any(|n| n.kind == "fitness-age"));
    let mut settings = s.settings();
    settings.notify_metric_changes = false;
    s.set_settings(&settings).unwrap();
    assert!(!notifications::notifications(&s, when)
        .iter()
        .any(|n| n.kind == "vo2max" || n.kind == "fitness-age"));
}
#[test]
fn metric_notifications_compare_rounded_values_past_missing_days() {
    let s = Store::open(":memory:").unwrap();
    let when = chrono::NaiveDate::from_ymd_opt(2025, 1, 3)
        .unwrap()
        .and_hms_opt(9, 0, 0)
        .unwrap()
        .and_local_timezone(Local)
        .single()
        .unwrap();
    s.put(
        "fitness_age",
        "2025-01-01",
        "now",
        &json!({"fitnessAge":39.49}),
    )
    .unwrap();
    s.put("fitness_age", "2025-01-02", "now", &Value::Null)
        .unwrap();
    s.put(
        "fitness_age",
        "2025-01-03",
        "now",
        &json!({"fitnessAge":39.40}),
    )
    .unwrap();
    s.put(
        "max_metrics",
        "2025-01-01",
        "now",
        &json!([{"generic":{"vo2MaxValue":50.49}}]),
    )
    .unwrap();
    s.put(
        "max_metrics",
        "2025-01-03",
        "now",
        &json!([{"generic":{"vo2MaxValue":50.40}}]),
    )
    .unwrap();
    assert!(!notifications::notifications(&s, when)
        .iter()
        .any(|n| n.kind == "fitness-age" || n.kind == "vo2max"));
    s.put(
        "fitness_age",
        "2025-01-03",
        "now",
        &json!({"fitnessAge":40.10}),
    )
    .unwrap();
    s.put(
        "max_metrics",
        "2025-01-03",
        "now",
        &json!([{"generic":{"vo2MaxValue":51.10}}]),
    )
    .unwrap();
    assert!(notifications::notifications(&s, when)
        .iter()
        .any(|n| n.kind == "fitness-age"));
    assert!(notifications::notifications(&s, when)
        .iter()
        .any(|n| n.kind == "vo2max"));
}
#[test]
fn privacy_guards() {
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let files = std::fs::read_dir(folder).unwrap();
    fn walk(v: &Value) {
        match v {
            Value::Object(o) => {
                for (k, v) in o {
                    if k.to_ascii_lowercase().contains("displayname") {
                        assert_eq!(v, "Sample User");
                    }
                    walk(v)
                }
            }
            Value::Array(a) => {
                for v in a {
                    walk(v)
                }
            }
            Value::String(s) => {
                for word in s.split_whitespace() {
                    let parts: Vec<&str> = word.split('.').collect();
                    assert!(
                        !(parts.len() == 3
                            && parts[0].len() >= 3
                            && parts[1].len() >= 8
                            && parts[2].len() >= 8
                            && parts.iter().all(|p| p
                                .bytes()
                                .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_'))),
                        "JWT-looking fixture value"
                    );
                }
            }
            _ => {}
        }
    }
    for entry in files {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|s| s != "json") {
            continue;
        }
        let source = std::fs::read_to_string(&path).unwrap();
        assert!(!source.contains('@'), "{}", path.display());
        assert!(!source.contains("eyJ"), "{}", path.display());
        assert!(!source.contains("access_token"), "{}", path.display());
        let data: Value = serde_json::from_str(&source).unwrap();
        walk(&data);
    }
}
#[test]
fn settings_are_a_single_json_kv_record() {
    let store = Store::open(":memory:").unwrap();
    let mut value = store.settings();
    assert!(value.notify_weekly_checkin);
    assert!(!value.notify_bedtime);
    value.notify_metric_changes = false;
    store.set_settings(&value).unwrap();
    let json = store.setting("settings").unwrap();
    assert_eq!(
        serde_json::from_str::<pepometer_core::view::Settings>(&json).unwrap(),
        value
    );
    assert_eq!(store.settings(), value);
}
#[test]
fn facade_settings_and_forget() {
    let core = Pepometer::open(":memory:").unwrap();
    assert!(!core.setting("bedtime"));
    assert!(core.setting("morning"));
    core.set_setting("morning", false).unwrap();
    assert!(!core.setting("morning"));
    assert!(!core.settings().notify_morning_readiness);
    let mut settings = core.settings();
    settings.notify_training_status = true;
    core.set_settings(settings).unwrap();
    assert!(core.settings().notify_training_status);
    core.forget_account().unwrap();
    assert!(core.setting("morning"));
}
fn token(exp: i64) -> String {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    format!(
        "e30.{}.sig",
        URL_SAFE_NO_PAD.encode(json!({"exp":exp,"client_id":"TEST_DI"}).to_string())
    )
}
#[test]
fn token_import_and_redaction() {
    let t = Tokens::import_python(
        &json!({"di_token":token(1900000000),"di_refresh_token":"secret","di_client_id":"TEST_DI"})
            .to_string(),
    )
    .unwrap();
    assert_eq!(t.client_id, "TEST_DI");
    assert_eq!(t.expires_at.timestamp(), 1900000000);
    assert!(!format!("{t:?}").contains("secret"));
    assert!(t.is_expiring(Utc::now() + Duration::days(2000), Duration::minutes(5)));
}
// Local-only HTTP server; records requests and returns scripted responses. No Garmin traffic.
fn mock(
    replies: Vec<(u16, String)>,
) -> (String, Arc<Mutex<Vec<String>>>, std::thread::JoinHandle<()>) {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(false).unwrap();
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
            log.lock()
                .unwrap()
                .push(String::from_utf8_lossy(&buf[..n]).into_owned());
            let text=format!("HTTP/1.1 {status} OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len());
            stream.write_all(text.as_bytes()).unwrap();
        }
    });
    (url, seen, thread)
}
#[tokio::test]
async fn no_content_is_cached_and_bad_item_does_not_stop_readiness() {
    let store = Store::open(":memory:").unwrap();
    store
        .put(
            "profile",
            "me",
            "now",
            &json!({"displayName":"Sample User"}),
        )
        .unwrap();
    let (url, seen, thread) = mock(vec![
        (204, "".into()),
        (500, "{}".into()),
        (200, "{}".into()),
        (200, "[{\"score\":80}]".into()),
    ]);
    let c = Client::with_urls(
        Tokens {
            access_token: token(1900000000),
            refresh_token: "refresh".into(),
            client_id: "TEST_DI".into(),
            expires_at: Utc::now() + Duration::days(30),
        },
        &url,
        &url,
        std::time::Duration::ZERO,
    )
    .unwrap();
    let report = pepometer_core::sync::sync(&store, &c, Local::now(), 4).await;
    thread.join().unwrap();
    assert_eq!(report.requests, 4);
    assert_eq!(report.data_failures, 1);
    assert!(report.error.is_none());
    assert!(!report.auth_failed);
    assert!(store
        .get("summary", &Local::now().date_naive().to_string())
        .unwrap()
        .is_null());
    assert!(report.readiness_present);
    assert!(seen
        .lock()
        .unwrap()
        .iter()
        .any(|r| r.contains("trainingreadiness")));
}
#[tokio::test]
async fn backfill_scans_existing_days_to_fill_older_gaps() {
    let s = Store::open(":memory:").unwrap();
    let now = chrono::NaiveDate::from_ymd_opt(2025, 1, 10)
        .unwrap()
        .and_hms_opt(12, 0, 0)
        .unwrap()
        .and_local_timezone(Local)
        .single()
        .unwrap();
    s.put(
        "profile",
        "me",
        "now",
        &json!({"displayName":"Sample User"}),
    )
    .unwrap();
    for day in ["2025-01-10", "2025-01-08"] {
        for kind in [
            "summary",
            "sleep",
            "hrv",
            "readiness",
            "training",
            "training_daily",
            "load_focus",
            "battery",
            "spo2",
            "respiration",
            "stress",
            "max_metrics",
        ] {
            s.put(kind, day, "now", &Value::Null).unwrap();
        }
    }
    for kind in ["fitness_age", "races", "endurance", "hill", "tolerance"] {
        s.put(kind, "2025-01-10", "now", &Value::Null).unwrap();
    }
    s.put("records", "all", "now", &Value::Null).unwrap();
    s.set_cursor("records_checked", &Utc::now().timestamp().to_string())
        .unwrap();
    let mut replies = vec![(200, "{}".into()); 12];
    replies.extend([(200, "[]".into()), (200, "{}".into())]);
    let (url, seen, thread) = mock(replies);
    let c = Client::with_urls(
        Tokens {
            access_token: token(1900000000),
            refresh_token: "refresh".into(),
            client_id: "TEST_DI".into(),
            expires_at: Utc::now() + Duration::days(30),
        },
        &url,
        &url,
        std::time::Duration::ZERO,
    )
    .unwrap();
    let report = pepometer_core::sync::sync(&s, &c, now, 14).await;
    thread.join().unwrap();
    assert_eq!(report.requests, 14);
    assert!(s.get("summary", "2025-01-07").is_some());
    assert!(seen.lock().unwrap()[13].contains("2025-01-07"));
}
#[tokio::test]
async fn forbidden_is_not_signout_and_rejected_refresh_is() {
    let t = Tokens {
        access_token: token(1900000000),
        refresh_token: "refresh".into(),
        client_id: "TEST_DI".into(),
        expires_at: Utc::now() + Duration::days(30),
    };
    let (url, _, thread) = mock(vec![(403, "{}".into())]);
    let c = Client::with_urls(t.clone(), &url, &url, std::time::Duration::ZERO).unwrap();
    assert!(matches!(
        c.get("/data", &[]).await,
        Err(garmin::Error::Http(403))
    ));
    thread.join().unwrap();
    let (url, _, thread) = mock(vec![(400, "{\"error\":\"invalid_grant\"}".into())]);
    assert!(matches!(
        garmin::refresh_at(&reqwest::Client::new(), &url, &t).await,
        Err(garmin::Error::NeedsSignIn)
    ));
    thread.join().unwrap();
}
#[test]
fn nested_training_and_midnight_spread() {
    let s = Store::open(":memory:").unwrap();
    s.put("training", "2025-01-02", "now", &json!({"mostRecentTrainingStatus":{"latestTrainingStatusData":{"deviceA":{"trainingStatus":"PRODUCTIVE","acuteChronicWorkloadRatio":1.4,"acuteTrainingLoadDTO":{"dailyTrainingLoadAcute":630,"minOptimalLoad":400,"maxOptimalLoad":700}}}}})).unwrap();
    s.put("load_focus", "2025-01-02", "now", &json!({"mostRecentTrainingLoadBalance":{"metricsTrainingLoadBalanceDTOMap":{"deviceA":{"lowAerobic":{"value":300,"optimalMin":250,"optimalMax":400}}}}})).unwrap();
    let training = pepometer_core::screens::training(&s, "2025-01-02");
    assert_eq!(training.acute.unwrap().value, 630.);
    assert_eq!(training.status.as_deref(), Some("Productive"));
    assert_eq!(training.load_focus.len(), 1);
    assert_eq!(
        pepometer_core::screens::today(&s, "2025-01-02")
            .load
            .unwrap()
            .band,
        Some([400., 700.])
    );
    for i in 0..14 {
        let day =
            (chrono::NaiveDate::from_ymd_opt(2025, 1, 2).unwrap() - Duration::days(i)).to_string();
        let start = format!("{day}T{}", if i % 2 == 0 { "00:20:00" } else { "23:40:00" });
        s.put("sleep", &day, "now", &json!({"dailySleepDTO":{"sleepStartTimestampLocal":start,"sleepEndTimestampLocal":format!("{day}T08:00:00")}})).unwrap();
    }
    assert!(
        pepometer_core::screens::sleep_view(&s, "2025-01-02")
            .consistency_spread_min
            .unwrap()
            < 60.
    );
}
#[tokio::test]
async fn endpoint_paths_and_query_match_reference() {
    let (url, seen, thread) = mock(vec![(200, "{}".into()); 4]);
    let tokens = Tokens {
        access_token: token(1900000000),
        refresh_token: "refresh".into(),
        client_id: "TEST_DI".into(),
        expires_at: Utc::now() + Duration::days(30),
    };
    let c = Client::with_urls(tokens, &url, &url, std::time::Duration::ZERO).unwrap();
    for (kind, key) in [
        ("records", "all"),
        ("max_metrics", "2025-01-02"),
        ("sleep", "2025-01-02"),
        ("training", "2025-01-02"),
    ] {
        c.endpoint(kind, key, Some("Sample User")).await.unwrap();
    }
    thread.join().unwrap();
    let req = seen.lock().unwrap().clone();
    assert!(
        req[0].starts_with("GET /personalrecord-service/personalrecord/prs/Sample%20User"),
        "{}",
        req[0]
    );
    assert!(req[1].starts_with("GET /metrics-service/metrics/maxmet/daily/2025-01-02/2025-01-02"));
    assert!(req[2].contains("nonSleepBufferMinutes=60"));
    assert!(req[3].contains("/trainingstatus/aggregated/2025-01-02"));
}
#[tokio::test]
async fn exchange_uses_ticket_and_service_url() {
    let (url, seen, thread) = mock(vec![(
        200,
        json!({"access_token":token(1900000000),"refresh_token":"refresh"}).to_string(),
    )]);
    let http = reqwest::Client::new();
    let t = garmin::exchange_ticket_at(&http, &url, "ST-synthetic", "https://example.invalid/app")
        .await
        .unwrap();
    thread.join().unwrap();
    assert_eq!(t.client_id, "TEST_DI");
    let request = seen.lock().unwrap().join(" ");
    assert!(request.contains("service_ticket=ST-synthetic"));
    assert!(request.contains("service_url=https%3A%2F%2Fexample.invalid%2Fapp"));
    assert!(request.contains("authorization: Basic") || request.contains("Authorization: Basic"));
}
#[tokio::test]
async fn refresh_on_401_and_pacing() {
    let fresh = token(1900000000);
    let (url, seen, thread) = mock(vec![
        (401, "{}".into()),
        (200, json!({"access_token":fresh}).to_string()),
        (200, "{\"ok\":true}".into()),
        (200, "{}".into()),
    ]);
    let tokens = Tokens {
        access_token: token(1900000000),
        refresh_token: "refresh".into(),
        client_id: "TEST_DI".into(),
        expires_at: Utc::now() + Duration::days(30),
    };
    let client =
        Client::with_urls(tokens, &url, &url, std::time::Duration::from_millis(40)).unwrap();
    let start = Instant::now();
    assert_eq!(client.get("/data", &[]).await.unwrap()["ok"], true);
    client.get("/data", &[]).await.unwrap();
    thread.join().unwrap();
    assert!(start.elapsed() >= std::time::Duration::from_millis(75));
    assert_eq!(seen.lock().unwrap().len(), 4);
}
#[tokio::test]
async fn sync_persists_rate_limit_backoff() {
    let store = Store::open(":memory:").unwrap();
    store
        .put(
            "profile",
            "me",
            "2025-01-02T00:00Z",
            &json!({"displayName":"Sample User"}),
        )
        .unwrap();
    let (url, _, thread) = mock(vec![(429, "{}".into())]);
    let tokens = Tokens {
        access_token: token(1900000000),
        refresh_token: "refresh".into(),
        client_id: "TEST_DI".into(),
        expires_at: Utc::now() + Duration::days(30),
    };
    let c = Client::with_urls(tokens, &url, &url, std::time::Duration::ZERO).unwrap();
    let first = pepometer_core::sync::sync(&store, &c, Local::now(), 1).await;
    assert!(first.rate_limited);
    assert_eq!(first.requests, 1);
    let next = pepometer_core::sync::sync(&store, &c, Local::now(), 1).await;
    assert!(next.rate_limited);
    assert_eq!(next.requests, 0);
    thread.join().unwrap();
}
#[tokio::test]
async fn sync_reports_auth_failure_structurally() {
    let store = Store::open(":memory:").unwrap();
    let (url, _, thread) = mock(vec![(401, "{}".into()), (401, "{}".into())]);
    let tokens = Tokens {
        access_token: token(1900000000),
        refresh_token: "refresh".into(),
        client_id: "TEST_DI".into(),
        expires_at: Utc::now() + Duration::days(30),
    };
    let client = Client::with_urls(tokens, &url, &url, std::time::Duration::ZERO).unwrap();
    let report = pepometer_core::sync::sync(&store, &client, Local::now(), 1).await;
    assert!(report.auth_failed);
    thread.join().unwrap();
}
#[tokio::test]
async fn rate_limit_returns_backoff() {
    let (url, _, thread) = mock(vec![(429, "{}".into())]);
    let tokens = Tokens {
        access_token: token(1900000000),
        refresh_token: "refresh".into(),
        client_id: "TEST_DI".into(),
        expires_at: Utc::now() + Duration::days(30),
    };
    let c = Client::with_urls(tokens, &url, &url, std::time::Duration::ZERO).unwrap();
    assert!(matches!(
        c.get("/data", &[]).await,
        Err(garmin::Error::RateLimited { .. })
    ));
    thread.join().unwrap();
}
