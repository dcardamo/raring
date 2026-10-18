use crate::{
    garmin::{self, Client, Tokens},
    store::Store,
    strength,
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
    "health_status",
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
    // Watch freshness takes priority over backfill; a small budget must not
    // leave today's upload time hidden behind activity history.
    sync_watch(store, client, &mut report, budget, now).await;
    for i in 0..2 {
        if report.error.is_some() || report.rate_limited || report.requests >= budget {
            break;
        }
        let key = date(today - Duration::days(i));
        if !fetch(
            store,
            client,
            &mut report,
            "daily_hr",
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
    if !report.rate_limited && report.error.is_none() {
        sync_intensity_weeks(store, client, &mut report, today, budget, now).await;
        sync_weight(store, client, &mut report, today, budget, now).await;
        sync_gear(store, client, &mut report, budget, now).await;
        sync_thresholds(store, client, &mut report, today, budget, now).await;
    }
    // The app uses a 60-request sync. Tiny explicit budgets keep existing daily
    // metrics first; planned data waits for a normal run.
    if budget >= 20 && !report.rate_limited && report.error.is_none() {
        sync_planned(store, client, &mut report, today, budget, now).await;
    }
    if budget >= 20 && !report.rate_limited && report.error.is_none() {
        sync_naps_journal(store, client, &mut report, today, budget, now).await;
    }
    if !report.rate_limited && report.error.is_none() && report.requests < budget {
        // Scan newest-first on every run. A previous run may have exhausted its budget
        // after indexing a list but before retrieving its activity details.
        let cutoff = date(today - Duration::days(RECENT_DAYS - 1));
        // The oldest day this run's freshly read pages list completely.
        let mut listed_from: Option<NaiveDate> = None;
        for page in 0..20 {
            let key = (page * 20).to_string();
            let before = report.stored;
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
            // A page served from the cache (retry wait) says nothing new.
            let fresh = report.stored > before
                && items.iter().all(|item| {
                    item["startTimeLocal"].as_str().and_then(day).is_some()
                        && (item["activityId"].as_u64().is_some()
                            || item["activityId"].as_str().is_some_and(|s| !s.is_empty()))
                });
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
                if strength::is_strength(item)
                    && sets_due(store, &id, item, now)
                    && !fetch(
                        store,
                        client,
                        &mut report,
                        "exercise_sets",
                        &id,
                        None,
                        budget,
                        now,
                    )
                    .await
                {
                    break;
                }
                if is_new && store.get("activity", &id).is_some() {
                    report.activities += 1;
                }
                if report.rate_limited || report.error.is_some() || report.requests >= budget {
                    break;
                }
            }
            if fresh {
                listed_from = if items.len() < 20 {
                    Some(floor())
                } else if items.iter().any(|i| {
                    i["startTimeLocal"]
                        .as_str()
                        .is_some_and(|d| d < cutoff.as_str())
                }) {
                    day(&cutoff)
                } else {
                    // The oldest listed day may continue on the next page.
                    items
                        .iter()
                        .filter_map(|i| i["startTimeLocal"].as_str().and_then(day))
                        .min()
                        .map(|d| d + Duration::days(1))
                };
            } else if !fresh {
                break;
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
        if let Some(from) = listed_from.filter(|from| *from <= today) {
            mark_listed(store, from, today);
        }
    }
    if !report.rate_limited && report.error.is_none() && report.requests < budget {
        strength_reference(store, client, &mut report, budget, now).await;
    }
    if !report.rate_limited && report.error.is_none() && report.requests < budget {
        // Outside the morning refresh window, yesterday still needs its first backfill.
        let mut day = today - Duration::days(1);
        while day >= today - Duration::days(RECENT_DAYS - 1) && report.requests < budget {
            let key = date(day);
            for kind in DAILY {
                // Health Status is only charted for 30 nights; keep older-history
                // request budget for Garmin's range endpoints.
                if *kind == "health_status" && day < today - Duration::days(29) {
                    continue;
                }
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
    if !report.rate_limited && report.error.is_none() && report.requests < budget {
        // Sets of older strength activities, found by the history download.
        for id in store.strength_without_sets(budget) {
            if !fetch(
                store,
                client,
                &mut report,
                "exercise_sets",
                &id,
                None,
                budget,
                now,
            )
            .await
                || report.requests >= budget
            {
                break;
            }
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
        if report.data_failures == 0 {
            let _ = store.set_cursor("last_error", "");
        }
    }
    report.tokens = Some(client.tokens().await);
    report
}
/// Sets can be corrected in Connect soon after a workout: fetch them once, and
/// again at most hourly for two days after the activity starts.
fn sets_due(store: &Store, id: &str, item: &Value, now: DateTime<Utc>) -> bool {
    let Some(fetched) = store.fetched_at("exercise_sets", id) else {
        return true;
    };
    let started = item["startTimeGMT"]
        .as_str()
        .and_then(|s| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").ok())
        .map(|t| t.and_utc());
    let fetched = DateTime::parse_from_rfc3339(&fetched).map(|t| t.with_timezone(&Utc));
    started.is_some_and(|t| now - t < Duration::hours(48))
        && fetched.is_ok_and(|f| now - f >= Duration::hours(1))
}
/// Garmin's exercise catalog (monthly) requires sets; reduced user settings
/// provide display units for strength, weight and recorded Training metrics.
async fn strength_reference(
    store: &Store,
    client: &Client,
    report: &mut SyncReport,
    budget: usize,
    now: DateTime<Utc>,
) {
    let has_sets = !store.history("exercise_sets", "~", 1).is_empty();
    let has_weight = !store.history("weight", "~", 1).is_empty()
        || store
            .get("weight_latest", "me")
            .is_some_and(|v| !v.is_null());
    let has_threshold = ["lactate_threshold", "running_power_threshold", "cycling_ftp", "acclimation"]
        .iter()
        .any(|kind| store.get(kind, "latest").is_some_and(|v| !v.is_null()));
    if !has_sets && !has_weight && !has_threshold {
        return;
    }
    let due = |cursor: &str, days: i64| {
        store
            .cursor(cursor)
            .and_then(|s| s.parse::<i64>().ok())
            .is_none_or(|t| t > now.timestamp() || now.timestamp() - t >= days * 86400)
    };
    for (kind, key, cursor, days) in [
        ("profile_settings", "me", "profile_settings_checked", 7),
        ("exercise_catalog", "garmin", "exercise_catalog_checked", 30),
    ] {
        if !due(cursor, days) || (kind == "exercise_catalog" && !has_sets) {
            continue;
        }
        match request(store, client, report, kind, key, None, budget, now).await {
            None => return,
            Some(Got::Value(v)) => {
                let value = if kind == "profile_settings" {
                    strength::reduce_settings(&v)
                } else {
                    v
                };
                if store.put(kind, key, &now.to_rfc3339(), &value).is_err() {
                    report.error = Some("local storage failed".into());
                    return;
                }
                report.stored += 1;
                let _ = store.set_cursor(cursor, &now.timestamp().to_string());
            }
            Some(Got::Absent) => {
                let _ = store.set_cursor(cursor, &now.timestamp().to_string());
            }
            Some(Got::Later) => {}
        }
    }
}
/// The list is hourly, but the selected watch's upload time changes on every sync.
/// Keep the raw list and raw detail separately so a changed selection cannot
/// accidentally display the previous watch's timestamp.
pub(crate) async fn sync_watch(
    store: &Store,
    client: &Client,
    report: &mut SyncReport,
    budget: usize,
    now: DateTime<Utc>,
) {
    let due = store
        .cursor("devices_checked")
        .and_then(|s| s.parse::<i64>().ok())
        .is_none_or(|t| t > now.timestamp() || now.timestamp() - t >= 3600);
    if due {
        match request(store, client, report, "devices", "", None, budget, now).await {
            Some(Got::Value(list)) if list.is_array() => {
                if store
                    .put("devices", "me", &now.to_rfc3339(), &list)
                    .is_err()
                    || store
                        .set_cursor("devices_checked", &now.timestamp().to_string())
                        .is_err()
                {
                    report.error = Some("local storage failed".into());
                    return;
                }
                report.stored += 1;
            }
            Some(Got::Value(_)) => {
                report.data_failures += 1;
                let _ = store.set_cursor("retry:devices:", &(now.timestamp() + 3600).to_string());
            }
            Some(Got::Absent) => {
                // An account without a device list should not retry every
                // 15-minute morning sync. Recheck after the usual hour.
                if store
                    .set_cursor("devices_checked", &now.timestamp().to_string())
                    .is_err()
                {
                    report.error = Some("local storage failed".into());
                    return;
                }
            }
            Some(Got::Later) | None => {}
        }
    }
    let Some(id) = store
        .get("devices", "me")
        .and_then(|v| selected_watch_id(&v))
    else {
        return;
    };
    if let Some(Got::Value(detail)) = request(
        store,
        client,
        report,
        "device_detail",
        &id,
        None,
        budget,
        now,
    )
    .await
    {
        // Only a matching detail may replace the selected snapshot.
        if detail["deviceId"]
            .as_u64()
            .is_some_and(|n| n.to_string() == id)
        {
            if store
                .put("device_detail", "selected", &now.to_rfc3339(), &detail)
                .is_err()
            {
                report.error = Some("local storage failed".into());
            } else {
                report.stored += 1;
            }
        } else {
            report.data_failures += 1;
        }
    }
}

/// Refresh the current schedule as a snapshot: an empty answer must replace the old
/// list, while a failed read leaves the old snapshot (visibly stale). All calls use
/// the shared request budget, pacing and backoff.
async fn sync_planned(
    store: &Store,
    client: &Client,
    report: &mut SyncReport,
    today: NaiveDate,
    budget: usize,
    now: DateTime<Utc>,
) {
    let start = date(today);
    let end = date(today + Duration::days(14));
    let calls = [
        ("coach_plans", start.as_str()),
        ("scheduled_workouts", ""),
        ("upcoming_events", "all"),
    ];
    let schedule_key = format!("{start}:{end}");
    for (kind, key) in calls {
        let key = if kind == "scheduled_workouts" {
            &schedule_key
        } else {
            key
        };
        let checked = format!("planned_checked:{kind}");
        if store
            .cursor(&checked)
            .and_then(|s| s.parse::<i64>().ok())
            .is_some_and(|t| t <= now.timestamp() && now.timestamp() - t < 6 * 3600)
        {
            continue;
        }
        let result = request(store, client, report, kind, key, None, budget, now).await;
        let value = match result {
            Some(Got::Value(v)) => {
                let valid = match kind {
                    "coach_plans" => v["trainingPlanList"].is_array(),
                    _ => v.is_array(),
                };
                if !valid {
                    report.data_failures += 1;
                    let _ = store.set_cursor(&format!("retry:{kind}:{key}"), &(now.timestamp() + 3600).to_string());
                    continue;
                }
                report.stored += 1;
                v
            }
            Some(Got::Absent) => Value::Null,
            Some(Got::Later) => continue,
            None => return,
        };
        let snapshot = if kind == "scheduled_workouts" {
            serde_json::json!({"startDate": start, "endDate": end, "entries": value})
        } else {
            value
        };
        if store
            .put(kind, "current", &now.to_rfc3339(), &snapshot)
            .is_err()
            || store
                .set_cursor(&checked, &now.timestamp().to_string())
                .is_err()
        {
            report.error = Some("local storage failed".into());
            return;
        }
    }
}

/// Garmin's weekly intensity rows, read at most daily. Their dates mark where
/// Garmin's intensity week starts, which can differ from the account's
/// start-of-week setting.
async fn sync_intensity_weeks(
    store: &Store,
    client: &Client,
    report: &mut SyncReport,
    today: NaiveDate,
    budget: usize,
    now: DateTime<Utc>,
) {
    let due = store
        .cursor("intensity_weeks_checked")
        .and_then(|s| s.parse::<i64>().ok())
        .is_none_or(|t| t > now.timestamp() || now.timestamp() - t >= 86400);
    if !due {
        return;
    }
    let key = date(today);
    let rows = match request(
        store,
        client,
        report,
        "intensity_weeks",
        &key,
        None,
        budget,
        now,
    )
    .await
    {
        Some(Got::Value(v))
            if v.as_array().is_some_and(|rows| {
                rows.iter()
                    .all(|r| r["calendarDate"].as_str().and_then(day).is_some())
            }) =>
        {
            Some(v)
        }
        Some(Got::Value(_)) => {
            report.data_failures += 1;
            None
        }
        Some(Got::Absent) => None,
        Some(Got::Later) | None => return,
    };
    if rows.as_ref().is_some_and(|v| {
        store
            .put("intensity_weeks", "me", &now.to_rfc3339(), v)
            .is_err()
    }) || store
        .set_cursor("intensity_weeks_checked", &now.timestamp().to_string())
        .is_err()
    {
        report.error = Some("local storage failed".into());
        return;
    }
    if rows.is_some() {
        report.stored += 1;
    }
}

/// Latest-only device metrics: four reads at most once per day, including
/// empty responses and 404s. Failed reads preserve cached values; the request
/// helper applies bounded retry and rate-limit backoff.
async fn sync_thresholds(store: &Store, client: &Client, report: &mut SyncReport, today: NaiveDate, budget: usize, now: DateTime<Utc>) {
    for kind in ["lactate_threshold", "running_power_threshold", "cycling_ftp", "acclimation"] {
        let cursor = format!("{kind}_checked");
        if !threshold_due(store, &cursor, now.timestamp()) { continue; }
        let value = match request(store, client, report, kind, &date(today), None, budget, now).await {
            Some(Got::Value(v)) => v,
            Some(Got::Absent) => {
                // An unavailable latest endpoint does not invalidate a previously
                // recorded reading. Cache the check so a 404 cannot poll every sync.
                if store.get(kind, "latest").is_some_and(|v| !v.is_null()) {
                    if store.set_cursor(&cursor, &now.timestamp().to_string()).is_err() {
                        report.error = Some("local storage failed".into());
                        return;
                    }
                    continue;
                }
                Value::Null
            }
            // Retry is per metric; request still enforces global auth/rate/budget stops.
            Some(Got::Later) => continue,
            None => return,
        };
        if store.put(kind, "latest", &now.to_rfc3339(), &value).is_err()
            || store.set_cursor(&cursor, &now.timestamp().to_string()).is_err() {
            report.error = Some("local storage failed".into());
            return;
        }
        if !value.is_null() { report.stored += 1; }
    }
}

fn threshold_due(store: &Store, cursor: &str, now: i64) -> bool {
    !store.cursor(cursor).and_then(|v| v.parse::<i64>().ok())
        .is_some_and(|t| t <= now && now - t < 86400)
}

/// Weight is sparse. Check the latest entry daily and reread the recent range
/// weekly for edits; empty ranges are checked too, without per-day requests.
async fn sync_weight(
    store: &Store,
    client: &Client,
    report: &mut SyncReport,
    today: NaiveDate,
    budget: usize,
    now: DateTime<Utc>,
) {
    for (kind, interval) in [("weight_latest", 86400), ("weight_recent", 7 * 86400)] {
        let cursor = format!("{kind}_checked");
        if store
            .cursor(&cursor)
            .and_then(|v| v.parse::<i64>().ok())
            .is_some_and(|t| t <= now.timestamp() && now.timestamp() - t < interval)
        {
            continue;
        }
        let key = if kind == "weight_latest" {
            date(today)
        } else {
            format!("{}:{}", date(today - Duration::days(89)), date(today))
        };
        let endpoint = if kind == "weight_latest" {
            kind
        } else {
            "weight_range"
        };
        let value = match request(store, client, report, endpoint, &key, None, budget, now).await {
            Some(Got::Value(v)) => {
                if kind == "weight_recent" && !valid_weight_range(&v) {
                    report.data_failures += 1;
                    return;
                }
                Some(v)
            }
            Some(Got::Absent) => None,
            Some(Got::Later) | None => return,
        };
        let result = if kind == "weight_latest" {
            store
                .put(
                    "weight_latest",
                    "me",
                    &now.to_rfc3339(),
                    &value.unwrap_or(Value::Null),
                )
                .map(|_| 1)
        } else if let Some(v) = value {
            store_range(store, "weight_range", &v, &now.to_rfc3339())
        } else {
            Ok(0)
        };
        match result {
            Ok(n) => report.stored += n,
            Err(_) => {
                report.error = Some("local storage failed".into());
                return;
            }
        }
        if store
            .set_cursor(&cursor, &now.timestamp().to_string())
            .is_err()
        {
            report.error = Some("local storage failed".into());
            return;
        }
    }
}

/// Gear is a complete snapshot, not an append-only series. A failed or malformed
/// list must not remove previously retired/deleted items. An empty valid list does.
#[doc(hidden)]
pub async fn sync_gear(
    store: &Store,
    client: &Client,
    report: &mut SyncReport,
    budget: usize,
    now: DateTime<Utc>,
) {
    if store.cursor("gear_checked")
        .and_then(|v| v.parse::<i64>().ok())
        .is_some_and(|t| t <= now.timestamp() && now.timestamp() - t < 86400)
    {
        return;
    }
    let result = request(store, client, report, "gear", "me", None, budget, now).await;
    let snapshot = match result {
        Some(Got::Value(v)) if crate::gear::valid_list(&v) => Some(v),
        Some(Got::Value(_)) => {
            report.data_failures += 1;
            // A malformed 200 is not an empty list, but must not spin on
            // every hourly worker run either.
            if store.set_cursor("gear_checked", &now.timestamp().to_string()).is_err() {
                report.error = Some("local storage failed".into());
            }
            return;
        }
        Some(Got::Absent) => None,
        Some(Got::Later) | None => return,
    };
    if snapshot.as_ref().is_some_and(|v| store.put("gear", "me", &now.to_rfc3339(), v).is_err())
        || store.set_cursor("gear_checked", &now.timestamp().to_string()).is_err()
    {
        report.error = Some("local storage failed".into());
    } else if snapshot.is_some() {
        report.stored += 1;
    }
}

fn valid_weight_range(value: &Value) -> bool {
    value["dateWeightList"].as_array().is_some_and(|rows| {
        rows.iter().all(|row| {
            row["calendarDate"].as_str().and_then(day).is_some()
                && row["weight"]
                    .as_f64()
                    .is_some_and(|w| w.is_finite() && w > 0.)
        })
    })
}

/// A unique primary tracker is authoritative. Without one, a lone registration
/// is the only possible watch; with multiple registrations, require one
/// watch-like candidate and explicit evidence that every other is an accessory.
pub(crate) fn selected_watch(list: &Value) -> Option<&Value> {
    let items = list.as_array()?;
    let id = |v: &&Value| v["deviceId"].as_u64().filter(|n| *n > 0).is_some();
    let primaries: Vec<_> = items
        .iter()
        .filter(id)
        .filter(|v| v["primaryActivityTrackerIndicator"] == true)
        .collect();
    if primaries.len() == 1 {
        return primaries.into_iter().next();
    }
    if !primaries.is_empty() {
        return None;
    }
    let accessory = |v: &Value| {
        v["deviceCategories"].as_array().is_some_and(|categories| {
            categories
                .iter()
                .any(|category| category == "DEDICATED_BIKE_COMPUTER")
        })
    };
    if items.len() == 1 {
        return items.first().filter(|v| id(v) && !accessory(v));
    }
    let watch_like = |v: &Value| {
        [
            "sleepScoreCapable",
            "hrvStatusCapable",
            "trainingReadinessCapable",
        ]
        .iter()
        .any(|field| v[*field] == true)
            || v["deviceCategories"]
                .as_array()
                .is_some_and(|categories| categories.iter().any(|category| category == "WELLNESS"))
    };
    let candidates: Vec<_> = items.iter().filter(|v| id(v) && !accessory(v)).collect();
    (candidates.len() == 1
        && watch_like(candidates[0])
        && items
            .iter()
            .all(|v| std::ptr::eq(v, candidates[0]) || accessory(v)))
    .then(|| candidates[0])
}

fn selected_watch_id(list: &Value) -> Option<String> {
    selected_watch(list)?["deviceId"]
        .as_u64()
        .map(|id| id.to_string())
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
    if report.requests >= budget || report.rate_limited || report.error.is_some()
        || (kind == "journal" && !store.settings().journal_enabled)
    {
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
    let response = if kind == "journal" {
        client.endpoint_guarded(kind, key, name, || store.settings().journal_enabled).await
    } else if kind == "health_status" {
        if NaiveDate::parse_from_str(key, "%Y-%m-%d").is_err() {
            Err(garmin::Error::Decode)
        } else {
            client
                .graphql(
                    &format!("query {{ healthStatusSummary(calendarDate: \"{key}\") {{ calendarDate outliersCount metrics {{ value baselineUpperLimit baselineLowerLimit status percentage feedbackKey type }} }} }}"),
                    serde_json::json!({}),
                )
                .await
                .and_then(|v| {
                    let summary = v.get("healthStatusSummary").ok_or(garmin::Error::Decode)?;
                    if summary.is_null() {
                        return Ok(Value::Null);
                    }
                    let valid = summary["calendarDate"] == key
                        && summary["outliersCount"].as_u64().is_some()
                        && summary["metrics"].as_array().is_some_and(|metrics| {
                            metrics.iter().all(|metric| metric["type"].is_string())
                        });
                    valid.then(|| summary.clone()).ok_or(garmin::Error::Decode)
                })
                .map(Some)
        }
    } else if kind == "exercise_catalog" {
        client
            .exercise_catalog()
            .await
            .and_then(|v| strength::reduce_catalog(&v).ok_or(garmin::Error::Decode))
            .map(Some)
    } else {
        client.endpoint(kind, key, name).await.map(Some)
    };
    // Never treat a reply after opt-out as a journal result. Authentication
    // and rate-limit errors still update their global backoff state.
    if kind == "journal"
        && !matches!(&response, Err(garmin::Error::NeedsSignIn | garmin::Error::RateLimited { .. }))
        && !store.settings().journal_enabled
    {
        return None;
    }
    match response {
        Ok(None) => None,
        Ok(Some(v)) => {
            if kind != "journal" { let _ = store.set_cursor(&retry_key, "0"); }
            Some(Got::Value(v))
        }
        Err(garmin::Error::NotFound) => Some(Got::Absent),
        Err(garmin::Error::Http(403)) if matches!(kind, "naps" | "journal") => Some(Got::Absent),
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
    if kind == "journal" {
        match store.put_journal_if_enabled(key, &now.to_rfc3339(), &value) {
            Ok(true) => { let _ = store.set_cursor(&format!("retry:{kind}:{key}"), "0"); }
            Ok(false) => return false,
            Err(_) => { report.error = Some("local storage failed".into()); return false }
        }
    } else if store.put(kind, key, &now.to_rfc3339(), &value).is_err() {
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

/// Recent nap reads and consented journal reads share the regular budget and
/// retry/backoff path. Cached empty days obey the same cadence as populated days.
async fn sync_naps_journal(store: &Store, client: &Client, report: &mut SyncReport, today: NaiveDate, budget: usize, now: DateTime<Utc>) {
    for (kind, days) in [("naps", 2), ("journal", 7)] {
        for i in 0..days {
            if kind == "journal" && !store.settings().journal_enabled { return }
            let key = date(today - Duration::days(i));
            let cadence = if i == 0 { 6 * 3600 } else if kind == "journal" || i == 1 { 24 * 3600 } else { 0 };
            let recent = store.fetched_at(kind, &key)
                .and_then(|t| DateTime::parse_from_rfc3339(&t).ok())
                .is_some_and(|t| cadence == 0 || now.timestamp() - t.timestamp() < cadence);
            if recent { continue }
            if !fetch(store, client, report, kind, &key, None, budget, now).await { return }
        }
    }
}

#[cfg(test)]
mod naps_journal_tests {
    use super::*;
    #[tokio::test]
    async fn journal_never_requests_before_local_consent() {
        let store = Store::open(":memory:").unwrap();
        let today = Local::now().date_naive();
        let now = Utc::now();
        for i in 0..2 {
            store.put("naps", &date(today - Duration::days(i)), &now.to_rfc3339(), &serde_json::json!([])).unwrap();
        }
        let client = Client::with_urls(Tokens {
            access_token: "synthetic".into(), refresh_token: "synthetic".into(),
            client_id: "synthetic".into(), expires_at: now + Duration::days(1),
        }, "http://127.0.0.1:1", "http://127.0.0.1:1", std::time::Duration::ZERO).unwrap();
        let mut report = SyncReport::default();
        sync_naps_journal(&store, &client, &mut report, today, 20, now).await;
        assert_eq!(report.requests, 0);
        let mut settings = store.settings();
        settings.journal_enabled = true;
        store.set_settings(&settings).unwrap();
        sync_naps_journal(&store, &client, &mut report, today, 20, now).await;
        assert_eq!(report.requests, 1);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn journal_rate_limit_still_sets_global_backoff_after_opt_out() {
        use std::{io::{Read, Write}, net::TcpListener, sync::{Arc, mpsc}};
        let store = Arc::new(Store::open(":memory:").unwrap());
        let mut settings = store.settings();
        settings.journal_enabled = true;
        store.set_settings(&settings).unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let client = Arc::new(Client::with_urls(Tokens {
            access_token: "synthetic".into(), refresh_token: "synthetic".into(),
            client_id: "synthetic".into(), expires_at: Utc::now() + Duration::days(1),
        }, &url, &url, std::time::Duration::ZERO).unwrap());
        let (arrived_tx, arrived_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = [0; 2048];
            assert!(stream.read(&mut buf).unwrap() > 0);
            arrived_tx.send(()).unwrap();
            release_rx.recv_timeout(std::time::Duration::from_secs(5)).unwrap();
            stream.write_all(b"HTTP/1.1 429 Too Many Requests\r\nRetry-After: 15\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").unwrap();
        });
        let now = Utc::now();
        let day = Local::now().date_naive().to_string();
        let worker = tokio::spawn({
            let store = Arc::clone(&store); let client = Arc::clone(&client); let day = day.clone();
            async move {
                let mut report = SyncReport::default();
                fetch(&store, &client, &mut report, "journal", &day, None, 1, now).await;
                report
            }
        });
        arrived_rx.recv_timeout(std::time::Duration::from_secs(5)).unwrap();
        settings.journal_enabled = false;
        store.set_settings(&settings).unwrap();
        release_tx.send(()).unwrap();
        let report = worker.await.unwrap();
        server.join().unwrap();
        assert!(report.rate_limited);
        assert_eq!(report.data_failures, 0);
        assert_eq!(report.stored, 0);
        assert!(store.cursor("backoff_until").and_then(|s| s.parse::<i64>().ok()).is_some_and(|t| t > now.timestamp()));
        assert_eq!(store.get("journal", &day), None);
    }

    #[tokio::test]
    async fn opt_out_during_client_pacing_never_sends_unsent_get() {
        use std::{io::{Read, Write}, net::TcpListener, sync::{Arc, atomic::{AtomicUsize, Ordering}}};
        let store = Arc::new(Store::open(":memory:").unwrap());
        let mut settings = store.settings();
        settings.journal_enabled = true;
        store.set_settings(&settings).unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let server = std::thread::spawn({
            let listener = listener.try_clone().unwrap();
            move || {
                let (mut stream, _) = listener.accept().unwrap();
                let mut buf = [0; 2048];
                assert!(stream.read(&mut buf).unwrap() > 0);
                stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}").unwrap();
            }
        });
        let client = Client::with_urls(Tokens {
            access_token: "synthetic".into(), refresh_token: "synthetic".into(),
            client_id: "synthetic".into(), expires_at: Utc::now() + Duration::days(1),
        }, &url, &url, std::time::Duration::from_secs(60)).unwrap();
        client.get("/prime", &[]).await.unwrap();
        server.join().unwrap();
        tokio::time::pause();
        let checks = Arc::new(AtomicUsize::new(0));
        let (entered_tx, mut entered_rx) = tokio::sync::mpsc::unbounded_channel();
        let worker = tokio::spawn({
            let store = Arc::clone(&store); let checks = Arc::clone(&checks);
            async move {
                client.get_guarded("/lifestylelogging-service/dailyLog/2025-10-16", &[], || {
                    let enabled = store.settings().journal_enabled;
                    if checks.fetch_add(1, Ordering::SeqCst) == 2 { let _ = entered_tx.send(()); }
                    enabled
                }).await
            }
        });
        entered_rx.recv().await.unwrap(); // Third guard: entered the client and about to wait for pacing.
        assert!(store.settings().journal_enabled);
        settings.journal_enabled = false;
        store.set_settings(&settings).unwrap();
        tokio::time::advance(std::time::Duration::from_secs(61)).await;
        assert!(worker.await.unwrap().unwrap().is_none());
        assert!(checks.load(Ordering::SeqCst) >= 4, "guard must run after pacing");
        listener.set_nonblocking(true).unwrap();
        assert!(listener.accept().is_err(), "paced journal GET was cancelled before dispatch");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn opt_out_during_401_refresh_keeps_tokens_but_skips_retry_get() {
        use std::{io::{Read, Write}, net::TcpListener, sync::{Arc, mpsc}};
        let store = Arc::new(Store::open(":memory:").unwrap());
        let mut settings = store.settings();
        settings.journal_enabled = true;
        store.set_settings(&settings).unwrap();
        let now = Utc::now();
        let today = Local::now().date_naive();
        for i in 0..2 {
            store.put("naps", &date(today - Duration::days(i)), &now.to_rfc3339(), &serde_json::json!([])).unwrap();
        }
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let client = Arc::new(Client::with_urls(Tokens {
            access_token: "old-access".into(), refresh_token: "old-refresh".into(),
            client_id: "synthetic".into(), expires_at: now + Duration::days(1),
        }, &url, &url, std::time::Duration::ZERO).unwrap());
        let (arrived_tx, arrived_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let server = std::thread::spawn({
            let listener = listener.try_clone().unwrap();
            move || {
                let (mut first, _) = listener.accept().unwrap();
                let mut buf = [0; 2048];
                let size = first.read(&mut buf).unwrap();
                assert!(String::from_utf8_lossy(&buf[..size]).starts_with("GET /lifestylelogging-service/dailyLog/"));
                first.write_all(b"HTTP/1.1 401 Unauthorized\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").unwrap();
                let (mut stream, _) = listener.accept().unwrap();
                let size = stream.read(&mut buf).unwrap();
                arrived_tx.send(String::from_utf8_lossy(&buf[..size]).lines().next().unwrap().to_owned()).unwrap();
                release_rx.recv_timeout(std::time::Duration::from_secs(5)).unwrap();
                let body = r#"{"access_token":"rotated-access","refresh_token":"rotated-refresh","expires_in":3600}"#;
                write!(stream, "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).unwrap();
            }
        });
        let worker = tokio::spawn({
            let store = Arc::clone(&store); let client = Arc::clone(&client);
            async move {
                let mut report = SyncReport::default();
                sync_naps_journal(&store, &client, &mut report, today, 20, now).await;
                report
            }
        });
        assert!(arrived_rx.recv_timeout(std::time::Duration::from_secs(5)).unwrap().starts_with("POST "));
        settings.journal_enabled = false;
        store.set_settings(&settings).unwrap();
        release_tx.send(()).unwrap();
        let report = tokio::time::timeout(std::time::Duration::from_secs(5), worker).await.unwrap().unwrap();
        server.join().unwrap();
        assert_eq!(report.requests, 1); // The first GET counts; its retry was cancelled.
        assert_eq!(report.data_failures, 0);
        assert_eq!(report.stored, 0);
        assert_eq!(store.get("journal", &date(today)), None);
        let rotated = client.tokens().await;
        assert_eq!(rotated.access_token, "rotated-access");
        assert_eq!(rotated.refresh_token, "rotated-refresh");
        listener.set_nonblocking(true).unwrap();
        assert!(listener.accept().is_err(), "refresh must not dispatch a cancelled journal GET");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn opt_out_before_401_skips_refresh_and_retry_get() {
        use std::{io::{Read, Write}, net::TcpListener, sync::{Arc, mpsc}};
        let store = Arc::new(Store::open(":memory:").unwrap());
        let mut settings = store.settings();
        settings.journal_enabled = true;
        store.set_settings(&settings).unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let client = Arc::new(Client::with_urls(Tokens {
            access_token: "synthetic".into(), refresh_token: "synthetic".into(),
            client_id: "synthetic".into(), expires_at: Utc::now() + Duration::days(1),
        }, &url, &url, std::time::Duration::ZERO).unwrap());
        let (arrived_tx, arrived_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let server = std::thread::spawn({
            let listener = listener.try_clone().unwrap();
            move || {
                let (mut stream, _) = listener.accept().unwrap();
                let mut buf = [0; 2048];
                assert!(stream.read(&mut buf).unwrap() > 0);
                arrived_tx.send(()).unwrap();
                release_rx.recv_timeout(std::time::Duration::from_secs(5)).unwrap();
                stream.write_all(b"HTTP/1.1 401 Unauthorized\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").unwrap();
            }
        });
        let worker = tokio::spawn({
            let store = Arc::clone(&store); let client = Arc::clone(&client);
            async move {
                let mut report = SyncReport::default();
                fetch(&store, &client, &mut report, "journal", &date(Local::now().date_naive()), None, 1, Utc::now()).await;
                report
            }
        });
        arrived_rx.recv_timeout(std::time::Duration::from_secs(5)).unwrap();
        settings.journal_enabled = false;
        store.set_settings(&settings).unwrap();
        release_tx.send(()).unwrap();
        let report = tokio::time::timeout(std::time::Duration::from_secs(5), worker).await.unwrap().unwrap();
        server.join().unwrap();
        assert_eq!(report.requests, 1);
        assert_eq!(report.data_failures, 0);
        assert!(!report.auth_failed);
        assert_eq!(report.stored, 0);
        listener.set_nonblocking(true).unwrap();
        assert!(listener.accept().is_err(), "neither refresh nor retry may dispatch after opt-out");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn opt_out_during_reply_discards_it_across_store_connections() {
        use std::{io::{Read, Write}, net::TcpListener, sync::{Arc, mpsc}};
        let path = std::env::temp_dir().join(format!("raring-journal-race-{}-{}.sqlite", std::process::id(), Utc::now().timestamp_nanos_opt().unwrap()));
        let store = Arc::new(Store::open(&path).unwrap());
        let other = Store::open(&path).unwrap();
        let now = Utc::now();
        let today = Local::now().date_naive();
        for i in 0..2 {
            store.put("naps", &date(today - Duration::days(i)), &now.to_rfc3339(), &serde_json::json!([])).unwrap();
        }
        let mut settings = other.settings();
        settings.journal_enabled = true;
        other.set_settings(&settings).unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let client = Arc::new(Client::with_urls(Tokens {
            access_token: "synthetic".into(), refresh_token: "synthetic".into(),
            client_id: "synthetic".into(), expires_at: now + Duration::days(1),
        }, &url, &url, std::time::Duration::ZERO).unwrap());
        let (arrived_tx, arrived_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let server = std::thread::spawn({
            let listener = listener.try_clone().unwrap();
            move || {
                let (mut stream, _) = listener.accept().unwrap();
                stream.set_read_timeout(Some(std::time::Duration::from_secs(5))).unwrap();
                let mut buf = [0; 2048];
                let size = stream.read(&mut buf).unwrap();
                arrived_tx.send(String::from_utf8_lossy(&buf[..size]).lines().next().unwrap().to_owned()).unwrap();
                release_rx.recv_timeout(std::time::Duration::from_secs(5)).unwrap();
                let body = r#"{"dailyLogsReport":[]}"#;
                write!(stream, "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).unwrap();
            }
        });
        let worker = tokio::spawn({
            let store = Arc::clone(&store); let client = Arc::clone(&client);
            async move {
                let mut report = SyncReport::default();
                sync_naps_journal(&store, &client, &mut report, today, 20, now).await;
                report
            }
        });
        assert!(arrived_rx.recv_timeout(std::time::Duration::from_secs(5)).unwrap().contains("/lifestylelogging-service/dailyLog/"));
        settings.journal_enabled = false;
        other.set_settings(&settings).unwrap();
        release_tx.send(()).unwrap();
        let report = worker.await.unwrap();
        server.join().unwrap();
        assert_eq!(report.requests, 1);
        assert_eq!(report.stored, 0);
        assert_eq!(report.data_failures, 0);
        assert!(report.error.is_none());
        assert_eq!(store.get("journal", &date(today)), None);
        assert_eq!(store.fetched_at("journal", &date(today)), None);
        assert_eq!(store.cursor(&format!("retry:journal:{}", date(today))), None);
        listener.set_nonblocking(true).unwrap();
        assert!(listener.accept().is_err(), "opt-out must stop remaining journal requests");

        settings.journal_enabled = true;
        other.set_settings(&settings).unwrap();
        let listener2 = listener.try_clone().unwrap();
        let next = std::thread::spawn(move || {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
            for _ in 0..2 {
                let (mut stream, _) = loop {
                    assert!(std::time::Instant::now() < deadline, "re-enabled journal did not fetch");
                    match listener2.accept() {
                        Ok(stream) => break stream,
                        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => std::thread::yield_now(),
                        Err(e) => panic!("{e}"),
                    }
                };
                let mut buf = [0; 2048];
                assert!(stream.read(&mut buf).unwrap() > 0);
                let body = r#"{"dailyLogsReport":[]}"#;
                write!(stream, "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).unwrap();
            }
        });
        let mut report = SyncReport::default();
        sync_naps_journal(&store, &client, &mut report, today, 2, now).await;
        next.join().unwrap();
        assert_eq!(report.requests, 2);
        assert_eq!(report.stored, 2);
        assert!(store.fetched_at("journal", &date(today)).is_some());
        assert!(store.fetched_at("journal", &date(today - Duration::days(1))).is_some());
        drop(client); drop(store); drop(other); drop(listener);
        let _ = std::fs::remove_file(&path);
        for extension in ["-wal", "-shm"] { let _ = std::fs::remove_file(format!("{}{}", path.display(), extension)); }
    }
}

/// One explicitly opened older day. Cached empty days stay cached; transient
/// failures use the same retry cursor, budget and 429 backoff as normal sync.
pub async fn heart_rate_on_demand(store: &Store, client: &Client, key: &str) -> SyncReport {
    let mut report = SyncReport::default();
    let now = Utc::now();
    let valid = NaiveDate::parse_from_str(key, "%Y-%m-%d")
        .ok()
        .is_some_and(|d| d.to_string() == key && d <= Local::now().date_naive());
    if !valid || store.get("daily_hr", key).is_some() {
        return report;
    }
    if store
        .cursor("backoff_until")
        .and_then(|v| v.parse::<i64>().ok())
        .is_some_and(|t| t > now.timestamp())
    {
        report.rate_limited = true;
        return report;
    }
    let name = store
        .get("profile", "me")
        .and_then(|v| v["displayName"].as_str().map(str::to_owned));
    if let Some(name) = name {
        fetch(
            store,
            client,
            &mut report,
            "daily_hr",
            key,
            Some(&name),
            1,
            now,
        )
        .await;
    }
    report
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
    Series {
        name: "weight",
        kind: "weight_range",
        span: 365,
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
    if name == "summary" || name == "activities" || name == "weight" {
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
/// Days whose activities the newest-first scan listed completely, as
/// `listed:lo`/`listed:hi`. A new run extends the span when it reaches back
/// into it; otherwise only its own span counts.
fn mark_listed(store: &Store, from: NaiveDate, to: NaiveDate) {
    let span = |k: &str| store.cursor(k).as_deref().and_then(day);
    let lo = match (span("listed:lo"), span("listed:hi")) {
        (Some(lo), Some(hi)) if from <= hi + Duration::days(1) => lo.min(from),
        _ => from,
    };
    let _ = store.set_cursor("listed:lo", &date(lo));
    let _ = store.set_cursor("listed:hi", &date(to));
}
/// Whether every day from `from` to `to` has a complete recent activity list.
/// Legacy history cursors alone cannot distinguish absence from failed reads.
pub fn activities_listed(store: &Store, from: NaiveDate, to: NaiveDate) -> bool {
    let span = |lo: &str, hi: &str| {
        Some((
            store.cursor(lo).as_deref().and_then(day)?,
            store.cursor(hi).as_deref().and_then(day)?,
        ))
    };
    let spans: Vec<(NaiveDate, NaiveDate)> = [span("listed:lo", "listed:hi")]
        .into_iter()
        .flatten()
        .collect();
    from.iter_days()
        .take_while(|d| *d <= to)
        .all(|d| spans.iter().any(|(lo, hi)| (*lo..=*hi).contains(&d)))
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
        "weight_range" => value["dateWeightList"].as_array(),
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
        "weight_range" => "weight",
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
        } else if kind == "weight_range" {
            // More than one weigh-in on a day: show the last by Garmin's timestamp.
            entries
                .into_iter()
                .max_by_key(|v| v["timestampGMT"].as_i64().unwrap_or(0))
                .unwrap_or(Value::Null)
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
            if series.kind == "weight_range" && !value.is_null() && !valid_weight_range(&value) {
                report.data_failures += 1;
                break;
            }
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

#[cfg(test)]
mod threshold_tests {
    use super::*;
    #[test]
    fn empty_and_not_found_checks_wait_a_day() {
        let store = Store::open(":memory:").unwrap();
        let checked = "acclimation_checked";
        assert!(threshold_due(&store, checked, 100_000));
        store.set_cursor(checked, "100000").unwrap();
        assert!(!threshold_due(&store, checked, 100_001));
        assert!(!threshold_due(&store, checked, 186_399));
        assert!(threshold_due(&store, checked, 186_400));
    }
}

#[cfg(test)]
mod planned_tests;
