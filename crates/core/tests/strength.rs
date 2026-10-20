use chrono::{Duration, Local, Utc};
use raring_core::{
    garmin::{Client, Tokens},
    screens,
    store::Store,
    strength, sync,
    view::{Figure, FigureChoice, Muscle, SetFlag, WeightUnit},
};
use serde_json::{json, Value};
use std::{
    io::{Read, Write},
    sync::{Arc, Mutex},
};

fn fixture(name: &str) -> Value {
    let path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}
fn activity(store: &Store, id: &str, start: &str) {
    store
        .put(
            "activity",
            id,
            "now",
            &json!({"activityId": id.parse::<u64>().unwrap(), "activityName": "Strength",
                "activityTypeDTO": {"typeKey": "strength_training"},
                "summaryDTO": {"startTimeLocal": start, "duration": 1800}}),
        )
        .unwrap();
}
fn set(category: &str, name: Option<&str>, reps: Option<u32>, grams: f64) -> Value {
    json!({"setType": "ACTIVE", "repetitionCount": reps, "weight": grams, "duration": 30.0,
        "exercises": [{"category": category, "name": name, "probability": 90.0}]})
}
fn rest(seconds: f64) -> Value {
    json!({"setType": "REST", "duration": seconds, "exercises": []})
}
fn session(store: &Store, id: &str, start: &str, sets: Vec<Value>) {
    activity(store, id, start);
    store
        .put("exercise_sets", id, "now", &json!({"activityId": 1, "exerciseSets": sets}))
        .unwrap();
}
fn with_catalog() -> Store {
    let store = Store::open(":memory:").unwrap();
    let catalog = strength::reduce_catalog(&fixture("exercise_catalog.json")).unwrap();
    store
        .put("exercise_catalog", "garmin", "now", &catalog)
        .unwrap();
    store
}
fn sets_for(muscles: &[raring_core::view::MuscleSets], muscle: Muscle) -> f64 {
    muscles.iter().find(|m| m.muscle == muscle).unwrap().sets
}

#[test]
fn a_session_groups_sets_by_exercise_and_flags_miscounts() {
    let store = with_catalog();
    activity(&store, "123456", "2025-03-04T18:00:00.0");
    store
        .put("exercise_sets", "123456", "now", &fixture("exercise_sets.json"))
        .unwrap();
    let view = screens::workout(&store, "123456").unwrap();
    let s = view.strength.unwrap();
    assert_eq!((s.sets, s.reps, s.unknown_rep_sets, s.rest_s), (6, Some(51), 0, 418.));
    assert!(s.sets_recorded);
    assert_eq!(
        (s.zero_rep_sets, s.missing_rep_sets, s.doubled_sets, s.unnamed_sets),
        (1, 0, 1, 1)
    );
    assert_eq!(s.unit, WeightUnit::Kg);
    let names: Vec<_> = s.exercises.iter().map(|e| e.name.clone()).collect();
    assert_eq!(
        names,
        [
            Some("Goblet Squat".into()),
            Some("One Arm Dumbbell Row".into()),
            None
        ]
    );
    let squat = &s.exercises[0];
    assert_eq!(squat.key.as_deref(), Some("SQUAT/GOBLET_SQUAT"));
    assert_eq!(squat.muscles, [Muscle::Quads]);
    assert_eq!(squat.sets[0].weight, Some(24.));
    // 16 is at least 1.8 times the median (8) of the other two sets; 8 is not.
    assert_eq!(
        squat.sets.iter().map(|s| s.flags.clone()).collect::<Vec<_>>(),
        [vec![], vec![], vec![SetFlag::Doubled]]
    );
    assert_eq!(s.exercises[1].muscles, [Muscle::Back, Muscle::Arms]);
    assert_eq!(s.exercises[1].sets[1].flags, [SetFlag::ZeroReps]);
    let unknown = &s.exercises[2];
    assert_eq!(unknown.sets[0].flags, [SetFlag::Unnamed]);
    assert_eq!(unknown.sets[0].weight, None);
    assert!(unknown.muscles.is_empty());
    // Each set counts once per muscle group; unnamed sets count for none.
    assert_eq!(sets_for(&s.muscles, Muscle::Quads), 3.);
    assert_eq!(sets_for(&s.muscles, Muscle::Back), 2.);
    assert_eq!(sets_for(&s.muscles, Muscle::Arms), 2.);
    assert_eq!(sets_for(&s.muscles, Muscle::Core), 0.);
    // A single first session has no records.
    assert_eq!(s.records, 0);
}

#[test]
fn a_session_without_lap_presses_is_not_called_recorded() {
    let store = with_catalog();
    activity(&store, "123456", "2025-03-06T18:00:00.0");
    store
        .put(
            "exercise_sets",
            "123456",
            "now",
            &fixture("exercise_sets_untracked.json"),
        )
        .unwrap();
    let s = screens::workout(&store, "123456").unwrap().strength.unwrap();
    assert_eq!((s.sets, s.reps, s.unknown_rep_sets), (1, None, 1));
    assert!(!s.sets_recorded);
    assert_eq!(s.exercises[0].sets[0].reps, None);
    assert_eq!(
        s.exercises[0].sets[0].flags,
        [SetFlag::Unnamed, SetFlag::MissingReps]
    );
}

#[test]
fn doubled_needs_another_set_and_uses_the_median_of_the_others() {
    let store = with_catalog();
    let squat = |reps| set("SQUAT", Some("GOBLET_SQUAT"), Some(reps), 20_000.);
    session(&store, "1", "2025-03-04T18:00:00", vec![squat(30)]);
    session(&store, "2", "2025-03-05T18:00:00", vec![squat(10), squat(17)]);
    session(&store, "3", "2025-03-06T18:00:00", vec![squat(10), squat(18)]);
    session(&store, "4", "2025-03-07T18:00:00", vec![squat(10), squat(12), squat(30), squat(19)]);
    let flagged = |id: &str| {
        screens::workout(&store, id).unwrap().strength.unwrap().exercises[0]
            .sets
            .iter()
            .map(|s| s.flags.contains(&SetFlag::Doubled))
            .collect::<Vec<_>>()
    };
    assert_eq!(flagged("1"), [false]);
    assert_eq!(flagged("2"), [false, false]);
    assert_eq!(flagged("3"), [false, true]);
    // Others of 19: 10, 12, 30 → median 12; 19 < 21.6. Others of 30: median 12 → flagged.
    assert_eq!(flagged("4"), [false, false, true, false]);
}

#[test]
fn records_need_an_earlier_session_and_no_set_as_heavy_for_as_many_reps() {
    let store = with_catalog();
    let squat = |reps, kg: f64| set("SQUAT", Some("LEG_PRESS"), Some(reps), kg * 1000.);
    session(&store, "1", "2025-03-03T18:00:00", vec![squat(8, 100.), squat(5, 110.)]);
    session(
        &store,
        "2",
        "2025-03-05T18:00:00",
        vec![
            squat(8, 105.),
            squat(8, 105.),
            squat(10, 90.),
            squat(5, 100.),
            squat(0, 200.),
            squat(20, 105.),
            squat(8, 105.),
        ],
    );
    let first = screens::workout(&store, "1").unwrap().strength.unwrap();
    assert_eq!(first.records, 0);
    let second = screens::workout(&store, "2").unwrap().strength.unwrap();
    let records: Vec<bool> = second.exercises[0].sets.iter().map(|s| s.record).collect();
    // 105×8 beats 100×8; the repeat ties; 90×10 is a new 10-rep best; 100×5
    // is under 110×5; the 0-rep and doubled sets never count.
    assert_eq!(records, [true, false, true, false, false, false, false]);
    assert_eq!(second.records, 2);
    let pill = second.exercises[0].record.clone().unwrap();
    assert_eq!((pill.weight, pill.reps), (Some(105.), Some(8)));
    assert_eq!(second.exercises[0].last_date.as_deref(), Some("2025-03-03"));
    assert_eq!(second.exercises[0].last_sets.len(), 2);

    let history = screens::strength(&store, "2025-03-06").unwrap();
    let press = &history.exercises[0];
    assert_eq!(press.name, "Leg Press");
    assert_eq!(press.sessions.len(), 2);
    assert_eq!(press.sessions[0].date, "2025-03-05");
    assert!(press.sessions[0].record);
    let top = press.sessions[0].top.clone().unwrap();
    assert_eq!((top.weight, top.reps), (Some(105.), Some(8)));
    assert_eq!(press.sessions[0].reps.len(), 7);
    let best = |n: u32| {
        press
            .bests
            .iter()
            .find(|b| b.reps == n)
            .map(|b| (b.weight, b.date.clone()))
            .unwrap()
    };
    assert_eq!(best(1), (Some(110.), Some("2025-03-03".into())));
    assert_eq!(best(8), (Some(105.), Some("2025-03-05".into())));
    assert_eq!(best(10), (Some(90.), Some("2025-03-05".into())));
    assert_eq!(press.bests.len(), 5);
}

#[test]
fn bodyweight_sets_get_a_most_reps_best_not_a_weight_record() {
    let store = with_catalog();
    let crunch = |reps| set("CORE", Some("BICYCLE_CRUNCH"), Some(reps), 0.);
    session(&store, "1", "2025-03-03T18:00:00", vec![crunch(20), crunch(18)]);
    session(&store, "2", "2025-03-05T18:00:00", vec![crunch(30), crunch(30)]);
    // 40 is flagged (1.8 × the 20-rep median of its others); the unnamed
    // 100-rep set has no exercise to belong to.
    session(
        &store,
        "3",
        "2025-03-07T18:00:00",
        vec![crunch(20), crunch(20), crunch(40), set("CORE", None, Some(100), 0.)],
    );
    let history = screens::strength(&store, "2025-03-08").unwrap();
    let core = history.exercises.iter().find(|e| e.key == "CORE/BICYCLE_CRUNCH").unwrap();
    assert_eq!(core.reps_best.as_ref().map(|b| (b.reps, b.date.as_str())), Some((30, "2025-03-05")));
    assert!(core.bests.iter().all(|b| b.weight.is_none()), "no weight, no weighted best");
    let workout = screens::workout(&store, "2").unwrap().strength.unwrap();
    assert_eq!(workout.records, 0, "bodyweight sets never award a weighted record");
    // Garmin's weight 0 and a missing weight are the same bodyweight set.
    session(&store, "4", "2025-03-09T18:00:00", vec![set("CORE", Some("BICYCLE_CRUNCH"), Some(35), 0.)]);
    let history = screens::strength(&store, "2025-03-10").unwrap();
    let core = history.exercises.iter().find(|e| e.key == "CORE/BICYCLE_CRUNCH").unwrap();
    assert_eq!(core.reps_best.as_ref().map(|b| b.reps), Some(35));
}

#[test]
fn weeks_units_and_figure_follow_garmin_settings() {
    let store = with_catalog();
    store
        .put(
            "profile_settings",
            "me",
            "now",
            &json!({"userData": {"measurementSystem": "statute_us", "gender": "MALE",
                "firstDayOfWeek": {"dayId": 2, "dayName": "sunday"}, "birthDate": "2000-01-01",
                "timeFormat": "time_twelve_hr", "weight": 65000}}),
        )
        .unwrap();
    let row = |reps| set("ROW", Some("ONE_ARM_DUMBBELL_ROW"), Some(reps), 22_679.6);
    // 2025-03-09 is a Sunday: it starts the week that holds Thursday 2025-03-13.
    session(&store, "1", "2025-03-08T09:00:00", vec![row(10), row(10)]);
    session(&store, "2", "2025-03-09T09:00:00", vec![row(10), rest(60.), row(10)]);
    session(
        &store,
        "3",
        "2025-03-12T09:00:00",
        vec![set("CRUNCH", Some("BICYCLE_CRUNCH"), Some(20), 0.)],
    );
    let week = screens::training(&store, "2025-03-13").strength.unwrap();
    assert_eq!(week.week_start, "2025-03-09");
    assert_eq!(
        (week.sets, week.sessions_with_sets, week.strength_activities),
        (3, 2, 2)
    );
    assert_eq!(week.figure, Figure::Male);
    assert_eq!(sets_for(&week.muscles, Muscle::Back), 2.);
    // ABS and OBLIQUES are both Core: the set counts once.
    assert_eq!(sets_for(&week.muscles, Muscle::Core), 1.);
    let s = screens::workout(&store, "1").unwrap().strength.unwrap();
    assert_eq!(s.unit, WeightUnit::Lb);
    assert_eq!(s.exercises[0].sets[0].weight, Some(50.));
    let history = screens::strength(&store, "2025-03-13").unwrap();
    assert_eq!(history.weeks.len(), 12);
    assert_eq!(history.weeks[11].week_start, "2025-03-09");
    assert_eq!(history.weeks[10].week_start, "2025-03-02");
    assert_eq!(sets_for(&history.weeks[10].muscles, Muscle::Back), 2.);
    assert_eq!(sets_for(&history.this_week, Muscle::Back), 2.);
    // The previous four weeks average 2 back sets over 4 weeks.
    assert_eq!(sets_for(&history.average_4w, Muscle::Back), 0.5);
    assert_eq!(sets_for(&history.last_4w, Muscle::Back), 4.);
    assert_eq!(history.sessions_with_sets_this_week, 2);
    assert_eq!(history.workouts.len(), 3);
    assert_eq!(
        strength::reduce_settings(&store.get("profile_settings", "me").unwrap()),
        json!({"measurementSystem": "statute_us", "timeFormat": "time_twelve_hr", "firstDayOfWeek": "sunday", "gender": "MALE"})
    );
    let safe = strength::reduce_settings(&store.get("profile_settings", "me").unwrap());
    assert_eq!(strength::reduce_settings(&safe), safe);
}

#[test]
fn strength_s_figure_picker_overrides_the_garmin_profile() {
    let store = with_catalog();
    store
        .put("profile_settings", "me", "now", &json!({"userData": {"gender": "FEMALE"}}))
        .unwrap();
    session(&store, "1", "2025-03-04T18:00:00", vec![set("SQUAT", Some("GOBLET_SQUAT"), Some(8), 20_000.)]);
    let figure = |store: &Store| screens::strength(store, "2025-03-04").unwrap().figure;
    assert_eq!(figure(&store), Figure::Female, "the profile's figure until someone picks");
    let mut settings = store.settings();
    settings.body_figure = FigureChoice::Male;
    store.set_settings(&settings).unwrap();
    assert_eq!(figure(&store), Figure::Male);
    settings.body_figure = FigureChoice::Female;
    store.set_settings(&settings).unwrap();
    assert_eq!(figure(&store), Figure::Female);
    settings.body_figure = FigureChoice::Profile;
    store.set_settings(&settings).unwrap();
    assert_eq!(figure(&store), Figure::Female, "Profile returns to Garmin's figure");
}

#[test]
fn muscles_fall_back_from_exercise_to_category_to_raring_mapping() {
    let store = with_catalog();
    session(
        &store,
        "1",
        "2025-03-04T18:00:00",
        vec![
            // Not in the catalog excerpt: the category's muscles.
            set("SQUAT", Some("PISTOL_SQUAT"), Some(5), 0.),
            // Only a category: the category's muscles.
            set("ROW", None, Some(5), 0.),
            // A catalog exercise in a category without muscles of its own.
            set("BANDED_EXERCISES", Some("CLAM_SHELLS"), Some(5), 0.),
            // Not in the catalog at all: Raring's mapping.
            set("CALF_RAISE", Some("STANDING_CALF_RAISE"), Some(5), 0.),
        ],
    );
    let s = screens::workout(&store, "1").unwrap().strength.unwrap();
    let muscles: Vec<_> = s.exercises.iter().map(|e| e.muscles.clone()).collect();
    assert_eq!(
        muscles,
        [
            vec![Muscle::Quads, Muscle::Glutes],
            vec![Muscle::Back],
            vec![Muscle::Hips],
            vec![Muscle::Calves],
        ]
    );
    // Without any catalog, Raring's mapping still names the muscles.
    let bare = Store::open(":memory:").unwrap();
    session(&bare, "1", "2025-03-04T18:00:00", vec![set("SQUAT", Some("PISTOL_SQUAT"), Some(5), 0.)]);
    let s = screens::workout(&bare, "1").unwrap().strength.unwrap();
    assert_eq!(s.exercises[0].muscles, [Muscle::Quads, Muscle::Glutes]);
    assert!(screens::strength(&Store::open(":memory:").unwrap(), "2025-03-04").is_none());
    // Timed sessions without sets show no Training card and no history.
    let timed = Store::open(":memory:").unwrap();
    activity(&timed, "123456", "2025-03-06T18:00:00");
    timed
        .put("exercise_sets", "123456", "now", &fixture("exercise_sets_untracked.json"))
        .unwrap();
    assert!(screens::training(&timed, "2025-03-07").strength.is_none());
    assert!(screens::strength(&timed, "2025-03-07").is_none());
}

#[test]
fn timed_sessions_add_no_sets_but_stay_real_workouts() {
    let store = with_catalog();
    let squat = |reps| set("SQUAT", Some("GOBLET_SQUAT"), Some(reps), 20_000.);
    // Monday-start week of Thursday 2025-03-13: a full session, a session
    // timed without lap presses, and a strength activity Garmin has no sets for.
    session(&store, "1", "2025-03-10T18:00:00", vec![squat(8), rest(90.), squat(8), squat(8)]);
    activity(&store, "2", "2025-03-11T18:00:00");
    store
        .put("exercise_sets", "2", "now", &fixture("exercise_sets_untracked.json"))
        .unwrap();
    activity(&store, "3", "2025-03-12T18:00:00");
    store.put("exercise_sets", "3", "now", &json!({"activityId": 3, "exerciseSets": null})).unwrap();

    let training = screens::training(&store, "2025-03-13");
    let week = training.strength.unwrap();
    assert_eq!(
        (week.sets, week.sessions_with_sets, week.strength_activities, week.records),
        (3, 1, 3, 0)
    );
    assert_eq!(sets_for(&week.muscles, Muscle::Quads), 3.);
    let listed: Vec<_> = training.workouts.iter().map(|w| w.id.as_str()).collect();
    assert!(["1", "2", "3"].iter().all(|id| listed.contains(id)), "{listed:?}");

    let history = screens::strength(&store, "2025-03-13").unwrap();
    assert_eq!(history.sessions_with_sets_this_week, 1);
    assert_eq!(sets_for(&history.this_week, Muscle::Quads), 3.);
    let workouts: Vec<_> = history
        .workouts
        .iter()
        .map(|w| (w.activity.id.as_str(), w.sets_recorded))
        .collect();
    assert_eq!(workouts, [("3", false), ("2", false), ("1", true)]);
    assert_eq!(history.exercises.len(), 1);
    assert_eq!(history.exercises[0].sessions.len(), 1);

    // The timed workout itself still opens, saying no sets were recorded.
    let timed = screens::workout(&store, "2").unwrap().strength.unwrap();
    assert!(!timed.sets_recorded);
    // A week of only timed sessions has no strength card.
    let timed_only = with_catalog();
    activity(&timed_only, "2", "2025-03-11T18:00:00");
    timed_only
        .put("exercise_sets", "2", "now", &fixture("exercise_sets_untracked.json"))
        .unwrap();
    assert!(screens::training(&timed_only, "2025-03-13").strength.is_none());
}

#[test]
fn history_uses_week_boundary_and_filters_sports_before_any_limit() {
    let store = with_catalog();
    store.put("profile_settings", "me", "now", &json!({"firstDayOfWeek": "sunday"})).unwrap();
    let squat = || set("SQUAT", Some("GOBLET_SQUAT"), Some(8), 20_000.);
    // Twelve Sunday-start weeks through March 13 begin on December 22.
    session(&store, "1", "2024-12-22T00:00:00", vec![squat()]);
    session(&store, "2", "2024-12-21T23:59:59", vec![squat()]);
    let item = |id, start, kind| json!({"activityId": id, "activityName": "Synthetic workout",
        "startTimeLocal": start, "activityType": {"typeKey": kind}, "duration": 1200});
    // Also retain list-only and historical activities, including a timer-only one.
    store.put("activities", "0", "now", &json!([item(3, "2024-12-23 08:00:00", "strength_training")])).unwrap();
    store.put("listed", "4", "now", &item(4, "2024-12-24T08:00:00", "strength_training")).unwrap();
    session(&store, "5", "2024-12-25T08:00:00", vec![set("UNKNOWN", None, None, 0.)]);
    activity(&store, "6", "2025-03-14T08:00:00");
    for i in 0..225 {
        let id = 1000 + i;
        store.put("listed", &id.to_string(), "now", &item(id, "2025-03-12T08:00:00", "running")).unwrap();
        // More than 100 pages also must not hide the list-only strength item.
        store.put("activities", &(i + 1).to_string(), "now", &json!([item(id, "2025-03-12T08:00:00", "running")])).unwrap();
    }
    let history = screens::strength(&store, "2025-03-13").unwrap();
    assert_eq!(history.weeks[0].week_start, "2024-12-22");
    assert_eq!(history.workouts.iter().map(|w| w.activity.id.as_str()).collect::<Vec<_>>(), ["5", "4", "3", "1"]);
    assert!(!history.workouts[0].sets_recorded);
    assert_eq!(history.exercises[0].sessions.iter().map(|s| s.activity_id.as_str()).collect::<Vec<_>>(), ["1"]);
    assert_eq!(sets_for(&history.weeks[0].muscles, Muscle::Quads), 1.);
}

#[test]
fn rep_totals_keep_missing_counts_apart_from_zero() {
    let store = with_catalog();
    let squat = |reps| set("SQUAT", Some("GOBLET_SQUAT"), reps, 20_000.);
    session(&store, "1", "2025-03-04T18:00:00", vec![squat(Some(8)), squat(None), squat(Some(6))]);
    session(&store, "2", "2025-03-05T18:00:00", vec![squat(None), squat(None)]);
    session(&store, "3", "2025-03-06T18:00:00", vec![squat(Some(0)), squat(None)]);
    let s = |id: &str| screens::workout(&store, id).unwrap().strength.unwrap();

    let partial = s("1");
    assert_eq!((partial.reps, partial.unknown_rep_sets), (Some(14), 1));
    assert_eq!((partial.missing_rep_sets, partial.zero_rep_sets), (1, 0));
    assert_eq!(partial.exercises[0].sets[1].reps, None);
    assert_eq!(partial.exercises[0].sets[1].flags, [SetFlag::MissingReps]);
    assert!(partial.sets_recorded);

    let unknown = s("2");
    assert_eq!((unknown.reps, unknown.unknown_rep_sets), (None, 2));
    assert_eq!(unknown.missing_rep_sets, 2);
    // Named sets are recorded sets even without counts.
    assert!(unknown.sets_recorded);

    let zero = s("3");
    assert_eq!((zero.reps, zero.unknown_rep_sets), (Some(0), 1));
    assert_eq!((zero.zero_rep_sets, zero.missing_rep_sets), (1, 1));
    assert_eq!(zero.exercises[0].sets[0].flags, [SetFlag::ZeroReps]);
}

/// Local-only HTTP server answering by path. No Garmin traffic.
fn mock(answer: fn(&str) -> (u16, String)) -> (Client, Arc<Mutex<Vec<String>>>) {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let seen = Arc::new(Mutex::new(vec![]));
    let log = seen.clone();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { break };
            stream
                .set_read_timeout(Some(std::time::Duration::from_secs(3)))
                .unwrap();
            let mut buf = [0u8; 8192];
            let n = stream.read(&mut buf).unwrap_or(0);
            let request = String::from_utf8_lossy(&buf[..n]).into_owned();
            let line = request.lines().next().unwrap_or_default().to_owned();
            let (status, body) = answer(&line);
            log.lock().unwrap().push(request);
            let text = format!("HTTP/1.1 {status} OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len());
            let _ = stream.write_all(text.as_bytes());
        }
    });
    let tokens = Tokens {
        access_token: "synthetic-access".into(),
        refresh_token: "synthetic-refresh".into(),
        client_id: "TEST_DI".into(),
        expires_at: Utc::now() + Duration::days(30),
    };
    (
        Client::with_urls(tokens, &url, &url, std::time::Duration::ZERO).unwrap(),
        seen,
    )
}

#[tokio::test]
async fn an_explicit_refresh_reads_one_workout_and_obeys_the_backoff() {
    fn answer(line: &str) -> (u16, String) {
        if line.contains("/exerciseSets") {
            return (200, fixture("exercise_sets.json").to_string());
        }
        (404, String::new())
    }
    let store = with_catalog();
    // Older than the 48-hour automatic window; still explicitly refreshable.
    activity(&store, "123456", "2025-03-04T18:00:00");
    activity(&store, "99", "2025-03-04T18:00:00");
    store
        .put("activity", "99", "now", &json!({"activityId": 99, "activityTypeDTO": {"typeKey": "running"},
            "summaryDTO": {"startTimeLocal": "2025-03-04 18:00:00"}}))
        .unwrap();
    let (client, seen) = mock(answer);
    // A non-strength activity never spends a request.
    let ignored = sync::refresh_strength_sets(&store, &client, "99").await;
    assert_eq!((ignored.requests, ignored.error.is_none()), (0, true));
    assert!(seen.lock().unwrap().is_empty());

    let report = sync::refresh_strength_sets(&store, &client, "123456").await;
    assert_eq!((report.requests, report.stored), (1, 1), "{report:?}");
    let lines: Vec<String> = seen.lock().unwrap().iter().map(|r| r.lines().next().unwrap_or_default().to_owned()).collect();
    assert_eq!(lines, ["GET /activity-service/activity/123456/exerciseSets HTTP/1.1"]);
    let session = screens::workout(&store, "123456").unwrap().strength.unwrap();
    assert_eq!(session.sets, 6);
    assert!(session.checked_at.is_some(), "the detail says when Garmin's sets were read");

    // Garmin's rate-limit backoff blocks every explicit read too.
    store
        .set_cursor("backoff_until", &(Utc::now() + Duration::hours(1)).timestamp().to_string())
        .unwrap();
    let blocked = sync::refresh_strength_sets(&store, &client, "123456").await;
    assert!(blocked.rate_limited && blocked.requests == 0);
    assert_eq!(seen.lock().unwrap().len(), 1, "no request past an active backoff");
}

#[tokio::test]
async fn sync_fetches_sets_for_strength_activities_then_the_catalog_and_settings() {
    fn answer(line: &str) -> (u16, String) {
        let now = Utc::now();
        let fresh = (now - Duration::hours(3)).format("%Y-%m-%d %H:%M:%S").to_string();
        let old = (now - Duration::days(5)).format("%Y-%m-%d %H:%M:%S").to_string();
        let local = |s: &str| s.replace(' ', "T");
        if line.contains("/activities/search/activities?start=0") {
            let list = json!([
                {"activityId": 11, "activityType": {"typeKey": "strength_training"},
                 "startTimeLocal": local(&fresh), "startTimeGMT": fresh},
                {"activityId": 12, "activityType": {"typeKey": "running"},
                 "startTimeLocal": local(&old), "startTimeGMT": old},
                {"activityId": 13, "activityType": {"typeKey": "strength_training"},
                 "startTimeLocal": local(&old), "startTimeGMT": old},
            ]);
            return (200, list.to_string());
        }
        if line.contains("/exerciseSets") {
            return (200, fixture("exercise_sets.json").to_string());
        }
        if line.contains("/web-data/exercises/Exercises.json") {
            return (200, fixture("exercise_catalog.json").to_string());
        }
        if line.contains("/userprofile-service/userprofile/user-settings") {
            let settings = json!({"userData": {"measurementSystem": "metric", "weight": 70000.0,
                "gender": "FEMALE", "firstDayOfWeek": {"dayName": "monday"}}});
            return (200, settings.to_string());
        }
        (404, String::new())
    }
    let store = Store::open(":memory:").unwrap();
    store
        .put("profile", "me", "now", &json!({"displayName": "Synthetic"}))
        .unwrap();
    store
        .put(
            "listed",
            "14",
            "now",
            &json!({"activityId": 14, "activityType": {"typeKey": "strength_training"},
                "startTimeLocal": "2024-01-02 09:00:00"}),
        )
        .unwrap();
    let (client, seen) = mock(answer);
    // Older strength sets come after the per-day backfill and history download.
    let report = sync::sync(&store, &client, Local::now(), 5000).await;
    assert!(report.error.is_none() && !report.rate_limited, "{report:?}");
    let requests = seen.lock().unwrap().clone();
    let sets: Vec<&String> = requests.iter().filter(|r| r.contains("/exerciseSets")).collect();
    let line = |r: &String| r.lines().next().unwrap_or_default().to_owned();
    assert_eq!(
        sets.iter().map(|r| line(r)).collect::<Vec<_>>(),
        [
            "GET /activity-service/activity/11/exerciseSets HTTP/1.1",
            "GET /activity-service/activity/13/exerciseSets HTTP/1.1",
            "GET /activity-service/activity/14/exerciseSets HTTP/1.1",
        ]
    );
    let catalog = requests
        .iter()
        .find(|r| r.contains("/web-data/exercises/Exercises.json"))
        .unwrap();
    assert!(!catalog.to_ascii_lowercase().contains("authorization"));
    assert_eq!(
        store.get("exercise_catalog", "garmin").unwrap()["SQUAT"]["exercises"]["GOBLET_SQUAT"],
        json!(["QUADS"])
    );
    assert_eq!(
        store.get("profile_settings", "me"),
        Some(json!({"measurementSystem": "metric", "timeFormat": null, "firstDayOfWeek": "monday", "gender": "FEMALE"}))
    );

    // Two days after a workout, sets are refetched at most hourly; older ones never.
    let at = (Utc::now() - Duration::hours(2)).to_rfc3339();
    for id in ["11", "13"] {
        store
            .put("exercise_sets", id, &at, &fixture("exercise_sets.json"))
            .unwrap();
    }
    seen.lock().unwrap().clear();
    sync::sync(&store, &client, Local::now(), 400).await;
    let again: Vec<String> = seen
        .lock()
        .unwrap()
        .iter()
        .filter(|r| r.contains("/exerciseSets") || r.contains("Exercises.json") || r.contains("user-settings"))
        .map(line)
        .collect();
    assert_eq!(again, ["GET /activity-service/activity/11/exerciseSets HTTP/1.1"]);
}

#[test]
fn lift_series_takes_max_and_volume_from_counted_sets_only() {
    use raring_core::view::TrendRange;
    let store = with_catalog();
    store.put("profile_settings", "me", "now", &json!({"measurementSystem": "metric"})).unwrap();
    // Squat on 1 March, then 10 March with a bodyweight set and a zero-rep set.
    session(&store, "11", "2025-03-01T18:00:00.0", vec![set("SQUAT", Some("BARBELL_BACK_SQUAT"), Some(5), 100_000.), rest(90.), set("SQUAT", Some("BARBELL_BACK_SQUAT"), Some(5), 100_000.)]);
    session(&store, "12", "2025-03-10T18:00:00.0", vec![
        set("SQUAT", Some("BARBELL_BACK_SQUAT"), Some(5), 105_000.), set("SQUAT", Some("BARBELL_BACK_SQUAT"), Some(3), 110_000.),
        set("SQUAT", Some("BARBELL_BACK_SQUAT"), Some(10), 0.), set("SQUAT", Some("BARBELL_BACK_SQUAT"), Some(0), 60_000.),
        set("BENCH_PRESS", Some("BARBELL_BENCH_PRESS"), Some(5), 70_000.)]);
    let today = chrono::NaiveDate::from_ymd_opt(2025, 3, 13).unwrap();
    let key = "SQUAT/BARBELL_BACK_SQUAT";
    let all = strength::lift_series(&store, key, TrendRange::All, today).unwrap();
    assert_eq!((all.name.as_str(), all.unit), ("Barbell Back Squat", WeightUnit::Kg));
    let rows: Vec<_> = all.points.iter().map(|p| (p.date.as_str(), p.max, p.volume, p.excluded)).collect();
    assert_eq!(rows, vec![("2025-03-01", Some(100.), Some(1000.), 0), ("2025-03-10", Some(110.), Some(855.), 2)],
        "bodyweight and zero-rep sets are left out of both and counted");
    let week = strength::lift_series(&store, key, TrendRange::Days7, today).unwrap();
    assert_eq!(week.points.len(), 1, "7 days ends today");
    assert!(strength::lift_series(&store, "SQUAT/NOT_DONE", TrendRange::All, today).is_none());
    assert!(strength::lift_series(&store, key, TrendRange::Custom, today).is_none());
}

#[test]
fn muscle_ranges_count_sets_and_flag_gaps() {
    use raring_core::view::TrendRange;
    let store = with_catalog();
    session(&store, "21", "2025-02-01T18:00:00.0", vec![set("BENCH_PRESS", Some("BARBELL_BENCH_PRESS"), Some(5), 70_000.)]);
    session(&store, "22", "2025-03-10T18:00:00.0", vec![set("SQUAT", Some("BARBELL_BACK_SQUAT"), Some(5), 100_000.), set("SQUAT", Some("BARBELL_BACK_SQUAT"), Some(5), 100_000.)]);
    let today = chrono::NaiveDate::from_ymd_opt(2025, 3, 13).unwrap();
    let week = strength::muscles(&store, TrendRange::Days7, today).unwrap();
    assert_eq!((week.from.as_str(), week.days, week.gap_at), ("2025-03-07", 7, 1.), "7 days: one set or fewer is a gap");
    assert_eq!(sets_for(&week.muscles, Muscle::Quads), 2.);
    assert_eq!(sets_for(&week.muscles, Muscle::Chest), 0., "the bench press is outside the week");
    assert!(week.gaps.contains(&Muscle::Chest) && !week.gaps.contains(&Muscle::Quads));
    let all = strength::muscles(&store, TrendRange::All, today).unwrap();
    assert_eq!((all.from.as_str(), all.days, all.gap_at), ("2025-02-01", 41, 3.), "Max starts at the first stored session");
    assert_eq!(sets_for(&all.muscles, Muscle::Chest), 1.);
    assert!(all.gaps.contains(&Muscle::Quads), "two sets in six weeks is still a gap");
    assert!(strength::muscles(&Store::open(":memory:").unwrap(), TrendRange::All, today).is_none(), "no sessions, no muscle view");
}
