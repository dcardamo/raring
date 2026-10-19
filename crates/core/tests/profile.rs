//! Profile: Garmin facts, latest weigh-in and limits from local data only.
use chrono::{Local, TimeZone};
use raring_core::{store::Store, view::*, Raring};
use serde_json::json;

#[test]
fn profile_reads_stored_facts_and_your_limits() {
    let dir = std::env::temp_dir().join(format!("raring-profile-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let now = Local.with_ymd_and_hms(2025, 3, 13, 9, 0, 0).unwrap();
    let core = Raring::open(dir.join("raring.db")).unwrap();
    let empty = core.profile(now);
    assert_eq!((empty.age, empty.units.as_deref(), empty.weight, empty.watch_last_upload.as_deref()), (None, None, None, None), "nothing invented");
    let store = Store::open(dir.join("raring.db")).unwrap();
    store.put("fitness_age", "2025-03-13", "now", &json!({"fitnessAge": 41, "chronologicalAge": 46})).unwrap();
    store.put("profile_settings", "me", "now", &json!({"measurementSystem": "metric"})).unwrap();
    store.put("weight", "2025-03-01", "now", &json!({"weight": 72400.0, "timestampGMT": 1740823200000_i64})).unwrap();
    core.local_action(&LocalAction::SetLimits { limits: Limits { excluded: vec![Sport::Running], note: "knee".into() } }, now).unwrap();
    let p = core.profile(now);
    assert_eq!((p.age, p.units.as_deref()), (Some(46.), Some("metric")));
    assert_eq!((p.limits.excluded.as_slice(), p.limits.note.as_str()), ([Sport::Running].as_slice(), "knee"));
    assert_eq!((p.weight, p.weight_unit), (Some(72.4), Some(WeightUnit::Kg)));
    assert_eq!((p.weight_freshness.recorded_on.as_deref(), p.weight_freshness.stale), (Some("2025-03-01"), true), "a 12-day-old weigh-in is flagged");
    drop(core);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn fitness_age_carries_the_date_of_garmins_estimate() {
    let store = Store::open(":memory:").unwrap();
    store.put("fitness_age", "2025-03-10", "now", &json!({"fitnessAge": 41, "chronologicalAge": 46})).unwrap();
    let card = raring_core::screens::health(&store, "2025-03-13").fitness_age;
    assert_eq!((card.fitness_age, card.recorded_on.as_deref()), (Some(41.), Some("2025-03-10")), "the latest estimate before the day, with its date");
    assert_eq!(raring_core::screens::health(&store, "2025-03-09").fitness_age.recorded_on, None, "no estimate, no date");
}
