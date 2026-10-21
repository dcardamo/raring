//! Last night's resting HR, Sleep HR and HRV against their usual ranges
//! (docs/spec.md, Rules: Night vitals on Today). All values are synthetic.
use chrono::{FixedOffset, NaiveDate, NaiveDateTime, TimeZone};
use raring_core::{screens, store::Store, view::*};
use serde_json::{json, Value};

const DATE: &str = "2025-03-12";

fn at(t: &str) -> chrono::DateTime<FixedOffset> {
    let tz = FixedOffset::west_opt(5 * 3600).unwrap();
    tz.from_local_datetime(&NaiveDateTime::parse_from_str(&format!("{DATE}T{t}"), "%Y-%m-%dT%H:%M").unwrap()).unwrap()
}
fn day(back: i64) -> String {
    (NaiveDate::parse_from_str(DATE, "%Y-%m-%d").unwrap() - chrono::Duration::days(back)).to_string()
}
/// A night ending 06:50 at UTC−5 with this overnight average heart rate.
fn night(store: &Store, date: &str, hr: Option<f64>) {
    let end = NaiveDateTime::parse_from_str(&format!("{date}T06:50"), "%Y-%m-%dT%H:%M").unwrap().and_utc().timestamp_millis();
    store.put("sleep", date, "synthetic", &json!({"dailySleepDTO": {
        "sleepTimeSeconds": 27000, "sleepStartTimestampLocal": end - 27_000_000, "sleepEndTimestampLocal": end,
        "sleepEndTimestampGMT": end + 5 * 3_600_000, "avgHeartRate": hr, "sleepNeed": {"actual": 470},
        "nextSleepNeed": {"actual": 470, "recommendedBedtimeStartMins": 1350, "recommendedBedtimeEndMins": 1390}}})).unwrap();
}
/// Twenty nights at 50–53 bpm, so the fallback usual range is about 50–53.
fn store(last: Option<f64>) -> Store {
    let store = Store::open(":memory:").unwrap();
    for i in 1..20 {
        night(&store, &day(i), Some(50. + (i % 4) as f64));
    }
    night(&store, DATE, last);
    // Resting HR far from Sleep HR: never the comparison.
    for i in 0..20 {
        store.put("summary", &day(i), "synthetic", &json!({"restingHeartRate": 70})).unwrap();
    }
    store
}
fn hrv(store: &Store, last: f64) {
    store.put("hrv", DATE, "synthetic", &json!({"hrvSummary": {"lastNightAvg": last, "status": "BALANCED",
        "baseline": {"balancedLow": 45, "balancedUpper": 55}}})).unwrap();
}
fn vitals(store: &Store, t: &str) -> Option<NightVitals> {
    screens::today_at(store, DATE, at(t)).night_vitals
}
fn health(store: &Store, metrics: Value) {
    store.put("health_status", DATE, "synthetic", &json!({"metrics": metrics})).unwrap();
}

#[test]
fn sleep_hr_and_hrv_are_within_above_or_below_their_usual_range() {
    // (Sleep HR, HRV) -> (state, delta, tone) for each
    let rows = [
        (51., 50., (UsualState::Within, None, Tone::Good), (UsualState::Within, None, Tone::Good)),
        (57., 40., (UsualState::Above, Some(6.), Tone::Fair), (UsualState::Below, Some(-10.), Tone::Fair)),
        // Outside in the welcome direction is still outside the usual range.
        (46., 60., (UsualState::Below, Some(-6.), Tone::Fair), (UsualState::Above, Some(10.), Tone::Fair)),
    ];
    for (hr, ms, want_hr, want_hrv) in rows {
        let s = store(Some(hr));
        hrv(&s, ms);
        let v = vitals(&s, "08:00").expect("shown in the morning");
        assert_eq!((v.sleep_hr.value, v.sleep_hr.range_source), (Some(hr), VitalRangeSource::Raring));
        assert_eq!(v.sleep_hr.usual.map(|[lo, hi]| (lo.round(), hi.round())), Some((50., 53.)), "the nights' own range, not resting HR");
        assert_eq!((v.sleep_hr.state, v.sleep_hr.delta, v.sleep_hr.tone), want_hr, "Sleep HR {hr}");
        assert_eq!((v.hrv.value, v.hrv.usual, v.hrv.range_source), (Some(ms), Some([45., 55.]), VitalRangeSource::Garmin));
        assert_eq!((v.hrv.state, v.hrv.delta, v.hrv.tone), want_hrv, "HRV {ms}");
    }
}

#[test]
fn missing_values_and_baselines_are_unset_and_health_status_comes_first() {
    // Too few nights for a usual range, and no HRV at all.
    let s = Store::open(":memory:").unwrap();
    night(&s, &day(1), Some(52.));
    night(&s, DATE, Some(55.));
    let v = vitals(&s, "08:00").unwrap();
    assert_eq!((v.sleep_hr.value, v.sleep_hr.usual, v.sleep_hr.state, v.sleep_hr.delta), (Some(55.), None, UsualState::Unset, None));
    assert_eq!((v.hrv.value, v.hrv.state, v.hrv.tone), (None, UsualState::Unset, Tone::Neutral));
    assert_eq!((v.resting_hr.value, v.resting_hr.usual, v.resting_hr.state), (None, None, UsualState::Unset));
    // Health Status sends HR with its own baseline: it replaces the nightly fallback.
    let s = store(Some(51.));
    health(&s, json!([{"type": "HR", "value": 58, "status": "ABOVE_RANGE", "baselineLowerLimit": 48, "baselineUpperLimit": 54}]));
    let v = vitals(&s, "08:00").unwrap();
    assert_eq!((v.sleep_hr.value, v.sleep_hr.usual, v.sleep_hr.range_source), (Some(58.), Some([48., 54.]), VitalRangeSource::Garmin));
    assert_eq!((v.sleep_hr.state, v.sleep_hr.delta, v.sleep_hr.tone), (UsualState::Above, Some(7.), Tone::Fair));
    // While Garmin calibrates, the bounds it sends are not a usual range.
    health(&s, json!([{"type": "HR", "value": 58, "status": "ONBOARDING", "baselineLowerLimit": 48, "baselineUpperLimit": 54},
        {"type": "HRV", "value": 40, "status": "ONBOARDING", "baselineLowerLimit": 45, "baselineUpperLimit": 55}]));
    let v = vitals(&s, "08:00").unwrap();
    for r in [&v.sleep_hr, &v.hrv] {
        assert_eq!((r.state, r.usual, r.delta, r.tone), (UsualState::Calibrating, None, None, Tone::Neutral));
    }
}

#[test]
fn resting_hr_is_garmins_daily_value_against_its_own_30_day_range() {
    // Sleep HR, from the nights or Health Status, sits above resting HR and is never its comparison.
    let s = store(Some(51.));
    health(&s, json!([{"type": "HR", "value": 58, "status": "IN_RANGE", "baselineLowerLimit": 52, "baselineUpperLimit": 62}]));
    for i in 1..20 {
        s.put("summary", &day(i), "synthetic", &json!({"restingHeartRate": 45 + i % 5})).unwrap();
    }
    let rows = [
        (47, (UsualState::Within, None, Tone::Good)),
        (53, (UsualState::Above, Some(6.), Tone::Fair)),
        (42, (UsualState::Below, Some(-5.), Tone::Fair)),
    ];
    for (rhr, want) in rows {
        s.put("summary", DATE, "synthetic", &json!({"restingHeartRate": rhr})).unwrap();
        let v = vitals(&s, "08:00").unwrap();
        let r = &v.resting_hr;
        assert_eq!((r.value, r.unit.as_str(), r.usual, r.range_source), (Some(rhr as f64), "bpm", Some([45., 49.]), VitalRangeSource::Raring));
        assert_eq!((r.state, r.delta, r.tone), want, "resting HR {rhr}");
        assert_eq!((v.sleep_hr.value, v.sleep_hr.usual), (Some(58.), Some([52., 62.])));
    }
}

#[test]
fn shown_in_the_morning_and_day_only_after_the_night_syncs() {
    let s = store(Some(51.));
    hrv(&s, 50.);
    for t in ["08:00", "13:00"] {
        assert!(vitals(&s, t).is_some(), "{t}");
    }
    for t in ["21:00", "22:15", "23:30"] {
        assert!(vitals(&s, t).is_none(), "{t}: tonight leads");
    }
    // The morning's night hasn't synced: the neutral wait shows none of yesterday's.
    let s = Store::open(":memory:").unwrap();
    for i in 1..20 {
        night(&s, &day(i), Some(51.));
    }
    let view = screens::today_at(&s, DATE, at("08:00"));
    assert!(view.focus.sleep_missing);
    assert!(view.night_vitals.is_none());
}
