use chrono::{Datelike, Local};
#[cfg(not(target_os = "android"))]
mod desktop_credentials;
#[cfg(not(target_os = "android"))]
use desktop_credentials::DesktopCredentials;
#[cfg(not(target_os = "android"))]
use raring_core::Tokens;
use raring_core::{garmin::sso, view::*, Raring};
#[cfg(target_os = "android")]
use serde::Deserialize;
use std::sync::Mutex;
use tauri::{Emitter, Manager};

#[derive(Default)]
struct SignInState(Mutex<Option<String>>);
#[derive(Default)]
struct SignInPending(Mutex<Option<u64>>);

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
        Ok(core.sleep(&date.unwrap_or_else(|| core.current_date(Local::now()))))
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
        core.workout(&id).ok_or_else(|| "not_found".into())
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
#[tauri::command]
async fn plan(app: tauri::AppHandle, date: Option<String>) -> Result<PlanView, String> {
    read_view(app, move |core| {
        Ok(core.plan(&date.unwrap_or_else(|| core.current_date(Local::now()))))
    })
    .await
}
/// Saves to the week the plan screen shows.
#[tauri::command]
async fn set_plan(app: tauri::AppHandle, update: PlanUpdate) -> Result<(), String> {
    read_view(app, move |core| {
        core.set_plan(&core.current_date(Local::now()), &update)
            .map_err(|e| e.to_string())
    })
    .await
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
    #[cfg(not(target_os = "android"))]
    {
        let sync_core = core(&app)?;
        if let Some(notifications) = with_desktop_tokens(&app, |tokens| async move {
            let outcome = sync_core
                .sync(tokens, Local::now(), 60)
                .await
                .map_err(|e| e.to_string())?;
            if let Some(error) = &outcome.snapshot_error { log::warn!("status snapshot publication failed: {error}"); }
            Ok((outcome.notifications, outcome.tokens))
        })
        .await?
        {
            core(&app)?
                .mark_notifications_sent(&notifications, Local::now())
                .map_err(|e| e.to_string())?;
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
        Ok(false)
    }
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

#[cfg(debug_assertions)]
#[tauri::command]
fn load_demo_data(app: tauri::AppHandle) -> Result<String, String> {
    let date = Local::now().date_naive().to_string();
    let store = raring_core::store::Store::open(database_path(&app)?).map_err(|e| e.to_string())?;
    raring_core::demo::seed(&store, Local::now().date_naive()).map_err(|e| e.to_string())?;
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
        .manage(SignInPending::default());
    #[cfg(not(target_os = "android"))]
    let builder = builder.manage(DesktopCredentials::default());
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
                    plan,
                    set_plan,
                    local_action,
                    sync_now,
                    begin_sign_in,
                    sign_out,
                    open_garmin_connect,
                    request_notification_permission,
                    get_settings,
                    patch_settings,
                    heart_rate_day,
                    export_archive,
                    load_demo_data
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
                    plan,
                    set_plan,
                    local_action,
                    sync_now,
                    begin_sign_in,
                    sign_out,
                    open_garmin_connect,
                    request_notification_permission,
                    get_settings,
                    patch_settings,
                    heart_rate_day,
                    export_archive
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
