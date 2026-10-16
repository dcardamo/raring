//! Shared, offline-first Garmin core. Callers own and persist credentials.
pub mod demo;
pub mod garmin;
pub mod notifications;
pub mod rules;
pub mod screens;
pub mod store;
pub mod sync;
pub mod view;

use chrono::{DateTime, Local};
pub use garmin::Tokens;
pub use notifications::Notification;
use std::{path::Path, sync::Arc};
pub use sync::SyncReport;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Store(#[from] rusqlite::Error),
    #[error(transparent)]
    Garmin(#[from] garmin::Error),
    #[error("unknown notification setting")]
    InvalidSetting,
}
pub type Result<T> = std::result::Result<T, Error>;
pub struct Pepometer {
    store: Arc<store::Store>,
}
pub struct SyncOutcome {
    pub tokens: Tokens,
    pub report: SyncReport,
    pub notifications: Vec<Notification>,
}
impl Pepometer {
    pub fn open(db_path: impl AsRef<Path>) -> Result<Self> {
        Ok(Self {
            store: Arc::new(store::Store::open(db_path)?),
        })
    }
    pub async fn sign_in_with_ticket(ticket: &str) -> Result<Tokens> {
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|_| garmin::Error::Network)?;
        Ok(garmin::exchange_ticket(&http, ticket, garmin::sso::PORTAL_SERVICE_URL).await?)
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
        let report = sync::sync(&store, &client, now, budget).await;
        let tokens = client.tokens().await;
        let notifications = notifications::notifications(&store, now);
        Ok(SyncOutcome {
            tokens,
            report,
            notifications,
        })
    }
    pub fn today(&self, date: &str) -> view::TodayView {
        screens::today(&self.store, date)
    }
    pub fn sleep(&self, date: &str) -> view::SleepView {
        screens::sleep_view(&self.store, date)
    }
    pub fn training(&self, date: &str) -> view::TrainingView {
        screens::training(&self.store, date)
    }
    pub fn workout(&self, id: &str) -> Option<view::WorkoutView> {
        screens::workout(&self.store, id)
    }
    pub fn health(&self, date: &str) -> view::HealthView {
        screens::health(&self.store, date)
    }
    pub fn checkin(&self, monday: &str) -> view::CheckinView {
        screens::checkin(&self.store, monday)
    }
    pub fn status(&self) -> view::AppStatus {
        view::AppStatus {
            signed_in: self.store.get("profile", "me").is_some(),
            syncing: false,
            last_sync: self.store.cursor("last_sync"),
            history_days: self.store.days(),
            last_error: self.store.cursor("last_error").filter(|s| !s.is_empty()),
        }
    }
    pub fn settings(&self) -> view::Settings {
        self.store.settings()
    }
    pub fn set_settings(&self, settings: view::Settings) -> Result<()> {
        self.store.set_settings(&settings)?;
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
            _ => return Err(Error::InvalidSetting),
        }
        self.set_settings(s)
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
    pub fn forget_account(&self) -> Result<()> {
        self.store.forget_account()?;
        Ok(())
    }
}
