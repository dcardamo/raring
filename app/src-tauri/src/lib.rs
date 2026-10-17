use chrono::{Datelike, Local};
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
        if date.is_none() {
            core.capture_sleep_plan(Local::now()).map_err(|e| e.to_string())?;
        }
        Ok(core.today(&date.unwrap_or_else(|| core.current_date(Local::now()))))
    })
    .await
}
#[tauri::command]
async fn sleep(app: tauri::AppHandle, date: Option<String>) -> Result<SleepView, String> {
    read_view(app, move |core| {
        if date.is_none() {
            core.capture_sleep_plan(Local::now()).map_err(|e| e.to_string())?;
        }
        Ok(core.sleep(&date.unwrap_or_else(|| core.current_date(Local::now()))))
    })
    .await
}
#[tauri::command]
async fn training(app: tauri::AppHandle, date: Option<String>) -> Result<TrainingView, String> {
    read_view(app, move |core| {
        Ok(core.training(&date.unwrap_or_else(|| core.current_date(Local::now()))))
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
async fn health(app: tauri::AppHandle, date: Option<String>) -> Result<HealthView, String> {
    read_view(app, move |core| {
        Ok(core.health(&date.unwrap_or_else(|| core.current_date(Local::now()))))
    })
    .await
}
#[tauri::command]
async fn checkin(app: tauri::AppHandle, week_start: Option<String>) -> Result<CheckinView, String> {
    read_view(app, move |core| {
        Ok(core.checkin(&week_start.unwrap_or_else(monday)))
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
    *app.state::<SignInPending>()
        .0
        .lock()
        .map_err(|e| e.to_string())? = true;
    app.get_webview_window("main")
        .ok_or("main window missing")?
        .navigate(tauri::Url::parse(&sso::sign_in_url()).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())
}

fn exchange_ticket_and_store(app: &tauri::AppHandle, ticket: String) {
    let outcome = runtime()
        .and_then(|rt| {
            rt.block_on(Raring::sign_in_with_ticket(&ticket))
                .map_err(|_| "Garmin sign-in failed — try again".to_owned())
        })
        .and_then(|tokens| {
            let json = serde_json::to_string(&tokens).map_err(|e| e.to_string())?;
            save_tokens(app, &json)
        });
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
fn sign_out(app: tauri::AppHandle) -> Result<(), String> {
    clear_tokens(&app)?;
    core(&app)?.forget_account().map_err(|e| e.to_string())?;
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
    if let Some(json) = load_tokens(&app)? {
        let tokens: Tokens = serde_json::from_str(&json).map_err(|e| e.to_string())?;
        let outcome = core(&app)?
            .sync(tokens, Local::now(), 60)
            .await
            .map_err(|e| e.to_string())?;
        save_tokens(
            &app,
            &serde_json::to_string(&outcome.tokens).map_err(|e| e.to_string())?,
        )?;
        core(&app)?
            .mark_notifications_sent(&outcome.notifications, Local::now())
            .map_err(|e| e.to_string())?;
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
    let store =
        raring_core::store::Store::open(database_path(&app)?).map_err(|e| e.to_string())?;
    raring_core::demo::seed(&store, Local::now().date_naive()).map_err(|e| e.to_string())?;
    let _ = app.emit("status-changed", status_read(app.clone())?);
    Ok(format!("{}:{}", date, raring_core::demo::WORKOUT_ID))
}

#[cfg(target_os = "android")]
mod background;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(SignInState::default())
        .manage(SignInPending::default())
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
                        std::thread::spawn(move || {
                            let _ = window.navigate(home_url());
                            exchange_ticket_and_store(&app, ticket);
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
                    health,
                    checkin,
                    plan,
                    set_plan,
                    sync_now,
                    begin_sign_in,
                    sign_out,
                    open_garmin_connect,
                    request_notification_permission,
                    get_settings,
                    set_settings,
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
                    health,
                    checkin,
                    plan,
                    set_plan,
                    sync_now,
                    begin_sign_in,
                    sign_out,
                    open_garmin_connect,
                    request_notification_permission,
                    get_settings,
                    set_settings
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
