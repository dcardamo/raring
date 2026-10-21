//! Fitness Age's vigorous-minutes driver, live from Garmin's daily summaries
//! (docs/spec.md, Rules: Fitness Age window). Raring sums Garmin's figures the
//! way Garmin's driver does; it never estimates a Fitness Age.
use crate::{
    plan, screens,
    store::Store,
    view::{DayPhase, Effort, FitnessCount, FitnessPlace, PlanFitness, TodayFitness, TodayView, VigorousWindow},
};
use chrono::{Duration, NaiveDate};
use serde_json::Value;

/// Garmin's driver is the vigorous minutes of the 42 days ending on the
/// estimate's day, that day included, per week (their sum ÷ 6).
pub const WINDOW_DAYS: i64 = 42;

/// Garmin's estimate for `date`: that day's, else the latest before it.
fn estimate(store: &Store, date: &str) -> Option<Value> {
    store.history("fitness_age", date, 1).pop().map(|(_, v)| v)
}

/// Garmin's weekly vigorous-minutes target and the age it gives there, from
/// either shape of Garmin's estimate.
fn component(v: &Value) -> Option<(f64, Option<f64>)> {
    let positive = |x: &Value| x.as_f64().filter(|n| n.is_finite() && *n > 0.);
    let c = &v["components"]["vigorousMinutesAvg"];
    if c.is_object() {
        return Some((positive(&c["targetValue"])?, positive(&c["potentialAge"])));
    }
    let legacy = &v["fitnessAgeData"]["vigorousMinutes"];
    Some((positive(&legacy["target"])?, positive(&legacy["ageIfMet"])))
}

/// Garmin's daily vigorous minutes; a missing summary or figure is unknown.
fn minutes(store: &Store, day: NaiveDate) -> Option<f64> {
    store.get("summary", &day.to_string())?["vigorousIntensityMinutes"]
        .as_f64()
        .filter(|n| n.is_finite() && *n >= 0.)
}

/// The window ending on `end`, while Garmin's estimate there has a target.
pub fn window(store: &Store, end: NaiveDate) -> Option<VigorousWindow> {
    let (target, age_at_target) = component(&estimate(store, &end.to_string())?)?;
    let days: Vec<Option<f64>> = (0..WINDOW_DAYS)
        .rev()
        .map(|back| minutes(store, end - Duration::days(back)))
        .collect();
    let sum = |d: &[Option<f64>]| d.iter().copied().sum::<Option<f64>>();
    let today = *days.last()?;
    let average = sum(&days).map(|s| s / 6.);
    Some(VigorousWindow {
        end: end.to_string(),
        weeks: days.chunks(7).map(sum).collect(),
        today,
        average,
        before_today: average.zip(today).map(|(a, t)| a - t / 6.),
        target,
        age_at_target,
    })
}

/// The previous estimate when today's is lower in whole years: the Fitness
/// Age notification's comparison, read for a fall only.
fn dropped_from(store: &Store, day: &str) -> Option<f64> {
    let now = store.get("fitness_age", day)?["fitnessAge"].as_f64()?;
    let before = store
        .history("fitness_age", day, 90)
        .into_iter()
        .find_map(|(d, v)| (d != day).then(|| v["fitnessAge"].as_f64()).flatten())?;
    (now.round() < before.round()).then_some(before)
}

/// The live Today's card and lever line on the calendar day, once `lead`
/// has settled Garmin's effort call.
pub fn today(store: &Store, view: &TodayView, calendar: NaiveDate) -> Option<TodayFitness> {
    let window = window(store, calendar)?;
    let day = calendar.to_string();
    let dropped_from = dropped_from(store, &day);
    let vigorous_today = window.today.is_some_and(|m| m > 0.);
    // Wind-down and late night keep tonight's plan alone, as This week does.
    let night = matches!(view.focus.phase, DayPhase::WindDown | DayPhase::LateNight);
    let card = if night {
        None
    } else if dropped_from.is_some() {
        Some(FitnessPlace::Hero)
    } else if vigorous_today {
        Some(FitnessPlace::AfterWeek)
    } else {
        None
    };
    let lever_line = view.call == Some(Effort::Push)
        && view.focus.effort
        && !vigorous_today
        && window.average.is_some()
        && screens::drivers(store, &day, screens::body_weight(store, &day).as_ref())
            .1
            .is_some_and(|l| matches!(l.key.as_str(), "vigorousMinutes" | "vigorousDays"));
    Some(TodayFitness {
        age: estimate(store, &day).and_then(|v| v["fitnessAge"].as_f64()),
        dropped_from,
        card,
        lever_line,
        window,
    })
}

/// A recording's vigorous minutes while they're inside Garmin's window.
pub fn count(store: &Store, minutes: Option<f64>, date: NaiveDate, today: NaiveDate) -> Option<FitnessCount> {
    let minutes = minutes.filter(|m| *m > 0.)?;
    let until = date + Duration::days(WINDOW_DAYS - 1);
    if date > today || until < today {
        return None;
    }
    let (target, _) = component(&estimate(store, &today.to_string())?)?;
    let starts = plan::week_starts_on(store);
    let start = plan::week_start(date, starts);
    Some(FitnessCount {
        minutes,
        week: plan::intensity(store, start, today).map(|i| i.vigorous),
        this_week: start == plan::week_start(today, starts),
        target,
        until: until.to_string(),
    })
}

/// The current plan week's vigorous minutes against the driver's target.
pub fn plan_week(store: &Store, start: NaiveDate, today: NaiveDate) -> Option<PlanFitness> {
    if today < start || today > start + Duration::days(6) {
        return None;
    }
    let (target, _) = component(&estimate(store, &today.to_string())?)?;
    Some(PlanFitness {
        vigorous: plan::intensity(store, start, today).map(|i| i.vigorous),
        target,
    })
}
