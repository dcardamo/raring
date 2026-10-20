//! Desktop notification delivery. Android posts reminders from Kotlin and
//! schedules them with an alarm. A desktop build has no WorkManager, so it
//! delivers while the app runs, re-reads the plan on its own wall clock, and
//! records only what the platform accepted.
use chrono::{DateTime, Duration, Local, NaiveDateTime};
use raring_core::{notifications::BedtimeReminder, Raring};
use std::sync::Mutex;
use tauri::{AppHandle, Manager};
use tauri_plugin_notification::{NotificationExt, PermissionState};

use crate::ReminderStatus;

/// Routine recheck: a new day, a changed clock or a plan that moved must not
/// leave a stale reminder armed, even without a sync.
const RECHECK: Duration = Duration::minutes(5);
/// Sleep in slices so a wall-clock jump is noticed instead of slept through.
const SLICE: Duration = Duration::seconds(15);

/// One delivery at a time: the running check, a sync and a resume can all see
/// the same due items.
static DELIVERY: Mutex<()> = Mutex::new(());
/// Why the last attempt could not show a reminder, for Settings.
static LAST_ERROR: Mutex<Option<String>> = Mutex::new(None);

/// The next wall-clock time worth waking for: tonight's pending wind-down.
/// Once it is due, the routine check delivers it; once its window passed, the
/// core no longer offers it.
fn wake_target(reminder: Option<&BedtimeReminder>, now: NaiveDateTime) -> Option<NaiveDateTime> {
    reminder.filter(|r| r.at > now).map(|r| r.at)
}

/// The wait before the next check: the pending wind-down when it is nearer,
/// never past the routine recheck.
fn wait_for(wake: Option<NaiveDateTime>, now: NaiveDateTime) -> Duration {
    let cap = now + RECHECK;
    wake.filter(|at| *at > now)
        .map_or(cap, |at| at.min(cap))
        - now
}

/// Desktop signs in with a token file rather than Android's account binding,
/// so this is the desktop equivalent of the core's session gate.
fn signed_in(app: &AppHandle) -> bool {
    crate::load_tokens(app).is_ok_and(|tokens| tokens.is_some())
}

fn supported(app: &AppHandle) -> bool {
    app.try_state::<tauri_plugin_notification::Notification<tauri::Wry>>()
        .is_some()
}

fn show(app: &AppHandle, item: &raring_core::Notification) -> Result<(), String> {
    app.notification()
        .builder()
        .title(&item.title)
        .body(&item.body)
        .show()
        .map_err(|e| e.to_string())
}

/// Deliver every notification the core reports due at `now`. Only the ones the
/// platform accepted are marked; the rest stay due and are retried, and the
/// last failure is kept for Settings.
pub fn deliver_due(
    app: &AppHandle,
    core: &Raring,
    now: DateTime<Local>,
) -> Result<(), String> {
    let _held = DELIVERY.lock().map_err(|_| "reminder delivery lock".to_string())?;
    if !supported(app) {
        return Err("this build cannot show notifications".into());
    }
    let due = core.notifications(now);
    if due.is_empty() {
        return Ok(());
    }
    let delivery = core
        .deliver_notifications(&due, now, |item| show(app, item))
        .map_err(|e| e.to_string())?;
    let error = delivery.failed.first().map(|f| f.error.clone());
    if let Some(message) = &error {
        log::warn!("{} reminder(s) not shown, left pending: {message}", delivery.failed.len());
    }
    if let Ok(mut last) = LAST_ERROR.lock() {
        *last = error;
    }
    Ok(())
}

/// Show what is due right now. Called by the running check, after a sync, and
/// when the app resumes or a notification setting changes.
pub fn check(app: &AppHandle) {
    let core = match crate::core(app) {
        Ok(core) => core,
        Err(_) => return,
    };
    if !signed_in(app) {
        return;
    }
    if let Err(error) = deliver_due(app, &core, Local::now()) {
        if let Ok(mut last) = LAST_ERROR.lock() {
            *last = Some(error.clone());
        }
        log::warn!("reminders not delivered: {error}");
    }
}

/// The next pending wind-down, for the runner's sleep and for Settings.
fn next_wake(app: &AppHandle, now: DateTime<Local>) -> Option<DateTime<Local>> {
    let core = crate::core(app).ok()?;
    if !signed_in(app) {
        return None;
    }
    wake_target(core.bedtime_reminder(now.naive_local()).as_ref(), now.naive_local())
        .and_then(|at| at.and_local_timezone(Local).earliest())
}

/// What Settings can say about desktop reminders: whether the build can show
/// them, the platform's permission, the last failure and the next pending time.
pub fn status(app: &AppHandle) -> ReminderStatus {
    let supported = supported(app);
    let granted = supported
        && app
            .notification()
            .permission_state()
            .is_ok_and(|state| state == PermissionState::Granted);
    ReminderStatus {
        supported,
        granted,
        error: LAST_ERROR.lock().ok().and_then(|last| last.clone()),
        next_at: next_wake(app, Local::now()).map(|at| at.format("%Y-%m-%dT%H:%M:%S").to_string()),
    }
}

fn sleep_until(target: DateTime<Local>) {
    loop {
        let remaining = target.signed_duration_since(Local::now());
        if remaining <= Duration::zero() {
            return;
        }
        let slice = remaining.min(SLICE).to_std().unwrap_or_default();
        std::thread::sleep(slice);
    }
}

/// Keep checking while the app runs. The thread ends with the process, and a
/// sync, resume or settings change can run a check sooner.
pub fn start(app: AppHandle) {
    let spawned = std::thread::Builder::new()
        .name("raring-reminders".into())
        .spawn(move || loop {
            check(&app);
            let now = Local::now();
            let wake = next_wake(&app, now);
            sleep_until(now + wait_for(wake.map(|at| at.naive_local()), now.naive_local()));
        });
    if let Err(error) = spawned {
        log::warn!("could not start the reminder check: {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use raring_core::store::Store;
    use serde_json::json;

    const DATE: &str = "2025-03-12";

    fn at(date: &str, time: &str) -> NaiveDateTime {
        NaiveDateTime::parse_from_str(&format!("{date}T{time}:00"), "%Y-%m-%dT%H:%M:%S").unwrap()
    }

    fn reminder(at_time: NaiveDateTime) -> BedtimeReminder {
        BedtimeReminder {
            at: at_time,
            until: at_time + Duration::hours(2),
            night: DATE.into(),
            title: "Time to wind down".into(),
            body: "Synthetic plan".into(),
        }
    }

    #[test]
    fn a_pending_wind_down_is_the_wake_target() {
        let now = at(DATE, "20:00");
        assert_eq!(wake_target(Some(&reminder(at(DATE, "21:45"))), now), Some(at(DATE, "21:45")));
        assert_eq!(wait_for(Some(at(DATE, "21:45")), now), RECHECK);
        assert_eq!(wait_for(Some(at(DATE, "20:01")), now), Duration::seconds(60));
        assert_eq!(wait_for(None, now), RECHECK);
    }

    #[test]
    fn a_reminder_already_due_or_past_is_not_a_wake_target() {
        let now = at(DATE, "22:00");
        assert_eq!(wake_target(Some(&reminder(at(DATE, "21:45"))), now), None);
        assert_eq!(wait_for(Some(at(DATE, "21:45")), now), RECHECK);
    }

    #[test]
    fn the_core_drops_the_target_once_the_night_was_sent() {
        let mut random = [0u8; 16];
        getrandom::fill(&mut random).unwrap();
        let directory =
            std::env::temp_dir().join(format!("raring-wake-{:x}", u128::from_le_bytes(random)));
        std::fs::create_dir(&directory).unwrap();
        let database = directory.join("data.db");
        let store = Store::open(&database).unwrap();
        store
            .put(
                "sleep",
                DATE,
                "now",
                &json!({"dailySleepDTO": {
                    "calendarDate": DATE,
                    "sleepTimeSeconds": 25000,
                    "sleepStartTimestampLocal": format!("{DATE}T23:00:00"),
                    "sleepEndTimestampLocal": format!("{DATE}T07:00:00"),
                    "sleepNeed": {"actual": 450, "baseline": 450},
                    "nextSleepNeed": {
                        "calendarDate": "2025-03-13", "actual": 450, "baseline": 450,
                        "recommendedBedtimeStartMins": 1350, "recommendedBedtimeEndMins": 1410,
                    },
                }}),
            )
            .unwrap();
        let mut settings = store.settings();
        settings.notify_bedtime = true;
        settings.bedtime_lead_min = 45;
        store.set_settings(&settings).unwrap();
        let core = Raring::open(&database).unwrap();
        let now = at(DATE, "20:00");
        let plan = core.bedtime_reminder(now).unwrap();
        assert_eq!(plan.at, at(DATE, "21:45"));
        assert_eq!(wake_target(Some(&plan), now), Some(at(DATE, "21:45")));

        core.mark_notifications_sent(
            &[raring_core::Notification {
                kind: "bedtime".into(),
                title: plan.title.clone(),
                body: plan.body.clone(),
                tag: plan.night.clone(),
            }],
            at(DATE, "21:46").and_local_timezone(Local).earliest().unwrap(),
        )
        .unwrap();
        assert!(core.bedtime_reminder(at(DATE, "21:47")).is_none());
        assert!(core.bedtime_reminder(at(DATE, "23:30")).is_none());
        std::fs::remove_dir_all(directory).unwrap();
    }
}
