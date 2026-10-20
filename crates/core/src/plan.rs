//! The weekly plan: local targets measured with Garmin's numbers, plus
//! Raring's pace, timing and completion math (docs/spec.md, Rules).
use crate::{
    screens::{activity_kind, local_time, recorded_sleep, sleep, sleep_plan},
    store::Store,
    sync::activities_listed,
    view::*,
};
use chrono::{DateTime, Datelike, Duration, Local, NaiveDate, NaiveDateTime, Weekday};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

/// The settings row holding the plan.
const KEY: &str = "plan";
/// Asleep within this many minutes of the suggested time counts as on time.
const ON_TIME_MIN: i64 = 15;
const HISTORY_WEEKS: i64 = 8;
const DAYS: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

/// Targets are kept by the week they took effect, so each past week is
/// measured against the targets it had.
#[derive(Serialize, Deserialize, Default)]
struct Stored {
    versions: BTreeMap<String, PlanTargets>,
    paused: BTreeSet<String>,
}

/// The earlier plan row's saved versions and paused weeks, for migration.
/// None when the row is malformed.
pub(crate) fn legacy(json: &str) -> Option<(BTreeMap<String, PlanTargets>, BTreeSet<String>)> {
    serde_json::from_str::<Stored>(json)
        .ok()
        .map(|s| (s.versions, s.paused))
}

fn stored_row(store: &Store) -> Stored {
    store
        .setting(KEY)
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

/// Weekly results read the local goals (migrated from the plan row), so the
/// check-in and the Plan tab agree. Goal kinds the earlier weekly plan never
/// had are left out here.
fn stored(store: &Store) -> Stored {
    let local = crate::local::load(store);
    if local.goals.versions.is_empty() && local.circumstances.pauses.is_empty() {
        return stored_row(store);
    }
    let versions = local
        .goals
        .versions
        .iter()
        .map(|v| {
            let t = &v.targets;
            (v.from.clone(), PlanTargets {
                preset: PlanPreset::Custom,
                strength_sessions: t.strength_sessions,
                intensity_min: (!t.intensity_garmin_goal).then_some(t.intensity_min),
                zone2_min: t.zone2_min,
                asleep_nights: if t.sleep_kind == SleepTimingKind::SavedPlan { t.sleep_nights } else { 0 },
                rest_days: t.rest_days.clone(),
            })
        })
        .collect();
    let first = week_starts_on(store);
    let paused = local
        .circumstances
        .pauses
        .iter()
        .filter_map(|p| {
            let since = NaiveDate::parse_from_str(&p.since, "%Y-%m-%d").ok()?;
            let until = NaiveDate::parse_from_str(p.until.as_deref()?, "%Y-%m-%d").ok()?;
            Some((since, until))
        })
        .flat_map(|(since, until)| {
            let mut weeks = vec![];
            let mut monday = week_start(since, first);
            while monday <= until {
                weeks.push(monday);
                monday += Duration::days(7);
            }
            weeks.into_iter().filter(move |w| *w >= since && *w + Duration::days(6) <= until)
        })
        .map(|w| w.to_string())
        .collect();
    Stored { versions, paused }
}

/// The first day of Garmin's intensity-minutes week, from the dates of its
/// weekly intensity rows, so plan totals match Garmin's weekly total and the
/// watch. Garmin's intensity week can differ from the account's
/// start-of-week setting. Monday until synced.
pub fn week_starts_on(store: &Store) -> Weekday {
    store
        .get("intensity_weeks", "me")
        .and_then(|v| {
            v.as_array()?
                .iter()
                .filter_map(|row| row["calendarDate"].as_str())
                .filter_map(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
                .max()
        })
        .map_or(Weekday::Mon, |d| d.weekday())
}

pub fn week_start(day: NaiveDate, first: Weekday) -> NaiveDate {
    let back = (day.weekday().num_days_from_monday() + 7 - first.num_days_from_monday()) % 7;
    day - Duration::days(back.into())
}

fn targets_for(stored: &Stored, start: NaiveDate) -> Option<PlanTargets> {
    // A version saved under the default boundary still applies to the whole
    // week when Garmin later supplies a different boundary. Earlier weeks
    // remain unconfigured because their ends precede the first version.
    let key = (start + Duration::days(6)).to_string();
    stored
        .versions
        .range(..=key)
        .next_back()
        .map(|(_, targets)| targets.clone())
}

/// Saves this week's targets and pause; later weeks keep them until changed.
pub fn set(store: &Store, today: NaiveDate, update: &PlanUpdate) -> crate::Result<()> {
    let mut stored = stored_row(store);
    let first_day = week_start(today, week_starts_on(store));
    let start = first_day.to_string();
    let mut targets = update.targets.clone();
    targets.rest_days.retain(|day| *day < 7);
    targets.rest_days.sort_unstable();
    targets.rest_days.dedup();
    stored.versions.insert(start.clone(), targets);
    if update.paused {
        stored.paused.insert(start.clone());
    } else {
        stored.paused.remove(&start);
    }
    let json = serde_json::to_string(&stored)
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
    // Goals migrated from this row follow later saves, so the two agree; a
    // refused mirror leaves the row unchanged too.
    crate::local::mirror_legacy_plan(store, first_day, update)?;
    store.set_setting(KEY, &json)?;
    Ok(())
}

fn number(v: &Value, key: &str) -> Option<f64> {
    v[key].as_f64().filter(|n| n.is_finite() && *n >= 0.)
}

/// A complete set of elapsed daily summaries, or Garmin's authoritative weekly
/// row. Never turn one synced day into a whole week's zero elsewhere.
pub fn intensity(store: &Store, start: NaiveDate, today: NaiveDate) -> Option<IntensityCard> {
    let mut moderate = 0.;
    let mut vigorous = 0.;
    let mut goal = None;
    let through = (start + Duration::days(6)).min(today);
    let mut reported = false;
    let mut complete = start <= through;
    for d in start.iter_days().take_while(|d| *d <= through) {
        let Some(day) = store.get("summary", &d.to_string()) else {
            complete = false;
            continue;
        };
        if let (Some(m), Some(v)) = (
            number(&day, "moderateIntensityMinutes"),
            number(&day, "vigorousIntensityMinutes"),
        ) {
            moderate += m;
            vigorous += v;
            reported = true;
        } else {
            complete = false;
        }
        goal = number(&day, "intensityMinutesGoal")
            .filter(|g| *g > 0.)
            .or(goal);
    }
    // Current daily summaries update on every sync. Weekly rows are refreshed
    // daily; use them to fill incomplete daily coverage, never mix the two.
    if !complete || !reported {
        let rows = store.get("intensity_weeks", "me")?;
        let row = rows
            .as_array()?
            .iter()
            .find(|r| r["calendarDate"] == start.to_string())?;
        moderate = number(row, "moderateValue")?;
        vigorous = number(row, "vigorousValue")?;
        goal = number(row, "weeklyGoal").filter(|g| *g > 0.);
    }
    Some(IntensityCard {
        start: start.to_string(),
        end: (start + Duration::days(6)).to_string(),
        total: moderate + 2. * vigorous,
        moderate,
        vigorous,
        goal,
    })
}

/// Garmin's latest weekly intensity goal on or before `day`.
pub(crate) fn garmin_goal(store: &Store, day: NaiveDate) -> Option<f64> {
    store
        .history("summary", &day.to_string(), 14)
        .iter()
        .find_map(|(_, v)| number(v, "intensityMinutesGoal").filter(|g| *g > 0.))
}

/// A shared sleep plan captured prospectively, never reconstructed by a chart.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SavedNight {
    plan: SleepPlan,
    captured_at: String,
    captured_local: String,
}

/// A prospective sleep-plan row from an export is imported only if this build
/// can read it, so a newer capture shape is never half-understood.
pub(crate) fn saved_night_ok(json: &str) -> bool {
    serde_json::from_str::<SavedNight>(json).is_ok()
}

/// Called on live sync/open only. Keep the first available plan for a night,
/// before its window and before any recorded outcome. Immutable insert also
/// protects against simultaneous foreground/background captures.
pub fn capture_sleep_plan(store: &Store, now: DateTime<Local>) -> rusqlite::Result<()> {
    let date = crate::screens::current_date(store, now.naive_local());
    let Some(plan) = sleep_plan(store, &date.to_string()).filter(|p| !p.stale) else {
        return Ok(());
    };
    let parse = |s: &str| NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.f").ok();
    let first = plan.bedtime_start_at.as_deref().unwrap_or(&plan.bedtime_at);
    let prospective = parse(first)
        .zip(parse(&plan.bedtime_at))
        .is_some_and(|(start, end)| {
            start <= end && now.naive_local() < start - Duration::minutes(ON_TIME_MIN)
        });
    let night = NaiveDate::parse_from_str(&plan.night, "%Y-%m-%d").ok();
    if !prospective
        || !night
            .is_some_and(|d| d >= now.date_naive() && d <= now.date_naive() + Duration::days(1))
        || ["sleep", "sleep_summary"].iter().any(|kind| {
            store
                .get(kind, &plan.night)
                .is_some_and(|v| recorded_sleep(&v))
        })
    {
        return Ok(());
    }
    let key = format!("sleep_plan:{}", plan.night);
    let saved = SavedNight {
        plan,
        captured_at: now.to_rfc3339(),
        captured_local: now.naive_local().format("%Y-%m-%dT%H:%M:%S").to_string(),
    };
    let json = serde_json::to_string(&saved)
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
    store.insert_setting(&key, &json)
}

/// Missing prospective evidence is excluded, never a miss. The capture must
/// precede the actual sleep start too (a later upload can reveal early sleep).
pub(crate) fn asleep_on_time(store: &Store, morning: NaiveDate) -> Option<bool> {
    let raw = store.get("sleep", &morning.to_string())?;
    let parse = |t: &str| NaiveDateTime::parse_from_str(t, "%Y-%m-%dT%H:%M:%S%.f").ok();
    let asleep = local_time(sleep(&raw), &["sleepStartTimestampLocal"]).and_then(|t| parse(&t))?;
    let saved: SavedNight =
        serde_json::from_str(&store.setting(&format!("sleep_plan:{morning}"))?).ok()?;
    if parse(&saved.captured_local)? >= asleep {
        return None;
    }
    let plan = saved.plan;
    let by = parse(&plan.bedtime_at)?;
    let from = plan
        .bedtime_start_at
        .as_deref()
        .and_then(parse)
        .unwrap_or(by);
    let slack = Duration::minutes(ON_TIME_MIN);
    Some(asleep >= from - slack && asleep <= by + slack)
}

struct Measures {
    strength: Vec<NaiveDate>,
    zone2_min: f64,
    intensity: Option<IntensityCard>,
    on_time: Vec<NaiveDate>,
    /// Nights that have ended, with and without a plan and recorded sleep.
    nights_counted: u32,
    nights_left_out: u32,
    strength_unknown: Option<String>,
    zones_unknown: Option<String>,
}

fn zone2_seconds(store: &Store, id: &str) -> Option<f64> {
    let raw = store.get("zones", id)?;
    let zones = raw.as_array()?;
    // An explicit empty array is a verified zero; null/absence isn't evidence.
    if zones.is_empty() {
        return Some(0.);
    }
    let zone = zones.iter().find(|z| z["zoneNumber"].as_u64() == Some(2))?;
    number(zone, "secsInZone").or_else(|| number(zone, "timeInZone"))
}

fn measure(store: &Store, start: NaiveDate, today: NaiveDate) -> Measures {
    let end = start + Duration::days(6);
    let mut strength = vec![];
    let mut zone2_s = 0.;
    let through = end.min(today);
    let listed = activities_listed(store, start, through);
    let mut strength_unknown = (!listed).then(|| "Workout list incomplete".to_owned());
    let mut zones_unknown = strength_unknown.clone();
    let ids = store.activities_between(&start.to_string(), &through.to_string());
    for (id, local) in &ids {
        match activity_kind(store, id) {
            Some(kind) if kind.contains("strength") => {
                if let Some(day) = local
                    .get(..10)
                    .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
                {
                    strength.push(day);
                } else {
                    strength_unknown = Some("Workout dates incomplete".to_owned());
                }
            }
            Some(_) => {}
            None => strength_unknown = Some("Workout details incomplete".to_owned()),
        }
        match zone2_seconds(store, id) {
            Some(seconds) => zone2_s += seconds,
            None => zones_unknown = Some("Heart-rate zones incomplete".to_owned()),
        }
    }
    let mut on_time = vec![];
    let (mut nights_counted, mut nights_left_out) = (0, 0);
    for i in 0..7 {
        let morning = start + Duration::days(i);
        if morning > today {
            break;
        }
        match asleep_on_time(store, morning) {
            Some(on) => {
                nights_counted += 1;
                if on {
                    on_time.push(morning);
                }
            }
            None => nights_left_out += 1,
        }
    }
    let intensity = intensity(store, start, today);
    Measures {
        strength_unknown,
        zones_unknown,
        strength,
        zone2_min: zone2_s / 60.,
        intensity,
        on_time,
        nights_counted,
        nights_left_out,
    }
}

fn day_names(days: &[NaiveDate]) -> String {
    let mut names: Vec<&str> = days
        .iter()
        .map(|d| DAYS[d.weekday().num_days_from_monday() as usize])
        .collect();
    names.dedup();
    names.join(", ")
}

fn and_list(names: &[&str]) -> String {
    match names {
        [] => String::new(),
        [one] => (*one).to_owned(),
        [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
    }
}

fn name(kind: PlanTargetKind) -> &'static str {
    match kind {
        PlanTargetKind::Strength => "Strength",
        PlanTargetKind::Intensity => "Intensity",
        PlanTargetKind::Zone2 => "Zone 2",
        PlanTargetKind::AsleepOnTime => "Asleep on time",
    }
}

/// Done at the target; behind when the share reached trails the share of the
/// week that has passed.
fn state(value: f64, target: f64, pace: f64) -> PlanState {
    if value >= target {
        PlanState::Done
    } else if value / target < pace {
        PlanState::Behind
    } else {
        PlanState::OnTrack
    }
}

fn week(
    store: &Store,
    start: NaiveDate,
    today: NaiveDate,
    targets: &PlanTargets,
    paused: bool,
) -> (PlanWeek, Measures) {
    let end = start + Duration::days(6);
    let m = measure(store, start, today);
    let current = (start..=end).contains(&today);
    let training: Vec<NaiveDate> = (0..7)
        .map(|i| start + Duration::days(i))
        .filter(|d| {
            !targets
                .rest_days
                .contains(&(d.weekday().num_days_from_monday() as u8))
        })
        .collect();
    // Today still counts as ahead: a session later today is not late.
    let passed = training.iter().filter(|d| **d < today).count();
    let day_pace = if training.is_empty() {
        0.
    } else {
        passed as f64 / training.len() as f64
    };
    let possible_nights = 7 - m.nights_left_out;
    let night_pace = if possible_nights == 0 {
        0.
    } else {
        f64::from(m.nights_counted) / f64::from(possible_nights)
    };
    let garmin_intensity = targets.intensity_min.is_none();
    let intensity_target = targets
        .intensity_min
        .map(f64::from)
        .or_else(|| m.intensity.as_ref().and_then(|i| i.goal))
        .or_else(|| garmin_goal(store, end.min(today)));
    let none_yet = |what: &str| {
        if current {
            format!("No {what} yet")
        } else {
            format!("No {what}")
        }
    };
    let mut items = vec![];
    let mut push =
        |kind, value: f64, target: f64, pace: f64, note: String, unavailable: Option<String>| {
            if target <= 0. {
                return;
            }
            let state = if unavailable.is_some() {
                PlanState::OnTrack
            } else {
                state(value, target, pace)
            };
            items.push(PlanProgress {
                kind,
                value,
                target,
                completion: (value / target).min(1.),
                pace,
                state,
                note,
                unavailable,
            });
        };
    let sessions = m.strength.len() as f64;
    push(
        PlanTargetKind::Strength,
        sessions,
        f64::from(targets.strength_sessions),
        day_pace,
        if m.strength.is_empty() {
            none_yet("sessions")
        } else {
            day_names(&m.strength)
        },
        m.strength_unknown.clone(),
    );
    let minutes_left = |value: f64, target: f64| {
        if value >= target {
            "Reached".to_owned()
        } else {
            format!("{:.0} min to go", (target - value).ceil())
        }
    };
    if targets.intensity_min != Some(0) {
        let value = m.intensity.as_ref().map_or(0., |i| i.total);
        let source = if garmin_intensity {
            "Garmin's weekly goal"
        } else {
            "Your target"
        };
        match intensity_target {
            Some(target) => push(
                PlanTargetKind::Intensity,
                value,
                target,
                day_pace,
                format!("{source} · {}", minutes_left(value, target)),
                m.intensity
                    .is_none()
                    .then(|| "Intensity summaries incomplete".to_owned()),
            ),
            None => push(
                PlanTargetKind::Intensity,
                0.,
                1.,
                day_pace,
                String::new(),
                Some("Garmin has no weekly goal".to_owned()),
            ),
        }
    }
    push(
        PlanTargetKind::Zone2,
        m.zone2_min.floor(),
        f64::from(targets.zone2_min),
        day_pace,
        minutes_left(m.zone2_min.floor(), f64::from(targets.zone2_min)),
        m.zones_unknown.clone(),
    );
    let planned = m.nights_counted > 0;
    push(
        PlanTargetKind::AsleepOnTime,
        m.on_time.len() as f64,
        f64::from(targets.asleep_nights),
        night_pace,
        if m.on_time.is_empty() {
            none_yet("nights")
        } else {
            day_names(&m.on_time)
        },
        (!planned).then(|| "Needs a saved sleep plan and recorded night".to_owned()),
    );
    let measured: Vec<&PlanProgress> = items.iter().filter(|i| i.unavailable.is_none()).collect();
    let total = measured.len() as u32;
    let reached = measured
        .iter()
        .filter(|i| i.state == PlanState::Done)
        .count() as u32;
    let on_track = measured
        .iter()
        .filter(|i| i.state != PlanState::Behind)
        .count() as u32;
    let completion = (total > 0 && !paused)
        .then(|| measured.iter().map(|i| i.completion).sum::<f64>() / f64::from(total));
    let days_left = if current {
        (end - today).num_days() as u32
    } else {
        0
    };
    let short: Vec<&str> = measured
        .iter()
        .filter(|i| i.state == PlanState::Behind)
        .map(|i| name(i.kind))
        .collect();
    let unknown = items.len() as u32 - total;
    let all_reached = total > 0 && reached == total && unknown == 0;
    let unknown_note = if unknown > 0 {
        format!("{unknown} targets awaiting data")
    } else {
        String::new()
    };
    let (headline, detail) = if paused {
        ("Paused".to_owned(), "Left out of history".to_owned())
    } else if total == 0 {
        (
            "Not enough evidence".to_owned(),
            if items.is_empty() {
                "No targets enabled".to_owned()
            } else {
                unknown_note.clone()
            },
        )
    } else if current {
        let left = match days_left {
            0 => "Last day".to_owned(),
            1 => "1 day left".to_owned(),
            n => format!("{n} days left"),
        };
        let behind = match short.len() {
            0 => String::new(),
            1 => format!(" · {} is behind", short[0]),
            2 => format!(" · {} are behind", and_list(&short)),
            n => format!(" · {n} targets are behind"),
        };
        let headline = if all_reached {
            "All reached"
        } else if unknown > 0 {
            "Partial evidence"
        } else if on_track == 0 {
            "Behind pace"
        } else {
            "On track"
        };
        let detail = left
            + &behind
            + &if unknown > 0 {
                format!(" · {unknown_note}")
            } else {
                String::new()
            };
        (headline.to_owned(), detail)
    } else {
        let detail = match short.len() {
            0 if all_reached => "Every target reached".to_owned(),
            0 => String::new(),
            1 | 2 => format!("{} came up short", and_list(&short)),
            n => format!("{n} targets came up short"),
        };
        let headline = if all_reached {
            "All reached"
        } else if unknown > 0 {
            "Partial evidence"
        } else if reached == 0 {
            "No targets reached"
        } else {
            "Partly reached"
        };
        let detail = [detail, unknown_note]
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(" · ");
        (headline.to_owned(), detail)
    };
    (
        PlanWeek {
            start: start.to_string(),
            end: end.to_string(),
            paused,
            days_left,
            items,
            total,
            on_track,
            reached,
            headline,
            detail,
            completion,
        },
        m,
    )
}

fn default_targets() -> PlanTargets {
    PlanTargets::preset(PlanPreset::Strength)
}

/// This week's plan for Today; None until targets are set.
pub fn this_week(store: &Store, date: &str) -> Option<PlanWeek> {
    let today = NaiveDate::parse_from_str(date, "%Y-%m-%d").ok()?;
    let stored = stored(store);
    let start = week_start(today, week_starts_on(store));
    let targets = targets_for(&stored, start)?;
    Some(
        week(
            store,
            start,
            today,
            &targets,
            stored.paused.contains(&start.to_string()),
        )
        .0,
    )
}

pub fn view(store: &Store, date: &str) -> PlanView {
    let today = NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap_or_default();
    let first = week_starts_on(store);
    let stored = stored(store);
    let start = week_start(today, first);
    let targets = targets_for(&stored, start).unwrap_or_else(default_targets);
    let paused = |s: NaiveDate| stored.paused.contains(&s.to_string());
    let history = (0..HISTORY_WEEKS)
        .rev()
        .map(|i| {
            let s = start - Duration::weeks(i);
            let w = targets_for(&stored, s).map(|t| week(store, s, today, &t, paused(s)).0);
            PlanHistoryWeek {
                start: s.to_string(),
                completion: w.as_ref().and_then(|w| w.completion),
                partial: w.as_ref().is_some_and(|w| w.total < w.items.len() as u32),
                paused: paused(s),
                current: i == 0,
            }
        })
        .collect();
    PlanView {
        configured: targets_for(&stored, start).is_some(),
        garmin_intensity_goal: garmin_goal(store, today),
        week_starts_on: first.num_days_from_monday() as u8,
        week: week(store, start, today, &targets, paused(start)).0,
        last_week: week(store, start - Duration::weeks(1), today, &targets, false).0,
        targets,
        history,
        presets: [
            PlanPreset::Strength,
            PlanPreset::Endurance,
            PlanPreset::Sleep,
        ]
        .map(PlanTargets::preset)
        .to_vec(),
    }
}

/// The plan week that ended most recently before `date`.
pub fn last_full_week(store: &Store, date: NaiveDate) -> NaiveDate {
    let start = week_start(date - Duration::days(1), week_starts_on(store));
    if start + Duration::days(6) >= date {
        start - Duration::weeks(1)
    } else {
        start
    }
}

fn rest_names(days: &[u8], first: Weekday) -> String {
    let mut order: Vec<u8> = days.to_vec();
    let from = first.num_days_from_monday() as u8;
    order.sort_by_key(|d| (d + 7 - from) % 7);
    let names: Vec<&str> = order.iter().map(|d| DAYS[*d as usize]).collect();
    if names.is_empty() {
        "no rest days".to_owned()
    } else {
        format!("rest {}", and_list(&names))
    }
}

/// The check-in's plan card for the week ending before `date`.
pub fn checkin(store: &Store, date: NaiveDate) -> Option<PlanCheckin> {
    let stored = stored(store);
    let start = last_full_week(store, date);
    let targets = targets_for(&stored, start)?;
    let paused = |s: NaiveDate| stored.paused.contains(&s.to_string());
    let (week, _) = week(store, start, date, &targets, paused(start));
    let before = start - Duration::weeks(1);
    let previous =
        targets_for(&stored, before).map(|t| self::week(store, before, date, &t, paused(before)).0);
    let next = targets_for(&stored, start + Duration::weeks(1)).unwrap_or_else(|| targets.clone());
    let same = PlanTargets {
        preset: targets.preset,
        ..next.clone()
    } == targets;
    Some(PlanCheckin {
        previous: week
            .items
            .iter()
            .map(|item| {
                previous
                    .as_ref()
                    .filter(|w| !w.paused)?
                    .items
                    .iter()
                    .find(|p| p.kind == item.kind && p.unavailable.is_none())
                    .map(|p| p.value)
            })
            .collect(),
        next: format!(
            "{} · {}",
            if same { "Same targets" } else { "New targets" },
            rest_names(&next.rest_days, week_starts_on(store))
        ),
        week,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weeks_start_on_garmins_first_day() {
        let saturday = NaiveDate::from_ymd_opt(2025, 3, 15).unwrap();
        assert_eq!(week_start(saturday, Weekday::Mon).to_string(), "2025-03-10");
        assert_eq!(week_start(saturday, Weekday::Sun).to_string(), "2025-03-09");
        assert_eq!(week_start(saturday, Weekday::Sat), saturday);
    }

    #[test]
    fn pace_decides_behind() {
        assert_eq!(state(3., 3., 0.5), PlanState::Done);
        assert_eq!(state(1., 3., 0.5), PlanState::Behind);
        assert_eq!(state(2., 3., 0.5), PlanState::OnTrack);
        assert_eq!(state(0., 3., 0.), PlanState::OnTrack);
    }

    #[test]
    fn lists_read_naturally() {
        assert_eq!(and_list(&["Zone 2"]), "Zone 2");
        assert_eq!(and_list(&["Zone 2", "Strength"]), "Zone 2 and Strength");
        assert_eq!(rest_names(&[0, 6], Weekday::Sun), "rest Sun and Mon");
        assert_eq!(rest_names(&[4, 0], Weekday::Mon), "rest Mon and Fri");
        assert_eq!(rest_names(&[], Weekday::Mon), "no rest days");
    }
}
