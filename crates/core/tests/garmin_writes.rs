//! Rules: Garmin writes, end to end against a local mock of the device
//! settings and weigh-in endpoints. Every payload is synthetic.
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::{DateTime, TimeZone, Utc};
use raring_core::{
    garmin::Client,
    store::Store,
    view::{ChangedField, GarminWrite, TrendRange, WeightUnit, WriteOutcome},
    writes::AlarmEdit,
    Raring, Tokens,
};
use serde_json::{json, Value};
use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};

#[derive(Debug, Clone)]
struct Seen {
    method: String,
    path: String,
    auth: String,
    body: Option<Value>,
}

/// Answers scripted replies in order, then 500 for anything unscripted, and
/// records every request so tests can prove which ones were (not) made.
struct Mock {
    url: String,
    seen: Arc<Mutex<Vec<Seen>>>,
    stop: Arc<AtomicBool>,
    handle: Option<std::thread::JoinHandle<()>>,
}
impl Mock {
    fn new(replies: Vec<(u16, &'static str, String)>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let seen = Arc::new(Mutex::new(Vec::new()));
        let stop = Arc::new(AtomicBool::new(false));
        let (log, done) = (Arc::clone(&seen), Arc::clone(&stop));
        let handle = std::thread::spawn(move || {
            let mut replies = replies.into_iter();
            while !done.load(Ordering::SeqCst) {
                let Ok((mut stream, _)) = listener.accept() else {
                    std::thread::sleep(std::time::Duration::from_millis(5));
                    continue;
                };
                stream.set_nonblocking(false).unwrap();
                stream.set_read_timeout(Some(std::time::Duration::from_secs(4))).unwrap();
                let mut bytes = Vec::new();
                let mut chunk = [0; 4096];
                let header_end = loop {
                    let n = stream.read(&mut chunk).unwrap();
                    bytes.extend_from_slice(&chunk[..n]);
                    if let Some(at) = bytes.windows(4).position(|w| w == b"\r\n\r\n") {
                        break at + 4;
                    }
                    assert!(n > 0, "request ended early");
                };
                let head = String::from_utf8_lossy(&bytes[..header_end]).into_owned();
                let header = |name: &str| {
                    head.lines()
                        .find_map(|l| l.split_once(':').filter(|(k, _)| k.eq_ignore_ascii_case(name)).map(|(_, v)| v.trim().to_owned()))
                        .unwrap_or_default()
                };
                let length: usize = header("content-length").parse().unwrap_or(0);
                while bytes.len() < header_end + length {
                    let n = stream.read(&mut chunk).unwrap();
                    bytes.extend_from_slice(&chunk[..n]);
                }
                let mut first = head.lines().next().unwrap().split(' ');
                let body = &bytes[header_end..header_end + length];
                log.lock().unwrap().push(Seen {
                    method: first.next().unwrap().into(),
                    path: first.next().unwrap().into(),
                    auth: header("authorization"),
                    body: (!body.is_empty()).then(|| serde_json::from_slice(body).unwrap_or(Value::String(String::from_utf8_lossy(body).into()))),
                });
                let (status, extra, body) = replies.next().unwrap_or((500, "", "unscripted".into()));
                let response = format!(
                    "HTTP/1.1 {status} X\r\nContent-Type: application/json\r\n{extra}Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                stream.write_all(response.as_bytes()).unwrap();
            }
        });
        Self { url, seen, stop, handle: Some(handle) }
    }
    fn seen(&self) -> Vec<Seen> {
        self.seen.lock().unwrap().clone()
    }
    fn calls(&self) -> Vec<String> {
        self.seen().iter().map(|s| format!("{} {}", s.method, s.path)).collect()
    }
}
impl Drop for Mock {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

fn token(name: &str) -> String {
    format!("e30.{}.{name}", URL_SAFE_NO_PAD.encode(json!({"exp":1900000000,"client_id":"TEST_DI"}).to_string()))
}
fn client(url: &str) -> Client {
    Client::with_urls(
        Tokens { access_token: token("first"), refresh_token: "refresh".into(), client_id: "TEST_DI".into(), expires_at: Utc::now() + chrono::Duration::days(30) },
        url,
        url,
        std::time::Duration::ZERO,
    )
    .unwrap()
}

/// A signed-in account 101 with one database open twice, as the app and sync do.
fn fixture(name: &str) -> (std::path::PathBuf, Store, Raring) {
    let dir = std::env::temp_dir().join(format!("raring-writes-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let db = dir.join("raring.db");
    let store = Store::open(&db).unwrap();
    store.put("profile", "me", "synthetic", &json!({"profileId":101,"displayName":"fake-A"})).unwrap();
    store.set_setting("account_session", &json!({"identity":"101","active":true}).to_string()).unwrap();
    (dir, store, Raring::open(db).unwrap())
}

const SETTINGS: &str = "/device-service/deviceservice/device-info/settings/3001";
const DETAIL: &str = "/device-service/deviceservice/user-device/3001";

fn alarm(id: u64, time: u32, mode: &str, days: &[&str]) -> Value {
    json!({"alarmId":id,"alarmTime":time,"alarmMode":mode,"alarmDays":days,"alarmSound":"TONE","backlight":"ON","changeState":"UNCHANGED","enabled":null,"alarmType":null})
}
fn settings(alarms: Vec<Value>) -> Value {
    json!({"deviceId":3001,"dndEnabled":false,"startOfWeek":"MONDAY","alarms":alarms,
        "sleep":{"sleepStart":79200,"sleepEnd":23400},"activityTracking":{"moveAlertEnabled":true}})
}
fn two_alarms() -> Value {
    settings(vec![alarm(7, 390, "ON", &["MONDAY", "TUESDAY"]), alarm(8, 480, "OFF", &["ONCE"])])
}
fn detail(downloaded: i64) -> String {
    json!({"deviceId":3001,"lastUploadTimestamp":downloaded,"lastDownloadTimestamp":downloaded}).to_string()
}
fn at() -> DateTime<Utc> {
    "2026-10-19T08:00:00Z".parse().unwrap()
}
const SEVEN_LATER: AlarmEdit = AlarmEdit::Time { alarm_id: 7, from: 390, to: 420 };

fn field(label: &str, before: Option<&str>, after: Option<&str>, key: &str) -> ChangedField {
    ChangedField { field: key.into(), label: label.into(), before: before.map(Into::into), after: after.map(Into::into) }
}

#[tokio::test]
async fn puts_only_the_changed_field_from_a_fresh_read_and_waits_for_the_watch() {
    let (_dir, store, core) = fixture("applied");
    let mut after = two_alarms();
    after["alarms"][0]["alarmTime"] = 420.into();
    let mock = Mock::new(vec![
        (200, "", two_alarms().to_string()),
        (204, "", String::new()),
        (200, "", after.to_string()),
        (200, "", detail(1_000)),
    ]);
    let change = core.change_watch_alarms(&client(&mock.url), "3001", &[SEVEN_LATER], false, at()).await.unwrap();
    assert_eq!(change.outcome, WriteOutcome::Applied);
    assert_eq!(change.fields, vec![field("Alarm time", Some("06:30"), Some("07:00"), "alarm:7.alarmTime")]);
    assert!(change.mismatches.is_empty() && change.message.is_none());
    assert_eq!(mock.calls(), [format!("GET {SETTINGS}"), format!("PUT {SETTINGS}"), format!("GET {SETTINGS}"), format!("GET {DETAIL}")]);
    // A partial object: the device id and the one list holding the change,
    // every other alarm and alarm field exactly as freshly read.
    assert_eq!(mock.seen()[1].body, Some(json!({"deviceId":3001,"alarms":after["alarms"]})));
    assert!(change.waiting_for_watch);
    assert_eq!(core.garmin_changes(), vec![change.clone()]);

    // Waiting ends only when the same watch's download time moves past the baseline.
    store.put("device_detail", "selected", "later", &serde_json::from_str(&detail(1_000)).unwrap()).unwrap();
    assert!(core.garmin_changes()[0].waiting_for_watch);
    store.put("device_detail", "selected", "later", &json!({"deviceId":3002,"lastDownloadTimestamp":9_000})).unwrap();
    assert!(core.garmin_changes()[0].waiting_for_watch);
    store.put("device_detail", "selected", "later", &serde_json::from_str(&detail(2_000)).unwrap()).unwrap();
    assert!(!core.garmin_changes()[0].waiting_for_watch);
}

#[tokio::test]
async fn a_change_made_elsewhere_is_kept_and_one_to_the_same_field_is_not_overwritten() {
    let (_dir, _store, core) = fixture("concurrent");
    // Another alarm and do-not-disturb changed in Connect after the person looked.
    let mut elsewhere = two_alarms();
    elsewhere["alarms"][1]["alarmTime"] = 510.into();
    elsewhere["dndEnabled"] = true.into();
    let mut after = elsewhere.clone();
    after["alarms"][0]["alarmTime"] = 420.into();
    let mock = Mock::new(vec![
        (200, "", elsewhere.to_string()),
        (200, "", "{}".into()),
        (200, "", after.to_string()),
        (200, "", detail(1_000)),
    ]);
    let change = core.change_watch_alarms(&client(&mock.url), "3001", &[SEVEN_LATER], false, at()).await.unwrap();
    assert_eq!(change.outcome, WriteOutcome::Applied);
    let put = mock.seen()[1].body.clone().unwrap();
    assert_eq!(put["alarms"][1]["alarmTime"], 510, "the other alarm keeps Connect's change");
    assert!(put.get("dndEnabled").is_none(), "untouched settings are not sent");
    drop(mock);

    // The same alarm's time moved to 06:40 in Connect: nothing is written.
    let mut moved = two_alarms();
    moved["alarms"][0]["alarmTime"] = 400.into();
    let mock = Mock::new(vec![(200, "", moved.to_string())]);
    let change = core.change_watch_alarms(&client(&mock.url), "3001", &[SEVEN_LATER], false, at()).await.unwrap();
    assert_eq!(change.outcome, WriteOutcome::Conflict);
    assert_eq!(change.message.as_deref(), Some("Changed in Garmin since you looked: alarm time is now 06:40"));
    assert!(!change.waiting_for_watch);
    assert_eq!(mock.calls(), [format!("GET {SETTINGS}")]);
    drop(mock);

    // A deleted alarm is a conflict too.
    let mock = Mock::new(vec![(200, "", settings(vec![alarm(8, 480, "OFF", &["ONCE"])]).to_string())]);
    let change = core.change_watch_alarms(&client(&mock.url), "3001", &[SEVEN_LATER], false, at()).await.unwrap();
    assert_eq!(change.outcome, WriteOutcome::Conflict);
    assert_eq!(mock.calls().len(), 1);
    let log = core.garmin_changes();
    assert_eq!(log.iter().map(|c| c.outcome).collect::<Vec<_>>(), [WriteOutcome::Conflict, WriteOutcome::Conflict, WriteOutcome::Applied]);
}

#[tokio::test]
async fn read_back_compares_what_was_sent() {
    let (_dir, _store, core) = fixture("readback");
    let edits = [
        SEVEN_LATER,
        AlarmEdit::Days { alarm_id: 7, from: vec!["MONDAY".into(), "TUESDAY".into()], to: vec!["MONDAY".into(), "FRIDAY".into()] },
    ];
    // Garmin reorders day lists and may return decimals: still applied.
    let mut reordered = two_alarms();
    reordered["alarms"][0]["alarmTime"] = json!(420.0);
    reordered["alarms"][0]["alarmDays"] = json!(["MONDAY", "FRIDAY"]);
    let mock = Mock::new(vec![(200, "", two_alarms().to_string()), (200, "", String::new()), (200, "", reordered.to_string()), (200, "", detail(1))]);
    let change = core.change_watch_alarms(&client(&mock.url), "3001", &edits, false, at()).await.unwrap();
    assert_eq!(change.outcome, WriteOutcome::Applied, "{:?}", change.mismatches);
    assert_eq!(change.fields[1], field("Alarm days", Some("Monday, Tuesday"), Some("Monday, Friday"), "alarm:7.alarmDays"));
    drop(mock);

    // The time didn't stick.
    let mock = Mock::new(vec![(200, "", two_alarms().to_string()), (200, "", String::new()), (200, "", two_alarms().to_string()), (200, "", detail(1))]);
    let change = core.change_watch_alarms(&client(&mock.url), "3001", &[SEVEN_LATER], false, at()).await.unwrap();
    assert_eq!(change.outcome, WriteOutcome::Differs);
    assert_eq!(change.mismatches, vec![field("Alarm time", Some("07:00"), Some("06:30"), "alarm:7.alarmTime")]);
    assert!(change.waiting_for_watch, "Garmin accepted it, so the watch may still pick it up");
    drop(mock);

    // The time stuck but another alarm in the sent list came back different.
    let mut other = two_alarms();
    other["alarms"][0]["alarmTime"] = 420.into();
    other["alarms"][1]["alarmMode"] = "ON".into();
    let mock = Mock::new(vec![(200, "", two_alarms().to_string()), (200, "", String::new()), (200, "", other.to_string()), (200, "", detail(1))]);
    let change = core.change_watch_alarms(&client(&mock.url), "3001", &[SEVEN_LATER], false, at()).await.unwrap();
    assert_eq!(change.outcome, WriteOutcome::Differs);
    assert_eq!(change.mismatches, vec![field("Other alarm settings", None, None, "other")]);
    drop(mock);

    // Accepted but the read-back failed: unconfirmed, not applied.
    let mock = Mock::new(vec![(200, "", two_alarms().to_string()), (200, "", String::new()), (503, "", String::new())]);
    let change = core.change_watch_alarms(&client(&mock.url), "3001", &[SEVEN_LATER], false, at()).await.unwrap();
    assert_eq!(change.outcome, WriteOutcome::Unconfirmed);
    assert_eq!(change.message.as_deref(), Some("Saved, but Garmin's read-back failed"));
    assert_eq!(mock.calls().len(), 3);
}

#[tokio::test]
async fn a_rejection_keeps_garmins_message_and_reads_nothing_back() {
    let (_dir, _store, core) = fixture("rejected");
    let mock = Mock::new(vec![(200, "", two_alarms().to_string()), (400, "", json!({"message":"Invalid alarm time"}).to_string())]);
    let change = core.change_watch_alarms(&client(&mock.url), "3001", &[SEVEN_LATER], false, at()).await.unwrap();
    assert_eq!(change.outcome, WriteOutcome::Rejected);
    assert_eq!(change.message.as_deref(), Some("Invalid alarm time"));
    assert!(!change.waiting_for_watch);
    assert_eq!(mock.calls(), [format!("GET {SETTINGS}"), format!("PUT {SETTINGS}")]);
    assert_eq!(core.garmin_changes()[0].message.as_deref(), Some("Invalid alarm time"));
}

#[tokio::test]
async fn sign_in_is_refreshed_once_under_the_shared_session_then_needed() {
    let (_dir, _store, core) = fixture("auth");
    // A 401 on the write refreshes the token and retries the same write once.
    let mut after = two_alarms();
    after["alarms"][0]["alarmTime"] = 420.into();
    let refreshed = json!({"access_token":token("second"),"refresh_token":"refresh-2","expires_in":3600}).to_string();
    let mock = Mock::new(vec![
        (200, "", two_alarms().to_string()),
        (401, "", String::new()),
        (200, "", refreshed),
        (204, "", String::new()),
        (200, "", after.to_string()),
        (200, "", detail(1)),
    ]);
    let session = client(&mock.url);
    let change = core.change_watch_alarms(&session, "3001", &[SEVEN_LATER], false, at()).await.unwrap();
    assert_eq!(change.outcome, WriteOutcome::Applied);
    let seen = mock.seen();
    assert_eq!(seen.iter().map(|s| s.method.as_str()).collect::<Vec<_>>(), ["GET", "PUT", "POST", "PUT", "GET", "GET"]);
    assert!(seen[1].auth.ends_with(".first") && seen[3].auth.ends_with(".second"));
    assert_eq!(seen[1].body, seen[3].body);
    assert_eq!(session.tokens().await.refresh_token, "refresh-2");
    drop(mock);

    // A rejected refresh means signing in again; nothing is written.
    let mock = Mock::new(vec![(401, "", String::new()), (401, "", String::new())]);
    let change = core.change_watch_alarms(&client(&mock.url), "3001", &[SEVEN_LATER], false, at()).await.unwrap();
    assert_eq!(change.outcome, WriteOutcome::NeedsSignIn);
    assert_eq!(mock.seen().iter().map(|s| s.method.as_str()).collect::<Vec<_>>(), ["GET", "POST"]);
    assert_eq!(core.garmin_changes()[0].outcome, WriteOutcome::NeedsSignIn);
}

#[tokio::test]
async fn rate_limits_back_off_and_later_writes_wait_without_requests() {
    let (_dir, store, core) = fixture("rate");
    let mock = Mock::new(vec![(429, "Retry-After: 120\r\n", String::new())]);
    let change = core.change_watch_alarms(&client(&mock.url), "3001", &[SEVEN_LATER], false, at()).await.unwrap();
    assert_eq!(change.outcome, WriteOutcome::Failed);
    assert_eq!(change.message.as_deref(), Some("Garmin is limiting requests; try again later"));
    assert_eq!(store.cursor("backoff_until"), Some((at().timestamp() + 120).to_string()));
    let change = core.change_watch_alarms(&client(&mock.url), "3001", &[SEVEN_LATER], false, at() + chrono::Duration::seconds(60)).await.unwrap();
    assert_eq!(change.outcome, WriteOutcome::Failed);
    assert_eq!(mock.calls().len(), 1, "no request during the backoff");
    drop(mock);

    // A 429 on the write itself: not saved, no read-back.
    let later = at() + chrono::Duration::seconds(121);
    let mock = Mock::new(vec![(200, "", two_alarms().to_string()), (429, "", String::new())]);
    let change = core.change_watch_alarms(&client(&mock.url), "3001", &[SEVEN_LATER], false, later).await.unwrap();
    assert_eq!(change.outcome, WriteOutcome::Failed);
    assert_eq!(mock.calls().len(), 2);
    assert_eq!(store.cursor("backoff_until"), Some((later.timestamp() + 300).to_string()));
}

#[tokio::test]
async fn a_server_failure_on_the_write_is_unconfirmed_not_failed() {
    let (_dir, _store, core) = fixture("server");
    let mock = Mock::new(vec![(200, "", two_alarms().to_string()), (502, "", String::new())]);
    let change = core.change_watch_alarms(&client(&mock.url), "3001", &[SEVEN_LATER], false, at()).await.unwrap();
    assert_eq!(change.outcome, WriteOutcome::Unconfirmed);
    assert_eq!(change.message.as_deref(), Some("Garmin answered with error 502; Garmin may or may not have saved it"));
}

#[tokio::test]
async fn deletes_need_an_explicit_confirmation() {
    let (_dir, _store, core) = fixture("delete");
    let mock = Mock::new(vec![]);
    let remove = [AlarmEdit::Remove { alarm_id: 7 }];
    assert!(core.change_watch_alarms(&client(&mock.url), "3001", &remove, false, at()).await.is_err());
    assert!(mock.calls().is_empty() && core.garmin_changes().is_empty());
    drop(mock);

    let only_eight = settings(vec![alarm(8, 480, "OFF", &["ONCE"])]);
    let mock = Mock::new(vec![(200, "", two_alarms().to_string()), (200, "", String::new()), (200, "", only_eight.to_string()), (200, "", detail(1))]);
    let change = core.change_watch_alarms(&client(&mock.url), "3001", &remove, true, at()).await.unwrap();
    assert_eq!(change.outcome, WriteOutcome::Applied);
    assert_eq!(change.fields, vec![field("Alarm removed", Some("06:30"), None, "alarm:7")]);
    assert_eq!(mock.seen()[1].body, Some(json!({"deviceId":3001,"alarms":only_eight["alarms"]})));
}

#[tokio::test]
async fn nothing_is_sent_without_a_session_or_a_valid_change() {
    let (_dir, store, core) = fixture("refused");
    let mock = Mock::new(vec![]);
    assert!(core.change_watch_alarms(&client(&mock.url), "3001", &[], false, at()).await.is_err());
    assert!(core.change_watch_alarms(&client(&mock.url), "../3001", &[SEVEN_LATER], false, at()).await.is_err());
    store.suspend_account().unwrap();
    assert!(core.change_watch_alarms(&client(&mock.url), "3001", &[SEVEN_LATER], false, at()).await.is_err());
    assert!(mock.calls().is_empty() && core.garmin_changes().is_empty());
}

#[tokio::test]
async fn the_log_is_exported_and_erased_on_sign_out_and_account_switch() {
    let applied = || {
        let mut after = two_alarms();
        after["alarms"][0]["alarmTime"] = 420.into();
        vec![(200, "", two_alarms().to_string()), (204, "", String::new()), (200, "", after.to_string()), (200, "", detail(1))]
    };
    let (dir, store, core) = fixture("export");
    let mock = Mock::new(applied());
    core.change_watch_alarms(&client(&mock.url), "3001", &[SEVEN_LATER], false, at()).await.unwrap();
    let zip = dir.join("export.zip");
    core.export(&zip, "test").unwrap();
    let mut archive = zip::ZipArchive::new(std::fs::File::open(&zip).unwrap()).unwrap();
    let mut text = String::new();
    archive.by_name("settings.json").unwrap().read_to_string(&mut text).unwrap();
    let exported: Value = serde_json::from_str(&text).unwrap();
    let entry = &exported["garmin:changes"]["entries"][0];
    assert_eq!(exported["garmin:changes"]["schema"], 1);
    assert_eq!((entry["outcome"].as_str(), entry["fields"][0]["afterValue"].as_u64()), (Some("applied"), Some(420)));
    core.forget_account().unwrap();
    assert!(core.garmin_changes().is_empty());
    assert!(store.setting("garmin:changes").is_none());
    drop(mock);

    let (_dir, store, core) = fixture("switch");
    let mock = Mock::new(applied());
    core.change_watch_alarms(&client(&mock.url), "3001", &[SEVEN_LATER], false, at()).await.unwrap();
    assert!(store.bind_account("101", &json!({"profileId":101,"displayName":"fake-A"}), "later").unwrap());
    assert_eq!(core.garmin_changes().len(), 1, "the same account keeps its log");
    assert!(store.bind_account("202", &json!({"profileId":202,"displayName":"fake-B"}), "later").unwrap());
    assert!(core.garmin_changes().is_empty(), "another account never sees it");
}

// Weigh-ins (Rules: Garmin writes): created and deleted, never edited.
const ADD: &str = "/weight-service/user-weight";
// One request covers the day before to the day after.
const DAY: &str = "/weight-service/weight/dateRange?startDate=2026-10-20&endDate=2026-10-22";
const MORNING: &str = "2026-10-21T07:58:00-04:00";

fn weigh_fixture(name: &str, system: &str) -> (std::path::PathBuf, Store, Raring) {
    let (dir, store, core) = fixture(name);
    store.put("profile_settings", "me", "synthetic", &json!({"measurementSystem":system})).unwrap();
    (dir, store, core)
}
fn noon() -> DateTime<Utc> {
    "2026-10-21T16:00:00Z".parse().unwrap()
}
fn morning_ms() -> i64 {
    DateTime::parse_from_rfc3339(MORNING).unwrap().timestamp_millis()
}
/// A synthetic day entry as `weight/dateRange` lists it; `date` is local wall time as UTC.
fn weigh_in(pk: u64, gmt: i64, grams: f64, source: &str) -> Value {
    json!({"calendarDate":"2026-10-21","date":gmt - 4 * 3_600_000,"timestampGMT":gmt,"samplePk":pk,"sourceType":source,"weight":grams,"bmi":null,"bodyFat":null})
}
fn day(entries: Vec<Value>) -> String {
    json!({"startDate":"2026-10-21","endDate":"2026-10-21","dateWeightList":entries,"totalAverage":{}}).to_string()
}
fn add(value: f64, unit: WeightUnit) -> GarminWrite {
    GarminWrite::AddWeight { value, unit, at: MORNING.into() }
}

#[tokio::test]
async fn a_weigh_in_is_posted_once_found_by_its_timestamp_and_stored_for_health() {
    let (_dir, store, core) = weigh_fixture("weigh-add", "metric");
    let earlier = weigh_in(5, morning_ms() - 7_200_000, 80_900., "MANUAL");
    let created = weigh_in(9_001, morning_ms(), 81_300., "MANUAL");
    let mock = Mock::new(vec![
        (200, "", day(vec![earlier.clone()])),
        (204, "", String::new()),
        (200, "", day(vec![earlier, created])),
    ]);
    let change = core.write_garmin(&client(&mock.url), &add(81.3, WeightUnit::Kg), noon()).await.unwrap();
    assert_eq!(change.outcome, WriteOutcome::Applied, "{:?}", change.message);
    assert_eq!(change.resource, "Weight");
    assert_eq!(change.fields, vec![field("Weigh-in added", None, Some("81.3 kg at 07:58 on 21 Oct"), &format!("weighIn:{}", morning_ms()))]);
    assert_eq!(mock.calls(), [format!("GET {DAY}"), format!("POST {ADD}"), format!("GET {DAY}")]);
    assert_eq!(
        mock.seen()[1].body,
        Some(json!({"date":"2026-10-21","dateTimestamp":"2026-10-21T07:58:00.000","gmtTimestamp":"2026-10-21T11:58:00.000","unitKey":"kg","sourceType":"MANUAL","value":81.3}))
    );
    // Health shows it without a sync, with the id a later delete needs.
    assert_eq!(store.get("weight", "2026-10-21").unwrap()["samplePk"], 9_001);
    let today = chrono::Local.from_utc_datetime(&noon().naive_utc());
    let last = core.weight_trend(TrendRange::Days30, "2026-10-21", today).unwrap().last_weigh_in.unwrap();
    assert_eq!((last.value, last.time.as_deref(), last.sample_pk.as_deref(), last.timestamp_gmt), (81.3, Some("07:58"), Some("9001"), morning_ms()));
    assert_eq!(core.garmin_changes(), vec![change]);
}

#[tokio::test]
async fn pounds_are_sent_as_garmins_kilograms_and_compared_as_shown() {
    let (_dir, _store, core) = weigh_fixture("weigh-lb", "statute_us");
    // 165.0 lb is 74 842.74 g; Garmin keeping whole grams still reads 165.0 lb.
    let mock = Mock::new(vec![(200, "", day(vec![])), (204, "", String::new()), (200, "", day(vec![weigh_in(1, morning_ms(), 74_843., "MANUAL")]))]);
    let change = core.write_garmin(&client(&mock.url), &add(165., WeightUnit::Lb), noon()).await.unwrap();
    assert_eq!(change.outcome, WriteOutcome::Applied);
    assert_eq!(mock.seen()[1].body.as_ref().unwrap()["value"], json!(74.843));
    assert_eq!(change.fields[0].after.as_deref(), Some("165.0 lb at 07:58 on 21 Oct"));
    drop(mock);

    // Saved with another value: Garmin's reading is shown, not ours.
    let mock = Mock::new(vec![(200, "", day(vec![])), (204, "", String::new()), (200, "", day(vec![weigh_in(1, morning_ms(), 70_000., "MANUAL")]))]);
    let change = core.write_garmin(&client(&mock.url), &add(165., WeightUnit::Lb), noon()).await.unwrap();
    assert_eq!(change.outcome, WriteOutcome::Differs);
    assert_eq!(change.mismatches, vec![field("Weight", Some("165.0 lb"), Some("154.3 lb"), "weight")]);
}

#[tokio::test]
async fn the_same_value_that_day_is_not_sent_again() {
    let (_dir, _store, core) = weigh_fixture("weigh-dup", "metric");
    let mock = Mock::new(vec![(200, "", day(vec![weigh_in(5, morning_ms() - 3_600_000, 81_280., "INDEX_SCALE")]))]);
    let change = core.write_garmin(&client(&mock.url), &add(81.3, WeightUnit::Kg), noon()).await.unwrap();
    assert_eq!(change.outcome, WriteOutcome::Duplicate);
    assert_eq!(change.message.as_deref(), Some("Garmin already has 81.3 kg at 06:58 on 21 Oct that day"));
    assert_eq!(mock.calls(), [format!("GET {DAY}")]);
    drop(mock);

    // Another weigh-in at the very same moment can't be told apart afterwards.
    let mock = Mock::new(vec![(200, "", day(vec![weigh_in(5, morning_ms(), 85_000., "MANUAL")]))]);
    let change = core.write_garmin(&client(&mock.url), &add(81.3, WeightUnit::Kg), noon()).await.unwrap();
    assert_eq!(change.outcome, WriteOutcome::Conflict);
    assert_eq!(mock.calls().len(), 1);
}

#[tokio::test]
async fn a_sent_but_unconfirmed_weigh_in_is_confirmed_by_the_next_read_never_resent() {
    let (_dir, store, core) = weigh_fixture("weigh-unconfirmed", "metric");
    let mock = Mock::new(vec![(200, "", day(vec![])), (502, "", String::new())]);
    let change = core.write_garmin(&client(&mock.url), &add(81.3, WeightUnit::Kg), noon()).await.unwrap();
    assert_eq!(change.outcome, WriteOutcome::Unconfirmed);
    assert_eq!(change.message.as_deref(), Some("Garmin answered with error 502; Garmin may or may not have saved it"));
    assert_eq!(mock.calls(), [format!("GET {DAY}"), format!("POST {ADD}")], "no automatic resend or read-back");
    drop(mock);

    // The person tries again with the same weigh-in: Garmin had saved it.
    let mock = Mock::new(vec![(200, "", day(vec![weigh_in(9_001, morning_ms(), 81_300., "MANUAL")]))]);
    let change = core.write_garmin(&client(&mock.url), &add(81.3, WeightUnit::Kg), noon()).await.unwrap();
    assert_eq!(change.outcome, WriteOutcome::Applied);
    assert_eq!(change.message.as_deref(), Some("Already in Garmin from an earlier attempt; not sent again"));
    assert_eq!(mock.calls(), [format!("GET {DAY}")]);
    assert_eq!(store.get("weight", "2026-10-21").unwrap()["samplePk"], 9_001);
    drop(mock);

    // Accepted, but the read-back failed or doesn't list it yet.
    let (_dir, _store, core) = weigh_fixture("weigh-readback", "metric");
    let mock = Mock::new(vec![(200, "", day(vec![])), (204, "", String::new()), (503, "", String::new())]);
    let change = core.write_garmin(&client(&mock.url), &add(81.3, WeightUnit::Kg), noon()).await.unwrap();
    assert_eq!((change.outcome, change.message.as_deref()), (WriteOutcome::Unconfirmed, Some("Saved, but Garmin's read-back failed")));
    drop(mock);
    let mock = Mock::new(vec![(200, "", day(vec![])), (204, "", String::new()), (200, "", day(vec![]))]);
    let change = core.write_garmin(&client(&mock.url), &add(81.3, WeightUnit::Kg), noon()).await.unwrap();
    assert_eq!((change.outcome, change.message.as_deref()), (WriteOutcome::Unconfirmed, Some("Garmin accepted it, but its weigh-ins don't show it yet")));
    assert_eq!(mock.calls().len(), 3);
}

#[tokio::test]
async fn a_refused_weigh_in_keeps_garmins_message() {
    let (_dir, _store, core) = weigh_fixture("weigh-rejected", "metric");
    let mock = Mock::new(vec![(200, "", day(vec![])), (400, "", json!({"message":"Invalid weight value"}).to_string())]);
    let change = core.write_garmin(&client(&mock.url), &add(81.3, WeightUnit::Kg), noon()).await.unwrap();
    assert_eq!((change.outcome, change.message.as_deref()), (WriteOutcome::Rejected, Some("Invalid weight value")));
    assert_eq!(mock.calls().len(), 2, "nothing read back");
}

#[tokio::test]
async fn a_weigh_in_refreshes_sign_in_once_and_backs_off_on_rate_limits() {
    let (_dir, store, core) = weigh_fixture("weigh-auth", "metric");
    let refreshed = json!({"access_token":token("second"),"refresh_token":"refresh-2","expires_in":3600}).to_string();
    let mock = Mock::new(vec![
        (200, "", day(vec![])),
        (401, "", String::new()),
        (200, "", refreshed),
        (204, "", String::new()),
        (200, "", day(vec![weigh_in(1, morning_ms(), 81_300., "MANUAL")])),
    ]);
    let session = client(&mock.url);
    let change = core.write_garmin(&session, &add(81.3, WeightUnit::Kg), noon()).await.unwrap();
    assert_eq!(change.outcome, WriteOutcome::Applied);
    let seen = mock.seen();
    // A 401 saved nothing, so the same weigh-in goes once more with the new token.
    assert_eq!(seen.iter().map(|s| s.method.as_str()).collect::<Vec<_>>(), ["GET", "POST", "POST", "POST", "GET"]);
    assert!(seen[1].auth.ends_with(".first") && seen[3].auth.ends_with(".second"));
    assert_eq!(seen[1].body, seen[3].body);
    assert_eq!(session.tokens().await.refresh_token, "refresh-2");
    drop(mock);

    let mock = Mock::new(vec![(200, "", day(vec![])), (429, "Retry-After: 120\r\n", String::new())]);
    let change = core.write_garmin(&client(&mock.url), &add(81.4, WeightUnit::Kg), noon()).await.unwrap();
    assert_eq!((change.outcome, change.message.as_deref()), (WriteOutcome::Failed, Some("Garmin is limiting requests; try again later")));
    assert_eq!(store.cursor("backoff_until"), Some((noon().timestamp() + 120).to_string()));
    let change = core.write_garmin(&client(&mock.url), &add(81.4, WeightUnit::Kg), noon() + chrono::Duration::seconds(60)).await.unwrap();
    assert_eq!(change.outcome, WriteOutcome::Failed);
    assert_eq!(mock.calls().len(), 2, "no request during the backoff");
}

#[tokio::test]
async fn invalid_weigh_ins_are_refused_before_any_request() {
    let (_dir, store, core) = weigh_fixture("weigh-invalid", "metric");
    let mock = Mock::new(vec![]);
    let session = client(&mock.url);
    for request in [
        add(f64::NAN, WeightUnit::Kg),
        add(12., WeightUnit::Kg),
        add(700., WeightUnit::Lb),
        GarminWrite::AddWeight { value: 81.3, unit: WeightUnit::Kg, at: "2026-10-21T07:58".into() },
        GarminWrite::AddWeight { value: 81.3, unit: WeightUnit::Kg, at: "2026-10-21T13:58:00-04:00".into() },
        GarminWrite::DeleteWeighIn { date: "2026-10-21".into(), timestamp_gmt: morning_ms(), sample_pk: None, confirm: false },
        GarminWrite::DeleteWeighIn { date: "2026-10-21".into(), timestamp_gmt: morning_ms(), sample_pk: Some("../1".into()), confirm: true },
    ] {
        assert!(core.write_garmin(&session, &request, noon()).await.is_err(), "{request:?}");
    }
    store.suspend_account().unwrap();
    assert!(core.write_garmin(&session, &add(81.3, WeightUnit::Kg), noon()).await.is_err());
    assert!(mock.calls().is_empty() && core.garmin_changes().is_empty());
}

#[tokio::test]
async fn a_confirmed_delete_addresses_the_seen_weigh_in_and_the_local_copy_follows() {
    let (_dir, store, core) = weigh_fixture("weigh-delete", "metric");
    let earlier = weigh_in(5, morning_ms() - 7_200_000, 80_900., "MANUAL");
    let later = weigh_in(9_001, morning_ms(), 81_300., "MANUAL");
    store.put("weight", "2026-10-21", "synthetic", &later).unwrap();
    store.put("weight_latest", "me", "synthetic", &json!({"date":later["date"],"timestampGMT":morning_ms(),"weight":81_300.,"version":9_001})).unwrap();
    let mock = Mock::new(vec![(200, "", day(vec![earlier.clone(), later.clone()])), (204, "", String::new()), (200, "", day(vec![earlier.clone()]))]);
    let delete = GarminWrite::DeleteWeighIn { date: "2026-10-21".into(), timestamp_gmt: morning_ms(), sample_pk: Some("9001".into()), confirm: true };
    let change = core.write_garmin(&client(&mock.url), &delete, noon()).await.unwrap();
    assert_eq!(change.outcome, WriteOutcome::Applied, "{:?}", change.message);
    assert_eq!(change.fields, vec![field("Weigh-in deleted", Some("81.3 kg at 07:58 on 21 Oct"), None, &format!("weighIn:{}", morning_ms()))]);
    assert_eq!(mock.calls(), [format!("GET {DAY}"), "DELETE /weight-service/weight/2026-10-21/byversion/9001".into(), format!("GET {DAY}")]);
    assert_eq!(mock.seen()[1].body, None);
    // The day falls back to the earlier weigh-in, and the stale latest is dropped for the next sync.
    assert_eq!(store.get("weight", "2026-10-21").unwrap()["samplePk"], 5);
    assert!(store.get("weight_latest", "me").is_none());
    assert_eq!(store.cursor("weight_latest_checked").as_deref(), Some("0"));
    drop(mock);

    // Retrying a delete Garmin already did sends nothing; the only weigh-in left
    // is then deleted and the day is empty.
    let mock = Mock::new(vec![(200, "", day(vec![earlier.clone()]))]);
    let change = core.write_garmin(&client(&mock.url), &delete, noon()).await.unwrap();
    assert_eq!((change.outcome, change.message.as_deref()), (WriteOutcome::Conflict, Some("Already gone from Garmin")));
    assert_eq!(mock.calls().len(), 1);
    drop(mock);
    let mock = Mock::new(vec![(200, "", day(vec![earlier.clone()])), (204, "", String::new()), (200, "", day(vec![]))]);
    let first = GarminWrite::DeleteWeighIn { date: "2026-10-21".into(), timestamp_gmt: morning_ms() - 7_200_000, sample_pk: Some("5".into()), confirm: true };
    assert_eq!(core.write_garmin(&client(&mock.url), &first, noon()).await.unwrap().outcome, WriteOutcome::Applied);
    assert!(store.get("weight", "2026-10-21").is_none());
    drop(mock);

    // A server failure on the delete is unconfirmed and keeps the local copy.
    store.put("weight", "2026-10-21", "synthetic", &later).unwrap();
    let mock = Mock::new(vec![(200, "", day(vec![later.clone()])), (500, "", String::new())]);
    let change = core.write_garmin(&client(&mock.url), &delete, noon()).await.unwrap();
    assert_eq!(change.outcome, WriteOutcome::Unconfirmed);
    assert_eq!(store.get("weight", "2026-10-21").unwrap()["samplePk"], 9_001);
    assert_eq!(mock.calls().len(), 2);
}

/// The same entry filed under another calendar date, as when the profile's time zone differs.
fn filed(mut entry: Value, date: &str) -> Value {
    entry["calendarDate"] = date.into();
    entry
}

#[tokio::test]
async fn an_attempt_filed_under_a_neighbouring_date_is_still_found() {
    let (_dir, store, core) = weigh_fixture("weigh-around", "metric");
    let created = filed(weigh_in(9_001, morning_ms(), 81_300., "MANUAL"), "2026-10-22");
    let mock = Mock::new(vec![(200, "", day(vec![])), (204, "", String::new()), (200, "", day(vec![created.clone()]))]);
    let change = core.write_garmin(&client(&mock.url), &add(81.3, WeightUnit::Kg), noon()).await.unwrap();
    assert_eq!(change.outcome, WriteOutcome::Applied, "{:?}", change.message);
    assert_eq!(store.get("weight", "2026-10-22").unwrap()["samplePk"], 9_001, "stored under Garmin's date");
    assert!(store.get("weight", "2026-10-21").is_none());
    drop(mock);

    // A retry finds that earlier attempt on the other date and sends nothing.
    let mock = Mock::new(vec![(200, "", day(vec![created]))]);
    let change = core.write_garmin(&client(&mock.url), &add(81.3, WeightUnit::Kg), noon()).await.unwrap();
    assert_eq!(change.outcome, WriteOutcome::Applied);
    assert_eq!(mock.calls(), [format!("GET {DAY}")]);
}

#[tokio::test]
async fn the_same_value_on_a_neighbouring_day_is_not_a_duplicate_and_an_empty_answer_is_an_empty_day() {
    let (_dir, _store, core) = weigh_fixture("weigh-neighbour", "metric");
    let yesterday = filed(weigh_in(5, morning_ms() - 86_400_000, 81_300., "MANUAL"), "2026-10-20");
    let mock = Mock::new(vec![(200, "", day(vec![yesterday.clone()])), (204, "", String::new()), (200, "", day(vec![yesterday, weigh_in(6, morning_ms(), 81_300., "MANUAL")]))]);
    let change = core.write_garmin(&client(&mock.url), &add(81.3, WeightUnit::Kg), noon()).await.unwrap();
    assert_eq!(change.outcome, WriteOutcome::Applied);
    assert_eq!(mock.calls().len(), 3);
    drop(mock);

    for empty in [404, 204] {
        let (_dir, _store, core) = weigh_fixture(&format!("weigh-empty-{empty}"), "metric");
        let mock = Mock::new(vec![(empty, "", String::new()), (204, "", String::new()), (200, "", day(vec![weigh_in(6, morning_ms(), 81_300., "MANUAL")]))]);
        let change = core.write_garmin(&client(&mock.url), &add(81.3, WeightUnit::Kg), noon()).await.unwrap();
        assert_eq!(change.outcome, WriteOutcome::Applied, "{empty}: {:?}", change.message);
        assert_eq!(mock.calls()[1], format!("POST {ADD}"));
    }
}

#[tokio::test]
async fn a_delete_is_logged_with_its_delete_path() {
    let (_dir, store, core) = weigh_fixture("weigh-delete-path", "metric");
    let mock = Mock::new(vec![(200, "", day(vec![]))]);
    let delete = GarminWrite::DeleteWeighIn { date: "2026-10-21".into(), timestamp_gmt: morning_ms(), sample_pk: Some("9001".into()), confirm: true };
    assert_eq!(core.write_garmin(&client(&mock.url), &delete, noon()).await.unwrap().outcome, WriteOutcome::Conflict);
    let log: Value = serde_json::from_str(&store.setting("garmin:changes").unwrap()).unwrap();
    assert_eq!(log["entries"][0]["path"], "/weight-service/weight/2026-10-21/byversion/9001");
}
