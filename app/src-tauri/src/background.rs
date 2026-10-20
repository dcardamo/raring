use chrono::{DateTime, Duration, Local, NaiveDateTime, Utc};
use jni::{
    objects::{JClass, JString},
    JNIEnv,
};
use raring_core::{Raring, Tokens};
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SyncResult {
    tokens_json: String,
    readiness_missing: bool,
    auth_failed: bool,
    notifications: Vec<Notification>,
    error: Option<String>,
}

#[derive(Serialize)]
struct Notification {
    id: String,
    title: String,
    body: String,
    target: &'static str,
    kind: String,
    tag: String,
}

fn notification_target(kind: &str) -> &'static str {
    match kind {
        "morning" => "today",
        "weekly" => "checkin",
        "vitals" | "fitness-age" | "vo2max" => "health",
        "training-status" => "training",
        "bedtime" => "sleep",
        _ => "today",
    }
}

fn run_background_sync(db_path: &str, tokens_json: &str, now_iso: &str) -> SyncResult {
    let mut result = SyncResult {
        tokens_json: tokens_json.to_owned(),
        readiness_missing: true,
        auth_failed: false,
        notifications: vec![],
        error: None,
    };
    let tokens: Tokens = match serde_json::from_str(tokens_json) {
        Ok(t) => t,
        Err(_) => {
            result.auth_failed = true;
            return result;
        }
    };
    let now = match DateTime::parse_from_rfc3339(now_iso) {
        Ok(t) => t.with_timezone(&Local),
        Err(_) => {
            result.error = Some("invalid sync time".into());
            return result;
        }
    };
    // Verify Android's timezone resolution against the offset sent by Kotlin.
    log::info!(
        "Android local offset: {} (provided: {})",
        now.offset(),
        now_iso
    );
    let core = match Raring::open(db_path) {
        Ok(c) => c,
        Err(e) => {
            result.error = Some(e.to_string());
            return result;
        }
    };
    if core.session_identity().is_none() {
        result.auth_failed = true;
        return result;
    }
    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            result.error = Some(e.to_string());
            return result;
        }
    };
    let budget = if core.status().last_sync.is_none() {
        1400
    } else {
        60
    };
    match runtime.block_on(core.sync(tokens, now, budget)) {
        Ok(outcome) => {
            if let Some(error) = &outcome.snapshot_error {
                log::warn!("status snapshot publication failed (offline retry): {error}");
            }
            result.tokens_json = serde_json::to_string(&outcome.tokens).unwrap_or_default();
            result.readiness_missing = !outcome.report.readiness_present;
            result.auth_failed = outcome.report.auth_failed;
            // Kotlin's bedtime check posts reminders under one lock after every sync.
            result.notifications = outcome
                .notifications
                .into_iter()
                .filter(|item| item.kind != "bedtime")
                .map(|item| Notification {
                    id: format!("{}:{}", item.kind, item.tag),
                    title: item.title,
                    body: item.body,
                    target: notification_target(&item.kind),
                    kind: item.kind,
                    tag: item.tag,
                })
                .collect();
            log::info!(
                "background JNI sync completed (requests: {}, auth failed: {})",
                outcome.report.requests,
                result.auth_failed
            );
            if let Some(error) = outcome.report.error {
                if !result.auth_failed {
                    result.error = Some(error);
                }
            }
        }
        Err(raring_core::Error::Garmin(raring_core::garmin::Error::NeedsSignIn)) => {
            result.auth_failed = true;
        }
        Err(e) => result.error = Some(e.to_string()),
    }
    result
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_app_raring_NativeBridge_exchangeTicket(
    mut env: JNIEnv<'_>, _class: JClass<'_>, ticket: JString<'_>, test_api: JString<'_>,
) -> jni::sys::jstring {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> Result<String, String> {
        let ticket = env.get_string(&ticket).map_err(|e| e.to_string())?.to_string_lossy().into_owned();
        let test_api = env.get_string(&test_api).map_err(|e| e.to_string())?.to_string_lossy().into_owned();
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().map_err(|e| e.to_string())?;
        let tokens = if test_api.is_empty() {
            rt.block_on(Raring::sign_in_with_ticket(&ticket)).map_err(|e| e.to_string())?
        } else {
            #[cfg(not(debug_assertions))]
            return Err("Test origins are disabled".into());
            #[cfg(debug_assertions)]
            {
                let url = tauri::Url::parse(&test_api).map_err(|_| "Invalid test origin")?;
                if url.scheme() != "http" || url.host_str() != Some("127.0.0.1") || !url.username().is_empty() || url.password().is_some() {
                    return Err("Test origin must be loopback".into());
                }
                rt.block_on(Raring::sign_in_with_ticket_at(&ticket, &test_api)).map_err(|e| e.to_string())?
            }
        };
        serde_json::to_string(&tokens).map_err(|e| e.to_string())
    }));
    let value = match result {
        Ok(Ok(tokens)) => serde_json::json!({"tokensJson":tokens}),
        Ok(Err(error)) => serde_json::json!({"error":error}),
        Err(_) => serde_json::json!({"error":"Ticket exchange failed"}),
    };
    env.new_string(value.to_string()).map(|s| s.into_raw()).unwrap_or(std::ptr::null_mut())
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_app_raring_NativeBridge_prepareTokens(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    tokens_json: JString<'_>,
) -> jni::sys::jstring {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(
        || -> Result<String, String> {
            let json = env
                .get_string(&tokens_json)
                .map_err(|e| e.to_string())?
                .to_string_lossy()
                .into_owned();
            let tokens: Tokens = match serde_json::from_str(&json) {
                Ok(t) => t,
                Err(_) => return Ok("{\"authFailed\":true}".into()),
            };
            if !tokens.is_expiring(Utc::now(), Duration::minutes(5)) {
                return Ok(serde_json::json!({"tokensJson":json}).to_string());
            }
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| e.to_string())?;
            let refreshed = rt.block_on(Raring::refresh_tokens(&tokens));
            match refreshed {
                Ok(value) => {
                    let json = serde_json::to_string(&value).map_err(|e| e.to_string())?;
                    Ok(serde_json::json!({"tokensJson": json}).to_string())
                }
                Err(raring_core::Error::Garmin(raring_core::garmin::Error::NeedsSignIn)) => {
                    Ok("{\"authFailed\":true}".into())
                }
                Err(e) => Ok(serde_json::json!({"error": e.to_string()}).to_string()),
            }
        },
    ));
    let json = result
        .ok()
        .and_then(Result::ok)
        .unwrap_or_else(|| "{\"error\":\"native refresh failed\"}".into());
    match env.new_string(json) {
        Ok(v) => v.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_app_raring_NativeBridge_sync(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    db_path: JString<'_>,
    tokens_json: JString<'_>,
    now_iso: JString<'_>,
) -> jni::sys::jstring {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(
        || -> jni::errors::Result<String> {
            let db = env.get_string(&db_path)?.to_string_lossy().into_owned();
            let tokens = env.get_string(&tokens_json)?.to_string_lossy().into_owned();
            let now = env.get_string(&now_iso)?.to_string_lossy().into_owned();
            Ok(serde_json::to_string(&run_background_sync(&db, &tokens, &now)).unwrap_or_default())
        },
    ));
    let json = result
        .ok()
        .and_then(Result::ok)
        .unwrap_or_else(|| "{\"error\":\"native sync failed\"}".into());
    match env.new_string(json) {
        Ok(value) => value.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

// Re-evaluate the committed, private snapshot without starting the UI or a network client.
#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_app_raring_NativeBridge_refreshStatus(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    db_path: JString<'_>,
    now_iso: JString<'_>,
) -> jni::sys::jstring {
    let result =
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> Result<(), String> {
            let db = env
                .get_string(&db_path)
                .map_err(|e| e.to_string())?
                .to_string_lossy()
                .into_owned();
            let now = env
                .get_string(&now_iso)
                .map_err(|e| e.to_string())?
                .to_string_lossy()
                .into_owned();
            let now = DateTime::parse_from_rfc3339(&now)
                .map_err(|e| e.to_string())?
                .with_timezone(&Local);
            Raring::open(db)
                .map_err(|e| e.to_string())?
                .refresh_status_snapshot(now)
                .map_err(|e| e.to_string())
        }));
    let json = match result {
        Ok(Ok(())) => "{}".into(),
        Ok(Err(e)) => serde_json::json!({"error": e}).to_string(),
        Err(_) => "{\"error\":\"native status refresh failed\"}".into(),
    };
    match env.new_string(json) {
        Ok(v) => v.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_app_raring_NativeBridge_heartRateDay(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    db_path: JString<'_>,
    tokens_json: JString<'_>,
    date: JString<'_>,
) -> jni::sys::jstring {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(
        || -> Result<String, String> {
            let db = env
                .get_string(&db_path)
                .map_err(|e| e.to_string())?
                .to_string_lossy()
                .into_owned();
            let tokens = env
                .get_string(&tokens_json)
                .map_err(|e| e.to_string())?
                .to_string_lossy()
                .into_owned();
            let date = env
                .get_string(&date)
                .map_err(|e| e.to_string())?
                .to_string_lossy()
                .into_owned();
            let core = Raring::open(db).map_err(|e| e.to_string())?;
            let identity = core.session_identity().ok_or("Account verification required")?;
            let cached = core.with_verified_session(&identity, || core.cached_heart_rate_day(&date))
                .ok_or("Account verification required")?;
            if let Some(day) = cached {
                return Ok(serde_json::json!({"day":day}).to_string());
            }
            let tokens: Tokens = serde_json::from_str(&tokens).map_err(|e| e.to_string())?;
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| e.to_string())?;
            let (day, tokens, report) = rt
                .block_on(core.fetch_heart_rate_day(tokens, &date))
                .map_err(|e| e.to_string())?;
            Ok(serde_json::json!({"day":day,"tokensJson":serde_json::to_string(&tokens).map_err(|e| e.to_string())?,"authFailed":report.auth_failed,"error":report.error}).to_string())
        },
    ));
    let json = result
        .ok()
        .and_then(Result::ok)
        .unwrap_or_else(|| "{\"error\":\"heart rate read failed\"}".into());
    match env.new_string(json) {
        Ok(v) => v.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

/// An explicitly opened past day or older workout, under Android's SyncGate.
fn detail_on_demand(
    db: &str,
    tokens_json: &str,
    date: &str,
    activity_id: &str,
    scope: &str,
) -> serde_json::Value {
    let core = match Raring::open(db) {
        Ok(core) => core,
        Err(e) => return serde_json::json!({"error": e.to_string()}),
    };
    if core.session_identity().is_none() {
        return serde_json::json!({"authFailed": true});
    }
    let tokens: Tokens = match serde_json::from_str(tokens_json) {
        Ok(tokens) => tokens,
        Err(_) => return serde_json::json!({"authFailed": true}),
    };
    let rt = match tokio::runtime::Builder::new_current_thread().enable_all().build() {
        Ok(rt) => rt,
        Err(e) => return serde_json::json!({"error": e.to_string()}),
    };
    let result = if !activity_id.is_empty() {
        rt.block_on(core.fetch_workout_detail(tokens, activity_id))
    } else {
        let parts: Vec<raring_core::view::DayDetailPart> = match scope {
            "sleep" => vec![raring_core::view::DayDetailPart::Sleep],
            "day" => vec![
                raring_core::view::DayDetailPart::Stress,
                raring_core::view::DayDetailPart::Vitals,
            ],
            _ => return serde_json::json!({"error": "unknown day detail scope"}),
        };
        rt.block_on(core.fetch_day_detail(tokens, date, &parts))
    };
    match result {
        Ok((tokens, report)) => serde_json::json!({
            "tokensJson": serde_json::to_string(&tokens).unwrap_or_default(),
            "authFailed": report.auth_failed,
            "error": report.error,
        }),
        Err(raring_core::Error::Garmin(raring_core::garmin::Error::NeedsSignIn)) => {
            serde_json::json!({"authFailed": true})
        }
        Err(e) => serde_json::json!({"error": e.to_string()}),
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_app_raring_NativeBridge_fetchDetail(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    db_path: JString<'_>,
    tokens_json: JString<'_>,
    date: JString<'_>,
    activity_id: JString<'_>,
    scope: JString<'_>,
) -> jni::sys::jstring {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(
        || -> Result<String, String> {
            let db = env.get_string(&db_path).map_err(|e| e.to_string())?.to_string_lossy().into_owned();
            let tokens = env.get_string(&tokens_json).map_err(|e| e.to_string())?.to_string_lossy().into_owned();
            let date = env.get_string(&date).map_err(|e| e.to_string())?.to_string_lossy().into_owned();
            let id = env.get_string(&activity_id).map_err(|e| e.to_string())?.to_string_lossy().into_owned();
            let scope = env.get_string(&scope).map_err(|e| e.to_string())?.to_string_lossy().into_owned();
            Ok(detail_on_demand(&db, &tokens, &date, &id, &scope).to_string())
        },
    ));
    let json = result
        .ok()
        .and_then(Result::ok)
        .unwrap_or_else(|| "{\"error\":\"detail read failed\"}".into());
    match env.new_string(json) {
        Ok(v) => v.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_app_raring_NativeBridge_refreshStrength(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    db_path: JString<'_>,
    tokens_json: JString<'_>,
    activity_id: JString<'_>,
) -> jni::sys::jstring {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(
        || -> Result<String, String> {
            let db = env
                .get_string(&db_path)
                .map_err(|e| e.to_string())?
                .to_string_lossy()
                .into_owned();
            let tokens = env
                .get_string(&tokens_json)
                .map_err(|e| e.to_string())?
                .to_string_lossy()
                .into_owned();
            let id = env
                .get_string(&activity_id)
                .map_err(|e| e.to_string())?
                .to_string_lossy()
                .into_owned();
            let core = Raring::open(db).map_err(|e| e.to_string())?;
            core.session_identity().ok_or("Account verification required")?;
            let tokens: Tokens = serde_json::from_str(&tokens).map_err(|e| e.to_string())?;
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| e.to_string())?;
            let (session, tokens, report) = rt
                .block_on(core.refresh_strength_sets(tokens, &id))
                .map_err(|e| e.to_string())?;
            Ok(serde_json::json!({"session":session,"tokensJson":serde_json::to_string(&tokens).map_err(|e| e.to_string())?,"authFailed":report.auth_failed,"error":report.error}).to_string())
        },
    ));
    let json = result
        .ok()
        .and_then(Result::ok)
        .unwrap_or_else(|| "{\"error\":\"strength refresh failed\"}".into());
    match env.new_string(json) {
        Ok(v) => v.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BedtimeResult {
    due: Option<Notification>,
    /// Local wall-clock time for the next reminder; Kotlin maps it to an instant.
    next_at: Option<String>,
    error: Option<String>,
}

fn bedtime(db_path: &str, now_local: &str) -> BedtimeResult {
    let mut result = BedtimeResult {
        due: None,
        next_at: None,
        error: None,
    };
    let now = match NaiveDateTime::parse_from_str(now_local, "%Y-%m-%dT%H:%M:%S%.f") {
        Ok(t) => t,
        Err(_) => {
            result.error = Some("invalid local time".into());
            return result;
        }
    };
    let reminder = match Raring::open(db_path) {
        Ok(core) if core.session_identity().is_some() => core.bedtime_reminder(now),
        Ok(_) => None,
        Err(e) => {
            result.error = Some(e.to_string());
            return result;
        }
    };
    if let Some(r) = reminder {
        if r.at <= now {
            result.due = Some(Notification {
                id: format!("bedtime:{}", r.night),
                title: r.title,
                body: r.body,
                target: notification_target("bedtime"),
                kind: "bedtime".into(),
                tag: r.night,
            });
        } else {
            result.next_at = Some(r.at.format("%Y-%m-%dT%H:%M:%S").to_string());
        }
    }
    result
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_app_raring_NativeBridge_bedtime(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    db_path: JString<'_>,
    now_local: JString<'_>,
) -> jni::sys::jstring {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(
        || -> jni::errors::Result<String> {
            let db = env.get_string(&db_path)?.to_string_lossy().into_owned();
            let now = env.get_string(&now_local)?.to_string_lossy().into_owned();
            Ok(serde_json::to_string(&bedtime(&db, &now)).unwrap_or_default())
        },
    ));
    let json = result
        .ok()
        .and_then(Result::ok)
        .unwrap_or_else(|| "{\"error\":\"native bedtime failed\"}".into());
    match env.new_string(json) {
        Ok(value) => value.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_app_raring_NativeBridge_markSent(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    db_path: JString<'_>,
    tags_json: JString<'_>,
    now_iso: JString<'_>,
) -> jni::sys::jstring {
    let result =
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> Result<(), String> {
            let db = env
                .get_string(&db_path)
                .map_err(|e| e.to_string())?
                .to_string_lossy()
                .into_owned();
            let tags = env
                .get_string(&tags_json)
                .map_err(|e| e.to_string())?
                .to_string_lossy()
                .into_owned();
            let now = env
                .get_string(&now_iso)
                .map_err(|e| e.to_string())?
                .to_string_lossy()
                .into_owned();
            let items: Vec<raring_core::notifications::Notification> =
                serde_json::from_str(&tags).map_err(|e| e.to_string())?;
            let now = DateTime::parse_from_rfc3339(&now)
                .map_err(|e| e.to_string())?
                .with_timezone(&Local);
            Raring::open(db)
                .map_err(|e| e.to_string())?
                .mark_notifications_sent(&items, now)
                .map_err(|e| e.to_string())
        }));
    let json = match result {
        Ok(Ok(())) => "{}".into(),
        Ok(Err(e)) => serde_json::json!({"error": e}).to_string(),
        Err(_) => "{\"error\":\"native mark failed\"}".into(),
    };
    match env.new_string(json) {
        Ok(v) => v.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

// All mutating operations are invoked under Android's existing SyncGate.
fn account_session(db: &str, action: &str, tokens: &str, test_api: &str) -> Result<serde_json::Value, String> {
    let core = Raring::open(db).map_err(|e| e.to_string())?;
    let error = |e: raring_core::Error| e.to_string();
    match action {
        "suspend" => { core.suspend_session().map_err(error)?; Ok(serde_json::json!({"active":false})) }
        "state" => {
            #[cfg(debug_assertions)]
            let demo = core.is_demo_session();
            #[cfg(not(debug_assertions))]
            let demo = false;
            Ok(serde_json::json!({"active":core.session_identity().is_some(),"migrationAllowed":core.legacy_session_needs_verification(),"identity":core.session_identity(),"demo":demo}))
        }
        "snapshot" => Ok(serde_json::json!({"snapshot":core.surface_snapshot().map_err(error)?})),
        "activate" => { core.activate_session().map_err(error)?; Ok(serde_json::json!({"active":true})) }
        "forget" => { core.forget_account().map_err(error)?; Ok(serde_json::json!({})) }
        #[cfg(debug_assertions)]
        "testSettings" => {
            let settings = serde_json::from_str(tokens).map_err(|_| "Invalid settings")?;
            core.set_settings(settings).map_err(error)?;
            Ok(serde_json::json!({}))
        }
        #[cfg(debug_assertions)]
        "testSettingsPatch" => {
            let patch = serde_json::from_str(tokens).map_err(|_| "Invalid settings patch")?;
            core.patch_settings_for_account(patch, core.bound_account_identity()).map_err(error)?;
            Ok(serde_json::json!({}))
        }
        "verify" => {
            let tokens: Tokens = serde_json::from_str(tokens).map_err(|_| "Invalid credentials")?;
            let client = if test_api.is_empty() {
                raring_core::garmin::Client::new(tokens).map_err(|e| e.to_string())?
            } else {
                // Only debug builds permit an emulator-local synthetic profile server.
                #[cfg(not(debug_assertions))]
                return Err("Test origins are disabled".into());
                #[cfg(debug_assertions)]
                {
                    let url = tauri::Url::parse(test_api).map_err(|_| "Invalid test origin")?;
                    if url.scheme() != "http" || url.host_str() != Some("127.0.0.1") || !url.username().is_empty() || url.password().is_some() {
                        return Err("Test origin must be loopback".into());
                    }
                    raring_core::garmin::Client::with_urls(tokens, test_api, test_api, std::time::Duration::ZERO).map_err(|e| e.to_string())?
                }
            };
            let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().map_err(|e| e.to_string())?;
            rt.block_on(core.verify_account(&client, Utc::now())).map_err(error)?;
            let tokens = rt.block_on(client.tokens());
            Ok(serde_json::json!({"tokensJson":serde_json::to_string(&tokens).map_err(|e| e.to_string())?}))
        }
        _ => Err("Unknown account operation".into()),
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_app_raring_NativeBridge_accountSession(
    mut env: JNIEnv<'_>, _class: JClass<'_>, db: JString<'_>, action: JString<'_>,
    tokens: JString<'_>, test_api: JString<'_>,
) -> jni::sys::jstring {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> Result<serde_json::Value, String> {
        let db = env.get_string(&db).map_err(|e| e.to_string())?.to_string_lossy().into_owned();
        let action = env.get_string(&action).map_err(|e| e.to_string())?.to_string_lossy().into_owned();
        let tokens = env.get_string(&tokens).map_err(|e| e.to_string())?.to_string_lossy().into_owned();
        let test_api = env.get_string(&test_api).map_err(|e| e.to_string())?.to_string_lossy().into_owned();
        account_session(&db, &action, &tokens, &test_api)
    }));
    let value = match result {
        Ok(Ok(value)) => value,
        Ok(Err(error)) => serde_json::json!({"error":error}),
        Err(_) => serde_json::json!({"error":"Account operation failed"}),
    };
    env.new_string(value.to_string()).map(|s| s.into_raw()).unwrap_or(std::ptr::null_mut())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invalid_tokens_need_sign_in_without_network() {
        let r = run_background_sync(":memory:", "{}", "2025-01-02T09:00:00+01:00");
        assert!(r.auth_failed);
        assert!(r.notifications.is_empty());
        assert_eq!(notification_target("weekly"), "checkin");
    }
    #[test]
    fn detail_without_a_session_asks_to_sign_in() {
        // No session means no token refresh and no Garmin request.
        let value = detail_on_demand(":memory:", "{}", "2025-01-01", "", "sleep");
        assert_eq!(value["authFailed"], true);
        assert!(value.get("tokensJson").is_none());
    }
    #[test]
    fn bedtime_without_a_plan_schedules_nothing() {
        let r = bedtime(":memory:", "2025-01-02T21:00:00.123");
        assert!(r.due.is_none() && r.next_at.is_none() && r.error.is_none());
        assert!(bedtime(":memory:", "not a time").error.is_some());
    }
}
