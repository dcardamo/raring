use raring_core::{screens, store::Store, stress, view::*};
use serde_json::{json, Value};

const TODAY: &str = "2026-03-18";

fn put(store: &Store, endpoint: &str, key: &str, value: Value) {
    store.put(endpoint, key, "synthetic", &value).unwrap();
}
fn day(month: &StressMonth, date: &str) -> StressDay {
    month.days.iter().find(|d| d.date == date).cloned().unwrap()
}

#[test]
fn each_day_takes_garmins_word_and_partly_worn_days_stay_uncoloured() {
    let store = Store::open(":memory:").unwrap();
    for (date, average, word) in [
        ("2026-03-02", 30, "BALANCED"),
        ("2026-03-03", 38, "STRESSFUL"),
        ("2026-03-04", 44, "VERY_STRESSFUL"),
        ("2026-03-05", 21, "CALM"),
        ("2026-03-06", 55, "UNKNOWN"),
        ("2026-03-07", -1, "UNKNOWN"),
        ("2026-03-08", 46, "BALANCED_AWAKE"),
    ] {
        put(&store, "stress", date, json!({"avgStressLevel": average}));
        put(&store, "summary", date, json!({"averageStressLevel": average, "stressQualifier": word}));
    }
    // History rows spell the word in lowercase; a row without one keeps its average.
    put(&store, "daily_summary", "2026-03-09", json!({"stress": {"avgLevel": 40, "qualifier": "stressful"}}));
    put(&store, "summary", "2026-03-10", json!({"averageStressLevel": 33}));
    put(&store, "daily_summary", "2025-11-20", json!({"stress": {"avgLevel": 29, "qualifier": "balanced"}}));
    // Two workouts on the 2nd and one late on the 3rd, counted on the day they started.
    put(&store, "listed", "1", json!({"activityId": 1, "startTimeLocal": "2026-03-02 07:00:00", "duration": 1800, "activityType": {"typeKey": "running"}}));
    put(&store, "listed", "2", json!({"activityId": 2, "startTimeLocal": "2026-03-02 18:00:00", "duration": 2400, "activityType": {"typeKey": "strength_training"}}));
    put(&store, "listed", "3", json!({"activityId": 3, "startTimeLocal": "2026-03-03 23:30:00", "duration": 3600, "activityType": {"typeKey": "cycling"}}));

    let month = stress::month(&store, "2026-03", TODAY).unwrap();
    assert_eq!((month.month.as_str(), month.today.as_str(), month.days.len()), ("2026-03-01", TODAY, 31));
    assert_eq!(month.first.as_deref(), Some("2025-11-01"));
    let shown = |date: &str| {
        let d = day(&month, date);
        (d.kind, d.word, d.tone, d.average, d.workouts)
    };
    assert_eq!(shown("2026-03-02"), (StressDayKind::Rated, Some("Balanced".into()), Tone::Good, Some(30.), 2));
    assert_eq!(shown("2026-03-03"), (StressDayKind::Rated, Some("Stressful".into()), Tone::Fair, Some(38.), 1));
    assert_eq!(shown("2026-03-04"), (StressDayKind::Rated, Some("Very Stressful".into()), Tone::Poor, Some(44.), 0));
    assert_eq!(shown("2026-03-05").2, Tone::Good, "Calm");
    assert_eq!(shown("2026-03-06"), (StressDayKind::LowWear, None, Tone::Neutral, Some(55.), 0));
    assert_eq!(shown("2026-03-07"), (StressDayKind::None, None, Tone::Neutral, None, 0), "Garmin's -1 is no data");
    assert_eq!(shown("2026-03-08"), (StressDayKind::AwakeOnly, Some("Balanced".into()), Tone::Neutral, Some(46.), 0));
    assert_eq!(shown("2026-03-09"), (StressDayKind::Rated, Some("Stressful".into()), Tone::Fair, Some(40.), 0));
    assert_eq!(shown("2026-03-10"), (StressDayKind::Unrated, None, Tone::Neutral, Some(33.), 0));
    assert_eq!(shown("2026-03-11").0, StressDayKind::None);
    assert_eq!(shown("2026-03-18").0, StressDayKind::None, "today without readings");
    assert!(month.days[18..].iter().all(|d| d.kind == StressDayKind::Future && d.average.is_none()));
}

#[test]
fn a_month_is_named_by_any_date_in_it_and_never_runs_past_today() {
    let store = Store::open(":memory:").unwrap();
    let feb = stress::month(&store, "2024-02-17", TODAY).unwrap();
    assert_eq!((feb.month.as_str(), feb.days.len(), feb.first), ("2024-02-01", 29, None));
    assert!(feb.days.iter().all(|d| d.kind == StressDayKind::None));
    assert!(stress::month(&store, "2026-04", TODAY).unwrap_err().contains("no later than"));
    assert!(stress::month(&store, "1989-12", TODAY).unwrap_err().contains("1990"));
    assert!(stress::month(&store, "March", TODAY).unwrap_err().contains("Invalid month"));
}

#[test]
fn the_stress_detail_marks_workouts_and_leaves_awake_only_words_uncoloured() {
    let store = Store::open(":memory:").unwrap();
    // Midnight local is 05:00 GMT; one workout has a GMT start, the other only its local time.
    put(&store, "stress", "2026-03-02", json!({
        "startTimestampGMT": "2026-03-02T05:00:00.0", "startTimestampLocal": "2026-03-02T00:00:00.0",
        "endTimestampGMT": "2026-03-03T05:00:00.0", "endTimestampLocal": "2026-03-03T00:00:00.0",
        "avgStressLevel": 46, "stressValuesArray": [[1772427600000_i64, 20]]
    }));
    put(&store, "summary", "2026-03-02", json!({"averageStressLevel": 46, "stressQualifier": "VERY_STRESSFUL_AWAKE"}));
    put(&store, "listed", "1", json!({"activityId": 1, "startTimeLocal": "2026-03-02 07:00:00", "startTimeGMT": "2026-03-02 12:00:00", "duration": 1800, "activityType": {"typeKey": "running"}}));
    put(&store, "listed", "2", json!({"activityId": 2, "startTimeLocal": "2026-03-02T18:00:00", "duration": 3600, "activityType": {"typeKey": "cycling"}}));
    let card = screens::stress(&store, "2026-03-02");
    assert_eq!(card.workouts, vec![[420., 450.], [1080., 1140.]]);
    assert_eq!(card.qualifier.as_deref(), Some("Very Stressful Awake"));
    assert_eq!(card.tone, Tone::Neutral, "the word covers only the time awake");
    // A whole-day word keeps its colour.
    put(&store, "summary", "2026-03-02", json!({"averageStressLevel": 46, "stressQualifier": "VERY_STRESSFUL"}));
    assert_eq!(screens::stress(&store, "2026-03-02").tone, Tone::Poor);
}

#[test]
fn each_day_carries_garmins_rating_of_the_night_before() {
    let store = Store::open(":memory:").unwrap();
    // The night that ended on the morning of the 2nd, from its own record.
    put(&store, "sleep", "2026-03-02", json!({"dailySleepDTO": {"avgSleepStress": 39.0,
        "sleepScores": {"stress": {"qualifierKey": "POOR", "optimalStart": 0.0, "optimalEnd": 15.0}}}}));
    put(&store, "sleep", "2026-03-03", json!({"dailySleepDTO": {"avgSleepStress": 12.0, "sleepScores": {"stress": {"qualifierKey": "EXCELLENT"}}}}));
    // History rows keep the same fields at the top level; the night's own record wins.
    put(&store, "sleep_summary", "2026-03-04", json!({"avgSleepStress": 22.0, "sleepScores": {"stress": {"qualifierKey": "FAIR"}}}));
    put(&store, "sleep_summary", "2026-03-02", json!({"avgSleepStress": 20.0, "sleepScores": {"stress": {"qualifierKey": "FAIR"}}}));
    // A night without a rating keeps its average; Garmin's negative placeholder is no average.
    put(&store, "sleep", "2026-03-05", json!({"dailySleepDTO": {"avgSleepStress": 18.0}}));
    put(&store, "sleep", "2026-03-06", json!({"dailySleepDTO": {"avgSleepStress": -1.0}}));
    let month = stress::month(&store, "2026-03", TODAY).unwrap();
    let night = |date: &str| day(&month, date).night.map(|n| (n.rating, n.tone, n.average));
    assert_eq!(night("2026-03-02"), Some((Some("Poor".into()), Tone::Poor, Some(39.))));
    assert_eq!(night("2026-03-03"), Some((Some("Excellent".into()), Tone::Good, Some(12.))));
    assert_eq!(night("2026-03-04"), Some((Some("Fair".into()), Tone::Fair, Some(22.))));
    assert_eq!(night("2026-03-05"), Some((None, Tone::Neutral, Some(18.))));
    assert_eq!(night("2026-03-06"), None);
    assert_eq!(night("2026-03-07"), None);
}
