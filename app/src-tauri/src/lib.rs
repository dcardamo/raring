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
struct SignInPending(Mutex<bool>);

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

#[allow(dead_code)]
fn save_tokens(app: &tauri::AppHandle, value: &str) -> Result<(), String> {
    #[cfg(target_os = "android")]
    {
        vault(app)
            .0
            .run_mobile_plugin::<serde_json::Value>(
                "saveTokens",
                serde_json::json!({ "value": value }),
            )
            .map_err(|e| e.to_string())?;
    }
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
    let mut status = core(&app)?.status();
    status.signed_in = load_tokens(&app)?.is_some();
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
    read: impl FnOnce(Raring) -> Result<T, String> + Send + 'static,
) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(move || read(core(&app)?))
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
        Ok(core.health(&date.unwrap_or_else(|| core.current_date(Local::now()))))
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
        return vault(&app)
            .0
            .run_mobile_plugin_async::<Response>("heartRateDay", serde_json::json!({"date":date}))
            .await
            .map(|r| r.day)
            .map_err(|e| e.to_string());
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
fn begin_sign_in(app: tauri::AppHandle) -> Result<(), String> {
    #[cfg(not(target_os = "android"))]
    app.state::<DesktopCredentials>().invalidate_sign_in();
    *app.state::<SignInPending>()
        .0
        .lock()
        .map_err(|e| e.to_string())? = true;
    app.get_webview_window("main")
        .ok_or("main window missing")?
        .navigate(tauri::Url::parse(&sso::sign_in_url()).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())
}

fn exchange_ticket_and_store(app: &tauri::AppHandle, ticket: String, generation: Option<u64>) {
    let outcome = runtime().and_then(|rt| {
        let tokens = rt
            .block_on(Raring::sign_in_with_ticket(&ticket))
            .map_err(|_| "Garmin sign-in failed — try again".to_owned())?;
        let json = serde_json::to_string(&tokens).map_err(|e| e.to_string())?;
        #[cfg(not(target_os = "android"))]
        return rt.block_on(
            app.state::<DesktopCredentials>()
                .save_sign_in(generation.ok_or("missing sign-in generation")?, || {
                    save_tokens(app, &json)
                }),
        );
        #[cfg(target_os = "android")]
        {
            let _ = generation;
            save_tokens(app, &json).map(|_| true)
        }
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
    #[cfg(not(target_os = "android"))]
    app.state::<DesktopCredentials>()
        .sign_out(|| {
            clear_tokens(&app)?;
            core(&app)?.forget_account().map_err(|e| e.to_string())
        })
        .await?;
    #[cfg(target_os = "android")]
    {
        clear_tokens(&app)?;
        core(&app)?.forget_account().map_err(|e| e.to_string())?;
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

#[tauri::command]
async fn export_archive(app: tauri::AppHandle) -> Result<String, String> {
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
    tauri::async_runtime::spawn_blocking(move || {
        let result = (|| {
            let version = app_for_export.package_info().version.to_string();
            core(&app_for_export)?
                .export(&partial, &version)
                .map_err(|e| e.to_string())?;
            publish_archive_no_replace(&partial, &destination)
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
                serde_json::json!({"filename": filename}),
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
fn get_settings(app: tauri::AppHandle) -> Result<Settings, String> {
    Ok(core(&app)?.settings())
}

#[tauri::command]
fn set_settings(app: tauri::AppHandle, settings: Settings) -> Result<(), String> {
    core(&app)?
        .set_settings(settings)
        .map_err(|e| e.to_string())?;
    // The bedtime switch and wind-down lead move tonight's reminder.
    #[cfg(target_os = "android")]
    std::thread::spawn(move || {
        let _ = vault(&app)
            .0
            .run_mobile_plugin::<serde_json::Value>("schedule", serde_json::json!({}));
    });
    Ok(())
}

#[cfg(debug_assertions)]
#[tauri::command]
fn load_demo_data(app: tauri::AppHandle) -> Result<String, String> {
    let date = Local::now().date_naive().to_string();
    let store = raring_core::store::Store::open(database_path(&app)?).map_err(|e| e.to_string())?;
    raring_core::demo::seed(&store, Local::now().date_naive()).map_err(|e| e.to_string())?;
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
                    let pending = app.state::<SignInPending>().0.lock().is_ok_and(|s| *s);
                    if let Some(ticket) = sso::extract_ticket(url.as_str()).filter(|_| pending) {
                        if let Ok(mut state) = app.state::<SignInPending>().0.lock() {
                            *state = false;
                        }
                        let window = webview.clone();
                        let app = webview.app_handle().clone();
                        let generation = {
                            #[cfg(not(target_os = "android"))]
                            {
                                Some(app.state::<DesktopCredentials>().generation())
                            }
                            #[cfg(target_os = "android")]
                            {
                                None
                            }
                        };
                        std::thread::spawn(move || {
                            let _ = window.navigate(home_url());
                            exchange_ticket_and_store(&app, ticket, generation);
                        });
                        return false;
                    }
                    if url.as_str().starts_with(home_url().as_str()) {
                        if let Ok(mut state) = app.state::<SignInPending>().0.lock() {
                            *state = false;
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
                    sync_now,
                    begin_sign_in,
                    sign_out,
                    open_garmin_connect,
                    request_notification_permission,
                    get_settings,
                    set_settings,
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
                    sync_now,
                    begin_sign_in,
                    sign_out,
                    open_garmin_connect,
                    request_notification_permission,
                    get_settings,
                    set_settings,
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
