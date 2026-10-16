use pepometer_core::{garmin::sso, view::*};
#[cfg(target_os = "android")]
use serde::Deserialize;
use std::sync::Mutex;
use tauri::{Emitter, Manager};

#[derive(Default)]
struct SignInState(Mutex<Option<String>>);

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
        use std::os::unix::fs::OpenOptionsExt;
        let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .mode(0o600)
            .open(dir.join("tokens.json"))
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

#[tauri::command]
fn app_status(app: tauri::AppHandle) -> Result<AppStatus, String> {
    Ok(AppStatus {
        signed_in: load_tokens(&app)?.is_some(),
        syncing: false,
        last_sync: None,
        history_days: 0,
        last_error: None,
    })
}

macro_rules! not_ready {
    ($name:ident, $ty:ty, $($arg:ident : $arg_ty:ty),*) => {
        #[tauri::command]
        fn $name($($arg: $arg_ty),*) -> Result<$ty, String> {
            $(let _ = $arg;)*
            Err("not_ready".into())
        }
    };
}
not_ready!(today, TodayView, date: Option<String>);
not_ready!(sleep, SleepView, date: Option<String>);
not_ready!(training, TrainingView, date: Option<String>);
not_ready!(workout, WorkoutView, id: String);
not_ready!(health, HealthView, date: Option<String>);
not_ready!(checkin, Option<CheckinView>, week_start: Option<String>);

#[tauri::command]
fn begin_sign_in(app: tauri::AppHandle) -> Result<(), String> {
    app.get_webview_window("main")
        .ok_or("main window missing")?
        .navigate(tauri::Url::parse(&sso::sign_in_url()).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())
}

// Integration point: exchange the CAS ticket with the core, save the resulting
// token JSON with save_tokens, schedule work, then emit status-changed.
fn exchange_ticket_and_store(app: &tauri::AppHandle, ticket: String) {
    let state = app.state::<SignInState>();
    if let Ok(mut pending) = state.0.lock() {
        *pending = Some(ticket);
    }
    log::info!("garmin ticket received");
    if let Ok(status) = app_status(app.clone()) {
        let _ = app.emit("status-changed", status);
    }
}

#[tauri::command]
fn sign_out(app: tauri::AppHandle) -> Result<(), String> {
    clear_tokens(&app)?;
    let _ = app.emit("status-changed", app_status(app.clone())?);
    Ok(())
}

#[tauri::command]
fn sync_now(app: tauri::AppHandle) -> Result<AppStatus, String> {
    let status = app_status(app.clone())?;
    let _ = app.emit("status-changed", &status);
    Ok(status)
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

// Integration point: replace local JSON with core facade settings access.
#[allow(dead_code)]
fn database_path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    let root = app.path().app_data_dir().map_err(|e| e.to_string())?;
    // Tauri Android's app_data_dir is Context.dataDir, NOT Context.filesDir.
    #[cfg(target_os = "android")]
    let root = root.join("files");
    Ok(root.join("pepometer.db"))
}

fn settings_path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    Ok(app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("settings.json"))
}

#[tauri::command]
fn get_settings(app: tauri::AppHandle) -> Result<Settings, String> {
    let path = settings_path(&app)?;
    match std::fs::read(path) {
        Ok(bytes) => serde_json::from_slice(&bytes).map_err(|e| e.to_string()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Settings::default()),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
fn set_settings(app: tauri::AppHandle, settings: Settings) -> Result<(), String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    std::fs::write(
        settings_path(&app)?,
        serde_json::to_vec(&settings).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}

#[cfg(target_os = "android")]
mod background;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(SignInState::default())
        .plugin(
            tauri::plugin::Builder::<tauri::Wry>::new("vault")
                .setup(|app, api| {
                    #[cfg(target_os = "android")]
                    app.manage(AndroidVault(
                        api.register_android_plugin("ca.hld.pepometer", "VaultPlugin")?,
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
                    if let Some(ticket) = sso::extract_ticket(url.as_str()) {
                        let window = webview.clone();
                        let app = webview.app_handle().clone();
                        std::thread::spawn(move || {
                            exchange_ticket_and_store(&app, ticket);
                            let _ = window.navigate(home_url());
                        });
                        return false;
                    }
                    true
                })
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            app_status,
            today,
            sleep,
            training,
            workout,
            health,
            checkin,
            sync_now,
            begin_sign_in,
            sign_out,
            request_notification_permission,
            get_settings,
            set_settings
        ])
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
                vault(app.handle())
                    .0
                    .run_mobile_plugin::<serde_json::Value>("schedule", serde_json::json!({}))?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
