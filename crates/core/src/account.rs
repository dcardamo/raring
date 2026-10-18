//! Account binding is separate from credentials, which belong to the platform vault.
use crate::{garmin, store::Store, Result};
use chrono::{DateTime, Utc};
use serde_json::Value;

/// Garmin's user-profile identifier, not the separate social-profile `id` or a name.
/// Public Garmin clients use socialProfile.profileId as userProfileId.
pub(crate) fn valid_identity(identity: &str) -> bool {
    identity
        .parse::<u64>()
        .is_ok_and(|id| id > 0 && id.to_string() == identity)
}

pub fn profile_identity(profile: &Value) -> Option<String> {
    profile
        .get("profileId")?
        .as_u64()
        .filter(|id| *id > 0)
        .map(|id| id.to_string())
}

pub fn identity(store: &Store) -> Option<String> {
    store.account_identity()
}

pub fn active(store: &Store) -> bool {
    store.account_active()
}

pub fn suspend(store: &Store) -> Result<()> {
    let _boundary = crate::status::WRITE_LOCK.lock().unwrap();
    store.suspend_account()?;
    Ok(())
}

/// Always read a fresh authenticated profile before deciding whether history belongs to this user.
pub async fn verify(store: &Store, client: &garmin::Client, now: DateTime<Utc>) -> Result<()> {
    suspend(store)?;
    let profile = client.endpoint("profile", "me", None).await?;
    let id = profile_identity(&profile).ok_or(garmin::Error::Decode)?;
    if !profile["displayName"]
        .as_str()
        .is_some_and(|s| !s.trim().is_empty())
    {
        return Err(garmin::Error::Decode.into());
    }
    // Only local mutation is serialized; never hold this barrier over HTTP.
    let _boundary = crate::status::WRITE_LOCK.lock().unwrap();
    if !store.bind_account(&id, &profile, &now.to_rfc3339())? {
        return Err(garmin::Error::Decode.into());
    }
    Ok(())
}

pub fn activate(store: &Store) -> Result<()> {
    let _boundary = crate::status::WRITE_LOCK.lock().unwrap();
    if !store.activate_account()? {
        return Err(garmin::Error::Decode.into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::garmin::Tokens;
    use chrono::Duration;
    use serde_json::json;
    use std::time::Duration as StdDuration;
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpListener,
    };

    #[test]
    fn user_profile_id_is_not_the_social_id_or_display_name() {
        assert_eq!(profile_identity(&json!({"id":11,"displayName":"A"})), None);
        assert_eq!(profile_identity(&json!({"id":11,"profileId":22,"displayName":"A"})).as_deref(), Some("22"));
    }

    async fn client(status: u16, body: &str) -> garmin::Client {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let body = body.to_owned();
        tokio::spawn(async move {
            // A 401 can trigger a token refresh; answer its local auth request too.
            for _ in 0..2 {
                let Ok(Ok((mut socket, _))) =
                    tokio::time::timeout(StdDuration::from_millis(500), listener.accept()).await
                else {
                    break;
                };
                let mut buf = [0; 2048];
                let n = socket.read(&mut buf).await.unwrap();
                let request = String::from_utf8_lossy(&buf[..n]);
                let (code, reply) = if request.contains("/userprofile-service/socialProfile") {
                    (status, body.as_str())
                } else {
                    (400, "{}")
                };
                let response = format!("HTTP/1.1 {code} Test\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{reply}", reply.len());
                socket.write_all(response.as_bytes()).await.unwrap();
            }
        });
        garmin::Client::with_urls(
            Tokens {
                access_token: "synthetic".into(),
                refresh_token: "synthetic".into(),
                client_id: "synthetic".into(),
                expires_at: Utc::now() + Duration::hours(2),
            },
            &url,
            &format!("{url}/auth"),
            StdDuration::ZERO,
        )
        .unwrap()
    }

    fn seeded(store: &Store) {
        store
            .put("summary", "2026-01-01", "now", &json!({"account": "A"}))
            .unwrap();
        store
            .put(
                "listed",
                "42",
                "now",
                &json!({"activityId":42,"startTimeLocal":"2026-01-01 10:00:00"}),
            )
            .unwrap();
        store.set_setting("plan", "A targets").unwrap();
        store
            .insert_setting("sleep_plan:2026-01-01", "A sleep plan")
            .unwrap();
        store.set_setting("settings", "A opt-in").unwrap();
        store.set_cursor("last_sync", "A cursor").unwrap();
        store.mark_notified("bedtime", "A", "now").unwrap();
    }
    fn assert_seeded(store: &Store) {
        assert!(store.get("summary", "2026-01-01").is_some());
        assert_eq!(store.activity_ids(None, 10), ["42"]);
        assert_eq!(store.setting("plan").as_deref(), Some("A targets"));
        assert_eq!(
            store.setting("sleep_plan:2026-01-01").as_deref(),
            Some("A sleep plan")
        );
        assert_eq!(store.cursor("last_sync").as_deref(), Some("A cursor"));
        assert!(store.notified("bedtime", "A"));
    }

    #[tokio::test]
    async fn different_account_erases_everything_before_next_sync() {
        let store = Store::open(":memory:").unwrap();
        verify(
            &store,
            &client(200, r#"{"profileId":11,"displayName":"A"}"#).await,
            Utc::now(),
        )
        .await
        .unwrap();
        activate(&store).unwrap();
        seeded(&store);
        suspend(&store).unwrap();
        assert!(!active(&store));
        verify(
            &store,
            &client(200, r#"{"profileId":22,"displayName":"B"}"#).await,
            Utc::now(),
        )
        .await
        .unwrap();
        assert_eq!(identity(&store).as_deref(), Some("22"));
        assert!(!active(&store));
        assert!(store.get("summary", "2026-01-01").is_none());
        assert!(store.activity_ids(None, 10).is_empty());
        assert_eq!(store.setting("plan"), None);
        assert_eq!(store.setting("sleep_plan:2026-01-01"), None);
        assert_eq!(store.setting("settings"), None);
        assert_eq!(store.cursor("last_sync"), None);
        assert!(!store.notified("bedtime", "A"));
        // Even if a later data request fails, no A data can be read back.
        let report =
            crate::sync::sync(&store, &client(500, "{}").await, chrono::Local::now(), 1).await;
        assert!(report.error.is_some() || report.data_failures > 0);
        assert!(store.get("summary", "2026-01-01").is_none());
        activate(&store).unwrap();
        assert!(active(&store));
    }

    #[tokio::test]
    async fn same_identity_preserves_history() {
        let store = Store::open(":memory:").unwrap();
        verify(
            &store,
            &client(200, r#"{"profileId":11,"displayName":"A"}"#).await,
            Utc::now(),
        )
        .await
        .unwrap();
        seeded(&store);
        suspend(&store).unwrap();
        verify(
            &store,
            &client(200, r#"{"id":999,"profileId":11,"displayName":"Renamed"}"#).await,
            Utc::now(),
        )
        .await
        .unwrap();
        assert_seeded(&store);
        activate(&store).unwrap();
        assert!(active(&store));
    }

    #[tokio::test]
    async fn failed_verification_keeps_history_inactive() {
        for (status, body) in [
            (500, "{}"),
            (401, "{}"),
            (200, r#"{"displayName":"B"}"#),
            (200, r#"{"profileId":"22","displayName":"B"}"#),
            (200, r#"{"profileId":22,"displayName":" "}"#),
        ] {
            let store = Store::open(":memory:").unwrap();
            verify(
                &store,
                &client(200, r#"{"profileId":11,"displayName":"A"}"#).await,
                Utc::now(),
            )
            .await
            .unwrap();
            activate(&store).unwrap();
            seeded(&store);
            assert!(verify(&store, &client(status, body).await, Utc::now())
                .await
                .is_err());
            assert!(!active(&store));
            assert_eq!(
                serde_json::from_str::<Value>(&store.setting("account_session").unwrap()).unwrap(),
                json!({"identity":"11", "active":false})
            );
            assert_eq!(identity(&store).as_deref(), Some("11"));
            assert_seeded(&store);
            assert!(activate(&store).is_err()); // A cached profile is not fresh verification.
        }
    }

    #[tokio::test]
    async fn unidentifiable_history_fails_closed() {
        let store = Store::open(":memory:").unwrap();
        seeded(&store);
        assert!(verify(
            &store,
            &client(200, r#"{"profileId":22,"displayName":"B"}"#).await,
            Utc::now()
        )
        .await
        .is_err());
        assert_seeded(&store);
        assert_eq!(identity(&store), None);
        assert!(!active(&store));
        assert!(activate(&store).is_err());
    }

    #[tokio::test]
    async fn malformed_binding_does_not_authorize_erasure() {
        let store = Store::open(":memory:").unwrap();
        seeded(&store);
        store
            .set_setting(
                "account_session",
                r#"{"identity":"not-a-number","active":true}"#,
            )
            .unwrap();
        assert!(verify(
            &store,
            &client(200, r#"{"profileId":22,"displayName":"B"}"#).await,
            Utc::now()
        )
        .await
        .is_err());
        assert!(!active(&store));
        assert_seeded(&store);
    }

    #[tokio::test]
    async fn cached_profile_migrates_identity() {
        let store = Store::open(":memory:").unwrap();
        store
            .put("profile", "me", "now", &json!({"profileId":11,"displayName":"A"}))
            .unwrap();
        seeded(&store);
        verify(
            &store,
            &client(200, r#"{"profileId":11,"displayName":"A"}"#).await,
            Utc::now(),
        )
        .await
        .unwrap();
        assert_seeded(&store);
    }
}
