//! Shared, offline-first Garmin core. Callers own and persist credentials.
pub mod account;
pub mod day;
pub mod lanes;
pub mod demo;
mod export;
pub mod garmin;
mod gear;
pub mod import;
pub mod lead;
pub mod ledger;
pub mod local;
pub mod notifications;
mod naps_journal;
pub mod phase;
pub mod plan;
pub mod progress;
pub mod review;
pub mod planned;
pub mod rules;
pub mod screens;
pub mod settings;
pub mod status;
pub mod store;
pub mod strength;
pub mod stress;
pub mod sync;
pub mod trends;
pub mod view;
pub mod weight;
pub mod writes;

use chrono::{DateTime, Local, NaiveDateTime, Timelike, Utc};
pub use garmin::Tokens;
pub use notifications::Notification;
use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
pub use sync::SyncReport;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Store(#[from] rusqlite::Error),
    #[error(transparent)]
    Garmin(#[from] garmin::Error),
    #[error("invalid setting name or value")]
    InvalidSetting,
    #[error("local change refused: {0}")]
    InvalidLocal(&'static str),
    #[error("status snapshot: {0}")]
    Snapshot(#[from] std::io::Error),
}
pub type Result<T> = std::result::Result<T, Error>;
pub struct Raring {
    store: Arc<store::Store>,
    snapshot_path: Option<PathBuf>,
    snapshot_state: Mutex<bool>, // true after sign-out; serializes snapshot writes with erasure
}
pub struct SyncOutcome {
    pub tokens: Tokens,
    pub report: SyncReport,
    pub notifications: Vec<Notification>,
    /// Local publication failed; retry it offline, never repeat Garmin requests for it.
    pub snapshot_error: Option<String>,
}
impl Raring {
    pub fn open(db_path: impl AsRef<Path>) -> Result<Self> {
        let path = db_path.as_ref();
        Ok(Self {
            store: Arc::new(store::Store::open(path)?),
            snapshot_path: (path != Path::new(":memory:"))
                .then(|| path.with_extension("status.json")),
            snapshot_state: Mutex::new(false),
        })
    }
    pub async fn refresh_tokens(tokens: &Tokens) -> Result<Tokens> {
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(20))
            .build()
            .map_err(|_| garmin::Error::Network)?;
        Ok(garmin::refresh(&http, tokens).await?)
    }
    pub async fn sign_in_with_ticket(ticket: &str) -> Result<Tokens> {
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|_| garmin::Error::Network)?;
        Ok(garmin::exchange_ticket(&http, ticket, garmin::sso::PORTAL_SERVICE_URL).await?)
    }
    /// Synthetic ticket-exchange endpoint for debug integration tests only.
    #[cfg(debug_assertions)]
    pub async fn sign_in_with_ticket_at(ticket: &str, token_url: &str) -> Result<Tokens> {
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build().map_err(|_| garmin::Error::Network)?;
        Ok(garmin::exchange_ticket_at(&http, token_url, ticket, garmin::sso::PORTAL_SERVICE_URL).await?)
    }
    pub async fn sync(
        &self,
        tokens: Tokens,
        now: DateTime<Local>,
        budget: usize,
    ) -> Result<SyncOutcome> {
        let client = garmin::Client::new(tokens)?;
        let store = Arc::clone(&self.store);
        // SQLite calls are short and individually mutex-protected. Network awaits never hold the DB lock.
        let mut report = sync::sync(&store, &client, now, budget).await;
        // A sync may take minutes; evidence was available only after it ended.
        {
            let _boundary = status::WRITE_LOCK.lock().unwrap();
            if plan::capture_sleep_plan(&store, Local::now()).is_err() {
                report.error = Some("Could not save tonight's plan".into());
            }
        }
        let tokens = client.tokens().await;
        let snapshot_error = self.refresh_status_snapshot(Local::now()).err().map(|e| e.to_string());
        let notifications = {
            let _boundary = status::WRITE_LOCK.lock().unwrap();
            notifications::notifications(&store, now)
        };
        Ok(SyncOutcome {
            tokens,
            report,
            notifications,
            snapshot_error,
        })
    }
    /// Capture only a live prospective plan; historical view reads stay pure.
    pub fn capture_sleep_plan(&self, now: DateTime<Local>) -> Result<()> {
        plan::capture_sleep_plan(&self.store, now)?;
        Ok(())
    }
    pub fn current_date(&self, now: DateTime<Local>) -> String {
        screens::current_date(&self.store, now.naive_local()).to_string()
    }
    pub fn today(&self, date: &str) -> view::TodayView {
        screens::today(&self.store, date)
    }
    /// Today as of `now`, which sets the time-of-day phase.
    pub fn today_at(&self, date: &str, now: DateTime<Local>) -> view::TodayView {
        screens::today_at(&self.store, date, now.fixed_offset())
    }
    /// The live Today, with its lead and today's circumstances applied.
    pub fn today_live_at(&self, display_date: &str, now: DateTime<Local>) -> view::TodayView {
        let mut view = screens::today_at_calendar(&self.store, display_date, now.fixed_offset(), now.date_naive());
        lead::apply(&self.store, &mut view, now.date_naive(), now.fixed_offset());
        view.day = Some(day::timeline(&self.store, &view, now.date_naive(), now.fixed_offset(), day::day_minutes(now.date_naive())));
        view
    }
    pub fn sleep(&self, date: &str) -> view::SleepView {
        screens::sleep_view(&self.store, date)
    }
    /// The current Sleep: Tonight first from the evening (Today's phase
    /// rules), and whether tonight's wind-down was reported.
    pub fn sleep_live(&self, now: DateTime<Local>) -> view::SleepView {
        let date = self.current_date(now);
        let mut view = self.sleep(&date);
        let phase = self.today_at(&date, now).focus.phase;
        view.tonight_first = matches!(phase, view::DayPhase::Evening | view::DayPhase::WindDown | view::DayPhase::LateNight);
        let (evening, open) = self.wind_down_evening(now);
        view.wound_down = local::load(&self.store).circumstances.wind_downs.iter().any(|w| w.date == evening.to_string());
        view.wind_down_open = open;
        view
    }
    /// The evening whose wind-down Tonight shows: the evening before its
    /// plan's night ends. It can be reported from two hours before that
    /// night's window (the evening phase's start) until the plan moves on.
    /// Without a plan, a report before noon belongs to the evening before.
    fn wind_down_evening(&self, now: DateTime<Local>) -> (chrono::NaiveDate, bool) {
        let planned = screens::sleep_plan(&self.store, &self.current_date(now).to_string()).and_then(|plan| {
            let night = chrono::NaiveDate::parse_from_str(&plan.night, "%Y-%m-%d").ok()?;
            let start = plan.bedtime_start_at.as_deref().unwrap_or(&plan.bedtime_at);
            let start = NaiveDateTime::parse_from_str(start, "%Y-%m-%dT%H:%M:%S").ok()?;
            Some((night - chrono::Duration::days(1), now.naive_local() >= start - chrono::Duration::minutes(phase::PROMOTE_MIN)))
        });
        planned.unwrap_or_else(|| (if now.hour() < 12 { now.date_naive() - chrono::Duration::days(1) } else { now.date_naive() }, true))
    }
    pub fn training(&self, date: &str) -> view::TrainingView {
        screens::training(&self.store, date)
    }
    pub fn training_live_at(&self, display_date: &str, now: DateTime<Local>) -> view::TrainingView {
        let mut view = screens::training_at_calendar(&self.store, display_date, now.date_naive(), now.with_timezone(&Utc));
        view.lanes = Some(lanes::week(&self.store, &local::load(&self.store), now.date_naive(), now.with_timezone(&Utc)));
        view
    }
    pub fn workout(&self, id: &str) -> Option<view::WorkoutView> {
        screens::workout(&self.store, id)
    }
    /// A workout with how it counted toward its plan week's goals.
    pub fn workout_live(&self, id: &str, now: DateTime<Local>) -> Option<view::WorkoutView> {
        let mut view = screens::workout(&self.store, id)?;
        let date = chrono::NaiveDate::parse_from_str(view.summary.start_local.get(..10)?, "%Y-%m-%d").ok()?;
        let local = local::load(&self.store);
        let board = ledger::board_with(&self.store, &local, date, now.date_naive(), now.with_timezone(&Utc));
        let rec = board.recordings.iter().find(|r| r.id == id);
        let lines = board
            .ledger
            .iter()
            .filter(|l| !l.observation)
            .filter_map(|l| {
                let rec = rec?;
                let (amount, position) = match l.kind {
                    view::GoalKind::Strength => (rec.sport == view::Sport::Strength).then(|| {
                        let n = board.recordings.iter().filter(|r| r.sport == view::Sport::Strength && r.start_local <= rec.start_local).count();
                        (1., Some(n as u32))
                    })?,
                    view::GoalKind::Zone2 => (rec.zone2_min.filter(|m| *m > 0.)?, None),
                    view::GoalKind::HighIntensity => (rec.high_intensity_min.filter(|m| *m > 0.)?, None),
                    _ => return None,
                };
                Some(view::CountLine { kind: l.kind, amount, position, recorded: l.recorded, target: l.target })
            })
            .collect();
        view.counted = Some(view::WorkoutCount {
            week: board.start.clone(),
            goals: board.ledger.iter().any(|l| !l.observation),
            lines,
            plan_item: board.items.iter().find(|p| p.recording.as_deref() == Some(id)).map(|p| p.intention.name.clone().unwrap_or_else(|| lead::sport_name(p.intention.sport).to_owned())),
            bests: local.milestones.items.iter().filter(|m| m.activity.as_deref() == Some(id) && !m.withdrawn).map(|m| m.title.clone()).collect(),
        });
        Some(view)
    }
    /// Older stored recordings for Training → Activities, from local data only.
    pub fn activity_page(&self, before_start: &str, before_id: &str, icon: Option<view::Icon>) -> view::ActivityPage {
        screens::activity_page(&self.store, before_start, before_id, icon, 20)
    }
    /// Sets per muscle group over a range, from local sets only.
    pub fn muscles(&self, range: view::TrendRange, now: DateTime<Local>) -> Option<view::MuscleRange> {
        strength::muscles(&self.store, range, now.date_naive())
    }
    /// One exercise's sessions in a range, from local sets only.
    pub fn lift_series(&self, key: &str, range: view::TrendRange, now: DateTime<Local>) -> Option<view::LiftSeries> {
        strength::lift_series(&self.store, key, range, now.date_naive())
    }
    pub fn strength(&self, date: &str) -> Option<view::StrengthView> {
        screens::strength(&self.store, date)
    }
    pub fn health(&self, date: &str) -> view::HealthView {
        screens::health(&self.store, date)
    }
    pub fn health_live_at(&self, display_date: &str, now: DateTime<Local>) -> view::HealthView {
        screens::health_at_calendar(&self.store, display_date, &now.date_naive().to_string())
    }
    pub fn heart_rate_day(&self, date: &str) -> view::HeartRateDay {
        screens::heart_rate_day(&self.store, date)
    }
    pub fn cached_heart_rate_day(&self, date: &str) -> Option<view::HeartRateDay> {
        self.store.get("daily_hr", date).map(|_| self.heart_rate_day(date))
    }
    pub async fn fetch_heart_rate_day(
        &self,
        tokens: Tokens,
        date: &str,
    ) -> Result<(view::HeartRateDay, Tokens, SyncReport)> {
        let client = garmin::Client::new(tokens)?;
        let mut report = sync::heart_rate_on_demand(&self.store, &client, date).await;
        if report.rate_limited {
            report.error = Some("Garmin is rate limiting reads; try again later".into());
        } else if !report.auth_failed && report.error.is_none() && self.cached_heart_rate_day(date).is_none() {
            report.error = Some("Heart rate not available yet; try again later".into());
        }
        Ok((self.heart_rate_day(date), client.tokens().await, report))
    }
    /// One explicitly opened past day's missing detail, kept for later reads.
    pub async fn fetch_day_detail(
        &self,
        tokens: Tokens,
        date: &str,
        parts: &[view::DayDetailPart],
    ) -> Result<(Tokens, SyncReport)> {
        let client = garmin::Client::new(tokens)?;
        let mut report = sync::day_detail_on_demand(&self.store, &client, date, parts).await;
        if report.rate_limited {
            report.error = Some("Garmin is rate limiting reads; try again later".into());
        }
        Ok((client.tokens().await, report))
    }
    /// One explicitly opened workout's missing summary, charts, splits and zones.
    pub async fn fetch_workout_detail(
        &self,
        tokens: Tokens,
        id: &str,
    ) -> Result<(Tokens, SyncReport)> {
        let client = garmin::Client::new(tokens)?;
        let mut report = sync::workout_detail_on_demand(&self.store, &client, id).await;
        if report.rate_limited {
            report.error = Some("Garmin is rate limiting reads; try again later".into());
        }
        Ok((client.tokens().await, report))
    }
    /// One explicit read of a strength workout's sets, with the updated detail.
    /// Navigation never calls this; only a person asking for a Garmin refresh does.
    pub async fn refresh_strength_sets(
        &self,
        tokens: Tokens,
        id: &str,
    ) -> Result<(Option<view::StrengthSession>, Tokens, SyncReport)> {
        let client = garmin::Client::new(tokens)?;
        let mut report = sync::refresh_strength_sets(&self.store, &client, id).await;
        if report.rate_limited {
            report.error = Some("Garmin is rate limiting reads; try again later".into());
        }
        Ok((strength::session(&self.store, id), client.tokens().await, report))
    }
    pub fn checkin(&self, monday: &str) -> view::CheckinView {
        screens::checkin(&self.store, monday)
    }
    /// A trend chart's points; `today` is the default last day.
    pub fn trend(&self, query: &view::TrendQuery, today: &str) -> std::result::Result<view::TrendView, String> {
        trends::trend(&self.store, query, today)
    }
    pub fn plan(&self, date: &str) -> view::PlanView {
        plan::view(&self.store, date)
    }
    /// Saves this week's targets; `today` picks the week.
    pub fn set_plan(&self, today: &str, update: &view::PlanUpdate) -> Result<()> {
        let today = chrono::NaiveDate::parse_from_str(today, "%Y-%m-%d")
            .map_err(|_| Error::InvalidSetting)?;
        plan::set(&self.store, today, update)
    }
    /// The plan week containing `date`, measured on the phone's calendar day.
    pub fn plan_board(&self, date: &str, now: DateTime<Local>) -> Result<view::PlanBoard> {
        let anchor = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").map_err(|_| Error::InvalidLocal("date"))?;
        let mut board = ledger::board(&self.store, anchor, now.date_naive(), now.with_timezone(&Utc));
        // Never in wind-down or late night, whichever page asks (Rules: Plan prompt).
        if board.prompt.is_some() {
            let phase = self.today_at(&self.current_date(now), now).focus.phase;
            if matches!(phase, view::DayPhase::WindDown | view::DayPhase::LateNight) {
                board.prompt = None;
            }
        }
        Ok(board)
    }
    /// Garmin facts, gear and your limits, from local data only.
    pub fn profile(&self, now: DateTime<Local>) -> view::ProfileView {
        let today = now.date_naive();
        let weight = weight::view(&self.store, view::TrendRange::All, today, today).ok();
        let (_, uploaded) = screens::watch_info(&self.store);
        view::ProfileView {
            age: screens::age(&self.store, &today.to_string()).chronological_age,
            units: self.store.get("profile_settings", "me").and_then(|s| s["measurementSystem"].as_str().map(str::to_owned)),
            weight: weight.as_ref().and_then(|w| w.scale),
            weight_unit: weight.as_ref().and_then(|w| w.unit),
            weight_freshness: weight.map(|w| w.freshness).unwrap_or(view::ReadingFreshness { recorded_on: None, age_days: None, stale: false, threshold_days: 7 }),
            watch_last_upload: uploaded.map(|t| t.to_rfc3339_opts(chrono::SecondsFormat::Millis, true)),
            gear: gear::view(&self.store),
            limits: local::load(&self.store).circumstances.limits,
        }
    }
    /// Scale and trend weight from local history only; never a Garmin request.
    pub fn weight_trend(&self, range: view::TrendRange, date: &str, now: DateTime<Local>) -> Result<view::WeightView> {
        let end = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").map_err(|_| Error::InvalidLocal("weight date"))?;
        weight::view(&self.store, range, end, now.date_naive())
    }
    /// The review of the plan week containing `date`, with next week's proposals.
    pub fn week_review(&self, date: &str, now: DateTime<Local>) -> Result<view::WeekReview> {
        let anchor = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").map_err(|_| Error::InvalidLocal("date"))?;
        Ok(review::week_review(&self.store, anchor, now.date_naive(), now.with_timezone(&chrono::Utc)))
    }
    /// Progress for the period containing `date`.
    /// Every milestone in available history, newest first.
    pub fn milestone_archive(&self, now: DateTime<Local>) -> Result<Vec<view::Milestone>> {
        progress::milestone_archive(&self.store, now.date_naive(), now.with_timezone(&chrono::Utc))
    }
    /// Progress's Cardio, Strength and Long-term views, from local data only.
    pub fn progress_domains(&self, now: DateTime<Local>) -> view::ProgressDomains {
        progress::domains(&self.store, now.date_naive(), now.with_timezone(&chrono::Utc))
    }
    pub fn progress(&self, period: view::Period, date: &str, now: DateTime<Local>) -> Result<view::ProgressView> {
        let anchor = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").map_err(|_| Error::InvalidLocal("date"))?;
        progress::progress(&self.store, period, anchor, now.date_naive(), now.with_timezone(&chrono::Utc))
    }
    /// Reviewed weeks with goals, newest first.
    pub fn reviews(&self, now: DateTime<Local>) -> Vec<view::ReviewRow> {
        review::list(&self.store, now.date_naive(), 26, now.with_timezone(&chrono::Utc))
    }
    /// Goals, sessions, links, circumstances, prompts and events, migrated.
    pub fn local_records(&self) -> local::LocalRecords {
        local::load(&self.store)
    }
    /// One local change, for fixtures and the CLI.
    pub fn local_action(&self, action: &view::LocalAction, now: DateTime<Local>) -> Result<()> {
        self.save_local(None, action, now)
    }
    /// One local change from the UI, refused if the account changed since it was queued.
    pub fn local_action_for_account(&self, action: &view::LocalAction, now: DateTime<Local>, expected: Option<String>) -> Result<()> {
        self.save_local(Some(expected), action, now)
    }
    fn save_local(&self, expected: Option<Option<String>>, action: &view::LocalAction, now: DateTime<Local>) -> Result<()> {
        let signed_out = self.snapshot_state.lock().unwrap();
        let _write = status::WRITE_LOCK.lock().unwrap();
        if expected.is_some_and(|id| id != self.bound_account_identity()) {
            return Err(Error::InvalidLocal("the account changed"));
        }
        let first = plan::week_starts_on(&self.store);
        // Proposals are recomputed now, so only what the review offers can be accepted.
        let proposals = match action {
            view::LocalAction::DecideReview { week, .. } => chrono::NaiveDate::parse_from_str(week, "%Y-%m-%d")
                .map(|w| review::proposals(&self.store, w, now.date_naive(), now.with_timezone(&chrono::Utc)))
                .unwrap_or_default(),
            _ => vec![],
        };
        let evening = match action {
            view::LocalAction::ReportWindDown => match self.wind_down_evening(now) {
                (evening, true) => Some(evening),
                (_, false) => return Err(Error::InvalidLocal("wind-down opens two hours before tonight's window")),
            },
            _ => None,
        };
        local::apply(&self.store, action, now, first, &proposals, evening)?;
        // Surfaces follow rest, pauses and limits without waiting for a sync;
        // a failed publication is retried offline and never undoes the change.
        let _ = self.write_status_snapshot(now, *signed_out);
        Ok(())
    }
    /// Change a watch's alarms, only from the person's explicit action (Rules:
    /// Garmin writes). The caller holds its credential gate as for sync; refreshed
    /// tokens stay in `client`. Invalid edits and an unconfirmed delete are
    /// refused before any request; every other attempt is logged.
    pub async fn change_watch_alarms(
        &self,
        client: &garmin::Client,
        device_id: &str,
        edits: &[writes::AlarmEdit],
        confirm_delete: bool,
        now: DateTime<Utc>,
    ) -> Result<view::GarminChange> {
        let expected = self.session_identity().ok_or(Error::InvalidLocal("not signed in"))?;
        let (resource, edits) = writes::alarms(device_id, edits, confirm_delete)?;
        let entry = writes::write(&self.store, client, resource, edits, now).await;
        {
            let _write = status::WRITE_LOCK.lock().unwrap();
            // A write that outlived its account is not logged for the next one.
            if self.session_identity().as_deref() == Some(expected.as_str()) {
                // The outcome matters more than its log entry; a failed log save
                // still returns what Garmin did.
                let _ = writes::record(&self.store, &entry);
            }
        }
        Ok(writes::shown(&self.store, &entry))
    }
    /// Recent Garmin writes, newest first, from local data only.
    pub fn garmin_changes(&self) -> Vec<view::GarminChange> {
        writes::changes(&self.store)
    }
    pub fn status(&self) -> view::AppStatus {
        let (history_from, history_complete) =
            sync::history_progress(&self.store, Local::now().date_naive());
        view::AppStatus {
            history_from,
            history_complete,
            signed_in: self.store.get("profile", "me").is_some(),
            syncing: false,
            last_sync: self.store.cursor("last_sync"),
            history_days: self.store.days(),
            last_error: self.store.cursor("last_error").filter(|s| !s.is_empty()),
        }
    }
    /// Build a complete, token-free archive at a caller-owned private path.
    pub fn export(&self, path: &Path, app_version: &str) -> std::result::Result<(), export::ExportError> {
        export::write(&self.store, path, app_version)
    }
    /// Export only if the active account and cached profile belong to `expected`
    /// in the same SQLite snapshot used for all archive contents.
    pub fn export_for_account(&self, path: &Path, app_version: &str, expected: &str) -> std::result::Result<(), export::ExportError> {
        export::write_for_account(&self.store, path, app_version, expected)
    }
    /// Validate a chosen export and preview what importing it would change.
    /// Read-only, under the account barrier, and refused for another account.
    pub fn import_preview(&self, path: &Path, expected: Option<String>) -> std::result::Result<view::ImportPreview, import::ImportError> {
        let _write = status::WRITE_LOCK.lock().unwrap();
        let identity = self.session_identity();
        if expected.is_some() && expected != identity {
            return Err(import::ImportError::AccountChanged);
        }
        import::preview(&self.store, path, identity.as_deref())
    }
    /// Apply a chosen export once, in one transaction under the account
    /// barrier, and refresh the offline status snapshot afterwards.
    pub fn import_settings(&self, path: &Path, expected: Option<String>) -> std::result::Result<view::ImportPreview, import::ImportError> {
        let signed_out = self.snapshot_state.lock().unwrap();
        let _write = status::WRITE_LOCK.lock().unwrap();
        let identity = self.session_identity().ok_or(import::ImportError::NotSignedIn)?;
        if expected.is_some_and(|id| id != identity) {
            return Err(import::ImportError::AccountChanged);
        }
        let preview = import::apply(&self.store, path, &identity)?;
        // Imported rest, pauses and limits change the native surfaces; a
        // failed publication is retried offline and never undoes the import.
        let _ = self.write_status_snapshot(Local::now(), *signed_out);
        Ok(preview)
    }
    pub fn settings(&self) -> view::Settings {
        self.store.settings()
    }
    pub fn bound_account_identity(&self) -> Option<String> { account::identity(&self.store) }
    /// Local view construction may capture a prospective plan. Keep it in one account.
    pub fn with_verified_session<T>(&self, expected: &str, read: impl FnOnce() -> T) -> Option<T> {
        let _boundary = status::WRITE_LOCK.lock().unwrap();
        if self.session_identity().as_deref() != Some(expected) { return None; }
        Some(read())
    }
    /// Full replacement for explicit fixtures/CLI callers, never UI transport.
    pub fn set_settings(&self, settings: view::Settings) -> Result<()> {
        self.save_settings(None, |_| Ok(settings))
    }
    pub fn set_settings_for_account(&self, settings: view::Settings, expected: Option<String>) -> Result<()> {
        self.save_settings(Some(expected), |_| Ok(settings))
    }
    /// Merge only the requested fields after taking the account/write barrier.
    pub fn patch_settings_for_account(&self, patch: settings::SettingsPatch, expected: Option<String>) -> Result<()> {
        self.save_settings(Some(expected), |current| patch.apply(current))
    }
    fn save_settings(&self, expected: Option<Option<String>>, update: impl FnOnce(&view::Settings) -> Result<view::Settings>) -> Result<()> {
        let signed_out = self.snapshot_state.lock().unwrap();
        let _write = status::WRITE_LOCK.lock().unwrap();
        if expected.is_some_and(|id| id != self.bound_account_identity()) {
            return Err(Error::InvalidSetting);
        }
        let previous = self.store.settings();
        let mut settings = update(&previous)?;
        self.store.set_settings(&settings)?;
        if let Err(error) = self.write_status_snapshot(Local::now(), *signed_out) {
            // Only a new status opt-in depends on publishing its cache. Never
            // restore an older settings object: all other edits, particularly
            // revocations (including journalEnabled=false), stay
            // authoritative even if the native presentation cache cannot update.
            if settings.notify_status && !previous.notify_status {
                settings.notify_status = false;
                self.store.set_settings(&settings)?;
                return Err(error);
            }
        }
        Ok(())
    }
    /// Suspend presentation without discarding the account's history or local plans.
    pub fn suspend_session(&self) -> Result<()> {
        let _write = status::WRITE_LOCK.lock().unwrap();
        self.store.suspend_account()?;
        if let Some(path) = &self.snapshot_path { let _ = status::erase(path); }
        Ok(())
    }
    #[cfg(debug_assertions)]
    pub fn is_demo_session(&self) -> bool {
        let profile = self.store.get("profile", "me");
        self.session_identity().as_deref() == Some("900000000001")
            && profile.is_some_and(|p| p["displayName"] == "sample-demo")
    }
    pub fn legacy_session_needs_verification(&self) -> bool {
        self.store.setting("account_session").is_none() && self.store.get("profile", "me").is_some()
    }
    pub fn session_identity(&self) -> Option<String> {
        account::active(&self.store).then(|| account::identity(&self.store)).flatten()
    }
    /// Caller holds its credential gate across verification, vault save and activation.
    pub async fn verify_account(&self, client: &garmin::Client, now: chrono::DateTime<chrono::Utc>) -> Result<()> {
        account::verify(&self.store, client, now).await
    }
    pub fn activate_session(&self) -> Result<()> {
        let _write = status::WRITE_LOCK.lock().unwrap();
        if !self.store.activate_account()? { return Err(garmin::Error::Decode.into()); }
        // Cache failure must not undo verified identity; the reader rejects old-account files.
        if let Some(path) = &self.snapshot_path {
            let _ = status::write(path, &status::snapshot(&self.store, Local::now()));
        }
        Ok(())
    }
    /// Read the cache with authoritative privacy/opt-out policy. Never trust file enablement.
    pub fn surface_snapshot(&self) -> Result<Option<serde_json::Value>> {
        let _write = status::WRITE_LOCK.lock().unwrap();
        let Some(identity) = self.session_identity() else { return Ok(None) };
        let Some(path) = &self.snapshot_path else { return Ok(None) };
        let bytes = match std::fs::read(path) {
            Ok(bytes) => bytes,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e.into()),
        };
        let Ok(mut snapshot) = serde_json::from_slice::<serde_json::Value>(&bytes) else { return Ok(None) };
        if snapshot["version"] != 1 || snapshot["accountId"].as_str() != Some(&identity) {
            return Ok(None);
        }
        snapshot["enabled"] = self.settings().notify_status.into();
        Ok(Some(snapshot))
    }
    /// Recompute the native snapshot from local data without network access.
    pub fn refresh_status_snapshot(&self, now: DateTime<Local>) -> Result<()> {
        let signed_out = self.snapshot_state.lock().unwrap();
        let _write = status::WRITE_LOCK.lock().unwrap();
        self.write_status_snapshot(now, *signed_out)
    }
    fn write_status_snapshot(&self, now: DateTime<Local>, signed_out: bool) -> Result<()> {
        if !signed_out {
            if let Some(path) = &self.snapshot_path {
                status::write(path, &status::snapshot(&self.store, now))?;
            }
        }
        Ok(())
    }
    pub fn setting(&self, key: &str) -> bool {
        let s = self.settings();
        match key {
            "morning" => s.notify_morning_readiness,
            "weekly" => s.notify_weekly_checkin,
            "metric-changes" | "vo2max" | "fitness-age" => s.notify_metric_changes,
            "vitals" => s.notify_vitals,
            "training-status" => s.notify_training_status,
            "bedtime" => s.notify_bedtime,
            "status" => s.notify_status,
            _ => false,
        }
    }
    pub fn set_setting(&self, key: &str, value: bool) -> Result<()> {
        let mut s = self.settings();
        match key {
            "morning" => s.notify_morning_readiness = value,
            "weekly" => s.notify_weekly_checkin = value,
            "metric-changes" | "vo2max" | "fitness-age" => s.notify_metric_changes = value,
            "vitals" => s.notify_vitals = value,
            "training-status" => s.notify_training_status = value,
            "bedtime" => s.notify_bedtime = value,
            "status" => s.notify_status = value,
            _ => return Err(Error::InvalidSetting),
        }
        self.set_settings(s)
    }
    /// Tonight's pending wind-down reminder, for the platform to schedule.
    pub fn bedtime_reminder(&self, now: NaiveDateTime) -> Option<notifications::BedtimeReminder> {
        notifications::bedtime(&self.store, now)
    }
    pub fn notifications(&self, now: DateTime<Local>) -> Vec<Notification> {
        notifications::notifications(&self.store, now)
    }
    pub fn mark_notifications_sent(
        &self,
        items: &[Notification],
        now: DateTime<Local>,
    ) -> Result<()> {
        notifications::mark_sent(&self.store, items, now)?;
        Ok(())
    }
    /// Show due notifications through `show` and record only the ones the
    /// platform accepted.
    pub fn deliver_notifications(
        &self,
        items: &[Notification],
        now: DateTime<Local>,
        show: impl FnMut(&Notification) -> std::result::Result<(), String>,
    ) -> Result<notifications::Delivery> {
        Ok(notifications::deliver(&self.store, items, now, show)?)
    }
    pub fn forget_account(&self) -> Result<()> {
        let mut signed_out = self.snapshot_state.lock().unwrap();
        *signed_out = true;
        let _write = status::WRITE_LOCK.lock().unwrap();
        if let Some(path) = &self.snapshot_path {
            status::erase(path)?;
        }
        self.store.forget_account()?;
        Ok(())
    }
}
