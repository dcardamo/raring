//! The plan board (docs/spec.md, Rules: Plan matching, Goal ledger, Rings,
//! Backlog): local goals and sessions measured against Garmin's recordings.
use crate::{
    local::{self, LocalRecords},
    plan, planned, screens,
    store::Store,
    sync::activities_listed,
    view::*,
};
use chrono::{DateTime, Duration, NaiveDate, NaiveDateTime, Utc};
use serde_json::Value;
use std::collections::{BTreeMap, HashSet};

/// Ring order, from the outside. Sleep Score is a reading, never a ring.
pub const RING_KINDS: [GoalKind; 5] = [
    GoalKind::Strength,
    GoalKind::Zone2,
    GoalKind::HighIntensity,
    GoalKind::Intensity,
    GoalKind::SleepTiming,
];
const ENDURANCE: [Sport; 4] = [Sport::Running, Sport::Cycling, Sport::Swimming, Sport::Hiking];
/// Asleep within this many minutes after the chosen time still counts.
const SLEEP_SLACK_MIN: i64 = 15;

fn number(v: &Value, key: &str) -> Option<f64> {
    v[key].as_f64().filter(|n| n.is_finite() && *n >= 0.)
}

/// Seconds in the given Garmin zones; an explicit empty list is a verified zero.
fn zone_seconds(store: &Store, id: &str, zones: &[u64]) -> Option<f64> {
    let raw = store.get("zones", id)?;
    let list = raw.as_array()?;
    if list.is_empty() {
        return Some(0.);
    }
    zones.iter().try_fold(0., |sum, n| {
        let z = list.iter().find(|z| z["zoneNumber"].as_u64() == Some(*n))?;
        Some(sum + number(z, "secsInZone").or_else(|| number(z, "timeInZone"))?)
    })
}

/// A recording with the seconds the zone goals sum before rounding.
struct Counted {
    rec: Recording,
    zone2_s: Option<f64>,
    high_s: Option<f64>,
    /// Garmin's type is known.
    typed: bool,
}

fn counted(store: &Store, id: &str, start_local: &str) -> Option<Counted> {
    let card = screens::activity(store, id)?;
    let kind = screens::activity_kind(store, id).unwrap_or_default();
    let date = start_local.get(..10)?.to_owned();
    let zone2_s = zone_seconds(store, id, &[2]);
    let high_s = zone_seconds(store, id, &[4, 5]);
    Some(Counted {
        rec: Recording {
            id: id.to_owned(),
            date,
            start_local: card.start_local,
            name: card.name,
            icon: card.icon,
            sport: local::sport_of(&kind),
            type_key: kind.clone(),
            duration_min: card.duration_s / 60.,
            zone2_min: zone2_s.map(|s| (s / 60.).floor()),
            high_intensity_min: high_s.map(|s| (s / 60.).floor()),
            intensity_min: card.intensity_min,
        },
        zone2_s,
        high_s,
        typed: !kind.is_empty(),
    })
}

fn recordings(store: &Store, from: NaiveDate, to: NaiveDate) -> Vec<Counted> {
    if to < from {
        return vec![];
    }
    let mut list: Vec<Counted> = store
        .activities_between(&from.to_string(), &to.to_string())
        .iter()
        .filter_map(|(id, start)| counted(store, id, start))
        .collect();
    list.sort_by(|a, b| a.rec.start_local.cmp(&b.rec.start_local));
    list
}

/// Marks a Garmin Coach cardio session, which has no sport until it's done.
const COACH_CARDIO: &str = "garmin_coach_cardio";
const CARDIO: [Sport; 5] = [Sport::Running, Sport::Cycling, Sport::Swimming, Sport::Walking, Sport::Hiking];

fn coach_intention(c: &CoachSession) -> Intention {
    let strength = c.kind == Some(CoachKind::Strength);
    Intention {
        id: c.id.clone(),
        date: c.date.clone(),
        time: None,
        sport: if strength { Sport::Strength } else { Sport::Other },
        type_key: (c.kind == Some(CoachKind::Cardio)).then(|| COACH_CARDIO.to_owned()),
        name: Some(c.name.clone()),
        duration_min: c.duration_min,
        allocation: Allocation { strength_sessions: u32::from(strength), ..Allocation::default() },
        kind: IntentionKind::Workout,
        garmin_id: Some(c.id.clone()),
        status: IntentionStatus::Open,
        history: vec![],
    }
}

fn same_sport(i: &Intention, r: &Recording) -> bool {
    if i.type_key.as_deref() == Some(COACH_CARDIO) {
        return CARDIO.contains(&r.sport);
    }
    i.sport == r.sport && (i.sport != Sport::Other || i.type_key.as_deref() == Some(r.type_key.as_str()))
}

pub(crate) struct Matched {
    pub state: ItemState,
    pub recording: Option<String>,
    pub by: Option<MatchKind>,
    pub candidates: Vec<String>,
}

/// Explicit links first, then unambiguous same-day, same-sport pairs; Raring
/// asks only when it can't tell. `unknown` marks passed days whose activity
/// list is incomplete.
pub(crate) fn match_items(
    items: &[Intention],
    recs: &[Recording],
    links: &local::Links,
    wind_downs: &[WindDownReport],
    today: NaiveDate,
    linked_elsewhere: impl Fn(&str) -> bool,
    unknown: impl Fn(&str) -> bool,
) -> Vec<Matched> {
    let today_s = today.to_string();
    let mut out: Vec<Option<Matched>> = items.iter().map(|_| None).collect();
    let mut taken: HashSet<&str> = HashSet::new();
    let live = |i: &Intention| i.status == IntentionStatus::Open && i.kind == IntentionKind::Workout;
    for (n, i) in items.iter().enumerate() {
        match i.status {
            IntentionStatus::LetGo => out[n] = Some(Matched { state: ItemState::LetGo, recording: None, by: None, candidates: vec![] }),
            IntentionStatus::Missed => out[n] = Some(Matched { state: ItemState::Missed, recording: None, by: None, candidates: vec![] }),
            IntentionStatus::Open => {}
        }
        if !live(i) {
            continue;
        }
        let Some(link) = links.links.iter().find(|l| l.plan_id == i.id) else {
            continue;
        };
        if recs.iter().any(|r| r.id == link.activity_id) || linked_elsewhere(&link.activity_id) {
            taken.insert(link.activity_id.as_str());
            out[n] = Some(Matched { state: ItemState::Done, recording: Some(link.activity_id.clone()), by: Some(MatchKind::Linked), candidates: vec![] });
        }
    }
    let unlinked = |i: &Intention, r: &Recording| links.unlinks.iter().any(|u| u.plan_id == i.id && u.activity_id == r.id);
    let open: Vec<usize> = (0..items.len()).filter(|n| out[*n].is_none() && live(&items[*n])).collect();
    let free = |taken: &HashSet<&str>, i: &Intention| -> Vec<usize> {
        (0..recs.len())
            .filter(|r| !taken.contains(recs[*r].id.as_str()) && recs[*r].date == i.date && same_sport(i, &recs[*r]))
            .collect()
    };
    let before = taken.clone();
    for &n in &open {
        let i = &items[n];
        let c = free(&before, i);
        if let [r] = c[..] {
            let rival = open
                .iter()
                .filter(|m| items[**m].date == i.date && same_sport(&items[**m], &recs[r]) && !unlinked(&items[**m], &recs[r]))
                .count();
            if rival == 1 && !unlinked(i, &recs[r]) {
                taken.insert(recs[r].id.as_str());
                out[n] = Some(Matched { state: ItemState::Done, recording: Some(recs[r].id.clone()), by: Some(MatchKind::Automatic), candidates: vec![] });
            }
        }
    }
    let unmatched: Vec<usize> = open.iter().copied().filter(|n| out[*n].is_none()).collect();
    for &n in &unmatched {
        let i = &items[n];
        if i.date > today_s {
            continue;
        }
        let c = free(&taken, i);
        let day_left: Vec<&Recording> = recs.iter().filter(|r| r.date == i.date && !taken.contains(r.id.as_str())).collect();
        let day_open = unmatched.iter().filter(|m| items[**m].date == i.date).count();
        let similar = |r: &&&Recording| {
            r.sport != i.sport && ENDURANCE.contains(&r.sport) && ENDURANCE.contains(&i.sport)
                && i.duration_min.is_none_or(|d| r.duration_min >= f64::from(d) * 0.5 && r.duration_min <= f64::from(d) * 2.)
        };
        let swap: Vec<&&Recording> = day_left.iter().filter(similar).collect();
        let other_sport = i.date < today_s && c.is_empty() && swap.len() == 1 && day_open == 1 && day_left.len() == 1;
        if !c.is_empty() || other_sport {
            let candidates = if c.is_empty() { swap.iter().map(|r| r.id.clone()).collect() } else { c.iter().map(|r| recs[*r].id.clone()).collect() };
            out[n] = Some(Matched { state: ItemState::Review, recording: None, by: None, candidates });
        }
    }
    items
        .iter()
        .zip(out)
        .map(|(i, m)| {
            m.unwrap_or_else(|| {
                let state = if i.kind == IntentionKind::WindDown {
                    if wind_downs.iter().any(|w| w.date == i.date) {
                        ItemState::Done
                    } else if i.date >= today_s {
                        ItemState::Planned
                    } else {
                        ItemState::NotRecorded
                    }
                } else if i.date >= today_s {
                    ItemState::Planned
                } else if unknown(&i.date) {
                    ItemState::Awaiting
                } else {
                    ItemState::NotRecorded
                };
                Matched { state, recording: None, by: None, candidates: vec![] }
            })
        })
        .collect()
}

fn allocated(a: &Allocation, kind: GoalKind) -> f64 {
    f64::from(match kind {
        GoalKind::Strength => a.strength_sessions,
        GoalKind::Zone2 => a.zone2_min,
        GoalKind::HighIntensity => a.high_intensity_min,
        GoalKind::Intensity => a.intensity_min,
        GoalKind::SleepTiming => a.sleep_nights,
        GoalKind::SleepScore => 0,
    })
}

/// The weekly target for a kind; None when off, or when it follows a
/// Garmin goal that hasn't synced.
fn target_of(t: &GoalTargets, kind: GoalKind, garmin_goal: Option<f64>) -> Option<f64> {
    let n = match kind {
        GoalKind::Strength => t.strength_sessions,
        GoalKind::Zone2 => t.zone2_min,
        GoalKind::HighIntensity => t.high_intensity_min,
        GoalKind::Intensity if t.intensity_garmin_goal => return garmin_goal,
        GoalKind::Intensity => t.intensity_min,
        GoalKind::SleepTiming => t.sleep_nights,
        GoalKind::SleepScore => t.sleep_score,
    };
    (n > 0).then_some(f64::from(n))
}

fn parse_local(s: &str) -> Option<NaiveDateTime> {
    NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.f").ok()
}

/// Whether the night beginning on the evening of `date` met the goal; None
/// when there is no evidence yet (no recorded sleep, or a night still ahead).
pub(crate) fn night_met(store: &Store, local: &LocalRecords, t: &GoalTargets, date: NaiveDate, today: NaiveDate) -> Option<bool> {
    let morning = date + Duration::days(1);
    match t.sleep_kind {
        SleepTimingKind::WindDown => {
            let reported = local.circumstances.wind_downs.iter().any(|w| w.date == date.to_string());
            if reported {
                Some(true)
            } else {
                (morning <= today).then_some(false)
            }
        }
        SleepTimingKind::SavedPlan => plan::asleep_on_time(store, morning),
        SleepTimingKind::AsleepBy => {
            if morning > today {
                return None;
            }
            let raw = store.get("sleep", &morning.to_string())?;
            let asleep = screens::local_time(screens::sleep(&raw), &["sleepStartTimestampLocal"]).and_then(|t| parse_local(&t))?;
            let at = chrono::NaiveTime::parse_from_str(t.sleep_time.as_deref()?, "%H:%M").ok()?;
            // A chosen time before noon falls after midnight.
            let day = if at < chrono::NaiveTime::from_hms_opt(12, 0, 0)? { morning } else { date };
            Some(asleep <= day.and_time(at) + Duration::minutes(SLEEP_SLACK_MIN))
        }
    }
}

pub(crate) fn daily_intensity(store: &Store, date: NaiveDate) -> Option<f64> {
    let day = store.get("summary", &date.to_string())?;
    Some(number(&day, "moderateIntensityMinutes")? + 2. * number(&day, "vigorousIntensityMinutes")?)
}

fn pause_on<'a>(local: &'a LocalRecords, date: &str) -> Option<&'a Pause> {
    local
        .circumstances
        .pauses
        .iter()
        .find(|p| p.since.as_str() <= date && p.until.as_deref().is_none_or(|u| date <= u))
}

/// The cardio sport recorded most in the last eight weeks that the person's
/// limits allow, otherwise running or walking; None when limits exclude both.
fn usual_cardio(store: &Store, today: NaiveDate, excluded: &[Sport]) -> Option<Sport> {
    let mut n: BTreeMap<Sport, u32> = BTreeMap::new();
    for (id, _) in store.activities_between(&(today - Duration::days(56)).to_string(), &today.to_string()) {
        let sport = local::sport_of(&screens::activity_kind(store, &id).unwrap_or_default());
        if [Sport::Running, Sport::Cycling, Sport::Walking, Sport::Swimming, Sport::Hiking].contains(&sport) && !excluded.contains(&sport) {
            *n.entry(sport).or_default() += 1;
        }
    }
    n.into_iter()
        .max_by_key(|(_, c)| *c)
        .map(|(s, _)| s)
        .or_else(|| [Sport::Running, Sport::Walking].into_iter().find(|s| !excluded.contains(s)))
}

/// The plan week containing `anchor`, measured on `today`.
pub fn board(store: &Store, anchor: NaiveDate, today: NaiveDate, now: DateTime<Utc>) -> PlanBoard {
    board_with(store, &local::load(store), anchor, today, now)
}

/// The plan-your-week prompt, for people who have chosen goals before. It
/// offers last week's review at the start of a week that followed goals, and
/// planning when this week has nothing placed; answered weeks stay quiet.
fn prompt(store: &Store, local: &LocalRecords, start: NaiveDate, today: NaiveDate, items: &[PlanItem]) -> Option<PlanPrompt> {
    let week = start.to_string();
    if local.goals.versions.is_empty() || !store.settings().plan_prompt || local.reviews.prompts.contains_key(&week) {
        return None;
    }
    let active = |d: NaiveDate| local::goals_on(&local.goals, &d.to_string()).is_some_and(|v| RING_KINDS.iter().chain([GoalKind::SleepScore].iter()).any(|k| target_of(&v.targets, *k, Some(1.)).is_some()));
    let placed = active(today) && items.iter().any(|p| !matches!(p.state, ItemState::LetGo | ItemState::Missed));
    let had = active(start - Duration::days(1));
    let opening = today <= start + Duration::days(1);
    if placed && (!opening || !had) {
        return None;
    }
    Some(PlanPrompt { review: (had && opening).then(|| (start - Duration::days(7)).to_string()) })
}

/// Garmin's next race and your local events from today, soonest first.
fn events(store: &Store, local: &LocalRecords, today: NaiveDate, now: DateTime<Utc>) -> Vec<EventLine> {
    let mut list: Vec<EventLine> = local
        .events
        .events
        .iter()
        .filter_map(|e| {
            let d = NaiveDate::parse_from_str(&e.date, "%Y-%m-%d").ok().filter(|d| *d >= today)?;
            Some(EventLine { id: Some(e.id.clone()), name: e.name.clone(), date: e.date.clone(), days_until: (d - today).num_days() as i32, sport: e.sport, source: EventSource::Local, partial: false })
        })
        .collect();
    if let (state, Some(race)) = planned::race(store, &today.to_string(), now) {
        list.push(EventLine { id: None, name: race.name, date: race.date, days_until: race.days_until, sport: None, source: EventSource::Garmin, partial: state == PlannedState::Partial });
    }
    list.sort_by(|a, b| a.date.cmp(&b.date));
    list
}

/// The plan week containing `anchor`, from already loaded local records.
pub fn board_with(store: &Store, local: &LocalRecords, anchor: NaiveDate, today: NaiveDate, now: DateTime<Utc>) -> PlanBoard {
    let start = plan::week_start(anchor, plan::week_starts_on(store));
    let end = start + Duration::days(6);
    let through = end.min(today);
    let as_of = if end < today { end } else if start > today { start } else { today };
    let (start_s, as_of_s) = (start.to_string(), as_of.to_string());
    let version = local::goals_in_week(&local.goals, start, as_of).cloned();
    let opening = local::goals_in_week(&local.goals, start, start).cloned();

    let counted = recordings(store, start, through);
    let recs: Vec<Recording> = counted.iter().map(|c| c.rec.clone()).collect();
    let listed = through >= start && activities_listed(store, start, through);
    let unknown_day = |d: &str| NaiveDate::parse_from_str(d, "%Y-%m-%d").is_ok_and(|d| d <= today && !activities_listed(store, d, d));
    let mut intentions: Vec<Intention> = local
        .intentions
        .items
        .iter()
        .filter(|i| i.date >= start_s && i.date <= end.to_string())
        .cloned()
        .collect();
    // Garmin Coach sessions join your sessions as read-only planned work:
    // strength plans one strength session; cardio plans no zone or intensity
    // minutes, since Garmin gives no steps to infer them from.
    let coach = planned::coach_week(store, &start_s, &end.to_string(), now);
    let coach_sessions: Vec<CoachSession> = coach.iter().flat_map(|c| c.sessions.iter().filter(|s| s.kind.is_some()).cloned()).collect();
    intentions.extend(coach_sessions.iter().map(coach_intention));
    intentions.sort_by(|a, b| (a.date.as_str(), a.time.as_deref().unwrap_or("99")).cmp(&(b.date.as_str(), b.time.as_deref().unwrap_or("99"))));
    let matched = match_items(
        &intentions,
        &recs,
        &local.links,
        &local.circumstances.wind_downs,
        today,
        |id| screens::activity_kind(store, id).is_some(),
        unknown_day,
    );
    let items: Vec<PlanItem> = intentions
        .into_iter()
        .zip(matched)
        .map(|(intention, m)| {
            let coach = coach_sessions.iter().find(|c| c.id == intention.id).cloned();
            PlanItem { intention, state: m.state, recording: m.recording, matched_by: m.by, candidates: m.candidates, coach }
        })
        .collect();

    let intensity_card = plan::intensity(store, start, today);
    let garmin_goal = intensity_card.as_ref().and_then(|c| c.goal).or_else(|| plan::garmin_goal(store, through.max(start)));
    let paused_in_week = local.circumstances.pauses.iter().any(|p| p.since <= as_of_s && p.until.as_deref().is_none_or(|u| u >= start_s.as_str()));
    let future_week = start > today;
    let ledger: Vec<GoalLedger> = match &version {
        None => vec![],
        Some(v) => RING_KINDS
            .iter()
            .filter_map(|&kind| {
                let target = target_of(&v.targets, kind, garmin_goal)?;
                let observation = kind == GoalKind::SleepTiming && v.targets.sleep_kind == SleepTimingKind::SavedPlan;
                let (recorded, unavailable) = if future_week {
                    (Some(0.), None)
                } else {
                    match kind {
                        GoalKind::Strength if !listed => (None, Some("Workout list incomplete")),
                        GoalKind::Strength if counted.iter().any(|c| !c.typed) => (None, Some("Workout details incomplete")),
                        GoalKind::Strength => (Some(recs.iter().filter(|r| r.sport == Sport::Strength).count() as f64), None),
                        GoalKind::Zone2 | GoalKind::HighIntensity if !listed => (None, Some("Workout list incomplete")),
                        GoalKind::Zone2 | GoalKind::HighIntensity => {
                            let secs: Option<f64> = counted.iter().map(|c| if kind == GoalKind::Zone2 { c.zone2_s } else { c.high_s }).sum();
                            match secs {
                                Some(s) => (Some((s / 60.).floor()), None),
                                None => (None, Some("Heart-rate zones incomplete")),
                            }
                        }
                        GoalKind::Intensity => match &intensity_card {
                            Some(c) => (Some(c.total), None),
                            None => (None, Some("Intensity minutes not synced")),
                        },
                        GoalKind::SleepTiming => {
                            let nights: Vec<Option<bool>> = (0..7).map(|i| night_met(store, local, &v.targets, start + Duration::days(i), today)).collect();
                            let passed = (0..7).filter(|i| start + Duration::days(i + 1) <= today).count();
                            // Nights without evidence aren't misses, and none at all is unknown, not zero.
                            if passed > 0 && nights.iter().all(Option::is_none) {
                                (None, Some("No recorded nights yet"))
                            } else {
                                (Some(nights.iter().filter(|n| **n == Some(true)).count() as f64), None)
                            }
                        }
                        GoalKind::SleepScore => (None, None),
                    }
                };
                let planned: f64 = items
                    .iter()
                    .filter(|p| p.state == ItemState::Planned)
                    .map(|p| allocated(&p.intention.allocation, kind))
                    .sum();
                let start_target = opening.as_ref().and_then(|o| target_of(&o.targets, kind, garmin_goal));
                let revised = version.as_ref().is_some_and(|v| v.from > start_s) && start_target != Some(target);
                let remaining = if observation || paused_in_week { None } else { recorded.map(|r| (target - r - planned).max(0.)) };
                Some(GoalLedger {
                    kind,
                    target,
                    original: revised.then_some(start_target.unwrap_or(0.)),
                    revised_on: revised.then(|| v.from.clone()),
                    recorded,
                    planned,
                    remaining,
                    reached: !observation && recorded.is_some_and(|r| r >= target),
                    review: items.iter().filter(|p| p.state == ItemState::Review && allocated(&p.intention.allocation, kind) > 0.).count() as u32,
                    unavailable: unavailable.map(str::to_owned),
                    observation,
                })
            })
            .collect(),
    };

    let (garmin_state, garmin) = planned::sessions(store, &start_s, &end.to_string(), now);
    let days: Vec<DayCell> = (0..7)
        .map(|i| {
            let d = start + Duration::days(i);
            let ds = d.to_string();
            let future = d > today;
            let mark = if local.circumstances.rest.iter().any(|r| r.date == ds) {
                Some(DayMark::Rest)
            } else if let Some(p) = pause_on(local, &ds) {
                Some(if p.reason == PauseReason::Ill { DayMark::Ill } else { DayMark::Paused })
            } else if !future && unknown_day(&ds) {
                Some(DayMark::Unknown)
            } else {
                None
            };
            let day_items: Vec<&PlanItem> = items.iter().filter(|p| p.intention.date == ds).collect();
            let live: Vec<&&PlanItem> = day_items.iter().filter(|p| !matches!(p.state, ItemState::LetGo | ItemState::Missed)).collect();
            let day_recs: Vec<&Counted> = counted.iter().filter(|c| c.rec.date == ds).collect();
            let rings = ledger
                .iter()
                .filter(|l| !l.observation)
                .map(|l| {
                    let planned: f64 = live.iter().map(|p| allocated(&p.intention.allocation, l.kind)).sum();
                    let recorded = if future || mark == Some(DayMark::Unknown) {
                        0.
                    } else {
                        match l.kind {
                            GoalKind::Strength => day_recs.iter().filter(|c| c.rec.sport == Sport::Strength).count() as f64,
                            // A day with unknown zones draws nothing rather than a part.
                            GoalKind::Zone2 => day_recs.iter().map(|c| c.zone2_s).sum::<Option<f64>>().map_or(0., |s| (s / 60.).floor()),
                            GoalKind::HighIntensity => day_recs.iter().map(|c| c.high_s).sum::<Option<f64>>().map_or(0., |s| (s / 60.).floor()),
                            GoalKind::Intensity => daily_intensity(store, d).unwrap_or(0.),
                            GoalKind::SleepTiming => {
                                let met = version.as_ref().is_some_and(|v| night_met(store, local, &v.targets, d, today) == Some(true));
                                if met { 1. } else { 0. }
                            }
                            GoalKind::SleepScore => 0.,
                        }
                    };
                    let fill = if recorded > 0. { recorded / planned.max(recorded) } else { 0. };
                    Ring { kind: l.kind, recorded, planned, fill }
                })
                .collect();
            let workouts: Vec<&&&PlanItem> = live.iter().filter(|p| p.intention.kind == IntentionKind::Workout).collect();
            DayCell {
                date: ds.clone(),
                today: d == today,
                future,
                mark,
                rings,
                done: !workouts.is_empty() && workouts.iter().all(|p| p.state == ItemState::Done),
                not_recorded: day_items.iter().any(|p| p.state == ItemState::NotRecorded),
                recordings: day_recs.iter().map(|c| c.rec.id.clone()).collect(),
                items: day_items.iter().map(|p| p.intention.id.clone()).collect(),
                garmin: (garmin.iter().filter(|g| g.date == ds).count() + coach_sessions.iter().filter(|c| c.date == ds).count()) as u32,
                sleep_score: (!future).then(|| store.get("sleep", &ds)).flatten().and_then(|raw| screens::sscore(&raw)),
            }
        })
        .collect();
    let week_rings = ledger
        .iter()
        .filter(|l| !l.observation)
        .map(|l| {
            let recorded = l.recorded.unwrap_or(0.);
            Ring { kind: l.kind, recorded, planned: l.planned, fill: (recorded / l.target).min(1.) }
        })
        .collect();
    let sleep_score = version.as_ref().and_then(|v| target_of(&v.targets, GoalKind::SleepScore, None)).map(|target| {
        let scores: Vec<f64> = days.iter().filter_map(|d| d.sleep_score).collect();
        SleepScoreGoal {
            target,
            average: (!scores.is_empty()).then(|| (scores.iter().sum::<f64>() / scores.len() as f64).round()),
            nights: scores.len() as u32,
        }
    });

    let current = (start..=end).contains(&today);
    let today_s = today.to_string();
    let backlog_paused = current && pause_on(local, &today_s).is_some();
    let cardio = if current { usual_cardio(store, today, &local.circumstances.limits.excluded) } else { None };
    let backlog = if !current || backlog_paused {
        vec![]
    } else {
        ledger
            .iter()
            .filter(|l| !l.observation)
            .filter_map(|l| Some((l, l.remaining.filter(|r| *r > 0.)?)))
            .flat_map(|(l, left)| {
                let sport = match l.kind {
                    GoalKind::Strength => Some(Sport::Strength),
                    GoalKind::SleepTiming => None,
                    _ => cardio,
                };
                let n = if l.kind == GoalKind::Strength { left as usize } else { 1 };
                let amount = if l.kind == GoalKind::Strength { 1. } else { left };
                std::iter::repeat_n(BacklogItem { kind: l.kind, amount, sport }, n)
            })
            .collect()
    };
    let pause = local
        .circumstances
        .pauses
        .iter()
        .rev()
        .find(|p| p.since <= end.to_string() && p.until.as_deref().is_none_or(|u| u >= start_s.as_str()))
        .map(|p| WeekPause {
            since: p.since.clone(),
            until: p.until.clone(),
            reason: p.reason,
            whole_week: p.since <= start_s && p.until.as_deref().is_none_or(|u| u >= end.to_string().as_str()),
        });
    let plan_prompt = if current { prompt(store, local, start, today, &items) } else { None };
    let event_lines = if current { events(store, local, today, now) } else { vec![] };
    // Garmin's race has no verified sport, so only a local event with one groups sessions.
    let event_week = event_lines.iter().find_map(|e| {
        let sport = e.sport?;
        let ids: Vec<String> = items
            .iter()
            .filter(|p| p.intention.kind == IntentionKind::Workout && p.intention.sport == sport && p.state != ItemState::LetGo)
            .map(|p| p.intention.id.clone())
            .collect();
        (!ids.is_empty()).then(|| EventWeek {
            event: e.clone(),
            sport,
            items: ids,
            recorded_min: recs.iter().filter(|r| r.sport == sport && r.date <= today_s).map(|r| r.duration_min).sum(),
        })
    });
    PlanBoard {
        today: today_s,
        start: start_s,
        end: end.to_string(),
        explain_observation: version.as_ref().is_some_and(|v| v.targets.sleep_kind == SleepTimingKind::SavedPlan && v.targets.sleep_nights > 0)
            && !local.goals.observation_explained,
        goals: version,
        versions: local.goals.versions.clone(),
        garmin_intensity_goal: garmin_goal,
        ledger,
        week_rings,
        sleep_score,
        days,
        items,
        recordings: recs,
        garmin,
        garmin_state,
        backlog,
        backlog_paused,
        pause,
        coverage_complete: listed,
        read_only: !local.read_only.is_empty(),
        prompt: plan_prompt,
        events: event_lines,
        event_week,
        coach,
    }
}
