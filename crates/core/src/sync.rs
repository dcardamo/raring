use crate::{
    garmin::{self, Client, Tokens},
    store::Store,
};
use chrono::{DateTime, Duration, Local, NaiveDate, Timelike, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SyncReport {
    pub requests: usize,
    pub stored: usize,
    pub activities: usize,
    pub data_failures: usize,
    pub sleep_present: bool,
    pub readiness_present: bool,
    pub rate_limited: bool,
    pub auth_failed: bool,
    pub error: Option<String>,
    // Refreshed credentials are persisted separately, never included in diagnostic JSON.
    #[serde(skip)]
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
    for i in 0..(if (4..=10).contains(&now_local.hour()) {
        2
    } else {
        1
    }) {
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
        // Garmin's seven-day averages change with each new night: refetch until
        // they include last night once it has synced.
        let key = date(today);
        let night = store.get("sleep", &key).is_some_and(|v| {
            v["dailySleepDTO"]["sleepTimeSeconds"]
                .as_f64()
                .is_some_and(|s| s > 0.)
        });
        let stats = store.get("sleep_stats", &key);
        let current = stats.as_ref().is_some_and(|v| {
            !night
                || v["individualStats"]
                    .as_array()
                    .is_some_and(|a| a.iter().any(|n| n["calendarDate"] == key.as_str()))
        });
        // Garmin may answer empty, null or not at all; ask at most hourly.
        let checked = format!("checked:sleep_stats:{key}");
        let recent = store
            .cursor(&checked)
            .and_then(|s| s.parse::<i64>().ok())
            .is_some_and(|t| now.timestamp() - t < 3600);
        if !current && !recent {
            let before = report.requests;
            fetch(
                store,
                client,
                &mut report,
                "sleep_stats",
                &key,
                Some(name),
                budget,
                now,
            )
            .await;
            if report.requests > before {
                let _ = store.set_cursor(&checked, &now.timestamp().to_string());
            }
        }
    }
    if !report.rate_limited && report.error.is_none() {
        for kind in SLOW {
            let key = if *kind == "records" {
                "all".to_owned()
            } else {
                date(today)
            };
            if (store.get(kind, &key).is_none()
                || (*kind == "records"
                    && store
                        .cursor("records_checked")
                        .and_then(|s| s.parse::<i64>().ok())
                        .is_none_or(|t| now.timestamp() - t > 7 * 86400)))
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
        let cutoff = date(today - Duration::days(RECENT_DAYS - 1));
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
        // Outside the morning refresh window, yesterday still needs its first backfill.
        let mut day = today - Duration::days(1);
        while day >= today - Duration::days(RECENT_DAYS - 1) && report.requests < budget {
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
        }
    }
    if !report.rate_limited && report.error.is_none() && report.requests < budget {
        history(store, client, &mut report, today, budget, now).await;
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
        if report.data_failures == 0 {
            let _ = store.set_cursor("last_error", "");
        }
    }
    report.tokens = Some(client.tokens().await);
    report
}
enum Got {
    Value(Value),
    /// Garmin answered with no content.
    Absent,
    /// A data failure, now or within the retry hour; try again on a later run.
    Later,
}
/// `None` means the run must stop: budget spent, rate limited, signed out or offline.
#[allow(clippy::too_many_arguments)] // Explicit request context keeps budget and timestamp consistent across every endpoint.
async fn request(
    store: &Store,
    client: &Client,
    report: &mut SyncReport,
    kind: &str,
    key: &str,
    name: Option<&str>,
    budget: usize,
    now: DateTime<Utc>,
) -> Option<Got> {
    if report.requests >= budget || report.rate_limited || report.error.is_some() {
        return None;
    }
    let retry_key = format!("retry:{kind}:{key}");
    if store
        .cursor(&retry_key)
        .and_then(|s| s.parse::<i64>().ok())
        .is_some_and(|t| t > now.timestamp())
    {
        return Some(Got::Later);
    }
    report.requests += 1;
    match client.endpoint(kind, key, name).await {
        Ok(v) => {
            let _ = store.set_cursor(&retry_key, "0");
            Some(Got::Value(v))
        }
        Err(garmin::Error::NotFound) => Some(Got::Absent),
        Err(e @ (garmin::Error::Decode | garmin::Error::Http(_))) => {
            report.data_failures += 1;
            let _ = store.set_cursor(&retry_key, &(now.timestamp() + 3600).to_string());
            let _ = store.set_cursor("last_error", &format!("{kind}: {e}"));
            Some(Got::Later)
        }
        Err(e) => {
            failure(store, report, e, now);
            None
        }
    }
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
    let (value, found) = match request(store, client, report, kind, key, name, budget, now).await {
        None => return false,
        Some(Got::Later) => return true,
        Some(Got::Value(v)) => (v, true),
        // Cache absence so optional device metrics do not consume backfill budget forever.
        Some(Got::Absent) => (Value::Null, false),
    };
    if store.put(kind, key, &now.to_rfc3339(), &value).is_err() {
        report.error = Some("local storage failed".into());
        return false;
    }
    if found {
        report.stored += 1;
        if kind == "records" {
            let _ = store.set_cursor("records_checked", &now.timestamp().to_string());
        }
    }
    true
}

/// Days older than the recent window come from range requests, one row per day.
struct Series {
    name: &'static str,
    kind: &'static str,
    /// Inclusive days per request; Garmin rejects longer ranges.
    span: i64,
}
const HISTORY: &[Series] = &[
    // Summaries and activities go first: they bound how far back the others look.
    Series {
        name: "summary",
        kind: "summary_range",
        span: 365,
    },
    Series {
        name: "activities",
        kind: "activities_range",
        span: 365,
    },
    Series {
        name: "readiness",
        kind: "readiness_range",
        span: 365,
    },
    Series {
        name: "hrv",
        kind: "hrv_range",
        span: 365,
    },
    Series {
        name: "max_metrics",
        kind: "max_metrics_range",
        span: 365,
    },
    Series {
        name: "sleep",
        kind: "sleep_range",
        span: 28,
    },
];
/// The per-day sync covers today and the 89 days before it.
pub const RECENT_DAYS: i64 = 90;
/// Older than any Garmin Connect account; an empty year costs one request.
const FLOOR: (i32, u32, u32) = (2000, 1, 1);

fn floor() -> NaiveDate {
    NaiveDate::from_ymd_opt(FLOOR.0, FLOOR.1, FLOOR.2).unwrap_or_default()
}
fn day(s: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(s.get(..10)?, "%Y-%m-%d").ok()
}
fn covered(store: &Store, name: &str) -> Option<(NaiveDate, NaiveDate)> {
    let lo = day(&store.cursor(&format!("history:{name}:lo"))?)?;
    let hi = day(&store.cursor(&format!("history:{name}:hi"))?)?;
    Some((lo, hi))
}
/// How far back a series has to go, or `None` while the series it depends on are unfinished.
fn series_floor(store: &Store, name: &str) -> Option<NaiveDate> {
    if name == "summary" || name == "activities" {
        return Some(floor());
    }
    let done = |n: &str| covered(store, n).is_some_and(|(lo, _)| lo <= floor());
    if !done("summary") || !done("activities") {
        return None;
    }
    let wellness = store
        .earliest_day(&["summary", "daily_summary"])
        .and_then(|d| day(&d));
    let activity = store.oldest_activity().and_then(|d| day(&d));
    // VO2 max can come from workouts recorded before any all-day wellness data.
    let start = if name == "max_metrics" {
        wellness.into_iter().chain(activity).min()
    } else {
        wellness
    };
    // With no data at all there is nothing older to fetch.
    Some(start.unwrap_or(NaiveDate::MAX))
}
/// The next range to fetch: newer days that left the recent window first, then older ones.
fn next_chunk(store: &Store, series: &Series, today: NaiveDate) -> Option<(NaiveDate, NaiveDate)> {
    let target = today - Duration::days(RECENT_DAYS);
    let floor = series_floor(store, series.name)?;
    let width = Duration::days(series.span - 1);
    match covered(store, series.name) {
        Some((_, hi)) if hi < target => Some((hi + Duration::days(1), (hi + width).min(target))),
        Some((lo, _)) if lo > floor => {
            let end = lo - Duration::days(1);
            Some(((end - width).max(floor), end))
        }
        Some(_) => None,
        None if target < floor => None,
        None => Some(((target - width).max(floor), target)),
    }
}
fn mark_covered(store: &Store, name: &str, start: NaiveDate, end: NaiveDate) {
    let (lo, hi) =
        covered(store, name).map_or((start, end), |(lo, hi)| (lo.min(start), hi.max(end)));
    let _ = store.set_cursor(&format!("history:{name}:lo"), &date(lo));
    let _ = store.set_cursor(&format!("history:{name}:hi"), &date(hi));
}
/// Writes one row per day. Readiness and VO2 max ranges hold exactly the per-day responses,
/// so they fill those rows; the others keep Garmin's range item under their own endpoint.
fn store_range(store: &Store, kind: &str, value: &Value, at: &str) -> rusqlite::Result<usize> {
    let items = match kind {
        "summary_range" => value["data"].as_array(),
        "hrv_range" => value["hrvSummaries"].as_array(),
        _ => value.as_array(),
    };
    let items: &[Value] = items.map_or(&[], Vec::as_slice);
    if kind == "activities_range" {
        let mut stored = 0;
        for item in items {
            let id = item["activityId"]
                .as_u64()
                .map(|v| v.to_string())
                .or_else(|| item["activityId"].as_str().map(str::to_owned));
            if let Some(id) = id {
                store.put("listed", &id, at, item)?;
                stored += 1;
            }
        }
        return Ok(stored);
    }
    let mut days: std::collections::BTreeMap<String, Vec<Value>> = Default::default();
    for item in items {
        let date = if kind == "max_metrics_range" {
            ["generic", "cycling", "heatAltitudeAcclimation"]
                .iter()
                .find_map(|k| item[*k]["calendarDate"].as_str())
        } else {
            item["calendarDate"].as_str()
        };
        if let Some(date) = date.filter(|d| day(d).is_some()) {
            days.entry(date.to_owned()).or_default().push(item.clone());
        }
    }
    let endpoint = match kind {
        "summary_range" => "daily_summary",
        "sleep_range" => "sleep_summary",
        "hrv_range" => "hrv_summary",
        "readiness_range" => "readiness",
        _ => "max_metrics",
    };
    let per_day = matches!(kind, "readiness_range" | "max_metrics_range");
    let mut stored = 0;
    for (date, mut entries) in days {
        if per_day && store.get(endpoint, &date).is_some() {
            continue;
        }
        let value = if per_day {
            Value::Array(entries)
        } else {
            entries.swap_remove(0)
        };
        store.put(endpoint, &date, at, &value)?;
        stored += 1;
    }
    Ok(stored)
}
/// Downloads history older than the recent window with whatever budget the run has left.
/// Each series resumes where the last run stopped, so a new install catches up over a few runs.
pub async fn history(
    store: &Store,
    client: &Client,
    report: &mut SyncReport,
    today: NaiveDate,
    budget: usize,
    now: DateTime<Utc>,
) {
    let page: usize = garmin::ACTIVITY_PAGE.parse().unwrap_or(100);
    for series in HISTORY {
        while let Some((start, end)) = next_chunk(store, series, today) {
            let range = format!("{}:{}", date(start), date(end));
            // A partly read activity range resumes at its saved offset.
            let page_key = format!("history:{}:page", series.name);
            let offset = store
                .cursor(&page_key)
                .and_then(|s| s.strip_prefix(&format!("{range}:")).map(str::to_owned))
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0);
            let key = if series.kind == "activities_range" {
                format!("{range}:{offset}")
            } else {
                range.clone()
            };
            let value =
                match request(store, client, report, series.kind, &key, None, budget, now).await {
                    None => return,
                    Some(Got::Later) => break,
                    Some(Got::Absent) => Value::Null,
                    Some(Got::Value(v)) => v,
                };
            match store_range(store, series.kind, &value, &now.to_rfc3339()) {
                Ok(n) => report.stored += n,
                Err(_) => {
                    report.error = Some("local storage failed".into());
                    return;
                }
            }
            let full = value.as_array().is_some_and(|a| a.len() >= page);
            if series.kind == "activities_range" && full {
                let _ = store.set_cursor(&page_key, &format!("{range}:{}", offset + page));
                continue;
            }
            mark_covered(store, series.name, start, end);
        }
    }
}
/// The oldest day stored, and whether older history is still downloading.
pub fn history_progress(store: &Store, today: NaiveDate) -> (Option<String>, bool) {
    let target = today - Duration::days(RECENT_DAYS);
    let from = [
        store.earliest_day(&["summary", "daily_summary"]),
        store
            .oldest_activity()
            .and_then(|d| d.get(..10).map(str::to_owned)),
    ]
    .into_iter()
    .flatten()
    .min();
    let complete = HISTORY.iter().all(|s| {
        series_floor(store, s.name).is_some_and(|floor| match covered(store, s.name) {
            Some((lo, _)) => lo <= floor,
            None => target < floor,
        })
    });
    (from, complete)
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
