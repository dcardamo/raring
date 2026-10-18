use chrono::{Duration, Local, Utc};
use raring_core::{Raring, Tokens};
use std::fs;

#[tokio::test]
async fn partial_sync_reports_snapshot_failure_without_losing_tokens_or_first_error() {
    let dir = std::env::temp_dir().join(format!("raring-status-sync-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("raring.db");
    let core = Raring::open(&path).unwrap();
    fs::create_dir_all(path.with_extension("status.json")).unwrap();
    let tokens = Tokens {
        access_token: "synthetic-access".into(),
        refresh_token: "synthetic-refresh".into(),
        client_id: "synthetic-client".into(),
        expires_at: Utc::now() + Duration::days(2),
    };
    // A local snapshot failure is separate from the network report, so it
    // cannot cause native callers to repeat Garmin requests.
    let outcome = core.sync(tokens, Local::now(), 0).await.unwrap();
    assert_eq!(outcome.tokens.refresh_token, "synthetic-refresh");
    let error = outcome.report.error.unwrap();
    assert!(error.contains("Missing Garmin display name"), "{error}");
    assert!(!error.contains("status snapshot"), "{error}");
    assert!(outcome.snapshot_error.unwrap().contains("status snapshot"));
    assert_eq!(outcome.report.requests, 0);
    raring_core::store::Store::open(&path).unwrap().put(
        "profile", "me", "synthetic-fetch", &serde_json::json!({"displayName":"synthetic"})
    ).unwrap();
    let outcome = core.sync(outcome.tokens, Local::now(), 0).await.unwrap();
    assert!(outcome.report.error.is_none(), "snapshot errors cannot trigger network retries");
    assert!(outcome.snapshot_error.is_some());
    assert_eq!(outcome.report.requests, 0);
}
