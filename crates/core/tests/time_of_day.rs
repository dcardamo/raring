//! Today's time-of-day phases: boundaries, fallbacks and advice priority.
//! All times and values are synthetic.
use chrono::{Duration, FixedOffset, Local, NaiveDate, NaiveDateTime, TimeZone};
use raring_core::{
    phase::{self, PhaseInput},
    screens,
    store::Store,
    Raring,
    view::{BedtimeSource, DayPhase, LastNight, SleepNeedSource, SleepPlan, TodayCard},
};
use serde_json::json;

const DATE: &str = "2025-03-12";

fn at(s: &str) -> NaiveDateTime {
    NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M").unwrap()
}
/// Garmin's window 22:30–23:10 tonight, wind-down 22:00.
fn plan() -> SleepPlan {
    SleepPlan {
        need_read_at: None,
        night: "2025-03-13".into(),
        bedtime: "23:10".into(),
        bedtime_start: Some("22:30".into()),
        bedtime_source: BedtimeSource::Garmin,
        wind_down: "22:00".into(),
        wind_down_lead_min: 30,
        wind_down_at: "2025-03-12T22:00:00".into(),
        bedtime_start_at: Some("2025-03-12T22:30:00".into()),
        bedtime_at: "2025-03-12T23:10:00".into(),
        wake_time: Some("07:00".into()),
        need_min: 470.,
        source: SleepNeedSource::Tonight,
        baseline_min: Some(460.),
        adjustments: vec![],
        reason: None,
        tip: None,
        wake_nights: 14,
        stale: false,
    }
}
const WAKES: [f64; 8] = [6.75, 7.0, 7.0, 7.25, 6.9, 7.1, 7.0, 6.8];

fn input<'a>(now: &str, plan: Option<&'a SleepPlan>, wakes: &'a [f64]) -> PhaseInput<'a> {
    PhaseInput {
        now: at(now),
        date: NaiveDate::parse_from_str(DATE, "%Y-%m-%d").unwrap(),
        plan,
        wakes,
        last_night: Some(LastNight {
            asleep_min: Some(450.),
            need_min: Some(465.),
            woke: Some("06:40".into()),
        }),
        woke_at: Some(at("2025-03-12T06:40")),
        adaptive: true,
        jet_lag: false,
        zone_shift_min: Some(0),
        vitals_off: 0,
    }
}

#[test]
fn phases_follow_wake_and_tonights_plan() {
    let plan = plan();
    let cases = [
        ("2025-03-12T06:50", DayPhase::Wake, "2025-03-12T09:40:00", true),
        ("2025-03-12T09:39", DayPhase::Wake, "2025-03-12T09:40:00", true),
        ("2025-03-12T09:40", DayPhase::Day, "2025-03-12T20:30:00", true),
        ("2025-03-12T20:30", DayPhase::Evening, "2025-03-12T22:00:00", true),
        ("2025-03-12T22:00", DayPhase::WindDown, "2025-03-12T23:10:00", false),
        ("2025-03-12T23:10", DayPhase::LateNight, "2025-03-13T07:00:00", false),
        // Crossing midnight keeps tonight's plan until the usual wake.
        ("2025-03-13T03:00", DayPhase::LateNight, "2025-03-13T07:00:00", false),
    ];
    for (now, phase, until, effort) in cases {
        let f = phase::focus(&input(now, Some(&plan), &WAKES));
        assert_eq!(f.phase, phase, "{now}");
        assert_eq!(f.until.as_deref(), Some(until), "{now}");
        assert_eq!(f.effort, effort, "{now}");
        assert!(f.note.is_none());
    }
}

#[test]
fn one_leading_card_per_phase_with_stable_metrics_after_it() {
    let plan = plan();
    let cards = |now| phase::focus(&input(now, Some(&plan), &WAKES)).cards;
    use TodayCard::*;
    assert_eq!(cards("2025-03-12T07:00"), [Focus, SleepPlan]);
    assert_eq!(cards("2025-03-12T13:00"), [Now, Focus, SleepPlan]);
    assert_eq!(cards("2025-03-12T21:00"), [SleepPlan, Focus]);
    assert_eq!(cards("2025-03-12T23:30"), [SleepPlan, Focus]);
    // The plan is folded until it leads.
    let f = |now| phase::focus(&input(now, Some(&plan), &WAKES));
    assert!(!f("2025-03-12T13:00").plan_open);
    assert!(f("2025-03-12T21:00").plan_open);
    // Two outside-range vitals go above everything.
    let mut i = input("2025-03-12T21:00", Some(&plan), &WAKES);
    i.vitals_off = 2;
    assert_eq!(phase::focus(&i).cards, [Vitals, SleepPlan, Focus]);
}

#[test]
fn unsynced_night_is_neutral_and_drops_yesterdays_effort() {
    let plan = plan();
    // The displayed day is still yesterday's after the usual wake has passed.
    let f = phase::focus(&input("2025-03-13T07:30", Some(&plan), &WAKES));
    assert_eq!(f.phase, DayPhase::Wake);
    assert!(f.sleep_missing && !f.effort && f.last_night.is_none());
    assert_eq!(f.cards, [TodayCard::Focus]);
    // Three hours after the credible usual wake (07:00) the calendar day takes over.
    assert_eq!(f.until.as_deref(), Some("2025-03-13T10:00:00"));
    assert_eq!(
        phase::headline(&f, Some(&plan), "Ready to push"),
        "Waiting for last night's sleep"
    );
    // A day without a recorded night uses the usual wake, without a verdict.
    let mut i = input("2025-03-12T08:00", Some(&plan), &WAKES);
    i.last_night = None;
    i.woke_at = None;
    let f = phase::focus(&i);
    assert_eq!(f.phase, DayPhase::Wake);
    assert!(f.sleep_missing && f.effort);
    assert_eq!(f.until.as_deref(), Some("2025-03-12T10:00:00"));
}

#[test]
fn fixed_layout_without_a_credible_schedule() {
    let plan = plan();
    let check = |i: PhaseInput, note: bool| {
        let f = phase::focus(&i);
        assert_eq!(f.phase, DayPhase::Overview);
        assert_eq!(f.note.is_some(), note);
        assert!(f.plan_open && f.effort);
        assert_eq!(f.cards, [TodayCard::Focus, TodayCard::SleepPlan]);
    };
    let mut off = input("2025-03-12T21:00", Some(&plan), &WAKES);
    off.adaptive = false;
    check(off, false);
    check(input("2025-03-12T21:00", Some(&plan), &WAKES[..3]), true);
    check(input("2025-03-12T21:00", None, &WAKES), true);
    let irregular = [5.0, 5.5, 9.0, 13.0, 14.0, 6.0, 15.0, 10.0];
    check(input("2025-03-12T21:00", Some(&plan), &irregular), true);
    let mut jet = input("2025-03-12T21:00", Some(&plan), &WAKES);
    jet.jet_lag = true;
    check(jet, true);
    let mut travel = input("2025-03-12T21:00", Some(&plan), &WAKES);
    travel.zone_shift_min = Some(-180);
    check(travel, true);
}

#[test]
fn daylight_saving_shift_keeps_the_schedule() {
    let plan = plan();
    let mut i = input("2025-03-12T21:00", Some(&plan), &WAKES);
    i.zone_shift_min = Some(60);
    assert_eq!(phase::focus(&i).phase, DayPhase::Evening);
}

#[test]
fn headline_follows_the_phase_without_pressure() {
    let plan = plan();
    let headline = |now| {
        let f = phase::focus(&input(now, Some(&plan), &WAKES));
        phase::headline(&f, Some(&plan), "Ready to push")
    };
    assert_eq!(headline("2025-03-12T07:00"), "Ready to push");
    assert_eq!(headline("2025-03-12T13:00"), "Ready to push");
    assert_eq!(headline("2025-03-12T21:00"), "Wind down at 22:00");
    assert_eq!(headline("2025-03-12T22:30"), "Time to wind down");
    assert_eq!(headline("2025-03-13T01:00"), "Rest when you're ready");
}

#[test]
fn day_sleeper_schedule_is_anchored_to_their_wake() {
    // Wakes at 15:00 and sleeps 07:00–15:00: Raring's estimate from wake history.
    let wakes = [15.0; 8];
    let plan = SleepPlan {
        need_read_at: None,
        night: "2025-03-13".into(),
        bedtime: "07:10".into(),
        bedtime_start: None,
        bedtime_source: BedtimeSource::WakeHistory,
        wind_down: "06:40".into(),
        wind_down_at: "2025-03-13T06:40:00".into(),
        bedtime_start_at: None,
        bedtime_at: "2025-03-13T07:10:00".into(),
        wake_time: Some("15:00".into()),
        ..plan()
    };
    let mut i = input("2025-03-12T16:00", Some(&plan), &wakes);
    i.woke_at = Some(at("2025-03-12T15:00"));
    let phase_at = |now: &str| {
        let mut i = input(now, Some(&plan), &wakes);
        i.woke_at = Some(at("2025-03-12T15:00"));
        phase::focus(&i).phase
    };
    assert_eq!(phase::focus(&i).phase, DayPhase::Wake);
    assert_eq!(phase_at("2025-03-12T23:00"), DayPhase::Day);
    assert_eq!(phase_at("2025-03-13T05:30"), DayPhase::Evening);
    assert_eq!(phase_at("2025-03-13T07:30"), DayPhase::LateNight);
    // Before the usual wake on a day whose sleep hasn't synced: neutral.
    let mut early = input("2025-03-12T13:00", Some(&plan), &wakes);
    early.last_night = None;
    early.woke_at = None;
    let f = phase::focus(&early);
    assert_eq!((f.phase, f.effort), (DayPhase::Wake, false));
    assert_eq!(f.until.as_deref(), Some("2025-03-12T15:00:00"));
}

/// A stored night in Garmin's shape: local and GMT epoch milliseconds.
fn put_night(store: &Store, date: &str, wake: &str, offset_h: i64, next: serde_json::Value) {
    let end = at(&format!("{date}T{wake}")).and_utc().timestamp_millis();
    store
        .put(
            "sleep",
            date,
            "synthetic",
            &json!({"dailySleepDTO": {
                "sleepTimeSeconds": 27000,
                "sleepStartTimestampLocal": end - 27_000_000,
                "sleepEndTimestampLocal": end,
                "sleepEndTimestampGMT": end - offset_h * 3_600_000,
                "sleepNeed": {"actual": 470},
                "nextSleepNeed": next,
            }}),
        )
        .unwrap();
}

#[test]
fn today_view_carries_the_phase_and_travel_falls_back() {
    let store = Store::open(":memory:").unwrap();
    for i in 0..8 {
        let d = NaiveDate::parse_from_str(DATE, "%Y-%m-%d").unwrap() - chrono::Duration::days(i);
        put_night(
            &store,
            &d.to_string(),
            "06:50",
            -5,
            json!({"actual": 470, "recommendedBedtimeStartMins": 1350, "recommendedBedtimeEndMins": 1390}),
        );
    }
    let home = FixedOffset::west_opt(5 * 3600).unwrap();
    let now = |tz: FixedOffset, t: &str| tz.from_local_datetime(&at(t)).unwrap();
    let view = screens::today_at(&store, DATE, now(home, "2025-03-12T21:00"));
    assert_eq!(view.focus.phase, DayPhase::Evening);
    assert_eq!(view.headline, "Wind down at 22:00");
    assert_eq!(view.sleep_plan.as_ref().unwrap().wind_down, "22:00");
    assert_eq!(view.focus.last_night.as_ref().unwrap().woke.as_deref(), Some("06:50"));
    // Three hours east of the recorded nights: the fixed layout.
    let away = FixedOffset::west_opt(2 * 3600).unwrap();
    let view = screens::today_at(&store, DATE, now(away, "2025-03-12T21:00"));
    assert_eq!(view.focus.phase, DayPhase::Overview);
    assert!(view.focus.note.is_some());
    // Garmin's jet-lag rating counts only with recorded sleep.
    let mut raw = store.get("sleep", DATE).unwrap();
    raw["dailySleepDTO"]["sleepAlignment"] = json!({"status": "JET_LAG"});
    store.put("sleep", DATE, "synthetic", &raw).unwrap();
    let view = screens::today_at(&store, DATE, now(home, "2025-03-12T21:00"));
    assert_eq!(view.focus.phase, DayPhase::Overview);
    store
        .put("sleep", DATE, "synthetic", &json!({"dailySleepDTO": {"sleepAlignment": {"status": "JET_LAG"}}}))
        .unwrap();
    let view = screens::today_at(&store, DATE, now(home, "2025-03-12T21:00"));
    assert_ne!(view.focus.phase, DayPhase::Overview);
    assert!(view.focus.sleep_missing);
    // Turned off in Settings.
    let mut settings = store.settings();
    settings.adaptive_today = false;
    store.set_settings(&settings).unwrap();
    let view = screens::today_at(&store, DATE, now(home, "2025-03-12T21:00"));
    assert_eq!((view.focus.phase, view.focus.note.as_deref()), (DayPhase::Overview, None));
}

fn put_ready(store: &Store, date: &str) {
    store
        .put(
            "readiness",
            date,
            "synthetic",
            &json!([{"score": 84, "level": "HIGH", "timestamp": format!("{date}T07:30:00.0")}]),
        )
        .unwrap();
}
fn day(date: &str, back: i64) -> String {
    (NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap() - Duration::days(back)).to_string()
}
const WINDOW: fn() -> serde_json::Value =
    || json!({"actual": 470, "recommendedBedtimeStartMins": 1350, "recommendedBedtimeEndMins": 1390});

/// Every fallback keeps the morning neutral: after the next night's credible
/// expected wake (or the calendar rollover without one) and before it syncs,
/// yesterday's effort call is gone, in the full Today view.
#[test]
fn every_fallback_waits_neutrally_at_the_new_morning() {
    let home = FixedOffset::west_opt(5 * 3600).unwrap();
    let now = |tz: FixedOffset, t: &str| tz.from_local_datetime(&at(t)).unwrap();
    let morning = "2025-03-13T08:00";
    let regular = |store: &Store, nights: i64, next: serde_json::Value| {
        for i in 0..nights {
            put_night(store, &day(DATE, i), "07:00", -5, next.clone());
        }
        put_ready(store, DATE);
    };
    let check = |name: &str, store: &Store, tz: FixedOffset, handover: &str| {
        let date = screens::current_date(store, at(morning));
        assert_eq!(date.to_string(), DATE, "{name}: yesterday stays shown");
        let view = screens::today_at(store, DATE, now(tz, morning));
        assert_eq!(view.call, Some(raring_core::view::Effort::Push), "{name}: Garmin's call is stored");
        let f = &view.focus;
        assert_eq!((f.phase, f.effort, f.sleep_missing), (DayPhase::Wake, false, true), "{name}");
        assert!(f.last_night.is_none() && f.cards == [TodayCard::Focus], "{name}");
        assert_eq!(view.headline, "Waiting for last night's sleep", "{name}");
        assert_eq!(f.until.as_deref(), Some(handover), "{name}");
        // Today hands over to the calendar day exactly when the focus says.
        let until = at(&handover[..16]);
        assert_eq!(screens::current_date(store, until - Duration::minutes(1)).to_string(), DATE, "{name}");
        assert_eq!(screens::current_date(store, until).to_string(), "2025-03-13", "{name}");
    };

    let store = Store::open(":memory:").unwrap();
    regular(&store, 8, WINDOW());
    let mut settings = store.settings();
    settings.adaptive_today = false;
    store.set_settings(&settings).unwrap();
    check("switched off", &store, home, "2025-03-13T10:00:00");

    let store = Store::open(":memory:").unwrap();
    regular(&store, 2, WINDOW());
    check("few wake times", &store, home, "2025-03-13T12:00:00");

    let store = Store::open(":memory:").unwrap();
    for (i, wake) in ["05:00", "13:30", "09:00", "15:00", "06:00", "11:00", "14:00", "07:30"].iter().enumerate() {
        put_night(&store, &day(DATE, i as i64), wake, -5, WINDOW());
    }
    put_ready(&store, DATE);
    check("irregular", &store, home, "2025-03-13T12:00:00");

    let store = Store::open(":memory:").unwrap();
    regular(&store, 8, WINDOW());
    check("travel", &store, FixedOffset::west_opt(2 * 3600).unwrap(), "2025-03-13T10:00:00");

    let store = Store::open(":memory:").unwrap();
    regular(&store, 8, WINDOW());
    let mut raw = store.get("sleep", DATE).unwrap();
    raw["dailySleepDTO"]["sleepAlignment"] = json!({"status": "JET_LAG"});
    store.put("sleep", DATE, "synthetic", &raw).unwrap();
    check("jet lag", &store, home, "2025-03-13T10:00:00");

    let store = Store::open(":memory:").unwrap();
    for i in 0..8 {
        let d = day(DATE, i);
        let end = at(&format!("{d}T07:00")).and_utc().timestamp_millis();
        store
            .put("sleep", &d, "synthetic", &json!({"dailySleepDTO": {
                "sleepTimeSeconds": 27000, "sleepEndTimestampLocal": end,
                "sleepEndTimestampGMT": end + 5 * 3_600_000}}))
            .unwrap();
    }
    put_ready(&store, DATE);
    assert!(screens::today_at(&store, DATE, now(home, morning)).sleep_plan.is_none());
    check("no plan", &store, home, "2025-03-13T10:00:00");

    // Before the expected wake the fixed layout keeps its call; the night isn't over.
    let store = Store::open(":memory:").unwrap();
    regular(&store, 2, WINDOW());
    let view = screens::today_at(&store, DATE, now(home, "2025-03-12T20:00"));
    assert_eq!((view.focus.phase, view.focus.effort), (DayPhase::Overview, true));
}

/// A temporary database Raring opens like the app does.
fn raring_with(name: &str, fill: impl Fn(&Store)) -> (Raring, std::path::PathBuf) {
    let path = std::env::temp_dir().join(format!("raring-tod-{name}-{}.sqlite", std::process::id()));
    let _ = std::fs::remove_file(&path);
    fill(&Store::open(&path).unwrap());
    (Raring::open(&path).unwrap(), path)
}
fn local(t: &str) -> chrono::DateTime<Local> {
    Local.from_local_datetime(&at(t)).single().unwrap()
}
/// The local UTC offset on a date, hours, so stored nights aren't "travel".
fn local_offset(t: &str) -> i64 {
    i64::from(local(t).offset().local_minus_utc()) / 3600
}

#[test]
fn day_sleeper_keeps_the_night_through_its_afternoon_wake() {
    // Sleeps to 15:00 with a 470-minute need; Raring's estimate from wake history.
    let offset = local_offset("2025-06-10T12:00");
    let (core, path) = raring_with("day-sleeper", |store| {
        for i in 0..8 {
            put_night(store, &day("2025-06-09", i), "15:00", offset, json!(null));
        }
        put_ready(store, "2025-06-09");
    });
    let at = |t: &str| {
        let now = local(t);
        let date = core.current_date(now);
        (date.clone(), core.today_at(&date, now))
    };
    // Still asleep at 13:00: the previous day's night and plan stay, in late night.
    let (date, view) = at("2025-06-10T13:00");
    assert_eq!(date, "2025-06-09");
    assert_eq!(view.focus.phase, DayPhase::LateNight);
    assert_eq!(view.sleep_plan.as_ref().unwrap().bedtime_at, "2025-06-10T07:10:00");
    assert_eq!(view.focus.until.as_deref(), Some("2025-06-10T15:00:00"));
    // After the 15:00 wake without a sync: neutral waiting on the same day.
    let (date, view) = at("2025-06-10T16:00");
    assert_eq!(date, "2025-06-09");
    assert_eq!((view.focus.phase, view.focus.effort), (DayPhase::Wake, false));
    assert_eq!(view.headline, "Waiting for last night's sleep");
    assert_eq!(view.focus.until.as_deref(), Some("2025-06-10T18:00:00"));
    // Then the calendar day, its missing night stated, with tonight's plan.
    let (date, view) = at("2025-06-10T18:30");
    assert_eq!(date, "2025-06-10");
    assert_eq!(view.focus.phase, DayPhase::Day);
    assert!(view.focus.sleep_missing && view.focus.last_night.is_none());
    assert_eq!(view.sleep_plan.as_ref().unwrap().bedtime_at, "2025-06-11T07:10:00");
    // A synced start or end alone is a recorded night, even without duration/readiness.
    let store = Store::open(&path).unwrap();
    for (field, time) in [
        ("sleepStartTimestampLocal", "2025-06-10T07:10:00"),
        ("sleepEndTimestampLocal", "2025-06-10T15:00:00"),
    ] {
        store.put("sleep", "2025-06-10", "synthetic", &json!({"dailySleepDTO": {
            field: time, "sleepTimeSeconds": null
        }})).unwrap();
        let (date, view) = at("2025-06-10T16:00");
        assert_eq!(date, "2025-06-10", "{field}");
        assert!(!view.focus.sleep_missing, "{field}");
        assert!(view.focus.last_night.is_some(), "{field}");
    }
    // The completed night is shown with the morning card too.
    put_night(&store, "2025-06-10", "15:00", offset, json!(null));
    let (date, view) = at("2025-06-10T16:00");
    assert_eq!(date, "2025-06-10");
    assert_eq!((view.focus.phase, view.focus.effort), (DayPhase::Wake, true));
    drop(core);
    let _ = std::fs::remove_file(path);
}

#[test]
fn a_midnight_wake_cluster_is_a_schedule_not_noise() {
    // Wakes a few minutes either side of midnight, filed under the day they
    // end: after midnight on that date, or just before its end.
    let offset = local_offset("2025-06-10T12:00");
    let (core, path) = raring_with("midnight", |store| {
        for (key, wake) in [
            ("2025-06-01", "00:05"), ("2025-06-02", "00:00"), ("2025-06-03", "00:10"),
            ("2025-06-04", "00:05"), ("2025-06-05", "23:55"), ("2025-06-06", "23:50"),
            ("2025-06-07", "23:55"), ("2025-06-08", "00:05"), ("2025-06-09", "23:55"),
        ] {
            put_night(store, key, wake, offset, json!(null));
        }
        put_ready(store, "2025-06-09");
    });
    let at = |t: &str| {
        let now = local(t);
        let date = core.current_date(now);
        (date.clone(), core.today_at(&date, now))
    };
    let (date, view) = at("2025-06-10T10:00");
    assert_eq!(date, "2025-06-09");
    let plan = view.sleep_plan.clone().unwrap();
    assert_eq!(plan.wake_time.as_deref(), Some("00:00"), "usual wake on the clock circle");
    // Need 470 min before the next wake, ~24 h after the last one (06-09 23:55).
    assert_eq!(plan.bedtime_at, "2025-06-10T16:10:00");
    assert_eq!(plan.night, "2025-06-11");
    let store = Store::open(&path).unwrap();
    let mut settings = store.settings();
    settings.notify_bedtime = true;
    store.set_settings(&settings).unwrap();
    let reminder = core.bedtime_reminder(local("2025-06-10T15:45").naive_local()).unwrap();
    assert_eq!(reminder.night, plan.night);
    assert_eq!(reminder.until.format("%Y-%m-%dT%H:%M:%S").to_string(), plan.bedtime_at);
    store.mark_notified("bedtime", &plan.night, "synthetic").unwrap();
    assert!(core.bedtime_reminder(local("2025-06-10T15:45").naive_local()).is_none());
    assert_eq!((view.focus.phase, view.focus.note.as_deref()), (DayPhase::Day, None));
    let sleep = core.sleep("2025-06-09");
    assert_eq!(sleep.sleep_plan, Some(plan));
    let (_, view) = at("2025-06-10T17:00");
    assert_eq!(view.focus.phase, DayPhase::LateNight);
    // Past the expected wake without a sync, across midnight: neutral, still that day.
    let (date, view) = at("2025-06-11T01:00");
    assert_eq!(date, "2025-06-09");
    assert!(!view.focus.effort && view.focus.sleep_missing);
    drop(core);
    let _ = std::fs::remove_file(path);
}

#[test]
fn stored_wake_clusters_ordinary_and_broad() {
    let offset = local_offset("2025-06-10T12:00");
    let (core, path) = raring_with("ordinary", |store| {
        for i in 0..8 {
            put_night(store, &day("2025-06-10", i), ["06:50", "07:10"][i as usize % 2], offset, json!(null));
        }
        put_ready(store, "2025-06-10");
    });
    let view = core.today_at("2025-06-10", local("2025-06-10T13:00"));
    assert_eq!(view.sleep_plan.as_ref().unwrap().wake_time.as_deref(), Some("07:00"));
    assert_eq!((view.focus.phase, view.focus.note.as_deref()), (DayPhase::Day, None));
    drop(core);
    let _ = std::fs::remove_file(path);
    let (core, path) = raring_with("broad", |store| {
        for (i, wake) in ["03:00", "09:00", "15:00", "21:00", "06:00", "12:00", "18:00", "00:00"].iter().enumerate() {
            put_night(store, &day("2025-06-10", i as i64), wake, offset, json!(null));
        }
        put_ready(store, "2025-06-10");
    });
    let view = core.today_at("2025-06-10", local("2025-06-10T13:00"));
    assert_eq!(view.focus.phase, DayPhase::Overview);
    assert!(view.focus.note.unwrap().contains("vary too much"));
    drop(core);
    let _ = std::fs::remove_file(path);
}
