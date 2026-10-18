use chrono::Local;
use raring_core::{account, store::Store, Raring};
use serde_json::{json, Value};
use std::fs;

#[test]
fn authoritative_opt_out_rejects_an_old_enabled_file_after_failed_publication() {
    let dir = std::env::temp_dir().join(format!("raring-status-policy-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("raring.db");
    let store = Store::open(&path).unwrap();
    let profile = json!({"profileId":101,"displayName":"fake-A"});
    store.put("profile", "me", "synthetic", &profile).unwrap();
    // Test fixture for an already verified session; the HTTP verification has separate tests.
    store.set_setting("account_session", &json!({"identity":account::profile_identity(&profile),"active":true}).to_string()).unwrap();
    let core = Raring::open(&path).unwrap();
    core.set_setting("status", true).unwrap();
    let file = path.with_extension("status.json");
    let old_enabled = fs::read(&file).unwrap();
    fs::remove_file(&file).unwrap();
    fs::create_dir(&file).unwrap();
    core.set_setting("status", false).unwrap();
    assert!(!core.setting("status"));
    fs::remove_dir(&file).unwrap();
    fs::write(&file, old_enabled).unwrap();
    assert_eq!(serde_json::from_slice::<Value>(&fs::read(&file).unwrap()).unwrap()["enabled"], true);
    assert_eq!(core.surface_snapshot().unwrap().unwrap()["enabled"], false);
    core.refresh_status_snapshot(Local::now()).unwrap();
    assert_eq!(core.surface_snapshot().unwrap().unwrap()["enabled"], false);
    core.suspend_session().unwrap();
    assert!(core.surface_snapshot().unwrap().is_none());
    assert_eq!(store.get("profile", "me").unwrap(), profile);
    drop(core); drop(store);
    fs::remove_dir_all(dir).unwrap();
}
