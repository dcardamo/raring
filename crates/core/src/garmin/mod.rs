//! Native Garmin API. No request is made until a caller explicitly signs in or syncs.
pub mod sso;

use base64::{
    engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD},
    Engine,
};
use chrono::{DateTime, Duration, NaiveDate, Utc};
use reqwest::{header, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{fmt, sync::Arc, time::Instant};
use tokio::sync::Mutex;

/// Activities per history page; Garmin accepts up to 1000.
pub const ACTIVITY_PAGE: &str = "100";
const DI_GRANT: &str = "https://connectapi.garmin.com/di-oauth2-service/oauth/grant/service_ticket";
const IDS: &[&str] = &[
    "GARMIN_CONNECT_MOBILE_ANDROID_DI_2025Q2",
    "GARMIN_CONNECT_MOBILE_ANDROID_DI_2024Q4",
    "GARMIN_CONNECT_MOBILE_ANDROID_DI",
    "GARMIN_CONNECT_MOBILE_IOS_DI",
];
pub const IOS_SERVICE_URL: &str = "https://mobile.integration.garmin.com/gcm/ios";

#[derive(Clone, Serialize, Deserialize)]
pub struct Tokens {
    pub access_token: String,
    pub refresh_token: String,
    pub client_id: String,
    pub expires_at: DateTime<Utc>,
}
impl fmt::Debug for Tokens {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Tokens")
            .field("access_token", &"[redacted]")
            .field("refresh_token", &"[redacted]")
            .field("client_id", &self.client_id)
            .field("expires_at", &self.expires_at)
            .finish()
    }
}
impl Tokens {
    pub fn is_expiring(&self, now: DateTime<Utc>, margin: Duration) -> bool {
        self.expires_at <= now + margin
    }
    pub fn import_python(data: &str) -> Result<Self> {
        let v: Value = serde_json::from_str(data).map_err(|_| Error::Decode)?;
        let access = v["di_token"].as_str().ok_or(Error::NeedsSignIn)?;
        let refresh = v["di_refresh_token"].as_str().ok_or(Error::NeedsSignIn)?;
        let id = v["di_client_id"]
            .as_str()
            .map(str::to_owned)
            .or_else(|| jwt(access).and_then(|v| v["client_id"].as_str().map(str::to_owned)))
            .filter(|s| !s.is_empty())
            .ok_or(Error::NeedsSignIn)?;
        Ok(Self {
            access_token: access.into(),
            refresh_token: refresh.into(),
            client_id: id,
            expires_at: expiry(access, None),
        })
    }
}
fn jwt(token: &str) -> Option<Value> {
    let part = token.split('.').nth(1)?;
    serde_json::from_slice(&URL_SAFE_NO_PAD.decode(part).ok()?).ok()
}
fn expiry(token: &str, seconds: Option<i64>) -> DateTime<Utc> {
    jwt(token)
        .and_then(|v| v["exp"].as_i64())
        .and_then(|t| DateTime::from_timestamp(t, 0))
        .unwrap_or_else(|| Utc::now() + Duration::seconds(seconds.unwrap_or(0)))
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("sign in again")]
    NeedsSignIn,
    #[error("rate limited")]
    RateLimited { retry_after: Option<u64> },
    #[error("not found")]
    NotFound,
    #[error("HTTP {0}")]
    Http(u16),
    #[error("invalid response")]
    Decode,
    #[error("network error")]
    Network,
}
pub type Result<T> = std::result::Result<T, Error>;

fn native(req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
    req.header(header::USER_AGENT, "GCM-Android-5.23")
        .header("X-Garmin-User-Agent", "com.garmin.android.apps.connectmobile/5.23; ; Google/sdk_gphone64_arm64/google; Android/33; Dalvik/2.1.0")
        .header("X-Garmin-Paired-App-Version", "10861").header("X-Garmin-Client-Platform", "Android")
        .header("X-App-Ver", "10861").header("X-Lang", "en").header("X-GCExperience", "GC5")
}
fn http_error(r: &reqwest::Response) -> Error {
    match r.status() {
        StatusCode::UNAUTHORIZED => Error::NeedsSignIn,
        StatusCode::NOT_FOUND => Error::NotFound,
        StatusCode::TOO_MANY_REQUESTS => Error::RateLimited {
            retry_after: r
                .headers()
                .get(header::RETRY_AFTER)
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.parse().ok()),
        },
        other => Error::Http(other.as_u16()),
    }
}
fn map_network(_: reqwest::Error) -> Error {
    Error::Network
} // Never expose credential-bearing URLs.
fn token_from(v: Value, previous: Option<&Tokens>, id: &str) -> Result<Tokens> {
    let access = v["access_token"].as_str().ok_or(Error::Decode)?;
    Ok(Tokens {
        access_token: access.into(),
        refresh_token: v["refresh_token"]
            .as_str()
            .or_else(|| previous.map(|t| t.refresh_token.as_str()))
            .unwrap_or("")
            .into(),
        client_id: jwt(access)
            .and_then(|p| p["client_id"].as_str().map(str::to_owned))
            .unwrap_or_else(|| id.into()),
        expires_at: expiry(access, v["expires_in"].as_i64()),
    })
}
async fn post_token(
    http: &reqwest::Client,
    url: &str,
    id: &str,
    fields: &[(&str, &str)],
) -> Result<Value> {
    let r = native(http.post(url))
        .header(
            header::AUTHORIZATION,
            format!("Basic {}", STANDARD.encode(format!("{id}:"))),
        )
        .header(header::ACCEPT, "application/json")
        .header(header::CACHE_CONTROL, "no-cache")
        .form(fields)
        .send()
        .await
        .map_err(map_network)?;
    if !r.status().is_success() {
        return Err(http_error(&r));
    }
    r.json().await.map_err(|_| Error::Decode)
}
pub async fn exchange_ticket(
    http: &reqwest::Client,
    ticket: &str,
    service_url: &str,
) -> Result<Tokens> {
    exchange_ticket_at(
        http,
        "https://diauth.garmin.com/di-oauth2-service/oauth/token",
        ticket,
        service_url,
    )
    .await
}
pub async fn exchange_ticket_at(
    http: &reqwest::Client,
    url: &str,
    ticket: &str,
    service_url: &str,
) -> Result<Tokens> {
    for id in IDS {
        match post_token(
            http,
            url,
            id,
            &[
                ("client_id", id),
                ("service_ticket", ticket),
                ("grant_type", DI_GRANT),
                ("service_url", service_url),
            ],
        )
        .await
        {
            Ok(v) => {
                if let Ok(t) = token_from(v, None, id) {
                    return Ok(t);
                }
            }
            Err(Error::RateLimited { retry_after }) => {
                return Err(Error::RateLimited { retry_after })
            }
            Err(Error::Network) => return Err(Error::Network),
            _ => (),
        }
    }
    Err(Error::NeedsSignIn)
}
pub async fn refresh(http: &reqwest::Client, old: &Tokens) -> Result<Tokens> {
    refresh_at(
        http,
        "https://diauth.garmin.com/di-oauth2-service/oauth/token",
        old,
    )
    .await
}
pub async fn refresh_at(http: &reqwest::Client, url: &str, old: &Tokens) -> Result<Tokens> {
    let v = post_token(
        http,
        url,
        &old.client_id,
        &[
            ("grant_type", "refresh_token"),
            ("client_id", &old.client_id),
            ("refresh_token", &old.refresh_token),
        ],
    )
    .await
    .map_err(|e| match e {
        Error::Http(400) | Error::NeedsSignIn => Error::NeedsSignIn,
        other => other,
    })?;
    token_from(v, Some(old), &old.client_id)
}

/// Serializes requests and refreshes, including across clones of this client.
#[derive(Clone)]
pub struct Client {
    http: reqwest::Client,
    api: String,
    auth: String,
    state: Arc<Mutex<(Tokens, Option<Instant>)>>,
    interval: std::time::Duration,
}
impl Client {
    pub fn new(tokens: Tokens) -> Result<Self> {
        Self::with_urls(
            tokens,
            "https://connectapi.garmin.com",
            "https://diauth.garmin.com/di-oauth2-service/oauth/token",
            std::time::Duration::from_millis(300),
        )
    }
    pub fn with_urls(
        tokens: Tokens,
        api: &str,
        auth: &str,
        interval: std::time::Duration,
    ) -> Result<Self> {
        Ok(Self {
            http: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(20))
                .build()
                .map_err(map_network)?,
            api: api.trim_end_matches('/').into(),
            auth: auth.into(),
            state: Arc::new(Mutex::new((tokens, None))),
            interval,
        })
    }
    pub async fn tokens(&self) -> Tokens {
        self.state.lock().await.0.clone()
    }
    pub async fn get(&self, path: &str, params: &[(&str, &str)]) -> Result<Value> {
        if !path.starts_with('/')
            || path.contains("..")
            || path.contains('?')
            || path.contains('#')
            || path.contains('\\')
        {
            return Err(Error::Decode);
        }
        let mut guard = self.state.lock().await;
        if guard.0.is_expiring(Utc::now(), Duration::minutes(5)) {
            guard.0 = refresh_at(&self.http, &self.auth, &guard.0).await?;
        }
        for attempt in 0..2 {
            if let Some(last) = guard.1 {
                if let Some(wait) = self.interval.checked_sub(last.elapsed()) {
                    tokio::time::sleep(wait).await;
                }
            }
            guard.1 = Some(Instant::now());
            let r = native(self.http.get(format!("{}{path}", self.api)))
                .bearer_auth(&guard.0.access_token)
                .query(params)
                .send()
                .await
                .map_err(map_network)?;
            if r.status() == StatusCode::UNAUTHORIZED && attempt == 0 {
                guard.0 = refresh_at(&self.http, &self.auth, &guard.0).await?;
                continue;
            }
            if !r.status().is_success() {
                return Err(http_error(&r));
            }
            if r.status() == StatusCode::NO_CONTENT {
                return Err(Error::NotFound);
            }
            return r.json().await.map_err(|_| Error::Decode);
        }
        Err(Error::NeedsSignIn)
    }
    /// Execute a read-only GraphQL query through Garmin's gateway. Errors in a 200
    /// response are failures, not a missing metric to cache as an empty day.
    pub async fn graphql(&self, query: &str, variables: Value) -> Result<Value> {
        let mut guard = self.state.lock().await;
        if guard.0.is_expiring(Utc::now(), Duration::minutes(5)) {
            guard.0 = refresh_at(&self.http, &self.auth, &guard.0).await?;
        }
        for attempt in 0..2 {
            if let Some(last) = guard.1 {
                if let Some(wait) = self.interval.checked_sub(last.elapsed()) {
                    tokio::time::sleep(wait).await;
                }
            }
            guard.1 = Some(Instant::now());
            let r = native(
                self.http
                    .post(format!("{}/graphql-gateway/graphql", self.api)),
            )
            .bearer_auth(&guard.0.access_token)
            .json(&serde_json::json!({"query": query, "variables": variables}))
            .send()
            .await
            .map_err(map_network)?;
            if r.status() == StatusCode::UNAUTHORIZED && attempt == 0 {
                guard.0 = refresh_at(&self.http, &self.auth, &guard.0).await?;
                continue;
            }
            if !r.status().is_success() {
                return Err(http_error(&r));
            }
            let value: Value = r.json().await.map_err(|_| Error::Decode)?;
            if value.get("errors").is_some() || !value["data"].is_object() {
                return Err(Error::Decode);
            }
            return Ok(value["data"].clone());
        }
        Err(Error::NeedsSignIn)
    }
    pub async fn endpoint(&self, kind: &str, key: &str, display: Option<&str>) -> Result<Value> {
        let d = display.unwrap_or("");
        // display names are URL path segments; never allow them to alter the route.
        let d = url::form_urlencoded::byte_serialize(d.as_bytes())
            .collect::<String>()
            .replace('+', "%20");
        let (path, params): (String, Vec<(&str, &str)>) = match kind {
            "profile" => ("/userprofile-service/socialProfile".into(), vec![]),
            "summary" => (
                format!("/usersummary-service/usersummary/daily/{d}"),
                vec![("calendarDate", key)],
            ),
            "sleep" => (
                format!("/wellness-service/wellness/dailySleepData/{d}"),
                vec![("date", key), ("nonSleepBufferMinutes", "60")],
            ),
            "hrv" => (format!("/hrv-service/hrv/{key}"), vec![]),
            "readiness" => (
                format!("/metrics-service/metrics/trainingreadiness/{key}"),
                vec![],
            ),
            "training" => (
                format!("/metrics-service/metrics/trainingstatus/aggregated/{key}"),
                vec![],
            ),
            "training_daily" => (
                format!("/metrics-service/metrics/trainingstatus/daily/{key}"),
                vec![],
            ),
            "load_focus" => (
                format!("/metrics-service/metrics/trainingloadbalance/latest/{key}"),
                vec![],
            ),
            "battery" => (
                "/wellness-service/wellness/bodyBattery/reports/daily".into(),
                vec![("startDate", key), ("endDate", key)],
            ),
            "spo2" => (
                format!("/wellness-service/wellness/daily/spo2/{key}"),
                vec![],
            ),
            "respiration" => (
                format!("/wellness-service/wellness/daily/respiration/{key}"),
                vec![],
            ),
            "stress" => (
                format!("/wellness-service/wellness/dailyStress/{key}"),
                vec![],
            ),
            "max_metrics" => (
                format!("/metrics-service/metrics/maxmet/daily/{key}/{key}"),
                vec![],
            ),
            "fitness_age" => (format!("/fitnessage-service/fitnessage/{key}"), vec![]),
            "endurance" => (
                "/metrics-service/metrics/endurancescore".into(),
                vec![("calendarDate", key)],
            ),
            "hill" => (
                "/metrics-service/metrics/hillscore".into(),
                vec![("calendarDate", key)],
            ),
            "races" => (
                format!("/metrics-service/metrics/racepredictions/latest/{d}"),
                vec![],
            ),
            "tolerance" => (
                "/metrics-service/metrics/runningtolerance/stats".into(),
                vec![
                    ("startDate", key),
                    ("endDate", key),
                    ("aggregation", "daily"),
                ],
            ),
            "records" => (
                format!("/personalrecord-service/personalrecord/prs/{d}"),
                vec![],
            ),
            "activities" => (
                "/activitylist-service/activities/search/activities".into(),
                vec![("start", key), ("limit", "20")],
            ),
            // Range keys are "START:END" (inclusive dates); activity pages add ":OFFSET".
            "summary_range" | "sleep_range" | "hrv_range" | "readiness_range"
            | "max_metrics_range" | "activities_range" => {
                let mut parts = key.split(':');
                let (Some(start), Some(end)) = (parts.next(), parts.next()) else {
                    return Err(Error::Decode);
                };
                if [start, end]
                    .iter()
                    .any(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").is_err())
                {
                    return Err(Error::Decode);
                }
                let offset = parts.next().unwrap_or("0");
                if offset.is_empty()
                    || !offset.bytes().all(|b| b.is_ascii_digit())
                    || parts.next().is_some()
                {
                    return Err(Error::Decode);
                }
                let dates = vec![("startDate", start), ("endDate", end)];
                match kind {
                    "summary_range" => ("/usersummary-service/v2/dailysummaries".into(), dates),
                    "sleep_range" => (
                        "/sleep-service/sleep/dailySleepSummariesByDate".into(),
                        dates,
                    ),
                    "hrv_range" => (format!("/hrv-service/hrv/daily/{start}/{end}"), vec![]),
                    "readiness_range" => (
                        format!("/metrics-service/metrics/trainingreadiness/{start}/{end}"),
                        vec![],
                    ),
                    "max_metrics_range" => (
                        format!("/metrics-service/metrics/maxmet/daily/{start}/{end}"),
                        vec![],
                    ),
                    _ => (
                        "/activitylist-service/activities/search/activities".into(),
                        vec![
                            ("startDate", start),
                            ("endDate", end),
                            ("start", offset),
                            ("limit", ACTIVITY_PAGE),
                        ],
                    ),
                }
            }
            "activity" | "splits" | "zones" | "details" => {
                if !key.bytes().all(|b| b.is_ascii_digit()) || key.is_empty() {
                    return Err(Error::Decode);
                }
                let suffix = match kind {
                    "splits" => "/splits",
                    "zones" => "/hrTimeInZones",
                    "details" => "/details",
                    _ => "",
                };
                (
                    format!("/activity-service/activity/{key}{suffix}"),
                    if kind == "details" {
                        vec![("maxChartSize", "2000"), ("maxPolylineSize", "4000")]
                    } else {
                        vec![]
                    },
                )
            }
            _ => return Err(Error::Decode),
        };
        self.get(&path, &params).await
    }
}

/// CLI-only sign-in; the app uses the SSO webview instead.
pub async fn mobile_login<F>(email: &str, password: &str, mut mfa: F) -> Result<Tokens>
where
    F: FnMut() -> String,
{
    let http = reqwest::Client::builder()
        .cookie_store(true)
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(map_network)?;
    let params = [
        ("clientId", "GCM_IOS_DARK"),
        ("locale", "en-US"),
        ("service", IOS_SERVICE_URL),
    ];
    let headers = |r: reqwest::RequestBuilder| {
        r.header(header::USER_AGENT, "Mozilla/5.0 (iPhone; CPU iPhone OS 18_7 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Mobile/15E148")
        .header(header::ACCEPT, "application/json, text/plain, */*").header(header::ORIGIN, "https://sso.garmin.com")
    };
    let r = headers(http.post("https://sso.garmin.com/mobile/api/login")).query(&params)
        .json(&serde_json::json!({"username":email,"password":password,"rememberMe":true,"captchaToken":""})).send().await.map_err(map_network)?;
    if !r.status().is_success() {
        return Err(http_error(&r));
    }
    let mut v: Value = r.json().await.map_err(|_| Error::Decode)?;
    if v["responseStatus"]["type"] == "MFA_REQUIRED" {
        let r = headers(http.post("https://sso.garmin.com/mobile/api/mfa/verifyCode")).query(&params)
            .json(&serde_json::json!({"mfaVerificationCode":mfa(),"rememberMyBrowser":true,"reconsentList":[],"mfaSetup":false})).send().await.map_err(map_network)?;
        if !r.status().is_success() {
            return Err(http_error(&r));
        }
        v = r.json().await.map_err(|_| Error::Decode)?;
    }
    if v["responseStatus"]["type"] != "SUCCESSFUL" {
        return Err(Error::NeedsSignIn);
    }
    exchange_ticket(
        &http,
        v["serviceTicketId"].as_str().ok_or(Error::Decode)?,
        IOS_SERVICE_URL,
    )
    .await
}
