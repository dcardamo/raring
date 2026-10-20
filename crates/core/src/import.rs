//! Validated import of the local settings rows in a Raring export.
//!
//! The chosen archive is read from a local path; nothing here uses the
//! network. Only settings rows are imported: the weekly plan row, prospective
//! sleep-plan snapshots, the versioned `local:*` records and the plan prompt
//! preference. Garmin responses, journal logs and milestones are left out
//! because a sync rebuilds them from Garmin. Every export names the account
//! that made it, and a file that names another account is refused before
//! anything is written (docs/spec.md, Rules: Import).
use crate::{
    account, export, local,
    plan,
    store::{SettingsTx, Store},
    view::*,
    Error,
};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::Read,
    path::Path,
};
use zip::ZipArchive;

/// The archive format this build writes and can import.
pub(crate) const FORMAT_VERSION: u64 = 1;
/// Bounds on the two entries an import reads; the archive's Garmin responses
/// are never opened, so a large export is not a problem for the reader.
const MAX_SETTINGS: u64 = 8 * 1024 * 1024;
const MAX_MANIFEST: u64 = 64 * 1024;

#[derive(Debug, thiserror::Error)]
pub enum ImportError {
    #[error("This file isn't a Raring export.")]
    NotAnArchive,
    #[error("This export was made by a newer version of Raring. Update the app to import it.")]
    NewerExport,
    #[error("The export's {0} was written by a newer version of Raring. Update the app to import it.")]
    NewerRow(String),
    #[error("This export doesn't name the Garmin account that made it, so it can't be imported.")]
    MissingAccount,
    #[error("This export belongs to a different Garmin account. Nothing was imported.")]
    AccountMismatch,
    #[error("Sign in to Garmin before importing settings.")]
    NotSignedIn,
    #[error("The account changed during the import. Nothing was imported.")]
    AccountChanged,
    #[error("This file contains sign-in credentials and was not imported.")]
    Credentials,
    #[error("{0}")]
    Unreadable(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Zip(#[from] zip::result::ZipError),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Sql(#[from] rusqlite::Error),
    #[error(transparent)]
    Local(#[from] Error),
}

type Result<T> = std::result::Result<T, ImportError>;

#[derive(Deserialize)]
struct Manifest {
    /// The active account, as Garmin's numeric profile id.
    account: String,
    exported_at: Option<String>,
    app_version: Option<String>,
}

/// The earlier weekly plan row, as written by the earlier setup.
#[derive(Serialize, Deserialize, Default)]
struct PlanRow {
    versions: BTreeMap<String, PlanTargets>,
    paused: BTreeSet<String>,
}

#[derive(Default)]
struct Archive {
    account: String,
    exported_at: Option<String>,
    app_version: Option<String>,
    /// Parsed, schema-checked local records.
    records: local::Imported,
    plan_versions: BTreeMap<String, PlanTargets>,
    plan_paused: BTreeSet<String>,
    sleep_plans: Vec<(String, Value)>,
    plan_prompt: Option<bool>,
    /// Every date the imported settings cover, for the preview's range.
    dates: BTreeSet<String>,
}

#[derive(Default)]
struct Prepared {
    rows: Vec<ImportRow>,
    skipped: Vec<ImportSkip>,
}

#[derive(Default, Clone, Copy)]
struct Counts {
    added: u32,
    replaced: u32,
    kept: u32,
}

impl Counts {
    fn row(self, kind: &str) -> Option<ImportRow> {
        (self.added + self.replaced + self.kept > 0).then(|| ImportRow {
            kind: kind.to_owned(),
            added: self.added,
            replaced: self.replaced,
            kept: self.kept,
        })
    }
}

/// Validate an archive and report what importing it would change, without
/// writing anything. `account` is the verified active account.
pub fn preview(store: &Store, path: &Path, account: Option<&str>) -> Result<ImportPreview> {
    let account = account.ok_or(ImportError::NotSignedIn)?;
    let archive = read(path)?;
    if archive.account != account {
        return Err(ImportError::AccountMismatch);
    }
    let prepared = store.settings_read(|tx| run(tx, &archive, false))?;
    Ok(summary(&archive, prepared))
}

/// Validate an archive and apply it in one transaction.
pub fn apply(store: &Store, path: &Path, account: &str) -> Result<ImportPreview> {
    let archive = read(path)?;
    if archive.account != account {
        return Err(ImportError::AccountMismatch);
    }
    let prepared = store.settings_tx(|tx| run(tx, &archive, true))?;
    Ok(summary(&archive, prepared))
}

fn summary(archive: &Archive, prepared: Prepared) -> ImportPreview {
    ImportPreview {
        exported_at: archive.exported_at.clone(),
        app_version: archive.app_version.clone(),
        from: archive.dates.first().cloned(),
        to: archive.dates.last().cloned(),
        rows: prepared.rows,
        skipped: prepared.skipped,
    }
}

/// The diff, and with `write` the write, for one consistent settings
/// transaction. Everything the import changes happens here or not at all.
fn run(tx: &SettingsTx, archive: &Archive, write: bool) -> Result<Prepared> {
    let current = local::load_tx(tx)?;
    let (changes, read_only) = if write {
        local::import(tx, &current, &archive.records)?
    } else {
        let (_, rows, skipped) = local::merge(&current, &archive.records);
        (rows, skipped)
    };
    let mut prepared = Prepared::default();
    prepared.rows.extend(changes.into_iter().map(|row| ImportRow {
        kind: row.kind.to_owned(),
        added: row.added,
        replaced: row.replaced,
        kept: row.kept,
    }));
    for key in read_only {
        prepared.skipped.push(ImportSkip {
            what: record_label(key).to_owned(),
            reason: "This phone's copy is written by a newer version of Raring, so it was left alone.".to_owned(),
        });
    }

    if !archive.plan_versions.is_empty() || !archive.plan_paused.is_empty() {
        let stored = tx.get("plan")?;
        if stored.as_deref().is_some_and(|raw| serde_json::from_str::<PlanRow>(raw).is_err()) {
            prepared.skipped.push(ImportSkip {
                what: "Weekly plan history".to_owned(),
                reason: "This phone's row is written by a newer version of Raring, so it was left alone.".to_owned(),
            });
        } else {
            let mut row: PlanRow = stored.as_deref().and_then(|raw| serde_json::from_str(raw).ok()).unwrap_or_default();
            let mut counts = Counts::default();
            for (week, targets) in &archive.plan_versions {
                if row.versions.contains_key(week) {
                    counts.kept += 1;
                } else {
                    row.versions.insert(week.clone(), targets.clone());
                    counts.added += 1;
                }
            }
            for week in &archive.plan_paused {
                if row.paused.contains(week) {
                    counts.kept += 1;
                } else {
                    row.paused.insert(week.clone());
                    counts.added += 1;
                }
            }
            if write && counts.added > 0 {
                tx.set("plan", &serde_json::to_string(&row)?)?;
            }
            prepared.rows.extend(counts.row("Weekly plan history"));
        }
    }

    let mut sleep = Counts::default();
    for (key, value) in &archive.sleep_plans {
        if tx.get(key)?.is_some() {
            // A night's saved plan is prospective evidence and never replaced.
            sleep.kept += 1;
        } else {
            sleep.added += 1;
            if write {
                tx.set(key, &value.to_string())?;
            }
        }
    }
    prepared.rows.extend(sleep.row("Sleep plan snapshots"));

    if let Some(prompt) = archive.plan_prompt {
        // The prompt preference fills a phone that never saved it; this
        // phone's own choice is never overwritten.
        let stored = tx.get("settings")?;
        let mut prompt_row = Counts::default();
        match stored {
            Some(_) => prompt_row.kept += 1,
            None => {
                prompt_row.added += 1;
                if write {
                    let settings = Settings { plan_prompt: prompt, ..Settings::default() };
                    tx.set("settings", &serde_json::to_string(&settings)?)?;
                }
            }
        }
        prepared.rows.extend(prompt_row.row("Plan prompt preference"));
    }

    prepared.skipped.push(ImportSkip {
        what: "Garmin data and journal logs".to_owned(),
        reason: "Not imported: a sync downloads Garmin data for this phone, and journal reads stay this phone's own choice.".to_owned(),
    });
    prepared.skipped.push(ImportSkip {
        what: "Milestones and records".to_owned(),
        reason: "Not imported: they are awarded again from this phone's Garmin recordings.".to_owned(),
    });
    prepared.skipped.push(ImportSkip {
        what: "Notification settings".to_owned(),
        reason: "Not imported: this phone keeps its own choices.".to_owned(),
    });
    Ok(prepared)
}

fn record_label(key: &str) -> &'static str {
    match key {
        "local:goals" => "Goals",
        "local:intentions" => "Your sessions",
        "local:links" => "Plan links",
        "local:circumstances" => "Rest, pauses and limits",
        "local:reviews" => "Plan-week prompts",
        "local:events" => "Events",
        "local:routines" => "Routines",
        _ => "A local record",
    }
}

fn unreadable(what: impl Into<String>) -> ImportError {
    ImportError::Unreadable(what.into())
}

fn entry(zip: &mut ZipArchive<File>, name: &str, limit: u64) -> Result<Value> {
    let file = zip.by_name(name).map_err(|_| ImportError::NotAnArchive)?;
    let mut body = String::new();
    file.take(limit + 1)
        .read_to_string(&mut body)
        .map_err(|_| unreadable(format!("The export's {name} is not readable text.")))?;
    if body.len() as u64 > limit {
        return Err(unreadable(format!("The export's {name} is too large.")));
    }
    serde_json::from_str(&body).map_err(|_| unreadable(format!("The export's {name} is not readable JSON.")))
}

fn read(path: &Path) -> Result<Archive> {
    let mut zip = ZipArchive::new(File::open(path)?).map_err(|_| ImportError::NotAnArchive)?;
    let manifest = entry(&mut zip, "manifest.json", MAX_MANIFEST)?;
    let archive = &manifest["archive"];
    if archive["format"] != "raring-export" {
        return Err(ImportError::NotAnArchive);
    }
    let version = archive["version"].as_u64().ok_or(ImportError::NotAnArchive)?;
    if version > FORMAT_VERSION {
        return Err(ImportError::NewerExport);
    }
    if version != FORMAT_VERSION {
        return Err(ImportError::NotAnArchive);
    }
    let account = archive["account"]
        .as_str()
        .filter(|id| account::valid_identity(id))
        .ok_or(ImportError::MissingAccount)?
        .to_owned();
    let settings = entry(&mut zip, "settings.json", MAX_SETTINGS)?;
    let settings = settings
        .as_object()
        .ok_or_else(|| unreadable("The export's settings are not a JSON object."))?
        .clone();
    validate(settings, Manifest {
        account,
        exported_at: manifest["exported_at"].as_str().map(str::to_owned),
        app_version: manifest["app_version"].as_str().map(str::to_owned),
    })
}

/// A settings row is refused outright if it carries a credential at any depth:
/// an export never contains one, so such a file was altered after export.
fn credential_field(value: &Value) -> bool {
    match value {
        Value::Object(map) => map.iter().any(|(key, child)| export::credential_key(key) || credential_field(child)),
        Value::Array(items) => items.iter().any(credential_field),
        _ => false,
    }
}

fn validate(settings: Map<String, Value>, manifest: Manifest) -> Result<Archive> {
    let mut archive = Archive {
        account: manifest.account,
        exported_at: manifest.exported_at,
        app_version: manifest.app_version,
        ..Archive::default()
    };
    for (key, value) in &settings {
        if export::credential_key(key) || credential_field(value) {
            return Err(ImportError::Credentials);
        }
        match key.as_str() {
            "local:schema" => match value.as_u64() {
                Some(version) if version == local::SCHEMA => {}
                Some(version) if version > local::SCHEMA => return Err(ImportError::NewerExport),
                _ => return Err(unreadable("The export's local records are not one this version understands.")),
            },
            "local:goals" => {
                let goals: local::Goals = record(value, "goals", &["versions", "observationExplained"])?;
                check_goals(&goals, &mut archive.dates)?;
                archive.records.goals = Some(goals);
            }
            "local:intentions" => {
                let intentions: local::Intentions = record(value, "sessions", &["items"])?;
                check_intentions(&intentions, &mut archive.dates)?;
                archive.records.intentions = Some(intentions);
            }
            "local:links" => {
                let links: local::Links = record(value, "plan links", &["links", "unlinks"])?;
                check_links(&links)?;
                archive.records.links = Some(links);
            }
            "local:circumstances" => {
                let circumstances: local::Circumstances = record(value, "circumstances", &["rest", "pauses", "limits", "dismissals", "windDowns", "todayChoice"])?;
                check_circumstances(&circumstances, &mut archive.dates)?;
                archive.records.circumstances = Some(circumstances);
            }
            "local:reviews" => {
                let reviews: local::Reviews = record(value, "plan-week prompts", &["prompts", "decisions"])?;
                check_reviews(&reviews, &mut archive.dates)?;
                archive.records.reviews = Some(reviews);
            }
            "local:events" => {
                let events: local::Events = record(value, "events", &["events"])?;
                check_events(&events, &mut archive.dates)?;
                archive.records.events = Some(events);
            }
            "local:routines" => {
                let routines: local::Routines = record(value, "routines", &["items", "reports"])?;
                check_routines(&routines, &mut archive.dates)?;
                archive.records.routines = Some(routines);
            }
            // Milestones are rebuilt from this phone's Garmin recordings.
            "local:milestones" => {}
            "plan" => {
                let (versions, paused) = plan::legacy(&value.to_string())
                    .ok_or_else(|| unreadable("The export's weekly plan is not one this version understands."))?;
                for week in versions.keys().chain(paused.iter()) {
                    check_date(week, "a weekly plan week")?;
                    archive.dates.insert(week.clone());
                }
                archive.plan_versions = versions;
                archive.plan_paused = paused;
            }
            "settings" => {
                let object = object(value, "the export's settings")?;
                archive.plan_prompt = match object.get("planPrompt") {
                    None => None,
                    Some(prompt) => Some(prompt.as_bool().ok_or_else(|| unreadable("The export's plan prompt preference is not a switch."))?),
                };
            }
            key if key.starts_with("local:") => return Err(ImportError::NewerRow(key.to_owned())),
            key if key.starts_with("sleep_plan:") => {
                let night = key.strip_prefix("sleep_plan:").unwrap_or_default();
                check_date(night, "a saved sleep plan")?;
                if !plan::saved_night_ok(&value.to_string()) {
                    return Err(unreadable("The export's saved sleep plan is not one this version understands."));
                }
                archive.dates.insert(night.to_owned());
                archive.sleep_plans.push((key.to_owned(), value.clone()));
            }
            // Account markers, cursors and other device-only rows are never imported.
            _ => {}
        }
    }
    Ok(archive)
}

fn object<'a>(value: &'a Value, what: &str) -> Result<&'a Map<String, Value>> {
    value.as_object().ok_or_else(|| unreadable(format!("The export's {what} is not a JSON object.")))
}

/// A versioned record row: this build's schema, only fields it knows, and a
/// body that deserializes.
fn record<T: DeserializeOwned>(value: &Value, what: &str, fields: &[&str]) -> Result<T> {
    let object = object(value, what)?;
    match object.get("schema").and_then(Value::as_u64) {
        Some(version) if version == local::SCHEMA => {}
        Some(version) if version > local::SCHEMA => return Err(ImportError::NewerRow(what.to_owned())),
        _ => return Err(unreadable(format!("The export's {what} is not one this version understands."))),
    }
    if let Some(field) = object.keys().find(|key| key.as_str() != "schema" && !fields.contains(&key.as_str())) {
        return Err(unreadable(format!("The export's {what} has a field ({field}) this version doesn't know.")));
    }
    serde_json::from_value(value.clone()).map_err(|_| unreadable(format!("The export's {what} is not readable.")))
}

fn check_date(value: &str, what: &str) -> Result<()> {
    (value.len() == 10 && NaiveDate::parse_from_str(value, "%Y-%m-%d").is_ok())
        .then_some(())
        .ok_or_else(|| unreadable(format!("The export's {what} has a date this version can't read.")))
}

fn check_stamp(value: &str, what: &str) -> Result<()> {
    let ok = NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S").is_ok()
        || NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S%.f").is_ok();
    ok.then_some(())
        .ok_or_else(|| unreadable(format!("The export's {what} has a time this version can't read.")))
}

fn check_text(value: &str, max: usize, what: &str) -> Result<()> {
    (!value.trim().is_empty() && value.chars().count() <= max)
        .then_some(())
        .ok_or_else(|| unreadable(format!("The export's {what} has text this version can't read.")))
}

fn check_optional(value: &Option<String>, max: usize, what: &str) -> Result<()> {
    match value {
        Some(text) if text.chars().count() > max => Err(unreadable(format!("The export's {what} has text this version can't read."))),
        _ => Ok(()),
    }
}

fn check_time(value: &Option<String>, what: &str) -> Result<()> {
    match value {
        Some(time) if time.len() != 5 || NaiveTime::parse_from_str(time, "%H:%M").is_err() => {
            Err(unreadable(format!("The export's {what} has a time of day this version can't read.")))
        }
        _ => Ok(()),
    }
}

fn check_targets(targets: &GoalTargets) -> Result<()> {
    if targets.strength_sessions > 14
        || [targets.zone2_min, targets.high_intensity_min, targets.intensity_min].iter().any(|m| *m > 3000)
        || targets.sleep_nights > 7
        || targets.sleep_score > 100
    {
        return Err(unreadable("The export's weekly goals are out of range."));
    }
    check_time(&targets.sleep_time, "weekly goals")?;
    if targets.rest_days.iter().any(|day| *day > 6) || targets.rest_days.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(unreadable("The export's weekly goals have rest days this version can't read."));
    }
    Ok(())
}

fn check_allocation(allocation: &Allocation) -> Result<()> {
    if allocation.strength_sessions > 3
        || [allocation.zone2_min, allocation.high_intensity_min, allocation.intensity_min].iter().any(|m| *m > 600)
        || allocation.sleep_nights > 1
    {
        return Err(unreadable("The export's session allocation is out of range."));
    }
    Ok(())
}

fn check_goals(goals: &local::Goals, dates: &mut BTreeSet<String>) -> Result<()> {
    let mut seen = BTreeSet::new();
    for version in &goals.versions {
        check_date(&version.from, "a weekly goal")?;
        check_stamp(&version.saved_at, "a weekly goal")?;
        check_targets(&version.targets)?;
        if !seen.insert(version.from.as_str()) {
            return Err(unreadable("The export's weekly goals repeat a date."));
        }
        dates.insert(version.from.clone());
    }
    Ok(())
}

fn check_intentions(intentions: &local::Intentions, dates: &mut BTreeSet<String>) -> Result<()> {
    let mut seen = BTreeSet::new();
    for item in &intentions.items {
        check_text(&item.id, 100, "sessions")?;
        if !seen.insert(item.id.as_str()) {
            return Err(unreadable("The export's sessions repeat an id."));
        }
        check_date(&item.date, "sessions")?;
        check_time(&item.time, "sessions")?;
        if item.duration_min.is_some_and(|minutes| minutes == 0 || minutes > 600) {
            return Err(unreadable("The export's sessions have a duration out of range."));
        }
        check_allocation(&item.allocation)?;
        check_optional(&item.type_key, 60, "sessions")?;
        check_optional(&item.name, 60, "sessions")?;
        check_optional(&item.garmin_id, 100, "sessions")?;
        for event in &item.history {
            check_stamp(&event.at, "sessions")?;
            for day in [&event.from, &event.to].into_iter().flatten() {
                check_date(day, "sessions")?;
            }
        }
        dates.insert(item.date.clone());
    }
    Ok(())
}

fn check_links(links: &local::Links) -> Result<()> {
    let mut plans = BTreeSet::new();
    for link in &links.links {
        check_text(&link.plan_id, 120, "plan links")?;
        check_text(&link.activity_id, 100, "plan links")?;
        check_stamp(&link.at, "plan links")?;
        if !plans.insert(link.plan_id.as_str()) {
            return Err(unreadable("The export's plan links repeat a session."));
        }
    }
    let mut pairs = BTreeSet::new();
    for link in &links.unlinks {
        check_text(&link.plan_id, 120, "plan links")?;
        check_text(&link.activity_id, 100, "plan links")?;
        check_stamp(&link.at, "plan links")?;
        if !pairs.insert((link.plan_id.as_str(), link.activity_id.as_str())) {
            return Err(unreadable("The export's unlinks repeat a pair."));
        }
    }
    Ok(())
}

fn check_circumstances(circumstances: &local::Circumstances, dates: &mut BTreeSet<String>) -> Result<()> {
    let mut rest = BTreeSet::new();
    for day in &circumstances.rest {
        check_date(&day.date, "rest days")?;
        check_stamp(&day.at, "rest days")?;
        if !rest.insert(day.date.as_str()) {
            return Err(unreadable("The export's rest days repeat a date."));
        }
        dates.insert(day.date.clone());
    }
    let mut pauses = BTreeSet::new();
    for pause in &circumstances.pauses {
        check_text(&pause.id, 100, "pauses")?;
        check_date(&pause.since, "pauses")?;
        dates.insert(pause.since.clone());
        if !pauses.insert(pause.id.as_str()) {
            return Err(unreadable("The export's pauses repeat an id."));
        }
        if let Some(until) = &pause.until {
            check_date(until, "pauses")?;
            if until < &pause.since {
                return Err(unreadable("The export's pauses end before they start."));
            }
            dates.insert(until.clone());
        }
        if let Some(review) = &pause.review {
            check_date(review, "pauses")?;
        }
    }
    let excluded: BTreeSet<_> = circumstances.limits.excluded.iter().collect();
    if excluded.len() != circumstances.limits.excluded.len() {
        return Err(unreadable("The export's limits repeat a sport."));
    }
    if circumstances.limits.note.chars().count() > 500 {
        return Err(unreadable("The export's limits note is too long."));
    }
    let mut dismissals = BTreeSet::new();
    for dismissal in &circumstances.dismissals {
        check_text(&dismissal.id, 100, "dismissals")?;
        check_date(&dismissal.date, "dismissals")?;
        if !dismissals.insert((dismissal.id.as_str(), dismissal.date.as_str())) {
            return Err(unreadable("The export's dismissals repeat a suggestion."));
        }
        dates.insert(dismissal.date.clone());
    }
    let mut reports = BTreeSet::new();
    for report in &circumstances.wind_downs {
        check_date(&report.date, "wind-down reports")?;
        check_stamp(&report.at, "wind-down reports")?;
        if !reports.insert(report.date.as_str()) {
            return Err(unreadable("The export's wind-down reports repeat a date."));
        }
        dates.insert(report.date.clone());
    }
    if let Some(choice) = &circumstances.today_choice {
        check_date(&choice.date, "today's kept session")?;
        check_text(&choice.plan_id, 120, "today's kept session")?;
        dates.insert(choice.date.clone());
    }
    Ok(())
}

fn check_reviews(reviews: &local::Reviews, dates: &mut BTreeSet<String>) -> Result<()> {
    for week in reviews.prompts.keys() {
        check_date(week, "plan-week prompts")?;
        dates.insert(week.clone());
    }
    let mut weeks = BTreeSet::new();
    for decision in &reviews.decisions {
        check_date(&decision.week, "plan-week prompts")?;
        check_stamp(&decision.at, "plan-week prompts")?;
        if !weeks.insert(decision.week.as_str()) {
            return Err(unreadable("The export's plan-week prompts repeat a week."));
        }
        dates.insert(decision.week.clone());
    }
    Ok(())
}

fn check_events(events: &local::Events, dates: &mut BTreeSet<String>) -> Result<()> {
    let mut seen = BTreeSet::new();
    for event in &events.events {
        check_text(&event.id, 100, "events")?;
        if !seen.insert(event.id.as_str()) {
            return Err(unreadable("The export's events repeat an id."));
        }
        check_text(&event.name, 60, "events")?;
        check_date(&event.date, "events")?;
        dates.insert(event.date.clone());
    }
    Ok(())
}

fn check_routines(routines: &local::Routines, dates: &mut BTreeSet<String>) -> Result<()> {
    if routines.items.len() > 30 {
        return Err(unreadable("The export has more checklist items than this version keeps."));
    }
    let mut items = BTreeSet::new();
    for item in &routines.items {
        check_text(&item.id, 100, "routines")?;
        if !items.insert(item.id.as_str()) {
            return Err(unreadable("The export's routines repeat an id."));
        }
        check_text(&item.name, 80, "routines")?;
        check_optional(&item.how, 300, "routines")?;
    }
    let mut reports = BTreeSet::new();
    for report in &routines.reports {
        if !items.contains(report.id.as_str()) {
            return Err(unreadable("The export's routine reports name an item it doesn't have."));
        }
        check_date(&report.date, "routines")?;
        check_stamp(&report.at, "routines")?;
        if !reports.insert((report.id.as_str(), report.date.as_str())) {
            return Err(unreadable("The export's routine reports repeat a day."));
        }
        dates.insert(report.date.clone());
    }
    Ok(())
}
