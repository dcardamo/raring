use chrono::{DateTime, Local};
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
    match runtime.block_on(core.sync(tokens, now, 60)) {
        Ok(outcome) => {
            result.tokens_json = serde_json::to_string(&outcome.tokens).unwrap_or_default();
            result.readiness_missing = !outcome.report.readiness_present;
            result.auth_failed = outcome.report.auth_failed;
            if !result.auth_failed {
                if let Err(e) = core.mark_notifications_sent(&outcome.notifications, now) {
                    result.error = Some(e.to_string());
                    return result;
                }
                result.notifications = outcome
                    .notifications
                    .into_iter()
                    .map(|item| Notification {
                        id: format!("{}:{}", item.kind, item.tag),
                        title: item.title,
                        body: item.body,
                        target: notification_target(&item.kind),
                    })
                    .collect();
            }
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
pub extern "system" fn Java_ca_hld_pepometer_NativeBridge_sync(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    db_path: JString<'_>,
    tokens_json: JString<'_>,
    now_iso: JString<'_>,
) -> jni::sys::jstring {
    let result = (|| -> jni::errors::Result<String> {
        let db = env.get_string(&db_path)?.to_string_lossy().into_owned();
        let tokens = env.get_string(&tokens_json)?.to_string_lossy().into_owned();
        let now = env.get_string(&now_iso)?.to_string_lossy().into_owned();
        Ok(serde_json::to_string(&run_background_sync(&db, &tokens, &now)).unwrap_or_default())
    })();
    match env.new_string(result.unwrap_or_default()) {
        Ok(value) => value.into_raw(),
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
