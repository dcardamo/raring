use crate::{
    garmin::{self, Client, Tokens},
    store::Store,
};
use chrono::{DateTime, Duration, Local, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SyncReport {
    pub requests: usize,
    pub stored: usize,
    pub activities: usize,
    pub sleep_present: bool,
    pub readiness_present: bool,
    pub rate_limited: bool,
    pub auth_failed: bool,
    pub error: Option<String>,
    pub tokens: Option<Tokens>,
}
const DAILY: &[&str] = &[
    "summary",
    "sleep",
    "hrv",
    "readiness",
    "training",
    "training_daily",
    "load_focus",
    "battery",
    "spo2",
    "respiration",
    "stress",
    "max_metrics",
];
const SLOW: &[&str] = &[
    "fitness_age",
    "races",
    "endurance",
    "hill",
    "tolerance",
    "records",
];
fn date(d: NaiveDate) -> String {
    d.format("%Y-%m-%d").to_string()
}
/// Every network request consumes budget; failures do not erase previous raw responses.
pub async fn sync(
    store: &Store,
    client: &Client,
    now_local: DateTime<Local>,
    budget: usize,
) -> SyncReport {
    let mut report = SyncReport::default();
    let today = now_local.date_naive();
    let now = Utc::now();
    if store
        .cursor("backoff_until")
        .and_then(|s| s.parse::<i64>().ok())
        .is_some_and(|s| s > now.timestamp())
    {
        report.rate_limited = true;
        report.tokens = Some(client.tokens().await);
        return report;
    }
    let profile = store
        .get("profile", "me")
        .unwrap_or(serde_json::Value::Null);
    let profile = if profile.is_null() && budget > 0 {
        report.requests += 1;
        match client.endpoint("profile", "me", None).await {
            Ok(v) => {
                if store.put("profile", "me", &now.to_rfc3339(), &v).is_err() {
                    report.error = Some("local storage failed".into());
                    report.tokens = Some(client.tokens().await);
                    return report;
                }
                report.stored += 1;
                v
            }
            Err(e) => {
                failure(store, &mut report, e, now);
                report.tokens = Some(client.tokens().await);
                return report;
            }
        }
    } else {
        profile
    };
    let name = profile["displayName"].as_str();
    if name.is_none() {
        report.error = Some("Missing Garmin display name".into());
        report.tokens = Some(client.tokens().await);
        return report;
    }
    let name = name.unwrap_or_default();
    for i in 0..2 {
        let key = date(today - Duration::days(i));
        for kind in DAILY {
            if !fetch(
                store,
                client,
                &mut report,
                kind,
                &key,
                Some(name),
                budget,
                now,
            )
            .await
            {
                break;
            }
        }
        if report.error.is_some() || report.rate_limited || report.requests >= budget {
            break;
        }
    }
    if !report.rate_limited && report.error.is_none() {
        for kind in SLOW {
            let key = if *kind == "records" {
                "all".to_owned()
            } else {
                date(today)
            };
            if store.get(kind, &key).is_none()
                && !fetch(
                    store,
                    client,
                    &mut report,
                    kind,
                    &key,
                    Some(name),
                    budget,
                    now,
                )
                .await
            {
                break;
            }
        }
    }
    if !report.rate_limited && report.error.is_none() && report.requests < budget {
        // Scan newest-first on every run. A previous run may have exhausted its budget
        // after indexing a list but before retrieving its activity details.
        let cutoff = date(today - Duration::days(89));
        for page in 0..20 {
            let key = (page * 20).to_string();
            if !fetch(
                store,
                client,
                &mut report,
                "activities",
                &key,
                Some(name),
                budget,
                now,
            )
            .await
            {
                break;
            }
            let Some(list) = store.get("activities", &key) else {
                break;
            };
            let Some(items) = list.as_array() else { break };
            let mut reached_old = false;
            for item in items {
                if item["startTimeLocal"]
                    .as_str()
                    .is_some_and(|d| d < cutoff.as_str())
                {
                    reached_old = true;
                    break;
                }
                let Some(id) = item["activityId"]
                    .as_u64()
                    .map(|v| v.to_string())
                    .or_else(|| item["activityId"].as_str().map(str::to_owned))
                else {
                    continue;
                };
                let is_new = store.get("activity", &id).is_none();
                for kind in ["activity", "splits", "zones", "details"] {
                    if store.get(kind, &id).is_none()
                        && !fetch(
                            store,
                            client,
                            &mut report,
                            kind,
                            &id,
                            Some(name),
                            budget,
                            now,
                        )
                        .await
                    {
                        break;
                    }
                }
                if is_new && store.get("activity", &id).is_some() {
                    report.activities += 1;
                }
                if report.rate_limited || report.error.is_some() || report.requests >= budget {
                    break;
                }
            }
            if reached_old
                || items.len() < 20
                || report.rate_limited
                || report.error.is_some()
                || report.requests >= budget
            {
                break;
            }
        }
    }
    if !report.rate_limited && report.error.is_none() && report.requests < budget {
        let mut day = store
            .cursor("backfill_day")
            .and_then(|s| NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok())
            .unwrap_or(today - Duration::days(2));
        while day >= today - Duration::days(89) && report.requests < budget {
            let key = date(day);
            for kind in DAILY {
                if store.get(kind, &key).is_none()
                    && !fetch(
                        store,
                        client,
                        &mut report,
                        kind,
                        &key,
                        Some(name),
                        budget,
                        now,
                    )
                    .await
                {
                    break;
                }
            }
            if report.rate_limited || report.error.is_some() || report.requests >= budget {
                break;
            }
            day -= Duration::days(1);
            let _ = store.set_cursor("backfill_day", &date(day));
        }
    }
    let key = date(today);
    report.sleep_present = store
        .get("sleep", &key)
        .is_some_and(|v| !v.is_null() && v["dailySleepDTO"].is_object());
    report.readiness_present = store
        .get("readiness", &key)
        .is_some_and(|v| v.as_array().is_some_and(|a| !a.is_empty()) || v.is_object());
    if report.error.is_none() && !report.rate_limited {
        let _ = store.set_cursor("last_sync", &now_local.to_rfc3339());
        let _ = store.set_cursor("last_error", "");
    }
    report.tokens = Some(client.tokens().await);
    report
}
#[allow(clippy::too_many_arguments)] // Explicit request context keeps budget and timestamp consistent across every endpoint.
async fn fetch(
    store: &Store,
    client: &Client,
    report: &mut SyncReport,
    kind: &str,
    key: &str,
    name: Option<&str>,
    budget: usize,
    now: DateTime<Utc>,
) -> bool {
    if report.requests >= budget || report.rate_limited || report.error.is_some() {
        return false;
    }
    report.requests += 1;
    match client.endpoint(kind, key, name).await {
        Ok(v) => {
            if store.put(kind, key, &now.to_rfc3339(), &v).is_err() {
                report.error = Some("local storage failed".into());
                return false;
            }
            report.stored += 1;
            true
        }
        Err(garmin::Error::NotFound) => {
            // Cache absence so optional device metrics do not consume backfill budget forever.
            let _ = store.put(kind, key, &now.to_rfc3339(), &serde_json::Value::Null);
            true
        }
        Err(e) => {
            failure(store, report, e, now);
            false
        }
    }
}
fn failure(store: &Store, report: &mut SyncReport, error: garmin::Error, now: DateTime<Utc>) {
    report.auth_failed = matches!(error, garmin::Error::NeedsSignIn);
    if let garmin::Error::RateLimited { retry_after } = error {
        report.rate_limited = true;
        let _ = store.set_cursor(
            "backoff_until",
            &(now.timestamp() + retry_after.unwrap_or(300) as i64).to_string(),
        );
    } else {
        report.error = Some(error.to_string());
        let _ = store.set_cursor("last_error", report.error.as_deref().unwrap_or(""));
    }
}
