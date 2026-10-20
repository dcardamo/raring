//! Sleep Alignment, Garmin's consistency averages, the shared plan's coaching
//! fields and the wind-down reminder. All values are synthetic.
use chrono::{Duration, NaiveDate, NaiveDateTime};
use raring_core::{
    notifications, screens,
    store::Store,
    view::{AlignmentState, BedtimeSource, SleepNeedSource},
};
use serde_json::{json, Value};

const DATE: &str = "2025-03-12";

fn day(offset: i64) -> String {
    (NaiveDate::parse_from_str(DATE, "%Y-%m-%d").unwrap() + Duration::days(offset)).to_string()
}
fn at(date: &str, time: &str) -> NaiveDateTime {
    NaiveDateTime::parse_from_str(&format!("{date}T{time}:00"), "%Y-%m-%dT%H:%M:%S").unwrap()
}
/// A night that ends on `date`: asleep at `bed` the evening before (or after
/// midnight when earlier than noon), awake at `wake`.
fn night(date: &str, bed: &str, wake: &str) -> Value {
    let bed_day = if bed < "12:00" {
        date.to_owned()
    } else {
        (NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap() - Duration::days(1)).to_string()
    };
    json!({"dailySleepDTO": {
        "calendarDate": date,
        "sleepTimeSeconds": 25000,
        "sleepStartTimestampLocal": format!("{bed_day}T{bed}:00"),
        "sleepEndTimestampLocal": format!("{date}T{wake}:00"),
        "sleepNeed": {"actual": 450, "baseline": 450},
    }})
}
fn put(store: &Store, date: &str, value: &Value) {
    store.put("sleep", date, "now", value).unwrap();
}
fn aligned(mut v: Value, status: &str, window: Option<(i64, i64)>) -> Value {
    v["dailySleepDTO"]["sleepAlignment"] = json!({
        "status": status,
        "optimalSleepWindowStartMins": window.map(|w| w.0),
        "optimalSleepWindowEndMins": window.map(|w| w.1),
        "sleepNeedBaselineMins": 450,
    });
    v
}
fn with_next(mut v: Value, next: Value) -> Value {
    v["dailySleepDTO"]["nextSleepNeed"] = next;
    v
}
fn reminders_on(store: &Store, lead: u32) {
    let mut settings = store.settings();
    settings.notify_bedtime = true;
    settings.bedtime_lead_min = lead;
    store.set_settings(&settings).unwrap();
}

#[test]
fn current_alignment_places_sleep_against_garmins_window() {
    let store = Store::open(":memory:").unwrap();
    for i in 1..9 {
        put(
            &store,
            &day(-i),
            &aligned(
                night(&day(-i), "23:00", "06:30"),
                "ALIGNED",
                Some((-90, 360)),
            ),
        );
    }
    put(
        &store,
        DATE,
        &aligned(night(DATE, "00:45", "07:15"), "BEHIND", Some((-90, 360))),
    );
    let view = screens::sleep_view(&store, DATE);
    let alignment = view.alignment.unwrap();
    assert_eq!(alignment.state, AlignmentState::Current);
    assert_eq!(
        alignment.detail,
        "Last night ran later than Garmin's optimal window."
    );
    let latest = alignment.latest.unwrap();
    assert_eq!(latest.status.as_deref(), Some("Behind"));
    assert_eq!(latest.window, Some([-90., 360.]));
    assert_eq!(latest.sleep, Some([45., 435.]));
    assert_eq!(alignment.history.len(), 7);
    assert_eq!(alignment.history[0].date, day(-6));
    // Evening bedtimes count back from the morning's midnight.
    assert_eq!(alignment.history[0].sleep, Some([-60., 390.]));
    assert_eq!(alignment.history[6].date, DATE);
    // The plan coaches steadier timing from Garmin's current status.
    let tip = view.sleep_plan.unwrap().tip.unwrap();
    assert!(
        tip.starts_with("Sleep has run later than Garmin's optimal window."),
        "{tip}"
    );
}

#[test]
fn absent_alignment_keeps_the_plan_and_says_unavailable() {
    let store = Store::open(":memory:").unwrap();
    for i in 0..5 {
        put(&store, &day(-i), &night(&day(-i), "23:00", "07:00"));
    }
    let view = screens::sleep_view(&store, DATE);
    let alignment = view.alignment.unwrap();
    assert_eq!(alignment.state, AlignmentState::Unavailable);
    assert!(alignment.latest.is_none());
    assert_eq!(alignment.history.len(), 5);
    let plan = view.sleep_plan.unwrap();
    assert_eq!(plan.bedtime_source, BedtimeSource::WakeHistory);
    assert_eq!(plan.bedtime, "23:30");
    assert_eq!(
        plan.tip.as_deref(),
        Some("Keep waking near 07:00, weekends included.")
    );
    // Without any sleep there is nothing to show.
    assert!(screens::sleep_view(&Store::open(":memory:").unwrap(), DATE)
        .alignment
        .is_none());
}

#[test]
fn calibrating_and_stale_alignment_are_labelled() {
    let store = Store::open(":memory:").unwrap();
    put(
        &store,
        &day(-2),
        &aligned(
            night(&day(-2), "23:00", "07:00"),
            "ALIGNED",
            Some((-60, 390)),
        ),
    );
    put(&store, &day(-1), &night(&day(-1), "23:10", "07:00"));
    put(&store, DATE, &night(DATE, "23:20", "07:00"));
    let alignment = screens::sleep_view(&store, DATE).alignment.unwrap();
    assert_eq!(alignment.state, AlignmentState::Stale);
    assert_eq!(alignment.latest.unwrap().date, day(-2));
    assert_eq!(
        alignment.detail,
        "No alignment for last night yet. Showing Mon 10 Mar."
    );
    // A stale status does not drive tonight's tip.
    let plan = screens::sleep_view(&store, DATE).sleep_plan.unwrap();
    assert_eq!(
        plan.tip.as_deref(),
        Some("Keep waking near 07:00, weekends included.")
    );

    put(
        &store,
        DATE,
        &aligned(night(DATE, "23:20", "07:00"), "CALIBRATING", None),
    );
    let alignment = screens::sleep_view(&store, DATE).alignment.unwrap();
    assert_eq!(alignment.state, AlignmentState::Calibrating);
    assert_eq!(alignment.detail, "Garmin is still learning your rhythm.");

    put(
        &store,
        DATE,
        &aligned(night(DATE, "23:20", "07:00"), "JET_LAG", None),
    );
    let alignment = screens::sleep_view(&store, DATE).alignment.unwrap();
    assert_eq!(alignment.state, AlignmentState::Current);
    assert_eq!(alignment.latest.unwrap().status.as_deref(), Some("Jet lag"));
    assert_eq!(
        alignment.detail,
        "Garmin reports jet lag and has no optimal window for this night."
    );
}

#[test]
fn tonights_window_wins_over_the_optimal_window() {
    let store = Store::open(":memory:").unwrap();
    let v = aligned(night(DATE, "23:00", "06:45"), "AHEAD", Some((-30, 420)));
    let v = with_next(
        v,
        json!({
            "calendarDate": day(1),
            "actual": 500,
            "baseline": 450,
            "recommendedBedtimeStartMins": 1290,
            "recommendedBedtimeEndMins": 1330,
            "trainingFeedback": "HIGH_ACWR",
            "sleepHistoryAdjustment": "NO_CHANGE",
            "hrvAdjustment": "NO_CHANGE",
            "napAdjustment": "SOMETHING_NEW",
        }),
    );
    put(&store, DATE, &v);
    let plan = screens::today(&store, DATE).sleep_plan.unwrap();
    assert_eq!(plan.night, day(1));
    assert_eq!(plan.bedtime_source, BedtimeSource::Garmin);
    assert_eq!(plan.bedtime_start.as_deref(), Some("21:30"));
    assert_eq!(plan.bedtime, "22:10");
    assert_eq!(plan.wind_down, "21:00");
    assert_eq!(plan.wind_down_at, format!("{DATE}T21:00:00"));
    assert_eq!(plan.source, SleepNeedSource::Tonight);
    assert_eq!(plan.baseline_min, Some(450.));
    assert_eq!(plan.adjustments.len(), 2);
    assert_eq!(plan.adjustments[0].factor, "Training");
    assert_eq!(plan.adjustments[1].detail, "something new");
    assert_eq!(
        plan.reason.as_deref(),
        Some("Garmin added 50 min for high training load and something new.")
    );
    // Garmin's optimal window stays on the alignment, not in the plan.
    let alignment = screens::sleep_view(&store, DATE).alignment.unwrap();
    assert_eq!(alignment.latest.unwrap().window, Some([-30., 420.]));
}

#[test]
fn time_to_fall_asleep_moves_in_bed_and_an_earlier_wind_down_reminder() {
    let store = Store::open(":memory:").unwrap();
    // Garmin's window is 22:30–22:40 tonight; its need ends at 06:10.
    let v = with_next(
        night(DATE, "23:00", "07:00"),
        json!({
            "calendarDate": day(1), "actual": 450, "baseline": 450,
            "recommendedBedtimeStartMins": 1350, "recommendedBedtimeEndMins": 1360,
        }),
    );
    put(&store, DATE, &v);
    reminders_on(&store, 45);
    for (fall_asleep, in_bed, wind_down) in [(0, "22:40", "21:45"), (15, "22:25", "21:40"), (60, "21:40", "20:55")] {
        let mut settings = store.settings();
        settings.fall_asleep_min = fall_asleep;
        store.set_settings(&settings).unwrap();
        let plan = screens::today(&store, DATE).sleep_plan.unwrap();
        // Garmin's window and need are unchanged.
        assert_eq!((plan.bedtime_start.as_deref(), plan.bedtime.as_str(), plan.need_min), (Some("22:30"), "22:40", 450.));
        assert_eq!((plan.in_bed.as_str(), plan.wind_down.as_str(), plan.in_bed_min), (in_bed, wind_down, 450. + f64::from(fall_asleep)), "{fall_asleep} min");
        assert_eq!(plan.wake_at, format!("{}T06:10:00", day(1)));
        let reminder = notifications::bedtime(&store, at(DATE, "20:00")).unwrap();
        assert_eq!((reminder.at, reminder.night.as_str()), (at(DATE, wind_down), day(1).as_str()), "the reminder fires at the same wind-down, for the same night");
        assert!(reminder.body.starts_with(&format!("In bed by {in_bed} · ")), "{}", reminder.body);
    }
}

#[test]
fn reminder_follows_the_plan_lead_and_fires_once_per_night() {
    let store = Store::open(":memory:").unwrap();
    let v = with_next(
        night(DATE, "23:00", "07:00"),
        json!({
            "calendarDate": day(1), "actual": 450, "baseline": 450,
            "recommendedBedtimeStartMins": 1350, "recommendedBedtimeEndMins": 1410,
        }),
    );
    put(&store, DATE, &v);
    assert!(
        notifications::bedtime(&store, at(DATE, "20:00")).is_none(),
        "off by default"
    );
    reminders_on(&store, 45);
    let reminder = notifications::bedtime(&store, at(DATE, "20:00")).unwrap();
    assert_eq!(reminder.at, at(DATE, "21:45"));
    assert_eq!(reminder.until, at(DATE, "23:30"));
    assert_eq!(reminder.night, day(1));
    assert_eq!(
        reminder.body,
        "In bed by 23:15 · Garmin's bedtime window: 22:30–23:30 · need 7h 30m"
    );
    let local = |t: NaiveDateTime| t.and_local_timezone(chrono::Local).earliest().unwrap();
    // The hourly sync only posts once the wind-down time has come.
    assert!(
        !notifications::notifications(&store, local(at(DATE, "21:00")))
            .iter()
            .any(|n| n.kind == "bedtime")
    );
    let due = notifications::notifications(&store, local(at(DATE, "22:00")));
    let due: Vec<_> = due.into_iter().filter(|n| n.kind == "bedtime").collect();
    assert_eq!(due.len(), 1);
    assert_eq!(due[0].tag, day(1));
    notifications::mark_sent(&store, &due, local(at(DATE, "22:00"))).unwrap();
    assert!(
        notifications::bedtime(&store, at(DATE, "22:05")).is_none(),
        "once per night"
    );

    // Past the asleep-by time the reminder is dropped rather than sent late.
    let late = Store::open(":memory:").unwrap();
    put(&late, DATE, &v);
    reminders_on(&late, 30);
    assert!(notifications::bedtime(&late, at(DATE, "23:30")).is_none());
}

#[test]
fn a_window_past_midnight_stays_on_the_same_night() {
    let store = Store::open(":memory:").unwrap();
    let v = with_next(
        night(DATE, "23:00", "08:00"),
        json!({
            "calendarDate": day(1), "actual": 450,
            "recommendedBedtimeStartMins": 1450, "recommendedBedtimeEndMins": 1500,
        }),
    );
    put(&store, DATE, &v);
    reminders_on(&store, 30);
    let plan = screens::today(&store, DATE).sleep_plan.unwrap();
    assert_eq!(plan.wind_down_at, format!("{DATE}T23:40:00"));
    assert_eq!(plan.bedtime_start_at, Some(format!("{}T00:10:00", day(1))));
    assert_eq!(plan.bedtime_at, format!("{}T01:00:00", day(1)));
    // After midnight the next day has no night yet, so the same plan holds.
    let reminder = notifications::bedtime(&store, at(&day(1), "00:20")).unwrap();
    assert_eq!(reminder.night, day(1));
    assert_eq!(reminder.until, at(&day(1), "01:00"));
}

#[test]
fn stale_plans_show_but_never_remind() {
    let store = Store::open(":memory:").unwrap();
    for i in 3..6 {
        put(&store, &day(-i), &night(&day(-i), "23:00", "07:00"));
    }
    reminders_on(&store, 30);
    let plan = screens::today(&store, DATE).sleep_plan.unwrap();
    assert!(plan.stale);
    assert_eq!(plan.source, SleepNeedSource::Average);
    assert!(notifications::bedtime(&store, at(DATE, "23:00")).is_none());

    // An empty or all-null record for today is not a recorded night.
    for empty in [
        json!({"dailySleepDTO": {}}),
        json!({"dailySleepDTO": {"calendarDate": DATE, "sleepTimeSeconds": null,
            "sleepStartTimestampLocal": null, "sleepEndTimestampLocal": null}}),
        with_next(
            json!({"dailySleepDTO": {"sleepTimeSeconds": 0}}),
            json!({
                "calendarDate": day(1), "actual": 450,
                "recommendedBedtimeStartMins": 1350, "recommendedBedtimeEndMins": 1410,
            }),
        ),
    ] {
        put(&store, DATE, &empty);
        assert!(
            screens::today(&store, DATE).sleep_plan.unwrap().stale,
            "{empty}"
        );
        assert!(
            notifications::bedtime(&store, at(DATE, "22:50")).is_none(),
            "{empty}"
        );
    }
    put(&store, DATE, &night(DATE, "23:00", "07:00"));
    assert!(!screens::today(&store, DATE).sleep_plan.unwrap().stale);
    assert!(notifications::bedtime(&store, at(DATE, "23:00")).is_some());
}

#[test]
fn daylight_saving_nights_keep_wall_clock_times() {
    // The night clocks change: Garmin's minutes and the fallback stay wall-clock
    // times; the platform maps them to its own clock when scheduling.
    let store = Store::open(":memory:").unwrap();
    let change = "2025-11-02";
    let before = "2025-11-01";
    put(
        &store,
        before,
        &with_next(
            night(before, "23:00", "06:30"),
            json!({
                "calendarDate": change, "actual": 480,
                "recommendedBedtimeStartMins": 1320, "recommendedBedtimeEndMins": 1380,
            }),
        ),
    );
    let plan = screens::today(&store, before).sleep_plan.unwrap();
    assert_eq!(plan.bedtime_start_at, Some(format!("{before}T22:00:00")));
    assert_eq!(plan.bedtime_at, format!("{before}T23:00:00"));
    put(&store, change, &night(change, "23:30", "06:30"));
    let fallback = screens::today(&store, change).sleep_plan.unwrap();
    assert_eq!(fallback.bedtime_source, BedtimeSource::WakeHistory);
    assert_eq!(fallback.bedtime_at, format!("{change}T23:00:00"));
}

#[test]
fn garmin_averages_come_from_its_sleep_stats() {
    let store = Store::open(":memory:").unwrap();
    put(&store, DATE, &night(DATE, "23:00", "07:00"));
    assert!(screens::sleep_view(&store, DATE).garmin_averages.is_none());
    store
        .put("sleep_stats", DATE, "now", &json!({
            "overallStats": {"averageLocalSleepStartTime": -1500.0, "averageLocalSleepEndTime": 25500.0},
            "individualStats": [
                {"calendarDate": day(-1), "values": {"localSleepStartTimeInMillis": 1}},
                {"calendarDate": DATE, "values": {"localSleepStartTimeInMillis": 2}},
                {"calendarDate": day(-2), "values": {}},
            ],
        }))
        .unwrap();
    let averages = screens::sleep_view(&store, DATE).garmin_averages.unwrap();
    assert_eq!(averages.bedtime, "23:35");
    assert_eq!(averages.wake_time, "07:05");
    assert_eq!(averages.nights, 2);
    assert_eq!(averages.through, DATE);
}

#[test]
fn checkin_reviews_an_irregular_week_without_scoring_it() {
    let store = Store::open(":memory:").unwrap();
    let monday = "2025-03-17";
    let week = [
        ("2025-03-10", "22:50", "06:40", "ALIGNED"),
        ("2025-03-11", "01:30", "09:10", "BEHIND"),
        ("2025-03-12", "23:05", "06:55", "ALIGNED"),
        ("2025-03-13", "00:40", "08:20", "BEHIND"),
        ("2025-03-15", "02:00", "10:00", "BEHIND"),
        ("2025-03-16", "23:30", "07:10", "BEHIND"),
    ];
    for (date, bed, wake, status) in week {
        put(
            &store,
            date,
            &aligned(night(date, bed, wake), status, Some((-60, 390))),
        );
    }
    put(&store, "2025-03-05", &night("2025-03-05", "23:00", "07:00"));
    put(&store, "2025-03-06", &night("2025-03-06", "23:00", "07:20"));
    let timing = screens::checkin(&store, monday).timing.unwrap();
    assert_eq!(timing.nights.len(), 6);
    assert_eq!(timing.bed_range, Some(["22:50".into(), "02:00".into()]));
    assert_eq!(timing.wake_range, Some(["06:40".into(), "10:00".into()]));
    assert_eq!(timing.wake_spread_min, Some(200.));
    assert_eq!(timing.previous_wake_spread_min, Some(20.));
    assert_eq!((timing.aligned, timing.ahead, timing.behind), (2, 0, 4));
    assert_eq!(
        timing.focus,
        "Most nights ran later than Garmin's optimal window. Keep a steady wake time near 07:45, weekends included, and let bedtime follow."
    );
    assert!(screens::checkin(&Store::open(":memory:").unwrap(), monday)
        .timing
        .is_none());
}

#[test]
fn checkin_unwraps_both_weeks_wakes_across_midnight() {
    let store = Store::open(":memory:").unwrap();
    for (date, wake) in [
        ("2025-03-10", "23:55"), ("2025-03-11", "00:05"),
        ("2025-03-12", "00:00"), ("2025-03-13", "23:55"),
        ("2025-03-14", "00:05"), ("2025-03-15", "00:00"),
        ("2025-03-16", "00:00"),
        ("2025-03-05", "23:50"), ("2025-03-06", "00:10"),
    ] {
        put(&store, date, &night(date, "16:00", wake));
    }
    let timing = screens::checkin(&store, "2025-03-17").timing.unwrap();
    assert_eq!(timing.wake_range, Some(["23:55".into(), "00:05".into()]));
    assert_eq!(timing.wake_spread_min, Some(10.));
    assert_eq!(timing.previous_wake_spread_min, Some(20.));
    // Chart values keep Garmin's clock hours; only the range/spread unwrap.
    assert_eq!(timing.nights[0].wake_hour, 23. + 55. / 60.);
    assert!(timing.focus.contains("00:00"));
}

#[test]
fn settings_saved_before_the_lead_existed_keep_their_switches() {
    let store = Store::open(":memory:").unwrap();
    store
        .set_setting("settings", r#"{"notifyMorningReadiness":false,"notifyWeeklyCheckin":true,"notifyMetricChanges":true,"notifyVitals":true,"notifyTrainingStatus":false,"notifyBedtime":true}"#)
        .unwrap();
    let settings = store.settings();
    assert!(!settings.notify_morning_readiness);
    assert!(settings.notify_bedtime);
    assert_eq!(settings.bedtime_lead_min, 30);
}
