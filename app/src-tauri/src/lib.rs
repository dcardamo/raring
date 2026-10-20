use chrono::{Datelike, Local};
#[cfg(not(target_os = "android"))]
mod desktop_credentials;
#[cfg(desktop)]
mod desktop_reminders;
#[cfg(not(target_os = "android"))]
use desktop_credentials::DesktopCredentials;
#[cfg(not(target_os = "android"))]
use raring_core::Tokens;
use raring_core::{garmin::sso, view::*, Raring};
#[cfg(target_os = "android")]
use serde::Deserialize;
use serde::Serialize;
use std::sync::Mutex;
use tauri::{Emitter, Manager};

#[derive(Default)]
struct SignInState(Mutex<Option<String>>);
#[derive(Default)]
struct SignInPending(Mutex<Option<u64>>);

/// A chosen export, held between the preview and applying it. The core reads
/// only this path; an Android copy in the app's cache is removed afterwards.
#[derive(Clone)]
struct ChosenImport {
    path: std::path::PathBuf,
    /// The cache leaf the native picker wrote, when the file is a copy.
    cleanup: Option<String>,
}
#[derive(Default)]
struct ImportState(Mutex<Option<ChosenImport>>);

/// What a desktop build can say about reminders in Settings. Android reports
/// its permission through the native plugin, so it returns none.
#[derive(Serialize, Clone, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ReminderStatus {
    /// This build has a notification path at all.
    supported: bool,
    /// The platform allows Raring to show notifications.
    granted: bool,
    /// Why the last attempt could not show a reminder.
    error: Option<String>,
    /// The pending wind-down the running check will wake for.
    next_at: Option<String>,
}

fn core(app: &tauri::AppHandle) -> Result<Raring, String> {
    Raring::open(database_path(app)?).map_err(|e| e.to_string())
}

fn monday() -> String {
    let date = Local::now().date_naive();
    (date - chrono::Duration::days(date.weekday().num_days_from_monday() as i64)).to_string()
}

fn runtime() -> Result<tokio::runtime::Runtime, String> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| e.to_string())
}

fn home_url() -> tauri::Url {
    #[cfg(target_os = "android")]
    const URL: &str = "http://tauri.localhost/";
    #[cfg(not(target_os = "android"))]
    const URL: &str = "tauri://localhost/";
    tauri::Url::parse(URL).expect("valid app URL")
}

#[cfg(target_os = "android")]
struct AndroidVault(tauri::plugin::PluginHandle<tauri::Wry>);

#[cfg(target_os = "android")]
fn vault(app: &tauri::AppHandle) -> tauri::State<'_, AndroidVault> {
    app.state::<AndroidVault>()
}

#[cfg(not(target_os = "android"))]
fn load_tokens(app: &tauri::AppHandle) -> Result<Option<String>, String> {
    #[cfg(target_os = "android")]
    {
        #[derive(Deserialize)]
        struct Response {
            value: Option<String>,
        }
        return vault(app)
            .0
            .run_mobile_plugin::<Response>("loadTokens", serde_json::json!({}))
            .map(|r| r.value)
            .map_err(|e| e.to_string());
    }
    #[cfg(not(target_os = "android"))]
    {
        let path = app
            .path()
            .app_data_dir()
            .map_err(|e| e.to_string())?
            .join("tokens.json");
        match std::fs::read_to_string(path) {
            Ok(value) => Ok(Some(value)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e.to_string()),
        }
    }
}

#[cfg(not(target_os = "android"))]
fn save_tokens(app: &tauri::AppHandle, value: &str) -> Result<(), String> {
    #[cfg(not(target_os = "android"))]
    {
        use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
        let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .mode(0o600)
            .open(dir.join("tokens.json"))
            .map_err(|e| e.to_string())?;
        // OpenOptions::mode only restricts newly created files, not existing ones.
        file.set_permissions(std::fs::Permissions::from_mode(0o600))
            .map_err(|e| e.to_string())?;
        use std::io::Write;
        file.write_all(value.as_bytes())
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn clear_tokens(app: &tauri::AppHandle) -> Result<(), String> {
    #[cfg(target_os = "android")]
    {
        vault(app)
            .0
            .run_mobile_plugin::<serde_json::Value>("clearTokens", serde_json::json!({}))
            .map_err(|e| e.to_string())?;
    }
    #[cfg(not(target_os = "android"))]
    {
        let path = app
            .path()
            .app_data_dir()
            .map_err(|e| e.to_string())?
            .join("tokens.json");
        if let Err(e) = std::fs::remove_file(path) {
            if e.kind() != std::io::ErrorKind::NotFound {
                return Err(e.to_string());
            }
        }
    }
    Ok(())
}

#[cfg(target_os = "android")]
fn session_ready(app: &tauri::AppHandle) -> Result<bool, String> {
    #[derive(Deserialize)]
    struct State { active: bool }
    vault(app).0.run_mobile_plugin::<State>("sessionState", serde_json::json!({}))
        .map(|s| s.active).map_err(|e| e.to_string())
}

#[cfg(not(target_os = "android"))]
async fn with_desktop_tokens<T, F, Fut>(
    app: &tauri::AppHandle,
    operation: F,
) -> Result<Option<T>, String>
where
    F: FnOnce(Tokens) -> Fut,
    Fut: std::future::Future<Output = Result<(T, Tokens), String>>,
{
    app.state::<DesktopCredentials>()
        .use_tokens(
            || {
                load_tokens(app)?
                    .map(|json| serde_json::from_str(&json).map_err(|e| e.to_string()))
                    .transpose()
            },
            |tokens| {
                save_tokens(
                    app,
                    &serde_json::to_string(tokens).map_err(|e| e.to_string())?,
                )
            },
            operation,
        )
        .await
}

fn status_read(app: tauri::AppHandle) -> Result<AppStatus, String> {
    #[cfg(target_os = "android")]
    let mut status = {
        let hidden = || AppStatus { signed_in:false, syncing:false, last_sync:None,
            history_days:0, last_error:None, history_from:None, history_complete:false };
        let verified = || -> Option<AppStatus> {
            if !session_ready(&app).unwrap_or(false) { return None; }
            let core = core(&app).ok()?;
            let identity = core.session_identity()?;
            let snapshot = core.with_verified_session(&identity, || core.status())?;
            (session_ready(&app).unwrap_or(false) && core.session_identity().as_ref() == Some(&identity)).then_some(snapshot)
        };
        verified().unwrap_or_else(hidden)
    };
    #[cfg(not(target_os = "android"))]
    let mut status = { let mut s = core(&app)?.status(); s.signed_in = load_tokens(&app)?.is_some(); s };
    if let Ok(error) = app.state::<SignInState>().0.lock() {
        if error.is_some() {
            status.last_error = error.clone();
        }
    }
    Ok(status)
}

#[tauri::command]
async fn app_status(app: tauri::AppHandle) -> Result<AppStatus, String> {
    tauri::async_runtime::spawn_blocking(move || status_read(app))
        .await
        .map_err(|e| e.to_string())?
}

// View construction reads many SQLite rows; synchronous commands block Android's WebView.
async fn read_view<T: Send + 'static>(
    app: tauri::AppHandle,
    read: impl FnOnce(&Raring) -> Result<T, String> + Send + 'static,
) -> Result<T, String> {
    #[cfg(target_os = "android")]
    let identity = core(&app)?.session_identity().ok_or("Account verification required")?;
    tauri::async_runtime::spawn_blocking(move || {
        let core = core(&app)?;
        #[cfg(target_os = "android")]
        if !session_ready(&app)? { return Err("Account verification required".into()); }
        #[cfg(target_os = "android")]
        let result = core.with_verified_session(&identity, || read(&core)).ok_or("Account verification required")??;
        #[cfg(not(target_os = "android"))]
        let result = read(&core)?;
        #[cfg(target_os = "android")]
        if !session_ready(&app)? || core.session_identity().as_ref() != Some(&identity) {
            return Err("Account changed while reading data".into());
        }
        Ok(result)
    })
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn today(app: tauri::AppHandle, date: Option<String>) -> Result<TodayView, String> {
    read_view(app, move |core| {
        match date {
            Some(day) => Ok(core.today(&day)),
            None => {
                let now = Local::now();
                core.capture_sleep_plan(now).map_err(|e| e.to_string())?;
                Ok(core.today_live_at(&core.current_date(now), now))
            }
        }
    })
    .await
}
#[tauri::command]
async fn sleep(app: tauri::AppHandle, date: Option<String>) -> Result<SleepView, String> {
    read_view(app, move |core| {
        if date.is_none() {
            core.capture_sleep_plan(Local::now())
                .map_err(|e| e.to_string())?;
        }
        Ok(match date {
            Some(date) => core.sleep(&date),
            None => core.sleep_live(Local::now()),
        })
    })
    .await
}
#[tauri::command]
async fn training(app: tauri::AppHandle, date: Option<String>) -> Result<TrainingView, String> {
    read_view(app, move |core| {
        let now = Local::now();
        Ok(match date {
            Some(day) => core.training(&day),
            None => core.training_live_at(&core.current_date(now), now),
        })
    })
    .await
}
#[tauri::command]
async fn workout(app: tauri::AppHandle, id: String) -> Result<WorkoutView, String> {
    read_view(app, move |core| {
        core.workout_live(&id, Local::now()).ok_or_else(|| "not_found".into())
    })
    .await
}
#[tauri::command]
async fn strength(
    app: tauri::AppHandle,
    date: Option<String>,
) -> Result<Option<StrengthView>, String> {
    read_view(app, move |core| {
        Ok(core.strength(&date.unwrap_or_else(|| core.current_date(Local::now()))))
    })
    .await
}
#[tauri::command]
async fn health(app: tauri::AppHandle, date: Option<String>) -> Result<HealthView, String> {
    read_view(app, move |core| {
        let now = Local::now();
        Ok(match date {
            Some(day) => core.health(&day),
            None => core.health_live_at(&core.current_date(now), now),
        })
    })
    .await
}
#[tauri::command]
async fn heart_rate_day(app: tauri::AppHandle, date: String) -> Result<HeartRateDay, String> {
    #[cfg(target_os = "android")]
    {
        #[derive(Deserialize)]
        struct Response {
            day: HeartRateDay,
        }
        if !session_ready(&app)? { return Err("Account verification required".into()); }
        let expected = core(&app)?.session_identity().ok_or("Account verification required")?;
        let response = vault(&app)
            .0
            .run_mobile_plugin_async::<Response>("heartRateDay", serde_json::json!({"date":date}))
            .await
            .map_err(|e| e.to_string())?;
        if !session_ready(&app)? || core(&app)?.session_identity().as_ref() != Some(&expected) {
            return Err("Account changed while reading heart rate".into());
        }
        return Ok(response.day);
    }
    #[cfg(not(target_os = "android"))]
    {
        let core = core(&app)?;
        if let Some(day) = core.cached_heart_rate_day(&date) {
            return Ok(day);
        }
        let (day, report) = with_desktop_tokens(&app, |tokens| async move {
            let (day, tokens, report) = core
                .fetch_heart_rate_day(tokens, &date)
                .await
                .map_err(|e| e.to_string())?;
            Ok(((day, report), tokens))
        })
        .await?
        .ok_or("sign in again")?;
        if report.auth_failed {
            return Err("sign in again".into());
        }
        if let Some(error) = report.error {
            return Err(error);
        }
        Ok(day)
    }
}

fn day_detail_parts(scope: &str) -> Result<Vec<DayDetailPart>, String> {
    match scope {
        "sleep" => Ok(vec![DayDetailPart::Sleep]),
        "day" => Ok(vec![DayDetailPart::Stress, DayDetailPart::Vitals]),
        _ => Err("unknown day detail scope".into()),
    }
}

#[tauri::command]
async fn fetch_day_detail(app: tauri::AppHandle, date: String, scope: String) -> Result<(), String> {
    let parts = day_detail_parts(&scope)?;
    #[cfg(target_os = "android")]
    {
        if !session_ready(&app)? { return Err("Account verification required".into()); }
        let expected = core(&app)?.session_identity().ok_or("Account verification required")?;
        vault(&app)
            .0
            .run_mobile_plugin_async::<serde_json::Value>(
                "fetchDetail",
                serde_json::json!({"date":date,"activityId":"","scope":scope}),
            )
            .await
            .map_err(|e| e.to_string())?;
        if !session_ready(&app)? || core(&app)?.session_identity().as_ref() != Some(&expected) {
            return Err("Account changed while reading the day".into());
        }
        return Ok(());
    }
    #[cfg(not(target_os = "android"))]
    {
        let core = core(&app)?;
        let report = with_desktop_tokens(&app, |tokens| async move {
            let (tokens, report) = core
                .fetch_day_detail(tokens, &date, &parts)
                .await
                .map_err(|e| e.to_string())?;
            Ok((report, tokens))
        })
        .await?
        .ok_or("sign in again")?;
        if report.auth_failed {
            return Err("sign in again".into());
        }
        if let Some(error) = report.error {
            return Err(error);
        }
        Ok(())
    }
}

#[tauri::command]
async fn fetch_workout_detail(app: tauri::AppHandle, id: String) -> Result<(), String> {
    #[cfg(target_os = "android")]
    {
        if !session_ready(&app)? { return Err("Account verification required".into()); }
        let expected = core(&app)?.session_identity().ok_or("Account verification required")?;
        vault(&app)
            .0
            .run_mobile_plugin_async::<serde_json::Value>(
                "fetchDetail",
                serde_json::json!({"date":"","activityId":id}),
            )
            .await
            .map_err(|e| e.to_string())?;
        if !session_ready(&app)? || core(&app)?.session_identity().as_ref() != Some(&expected) {
            return Err("Account changed while reading the workout".into());
        }
        return Ok(());
    }
    #[cfg(not(target_os = "android"))]
    {
        let core = core(&app)?;
        let report = with_desktop_tokens(&app, |tokens| async move {
            let (tokens, report) = core
                .fetch_workout_detail(tokens, &id)
                .await
                .map_err(|e| e.to_string())?;
            Ok((report, tokens))
        })
        .await?
        .ok_or("sign in again")?;
        if report.auth_failed {
            return Err("sign in again".into());
        }
        if let Some(error) = report.error {
            return Err(error);
        }
        Ok(())
    }
}

#[tauri::command]
async fn refresh_strength(
    app: tauri::AppHandle,
    id: String,
) -> Result<Option<StrengthSession>, String> {
    #[cfg(target_os = "android")]
    {
        #[derive(Deserialize)]
        struct Response {
            session: Option<StrengthSession>,
        }
        if !session_ready(&app)? {
            return Err("Account verification required".into());
        }
        let expected = core(&app)?.session_identity().ok_or("Account verification required")?;
        let response = vault(&app)
            .0
            .run_mobile_plugin_async::<Response>("refreshStrength", serde_json::json!({"id": id}))
            .await
            .map_err(|e| e.to_string())?;
        if !session_ready(&app)? || core(&app)?.session_identity().as_ref() != Some(&expected) {
            return Err("Account changed while refreshing sets".into());
        }
        return Ok(response.session);
    }
    #[cfg(not(target_os = "android"))]
    {
        let core = core(&app)?;
        let (session, report) = with_desktop_tokens(&app, |tokens| async move {
            let (session, tokens, report) = core
                .refresh_strength_sets(tokens, &id)
                .await
                .map_err(|e| e.to_string())?;
            Ok(((session, report), tokens))
        })
        .await?
        .ok_or("sign in again")?;
        if report.auth_failed {
            return Err("sign in again".into());
        }
        if let Some(error) = report.error {
            return Err(error);
        }
        Ok(session)
    }
}

#[tauri::command]
async fn checkin(app: tauri::AppHandle, week_start: Option<String>) -> Result<CheckinView, String> {
    read_view(app, move |core| {
        Ok(core.checkin(&week_start.unwrap_or_else(monday)))
    })
    .await
}

#[tauri::command]
async fn trend(app: tauri::AppHandle, query: TrendQuery) -> Result<TrendView, String> {
    read_view(app, move |core| {
        core.trend(&query, &core.current_date(Local::now()))
    })
    .await
}
/// The plan week containing `date` (this week when absent), on the calendar day.
#[tauri::command]
async fn plan_board(app: tauri::AppHandle, date: Option<String>) -> Result<raring_core::view::PlanBoard, String> {
    read_view(app, move |core| {
        let now = Local::now();
        core.plan_board(&date.unwrap_or_else(|| now.date_naive().to_string()), now).map_err(|e| e.to_string())
    })
    .await
}

/// The review of the plan week containing `date`, with next week's proposals.
#[tauri::command]
async fn week_review(app: tauri::AppHandle, date: String) -> Result<raring_core::view::WeekReview, String> {
    read_view(app, move |core| core.week_review(&date, Local::now()).map_err(|e| e.to_string())).await
}
#[tauri::command]
async fn progress(app: tauri::AppHandle, period: raring_core::view::Period, date: Option<String>) -> Result<raring_core::view::ProgressView, String> {
    read_view(app, move |core| {
        let now = Local::now();
        core.progress(period, &date.unwrap_or_else(|| now.date_naive().to_string()), now).map_err(|e| e.to_string())
    })
    .await
}
#[tauri::command]
async fn weight_trend(app: tauri::AppHandle, range: TrendRange, date: Option<String>) -> Result<WeightView, String> {
    read_view(app, move |core| {
        let now = Local::now();
        core.weight_trend(range, &date.unwrap_or_else(|| now.date_naive().to_string()), now).map_err(|e| e.to_string())
    }).await
}
#[tauri::command]
async fn muscles(app: tauri::AppHandle, range: TrendRange) -> Result<Option<raring_core::view::MuscleRange>, String> {
    read_view(app, move |core| Ok(core.muscles(range, Local::now()))).await
}
#[tauri::command]
async fn lift_series(app: tauri::AppHandle, key: String, range: TrendRange) -> Result<Option<raring_core::view::LiftSeries>, String> {
    read_view(app, move |core| Ok(core.lift_series(&key, range, Local::now()))).await
}
#[tauri::command]
async fn progress_domains(app: tauri::AppHandle) -> Result<raring_core::view::ProgressDomains, String> {
    read_view(app, move |core| Ok(core.progress_domains(Local::now()))).await
}
#[tauri::command]
async fn milestone_archive(app: tauri::AppHandle) -> Result<Vec<raring_core::view::Milestone>, String> {
    read_view(app, move |core| core.milestone_archive(Local::now()).map_err(|e| e.to_string())).await
}
#[tauri::command]
async fn profile(app: tauri::AppHandle) -> Result<raring_core::view::ProfileView, String> {
    read_view(app, move |core| Ok(core.profile(Local::now()))).await
}
/// Recent Garmin writes for Settings; local data only, never a request.
#[tauri::command]
async fn garmin_changes(app: tauri::AppHandle) -> Result<Vec<raring_core::view::GarminChange>, String> {
    read_view(app, |core| Ok(core.garmin_changes())).await
}
#[tauri::command]
async fn reviews(app: tauri::AppHandle) -> Result<Vec<raring_core::view::ReviewRow>, String> {
    read_view(app, move |core| Ok(core.reviews(Local::now()))).await
}

#[tauri::command]
async fn begin_sign_in(app: tauri::AppHandle) -> Result<(), String> {
    *app.state::<SignInPending>().0.lock().map_err(|e| e.to_string())? = None;
    #[cfg(target_os = "android")]
    let generation = {
        #[derive(Deserialize)]
        struct Attempt { generation: u64 }
        let result = vault(&app).0.run_mobile_plugin_async::<Attempt>(
            "suspendSession", serde_json::json!({})
        ).await.map_err(|e| e.to_string());
        let _ = app.emit("status-changed", ());
        result?.generation
    };
    #[cfg(not(target_os = "android"))]
    let generation = app.state::<DesktopCredentials>().invalidate_sign_in();
    *app.state::<SignInPending>()
        .0
        .lock()
        .map_err(|e| e.to_string())? = Some(generation);
    app.get_webview_window("main")
        .ok_or("main window missing")?
        .navigate(tauri::Url::parse(&sso::sign_in_url()).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())
}

fn exchange_ticket_and_store(app: &tauri::AppHandle, ticket: String, generation: u64) {
    #[cfg(target_os = "android")]
    let outcome = {
        #[derive(Deserialize)]
        struct Promotion { saved: bool }
        vault(app).0.run_mobile_plugin::<Promotion>("exchangeTicket",
            serde_json::json!({"ticket":ticket,"generation":generation}))
            .map(|r| r.saved).map_err(|e| e.to_string())
    };
    #[cfg(not(target_os = "android"))]
    let outcome = runtime().and_then(|rt| {
        let tokens = rt
            .block_on(Raring::sign_in_with_ticket(&ticket))
            .map_err(|_| "Garmin sign-in failed — try again".to_owned())?;
        let json = serde_json::to_string(&tokens).map_err(|e| e.to_string())?;
        rt.block_on(
            app.state::<DesktopCredentials>()
                .save_sign_in(generation, || save_tokens(app, &json)),
        )
    });
    if outcome.as_ref().is_ok_and(|saved| !saved) {
        return;
    }
    let outcome = outcome.map(|_| ());
    if let Ok(mut error) = app.state::<SignInState>().0.lock() {
        *error = outcome.as_ref().err().cloned();
    }
    if let Ok(status) = status_read(app.clone()) {
        let _ = app.emit("status-changed", status);
    }
    if outcome.is_ok() {
        let app = app.clone();
        std::thread::spawn(move || {
            if let Ok(rt) = runtime() {
                let _ = rt.block_on(sync_now(app));
            }
        });
    }
}

#[tauri::command]
async fn sign_out(app: tauri::AppHandle) -> Result<(), String> {
    *app.state::<SignInPending>().0.lock().map_err(|e| e.to_string())? = None;
    #[cfg(not(target_os = "android"))]
    app.state::<DesktopCredentials>()
        .sign_out(|| {
            clear_tokens(&app)?;
            core(&app)?.forget_account().map_err(|e| e.to_string())
        })
        .await?;
    #[cfg(target_os = "android")]
    {
        let result = clear_tokens(&app);
        let _ = app.emit("status-changed", ());
        result?;
    }
    if let Ok(cache) = app.path().app_cache_dir() {
        let _ = std::fs::remove_dir_all(cache.join("exports"));
    }
    if let Ok(mut error) = app.state::<SignInState>().0.lock() {
        *error = None;
    }
    let _ = app.emit("status-changed", status_read(app.clone())?);
    Ok(())
}

#[tauri::command]
async fn sync_now(app: tauri::AppHandle) -> Result<AppStatus, String> {
    #[cfg(target_os = "android")]
    vault(&app)
        .0
        .run_mobile_plugin_async::<serde_json::Value>("syncNow", serde_json::json!({}))
        .await
        .map_err(|e| e.to_string())?;
    #[cfg(desktop)]
    {
        let sync_core = core(&app)?;
        let synced = with_desktop_tokens(&app, |tokens| async move {
            let outcome = sync_core
                .sync(tokens, Local::now(), 60)
                .await
                .map_err(|e| e.to_string())?;
            if let Some(error) = &outcome.snapshot_error { log::warn!("status snapshot publication failed: {error}"); }
            Ok(((), outcome.tokens))
        })
        .await?;
        if synced.is_some() {
            // A failed reminder stays unmarked and is retried; it never fails the sync.
            if let Err(error) = desktop_reminders::deliver_due(&app, &core(&app)?, Local::now()) {
                log::warn!("reminders not delivered after sync: {error}");
            }
        }
    }
    let status = app_status(app.clone()).await?;
    let _ = app.emit("status-changed", &status);
    Ok(status)
}

#[tauri::command]
fn open_garmin_connect(app: tauri::AppHandle) -> bool {
    #[cfg(target_os = "android")]
    {
        #[derive(Deserialize)]
        struct Response {
            launched: bool,
        }
        return vault(&app)
            .0
            .run_mobile_plugin::<Response>("openGarminConnect", serde_json::json!({}))
            .is_ok_and(|r| r.launched);
    }
    #[cfg(not(target_os = "android"))]
    {
        let _ = app;
        false
    }
}

#[tauri::command]
fn request_notification_permission(app: tauri::AppHandle) -> Result<bool, String> {
    #[cfg(target_os = "android")]
    {
        #[derive(Deserialize)]
        struct Response {
            granted: bool,
        }
        return vault(&app)
            .0
            .run_mobile_plugin::<Response>("requestNotificationPermission", serde_json::json!({}))
            .map(|r| r.granted)
            .map_err(|e| e.to_string());
    }
    #[cfg(not(target_os = "android"))]
    {
        let _ = app;
        // Desktop needs no permission prompt; Settings reads what the platform
        // allows through `reminders_status`.
        Ok(true)
    }
}

#[cfg(desktop)]
#[tauri::command]
fn check_reminders(app: tauri::AppHandle) {
    desktop_reminders::check(&app);
}

#[cfg(mobile)]
#[tauri::command]
fn check_reminders(_app: tauri::AppHandle) {}

#[cfg(desktop)]
#[tauri::command]
fn reminders_status(app: tauri::AppHandle) -> Option<ReminderStatus> {
    Some(desktop_reminders::status(&app))
}

#[cfg(mobile)]
#[tauri::command]
fn reminders_status(_app: tauri::AppHandle) -> Option<ReminderStatus> {
    None
}

fn database_path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    let root = app.path().app_data_dir().map_err(|e| e.to_string())?;
    // Tauri Android's app_data_dir is Context.dataDir, NOT Context.filesDir.
    #[cfg(target_os = "android")]
    let root = root.join("files");
    std::fs::create_dir_all(&root).map_err(|e| e.to_string())?;
    Ok(root.join("raring.db"))
}

// Android's app sandbox forbids hard links. The kernel's no-replace rename
// publishes a complete archive atomically without clobbering another export.
#[cfg(any(target_os = "linux", target_os = "android"))]
fn publish_archive_no_replace(
    source: &std::path::Path,
    destination: &std::path::Path,
) -> Result<(), String> {
    use std::{ffi::CString, os::unix::ffi::OsStrExt};
    let source = CString::new(source.as_os_str().as_bytes()).map_err(|e| e.to_string())?;
    let destination = CString::new(destination.as_os_str().as_bytes()).map_err(|e| e.to_string())?;
    // SAFETY: both pointers remain valid, NUL-terminated paths throughout the
    // syscall; renameat2 takes no writable buffers or owned file descriptors.
    let result = unsafe {
        libc::syscall(
            libc::SYS_renameat2,
            libc::AT_FDCWD,
            source.as_ptr(),
            libc::AT_FDCWD,
            destination.as_ptr(),
            libc::RENAME_NOREPLACE,
        )
    };
    if result == 0 {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error().to_string())
    }
}

#[cfg(not(any(target_os = "linux", target_os = "android")))]
fn publish_archive_no_replace(
    source: &std::path::Path,
    destination: &std::path::Path,
) -> Result<(), String> {
    std::fs::hard_link(source, destination).map_err(|e| e.to_string())?;
    if let Err(e) = std::fs::remove_file(source) {
        log::warn!("could not remove export staging file: {e}");
    }
    Ok(())
}

#[cfg(not(target_os = "android"))]
fn save_archive_to_downloads(
    source: &std::path::Path,
    downloads: &std::path::Path,
    filename: &str,
) -> Result<std::path::PathBuf, String> {
    use std::io::Write;
    let temporary = downloads.join(format!("{filename}.partial"));
    let destination = downloads.join(filename);
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut output = options.open(&temporary).map_err(|e| e.to_string())?;
    let result = (|| {
        let mut input = std::fs::File::open(source).map_err(|e| e.to_string())?;
        std::io::copy(&mut input, &mut output).map_err(|e| e.to_string())?;
        output.flush().map_err(|e| e.to_string())?;
        output.sync_all().map_err(|e| e.to_string())?;
        publish_archive_no_replace(&temporary, &destination)?;
        Ok(destination)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    result
}

// Hold the local account transition barrier only across the final check and rename.
// The ZIP construction and native IPC deliberately run outside this barrier.
#[cfg(any(target_os = "android", test))]
fn publish_bound_archive(
    core: &Raring,
    expected: &str,
    publish: impl FnOnce() -> Result<(), String>,
) -> Result<(), String> {
    core.with_verified_session(expected, publish)
        .ok_or_else(|| "Account changed during export".to_owned())?
}

#[tauri::command]
async fn export_archive(app: tauri::AppHandle) -> Result<String, String> {
    #[cfg(target_os = "android")]
    let expected = {
        if !session_ready(&app)? { return Err("Account verification required".into()); }
        core(&app)?.session_identity().ok_or("Account verification required")?
    };
    let mut random = [0u8; 16];
    getrandom::fill(&mut random).map_err(|e| e.to_string())?;
    let suffix = random
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let filename = format!(
        "raring-{}-{suffix}.zip",
        Local::now().format("%Y%m%d-%H%M%S-%3f")
    );
    let path = app
        .path()
        .app_cache_dir()
        .map_err(|e| e.to_string())?
        .join("exports");
    std::fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    #[cfg(target_os = "android")]
    vault(&app)
        .0
        .run_mobile_plugin::<serde_json::Value>(
            "prepareExport",
            serde_json::json!({"filename": filename}),
        )
        .map_err(|e| e.to_string())?;
    let file = path.join(&filename);
    let partial = path.join(format!("{filename}.partial"));
    let app_for_export = app.clone();
    let destination = file.clone();
    #[cfg(target_os = "android")]
    let exporting_account = expected.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let result = (|| {
            let version = app_for_export.package_info().version.to_string();
            #[cfg(target_os = "android")]
            {
                let core = core(&app_for_export)?;
                core.export_for_account(&partial, &version, &exporting_account)
                    .map_err(|e| e.to_string())?;
                if !session_ready(&app_for_export)? {
                    return Err("Account changed during export".into());
                }
                publish_bound_archive(&core, &exporting_account, || {
                    publish_archive_no_replace(&partial, &destination)
                })
            }
            #[cfg(not(target_os = "android"))]
            {
                core(&app_for_export)?.export(&partial, &version).map_err(|e| e.to_string())?;
                publish_archive_no_replace(&partial, &destination)
            }
        })();
        if result.is_err() {
            let _ = std::fs::remove_file(partial);
        }
        result
    })
    .await
    .map_err(|e| e.to_string())??;
    #[cfg(target_os = "android")]
    {
        // An IPC error can arrive after the chooser started; its URI must remain readable.
        vault(&app)
            .0
            .run_mobile_plugin::<serde_json::Value>(
                "shareExport",
                serde_json::json!({"filename": filename, "accountId": expected}),
            )
            .map_err(|e| e.to_string())?;
        Ok("Choose where to share the export".into())
    }
    #[cfg(not(target_os = "android"))]
    {
        let result = (|| {
            let downloads = app.path().download_dir().map_err(|e| e.to_string())?;
            let destination = save_archive_to_downloads(&file, &downloads, &filename)?;
            let _ = std::fs::remove_file(&file);
            Ok(format!("Saved to {}", destination.display()))
        })();
        if result.is_err() {
            let _ = std::fs::remove_file(file);
        }
        result
    }
}

#[cfg(target_os = "android")]
fn choose_archive(app: &tauri::AppHandle) -> Result<ChosenImport, String> {
    #[derive(Deserialize)]
    struct Picked {
        filename: String,
    }
    if !session_ready(app)? {
        return Err("Account verification required".into());
    }
    let picked = vault(app)
        .0
        .run_mobile_plugin::<Picked>("pickImport", serde_json::json!({}))
        .map_err(|e| e.to_string())?;
    let path = app
        .path()
        .app_cache_dir()
        .map_err(|e| e.to_string())?
        .join("imports")
        .join(&picked.filename);
    Ok(ChosenImport { path, cleanup: Some(picked.filename) })
}

// The desktop chooser runs on the GTK main thread, where the window lives.
#[cfg(not(target_os = "android"))]
fn choose_archive(app: &tauri::AppHandle) -> Result<ChosenImport, String> {
    let (sender, receiver) = std::sync::mpsc::channel();
    app.run_on_main_thread(move || {
        let picked = rfd::FileDialog::new()
            .set_title("Choose a Raring export")
            .add_filter("Raring export", &["zip"])
            .pick_file();
        let _ = sender.send(picked);
    })
    .map_err(|e| e.to_string())?;
    let path = receiver
        .recv_timeout(std::time::Duration::from_secs(600))
        .map_err(|_| "The file chooser was closed".to_owned())?
        .ok_or_else(|| "No file chosen".to_owned())?;
    Ok(ChosenImport { path, cleanup: None })
}

fn discard_import(app: &tauri::AppHandle, chosen: &ChosenImport) {
    #[cfg(target_os = "android")]
    if let Some(filename) = &chosen.cleanup {
        let _ = vault(app).0.run_mobile_plugin::<serde_json::Value>(
            "discardImport",
            serde_json::json!({"filename": filename}),
        );
    }
    #[cfg(not(target_os = "android"))]
    {
        // A desktop choice is the person's own file and is never deleted.
        let _ = app;
        debug_assert!(chosen.cleanup.is_none());
    }
}

/// Pick an export with the platform's own chooser and validate it. Nothing is
/// written: the chosen file waits until the person confirms the preview.
#[tauri::command]
async fn choose_import(app: tauri::AppHandle) -> Result<ImportPreview, String> {
    let previous = app.state::<ImportState>().0.lock().map_err(|e| e.to_string())?.take();
    if let Some(previous) = previous {
        let handle = app.clone();
        tauri::async_runtime::spawn_blocking(move || discard_import(&handle, &previous))
            .await
            .map_err(|e| e.to_string())?;
    }
    let handle = app.clone();
    let chosen = tauri::async_runtime::spawn_blocking(move || choose_archive(&handle))
        .await
        .map_err(|e| e.to_string())??;
    let handle = app.clone();
    let path = chosen.path.clone();
    let preview = tauri::async_runtime::spawn_blocking(move || -> Result<ImportPreview, String> {
        let core = core(&handle)?;
        core.import_preview(&path, core.session_identity()).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?;
    match preview {
        Ok(preview) => {
            *app.state::<ImportState>().0.lock().map_err(|e| e.to_string())? = Some(chosen);
            Ok(preview)
        }
        Err(error) => {
            let handle = app.clone();
            tauri::async_runtime::spawn_blocking(move || discard_import(&handle, &chosen))
                .await
                .map_err(|e| e.to_string())?;
            Err(error)
        }
    }
}

/// Apply the previewed archive once, under the core's account barrier.
#[tauri::command]
async fn apply_import(app: tauri::AppHandle) -> Result<ImportPreview, String> {
    let chosen = app
        .state::<ImportState>()
        .0
        .lock()
        .map_err(|e| e.to_string())?
        .clone()
        .ok_or("Choose a Raring export first")?;
    let handle = app.clone();
    let path = chosen.path.clone();
    let result = tauri::async_runtime::spawn_blocking(move || -> Result<ImportPreview, String> {
        let core = core(&handle)?;
        core.import_settings(&path, core.session_identity()).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?;
    // A refused import keeps the file chosen, so the person can retry or pick another.
    let preview = result?;
    *app.state::<ImportState>().0.lock().map_err(|e| e.to_string())? = None;
    let handle = app.clone();
    let done = chosen.clone();
    tauri::async_runtime::spawn_blocking(move || discard_import(&handle, &done))
        .await
        .map_err(|e| e.to_string())?;
    // Imported rest, pauses and limits change the native surfaces offline.
    #[cfg(target_os = "android")]
    vault(&app)
        .0
        .run_mobile_plugin_async::<serde_json::Value>("applyStatusSettings", serde_json::json!({}))
        .await
        .map_err(|e| e.to_string())?;
    Ok(preview)
}

#[tauri::command]
async fn get_settings(app: tauri::AppHandle) -> Result<Settings, String> {
    read_view(app, |core| Ok(core.settings())).await
}

#[tauri::command]
async fn patch_settings(app: tauri::AppHandle, patch: raring_core::settings::SettingsPatch) -> Result<(), String> {
    // Merge the patch only inside the existing write barrier, never before queueing.
    let expected = core(&app)?.bound_account_identity();
    let handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || core(&handle)?.patch_settings_for_account(patch, expected).map_err(|e| e.to_string()))
        .await.map_err(|e| e.to_string())??;
    // Complete opt-out cancellation before acknowledging the save. This command
    // reads authoritative local policy and never waits for the network SyncGate.
    #[cfg(target_os = "android")]
    vault(&app).0.run_mobile_plugin_async::<serde_json::Value>(
        "applyStatusSettings", serde_json::json!({})
    ).await.map_err(|e| e.to_string())?;
    // The bedtime switch and wind-down lead move tonight's reminder.
    #[cfg(target_os = "android")]
    std::thread::spawn(move || {
        let _ = vault(&app)
            .0
            .run_mobile_plugin::<serde_json::Value>("schedule", serde_json::json!({}));
    });
    Ok(())
}

#[tauri::command]
async fn local_action(app: tauri::AppHandle, action: raring_core::view::LocalAction) -> Result<(), String> {
    // Like settings saves: validated and applied inside the account barrier,
    // refused if the account changed after the change was queued.
    let expected = core(&app)?.bound_account_identity();
    let handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || core(&handle)?.local_action_for_account(&action, Local::now(), expected).map_err(|e| e.to_string()))
        .await.map_err(|e| e.to_string())??;
    // The core already rewrote the snapshot offline; redraw surfaces from it.
    #[cfg(target_os = "android")]
    vault(&app).0.run_mobile_plugin_async::<serde_json::Value>(
        "applyStatusSettings", serde_json::json!({})
    ).await.map_err(|e| e.to_string())?;
    Ok(())
}

/// Debug only: the cached account is the synthetic demo, so the emulator
/// checks may re-seed it.
#[cfg(debug_assertions)]
#[tauri::command]
fn demo_session(app: tauri::AppHandle) -> Result<bool, String> {
    Ok(core(&app)?.is_demo_session())
}

#[cfg(debug_assertions)]
#[tauri::command]
fn load_demo_data(app: tauri::AppHandle, planned: Option<bool>) -> Result<String, String> {
    let date = Local::now().date_naive().to_string();
    let store = raring_core::store::Store::open(database_path(&app)?).map_err(|e| e.to_string())?;
    raring_core::demo::seed_variant(&store, Local::now().date_naive(), !planned.unwrap_or(false)).map_err(|e| e.to_string())?;
    // Explicit debug-only synthetic binding; never used by sign-in or production sync.
    let profile = serde_json::json!({"profileId":900000000001_u64,"displayName":"sample-demo"});
    store.put("profile", "me", &Local::now().to_rfc3339(), &profile).map_err(|e| e.to_string())?;
    store.set_setting("account_session", &serde_json::json!({
        "identity":raring_core::account::profile_identity(&profile),"active":true
    }).to_string()).map_err(|e| e.to_string())?;
    #[cfg(target_os = "android")]
    vault(&app).0.run_mobile_plugin::<serde_json::Value>("demoSession", serde_json::json!({})).map_err(|e| e.to_string())?;
    let _ = app.emit("status-changed", status_read(app.clone())?);
    Ok(format!("{}:{}", date, raring_core::demo::WORKOUT_ID))
}

#[cfg(target_os = "android")]
mod background;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .manage(SignInState::default())
        .manage(SignInPending::default())
        .manage(ImportState::default());
    #[cfg(not(target_os = "android"))]
    let builder = builder.manage(DesktopCredentials::default());
    #[cfg(desktop)]
    let builder = builder.plugin(tauri_plugin_notification::init());
    builder
        .plugin(
            tauri::plugin::Builder::<tauri::Wry>::new("vault")
                .setup(|app, api| {
                    #[cfg(target_os = "android")]
                    app.manage(AndroidVault(
                        api.register_android_plugin("app.raring", "VaultPlugin")?,
                    ));
                    #[cfg(not(target_os = "android"))]
                    {
                        let _ = (app, api);
                    }
                    Ok(())
                })
                .build(),
        )
        .plugin(
            tauri::plugin::Builder::<tauri::Wry>::new("garmin-navigation")
                .on_navigation(|webview, url| {
                    if webview.label() != "main" {
                        return true;
                    }
                    let app = webview.app_handle().clone();
                    let candidate = sso::extract_ticket(url.as_str()).and_then(|ticket| {
                        app.state::<SignInPending>().0.lock().ok()?.take()
                            .map(|generation| (ticket, generation))
                    });
                    if let Some((ticket, generation)) = candidate {
                        let window = webview.clone();
                        let app = webview.app_handle().clone();
                        std::thread::spawn(move || {
                            let _ = window.navigate(home_url());
                            exchange_ticket_and_store(&app, ticket, generation);
                        });
                        return false;
                    }
                    if url.as_str().starts_with(home_url().as_str()) {
                        if let Ok(mut state) = app.state::<SignInPending>().0.lock() {
                            *state = None;
                        }
                    }
                    true
                })
                .build(),
        )
        .invoke_handler({
            #[cfg(debug_assertions)]
            {
                tauri::generate_handler![
                    app_status,
                    today,
                    sleep,
                    training,
                    workout,
                    strength,
                    health,
                    checkin,
                    trend,
                    local_action,
                    plan_board,
                    week_review,
                    reviews,
                    progress,
                    sync_now,
                    begin_sign_in,
                    sign_out,
                    open_garmin_connect,
                    request_notification_permission,
                    get_settings,
                    garmin_changes,
                    patch_settings,
                    heart_rate_day,
                    refresh_strength,
                    fetch_day_detail,
                    fetch_workout_detail,
                    export_archive,
                    choose_import,
                    apply_import,
                    weight_trend,
                    profile,
                    progress_domains,
                    milestone_archive,
                    lift_series,
                    muscles,
                    check_reminders,
                    reminders_status,
                    load_demo_data,
                    demo_session
                ]
            }
            #[cfg(not(debug_assertions))]
            {
                tauri::generate_handler![
                    app_status,
                    today,
                    sleep,
                    training,
                    workout,
                    strength,
                    health,
                    checkin,
                    trend,
                    local_action,
                    plan_board,
                    week_review,
                    reviews,
                    progress,
                    sync_now,
                    begin_sign_in,
                    sign_out,
                    open_garmin_connect,
                    request_notification_permission,
                    get_settings,
                    garmin_changes,
                    patch_settings,
                    heart_rate_day,
                    refresh_strength,
                    fetch_day_detail,
                    fetch_workout_detail,
                    export_archive,
                    choose_import,
                    apply_import,
                    weight_trend,
                    profile,
                    progress_domains,
                    milestone_archive,
                    lift_series,
                    muscles,
                    check_reminders,
                    reminders_status
                ]
            }
        })
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            #[cfg(target_os = "android")]
            {
                #[cfg(debug_assertions)]
                println!(
                    "Android chrono Local: {} ({})",
                    Local::now().to_rfc3339(),
                    Local::now().offset()
                );
                let handle = app.handle().clone();
                std::thread::spawn(move || {
                    let _ = vault(&handle)
                        .0
                        .run_mobile_plugin::<serde_json::Value>("schedule", serde_json::json!({}));
                });
            }
            #[cfg(desktop)]
            desktop_reminders::start(app.handle().clone());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod export_tests {
    use super::*;
    use std::{sync::{mpsc, Arc}, thread};

    fn bound_fixture() -> (Arc<Raring>, std::path::PathBuf, std::path::PathBuf, std::path::PathBuf) {
        let mut random = [0u8; 16];
        getrandom::fill(&mut random).unwrap();
        let directory = std::env::temp_dir().join(format!("raring-bound-publish-{:x}", u128::from_le_bytes(random)));
        std::fs::create_dir(&directory).unwrap();
        let database = directory.join("data.db");
        let store = raring_core::store::Store::open(&database).unwrap();
        assert!(store.bind_account("11", &serde_json::json!({"profileId":11,"displayName":"A"}), "now").unwrap());
        assert!(store.activate_account().unwrap());
        store.put("summary", "2026-01-02", "now", &serde_json::json!({"marker":"A"})).unwrap();
        let core = Arc::new(Raring::open(&database).unwrap());
        let partial = directory.join("staged.partial");
        core.export_for_account(&partial, "test", "11").unwrap();
        (core, partial, directory.join("archive.zip"), directory)
    }

    #[test]
    fn bound_publication_serializes_final_check_and_rename_with_transition() {
        let (core, partial, destination, directory) = bound_fixture();
        let expected_bytes = std::fs::read(&partial).unwrap();
        let (at_guard, guarded) = mpsc::channel();
        let (release, released) = mpsc::channel();
        let (transition_start, starting) = mpsc::channel();
        let (transition_done, finished) = mpsc::channel();
        let publisher = {
            let core = Arc::clone(&core);
            let partial = partial.clone();
            let destination = destination.clone();
            thread::spawn(move || publish_bound_archive(&core, "11", || {
                at_guard.send(()).unwrap();
                released.recv().unwrap();
                publish_archive_no_replace(&partial, &destination)
            }))
        };
        guarded.recv().unwrap();
        let transition = {
            let core = Arc::clone(&core);
            let destination = destination.clone();
            thread::spawn(move || {
                transition_start.send(()).unwrap();
                core.suspend_session().unwrap();
                assert!(destination.exists(), "transition crossed the check/rename gap");
                transition_done.send(()).unwrap();
            })
        };
        starting.recv().unwrap();
        // The publisher is paused inside the critical section. A transition
        // must remain blocked until we explicitly release the rename.
        let blocked = matches!(finished.recv_timeout(std::time::Duration::from_millis(250)), Err(mpsc::RecvTimeoutError::Timeout));
        release.send(()).unwrap();
        assert!(blocked);
        publisher.join().unwrap().unwrap();
        transition.join().unwrap();
        finished.recv().unwrap();
        assert!(!partial.exists());
        assert_eq!(std::fs::read(&destination).unwrap(), expected_bytes);
        assert!(core.session_identity().is_none());
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn transition_before_publication_guard_rejects_without_destination() {
        let (core, partial, destination, directory) = bound_fixture();
        core.suspend_session().unwrap();
        assert!(publish_bound_archive(&core, "11", || publish_archive_no_replace(&partial, &destination)).is_err());
        assert!(partial.exists());
        assert!(!destination.exists());
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn publishing_and_desktop_saving_never_replace_an_existing_export() {
        let mut random = [0u8; 16];
        getrandom::fill(&mut random).unwrap();
        let directory =
            std::env::temp_dir().join(format!("raring-publish-{:x}", u128::from_le_bytes(random)));
        std::fs::create_dir(&directory).unwrap();
        let source = directory.join("staged.partial");
        let destination = directory.join("archive.zip");
        std::fs::write(&source, b"new").unwrap();
        std::fs::write(&destination, b"existing").unwrap();
        assert!(publish_archive_no_replace(&source, &destination).is_err());
        assert_eq!(std::fs::read(&destination).unwrap(), b"existing");
        assert_eq!(std::fs::read(&source).unwrap(), b"new");
        #[cfg(not(target_os = "android"))]
        {
            assert!(save_archive_to_downloads(&source, &directory, "archive.zip").is_err());
            assert_eq!(std::fs::read(&destination).unwrap(), b"existing");
            assert!(!directory.join("archive.zip.partial").exists());
            let saved = save_archive_to_downloads(&source, &directory, "another.zip").unwrap();
            assert_eq!(std::fs::read(saved).unwrap(), b"new");
        }
        std::fs::remove_file(&destination).unwrap();
        publish_archive_no_replace(&source, &destination).unwrap();
        assert!(!source.exists());
        assert_eq!(std::fs::read(destination).unwrap(), b"new");
        std::fs::remove_dir_all(directory).unwrap();
    }
}

#[cfg(test)]
mod handler_lists {
    /// Debug builds add only the demo commands; every other command must be
    /// registered in the release list too, or release screens fail to load.
    #[test]
    fn release_registers_every_non_demo_command() {
        let src = include_str!("lib.rs");
        let list = |marker: &str| -> Vec<String> {
            let at = src.find(marker).expect("handler list");
            let body = &src[at..];
            let open = body.find("generate_handler![").unwrap() + "generate_handler![".len();
            let close = body[open..].find(']').unwrap();
            body[open..open + close].split(',').map(|s| s.trim().to_owned()).filter(|s| !s.is_empty()).collect()
        };
        let debug = list("#[cfg(debug_assertions)]\n            {");
        let release = list("#[cfg(not(debug_assertions))]\n            {");
        assert!(debug.len() > 20 && release.len() > 20, "both lists were found");
        let missing: Vec<_> = debug.iter().filter(|c| !release.contains(c) && !["load_demo_data", "demo_session"].contains(&c.as_str())).collect();
        assert!(missing.is_empty(), "missing from the release list: {missing:?}");
    }
}
