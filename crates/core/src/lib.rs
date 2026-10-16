//! Shared, offline-first Garmin core. Callers own and persist credentials.
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
    #[error("background task failed")]
    Task,
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
    pub fn setting(&self, key: &str) -> bool {
        self.store
            .setting(&format!("notify.{key}"))
            .map(|s| s == "true")
            .unwrap_or(!matches!(key, "training-status" | "bedtime"))
    }
    pub fn set_setting(&self, key: &str, value: bool) -> Result<()> {
        self.store.set_setting(
            &format!("notify.{key}"),
            if value { "true" } else { "false" },
        )?;
        Ok(())
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
