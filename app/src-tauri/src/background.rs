use chrono::{DateTime, Duration, Local, Utc};
use jni::{
    objects::{JClass, JString},
    JNIEnv,
};
use pepometer_core::{Pepometer, Tokens};
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
    let core = match Pepometer::open(db_path) {
        Ok(c) => c,
        Err(e) => {
            result.error = Some(e.to_string());
            return result;
        }
    };
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
            result.tokens_json = serde_json::to_string(&outcome.tokens).unwrap_or_default();
            result.readiness_missing = !outcome.report.readiness_present;
            result.auth_failed = outcome.report.auth_failed;
            result.notifications = outcome
                .notifications
                .into_iter()
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
        Err(pepometer_core::Error::Garmin(pepometer_core::garmin::Error::NeedsSignIn)) => {
            result.auth_failed = true;
        }
        Err(e) => result.error = Some(e.to_string()),
    }
    result
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_ca_hld_pepometer_NativeBridge_prepareTokens(
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
            let refreshed = rt.block_on(Pepometer::refresh_tokens(&tokens));
            match refreshed {
                Ok(value) => {
                    let json = serde_json::to_string(&value).map_err(|e| e.to_string())?;
                    Ok(serde_json::json!({"tokensJson": json}).to_string())
                }
                Err(pepometer_core::Error::Garmin(pepometer_core::garmin::Error::NeedsSignIn)) => {
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
pub extern "system" fn Java_ca_hld_pepometer_NativeBridge_sync(
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

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_ca_hld_pepometer_NativeBridge_markSent(
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
            let items: Vec<pepometer_core::notifications::Notification> =
                serde_json::from_str(&tags).map_err(|e| e.to_string())?;
            let now = DateTime::parse_from_rfc3339(&now)
                .map_err(|e| e.to_string())?
                .with_timezone(&Local);
            Pepometer::open(db)
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
}
