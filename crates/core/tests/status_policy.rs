use chrono::Local;
use raring_core::{account, store::Store, Raring};
use serde_json::{json, Value};
use std::fs;

#[test]
fn a_suspended_session_hides_the_snapshot_but_keeps_the_account() {
    let dir = std::env::temp_dir().join(format!("raring-status-policy-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("raring.db");
    let store = Store::open(&path).unwrap();
    let profile = json!({"profileId":101,"displayName":"fake-A"});
    store.put("profile", "me", "synthetic", &profile).unwrap();
    // Test fixture for an already verified session; the HTTP verification has separate tests.
    store.set_setting("account_session", &json!({"identity":account::profile_identity(&profile),"active":true}).to_string()).unwrap();
    let core = Raring::open(&path).unwrap();
    core.refresh_status_snapshot(Local::now()).unwrap();
    assert!(core.surface_snapshot().unwrap().is_some());
    // A version-1 file from before the HRV swap is never shown with the old fields.
    let file = path.with_extension("status.json");
    let mut old: Value = serde_json::from_slice(&fs::read(&file).unwrap()).unwrap();
    old["version"] = 1.into();
    fs::write(&file, old.to_string()).unwrap();
    assert!(core.surface_snapshot().unwrap().is_none());
    core.refresh_status_snapshot(Local::now()).unwrap();
    assert!(core.surface_snapshot().unwrap().is_some());
    core.suspend_session().unwrap();
    assert!(core.surface_snapshot().unwrap().is_none());
    assert_eq!(store.get("profile", "me").unwrap(), profile);
    drop(core); drop(store);
    fs::remove_dir_all(dir).unwrap();
}
