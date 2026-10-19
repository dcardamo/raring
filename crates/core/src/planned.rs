//! Read-only, date-based projections of Garmin's cached calendar snapshots.
use crate::{
    store::Store,
    view::{CoachKind, CoachSession, CoachWeek, GarminSession, PlannedSession, PlannedState, PlannedTraining, UpcomingRace},
};
use chrono::{DateTime, NaiveDate, Utc};
use serde_json::Value;

fn day(s: &str) -> Option<NaiveDate> {
    (s.len() == 10)
        .then(|| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
        .flatten()
}
fn field_date(v: &Value, fields: &[&str]) -> Option<NaiveDate> {
    fields.iter().find_map(|key| v[*key].as_str().and_then(day))
}
fn title(v: &Value, fields: &[&str]) -> Option<String> {
    fields
        .iter()
        .find_map(|key| v[*key].as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty() && s.chars().count() <= 100)
        .map(str::to_owned)
}
fn current(store: &Store, kind: &str, now: DateTime<Utc>) -> (PlannedState, Value) {
    let Some(v) = store.get(kind, "current") else {
        return (PlannedState::Unknown, Value::Null);
    };
    let Some(at) = store
        .fetched_at(kind, "current")
        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
    else {
        return (PlannedState::Stale, v);
    };
    let age = now.signed_duration_since(at);
    if age.num_seconds() < 0 || age.num_hours() >= 12 {
        (PlannedState::Stale, v)
    } else {
        (PlannedState::Available, v)
    }
}
fn unverified_state(v: &Value) -> bool {
    // No populated status/completion shape has been verified. Never count an
    // unfamiliar entry as the next workout (or a known cancellation).
    ["status", "scheduleStatus", "workoutStatus", "eventStatus", "cancelled", "canceled"]
        .iter().any(|key| !v[*key].is_null())
}
/// Garmin's next scheduled workout or adaptive Coach session, whichever is sooner.
pub fn session(store: &Store, date: &str, now: DateTime<Utc>) -> PlannedTraining {
    let mut planned = scheduled_session(store, date, now);
    let Some(today) = day(date) else { return planned };
    let next = coach_week(store, date, &(today + chrono::Duration::days(14)).to_string(), now)
        .filter(|c| c.state == PlannedState::Available)
        .and_then(|c| c.sessions.into_iter().find(|s| !s.rest_day));
    if let Some(c) = next {
        if planned.session.as_ref().is_none_or(|s| s.date > c.date) {
            planned.session = Some(PlannedSession { name: c.name, date: c.date });
            planned.state = PlannedState::Available;
        }
    }
    planned
}
fn scheduled_session(store: &Store, date: &str, now: DateTime<Utc>) -> PlannedTraining {
    let (state, snapshot) = current(store, "scheduled_workouts", now);
    let coverage_start = field_date(&snapshot, &["startDate"]).map(|d| d.to_string());
    let coverage_end = field_date(&snapshot, &["endDate"]).map(|d| d.to_string());
    let (coach_state, plans) = current(store, "coach_plans", now);
    let coach = (coach_state == PlannedState::Available)
        .then(|| {
            plans["trainingPlanList"]
                .as_array()
                .and_then(|a| a.first())
                .map(|p| {
                    title(p, &["planName", "trainingPlanName", "name"])
                        .unwrap_or_else(|| "Name unavailable".into())
                })
        })
        .flatten();
    let unavailable = |state| PlannedTraining {
        state, session: None, coach: coach.clone(),
        coverage_start: coverage_start.clone(), coverage_end: coverage_end.clone(),
    };
    let Some(today) = day(date) else { return unavailable(PlannedState::Unknown); };
    if state != PlannedState::Available { return unavailable(state); }
    if coverage_start.as_deref().and_then(day).is_none_or(|d| d > today)
        || coverage_end.as_deref().and_then(day).is_none_or(|d| d < today)
    { return unavailable(PlannedState::Stale); }
    let Some(rows) = snapshot["entries"].as_array() else { return unavailable(PlannedState::Unknown); };
    let known = rows.iter().all(|v| {
        !unverified_state(v) && field_date(v, &["calendarDate"]).is_some()
            && title(v, &["workoutName"]).or_else(|| title(&v["workout"], &["workoutName"])).is_some()
    });
    if !known { return unavailable(PlannedState::Unknown); }
    let next = rows
        .iter()
        .filter_map(|v| {
            let d = field_date(v, &["calendarDate"])?;
            if d < today || d > today + chrono::Duration::days(14) {
                return None;
            }
            // An unfamiliar item shape is not permission to invent a workout name.
            let name =
                title(v, &["workoutName"]).or_else(|| title(&v["workout"], &["workoutName"]))?;
            Some((d, name))
        })
        .min_by(|a, b| a.0.cmp(&b.0));
    let state = if next.is_some() {
        PlannedState::Available
    } else {
        PlannedState::Empty
    };
    PlannedTraining {
        state,
        session: next.map(|(d, name)| PlannedSession {
            name,
            date: d.to_string(),
        }),
        coach,
        coverage_start,
        coverage_end,
    }
}
fn plan_id(p: &Value) -> Option<String> {
    let id = p["trainingPlanId"].as_u64().or_else(|| p["trainingPlanId"].as_str().and_then(|s| s.parse().ok()))?;
    (id > 0).then(|| id.to_string())
}
/// The id of the plan list's adaptive Garmin Coach plan, when it is scheduled or active.
pub fn adaptive_plan_id(plans: &Value) -> Option<String> {
    plans["trainingPlanList"].as_array()?.iter().find_map(|p| {
        let status = p["trainingStatus"]["statusKey"].as_str().unwrap_or_default().to_ascii_lowercase();
        (p["trainingPlanCategory"] == "FBT_ADAPTIVE" && ["scheduled", "active"].contains(&status.as_str())).then(|| plan_id(p)).flatten()
    })
}
/// A stored adaptive plan must be the requested one, with a non-empty task
/// list of dated tasks.
pub fn adaptive_snapshot_valid(v: &Value, id: &str) -> bool {
    plan_id(v).as_deref() == Some(id)
        && v["trainingPlanCategory"] == "FBT_ADAPTIVE"
        && v["taskList"].as_array().is_some_and(|tasks| {
            !tasks.is_empty() && tasks.iter().all(|t| field_date(t, &["calendarDate"]).is_some() && t["taskWorkout"].is_object())
        })
}
const COACH_EFFECTS: [(&str, &str); 3] = [("AEROBIC_BASE", "Aerobic base"), ("LACTATE_THRESHOLD", "Lactate threshold"), ("RECOVERY", "Recovery")];
/// This plan week's adaptive Garmin Coach sessions, only while the plan list
/// still names the stored plan. Garmin lists the current week only.
pub fn coach_week(store: &Store, from: &str, to: &str, now: DateTime<Utc>) -> Option<CoachWeek> {
    let (plans_state, plans) = current(store, "coach_plans", now);
    let id = adaptive_plan_id(&plans)?;
    let (state, v) = current(store, "coach_adaptive", now);
    if !adaptive_snapshot_valid(&v, &id) {
        return None;
    }
    let state = if plans_state == PlannedState::Stale { PlannedState::Stale } else { state };
    let (from, to) = (day(from)?, day(to)?);
    let mut order: std::collections::HashMap<NaiveDate, u32> = Default::default();
    let mut sessions: Vec<CoachSession> = v["taskList"]
        .as_array()?
        .iter()
        .filter_map(|t| {
            let d = field_date(t, &["calendarDate"])?;
            let n = order.entry(d).or_insert(0);
            *n += 1;
            let w = &t["taskWorkout"];
            let rest_day = w["restDay"] == true;
            let kind = (!rest_day).then(|| match w["sportType"]["sportTypeKey"].as_str() {
                Some("strength_training") => CoachKind::Strength,
                Some("other") => CoachKind::Cardio,
                _ => CoachKind::Unknown,
            });
            let effect = w["trainingEffectLabel"].as_str().and_then(|l| COACH_EFFECTS.iter().find(|(k, _)| *k == l)).map(|(_, v)| (*v).to_owned());
            (from..=to).contains(&d).then(|| CoachSession {
                id: format!("coach:{d}:{}", t["workoutOrder"].as_u64().map_or(*n, |o| o as u32)),
                date: d.to_string(),
                name: title(w, &["workoutName"]).unwrap_or_else(|| (if rest_day { "Rest day" } else { "Garmin Coach session" }).into()),
                kind,
                duration_min: w["estimatedDurationInSecs"].as_f64().filter(|s| *s > 0. && *s < 86_400.).map(|s| (s / 60.).round() as u32),
                effect,
                rest_day,
                garmin_status: (w["adaptiveCoachingWorkoutStatus"] == "NOT_COMPLETE").then(|| "Not complete".into()),
            })
        })
        .collect();
    sessions.sort_by(|a, b| a.date.cmp(&b.date).then(a.id.cmp(&b.id)));
    // Like the schedule, an old snapshot is named but never presented as current.
    if state != PlannedState::Available {
        sessions.clear();
    }
    Some(CoachWeek { name: title(&v, &["trainingPlanName", "planName", "name"]).unwrap_or_else(|| "Garmin Coach".into()), state, sessions })
}
/// Every cached scheduled workout from `from` to `to`, by date and name only.
/// Unknown or unfamiliar entries make the whole list unknown, as for the next session.
pub fn sessions(store: &Store, from: &str, to: &str, now: DateTime<Utc>) -> (PlannedState, Vec<GarminSession>) {
    let (state, snapshot) = current(store, "scheduled_workouts", now);
    if state != PlannedState::Available {
        return (state, vec![]);
    }
    let Some(rows) = snapshot["entries"].as_array() else {
        return (PlannedState::Unknown, vec![]);
    };
    let named = |v: &Value| title(v, &["workoutName"]).or_else(|| title(&v["workout"], &["workoutName"]));
    if rows.iter().any(|v| unverified_state(v) || field_date(v, &["calendarDate"]).is_none() || named(v).is_none()) {
        return (PlannedState::Unknown, vec![]);
    }
    let (Some(from), Some(to)) = (day(from), day(to)) else {
        return (PlannedState::Unknown, vec![]);
    };
    let covered = field_date(&snapshot, &["startDate"]).zip(field_date(&snapshot, &["endDate"]));
    // Days outside the cached window are not known to be empty.
    let state = match covered {
        Some((lo, hi)) if lo <= from && hi >= to => PlannedState::Available,
        Some((lo, hi)) if lo <= to && hi >= from => PlannedState::Partial,
        _ => PlannedState::Stale,
    };
    let mut list: Vec<GarminSession> = rows
        .iter()
        .filter_map(|v| {
            let d = field_date(v, &["calendarDate"])?;
            (from..=to).contains(&d).then(|| GarminSession { date: d.to_string(), name: named(v).unwrap_or_default() })
        })
        .collect();
    list.sort_by(|a, b| a.date.cmp(&b.date));
    (state, list)
}
pub fn race(store: &Store, date: &str, now: DateTime<Utc>) -> (PlannedState, Option<UpcomingRace>) {
    let (state, raw) = current(store, "upcoming_events", now);
    if state != PlannedState::Available {
        return (state, None);
    }
    let Some(today) = day(date) else {
        return (PlannedState::Unknown, None);
    };
    let Some(rows) = raw.as_array() else {
        return (PlannedState::Unknown, None);
    };
    let race_type = |v: &Value| ["eventType", "eventTypeKey"].iter().filter_map(|k| v[*k].as_str()).any(|s| s.eq_ignore_ascii_case("RACE"));
    if rows.iter().any(|v| {
        if unverified_state(v) { return true; }
        let typed = v["eventType"].is_string() || v["eventTypeKey"].is_string();
        !typed || (race_type(v) && (field_date(v, &["calendarDate", "eventDate"]).is_none()
            || title(v, &["eventName", "name"]).is_none()))
    }) { return (PlannedState::Unknown, None); }
    let next = rows
        .iter()
        .filter(|v| race_type(v))
        .filter_map(|v| {
            let d = field_date(v, &["calendarDate", "eventDate"])?;
            let name = title(v, &["eventName", "name"])?;
            (d >= today).then_some((d, name))
        })
        .min_by(|a, b| a.0.cmp(&b.0));
    match next {
        Some((d, name)) => (
            if rows.len() >= 10 { PlannedState::Partial } else { PlannedState::Available },
            Some(UpcomingRace {
                name,
                date: d.to_string(),
                days_until: (d - today).num_days() as i32,
            }),
        ),
        None if rows.len() >= 10 => (PlannedState::Partial, None),
        None => (PlannedState::Empty, None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn setup() -> (Store, DateTime<Utc>) {
        (
            Store::open(":memory:").unwrap(),
            "2026-10-20T12:00:00Z".parse().unwrap(),
        )
    }
    #[test]
    fn missing_empty_stale_and_changed_snapshot() {
        let (s, now) = setup();
        assert_eq!(session(&s, "2026-10-20", now).state, PlannedState::Unknown);
        s.put(
            "scheduled_workouts",
            "current",
            &now.to_rfc3339(),
            &json!({"startDate":"2026-10-20","endDate":"2026-11-03","entries":[]}),
        )
        .unwrap();
        assert_eq!(session(&s, "2026-10-20", now).state, PlannedState::Empty);
        assert_eq!(
            session(&s, "2026-10-21", now + chrono::Duration::hours(12)).state,
            PlannedState::Stale
        );
        s.put("scheduled_workouts", "current", &now.to_rfc3339(), &json!({"startDate":"2026-10-20","endDate":"2026-11-03","entries":[{"calendarDate":"2026-10-21","workoutName":"Easy Run"}]})).unwrap();
        assert_eq!(
            session(&s, "2026-10-20", now).session.unwrap().name,
            "Easy Run"
        );
        s.put("scheduled_workouts", "current", &now.to_rfc3339(), &json!({"startDate":"2026-10-20","endDate":"2026-11-03","entries":[{"calendarDate":"2026-10-22","workoutName":"Ride"}]})).unwrap();
        assert_eq!(
            session(&s, "2026-10-20", now).session.unwrap().date,
            "2026-10-22"
        );
        s.put("scheduled_workouts", "current", &now.to_rfc3339(), &json!({"startDate":"2026-10-20","endDate":"2026-11-03","entries":[{"calendarDate":"2026-10-22","workoutName":"Ride"},{"unexpected":"field"}]})).unwrap();
        assert_eq!(session(&s, "2026-10-20", now).state, PlannedState::Unknown);
    }
    #[test]
    fn unverified_status_is_neutral_even_without_readiness() {
        let (s, now) = setup();
        s.put("scheduled_workouts", "current", &now.to_rfc3339(), &json!({"startDate":"2026-10-20","endDate":"2026-11-03","entries":[{"calendarDate":"2026-10-20","workoutName":"Cancelled Run","status":"CANCELLED"}]})).unwrap();
        assert_eq!(session(&s, "2026-10-20", now).state, PlannedState::Unknown);
        s.put("scheduled_workouts", "current", &now.to_rfc3339(), &json!({"startDate":"2026-10-20","endDate":"2026-11-03","entries":[{"unexpected":"field"}]})).unwrap();
        assert_eq!(session(&s, "2026-10-20", now).state, PlannedState::Unknown);
        s.put("scheduled_workouts", "current", &now.to_rfc3339(), &json!({"startDate":"2026-10-20","endDate":"2026-11-03","entries":[{"calendarDate":"2026-10-21","workoutName":"Easy Run"}]})).unwrap();
        let view = crate::screens::today_at(&s, "2026-10-20", "2026-10-20T08:00:00-04:00".parse().unwrap());
        assert!(view.readiness.is_none());
        assert!(view.planned.session.is_some());
    }
    #[test]
    fn all_day_date_survives_midnight_and_offset() {
        let (s, now) = setup();
        s.put("upcoming_events", "current", &now.to_rfc3339(), &json!([{"eventType":"RACE","eventName":"City 10K","calendarDate":"2026-10-21","startTimeGMT":"2026-10-20T22:00:00Z"}])).unwrap();
        assert_eq!(race(&s, "2026-10-20", now).1.unwrap().days_until, 1);
        assert_eq!(race(&s, "2026-10-21", now).1.unwrap().date, "2026-10-21");
        assert_eq!(race(&s, "2026-10-22", now).0, PlannedState::Empty);
        s.put("upcoming_events", "current", &now.to_rfc3339(), &json!([{"eventType":"RACE","eventName":"City 10K","calendarDate":"2026-10-21"},{"unexpected":"field"}])).unwrap();
        assert_eq!(race(&s, "2026-10-20", now).0, PlannedState::Unknown);
    }
    #[test]
    fn capped_calendar_is_partial_but_empty_is_complete() {
        let (s, now) = setup();
        s.put("upcoming_events", "current", &now.to_rfc3339(), &json!([])).unwrap();
        assert_eq!(race(&s, "2026-10-20", now), (PlannedState::Empty, None));
        let ten = (0..10).map(|_| json!({"eventType":"OTHER"})).collect::<Vec<_>>();
        s.put("upcoming_events", "current", &now.to_rfc3339(), &json!(ten)).unwrap();
        assert_eq!(race(&s, "2026-10-20", now), (PlannedState::Partial, None));
        let mut with_race = (0..9).map(|_| json!({"eventType":"OTHER"})).collect::<Vec<_>>();
        with_race.push(json!({"eventType":"RACE","eventName":"Town 5K","calendarDate":"2026-10-21"}));
        s.put("upcoming_events", "current", &now.to_rfc3339(), &json!(with_race)).unwrap();
        let (state, candidate) = race(&s, "2026-10-20", now);
        assert_eq!(state, PlannedState::Partial);
        assert_eq!(candidate.unwrap().days_until, 1);
    }
}
