//! Strength sets from `/activity-service/activity/{id}/exerciseSets`.
//!
//! Raring shows Garmin's sets as recorded. Its own math is the set counts per
//! muscle, miscount flags and records described in docs/spec.md, Rules.
use crate::{
    store::Store,
    view::{
        ActivityCard, ExerciseHistory, LiftPoint, LiftSeries, MuscleRange, TrendRange, ExerciseSession, Figure, FigureChoice, Lift, Muscle, MuscleSets,
        RepBest, RepCountBest, SetFlag, StrengthActivity, StrengthExercise, StrengthSession, StrengthSet,
        StrengthView, StrengthWeek, WeekMuscles, WeightUnit,
    },
};
use chrono::{Datelike, Duration, NaiveDate, Weekday};
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, BTreeSet, HashMap};

pub const ACTIVITY_TYPE: &str = "strength_training";
/// A set at least this many times the median of the exercise's other sets is flagged.
pub const DOUBLED_RATIO: f64 = 1.8;
/// Rep counts on the "best at each rep count" row.
pub const REP_COUNTS: [u32; 5] = [1, 3, 5, 8, 10];
pub const HISTORY_WEEKS: i64 = 12;
const GRAMS_PER_POUND: f64 = 453.592_37;

#[derive(Debug, Clone)]
struct Set {
    active: bool,
    reps: Option<u32>,
    /// Garmin stores weight in grams; 0 means none.
    grams: Option<f64>,
    duration: Option<f64>,
    key: Option<String>,
}

#[derive(Debug, Clone)]
struct Session {
    id: String,
    start: String,
    sets: Vec<Set>,
}
impl Session {
    fn date(&self) -> &str {
        self.start.get(..10).unwrap_or(&self.start)
    }
    /// A session timed without lap presses has one unnamed set and no reps.
    fn recorded(&self) -> bool {
        self.sets
            .iter()
            .any(|s| s.active && (s.key.is_some() || s.reps.is_some_and(|r| r > 0)))
    }
}

/// Garmin's exercise key: "CATEGORY/NAME", or the category alone when Garmin
/// named only the category. `UNKNOWN` or a missing category is no key.
fn exercise_key(set: &Value) -> Option<String> {
    let best = set["exercises"].as_array()?.iter().max_by(|a, b| {
        let p = |v: &Value| v["probability"].as_f64().unwrap_or(0.);
        p(a).total_cmp(&p(b))
    })?;
    let category = best["category"]
        .as_str()
        .filter(|c| !c.is_empty() && *c != "UNKNOWN")?;
    Some(match best["name"].as_str().filter(|n| !n.is_empty()) {
        Some(name) => format!("{category}/{name}"),
        None => category.to_owned(),
    })
}

fn parse(id: &str, start: &str, v: &Value) -> Option<Session> {
    let sets = v["exerciseSets"].as_array()?;
    Some(Session {
        id: id.to_owned(),
        start: start.replace(' ', "T"),
        sets: sets
            .iter()
            .filter_map(|s| {
                let active = match s["setType"].as_str()? {
                    "ACTIVE" => true,
                    "REST" => false,
                    _ => return None,
                };
                Some(Set {
                    active,
                    reps: s["repetitionCount"].as_u64().map(|r| r as u32),
                    grams: s["weight"].as_f64().filter(|w| *w > 0.),
                    duration: s["duration"].as_f64(),
                    key: if active { exercise_key(s) } else { None },
                })
            })
            .collect(),
    })
}

/// Every stored strength session with Garmin's sets, oldest first.
fn sessions(store: &Store) -> Vec<Session> {
    store
        .exercise_sets()
        .into_iter()
        .filter_map(|(id, start, v)| parse(&id, &start, &v))
        .filter(|s| !s.sets.is_empty())
        .collect()
}
/// Sessions with sets; timed sessions add nothing to sets, muscles or records.
fn recorded_sessions(store: &Store) -> Vec<Session> {
    sessions(store).into_iter().filter(Session::recorded).collect()
}

/// "BARBELL_BACK_SQUAT" reads "Barbell Back Squat".
pub fn title(code: &str) -> String {
    code.split('_')
        .filter(|w| !w.is_empty())
        .map(|w| {
            let lower = w.to_ascii_lowercase();
            let mut chars = lower.chars();
            chars
                .next()
                .map(|c| c.to_ascii_uppercase().to_string() + chars.as_str())
                .unwrap_or_default()
        })
        .collect::<Vec<_>>()
        .join(" ")
}
fn display_name(key: &str) -> String {
    title(key.rsplit('/').next().unwrap_or(key))
}

/// Raring's small mapping for categories Garmin's catalog doesn't cover.
const FALLBACK: &[(&str, &[&str])] = &[
    ("BENCH_PRESS", &["CHEST"]),
    ("CALF_RAISE", &["CALVES"]),
    ("CHOP", &["OBLIQUES"]),
    ("CORE", &["ABS"]),
    ("CRUNCH", &["ABS"]),
    ("CURL", &["BICEPS"]),
    ("DEADLIFT", &["HAMSTRINGS", "GLUTES"]),
    ("FLYE", &["CHEST"]),
    ("HIP_RAISE", &["GLUTES"]),
    ("HIP_STABILITY", &["HIPS"]),
    ("HYPEREXTENSION", &["LOWER_BACK"]),
    ("LATERAL_RAISE", &["SHOULDERS"]),
    ("LEG_CURL", &["HAMSTRINGS"]),
    ("LEG_RAISE", &["ABS"]),
    ("LUNGE", &["QUADS", "GLUTES"]),
    ("PLANK", &["ABS"]),
    ("PULL_UP", &["LATS"]),
    ("PUSH_UP", &["CHEST"]),
    ("ROW", &["LATS"]),
    ("SHOULDER_PRESS", &["SHOULDERS"]),
    ("SHOULDER_STABILITY", &["SHOULDERS"]),
    ("SHRUG", &["TRAPS"]),
    ("SIT_UP", &["ABS"]),
    ("SQUAT", &["QUADS", "GLUTES"]),
    ("TRICEPS_EXTENSION", &["TRICEPS"]),
];

/// Garmin's catalog muscle names, grouped for the body map.
pub fn group(muscle: &str) -> Option<Muscle> {
    Some(match muscle {
        "CHEST" => Muscle::Chest,
        "LATS" | "TRAPS" | "LOWER_BACK" => Muscle::Back,
        "SHOULDERS" => Muscle::Shoulders,
        "BICEPS" | "TRICEPS" | "FOREARM" => Muscle::Arms,
        "ABS" | "OBLIQUES" => Muscle::Core,
        "HIPS" | "ABDUCTORS" | "ADDUCTORS" => Muscle::Hips,
        "QUADS" => Muscle::Quads,
        "HAMSTRINGS" => Muscle::Hamstrings,
        "GLUTES" => Muscle::Glutes,
        "CALVES" => Muscle::Calves,
        _ => return None,
    })
}
pub const MUSCLES: [Muscle; 10] = [
    Muscle::Chest,
    Muscle::Back,
    Muscle::Shoulders,
    Muscle::Arms,
    Muscle::Core,
    Muscle::Hips,
    Muscle::Quads,
    Muscle::Hamstrings,
    Muscle::Glutes,
    Muscle::Calves,
];

/// Keeps only the primary muscles of Garmin's exercise catalog
/// (`connect.garmin.com/web-data/exercises/Exercises.json`):
/// `{CATEGORY: {"primary": [...], "exercises": {NAME: [...]}}}`.
pub fn reduce_catalog(v: &Value) -> Option<Value> {
    let categories = v["categories"].as_object()?;
    let primary = |v: &Value| {
        Value::Array(
            v["primaryMuscles"]
                .as_array()
                .map(|a| a.iter().filter(|m| m.is_string()).cloned().collect())
                .unwrap_or_default(),
        )
    };
    let reduced: Map<String, Value> = categories
        .iter()
        .map(|(category, c)| {
            let exercises: Map<String, Value> = c["exercises"]
                .as_object()
                .map(|e| e.iter().map(|(n, x)| (n.clone(), primary(x))).collect())
                .unwrap_or_default();
            (
                category.clone(),
                json!({"primary": primary(c), "exercises": exercises}),
            )
        })
        .collect();
    (!reduced.is_empty()).then_some(Value::Object(reduced))
}

struct Catalog(Value);
impl Catalog {
    fn load(store: &Store) -> Self {
        Self(store.get("exercise_catalog", "garmin").unwrap_or(Value::Null))
    }
    /// Primary muscles: the exercise's catalog entry, then its category's,
    /// then Raring's fallback for the category. Each group counts once.
    fn muscles(&self, key: &str) -> Vec<Muscle> {
        let (category, name) = key.split_once('/').unwrap_or((key, ""));
        let names = |v: &Value| -> Vec<String> {
            v.as_array()
                .map(|a| a.iter().filter_map(|m| m.as_str().map(str::to_owned)).collect())
                .unwrap_or_default()
        };
        let mut found = names(&self.0[category]["exercises"][name]);
        if found.is_empty() {
            found = names(&self.0[category]["primary"]);
        }
        if found.is_empty() {
            found = FALLBACK
                .iter()
                .find(|(c, _)| *c == category)
                .map(|(_, m)| m.iter().map(|s| (*s).to_owned()).collect())
                .unwrap_or_default();
        }
        found
            .iter()
            .filter_map(|m| group(m))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }
}

/// Measurement system, time format, week start and profile figure from Garmin's user
/// settings, and the sleep schedule (`userSleep`, `userSleepWindows`, top level, seconds
/// after local midnight) that anchors Today's phases. Only these fields are stored.
pub fn reduce_settings(v: &Value) -> Value {
    let data = if v["userData"].is_object() { &v["userData"] } else { v };
    let sleep = &v["userSleep"];
    let windows: Option<Vec<Value>> = v["userSleepWindows"].as_array().map(|list| {
        list.iter()
            .map(|w| json!({
                "sleepWindowFrequency": w["sleepWindowFrequency"],
                "startSleepTimeSecondsFromMidnight": w["startSleepTimeSecondsFromMidnight"],
                "endSleepTimeSecondsFromMidnight": w["endSleepTimeSecondsFromMidnight"],
            }))
            .collect()
    });
    json!({
        "measurementSystem": data["measurementSystem"],
        "timeFormat": data["timeFormat"],
        "firstDayOfWeek": data["firstDayOfWeek"]["dayName"].as_str().or(data["firstDayOfWeek"].as_str()),
        "gender": data["gender"],
        "userSleep": sleep.is_object().then(|| json!({
            "sleepTime": sleep["sleepTime"],
            "wakeTime": sleep["wakeTime"],
            "defaultSleepTime": sleep["defaultSleepTime"],
            "defaultWakeTime": sleep["defaultWakeTime"],
        })),
        "userSleepWindows": windows,
    })
}
struct Prefs {
    unit: WeightUnit,
    week_start: Weekday,
    figure: Figure,
}
pub(crate) fn weight_unit(store: &Store) -> WeightUnit {
    prefs(store).unit
}
fn prefs(store: &Store) -> Prefs {
    let v = store.get("profile_settings", "me").unwrap_or(Value::Null);
    let v = reduce_settings(&v);
    Prefs {
        unit: match v["measurementSystem"].as_str() {
            Some(s) if s.starts_with("statute") => WeightUnit::Lb,
            _ => WeightUnit::Kg,
        },
        week_start: v["firstDayOfWeek"]
            .as_str()
            .and_then(|d| d.parse().ok())
            .unwrap_or(Weekday::Mon),
        // Strength's own figure picker overrides Garmin's profile gender.
        figure: match store.settings().body_figure {
            FigureChoice::Female => Figure::Female,
            FigureChoice::Male => Figure::Male,
            FigureChoice::Profile => match v["gender"].as_str() {
                Some("MALE") => Figure::Male,
                _ => Figure::Female,
            },
        },
    }
}
pub(crate) fn weight(grams: Option<f64>, unit: WeightUnit) -> Option<f64> {
    grams.map(|g| {
        let w = match unit {
            WeightUnit::Kg => g / 1000.,
            WeightUnit::Lb => g / GRAMS_PER_POUND,
        };
        (w * 10.).round() / 10.
    })
}
fn week_start(date: NaiveDate, start: Weekday) -> NaiveDate {
    let back = (7 + date.weekday().num_days_from_monday() - start.num_days_from_monday()) % 7;
    date - Duration::days(back as i64)
}

fn median(values: &mut [u32]) -> Option<f64> {
    values.sort_unstable();
    let n = values.len();
    match n {
        0 => None,
        _ if n % 2 == 1 => Some(values[n / 2] as f64),
        _ => Some((values[n / 2 - 1] + values[n / 2]) as f64 / 2.),
    }
}

/// Flags for every set, by index into `session.sets` (rests get none).
fn flags(session: &Session) -> Vec<Vec<SetFlag>> {
    let mut out = vec![vec![]; session.sets.len()];
    for (i, set) in session.sets.iter().enumerate().filter(|(_, s)| s.active) {
        if set.key.is_none() {
            out[i].push(SetFlag::Unnamed);
        }
        let Some(reps) = set.reps else {
            out[i].push(SetFlag::MissingReps);
            continue;
        };
        if reps == 0 {
            out[i].push(SetFlag::ZeroReps);
        } else if set.key.is_some() {
            let mut others: Vec<u32> = session
                .sets
                .iter()
                .enumerate()
                .filter(|(j, s)| *j != i && s.active && s.key == set.key)
                .filter_map(|(_, s)| s.reps.filter(|r| *r > 0))
                .collect();
            if median(&mut others).is_some_and(|m| reps as f64 >= DOUBLED_RATIO * m) {
                out[i].push(SetFlag::Doubled);
            }
        }
    }
    out
}
/// Sets that can set a record or a best: named, weighted, with reps, and not
/// flagged as doubled.
fn counts_for_records(set: &Set, flags: &[SetFlag]) -> bool {
    set.key.is_some()
        && set.grams.is_some()
        && set.reps.is_some_and(|r| r > 0)
        && !flags.contains(&SetFlag::Doubled)
}
/// Bodyweight sets (Garmin weight 0 or null) have no weight to compare: the
/// same rules pick the most reps in one set instead.
fn counts_for_reps(set: &Set, flags: &[SetFlag]) -> bool {
    set.key.is_some()
        && set.grams.is_none()
        && set.reps.is_some_and(|r| r > 0)
        && !flags.contains(&SetFlag::Doubled)
}

/// Every session's flags and record sets, computed oldest first.
struct Analysed {
    session: Session,
    flags: Vec<Vec<SetFlag>>,
    records: Vec<bool>,
}
fn analyse(sessions: Vec<Session>) -> Vec<Analysed> {
    let mut prior: HashMap<String, Vec<(f64, u32)>> = HashMap::new();
    sessions
        .into_iter()
        .map(|session| {
            let flags = flags(&session);
            let mut records = vec![false; session.sets.len()];
            let mut seen: HashMap<String, Vec<(f64, u32)>> = HashMap::new();
            for (i, set) in session.sets.iter().enumerate() {
                if !set.active || !counts_for_records(set, &flags[i]) {
                    continue;
                }
                let (Some(key), Some(grams), Some(reps)) = (&set.key, set.grams, set.reps) else {
                    continue;
                };
                let before = prior.get(key).map(Vec::as_slice).unwrap_or(&[]);
                let now = seen.entry(key.clone()).or_default();
                let beaten = before
                    .iter()
                    .chain(now.iter())
                    .any(|(g, r)| *g >= grams && *r >= reps);
                records[i] = !before.is_empty() && !beaten;
                now.push((grams, reps));
            }
            for (key, lifts) in seen {
                prior.entry(key).or_default().extend(lifts);
            }
            Analysed {
                session,
                flags,
                records,
            }
        })
        .collect()
}

fn muscle_list(counts: &BTreeMap<Muscle, f64>) -> Vec<MuscleSets> {
    MUSCLES
        .iter()
        .map(|m| MuscleSets {
            muscle: *m,
            sets: counts.get(m).copied().unwrap_or(0.),
        })
        .collect()
}
/// Each active set with a known exercise counts once for each of its
/// muscle groups.
fn add_muscles(counts: &mut BTreeMap<Muscle, f64>, session: &Session, catalog: &Catalog) {
    for set in session.sets.iter().filter(|s| s.active) {
        if let Some(key) = &set.key {
            for m in catalog.muscles(key) {
                *counts.entry(m).or_default() += 1.;
            }
        }
    }
}

/// The detail of one strength activity, when Garmin has sets for it.
pub fn session(store: &Store, id: &str) -> Option<StrengthSession> {
    let prefs = prefs(store);
    let catalog = Catalog::load(store);
    let all = analyse(sessions(store));
    let index = all.iter().position(|a| a.session.id == id)?;
    let Analysed {
        session,
        flags,
        records,
    } = &all[index];
    let active: Vec<usize> = (0..session.sets.len())
        .filter(|i| session.sets[*i].active)
        .collect();
    let mut order: Vec<Option<String>> = vec![];
    for i in &active {
        let key = &session.sets[*i].key;
        if !order.contains(key) {
            order.push(key.clone());
        }
    }
    let exercises = order
        .into_iter()
        .map(|key| {
            let mine: Vec<usize> = active
                .iter()
                .copied()
                .filter(|i| session.sets[*i].key == key)
                .collect();
            let last = key.as_ref().and_then(|k| {
                all[..index].iter().rev().find_map(|a| {
                    let sets: Vec<Lift> = a
                        .session
                        .sets
                        .iter()
                        .filter(|s| s.active && s.key.as_ref() == Some(k))
                        .map(|s| Lift {
                            weight: weight(s.grams, prefs.unit),
                            reps: s.reps,
                        })
                        .collect();
                    (!sets.is_empty()).then(|| (a.session.date().to_owned(), sets))
                })
            });
            let record = mine
                .iter()
                .filter(|i| records[**i])
                .map(|i| &session.sets[*i])
                .max_by(|a, b| {
                    a.grams
                        .unwrap_or(0.)
                        .total_cmp(&b.grams.unwrap_or(0.))
                        .then(a.reps.cmp(&b.reps))
                })
                .map(|s| Lift {
                    weight: weight(s.grams, prefs.unit),
                    reps: s.reps,
                });
            let (last_date, last_sets) = last.map_or((None, vec![]), |(d, s)| (Some(d), s));
            StrengthExercise {
                name: key.as_deref().map(display_name),
                muscles: key.as_deref().map(|k| catalog.muscles(k)).unwrap_or_default(),
                key,
                sets: mine
                    .iter()
                    .map(|i| {
                        let s = &session.sets[*i];
                        StrengthSet {
                            reps: s.reps,
                            weight: weight(s.grams, prefs.unit),
                            duration_s: s.duration,
                            flags: flags[*i].clone(),
                            record: records[*i],
                        }
                    })
                    .collect(),
                last_date,
                last_sets,
                record,
            }
        })
        .collect();
    let count = |flag: SetFlag| active.iter().filter(|i| flags[**i].contains(&flag)).count() as u32;
    let mut muscles = BTreeMap::new();
    add_muscles(&mut muscles, session, &catalog);
    Some(StrengthSession {
        unit: prefs.unit,
        figure: prefs.figure,
        // Synthetic demo/test markers are not a real Garmin check time.
        checked_at: store
            .fetched_at("exercise_sets", id)
            .filter(|t| chrono::DateTime::parse_from_rfc3339(t).is_ok()),
        sets: active.len() as u32,
        reps: active
            .iter()
            .filter_map(|i| session.sets[*i].reps)
            .reduce(|a, b| a + b),
        unknown_rep_sets: active
            .iter()
            .filter(|i| session.sets[**i].reps.is_none())
            .count() as u32,
        rest_s: session
            .sets
            .iter()
            .filter(|s| !s.active)
            .filter_map(|s| s.duration)
            .sum(),
        records: records.iter().filter(|r| **r).count() as u32,
        sets_recorded: session.recorded(),
        zero_rep_sets: count(SetFlag::ZeroReps),
        missing_rep_sets: count(SetFlag::MissingReps),
        doubled_sets: count(SetFlag::Doubled),
        unnamed_sets: count(SetFlag::Unnamed),
        muscles: muscle_list(&muscles),
        exercises,
    })
}

struct Weeks {
    this_week: NaiveDate,
    first: NaiveDate,
}
fn weeks(date: &str, start: Weekday) -> Option<Weeks> {
    let today = NaiveDate::parse_from_str(date, "%Y-%m-%d").ok()?;
    let this_week = week_start(today, start);
    Some(Weeks {
        this_week,
        first: this_week - Duration::weeks(HISTORY_WEEKS - 1),
    })
}
/// The activity list and exercise history must start at the same week boundary.
pub(crate) fn history_start(store: &Store, date: &str) -> Option<NaiveDate> {
    Some(weeks(date, prefs(store).week_start)?.first)
}
fn in_range(a: &Analysed, from: NaiveDate, to: NaiveDate) -> bool {
    NaiveDate::parse_from_str(a.session.date(), "%Y-%m-%d").is_ok_and(|d| d >= from && d < to)
}

/// The Training card, when any strength session in the last 12 weeks has
/// sets. `activity_starts` are the local starts of Garmin's strength
/// activities, with or without sets.
pub fn week(store: &Store, date: &str, activity_starts: &[String]) -> Option<StrengthWeek> {
    let prefs = prefs(store);
    let range = weeks(date, prefs.week_start)?;
    let end = range.this_week + Duration::weeks(1);
    let all = analyse(recorded_sessions(store));
    if !all.iter().any(|a| in_range(a, range.first, end)) {
        return None;
    }
    let catalog = Catalog::load(store);
    let current: Vec<&Analysed> = all
        .iter()
        .filter(|a| in_range(a, range.this_week, end))
        .collect();
    let mut muscles = BTreeMap::new();
    for a in &current {
        add_muscles(&mut muscles, &a.session, &catalog);
    }
    Some(StrengthWeek {
        figure: prefs.figure,
        week_start: range.this_week.to_string(),
        sets: current
            .iter()
            .map(|a| a.session.sets.iter().filter(|s| s.active).count() as u32)
            .sum(),
        sessions_with_sets: current.len() as u32,
        strength_activities: activity_starts
            .iter()
            .filter_map(|s| NaiveDate::parse_from_str(s.get(..10)?, "%Y-%m-%d").ok())
            .filter(|d| *d >= range.this_week && *d < end)
            .count() as u32,
        records: current
            .iter()
            .map(|a| a.records.iter().filter(|r| **r).count() as u32)
            .sum(),
        muscles: muscle_list(&muscles),
    })
}

/// Strength history for the 12 weeks ending with `date`'s week.
/// `workouts` are Garmin's strength activities in the 12 weeks, newest first.
pub fn history(store: &Store, date: &str, workouts: Vec<ActivityCard>) -> Option<StrengthView> {
    let prefs = prefs(store);
    let range = weeks(date, prefs.week_start)?;
    let catalog = Catalog::load(store);
    let all = analyse(recorded_sessions(store));
    if all.is_empty() {
        return None;
    }
    let with_sets: BTreeSet<&str> = all.iter().map(|a| a.session.id.as_str()).collect();
    let week_counts = |from: NaiveDate| {
        let mut counts = BTreeMap::new();
        for a in all
            .iter()
            .filter(|a| in_range(a, from, from + Duration::weeks(1)))
        {
            add_muscles(&mut counts, &a.session, &catalog);
        }
        counts
    };
    let weeks_list: Vec<WeekMuscles> = (0..HISTORY_WEEKS)
        .rev()
        .map(|n| {
            let start = range.this_week - Duration::weeks(n);
            WeekMuscles {
                week_start: start.to_string(),
                muscles: muscle_list(&week_counts(start)),
            }
        })
        .collect();
    let mut average = BTreeMap::new();
    let mut last_4w = week_counts(range.this_week);
    for n in 1..=4 {
        for (m, v) in week_counts(range.this_week - Duration::weeks(n)) {
            *average.entry(m).or_default() += v / 4.;
            if n < 4 {
                *last_4w.entry(m).or_default() += v;
            }
        }
    }
    let end = range.this_week + Duration::weeks(1);
    let mut by_key: BTreeMap<String, Vec<&Analysed>> = BTreeMap::new();
    for a in &all {
        let keys: BTreeSet<&String> = a.session.sets.iter().filter_map(|s| s.key.as_ref()).collect();
        for key in keys {
            by_key.entry(key.clone()).or_default().push(a);
        }
    }
    let mut exercises: Vec<ExerciseHistory> = by_key
        .into_iter()
        .filter_map(|(key, list)| {
            let recent: Vec<&Analysed> = list
                .iter()
                .copied()
                .filter(|a| in_range(a, range.first, end))
                .collect();
            if recent.is_empty() {
                return None;
            }
            let sessions = recent
                .iter()
                .rev()
                .map(|a| {
                    let mine: Vec<usize> = (0..a.session.sets.len())
                        .filter(|i| {
                            let s = &a.session.sets[*i];
                            s.active && s.key.as_ref() == Some(&key)
                        })
                        .collect();
                    let top = mine
                        .iter()
                        .filter(|i| counts_for_records(&a.session.sets[**i], &a.flags[**i]))
                        .map(|i| &a.session.sets[*i])
                        .max_by(|x, y| {
                            x.grams
                                .unwrap_or(0.)
                                .total_cmp(&y.grams.unwrap_or(0.))
                                .then(x.reps.cmp(&y.reps))
                        });
                    ExerciseSession {
                        activity_id: a.session.id.clone(),
                        date: a.session.date().to_owned(),
                        top: top.map(|s| Lift {
                            weight: weight(s.grams, prefs.unit),
                            reps: s.reps,
                        }),
                        reps: mine.iter().map(|i| a.session.sets[*i].reps).collect(),
                        record: mine.iter().any(|i| a.records[*i]),
                    }
                })
                .collect();
            let bests = REP_COUNTS
                .iter()
                .map(|n| {
                    let mut best: Option<(f64, &str)> = None;
                    for a in &list {
                        for (i, s) in a.session.sets.iter().enumerate() {
                            if s.active
                                && s.key.as_ref() == Some(&key)
                                && counts_for_records(s, &a.flags[i])
                                && s.reps.is_some_and(|r| r >= *n)
                            {
                                let g = s.grams.unwrap_or(0.);
                                if best.is_none_or(|(b, _)| g > b) {
                                    best = Some((g, a.session.date()));
                                }
                            }
                        }
                    }
                    RepBest {
                        reps: *n,
                        weight: weight(best.map(|b| b.0), prefs.unit),
                        date: best.map(|b| b.1.to_owned()),
                    }
                })
                .collect();
            let mut reps_best: Option<(u32, &str)> = None;
            for a in &list {
                for (i, s) in a.session.sets.iter().enumerate() {
                    if s.active && s.key.as_ref() == Some(&key) && counts_for_reps(s, &a.flags[i]) {
                        let reps = s.reps.unwrap_or(0);
                        // A tie keeps the earliest set.
                        if reps_best.is_none_or(|(r, _)| reps > r) {
                            reps_best = Some((reps, a.session.date()));
                        }
                    }
                }
            }
            let reps_best = reps_best.map(|(reps, date)| RepCountBest { reps, date: date.to_owned() });
            Some(ExerciseHistory {
                name: display_name(&key),
                muscles: catalog.muscles(&key),
                key,
                reps_best,
                sessions,
                bests,
            })
        })
        .collect();
    exercises.sort_by(|a, b| {
        let last = |e: &ExerciseHistory| e.sessions.first().map(|s| s.date.clone());
        last(b)
            .cmp(&last(a))
            .then(b.sessions.len().cmp(&a.sessions.len()))
            .then(a.name.cmp(&b.name))
    });
    Some(StrengthView {
        unit: prefs.unit,
        figure: prefs.figure,
        date: date.to_owned(),
        week_start: range.this_week.to_string(),
        this_week: muscle_list(&week_counts(range.this_week)),
        average_4w: muscle_list(&average),
        last_4w: muscle_list(&last_4w),
        weeks: weeks_list,
        sessions_with_sets_this_week: all
            .iter()
            .filter(|a| in_range(a, range.this_week, end))
            .count() as u32,
        exercises,
        workouts: workouts
            .into_iter()
            .map(|activity| StrengthActivity {
                sets_recorded: with_sets.contains(activity.id.as_str()),
                activity,
            })
            .collect(),
    })
}

/// Strength activity ids, for activity list items Garmin types as strength.
/// Sets per muscle group in a range ending today, and the groups with few
/// or none: at most one set per fortnight of the range.
pub fn muscles(store: &Store, range: TrendRange, today: NaiveDate) -> Option<MuscleRange> {
    let all = analyse(recorded_sessions(store));
    let first = all.iter().filter_map(|a| NaiveDate::parse_from_str(a.session.date(), "%Y-%m-%d").ok()).min()?;
    let days: i64 = match range {
        TrendRange::Days7 => 7,
        TrendRange::Days14 => 14,
        TrendRange::Days30 => 30,
        TrendRange::Days90 => 90,
        TrendRange::Year => 365,
        TrendRange::All => (today - first).num_days().max(0) + 1,
        TrendRange::Custom => return None,
    };
    let from = today - Duration::days(days - 1);
    let catalog = Catalog::load(store);
    let mut counts = BTreeMap::new();
    for a in all.iter().filter(|a| in_range(a, from, today + Duration::days(1))) {
        add_muscles(&mut counts, &a.session, &catalog);
    }
    let muscles = muscle_list(&counts);
    let gap_at = (days as f64 / 14.).round();
    let mut gaps: Vec<&MuscleSets> = muscles.iter().filter(|m| m.sets <= gap_at).collect();
    gaps.sort_by(|a, b| a.sets.total_cmp(&b.sets));
    Some(MuscleRange {
        range,
        from: from.to_string(),
        to: today.to_string(),
        days: days as u32,
        figure: prefs(store).figure,
        gaps: gaps.iter().map(|m| m.muscle).collect(),
        muscles,
        gap_at,
    })
}

/// One exercise's sessions in a range: heaviest counted set and volume.
pub fn lift_series(store: &Store, key: &str, range: TrendRange, today: NaiveDate) -> Option<LiftSeries> {
    let days = match range {
        TrendRange::Days7 => Some(7),
        TrendRange::Days14 => Some(14),
        TrendRange::Days30 => Some(30),
        TrendRange::Days90 => Some(90),
        TrendRange::Year => Some(365),
        TrendRange::All => None,
        TrendRange::Custom => return None,
    };
    let from = days.map(|d| (today - Duration::days(d - 1)).to_string());
    let unit = prefs(store).unit;
    let today_s = today.to_string();
    let all = analyse(recorded_sessions(store));
    if !all.iter().any(|a| a.session.sets.iter().any(|s| s.key.as_deref() == Some(key))) {
        return None;
    }
    let points: Vec<LiftPoint> = all
        .iter()
        .filter(|a| a.session.date() <= today_s.as_str() && from.as_deref().is_none_or(|f| a.session.date() >= f))
        .filter_map(|a| {
            let mine: Vec<usize> = (0..a.session.sets.len())
                .filter(|i| a.session.sets[*i].active && a.session.sets[*i].key.as_deref() == Some(key))
                .collect();
            if mine.is_empty() {
                return None;
            }
            let counted: Vec<&Set> = mine
                .iter()
                .filter(|i| counts_for_records(&a.session.sets[**i], &a.flags[**i]))
                .map(|i| &a.session.sets[*i])
                .collect();
            let grams = |s: &Set| s.grams.unwrap_or(0.);
            Some(LiftPoint {
                date: a.session.date().to_owned(),
                activity_id: a.session.id.clone(),
                max: counted.iter().map(|s| grams(s)).max_by(f64::total_cmp).and_then(|g| weight(Some(g), unit)),
                volume: (!counted.is_empty())
                    .then(|| counted.iter().map(|s| grams(s) * s.reps.unwrap_or(0) as f64).sum::<f64>())
                    .and_then(|g| weight(Some(g), unit)),
                excluded: (mine.len() - counted.len()) as u32,
                most_reps: mine
                    .iter()
                    .filter(|i| counts_for_reps(&a.session.sets[**i], &a.flags[**i]))
                    .filter_map(|i| a.session.sets[*i].reps)
                    .max(),
            })
        })
        .collect();
    let from = from
        .or_else(|| points.first().map(|p| p.date.clone()))
        .unwrap_or_else(|| today_s.clone());
    Some(LiftSeries { key: key.to_owned(), name: display_name(key), unit, range, from, to: today_s, points })
}

pub fn is_strength(item: &Value) -> bool {
    item["activityType"]["typeKey"] == ACTIVITY_TYPE
        || item["activityTypeDTO"]["typeKey"] == ACTIVITY_TYPE
}

/// One exercise's heaviest set for at least `reps` reps over all stored
/// sessions, by the Records and bests rule (named, weighted, not flagged).
pub(crate) struct AllTimeBest {
    pub key: String,
    pub name: String,
    pub reps: u32,
    pub grams: f64,
    pub date: String,
    pub activity: String,
}

pub(crate) fn all_time_bests(store: &Store, through: &str) -> Vec<AllTimeBest> {
    let all = analyse(recorded_sessions(store).into_iter().filter(|s| s.date() <= through).collect());
    let mut best: BTreeMap<(String, u32), (f64, String, String)> = BTreeMap::new();
    for a in &all {
        for (i, s) in a.session.sets.iter().enumerate() {
            if !s.active || !counts_for_records(s, &a.flags[i]) {
                continue;
            }
            let (Some(key), Some(grams), Some(reps)) = (&s.key, s.grams, s.reps) else { continue };
            for n in [1, 3, 5, 8, 10] {
                if reps >= n {
                    let e = best.entry((key.clone(), n)).or_insert((0., String::new(), String::new()));
                    // Ties keep the earliest set.
                    if grams > e.0 {
                        *e = (grams, a.session.date().to_owned(), a.session.id.clone());
                    }
                }
            }
        }
    }
    best.into_iter()
        .map(|((key, reps), (grams, date, activity))| {
            AllTimeBest { name: display_name(&key), key, reps, grams, date, activity }
        })
        .collect()
}


/// One exercise's most reps in one bodyweight set over all stored sessions,
/// by the bodyweight best rule (named, unweighted, not flagged).
pub(crate) struct AllTimeRepsBest {
    pub key: String,
    pub name: String,
    pub reps: u32,
    pub date: String,
    pub activity: String,
}

pub(crate) fn all_time_reps_bests(store: &Store, through: &str) -> Vec<AllTimeRepsBest> {
    let all = analyse(recorded_sessions(store).into_iter().filter(|s| s.date() <= through).collect());
    let mut best: BTreeMap<String, (u32, String, String)> = BTreeMap::new();
    for a in &all {
        for (i, s) in a.session.sets.iter().enumerate() {
            if !s.active || !counts_for_reps(s, &a.flags[i]) {
                continue;
            }
            let (Some(key), Some(reps)) = (&s.key, s.reps) else { continue };
            let e = best.entry(key.clone()).or_insert((0, String::new(), String::new()));
            // Ties keep the earliest set.
            if reps > e.0 {
                *e = (reps, a.session.date().to_owned(), a.session.id.clone());
            }
        }
    }
    best.into_iter()
        .map(|(key, (reps, date, activity))| AllTimeRepsBest { name: display_name(&key), key, reps, date, activity })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn titles_and_weeks() {
        assert_eq!(title("BARBELL_BACK_SQUAT"), "Barbell Back Squat");
        assert_eq!(display_name("SQUAT/LEG_PRESS"), "Leg Press");
        assert_eq!(display_name("ROW"), "Row");
        let d = NaiveDate::from_ymd_opt(2026, 10, 15).unwrap(); // a Thursday
        assert_eq!(week_start(d, Weekday::Mon).to_string(), "2026-10-12");
        assert_eq!(week_start(d, Weekday::Sun).to_string(), "2026-10-11");
        assert_eq!(week_start(d, Weekday::Thu).to_string(), "2026-10-15");
        assert_eq!(week_start(d, Weekday::Sat).to_string(), "2026-10-10");
    }

    #[test]
    fn weights_follow_units() {
        assert_eq!(weight(Some(61_235.), WeightUnit::Kg), Some(61.2));
        assert_eq!(weight(Some(61_235.), WeightUnit::Lb), Some(135.));
        assert_eq!(weight(None, WeightUnit::Kg), None);
    }
}
