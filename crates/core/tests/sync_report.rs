use chrono::Utc;
use raring_core::{SyncReport, Tokens};

#[test]
fn sync_report_never_serializes_credentials() {
    let report = SyncReport {
        requests: 1,
        tokens: Some(Tokens {
            access_token: "synthetic-access-secret".into(),
            refresh_token: "synthetic-refresh-secret".into(),
            client_id: "synthetic-client".into(),
            expires_at: Utc::now(),
        }),
        ..Default::default()
    };
    let value = serde_json::to_value(&report).unwrap();
    assert_eq!(value["requests"], 1);
    assert!(value.get("tokens").is_none());
    assert!(!value.to_string().contains("secret"));
    assert!(!format!("{report:?}").contains("secret"));
    assert!(serde_json::from_value::<SyncReport>(value)
        .unwrap()
        .tokens
        .is_none());

    // Callers still need to serialize the separate credentials for secure storage.
    let credentials = serde_json::to_value(report.tokens.as_ref().unwrap()).unwrap();
    assert_eq!(credentials["access_token"], "synthetic-access-secret");
    assert_eq!(credentials["refresh_token"], "synthetic-refresh-secret");
}
