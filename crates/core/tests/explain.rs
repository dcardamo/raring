//! The "Why" sheets and the reconciliation line, on synthetic responses.
use raring_core::{screens, store::Store, view::Tone};
use serde_json::{json, Value};

const DAY: &str = "2025-03-04";

fn store_with(items: &[(&str, Value)]) -> Store {
    let store = Store::open(":memory:").unwrap();
    for (kind, value) in items {
        store.put(kind, DAY, "now", value).unwrap();
    }
    store
}

fn readiness(level: &str, score: f64, feedback: [&str; 6]) -> Value {
    let [sleep, recovery, hrv, acwr, sleep_history, stress_history] = feedback;
    json!([{
        "inputContext": "AFTER_WAKEUP_RESET", "score": score, "level": level,
        "feedbackShort": "TAKE_IT_EASY", "sleepScore": 66, "recoveryTime": 1500,
        "hrvWeeklyAverage": 61, "acuteLoad": 410,
        "sleepScoreFactorPercent": 52, "sleepScoreFactorFeedback": sleep,
        "recoveryTimeFactorPercent": 31, "recoveryTimeFactorFeedback": recovery,
        "hrvFactorPercent": 88, "hrvFactorFeedback": hrv,
        "acwrFactorPercent": 70, "acwrFactorFeedback": acwr,
        "sleepHistoryFactorPercent": 64, "sleepHistoryFactorFeedback": sleep_history,
        "stressHistoryFactorPercent": 77, "stressHistoryFactorFeedback": stress_history
    }])
}

#[test]
fn readiness_why_names_garmins_factors_and_leaves_unrated_ones_out() {
    let store = store_with(&[(
        "readiness",
        readiness(
            "LOW",
            41.,
            ["POOR", "POOR", "VERY_GOOD", "GOOD", "MODERATE", "NONE"],
        ),
    )]);
    let view = screens::today(&store, DAY);
    let stress = view
        .factors
        .iter()
        .find(|f| f.key == "stressHistory")
        .unwrap();
    assert_eq!(stress.percent, None, "Garmin's NONE is a gap, not 0%");
    assert_eq!(stress.tone, Tone::Neutral);
    assert_eq!(view.factors.last().unwrap().key, "stressHistory");
    assert_eq!(view.call_note.as_deref(), Some("Take it easy"));

    let why = view.readiness_why.unwrap();
    assert_eq!(why.title, "Why readiness is Low");
    assert_eq!(why.feedback.as_deref(), Some("Take it easy"));
    assert_eq!(
        why.summary,
        "Recovery time and sleep are holding readiness back; HRV and acute load are helping."
    );
    let keys: Vec<&str> = why.inputs.iter().map(|i| i.key.as_str()).collect();
    assert_eq!(
        keys,
        [
            "recovery",
            "sleep",
            "sleepHistory",
            "acuteLoad",
            "hrv",
            "stressHistory"
        ],
        "worst first by Garmin's percentage, unrated last"
    );
    let input = |key: &str| why.inputs.iter().find(|i| i.key == key).unwrap();
    assert_eq!(input("recovery").value.as_deref(), Some("25 h"));
    assert_eq!(input("recovery").rating.as_deref(), Some("Poor"));
    assert_eq!(input("hrv").value.as_deref(), Some("61 ms"));
    assert_eq!(input("hrv").rating.as_deref(), Some("Very good"));
    assert_eq!(input("stressHistory").rating, None);
    assert_eq!(input("stressHistory").value, None);
    assert!(input("stressHistory").detail.contains("didn't rate"));
}

#[test]
fn readiness_why_uses_moderate_factors_only_below_high() {
    let feedback = ["MODERATE", "GOOD", "GOOD", "MODERATE", "GOOD", "GOOD"];
    let summary = |level| {
        let store = store_with(&[("readiness", readiness(level, 60., feedback))]);
        screens::today(&store, DAY).readiness_why.unwrap().summary
    };
    assert_eq!(
        summary("MODERATE"),
        "Sleep and acute load are holding readiness back; HRV and stress history are helping."
    );
    assert_eq!(
        summary("HIGH"),
        "HRV and stress history are helping readiness most."
    );
}

fn battery_summary() -> Value {
    json!({
        "bodyBatteryMostRecentValue": 44, "bodyBatteryAtWakeTime": 76, "bodyBatteryDuringSleep": 50,
        "averageStressLevel": 40, "stressQualifier": "STRESSFUL",
        "bodyBatteryActivityEventList": [
            {"eventType": "ACTIVITY", "activityName": "Lunch Ride", "bodyBatteryImpact": -11,
             "eventStartTimeGmt": "2025-03-04T17:05:00", "timezoneOffset": -18000000,
             "durationInMilliseconds": 3600000, "shortFeedback": "IMPROVING_AEROBIC_BASE"},
            {"eventType": "SLEEP", "bodyBatteryImpact": 48,
             "eventStartTimeGmt": "2025-03-04T04:00:00", "timezoneOffset": -18000000,
             "durationInMilliseconds": 27000000, "shortFeedback": "NONE"},
            {"eventType": "RECOVERY", "bodyBatteryImpact": 2,
             "eventStartTimeGmt": "2025-03-04T19:30:00", "timezoneOffset": -18000000,
             "durationInMilliseconds": 1200000, "shortFeedback": "RESTFUL_PERIOD"}
        ],
        "bodyBatteryDynamicFeedbackEvent": {"feedbackShortType": "DAY_STRESSFUL_AND_ACTIVE", "bodyBatteryLevel": "MODERATE"}
    })
}

#[test]
fn body_battery_why_lists_the_charge_and_each_drain() {
    let store = store_with(&[("summary", battery_summary())]);
    let why = screens::today(&store, DAY).body_battery_why.unwrap();
    assert_eq!(why.title, "Why Body Battery is 44");
    assert_eq!(why.feedback.as_deref(), Some("Stressful and active so far"));
    assert_eq!(
        why.summary,
        "Sleep charged it by 48, to 76 at wake. Since then: Lunch Ride and a stressful day."
    );
    let rows: Vec<(&str, &str, Option<&str>)> = why
        .inputs
        .iter()
        .map(|i| {
            (
                i.name.as_str(),
                i.value.as_deref().unwrap(),
                i.rating.as_deref(),
            )
        })
        .collect();
    assert_eq!(
        rows,
        [
            ("Overnight", "+48", None),
            ("Stress", "40", Some("Stressful")),
            ("Lunch Ride", "−11", Some("Improving aerobic base")),
            ("Restful period", "+2", None),
        ]
    );
    assert_eq!(
        why.inputs[0].detail,
        "Charge while you slept, 23:00–06:30; 76 at wake."
    );
    assert_eq!(why.inputs[0].tone, Tone::Good);
    assert_eq!(why.inputs[2].detail, "12:05 · 60 min");
}

#[test]
fn body_battery_why_is_absent_without_garmin_events() {
    let store = store_with(&[("summary", json!({"bodyBatteryMostRecentValue": 50}))]);
    assert_eq!(screens::today(&store, DAY).body_battery_why, None);
}

#[test]
fn training_status_why_names_load_focus_hrv_and_acute_load() {
    let training = json!({
        "mostRecentTrainingStatus": {"latestTrainingStatusData": {"1": {
            "primaryTrainingDevice": true, "trainingStatus": 7,
            "trainingStatusFeedbackPhrase": "PRODUCTIVE_2", "trainingPaused": false,
            "acuteTrainingLoadDTO": {"acwrStatus": "HIGH", "dailyAcuteChronicWorkloadRatio": 1.6,
                "dailyTrainingLoadAcute": 520, "minTrainingLoadChronic": 200, "maxTrainingLoadChronic": 400}
        }}},
        "mostRecentTrainingLoadBalance": {"metricsTrainingLoadBalanceDTOMap": {"1": {
            "primaryTrainingDevice": true, "trainingBalanceFeedbackPhrase": "ANAEROBIC_SHORTAGE"
        }}}
    });
    let hrv = json!({"hrvSummary": {"status": "BALANCED", "weeklyAvg": 58}});
    let store = store_with(&[("training", training), ("hrv", hrv)]);
    let view = screens::training(&store, DAY);
    assert_eq!(view.status.as_deref(), Some("Productive"));
    let why = view.status_why.unwrap();
    assert_eq!(why.title, "Why Productive");
    assert_eq!(
        why.summary,
        "Your load is building fitness. Acute load is high; load focus shows an anaerobic shortage."
    );
    let rows: Vec<(&str, Option<&str>, Tone)> = why
        .inputs
        .iter()
        .map(|i| (i.key.as_str(), i.rating.as_deref(), i.tone))
        .collect();
    assert_eq!(
        rows,
        [
            ("acuteLoad", Some("High"), Tone::Poor),
            ("loadFocus", Some("Anaerobic shortage"), Tone::Fair),
            ("hrvStatus", Some("Balanced"), Tone::Good),
        ]
    );
    assert_eq!(why.inputs[0].value.as_deref(), Some("520 · ratio 1.6"));
}

/// Readiness level, sleep score, Body Battery at wake, HRV status, training
/// status, and the score expected to differ.
type Case = (
    &'static str,
    Option<f64>,
    Option<f64>,
    Option<&'static str>,
    Option<&'static str>,
    Option<&'static str>,
);

#[test]
fn scores_that_differ() {
    let cases: [Case; 9] = [
        (
            "HIGH",
            Some(85.),
            Some(90.),
            Some("BALANCED"),
            Some("PRODUCTIVE_1"),
            None,
        ),
        ("LOW", Some(85.), Some(90.), None, None, Some("sleep")),
        ("LOW", Some(70.), Some(90.), None, None, Some("bodyBattery")),
        (
            "LOW",
            Some(70.),
            Some(50.),
            Some("BALANCED"),
            None,
            Some("hrvStatus"),
        ),
        (
            "POOR",
            None,
            None,
            Some("UNBALANCED"),
            Some("PRODUCTIVE_1"),
            Some("trainingStatus"),
        ),
        ("HIGH", Some(55.), None, None, None, Some("sleep")),
        (
            "HIGH",
            Some(70.),
            Some(25.),
            Some("LOW"),
            None,
            Some("bodyBattery"),
        ),
        // Moderate readiness and fair scores never point a different way.
        (
            "MODERATE",
            Some(95.),
            Some(20.),
            Some("LOW"),
            Some("STRAINED_1"),
            None,
        ),
        (
            "LOW",
            Some(70.),
            Some(50.),
            Some("UNBALANCED"),
            Some("RECOVERY_1"),
            None,
        ),
    ];
    for (level, sleep, wake, hrv, status, expected) in cases {
        let mut summary = json!({});
        if let Some(wake) = wake {
            summary["bodyBatteryAtWakeTime"] = json!(wake);
        }
        let mut items = vec![
            (
                "readiness",
                readiness(
                    level,
                    50.,
                    ["MODERATE", "POOR", "GOOD", "VERY_GOOD", "MODERATE", "GOOD"],
                ),
            ),
            ("summary", summary),
        ];
        if let Some(sleep) = sleep {
            items.push((
                "sleep",
                json!({"dailySleepDTO": {"sleepScores": {"overall": {"value": sleep}}}}),
            ));
        }
        if let Some(hrv) = hrv {
            items.push(("hrv", json!({"hrvSummary": {"status": hrv}})));
        }
        if let Some(status) = status {
            items.push(("training", json!({"trainingStatus": status})));
        }
        let store = store_with(&items);
        let found = screens::today(&store, DAY).reconcile;
        assert_eq!(
            found.as_ref().map(|r| r.key.as_str()),
            expected,
            "{level} {sleep:?} {wake:?} {hrv:?} {status:?}"
        );
    }
}

#[test]
fn reconciliation_lines_name_garmins_factors() {
    let line = |level, sleep: f64, wake: Option<f64>| {
        let mut items = vec![
            (
                "readiness",
                readiness(
                    level,
                    50.,
                    ["MODERATE", "POOR", "GOOD", "VERY_GOOD", "MODERATE", "GOOD"],
                ),
            ),
            (
                "sleep",
                json!({"dailySleepDTO": {"sleepScores": {"overall": {"value": sleep}}}}),
            ),
        ];
        if let Some(wake) = wake {
            items.push(("summary", json!({"bodyBatteryAtWakeTime": wake})));
        }
        let r = screens::today(&store_with(&items), DAY).reconcile.unwrap();
        (r.label, r.line, r.tones)
    };
    assert_eq!(
        line("LOW", 70., Some(88.)),
        (
            "Why Body Battery and readiness differ".into(),
            "Body Battery charged to 88 overnight, but readiness is held back by recovery time."
                .into(),
            [Tone::Good, Tone::Poor]
        )
    );
    assert_eq!(
        line("HIGH", 52., None).1,
        "Sleep scored 52, but readiness is held up by HRV and stress history."
    );
}

#[test]
fn reconciliation_stays_generic_without_a_suitably_rated_factor() {
    let line = |level, feedback, sleep: f64| {
        let items = [
            ("readiness", readiness(level, 50., feedback)),
            (
                "sleep",
                json!({"dailySleepDTO": {"sleepScores": {"overall": {"value": sleep}}}}),
            ),
        ];
        screens::today(&store_with(&items), DAY)
            .reconcile
            .unwrap()
            .line
    };
    assert_eq!(
        line("LOW", ["GOOD"; 6], 85.),
        "Sleep scored 85, but readiness is low.",
        "a Good factor never holds readiness back"
    );
    assert_eq!(
        line(
            "HIGH",
            ["POOR", "MODERATE", "POOR", "MODERATE", "MODERATE", "POOR"],
            50.
        ),
        "Sleep scored 50, but readiness is high.",
        "a Poor or Moderate factor never holds readiness up"
    );
}

/// Each score's own boundaries: readiness by Garmin's level (Moderate from 50,
/// High from 75), Sleep Score 60 and 80, Body Battery 30 and 70.
#[test]
fn scores_that_differ_at_level_boundaries() {
    let differs = |score: f64, level: &str, sleep: Option<f64>, wake: Option<f64>| {
        let mut items = vec![(
            "readiness",
            readiness(
                level,
                score,
                ["MODERATE", "POOR", "GOOD", "VERY_GOOD", "MODERATE", "GOOD"],
            ),
        )];
        if let Some(sleep) = sleep {
            items.push((
                "sleep",
                json!({"dailySleepDTO": {"sleepScores": {"overall": {"value": sleep}}}}),
            ));
        }
        if let Some(wake) = wake {
            items.push(("summary", json!({"bodyBatteryAtWakeTime": wake})));
        }
        screens::today(&store_with(&items), DAY)
            .reconcile
            .map(|r| r.key)
    };
    let some = |key: &str| Some(key.to_owned());
    // Readiness against a good sleep, then a poor one.
    assert_eq!(differs(49., "LOW", Some(85.), None), some("sleep"));
    assert_eq!(differs(50., "MODERATE", Some(85.), None), None);
    assert_eq!(differs(74., "MODERATE", Some(50.), None), None);
    assert_eq!(differs(75., "HIGH", Some(50.), None), some("sleep"));
    // Sleep Score against poor, then high, readiness.
    assert_eq!(differs(40., "LOW", Some(79.), None), None);
    assert_eq!(differs(40., "LOW", Some(80.), None), some("sleep"));
    assert_eq!(differs(85., "HIGH", Some(59.), None), some("sleep"));
    assert_eq!(differs(85., "HIGH", Some(60.), None), None);
    // Body Battery at wake against poor, then high, readiness.
    assert_eq!(differs(40., "LOW", None, Some(69.)), None);
    assert_eq!(differs(40., "LOW", None, Some(70.)), some("bodyBattery"));
    assert_eq!(differs(85., "HIGH", None, Some(29.)), some("bodyBattery"));
    assert_eq!(differs(85., "HIGH", None, Some(30.)), None);
}

#[test]
fn training_status_inputs_are_unrated_without_garmins_rating() {
    let store = store_with(&[
        (
            "training",
            json!({"trainingStatus": "PRODUCTIVE", "acuteTrainingLoad": 612,
                   "optimalRangeMin": 420, "optimalRangeMax": 760}),
        ),
        (
            "load_focus",
            json!({"lowAerobic": {"value": 450}, "anaerobic": {"value": 60}}),
        ),
    ]);
    let why = screens::training(&store, DAY).status_why.unwrap();
    let input = |key: &str| why.inputs.iter().find(|i| i.key == key).unwrap();
    assert_eq!(input("acuteLoad").value.as_deref(), Some("612"));
    assert_eq!(input("acuteLoad").rating, None, "no acwrStatus, no rating");
    assert_eq!(input("acuteLoad").tone, Tone::Neutral);
    assert_eq!(
        input("loadFocus").rating,
        None,
        "no balance phrase, no rating"
    );
    assert_eq!(input("loadFocus").tone, Tone::Neutral);
    assert_eq!(why.summary, "Your load is building fitness.");
}

fn event(kind: &str, start: &str, impact: f64, name: Option<&str>) -> Value {
    let mut e = json!({"eventType": kind, "eventStartTimeGmt": start, "bodyBatteryImpact": impact,
        "timezoneOffset": 0, "durationInMilliseconds": 1_800_000, "shortFeedback": "NONE"});
    if let Some(name) = name {
        e["activityName"] = json!(name);
    }
    e
}

#[test]
fn body_battery_merges_partial_event_lists() {
    // The summary has only sleep and one named workout; the report has every
    // event, without names and with fractional seconds on the times.
    let summary = json!({"bodyBatteryActivityEventList": [
        event("SLEEP", "2025-03-04T04:00:00", 40., None),
        event("ACTIVITY", "2025-03-04T12:00:00", -8., Some("Lunch Ride")),
    ]});
    let report = json!({"bodyBatteryActivityEvent": [
        event("SLEEP", "2025-03-04T04:00:00.0", 40., None),
        event("ACTIVITY", "2025-03-04T12:00:00.0", -8., None),
        event("NAP", "2025-03-04T14:00:00.0", 3., None),
        event("ACTIVITY", "2025-03-04T17:00:00.0", -5., None),
    ]});
    let store = store_with(&[("summary", summary), ("battery", report)]);
    let why = screens::today(&store, DAY).body_battery_why.unwrap();
    let names: Vec<&str> = why.inputs.iter().map(|i| i.name.as_str()).collect();
    assert_eq!(names, ["Overnight", "Lunch Ride", "Nap", "Workout"]);
    assert!(
        why.summary.ends_with("Since then: 2 workouts."),
        "{}",
        why.summary
    );
}

#[test]
fn body_battery_ignores_absurd_offsets_and_durations() {
    let mut sleep = event("SLEEP", "2025-03-04T04:00:00", 40., None);
    sleep["timezoneOffset"] = json!(1e300);
    sleep["durationInMilliseconds"] = json!(1e300);
    let mut late = event("ACTIVITY", "+262142-12-31T23:59:59", -3., Some("Edge"));
    late["timezoneOffset"] = json!(18 * 3_600_000);
    late["durationInMilliseconds"] = json!(-5);
    let mut huge = event("NAP", "2025-03-04T13:00:00", 1., None);
    huge["timezoneOffset"] = json!(-9.2e18);
    huge["durationInMilliseconds"] = json!(i64::MAX);
    let store = store_with(&[(
        "summary",
        json!({"bodyBatteryActivityEventList": [sleep, late, huge]}),
    )]);
    let why = screens::today(&store, DAY).body_battery_why.unwrap();
    let overnight = &why.inputs[0];
    assert_eq!(
        overnight.detail, "Charge while you slept.",
        "no window from bad numbers"
    );
    for input in &why.inputs[1..] {
        assert_eq!(
            input.detail, "",
            "{}: no time or length from bad numbers",
            input.name
        );
    }
}

#[test]
fn body_battery_keeps_distinct_events_that_share_a_time() {
    let mut warmup = event("ACTIVITY", "2025-03-04T12:00:00", -2., Some("Warm-up"));
    warmup["activityId"] = json!(101);
    let mut intervals = event("ACTIVITY", "2025-03-04T12:00:00", -9., Some("Intervals"));
    intervals["activityId"] = json!(102);
    let summary = json!({"bodyBatteryActivityEventList": [warmup, intervals]});
    let mut longer = event("ACTIVITY", "2025-03-04T12:00:00.0", -4., None);
    longer["durationInMilliseconds"] = json!(3_600_000);
    let report = json!({"bodyBatteryActivityEvent": [
        // The report's copies of the two named workouts.
        event("ACTIVITY", "2025-03-04T12:00:00.0", -2., None),
        event("ACTIVITY", "2025-03-04T12:00:00.0", -9., None),
        // Same start, different length: another event.
        longer,
        // Two naps at the same time in one list both count.
        event("NAP", "2025-03-04T15:00:00.0", 1., None),
        event("NAP", "2025-03-04T15:00:00.0", 2., None),
    ]});
    let store = store_with(&[("summary", summary), ("battery", report)]);
    let why = screens::today(&store, DAY).body_battery_why.unwrap();
    let mut rows: Vec<(&str, &str)> = why
        .inputs
        .iter()
        .map(|i| (i.name.as_str(), i.value.as_deref().unwrap()))
        .collect();
    rows.sort();
    assert_eq!(
        rows,
        [
            ("Intervals", "−9"),
            ("Nap", "+1"),
            ("Nap", "+2"),
            ("Warm-up", "−2"),
            ("Workout", "−4"),
        ]
    );
}

#[test]
fn percent_only_factors_stay_unrated_in_why_and_reconciliation() {
    // Recovery has Garmin's percentage but no rating; the rest are rated.
    let mut r = readiness(
        "LOW",
        40.,
        ["GOOD", "GOOD", "GOOD", "GOOD", "MODERATE", "GOOD"],
    );
    r[0].as_object_mut()
        .unwrap()
        .remove("recoveryTimeFactorFeedback");
    let store = store_with(&[
        ("readiness", r),
        (
            "sleep",
            json!({"dailySleepDTO": {"sleepScores": {"overall": {"value": 85}}}}),
        ),
    ]);
    let view = screens::today(&store, DAY);
    let why = view.readiness_why.unwrap();
    let recovery = why.inputs.iter().find(|i| i.key == "recovery").unwrap();
    assert_eq!(
        (recovery.rating.as_deref(), recovery.tone),
        (None, Tone::Neutral)
    );
    assert_eq!(
        why.summary,
        "Sleep history is holding readiness back; HRV and stress history are helping."
    );
    assert_eq!(
        view.reconcile.unwrap().line,
        "Sleep scored 85, but readiness is held back by sleep history."
    );
    let factor = view.factors.iter().find(|f| f.key == "recovery").unwrap();
    assert_eq!((factor.tone, factor.percent), (Tone::Neutral, Some(31.)));
}
