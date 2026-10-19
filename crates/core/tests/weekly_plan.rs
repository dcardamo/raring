use raring_core::{
    store::Store,
    view::{PlanPreset, PlanState, PlanTargetKind, PlanTargets, PlanUpdate},
    Raring,
};
use serde_json::{json, Value};

// Synthetic week: Monday 10 to Sunday 16 March 2025, viewed on Thursday 13.
const TODAY: &str = "2025-03-13";

fn summary(store: &Store, date: &str, moderate: u32, vigorous: u32) {
    store
        .put(
            "summary",
            date,
            "now",
            &json!({"moderateIntensityMinutes": moderate, "vigorousIntensityMinutes": vigorous,
                    "intensityMinutesGoal": 150}),
        )
        .unwrap();
}

fn workout(store: &Store, id: &str, start: &str, kind: &str, zones: [u32; 5]) {
    store
        .put(
            "activity",
            id,
            "now",
            &json!({"activityId": id.parse::<u64>().unwrap(), "activityName": "Workout",
                    "activityTypeDTO": {"typeKey": kind},
                    "summaryDTO": {"startTimeLocal": start, "duration": 1800}}),
        )
        .unwrap();
    let zones: Vec<Value> = zones
        .iter()
        .enumerate()
        .map(|(i, s)| json!({"zoneNumber": i + 1, "secsInZone": s}))
        .collect();
    store.put("zones", id, "now", &json!(zones)).unwrap();
}

/// A night ending on `morning`, asleep at `start` (local), carrying Garmin's
/// window for the following night.
fn night(store: &Store, morning: &str, start: Option<&str>, window: (u32, u32)) {
    let mut dto = json!({"nextSleepNeed": {"actual": 480,
        "recommendedBedtimeStartMins": window.0, "recommendedBedtimeEndMins": window.1}});
    if let Some(start) = start {
        dto["sleepStartTimestampLocal"] = json!(start);
        dto["sleepTimeSeconds"] = json!(25000);
    }
    store
        .put("sleep", morning, "now", &json!({"dailySleepDTO": dto}))
        .unwrap();
}

fn targets() -> PlanTargets {
    PlanTargets {
        rest_days: vec![0, 4],
        ..PlanTargets::preset(PlanPreset::Strength)
    }
}

fn capture(store: &Store, evening: &str) {
    use chrono::TimeZone;
    let t =
        chrono::NaiveDateTime::parse_from_str(&format!("{evening}T18:00:00"), "%Y-%m-%dT%H:%M:%S")
            .unwrap();
    raring_core::plan::capture_sleep_plan(
        store,
        chrono::Local.from_local_datetime(&t).single().unwrap(),
    )
    .unwrap();
}

fn listed(store: &Store, from: &str, through: &str) {
    store.set_cursor("listed:lo", from).unwrap();
    store.set_cursor("listed:hi", through).unwrap();
}

fn seed(store: &Store) {
    listed(store, "2025-03-03", "2025-03-16");
    for d in [
        "2025-03-03",
        "2025-03-04",
        "2025-03-06",
        "2025-03-07",
        "2025-03-08",
        "2025-03-09",
        "2025-03-14",
        "2025-03-15",
        "2025-03-16",
    ] {
        summary(store, d, 0, 0);
    }
    summary(store, "2025-03-10", 10, 5);
    summary(store, "2025-03-11", 0, 0);
    summary(store, "2025-03-12", 20, 9);
    summary(store, "2025-03-13", 4, 0);
    workout(
        store,
        "101",
        "2025-03-11 18:00:00",
        "strength_training",
        [600, 300, 0, 0, 0],
    );
    workout(
        store,
        "102",
        "2025-03-12 07:00:00",
        "running",
        [100, 2400, 600, 0, 0],
    );
    // A workout from the previous week never counts.
    workout(
        store,
        "100",
        "2025-03-09 18:00:00",
        "strength_training",
        [0, 600, 0, 0, 0],
    );
    // Garmin's window is 22:30–23:00 for every night.
    let window = (1350, 1380);
    night(store, "2025-03-09", Some("2025-03-08T22:35:00"), window);
    capture(store, "2025-03-09");
    night(store, "2025-03-10", Some("2025-03-09T22:40:00"), window);
    capture(store, "2025-03-10");
    night(store, "2025-03-11", Some("2025-03-11T00:05:00"), window);
    capture(store, "2025-03-11");
    night(store, "2025-03-12", Some("2025-03-11T23:10:00"), window);
    capture(store, "2025-03-12");
    // Last night's sleep hasn't synced: left out, not a miss.
    night(store, "2025-03-13", None, window);
    // The week before: 240 intensity minutes and Sunday's session.
    summary(store, "2025-03-05", 30, 105);
}

fn save(core: &Raring, paused: bool) {
    core.set_plan(
        TODAY,
        &PlanUpdate {
            targets: targets(),
            paused,
        },
    )
    .unwrap();
}

fn open() -> (tempdir::Dir, Raring) {
    let dir = tempdir::Dir::new();
    let core = Raring::open(dir.path().join("raring.db")).unwrap();
    (dir, core)
}

mod tempdir {
    pub struct Dir(std::path::PathBuf);
    impl Dir {
        pub fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "raring-plan-{}-{:?}",
                std::process::id(),
                std::thread::current().id()
            ));
            let _ = std::fs::remove_dir_all(&path);
            std::fs::create_dir_all(&path).unwrap();
            Self(path)
        }
        pub fn path(&self) -> &std::path::Path {
            &self.0
        }
    }
    impl Drop for Dir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
}

fn store_of(dir: &tempdir::Dir) -> Store {
    Store::open(dir.path().join("raring.db")).unwrap()
}

#[test]
fn plan_is_off_until_targets_are_saved() {
    let (dir, core) = open();
    seed(&store_of(&dir));
    assert_eq!(core.today(TODAY).plan, None);
    let view = core.plan(TODAY);
    assert!(!view.configured);
    assert_eq!(view.targets, PlanTargets::preset(PlanPreset::Strength));
    assert_eq!(view.garmin_intensity_goal, Some(150.));
    assert_eq!(core.checkin("2025-03-17").plan, None);
}

#[test]
fn this_week_measures_garmin_data_against_pace() {
    let (dir, core) = open();
    seed(&store_of(&dir));
    save(&core, false);
    let week = core.today(TODAY).plan.unwrap();
    assert_eq!(
        (week.start.as_str(), week.end.as_str()),
        ("2025-03-10", "2025-03-16")
    );
    assert_eq!(week.days_left, 3);
    let item = |kind| week.items.iter().find(|i| i.kind == kind).unwrap();

    // Training days are Tue, Wed, Thu, Sat and Sun; two have passed.
    let strength = item(PlanTargetKind::Strength);
    assert_eq!((strength.value, strength.target), (1., 3.));
    assert!((strength.pace - 0.4).abs() < 1e-9);
    assert_eq!(strength.state, PlanState::Behind);
    assert_eq!(strength.note, "Tue");

    // Garmin's weekly total counts vigorous minutes twice.
    let intensity = item(PlanTargetKind::Intensity);
    assert_eq!((intensity.value, intensity.target), (62., 150.));
    assert_eq!(intensity.state, PlanState::OnTrack);
    assert_eq!(intensity.note, "Garmin's weekly goal · 88 min to go");

    // Zone 2 from each activity's heart-rate zones, both sports.
    let zone2 = item(PlanTargetKind::Zone2);
    assert_eq!(zone2.value, 45.);
    assert_eq!(zone2.state, PlanState::OnTrack);

    // Mon on time inside the window, Tue late, Wed within 15 minutes of its
    // end; Thu's sleep hasn't synced, so three of six possible nights passed.
    let nights = item(PlanTargetKind::AsleepOnTime);
    assert_eq!(nights.value, 2.);
    assert!((nights.pace - 0.5).abs() < 1e-9);
    assert_eq!(nights.state, PlanState::Behind);
    assert_eq!(nights.note, "Mon, Wed");

    assert_eq!((week.total, week.on_track, week.reached), (4, 2, 0));
    assert_eq!(week.headline, "On track");
    assert_eq!(
        week.detail,
        "3 days left · Strength and Asleep on time are behind"
    );
    let expected = (1. / 3. + 62. / 150. + 0.5 + 0.4) / 4.;
    assert!((week.completion.unwrap() - expected).abs() < 1e-9);

    let health = core.health(TODAY).intensity.unwrap();
    assert_eq!(
        (health.total, health.moderate, health.vigorous),
        (62., 34., 14.)
    );
    assert_eq!(health.goal, Some(150.));
}

#[test]
fn history_caps_each_target_and_skips_paused_weeks() {
    let (dir, core) = open();
    seed(&store_of(&dir));
    core.set_plan(
        "2025-03-03",
        &PlanUpdate {
            targets: targets(),
            paused: false,
        },
    )
    .unwrap();
    save(&core, false);
    let view = core.plan(TODAY);
    assert!(view.configured);
    assert_eq!(view.history.len(), 8);
    let last = &view.history[6];
    assert_eq!(last.start, "2025-03-03");
    // 240 intensity minutes count as 100%, not 160%; without a planned
    // night that week, asleep-on-time is left out.
    let expected = (1. / 3. + 1. + 10. / 90.) / 3.;
    assert!((last.completion.unwrap() - expected).abs() < 1e-9);
    assert!(view.history[7].current);
    // Weeks without any Garmin data are gaps, never zero.
    assert_eq!(view.history[0].completion, None);
    assert_eq!(view.last_week.start, "2025-03-03");

    save(&core, true);
    let view = core.plan(TODAY);
    assert!(view.week.paused);
    assert_eq!(view.week.headline, "Paused");
    assert_eq!(view.history[7].completion, None);
    assert!(view.history[7].paused);
}

#[test]
fn changed_targets_apply_from_their_week_on() {
    let (dir, core) = open();
    seed(&store_of(&dir));
    save(&core, false);
    let mut later = targets();
    later.strength_sessions = 1;
    later.preset = PlanPreset::Custom;
    later.rest_days = vec![6];
    core.set_plan(
        "2025-03-18",
        &PlanUpdate {
            targets: later,
            paused: false,
        },
    )
    .unwrap();
    // This week keeps its own targets.
    let week = core.plan(TODAY).week;
    assert_eq!(week.items[0].target, 3.);

    let checkin = core.checkin("2025-03-17").plan.unwrap();
    assert_eq!(checkin.week.start, "2025-03-10");
    assert_eq!(checkin.week.headline, "No targets reached");
    assert_eq!(checkin.week.reached, 0);
    assert_eq!(checkin.week.detail, "4 targets came up short");
    // The week before predates the plan; comparisons must stay absent.
    assert_eq!(checkin.previous, vec![None, None, None, None]);
    assert_eq!(checkin.next, "New targets · rest Sun");
    let intensity = core.checkin("2025-03-17").intensity.unwrap();
    assert_eq!(intensity.start, "2025-03-10");
}

#[test]
fn weeks_follow_garmins_intensity_week() {
    let (dir, core) = open();
    let store = store_of(&dir);
    seed(&store);
    // First setup precedes the weekly-intensity sync, so its key uses Monday.
    assert!(store.get("intensity_weeks", "me").is_none());
    save(&core, false);
    let view = core.plan(TODAY);
    assert_eq!(view.week_starts_on, 0);
    assert_eq!(view.week.start, "2025-03-10");
    // The plan's total is Garmin's row for the week: 34 + 2 × 14.
    let intensity = &view.week.items[1];
    assert_eq!(intensity.value, 34. + 2. * 14.);

    // The first synced rows then establish Sunday as the intensity boundary.
    let rows: Value = serde_json::from_str(include_str!("fixtures/intensity_weeks.json")).unwrap();
    let sunday: Vec<Value> = rows
        .as_array()
        .unwrap()
        .iter()
        .map(|r| {
            let mut r = r.clone();
            let d =
                chrono::NaiveDate::parse_from_str(r["calendarDate"].as_str().unwrap(), "%Y-%m-%d")
                    .unwrap()
                    - chrono::Duration::days(1);
            r["calendarDate"] = json!(d.to_string());
            r
        })
        .collect();
    store
        .put("intensity_weeks", "me", "now", &json!(sunday))
        .unwrap();
    let view = core.plan(TODAY);
    assert_eq!(view.week_starts_on, 6);
    assert_eq!(view.week.start, "2025-03-09");
    // Sunday's strength session is now this week's.
    assert_eq!(view.week.items[0].value, 2.);
    // The saved Monday version belongs to this Sunday-to-Saturday week.
    assert!(view.configured);
    // Weekly results read the migrated goals; the retired setup screen's preset isn't kept.
    assert_eq!(view.targets, PlanTargets { preset: PlanPreset::Custom, ..targets() });
    assert_eq!(core.today(TODAY).plan.unwrap().items[0].target, 3.);
    assert_eq!(
        core.checkin("2025-03-17").plan.unwrap().week.start,
        "2025-03-09"
    );
    // No first-version fallback: earlier weeks still have no plan or history.
    assert!(view.history[..7].iter().all(|w| w.completion.is_none()));
    assert!(view.history[7].completion.is_some());
    assert!(core.checkin("2025-03-10").plan.is_none());
}

#[test]
fn zero_targets_are_off_and_custom_intensity_replaces_garmins() {
    let (dir, core) = open();
    seed(&store_of(&dir));
    let mut custom = targets();
    custom.strength_sessions = 0;
    custom.intensity_min = Some(60);
    core.set_plan(
        TODAY,
        &PlanUpdate {
            targets: custom,
            paused: false,
        },
    )
    .unwrap();
    let week = core.plan(TODAY).week;
    assert_eq!(week.items.len(), 3);
    let intensity = &week.items[0];
    assert_eq!(
        (intensity.kind, intensity.target),
        (PlanTargetKind::Intensity, 60.)
    );
    assert_eq!(intensity.state, PlanState::Done);
    assert_eq!(intensity.note, "Your target · Reached");
}

#[test]
fn on_time_windows_cross_midnight() {
    let (dir, core) = open();
    let store = store_of(&dir);
    // Window 23:45–00:15 (Garmin counts minutes from the evening's midnight);
    // 15 minutes either side still counts.
    let window = (1425, 1455);
    night(&store, "2025-03-09", Some("2025-03-08T23:50:00"), window);
    for (morning, start) in [
        ("2025-03-10", "2025-03-10T00:28:00"),
        ("2025-03-11", "2025-03-10T23:31:00"),
        ("2025-03-12", "2025-03-11T23:20:00"),
        ("2025-03-13", "2025-03-13T00:40:00"),
    ] {
        let before = chrono::NaiveDate::parse_from_str(morning, "%Y-%m-%d").unwrap()
            - chrono::Duration::days(1);
        capture(&store, &before.to_string());
        night(&store, morning, Some(start), window);
    }
    core.set_plan(
        TODAY,
        &PlanUpdate {
            targets: PlanTargets::preset(PlanPreset::Sleep),
            paused: false,
        },
    )
    .unwrap();
    let week = core.plan(TODAY).week;
    let nights = week
        .items
        .iter()
        .find(|i| i.kind == PlanTargetKind::AsleepOnTime)
        .unwrap();
    assert_eq!(nights.value, 2.);
    assert_eq!(nights.note, "Mon, Tue");
}

fn mock(replies: Vec<String>) -> (String, std::thread::JoinHandle<Vec<String>>) {
    use std::io::{Read, Write};
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let handle = std::thread::spawn(move || {
        let mut seen = vec![];
        for body in replies {
            let (mut stream, _) = listener.accept().unwrap();
            let mut bytes = [0; 8192];
            let length = stream.read(&mut bytes).unwrap();
            let request = String::from_utf8_lossy(&bytes[..length]).into_owned();
            seen.push(request.lines().next().unwrap_or_default().to_owned());
            let response = format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len());
            stream.write_all(response.as_bytes()).unwrap();
        }
        seen
    });
    (url, handle)
}

fn client(url: &str) -> raring_core::garmin::Client {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    let token = format!(
        "e30.{}.sig",
        URL_SAFE_NO_PAD.encode(json!({"exp":1900000000,"client_id":"TEST_DI"}).to_string())
    );
    raring_core::garmin::Client::with_urls(
        raring_core::Tokens {
            access_token: token,
            refresh_token: "refresh".into(),
            client_id: "TEST_DI".into(),
            expires_at: chrono::Utc::now() + chrono::Duration::days(30),
        },
        url,
        url,
        std::time::Duration::ZERO,
    )
    .unwrap()
}

#[tokio::test]
async fn activity_scan_tracks_only_fresh_complete_listed_days() {
    use chrono::{Duration, Local, TimeZone, Utc};
    let today: chrono::NaiveDate = "2025-04-10".parse().unwrap();
    let now = Local
        .from_local_datetime(&today.and_hms_opt(15, 0, 0).unwrap())
        .single()
        .unwrap();
    // Empty is evidence of zero. A full page covers only days strictly newer
    // than its oldest entry (that day may continue on the next page).
    let full = json!((0..20)
        .map(|i| json!({"activityId":300+i,"startTimeLocal":"2025-04-08T12:00:00"}))
        .collect::<Vec<_>>());
    for (page, expected) in [
        (json!([]), true),
        (full, false),
        (json!([{"activityId":300}]), false),
    ] {
        let store = Store::open(":memory:").unwrap();
        put_for_scan(&store, today);
        let mut replies = vec!["{}".to_owned(); 13];
        replies.push(page.to_string());
        let (url, handle) = mock(replies);
        let report = raring_core::sync::sync(&store, &client(&url), now, 14).await;
        handle.join().unwrap();
        assert_eq!(report.requests, 14);
        assert_eq!(
            raring_core::sync::activities_listed(&store, today - Duration::days(3), today),
            expected
        );
        if page.as_array().unwrap().len() == 20 {
            assert!(raring_core::sync::activities_listed(
                &store,
                today - Duration::days(1),
                today
            ));
        }
        // Cached pages during a retry wait must not extend coverage.
        store.set_cursor("listed:lo", "2025-04-01").unwrap();
        store.set_cursor("listed:hi", "2025-04-06").unwrap();
        store
            .set_cursor(
                "retry:activities:0",
                &(Utc::now().timestamp() + 3600).to_string(),
            )
            .unwrap();
        let (url, handle) = mock(vec!["{}".to_owned(); 14]);
        raring_core::sync::sync(&store, &client(&url), now, 14).await;
        handle.join().unwrap();
        assert_eq!(store.cursor("listed:hi").as_deref(), Some("2025-04-06"));
    }
}

fn put_for_scan(store: &Store, today: chrono::NaiveDate) {
    for i in 0..2 {
        store.set_cursor(&format!("retry:daily_hr:{}", today - chrono::Duration::days(i)), &(chrono::Utc::now().timestamp() + 3600).to_string()).unwrap();
    }
    store
        .put(
            "profile",
            "me",
            "synthetic",
            &json!({"displayName":"Synthetic"}),
        )
        .unwrap();
    for kind in [
        "fitness_age",
        "races",
        "endurance",
        "hill",
        "tolerance",
        "sleep_stats",
    ] {
        store
            .put(kind, &today.to_string(), "synthetic", &json!({}))
            .unwrap();
    }
    store
        .put("records", "all", "synthetic", &json!({}))
        .unwrap();
    for key in [
        "records_checked",
        "devices_checked",
        "intensity_weeks_checked",
        "weight_latest_checked",
        "weight_recent_checked",
        "gear_checked",
        "lactate_threshold_checked",
        "running_power_threshold_checked",
        "cycling_ftp_checked",
        "acclimation_checked",
    ] {
        store
            .set_cursor(key, &chrono::Utc::now().timestamp().to_string())
            .unwrap();
    }
}

#[tokio::test]
async fn garmin_intensity_weeks_are_read_daily() {
    use chrono::TimeZone;
    let store = Store::open(":memory:").unwrap();
    store
        .put("profile", "me", "now", &json!({"displayName": "Synthetic"}))
        .unwrap();
    // Skip the device list; an afternoon run fetches one day of summaries.
    store
        .set_cursor(
            "devices_checked",
            &chrono::Utc::now().timestamp().to_string(),
        )
        .unwrap();
    let afternoon = chrono::Local::now()
        .date_naive()
        .and_hms_opt(15, 0, 0)
        .and_then(|t| chrono::Local.from_local_datetime(&t).single())
        .unwrap();
    let rows: Value = serde_json::from_str(include_str!("fixtures/intensity_weeks.json")).unwrap();
    for i in 0..2 {
        store.set_cursor(&format!("retry:daily_hr:{}", afternoon.date_naive() - chrono::Duration::days(i)), &(chrono::Utc::now().timestamp() + 3600).to_string()).unwrap();
    }
    // The weekly reply is the last of the run; earlier requests get "{}".
    let mut replies = vec!["{}".to_owned(); 20];
    replies.push(rows.to_string());
    let (url, handle) = mock(replies);
    let report = raring_core::sync::sync(&store, &client(&url), afternoon, 21).await;
    let seen = handle.join().unwrap();
    assert_eq!(report.requests, 21);
    assert!(
        seen[20].contains("/usersummary-service/stats/im/weekly/"),
        "{seen:?}"
    );
    assert_eq!(store.get("intensity_weeks", "me"), Some(rows));

    // Within a day the run moves on to activities without asking again.
    // Isolate the intensity-week cadence from latest-only and nap reads.
    for i in 0..2 {
        let day = (afternoon.date_naive() - chrono::Duration::days(i)).to_string();
        store.put("naps", &day, &chrono::Utc::now().to_rfc3339(), &json!([])).unwrap();
    }
    for kind in ["lactate_threshold", "running_power_threshold", "cycling_ftp", "acclimation"] {
        store.set_cursor(&format!("{kind}_checked"), &chrono::Utc::now().timestamp().to_string()).unwrap();
    }
    let (url, handle) = mock(vec!["{}".to_owned(); 20]);
    let report = raring_core::sync::sync(&store, &client(&url), afternoon, 20).await;
    let seen = handle.join().unwrap();
    assert_eq!(report.requests, 20);
    assert!(seen.iter().any(|r| r.contains("/activitylist-service/")));
    assert!(!seen.iter().any(|r| r.contains("/stats/im/weekly/")));
}
