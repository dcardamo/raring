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
use chrono::{DateTime, Datelike, Duration, Local, NaiveDate, NaiveTime, Timelike, Weekday};
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
    pub today_choice: Option<TodayChoice>,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Reviews {
    /// Plan week start → the answer to that week's prompt.
    pub prompts: BTreeMap<String, PromptChoice>,
    /// The latest decision on each reviewed week's proposals.
    pub decisions: Vec<ReviewDecision>,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Events {
    pub events: Vec<LocalEvent>,
}

/// A milestone as first awarded, with any later correction.
#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct MilestoneRecord {
    pub id: String,
    /// Canonical units: grams for lifts, kilometres for distances.
    pub value: f64,
    pub date: String,
    pub activity: Option<String>,
    pub seen: bool,
    pub corrected: bool,
    pub kind: Option<MilestoneKind>,
    pub title: String,
    pub route: Option<String>,
    pub withdrawn: bool,
    /// The set's own rep count, for showing a best; evidence only, never stored.
    #[serde(skip)]
    pub set_reps: Option<u32>,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Milestones {
    pub items: Vec<MilestoneRecord>,
}

/// Your own checklist and its self-reports (reports kept 90 days).
#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Routines {
    pub items: Vec<RoutineItem>,
    pub reports: Vec<RoutineReport>,
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
impl Record for Milestones {
    const KEY: &'static str = "local:milestones";
}
impl Record for Routines {
    const KEY: &'static str = "local:routines";
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
    pub milestones: Milestones,
    pub routines: Routines,
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

pub(crate) fn load_tx(tx: &SettingsTx) -> Result<LocalRecords> {
    let mut read_only = vec![];
    Ok(LocalRecords {
        goals: read(tx, &mut read_only)?,
        intentions: read(tx, &mut read_only)?,
        links: read(tx, &mut read_only)?,
        circumstances: read(tx, &mut read_only)?,
        reviews: read(tx, &mut read_only)?,
        events: read(tx, &mut read_only)?,
        milestones: read(tx, &mut read_only)?,
        routines: read(tx, &mut read_only)?,
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
    // Each row is protected on its own; a row this build may not rewrite
    // leaves the migration unfinished, to be completed by a build that can.
    let mut blocked = vec![];
    let mut goals: Goals = read(tx, &mut blocked)?;
    if !blocked.contains(&Goals::KEY) && goals.versions.is_empty() {
        goals.versions = versions
            .iter()
            .map(|(from, t)| GoalVersion {
                from: from.clone(),
                targets: migrated(t),
                source: GoalSource::Migrated,
                saved_at: format!("{from}T00:00:00"),
            })
            .collect();
        write(tx, &goals)?;
    }
    let mut circumstances: Circumstances = read(tx, &mut blocked)?;
    if !blocked.contains(&Circumstances::KEY) {
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
        write(tx, &circumstances)?;
    }
    if !blocked.is_empty() {
        return Ok(());
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

/// Every local record, migrating the earlier plan first if needed. Reads take
/// the write lock only when a migration is due.
pub fn load(store: &Store) -> LocalRecords {
    let due = store.setting(MARKER).is_none() && store.setting("plan").is_some();
    let read = |tx: &SettingsTx| -> Result<LocalRecords> {
        if due {
            migrate(tx)?;
        }
        load_tx(tx)
    };
    if due { store.settings_tx(read) } else { store.settings_read(read) }.unwrap_or_default()
}

/// Records an export brings, already parsed and validated by `import`.
#[derive(Default)]
pub(crate) struct Imported {
    pub goals: Option<Goals>,
    pub intentions: Option<Intentions>,
    pub links: Option<Links>,
    pub circumstances: Option<Circumstances>,
    pub reviews: Option<Reviews>,
    pub events: Option<Events>,
    pub routines: Option<Routines>,
}

impl Imported {
    /// An export made before local records existed carries only the earlier
    /// plan row, which must still migrate on this phone.
    fn is_empty(&self) -> bool {
        self.goals.is_none()
            && self.intentions.is_none()
            && self.links.is_none()
            && self.circumstances.is_none()
            && self.reviews.is_none()
            && self.events.is_none()
            && self.routines.is_none()
    }
}

/// What an import changes for one record kind.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RowChange {
    pub kind: &'static str,
    pub added: u32,
    pub replaced: u32,
    pub kept: u32,
}

#[derive(Default, Clone, Copy)]
struct Counts {
    added: u32,
    replaced: u32,
    kept: u32,
}

impl Counts {
    fn row(self, kind: &'static str) -> Option<RowChange> {
        (self.added + self.replaced + self.kept > 0).then_some(RowChange {
            kind,
            added: self.added,
            replaced: self.replaced,
            kept: self.kept,
        })
    }
}

/// Merge an export's records into this phone's. Only local records change; a
/// record this phone already has is kept unless the incoming copy is strictly
/// newer, and a row this build may not rewrite is left alone and reported.
/// Goals merge by effective date; a same-day copy whose `savedAt` is later
/// wins. Sessions, links and decisions take the later timestamp; a later link
/// re-pairs a pair an earlier unlink separated. Rest days, wind-downs,
/// dismissals, prompt answers and checklist items only fill what this phone
/// lacks; a pause with the same id takes a closed interval over an open one.
pub(crate) fn merge(current: &LocalRecords, incoming: &Imported) -> (LocalRecords, Vec<RowChange>, Vec<&'static str>) {
    let mut merged = current.clone();
    let mut rows = vec![];
    let mut skipped = vec![];
    let mut record_row = |kind: &'static str, counts: Counts| {
        rows.extend(counts.row(kind));
    };
    if let Some(value) = &incoming.goals {
        if current.read_only.contains(&Goals::KEY) {
            skipped.push(Goals::KEY);
        } else {
            let (goals, counts) = merge_goals(&current.goals, value);
            merged.goals = goals;
            record_row("Goals", counts);
        }
    }
    if let Some(value) = &incoming.intentions {
        if current.read_only.contains(&Intentions::KEY) {
            skipped.push(Intentions::KEY);
        } else {
            let (intentions, counts) = merge_intentions(&current.intentions, value);
            merged.intentions = intentions;
            record_row("Your sessions", counts);
        }
    }
    if let Some(value) = &incoming.links {
        if current.read_only.contains(&Links::KEY) {
            skipped.push(Links::KEY);
        } else {
            let (links, counts) = merge_links(&current.links, value);
            merged.links = links;
            record_row("Plan links", counts);
        }
    }
    if let Some(value) = &incoming.circumstances {
        if current.read_only.contains(&Circumstances::KEY) {
            skipped.push(Circumstances::KEY);
        } else {
            let (circumstances, counts) = merge_circumstances(&current.circumstances, value);
            merged.circumstances = circumstances;
            record_row("Rest, pauses and limits", counts);
        }
    }
    if let Some(value) = &incoming.reviews {
        if current.read_only.contains(&Reviews::KEY) {
            skipped.push(Reviews::KEY);
        } else {
            let (reviews, counts) = merge_reviews(&current.reviews, value);
            merged.reviews = reviews;
            record_row("Plan-week prompts", counts);
        }
    }
    if let Some(value) = &incoming.events {
        if current.read_only.contains(&Events::KEY) {
            skipped.push(Events::KEY);
        } else {
            let (events, counts) = merge_events(&current.events, value);
            merged.events = events;
            record_row("Events", counts);
        }
    }
    if let Some(value) = &incoming.routines {
        if current.read_only.contains(&Routines::KEY) {
            skipped.push(Routines::KEY);
        } else {
            let (routines, counts) = merge_routines(&current.routines, value);
            merged.routines = routines;
            record_row("Routines", counts);
        }
    }
    (merged, rows, skipped)
}

/// Write the merged rows in one transaction and mark the schema as current, so
/// the earlier plan row is not migrated again on top of imported records.
pub(crate) fn import(tx: &SettingsTx, current: &LocalRecords, incoming: &Imported) -> Result<(Vec<RowChange>, Vec<&'static str>)> {
    let (merged, rows, skipped) = merge(current, incoming);
    if merged.goals != current.goals {
        write(tx, &merged.goals)?;
    }
    if merged.intentions != current.intentions {
        write(tx, &merged.intentions)?;
    }
    if merged.links != current.links {
        write(tx, &merged.links)?;
    }
    if merged.circumstances != current.circumstances {
        write(tx, &merged.circumstances)?;
    }
    if merged.reviews != current.reviews {
        write(tx, &merged.reviews)?;
    }
    if merged.events != current.events {
        write(tx, &merged.events)?;
    }
    if merged.routines != current.routines {
        write(tx, &merged.routines)?;
    }
    // A row this build may not rewrite leaves the migration unfinished, as a change would.
    if !incoming.is_empty() && !skipped.contains(&Goals::KEY) && !skipped.contains(&Circumstances::KEY) {
        tx.set(MARKER, &SCHEMA.to_string())?;
    }
    Ok((rows, skipped))
}

fn merge_goals(current: &Goals, incoming: &Goals) -> (Goals, Counts) {
    let mut merged = current.clone();
    let mut counts = Counts::default();
    for version in &incoming.versions {
        match merged.versions.iter_mut().find(|v| v.from == version.from) {
            Some(existing) if version.saved_at > existing.saved_at => {
                *existing = version.clone();
                counts.replaced += 1;
            }
            Some(_) => counts.kept += 1,
            None => {
                merged.versions.push(version.clone());
                counts.added += 1;
            }
        }
    }
    merged.versions.sort_by(|a, b| a.from.cmp(&b.from));
    merged.observation_explained |= incoming.observation_explained;
    (merged, counts)
}

/// The time of a session's last change; sessions with no history sort first.
fn last_change(i: &Intention) -> &str {
    i.history.last().map_or("", |e| e.at.as_str())
}

fn merge_intentions(current: &Intentions, incoming: &Intentions) -> (Intentions, Counts) {
    let mut merged = current.clone();
    let mut counts = Counts::default();
    for item in &incoming.items {
        match merged.items.iter_mut().find(|i| i.id == item.id) {
            Some(existing) if last_change(item) > last_change(existing) => {
                *existing = item.clone();
                counts.replaced += 1;
            }
            Some(_) => counts.kept += 1,
            None => {
                merged.items.push(item.clone());
                counts.added += 1;
            }
        }
    }
    (merged, counts)
}

fn merge_links(current: &Links, incoming: &Links) -> (Links, Counts) {
    let mut merged = current.clone();
    let mut counts = Counts::default();
    for link in &incoming.links {
        match merged.links.iter_mut().find(|l| l.plan_id == link.plan_id) {
            Some(existing) if link.at > existing.at => {
                *existing = link.clone();
                counts.replaced += 1;
            }
            Some(_) => counts.kept += 1,
            None => {
                merged.links.push(link.clone());
                counts.added += 1;
            }
        }
    }
    for unlink in &incoming.unlinks {
        match merged.unlinks.iter_mut().find(|l| l.plan_id == unlink.plan_id && l.activity_id == unlink.activity_id) {
            Some(existing) if unlink.at > existing.at => {
                *existing = unlink.clone();
                counts.replaced += 1;
            }
            Some(_) => counts.kept += 1,
            None => {
                merged.unlinks.push(unlink.clone());
                counts.added += 1;
            }
        }
    }
    // A later link takes the pair back; a later unlink keeps that pairing apart.
    merged.unlinks.retain(|u| !merged.links.iter().any(|l| l.plan_id == u.plan_id && l.activity_id == u.activity_id && l.at > u.at));
    let unlinks = merged.unlinks.clone();
    merged.links.retain(|l| !unlinks.iter().any(|u| u.plan_id == l.plan_id && u.activity_id == l.activity_id && u.at >= l.at));
    (merged, counts)
}

fn merge_circumstances(current: &Circumstances, incoming: &Circumstances) -> (Circumstances, Counts) {
    let mut merged = current.clone();
    let mut counts = Counts::default();
    for day in &incoming.rest {
        if merged.rest.iter().any(|d| d.date == day.date) {
            counts.kept += 1;
        } else {
            merged.rest.push(day.clone());
            counts.added += 1;
        }
    }
    for pause in &incoming.pauses {
        match merged.pauses.iter_mut().find(|p| p.id == pause.id) {
            // A closed interval is a decision; an open one is still running here.
            Some(existing)
                if pause.until.is_some()
                    && (existing.until.is_none() || pause.until > existing.until) =>
            {
                *existing = pause.clone();
                counts.replaced += 1;
            }
            Some(_) => counts.kept += 1,
            None => {
                merged.pauses.push(pause.clone());
                counts.added += 1;
            }
        }
    }
    if incoming.limits != Limits::default() {
        if merged.limits == Limits::default() {
            merged.limits = incoming.limits.clone();
            counts.replaced += 1;
        } else {
            counts.kept += 1;
        }
    }
    for dismissal in &incoming.dismissals {
        if merged.dismissals.iter().any(|d| d.id == dismissal.id && d.date == dismissal.date) {
            counts.kept += 1;
        } else {
            merged.dismissals.push(dismissal.clone());
            counts.added += 1;
        }
    }
    for report in &incoming.wind_downs {
        if merged.wind_downs.iter().any(|w| w.date == report.date) {
            counts.kept += 1;
        } else {
            merged.wind_downs.push(report.clone());
            counts.added += 1;
        }
    }
    if let Some(choice) = &incoming.today_choice {
        match &merged.today_choice {
            Some(existing) if choice.date <= existing.date => counts.kept += 1,
            Some(_) => {
                merged.today_choice = Some(choice.clone());
                counts.replaced += 1;
            }
            None => {
                merged.today_choice = Some(choice.clone());
                counts.added += 1;
            }
        }
    }
    (merged, counts)
}

fn merge_reviews(current: &Reviews, incoming: &Reviews) -> (Reviews, Counts) {
    let mut merged = current.clone();
    let mut counts = Counts::default();
    for (week, choice) in &incoming.prompts {
        if merged.prompts.contains_key(week) {
            counts.kept += 1;
        } else {
            merged.prompts.insert(week.clone(), *choice);
            counts.added += 1;
        }
    }
    for decision in &incoming.decisions {
        match merged.decisions.iter_mut().find(|d| d.week == decision.week) {
            Some(existing) if decision.at > existing.at => {
                *existing = decision.clone();
                counts.replaced += 1;
            }
            Some(_) => counts.kept += 1,
            None => {
                merged.decisions.push(decision.clone());
                counts.added += 1;
            }
        }
    }
    (merged, counts)
}

fn merge_events(current: &Events, incoming: &Events) -> (Events, Counts) {
    let mut merged = current.clone();
    let mut counts = Counts::default();
    for event in &incoming.events {
        if merged.events.iter().any(|e| e.id == event.id) {
            counts.kept += 1;
        } else {
            merged.events.push(event.clone());
            counts.added += 1;
        }
    }
    merged.events.sort_by(|a, b| a.date.cmp(&b.date));
    (merged, counts)
}

fn merge_routines(current: &Routines, incoming: &Routines) -> (Routines, Counts) {
    let mut merged = current.clone();
    let mut counts = Counts::default();
    for item in &incoming.items {
        if merged.items.iter().any(|i| i.id == item.id) {
            counts.kept += 1;
        } else {
            merged.items.push(item.clone());
            counts.added += 1;
        }
    }
    for report in &incoming.reports {
        match merged.reports.iter_mut().find(|r| r.id == report.id && r.date == report.date) {
            Some(existing) if report.at > existing.at => {
                *existing = report.clone();
                counts.replaced += 1;
            }
            Some(_) => counts.kept += 1,
            None => {
                merged.reports.push(report.clone());
                counts.added += 1;
            }
        }
    }
    (merged, counts)
}

/// The goal version in effect on `date`, if any.
pub fn goals_on<'a>(goals: &'a Goals, date: &str) -> Option<&'a GoalVersion> {
    goals.versions.iter().rev().find(|v| v.from.as_str() <= date)
}

/// The version in effect on `date` in the plan week starting `start`. First
/// targets and migrated versions apply to the whole week they were saved in,
/// even when Garmin's week starts before their date.
pub fn goals_in_week(goals: &Goals, start: NaiveDate, date: NaiveDate) -> Option<&GoalVersion> {
    let (date, end) = (date.to_string(), (start + Duration::days(6)).to_string());
    goals.versions.iter().rev().find(|v| {
        v.from <= date || (matches!(v.source, GoalSource::Setup | GoalSource::Migrated) && v.from >= start.to_string() && v.from <= end)
    })
}

/// The old setup screen writes through until it is retired, so its saves
/// and the goals it migrated cannot diverge.
pub(crate) fn mirror_legacy_plan(store: &Store, start: NaiveDate, update: &PlanUpdate) -> Result<()> {
    store.settings_tx(|tx| {
        if tx.get(MARKER)?.is_none() && tx.get(Goals::KEY)?.is_none() && tx.get(Circumstances::KEY)?.is_none() {
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
        .filter(|d| (1990..=9999).contains(&d.year()))
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
    proposals: &[GoalProposal],
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
            LocalAction::LighterWeek => {
                writable(&r, &[Goals::KEY])?;
                let Some(current) = goals_on(&r.goals, &today.to_string()).map(|v| v.targets.clone()) else {
                    return Err(Error::InvalidLocal("no goals to lighten"));
                };
                let half = |n: u32| if n == 0 { 0 } else { (n / 2).max(1) };
                let half_min = |n: u32| if n == 0 { 0 } else { ((n / 2) / 5 * 5).max(5) };
                let lighter = GoalTargets {
                    strength_sessions: half(current.strength_sessions),
                    zone2_min: half_min(current.zone2_min),
                    high_intensity_min: half_min(current.high_intensity_min),
                    intensity_min: half_min(current.intensity_min),
                    ..current.clone()
                };
                let lighter = check_targets(&lighter, Some(&current))?;
                // The week's first targets stay on record; a same-day setup is edited in Goals instead.
                if r.goals.versions.iter().any(|v| v.from == today.to_string() && v.source == GoalSource::Setup) {
                    return Err(Error::InvalidLocal("these goals were set today; change them in Goals"));
                }
                let next = (week + Duration::days(7)).to_string();
                // Later weeks keep their own targets; otherwise next week returns to the current ones.
                if !r.goals.versions.iter().any(|v| v.from.as_str() >= next.as_str()) {
                    r.goals.versions.push(GoalVersion { from: next, targets: current, source: GoalSource::Revision, saved_at: at.clone() });
                }
                r.goals.versions.retain(|v| v.from != today.to_string());
                r.goals.versions.push(GoalVersion { from: today.to_string(), targets: lighter, source: GoalSource::Revision, saved_at: at.clone() });
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
                    || plan_id.starts_with("coach:")
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
            LocalAction::DecideReview { week: w, accepted } => {
                writable(&r, &[Goals::KEY, Reviews::KEY])?;
                let start = date(w)?;
                if start > today || plan::week_start(start, first) != start {
                    return Err(Error::InvalidLocal("review week"));
                }
                if !accepted.is_empty() && !crate::review::latest_completed(start, today, plan::week_start(today, first)) {
                    return Err(Error::InvalidLocal("only the latest completed week can change upcoming goals"));
                }
                // Once next week has begun, a change applies from today and keeps its starting targets.
                let next = (start + Duration::days(7)).max(today).to_string();
                let chosen: Vec<&GoalProposal> = proposals
                    .iter()
                    .filter(|p| accepted.contains(&p.kind) && p.proposed != p.current)
                    .collect();
                if chosen.len() != accepted.len() {
                    return Err(Error::InvalidLocal("only offered changes can be accepted"));
                }
                if !chosen.is_empty() {
                    let base = goals_on(&r.goals, &next).ok_or(Error::InvalidLocal("no goals"))?;
                    let mut targets = base.targets.clone();
                    for p in &chosen {
                        let v = p.proposed.round() as u32;
                        match p.kind {
                            GoalKind::Strength => targets.strength_sessions = v,
                            GoalKind::Zone2 => targets.zone2_min = v,
                            GoalKind::HighIntensity => targets.high_intensity_min = v,
                            GoalKind::Intensity => targets.intensity_min = v,
                            GoalKind::SleepTiming => targets.sleep_nights = v,
                            GoalKind::SleepScore => return Err(Error::InvalidLocal("Sleep Score is never proposed")),
                        }
                    }
                    r.goals.versions.retain(|v| v.from != next);
                    r.goals.versions.push(GoalVersion { from: next, targets, source: GoalSource::Proposal, saved_at: at.clone() });
                    r.goals.versions.sort_by(|a, b| a.from.cmp(&b.from));
                }
                r.reviews.decisions.retain(|d| d.week != start.to_string());
                r.reviews.decisions.push(ReviewDecision { week: start.to_string(), at: at.clone(), proposals: proposals.to_vec(), accepted: accepted.clone() });
                &[Goals::KEY, Reviews::KEY]
            }
            LocalAction::KeepToday { plan_id } => {
                writable(&r, &[Circumstances::KEY])?;
                let ok = r.intentions.items.iter().any(|i| {
                    i.id == *plan_id && i.date == today.to_string() && i.status == IntentionStatus::Open && !r.circumstances.limits.excluded.contains(&i.sport)
                })
                    || ["garmin:", "coach:"].iter().any(|p| plan_id.strip_prefix(p).is_some_and(|rest| rest.starts_with(&format!("{today}:")) && rest.len() > 11));
                if !ok {
                    return Err(Error::InvalidLocal("only one of today's sessions, outside your limits, can be kept"));
                }
                r.circumstances.today_choice = Some(TodayChoice { date: today.to_string(), plan_id: plan_id.clone() });
                &[Circumstances::KEY]
            }
            LocalAction::MilestonesSeen => {
                writable(&r, &[Milestones::KEY])?;
                r.milestones.items.iter_mut().for_each(|m| m.seen = true);
                &[Milestones::KEY]
            }
            LocalAction::AddRoutine { name, how } => {
                writable(&r, &[Routines::KEY])?;
                let name = text(&Some(name.clone()), 80)?.ok_or(Error::InvalidLocal("name"))?;
                let how = text(how, 300)?;
                if r.routines.items.len() >= 30 {
                    return Err(Error::InvalidLocal("at most 30 checklist items"));
                }
                let id = new_id("r", now, |id| r.routines.items.iter().any(|i| i.id == id));
                r.routines.items.push(RoutineItem { id, name, how });
                &[Routines::KEY]
            }
            LocalAction::RemoveRoutine { id } => {
                writable(&r, &[Routines::KEY])?;
                r.routines.items.retain(|i| i.id != *id);
                r.routines.reports.retain(|x| x.id != *id);
                &[Routines::KEY]
            }
            LocalAction::ReportRoutine { id, done } => {
                writable(&r, &[Routines::KEY])?;
                if !r.routines.items.iter().any(|i| i.id == *id) {
                    return Err(Error::InvalidLocal("not on your checklist"));
                }
                let day = today.to_string();
                r.routines.reports.retain(|x| !(x.id == *id && x.date == day));
                if *done {
                    r.routines.reports.push(RoutineReport { id: id.clone(), date: day, at: at.clone() });
                }
                let oldest = (today - Duration::days(90)).to_string();
                r.routines.reports.retain(|x| x.date >= oldest);
                &[Routines::KEY]
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
                Milestones::KEY => write(tx, &r.milestones)?,
                Routines::KEY => write(tx, &r.routines)?,
                _ => {}
            }
        }
        if !r.read_only.contains(&Goals::KEY) && !r.read_only.contains(&Circumstances::KEY) {
            tx.set(MARKER, &SCHEMA.to_string())?;
        }
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

/// Merge evidence atomically with acknowledgements. Backfill and corrections
/// are quiet; only a genuinely later recording dated today can be new.
pub(crate) fn sync_milestones(store: &Store, evidence: &[MilestoneRecord], today: NaiveDate) -> Result<Vec<MilestoneRecord>> {
    store.settings_tx(|tx| {
        let mut blocked = vec![];
        let current: Milestones = read(tx, &mut blocked)?;
        if !blocked.is_empty() {
            return Ok(current.items);
        }
        let mut next = current.clone();
        for e in evidence {
            match next.items.iter_mut().find(|m| m.id == e.id) {
                None => next.items.push(MilestoneRecord { seen: e.date != today.to_string(), ..e.clone() }),
                Some(m) if e.value != m.value || e.date != m.date || e.activity != m.activity || m.withdrawn => {
                    let improved = e.value > m.value && e.date > m.date && !m.withdrawn;
                    let seen = !improved || e.date != today.to_string();
                    *m = MilestoneRecord { seen, corrected: !improved, ..e.clone() };
                }
                Some(_) => {}
            }
        }
        for m in &mut next.items {
            if !evidence.iter().any(|e| e.id == m.id) {
                m.corrected = true;
                m.withdrawn = true;
                m.seen = true;
                m.route = None;
            }
        }
        if next != current {
            write(tx, &next)?;
        }
        Ok(next.items)
    })
}
