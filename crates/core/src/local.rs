//! Local records (docs/spec.md, Rules: Local records): goals, your sessions,
//! plan links, circumstances, plan-week prompts and local events. Each is a
//! versioned JSON row in the settings table, so exports, verified account
//! switches and sign-out already cover them.
use crate::{
    plan,
    store::{SettingsTx, Store},
    view::*,
    Error, Result,
};
use chrono::{DateTime, Duration, Local, NaiveDate, NaiveTime, Timelike, Weekday};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

/// The schema this build reads and writes.
pub const SCHEMA: u64 = 1;
/// Present once the earlier plan row has been migrated or a record written.
const MARKER: &str = "local:schema";

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Goals {
    /// Ordered by `from`.
    pub versions: Vec<GoalVersion>,
    /// Plan has explained the migrated sleep-timing observation once.
    pub observation_explained: bool,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Intentions {
    pub items: Vec<Intention>,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Links {
    pub links: Vec<PlanLink>,
    /// Automatic pairs the person took apart; they stay apart until linked.
    pub unlinks: Vec<PlanLink>,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Circumstances {
    pub rest: Vec<RestDay>,
    pub pauses: Vec<Pause>,
    pub limits: Limits,
    pub dismissals: Vec<Dismissal>,
    pub wind_downs: Vec<WindDownReport>,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Reviews {
    /// Plan week start → the answer to that week's prompt.
    pub prompts: BTreeMap<String, PromptChoice>,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Events {
    pub events: Vec<LocalEvent>,
}

trait Record: Serialize + DeserializeOwned + Default {
    const KEY: &'static str;
}
impl Record for Goals {
    const KEY: &'static str = "local:goals";
}
impl Record for Intentions {
    const KEY: &'static str = "local:intentions";
}
impl Record for Links {
    const KEY: &'static str = "local:links";
}
impl Record for Circumstances {
    const KEY: &'static str = "local:circumstances";
}
impl Record for Reviews {
    const KEY: &'static str = "local:reviews";
}
impl Record for Events {
    const KEY: &'static str = "local:events";
}

/// Every local record, after migration.
#[derive(Default, Clone, Debug, PartialEq)]
pub struct LocalRecords {
    pub goals: Goals,
    pub intentions: Intentions,
    pub links: Links,
    pub circumstances: Circumstances,
    pub reviews: Reviews,
    pub events: Events,
    /// Rows written by a newer build or unreadable: shown as far as they
    /// parse, never rewritten.
    pub read_only: Vec<&'static str>,
}

/// A row that parses and has a schema this build may rewrite.
fn read<T: Record>(tx: &SettingsTx, read_only: &mut Vec<&'static str>) -> Result<T> {
    let Some(raw) = tx.get(T::KEY)? else {
        return Ok(T::default());
    };
    let value: Option<Value> = serde_json::from_str(&raw).ok();
    let schema = value.as_ref().and_then(|v| v["schema"].as_u64());
    let parsed = value.and_then(|v| serde_json::from_value::<T>(v).ok());
    if schema != Some(SCHEMA) || parsed.is_none() {
        read_only.push(T::KEY);
    }
    Ok(parsed.unwrap_or_default())
}

fn write<T: Record>(tx: &SettingsTx, value: &T) -> Result<()> {
    let mut json = serde_json::to_value(value).map_err(|_| Error::InvalidLocal("encode"))?;
    json["schema"] = SCHEMA.into();
    tx.set(T::KEY, &json.to_string())?;
    Ok(())
}

fn load_tx(tx: &SettingsTx) -> Result<LocalRecords> {
    let mut read_only = vec![];
    Ok(LocalRecords {
        goals: read(tx, &mut read_only)?,
        intentions: read(tx, &mut read_only)?,
        links: read(tx, &mut read_only)?,
        circumstances: read(tx, &mut read_only)?,
        reviews: read(tx, &mut read_only)?,
        events: read(tx, &mut read_only)?,
        read_only,
    })
}

/// The earlier weekly plan becomes goals and pauses once. Its row and the
/// saved sleep plans stay as they are, so earlier weeks keep their results.
fn migrate(tx: &SettingsTx) -> Result<()> {
    if tx.get(MARKER)?.is_some() {
        return Ok(());
    }
    // An empty database stays empty: an account binding treats any settings
    // row as account history.
    let Some(raw) = tx.get("plan")? else {
        return Ok(());
    };
    let (versions, paused) = plan::legacy(&raw).unwrap_or_default();
    let mut ignored = vec![];
    let mut goals: Goals = read(tx, &mut ignored)?;
    let mut circumstances: Circumstances = read(tx, &mut ignored)?;
    if goals.versions.is_empty() {
        goals.versions = versions
            .iter()
            .map(|(from, t)| GoalVersion {
                from: from.clone(),
                targets: migrated(t),
                source: GoalSource::Migrated,
                saved_at: format!("{from}T00:00:00"),
            })
            .collect();
    }
    for week in &paused {
        let Ok(start) = NaiveDate::parse_from_str(week, "%Y-%m-%d") else {
            continue;
        };
        let id = format!("plan-{week}");
        if !circumstances.pauses.iter().any(|p| p.id == id) {
            circumstances.pauses.push(Pause {
                id,
                since: start.to_string(),
                until: Some((start + Duration::days(6)).to_string()),
                reason: PauseReason::Pause,
                review: None,
            });
        }
    }
    circumstances.pauses.sort_by(|a, b| a.since.cmp(&b.since));
    if ignored.is_empty() {
        write(tx, &goals)?;
        write(tx, &circumstances)?;
    }
    tx.set(MARKER, &SCHEMA.to_string())?;
    Ok(())
}

/// Saved asleep-on-time nights become a neutral observation against each
/// night's saved plan, not a new goal with a verdict.
fn migrated(t: &PlanTargets) -> GoalTargets {
    let mut rest_days = t.rest_days.clone();
    rest_days.retain(|d| *d < 7);
    GoalTargets {
        strength_sessions: t.strength_sessions,
        zone2_min: t.zone2_min,
        high_intensity_min: 0,
        intensity_min: t.intensity_min.unwrap_or(0),
        intensity_garmin_goal: t.intensity_min.is_none(),
        sleep_nights: t.asleep_nights,
        sleep_time: None,
        sleep_kind: SleepTimingKind::SavedPlan,
        sleep_score: 0,
        rest_days,
    }
}

/// Every local record, migrating the earlier plan first if needed.
pub fn load(store: &Store) -> LocalRecords {
    store
        .settings_tx(|tx| {
            migrate(tx)?;
            load_tx(tx)
        })
        .unwrap_or_default()
}

/// The goal version in effect on `date`, if any.
pub fn goals_on<'a>(goals: &'a Goals, date: &str) -> Option<&'a GoalVersion> {
    goals.versions.iter().rev().find(|v| v.from.as_str() <= date)
}

/// The old setup screen writes through until it is retired, so its saves
/// and the goals it migrated cannot diverge.
pub(crate) fn mirror_legacy_plan(store: &Store, start: NaiveDate, update: &PlanUpdate) -> Result<()> {
    store.settings_tx(|tx| {
        if tx.get(MARKER)?.is_none() {
            // The next read migrates the whole row, this save included.
            return Ok(());
        }
        let mut r = load_tx(tx)?;
        writable(&r, &[Goals::KEY, Circumstances::KEY])?;
        let from = start.to_string();
        r.goals.versions.retain(|v| v.from != from);
        r.goals.versions.push(GoalVersion {
            from: from.clone(),
            targets: migrated(&update.targets),
            source: GoalSource::Migrated,
            saved_at: Local::now().naive_local().format("%Y-%m-%dT%H:%M:%S").to_string(),
        });
        r.goals.versions.sort_by(|a, b| a.from.cmp(&b.from));
        let id = format!("plan-{from}");
        r.circumstances.pauses.retain(|p| p.id != id);
        if update.paused {
            r.circumstances.pauses.push(Pause {
                id,
                since: from,
                until: Some((start + Duration::days(6)).to_string()),
                reason: PauseReason::Pause,
                review: None,
            });
            r.circumstances.pauses.sort_by(|a, b| a.since.cmp(&b.since));
        }
        write(tx, &r.goals)?;
        write(tx, &r.circumstances)
    })
}

fn writable(r: &LocalRecords, keys: &[&'static str]) -> Result<()> {
    if keys.iter().any(|k| r.read_only.contains(k)) {
        Err(Error::InvalidLocal("written by a newer version of Raring"))
    } else {
        Ok(())
    }
}

fn date(s: &str) -> Result<NaiveDate> {
    (s.len() == 10)
        .then(|| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
        .flatten()
        .ok_or(Error::InvalidLocal("date"))
}

fn time(s: &Option<String>) -> Result<()> {
    match s {
        Some(t) if t.len() != 5 || NaiveTime::parse_from_str(t, "%H:%M").is_err() => {
            Err(Error::InvalidLocal("time"))
        }
        _ => Ok(()),
    }
}

fn text(s: &Option<String>, max: usize) -> Result<Option<String>> {
    let t = s.as_deref().map(str::trim).filter(|t| !t.is_empty());
    if t.is_some_and(|t| t.chars().count() > max) {
        return Err(Error::InvalidLocal("text too long"));
    }
    Ok(t.map(str::to_owned))
}

fn check_targets(t: &GoalTargets, current: Option<&GoalTargets>) -> Result<GoalTargets> {
    let mut t = t.clone();
    if t.strength_sessions > 14
        || [t.zone2_min, t.high_intensity_min, t.intensity_min].iter().any(|m| *m > 3000)
        || t.sleep_nights > 7
        || t.sleep_score > 100
    {
        return Err(Error::InvalidLocal("target out of range"));
    }
    time(&t.sleep_time)?;
    // A saved-plan observation is only ever carried over, never chosen anew.
    if t.sleep_nights > 0 && t.sleep_kind == SleepTimingKind::SavedPlan
        && current.is_none_or(|c| c.sleep_kind != SleepTimingKind::SavedPlan || c.sleep_nights == 0)
    {
        return Err(Error::InvalidLocal("sleep timing kind"));
    }
    if t.sleep_nights > 0 && t.sleep_kind != SleepTimingKind::SavedPlan && t.sleep_time.is_none() {
        return Err(Error::InvalidLocal("sleep time"));
    }
    t.rest_days.retain(|d| *d < 7);
    t.rest_days.sort_unstable();
    t.rest_days.dedup();
    Ok(t)
}

fn check_allocation(a: &Allocation) -> Result<()> {
    if a.strength_sessions > 3
        || [a.zone2_min, a.high_intensity_min, a.intensity_min].iter().any(|m| *m > 600)
        || a.sleep_nights > 1
    {
        return Err(Error::InvalidLocal("allocation out of range"));
    }
    Ok(())
}

fn new_id(prefix: &str, now: DateTime<Local>, taken: impl Fn(&str) -> bool) -> String {
    let base = format!("{prefix}{}", now.timestamp_millis());
    let mut id = base.clone();
    let mut n = 1;
    while taken(&id) {
        id = format!("{base}-{n}");
        n += 1;
    }
    id
}

/// Applies one change. `first` is the plan week's first day; `now` is the
/// phone's local time, whose date is today for every "today" action.
pub(crate) fn apply(
    store: &Store,
    action: &LocalAction,
    now: DateTime<Local>,
    first: Weekday,
) -> Result<()> {
    store.settings_tx(|tx| {
        migrate(tx)?;
        let mut r = load_tx(tx)?;
        let today = now.date_naive();
        let at = now.naive_local().format("%Y-%m-%dT%H:%M:%S").to_string();
        let week = plan::week_start(today, first);
        let event = |change, from: Option<&str>, to: Option<&str>| IntentionEvent {
            at: at.clone(),
            change,
            from: from.map(str::to_owned),
            to: to.map(str::to_owned),
        };
        let open_intention = |r: &LocalRecords, id: &str| {
            r.intentions
                .items
                .iter()
                .position(|i| i.id == id)
                .ok_or(Error::InvalidLocal("no such session"))
        };
        let changed: &[&'static str] = match action {
            LocalAction::SetGoals { targets } => {
                writable(&r, &[Goals::KEY])?;
                let current = goals_on(&r.goals, &today.to_string()).map(|v| v.targets.clone());
                let targets = check_targets(targets, current.as_ref())?;
                // A manual change supersedes accepted changes for later weeks.
                r.goals.versions.retain(|v| v.from.as_str() <= today.to_string().as_str());
                let (from, source) = if r.goals.versions.is_empty() {
                    (week, GoalSource::Setup)
                } else {
                    (today, GoalSource::Revision)
                };
                let from = from.to_string();
                let source = match r.goals.versions.iter().find(|v| v.from == from) {
                    // Same-day edits replace that day's change and keep its kind.
                    Some(v) if v.source == GoalSource::Setup => GoalSource::Setup,
                    _ => source,
                };
                r.goals.versions.retain(|v| v.from != from);
                r.goals.versions.push(GoalVersion { from, targets, source, saved_at: at.clone() });
                r.goals.versions.sort_by(|a, b| a.from.cmp(&b.from));
                &[Goals::KEY]
            }
            LocalAction::AddIntention { date: d, time: t, sport, type_key, name, duration_min, allocation, kind, garmin_id } => {
                writable(&r, &[Intentions::KEY])?;
                let d = date(d)?;
                if d < today {
                    return Err(Error::InvalidLocal("sessions are added for today or later"));
                }
                time(t)?;
                check_allocation(allocation)?;
                if duration_min.is_some_and(|m| m == 0 || m > 600) {
                    return Err(Error::InvalidLocal("duration"));
                }
                let id = new_id("i", now, |id| r.intentions.items.iter().any(|i| i.id == id));
                r.intentions.items.push(Intention {
                    id,
                    date: d.to_string(),
                    time: t.clone(),
                    sport: *sport,
                    type_key: text(type_key, 60)?,
                    name: text(name, 60)?,
                    duration_min: *duration_min,
                    allocation: allocation.clone(),
                    kind: *kind,
                    garmin_id: text(garmin_id, 100)?,
                    status: IntentionStatus::Open,
                    history: vec![event(IntentionChange::Added, None, Some(&d.to_string()))],
                });
                &[Intentions::KEY]
            }
            LocalAction::EditIntention { id, time: t, name, duration_min, allocation } => {
                writable(&r, &[Intentions::KEY])?;
                let i = open_intention(&r, id)?;
                time(t)?;
                check_allocation(allocation)?;
                if duration_min.is_some_and(|m| m == 0 || m > 600) {
                    return Err(Error::InvalidLocal("duration"));
                }
                let name = text(name, 60)?;
                let item = &mut r.intentions.items[i];
                item.time = t.clone();
                item.name = name;
                item.duration_min = *duration_min;
                item.allocation = allocation.clone();
                item.history.push(event(IntentionChange::Edited, None, None));
                &[Intentions::KEY]
            }
            LocalAction::MoveIntention { id, date: d } => {
                writable(&r, &[Intentions::KEY, Links::KEY])?;
                let i = open_intention(&r, id)?;
                let d = date(d)?;
                if d < today {
                    return Err(Error::InvalidLocal("sessions move to today or later"));
                }
                let from = r.intentions.items[i].date.clone();
                if from == d.to_string() {
                    return Ok(());
                }
                let linked = r.links.links.iter().any(|l| l.plan_id == *id);
                r.links.links.retain(|l| l.plan_id != *id);
                r.links.unlinks.retain(|l| l.plan_id != *id);
                let item = &mut r.intentions.items[i];
                item.date = d.to_string();
                item.status = IntentionStatus::Open;
                item.history.push(event(IntentionChange::Moved, Some(&from), Some(&d.to_string())));
                if linked {
                    item.history.push(event(IntentionChange::LinkCleared, None, None));
                }
                &[Intentions::KEY, Links::KEY]
            }
            LocalAction::SetIntentionStatus { id, status } => {
                writable(&r, &[Intentions::KEY])?;
                let i = open_intention(&r, id)?;
                let item = &mut r.intentions.items[i];
                // Only the person marks a passed session missed; nothing does it for them.
                if *status == IntentionStatus::Missed && date(&item.date)? >= today {
                    return Err(Error::InvalidLocal("only a passed session can be marked missed"));
                }
                if item.status == *status {
                    return Ok(());
                }
                item.status = *status;
                item.history.push(event(
                    match status {
                        IntentionStatus::Open => IntentionChange::Reopened,
                        IntentionStatus::LetGo => IntentionChange::LetGo,
                        IntentionStatus::Missed => IntentionChange::Missed,
                    },
                    None,
                    None,
                ));
                &[Intentions::KEY]
            }
            LocalAction::RemoveIntention { id } => {
                writable(&r, &[Intentions::KEY, Links::KEY])?;
                let i = open_intention(&r, id)?;
                // Passed sessions are let go instead, so their history stays.
                if date(&r.intentions.items[i].date)? < today {
                    return Err(Error::InvalidLocal("let a passed session go instead"));
                }
                r.intentions.items.remove(i);
                r.links.links.retain(|l| l.plan_id != *id);
                r.links.unlinks.retain(|l| l.plan_id != *id);
                &[Intentions::KEY, Links::KEY]
            }
            LocalAction::Link { plan_id, activity_id } => {
                writable(&r, &[Links::KEY])?;
                let known_plan = plan_id.starts_with("garmin:")
                    || r.intentions.items.iter().any(|i| i.id == *plan_id);
                if !known_plan || !tx.activity_exists(activity_id)? {
                    return Err(Error::InvalidLocal("no such session or recording"));
                }
                // One recording completes at most one plan item.
                r.links.links.retain(|l| l.plan_id != *plan_id && l.activity_id != *activity_id);
                r.links.unlinks.retain(|l| !(l.plan_id == *plan_id && l.activity_id == *activity_id));
                r.links.links.push(PlanLink { plan_id: plan_id.clone(), activity_id: activity_id.clone(), at: at.clone() });
                &[Links::KEY]
            }
            LocalAction::Unlink { plan_id, activity_id } => {
                writable(&r, &[Links::KEY])?;
                r.links.links.retain(|l| l.plan_id != *plan_id);
                if !r.links.unlinks.iter().any(|l| l.plan_id == *plan_id && l.activity_id == *activity_id) {
                    r.links.unlinks.push(PlanLink { plan_id: plan_id.clone(), activity_id: activity_id.clone(), at: at.clone() });
                }
                &[Links::KEY]
            }
            LocalAction::RestToday => {
                writable(&r, &[Circumstances::KEY])?;
                if !r.circumstances.rest.iter().any(|d| d.date == today.to_string()) {
                    r.circumstances.rest.push(RestDay { date: today.to_string(), at: at.clone() });
                }
                &[Circumstances::KEY]
            }
            LocalAction::CancelRest => {
                writable(&r, &[Circumstances::KEY])?;
                r.circumstances.rest.retain(|d| d.date != today.to_string());
                &[Circumstances::KEY]
            }
            LocalAction::Pause { reason, review } => {
                writable(&r, &[Circumstances::KEY])?;
                if let Some(d) = review {
                    if date(d)? <= today {
                        return Err(Error::InvalidLocal("a review date is after today"));
                    }
                }
                let today_s = today.to_string();
                match r.circumstances.pauses.iter_mut().find(|p| p.until.is_none()) {
                    Some(p) => {
                        p.reason = *reason;
                        p.review = review.clone();
                    }
                    None => {
                        let id = new_id("p", now, |id| r.circumstances.pauses.iter().any(|p| p.id == id));
                        r.circumstances.pauses.push(Pause {
                            id,
                            since: today_s,
                            until: None,
                            reason: *reason,
                            review: review.clone(),
                        });
                    }
                }
                &[Circumstances::KEY]
            }
            LocalAction::Resume => {
                writable(&r, &[Circumstances::KEY])?;
                let i = r
                    .circumstances
                    .pauses
                    .iter()
                    .position(|p| p.until.is_none())
                    .ok_or(Error::InvalidLocal("nothing is paused"))?;
                // Today is active again; a pause begun today leaves no interval.
                let until = today - Duration::days(1);
                if date(&r.circumstances.pauses[i].since)? > until {
                    r.circumstances.pauses.remove(i);
                } else {
                    r.circumstances.pauses[i].until = Some(until.to_string());
                    r.circumstances.pauses[i].review = None;
                }
                &[Circumstances::KEY]
            }
            LocalAction::SetLimits { limits } => {
                writable(&r, &[Circumstances::KEY])?;
                let mut excluded = limits.excluded.clone();
                excluded.sort_unstable();
                excluded.dedup();
                let note = limits.note.trim().to_owned();
                if note.chars().count() > 500 {
                    return Err(Error::InvalidLocal("text too long"));
                }
                r.circumstances.limits = Limits { excluded, note };
                &[Circumstances::KEY]
            }
            LocalAction::Dismiss { id } => {
                writable(&r, &[Circumstances::KEY])?;
                if id.is_empty() || id.len() > 100 {
                    return Err(Error::InvalidLocal("suggestion"));
                }
                // Dismissals last a local day; old ones are no longer needed.
                let keep = (today - Duration::days(30)).to_string();
                r.circumstances.dismissals.retain(|d| d.date >= keep);
                if !r.circumstances.dismissals.iter().any(|d| d.id == *id && d.date == today.to_string()) {
                    r.circumstances.dismissals.push(Dismissal { id: id.clone(), date: today.to_string() });
                }
                &[Circumstances::KEY]
            }
            LocalAction::ReportWindDown => {
                writable(&r, &[Circumstances::KEY])?;
                // A report after midnight and before noon belongs to the evening before.
                let evening = if now.hour() < 12 { today - Duration::days(1) } else { today };
                if !r.circumstances.wind_downs.iter().any(|w| w.date == evening.to_string()) {
                    r.circumstances.wind_downs.push(WindDownReport { date: evening.to_string(), at: at.clone() });
                }
                &[Circumstances::KEY]
            }
            LocalAction::SetPromptWeek { week: w, choice } => {
                writable(&r, &[Reviews::KEY])?;
                r.reviews.prompts.insert(date(w)?.to_string(), *choice);
                &[Reviews::KEY]
            }
            LocalAction::AddEvent { name, date: d, sport } => {
                writable(&r, &[Events::KEY])?;
                let name = text(&Some(name.clone()), 60)?.ok_or(Error::InvalidLocal("name"))?;
                let d = date(d)?;
                if d < today {
                    return Err(Error::InvalidLocal("events are dated today or later"));
                }
                let id = new_id("e", now, |id| r.events.events.iter().any(|e| e.id == id));
                r.events.events.push(LocalEvent { id, name, date: d.to_string(), sport: *sport });
                r.events.events.sort_by(|a, b| a.date.cmp(&b.date));
                &[Events::KEY]
            }
            LocalAction::ObservationExplained => {
                writable(&r, &[Goals::KEY])?;
                r.goals.observation_explained = true;
                &[Goals::KEY]
            }
            LocalAction::RemoveEvent { id } => {
                writable(&r, &[Events::KEY])?;
                r.events.events.retain(|e| e.id != *id);
                &[Events::KEY]
            }
        };
        for key in changed {
            match *key {
                Goals::KEY => write(tx, &r.goals)?,
                Intentions::KEY => write(tx, &r.intentions)?,
                Links::KEY => write(tx, &r.links)?,
                Circumstances::KEY => write(tx, &r.circumstances)?,
                Reviews::KEY => write(tx, &r.reviews)?,
                Events::KEY => write(tx, &r.events)?,
                _ => {}
            }
        }
        tx.set(MARKER, &SCHEMA.to_string())?;
        Ok(())
    })
}

/// A Garmin activity type key's sport family; mixed types stay `Other`.
pub fn sport_of(type_key: &str) -> Sport {
    let k = type_key.to_ascii_lowercase();
    if k.contains("run") {
        Sport::Running
    } else if k.contains("cycl") || k.contains("bik") || k.contains("ride") {
        Sport::Cycling
    } else if k.contains("swim") {
        Sport::Swimming
    } else if k.contains("walk") {
        Sport::Walking
    } else if k.contains("hik") {
        Sport::Hiking
    } else if k.contains("strength") {
        Sport::Strength
    } else {
        Sport::Other
    }
}
