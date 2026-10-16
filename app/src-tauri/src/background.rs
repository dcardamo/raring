use jni::{
    objects::{JClass, JString},
    JNIEnv,
};
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SyncResult {
    tokens_json: String,
    readiness_missing: bool,
    auth_failed: bool,
    notifications: Vec<Notification>,
}

#[derive(Serialize)]
struct Notification {
    id: String,
    title: String,
    body: String,
    target: String,
}

// Integration point: open Pepometer::open(db_path), parse tokens_json, call
// core.sync(tokens, now_iso), serialize refreshed tokens and notification decisions.
// JNI must remain independent of Tauri's runtime; db_path is filesDir/pepometer.db.
fn run_background_sync(_db_path: &str, tokens_json: &str, now_iso: &str) -> SyncResult {
    log::info!("background JNI sync ran");
    SyncResult {
        tokens_json: tokens_json.to_owned(),
        readiness_missing: false,
        auth_failed: false,
        notifications: vec![Notification {
            id: "background-sync".into(),
            title: "Background sync ran".into(),
            body: now_iso.into(),
            target: "today".into(),
        }],
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
