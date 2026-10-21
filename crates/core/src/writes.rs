//! Garmin writes (docs/spec.md, Rules: Garmin writes): a fresh read, only the
//! fields the person changed (or one weigh-in created or deleted), one paced
//! write, a read-back of what was sent, and an entry in the local change log.
//! Nothing here runs from sync.
use crate::{
    garmin::{self, Client, Reply},
    store::Store,
    view::{ChangedField, GarminChange, WeightUnit, WriteOutcome},
    weight, Error, Result,
};
use chrono::{DateTime, Duration, FixedOffset, NaiveDate, NaiveDateTime, Timelike, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// The change-log schema this build reads and writes.
pub const SCHEMA: u64 = 1;
/// A versioned settings row, so export, account switches and sign-out cover it.
pub(crate) const KEY: &str = "garmin:changes";
/// Entries kept, newest first.
pub const LOG_LIMIT: usize = 50;

/// Proof that a write comes from the person's explicit action. Only this
/// module creates one, so background sync cannot reach `Client::send`.
pub(crate) struct Explicit(());

/// Two saves must not interleave their fresh reads and writes.
static WRITES: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

/// One change to a watch alarm, with the value the person saw so a change
/// made elsewhere in the meantime is never overwritten.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", tag = "edit")]
pub enum AlarmEdit {
    /// Minutes after midnight, as Garmin stores them.
    #[serde(rename_all = "camelCase")]
    Time { alarm_id: u64, from: u32, to: u32 },
    #[serde(rename_all = "camelCase")]
    On { alarm_id: u64, from: bool, to: bool },
    /// Garmin's day keys, such as `MONDAY` or `ONCE`.
    #[serde(rename_all = "camelCase")]
    Days { alarm_id: u64, from: Vec<String>, to: Vec<String> },
    /// A delete: refused unless the person confirmed it.
    #[serde(rename_all = "camelCase")]
    Remove { alarm_id: u64 },
}

/// A step into a Garmin object: a key, or a list item found by its id, so a
/// reordered list cannot redirect an edit.
#[derive(Clone, Debug)]
pub(crate) enum Step {
    Key(&'static str),
    Item(&'static str, u64),
}

pub(crate) struct Edit {
    path: Vec<Step>,
    field: String,
    label: &'static str,
    /// What the person saw; `None` skips the check (a delete of an item that exists).
    seen: Option<Value>,
    /// `None` removes the item or key.
    to: Option<Value>,
    text: fn(&Value) -> String,
}

pub(crate) struct Resource {
    name: &'static str,
    path: String,
    /// Top-level keys sent besides the edited ones; `None` sends the whole object.
    keep: Option<&'static [&'static str]>,
    /// The watch that downloads the change, for "waiting for watch sync".
    device: Option<String>,
    other_label: &'static str,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Field {
    #[serde(flatten)]
    shown: ChangedField,
    #[serde(default)]
    before_value: Option<Value>,
    #[serde(default)]
    after_value: Option<Value>,
}

/// One logged write. Values are what the person chose; the text is for display.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Entry {
    at: String,
    resource: String,
    path: String,
    fields: Vec<Field>,
    outcome: WriteOutcome,
    #[serde(default)]
    message: Option<String>,
    #[serde(default)]
    mismatches: Vec<ChangedField>,
    #[serde(default)]
    device: Option<String>,
    /// The watch's `lastDownloadTimestamp` read right after the write.
    #[serde(default)]
    download_baseline: Option<i64>,
}

fn hhmm(v: &Value) -> String {
    v.as_u64()
        .filter(|m| *m < 1440)
        .map(|m| format!("{:02}:{:02}", m / 60, m % 60))
        .unwrap_or_else(|| "unknown".into())
}
fn on_off(v: &Value) -> String {
    match v.as_str() {
        Some("ON") => "On".into(),
        Some("OFF") => "Off".into(),
        Some(s) => title(s),
        None => "unknown".into(),
    }
}
fn title(s: &str) -> String {
    s.split('_')
        .map(|w| {
            let w = w.to_lowercase();
            let mut c = w.chars();
            c.next().map(|f| f.to_uppercase().chain(c).collect()).unwrap_or_default()
        })
        .collect::<Vec<String>>()
        .join(" ")
}
fn days(v: &Value) -> String {
    v.as_array()
        .map(|d| d.iter().filter_map(Value::as_str).map(title).collect::<Vec<_>>().join(", "))
        .unwrap_or_else(|| "unknown".into())
}
fn alarm_summary(v: &Value) -> String {
    hhmm(&v["alarmTime"])
}

/// The device settings write for alarms, validated before any request.
pub(crate) fn alarms(device: &str, edits: &[AlarmEdit], confirm_delete: bool) -> Result<(Resource, Vec<Edit>)> {
    if device.is_empty() || device.len() > 20 || !device.bytes().all(|b| b.is_ascii_digit()) {
        return Err(Error::InvalidLocal("device"));
    }
    if edits.is_empty() {
        return Err(Error::InvalidLocal("no change"));
    }
    let mut out: Vec<Edit> = vec![];
    for edit in edits {
        let item = |id: u64| vec![Step::Key("alarms"), Step::Item("alarmId", id)];
        let field = |id: u64, key: &'static str, label, seen: Value, to: Value, text| {
            let mut path = item(id);
            path.push(Step::Key(key));
            Edit { path, field: format!("alarm:{id}.{key}"), label, seen: Some(seen), to: Some(to), text }
        };
        let e = match edit {
            AlarmEdit::Time { alarm_id, from, to } => {
                if *from >= 1440 || *to >= 1440 || from == to {
                    return Err(Error::InvalidLocal("alarm time"));
                }
                field(*alarm_id, "alarmTime", "Alarm time", (*from).into(), (*to).into(), hhmm as fn(&Value) -> String)
            }
            AlarmEdit::On { alarm_id, from, to } => {
                if from == to {
                    return Err(Error::InvalidLocal("alarm on"));
                }
                let mode = |on: bool| Value::from(if on { "ON" } else { "OFF" });
                field(*alarm_id, "alarmMode", "Alarm", mode(*from), mode(*to), on_off)
            }
            AlarmEdit::Days { alarm_id, from, to } => {
                let key = |d: &String| !d.is_empty() && d.len() <= 16 && d.bytes().all(|b| b.is_ascii_uppercase() || b == b'_');
                let set = |v: &Vec<String>| { let mut s = v.clone(); s.sort(); s.dedup(); s };
                if to.is_empty() || !to.iter().all(key) || !from.iter().all(key) || set(from) == set(to) {
                    return Err(Error::InvalidLocal("alarm days"));
                }
                let mut chosen: Vec<String> = vec![];
                for d in to {
                    if !chosen.contains(d) {
                        chosen.push(d.clone());
                    }
                }
                field(*alarm_id, "alarmDays", "Alarm days", from.clone().into(), chosen.into(), days)
            }
            AlarmEdit::Remove { alarm_id } => {
                if !confirm_delete {
                    return Err(Error::InvalidLocal("confirm the delete first"));
                }
                Edit { path: item(*alarm_id), field: format!("alarm:{alarm_id}"), label: "Alarm removed", seen: None, to: None, text: alarm_summary }
            }
        };
        // One change per field, and a removed alarm takes no other change.
        let item_of = |f: &str| f.split('.').next().unwrap_or(f).to_owned();
        if out.iter().any(|o| o.field == e.field || ((o.to.is_none() || e.to.is_none()) && item_of(&o.field) == item_of(&e.field))) {
            return Err(Error::InvalidLocal("one change per alarm field"));
        }
        out.push(e);
    }
    Ok((
        Resource {
            name: "Watch alarms",
            path: format!("/device-service/deviceservice/device-info/settings/{device}"),
            keep: Some(&["deviceId"]),
            device: Some(device.to_owned()),
            other_label: "Other alarm settings",
        },
        out,
    ))
}

fn find<'a>(mut v: &'a Value, path: &[Step]) -> Option<&'a Value> {
    for step in path {
        v = match step {
            Step::Key(k) => v.as_object()?.get(*k)?,
            Step::Item(k, id) => v.as_array()?.iter().find(|i| i[*k].as_u64() == Some(*id))?,
        };
    }
    Some(v)
}
fn find_mut<'a>(mut v: &'a mut Value, path: &[Step]) -> Option<&'a mut Value> {
    for step in path {
        v = match step {
            Step::Key(k) => v.as_object_mut()?.get_mut(*k)?,
            Step::Item(k, id) => v.as_array_mut()?.iter_mut().find(|i| i[*k].as_u64() == Some(*id))?,
        };
    }
    Some(v)
}
/// Changes one existing value or removes it; false if it isn't there.
fn set(v: &mut Value, path: &[Step], to: Option<Value>) -> bool {
    let Some((last, parent)) = path.split_last() else { return false };
    let Some(parent) = find_mut(v, parent) else { return false };
    match (last, to) {
        (Step::Key(k), Some(to)) => parent.as_object_mut().and_then(|m| m.get_mut(*k)).map(|slot| *slot = to).is_some(),
        (Step::Key(k), None) => parent.as_object_mut().and_then(|m| m.remove(*k)).is_some(),
        (Step::Item(k, id), Some(to)) => parent
            .as_array_mut()
            .and_then(|a| a.iter_mut().find(|i| i[*k].as_u64() == Some(*id)))
            .map(|slot| *slot = to)
            .is_some(),
        (Step::Item(k, id), None) => parent.as_array_mut().is_some_and(|a| {
            let n = a.len();
            a.retain(|i| i[*k].as_u64() != Some(*id));
            a.len() < n
        }),
    }
}

/// Garmin may reorder lists and turn integers into decimals on the way back.
fn norm(v: &Value) -> Value {
    match v {
        Value::Number(n) => n.as_f64().map(Value::from).unwrap_or_else(|| v.clone()),
        Value::Object(m) => Value::Object(m.iter().map(|(k, v)| (k.clone(), norm(v))).collect()),
        Value::Array(a) => {
            let mut items: Vec<Value> = a.iter().map(norm).collect();
            items.sort_by_cached_key(|i| i.to_string());
            Value::Array(items)
        }
        _ => v.clone(),
    }
}
fn same(a: Option<&Value>, b: Option<&Value>) -> bool {
    a.map(norm) == b.map(norm)
}

fn rate_limited(store: &Store, retry_after: Option<u64>, now: DateTime<Utc>) -> String {
    let _ = store.set_cursor("backoff_until", &(now.timestamp() + retry_after.unwrap_or(300) as i64).to_string());
    "Garmin is limiting requests; try again later".into()
}
/// A failure before Garmin accepted anything: nothing was saved.
fn not_saved(store: &Store, error: garmin::Error, now: DateTime<Utc>) -> (WriteOutcome, Option<String>) {
    let message = match error {
        garmin::Error::NeedsSignIn => return (WriteOutcome::NeedsSignIn, None),
        garmin::Error::RateLimited { retry_after } => rate_limited(store, retry_after, now),
        garmin::Error::NotFound => "Garmin didn't find these settings".into(),
        garmin::Error::Http(code) => format!("Garmin answered with error {code}"),
        garmin::Error::Decode => "Garmin sent an unexpected response".into(),
        garmin::Error::Network => "Couldn't reach Garmin".into(),
    };
    (WriteOutcome::Failed, Some(message))
}

/// The whole write. Every outcome, including refusals, becomes a log entry.
pub(crate) async fn write(store: &Store, client: &Client, res: Resource, edits: Vec<Edit>, now: DateTime<Utc>) -> Entry {
    let _one = WRITES.lock().await;
    let mut entry = Entry {
        at: now.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        resource: res.name.into(),
        path: res.path.clone(),
        fields: edits
            .iter()
            .map(|e| Field {
                shown: ChangedField {
                    field: e.field.clone(),
                    label: e.label.into(),
                    before: e.seen.as_ref().map(e.text),
                    after: e.to.as_ref().map(e.text),
                },
                before_value: e.seen.clone(),
                after_value: e.to.clone(),
            })
            .collect(),
        outcome: WriteOutcome::Failed,
        message: None,
        mismatches: vec![],
        device: res.device.clone(),
        download_baseline: None,
    };
    if backing_off(store, now) {
        entry.message = Some("Garmin is limiting requests; try again later".into());
        return entry;
    }
    let fresh = match client.get(&res.path, &[]).await {
        Ok(v) if v.is_object() && res.device.as_ref().is_none_or(|d| v["deviceId"].as_u64().map(|n| n.to_string()).as_ref() == Some(d)) => v,
        Ok(_) => {
            entry.message = Some("Garmin sent an unexpected response".into());
            return entry;
        }
        Err(e) => {
            (entry.outcome, entry.message) = not_saved(store, e, now);
            return entry;
        }
    };
    let mut patched = fresh.clone();
    for (e, shown) in edits.iter().zip(entry.fields.iter_mut()) {
        let current = find(&fresh, &e.path);
        if e.seen.is_none() {
            shown.shown.before = current.map(e.text);
        }
        let conflict = match (current, &e.seen) {
            (None, _) => Some(format!("Changed in Garmin since you looked: {} is no longer there", e.label.to_lowercase())),
            (Some(now_value), Some(seen)) if !same(Some(now_value), Some(seen)) => {
                Some(format!("Changed in Garmin since you looked: {} is now {}", e.label.to_lowercase(), (e.text)(now_value)))
            }
            _ => None,
        };
        if let Some(message) = conflict {
            entry.outcome = WriteOutcome::Conflict;
            entry.message = Some(message);
            return entry;
        }
        if !set(&mut patched, &e.path, e.to.clone()) {
            entry.message = Some("Garmin sent an unexpected response".into());
            return entry;
        }
    }
    let body = match res.keep {
        None => patched.clone(),
        Some(keep) => {
            let mut keys: Vec<&str> = keep.to_vec();
            keys.extend(edits.iter().filter_map(|e| match e.path.first() {
                Some(Step::Key(k)) => Some(*k),
                _ => None,
            }));
            let body: Map<String, Value> = keys.into_iter().filter_map(|k| Some((k.to_owned(), patched.get(k)?.clone()))).collect();
            Value::Object(body)
        }
    };
    match client.send(&Explicit(()), reqwest::Method::PUT, &res.path, Some(&body)).await {
        Ok(Reply::Accepted) => {}
        Ok(Reply::Rejected { message, .. }) => {
            entry.outcome = WriteOutcome::Rejected;
            entry.message = message;
            return entry;
        }
        // Garmin may have saved it before the connection or server failed.
        Err(e @ (garmin::Error::Network | garmin::Error::Http(500..))) => {
            entry.outcome = WriteOutcome::Unconfirmed;
            entry.message = Some(format!("{}; Garmin may or may not have saved it", not_saved(store, e, now).1.unwrap_or_default()));
            return entry;
        }
        Err(e) => {
            (entry.outcome, entry.message) = not_saved(store, e, now);
            return entry;
        }
    }
    let back = match client.get(&res.path, &[]).await {
        Ok(v) if v.is_object() => v,
        other => {
            entry.outcome = WriteOutcome::Unconfirmed;
            entry.message = Some(match other {
                Err(garmin::Error::RateLimited { retry_after }) => format!("Saved, but {}", rate_limited(store, retry_after, now).to_lowercase()),
                _ => "Saved, but Garmin's read-back failed".into(),
            });
            return entry;
        }
    };
    for e in &edits {
        let got = find(&back, &e.path);
        if !same(got, e.to.as_ref()) {
            entry.mismatches.push(ChangedField {
                field: e.field.clone(),
                label: e.label.into(),
                before: e.to.as_ref().map(e.text),
                after: got.map(e.text),
            });
        }
    }
    if entry.mismatches.is_empty() {
        if let Some(sent) = body.as_object() {
            if sent.iter().any(|(k, v)| !same(Some(v), back.get(k))) {
                entry.mismatches.push(ChangedField { field: "other".into(), label: res.other_label.into(), before: None, after: None });
            }
        }
    }
    entry.outcome = if entry.mismatches.is_empty() { WriteOutcome::Applied } else { WriteOutcome::Differs };
    if let Some(device) = &res.device {
        match client.endpoint("device_detail", device, None).await {
            Ok(detail) if detail["deviceId"].as_u64().is_some_and(|n| n.to_string() == *device) => {
                entry.download_baseline = detail["lastDownloadTimestamp"].as_i64();
            }
            Err(garmin::Error::RateLimited { retry_after }) => {
                rate_limited(store, retry_after, now);
            }
            _ => {}
        }
    }
    entry
}

// Weigh-ins are created and deleted, never edited. The POST answers 204 with
// no id, so the created entry is found again by its source and Garmin's
// timestamp, which echoes the sent `gmtTimestamp` to the millisecond.
const ADD_WEIGH_IN: &str = "/weight-service/user-weight";
const DAY_WEIGH_INS: &str = "/weight-service/weight/dateRange";
const MANUAL: &str = "MANUAL";

/// A new weigh-in, validated before any request.
pub(crate) struct NewWeighIn {
    /// What the person typed, to 0.1 in their unit.
    shown: f64,
    unit: WeightUnit,
    grams: f64,
    at: DateTime<FixedOffset>,
}

pub(crate) fn new_weigh_in(value: f64, unit: WeightUnit, at: &str, now: DateTime<Utc>) -> Result<NewWeighIn> {
    let at = DateTime::parse_from_rfc3339(at)
        .ok()
        .and_then(|t| t.with_nanosecond(0))
        .filter(|t| *t <= now + Duration::minutes(1) && t.date_naive() >= NaiveDate::from_ymd_opt(1990, 1, 1).unwrap())
        .ok_or(Error::InvalidLocal("weigh-in time"))?;
    let shown = (value * 10.).round() / 10.;
    let kg = match unit {
        WeightUnit::Kg => shown,
        WeightUnit::Lb => shown * weight::KG_PER_LB,
    };
    if !value.is_finite() || !(20. ..=300.).contains(&kg) {
        return Err(Error::InvalidLocal("weight"));
    }
    Ok(NewWeighIn { shown, unit, grams: (kg * 1000.).round(), at })
}

/// Which weigh-in to delete: the one the person saw.
pub(crate) struct OldWeighIn {
    date: NaiveDate,
    timestamp_gmt: i64,
    sample_pk: Option<u64>,
}

pub(crate) fn old_weigh_in(date: &str, timestamp_gmt: i64, sample_pk: Option<&str>, confirm: bool) -> Result<OldWeighIn> {
    if !confirm {
        return Err(Error::InvalidLocal("confirm the delete first"));
    }
    let date = NaiveDate::parse_from_str(date, "%Y-%m-%d").map_err(|_| Error::InvalidLocal("weigh-in date"))?;
    let sample_pk = sample_pk
        .map(|s| s.parse::<u64>().ok().filter(|n| *n > 0 && s.bytes().all(|b| b.is_ascii_digit())).ok_or(Error::InvalidLocal("weigh-in id")))
        .transpose()?;
    if timestamp_gmt <= 0 {
        return Err(Error::InvalidLocal("weigh-in time"));
    }
    Ok(OldWeighIn { date, timestamp_gmt, sample_pk })
}

/// A day's weigh-ins as Garmin last answered, for the local copy.
pub(crate) struct Weighed {
    date: String,
    list: Vec<Value>,
    /// The deleted weigh-in's timestamp, so the stored latest can follow.
    deleted: Option<i64>,
}

fn grams(v: &Value) -> Option<f64> {
    v["weight"].as_f64().filter(|w| w.is_finite() && *w > 0.)
}
fn in_unit(grams: f64, unit: WeightUnit) -> f64 {
    match unit {
        WeightUnit::Kg => grams / 1000.,
        WeightUnit::Lb => grams / 1000. / weight::KG_PER_LB,
    }
}
/// The same reading as shown, to 0.1 in the person's unit.
fn same_reading(a: f64, b: f64, unit: WeightUnit) -> bool {
    (in_unit(a, unit) * 10.).round() == (in_unit(b, unit) * 10.).round()
}
fn unit_label(unit: WeightUnit) -> &'static str {
    match unit {
        WeightUnit::Kg => "kg",
        WeightUnit::Lb => "lb",
    }
}
fn reading(grams: f64, unit: WeightUnit) -> String {
    format!("{:.1} {}", in_unit(grams, unit), unit_label(unit))
}
/// Garmin's `date` is the local wall time written as if it were UTC.
fn local_time(v: &Value) -> Option<NaiveDateTime> {
    v["date"].as_i64().and_then(DateTime::from_timestamp_millis).map(|t| t.naive_utc())
}
fn weigh_in_text(v: &Value, unit: WeightUnit) -> String {
    let value = grams(v).map(|g| reading(g, unit)).unwrap_or_else(|| "unknown".into());
    match local_time(v) {
        Some(t) => format!("{value} at {}", t.format("%H:%M on %-d %b")),
        None => value,
    }
}
fn timestamp(t: NaiveDateTime) -> String {
    t.format("%Y-%m-%dT%H:%M:%S%.3f").to_string()
}

/// One day's weigh-ins; `None` if the answer isn't Garmin's usual shape.
async fn day_weigh_ins(client: &Client, date: &str) -> garmin::Result<Option<Vec<Value>>> {
    let v = client.get(DAY_WEIGH_INS, &[("startDate", date), ("endDate", date)]).await?;
    let Some(list) = v["dateWeightList"].as_array() else { return Ok(None) };
    let valid = list.iter().all(|e| e["calendarDate"].as_str().is_some() && grams(e).is_some() && e["timestampGMT"].as_i64().is_some());
    Ok(valid.then(|| list.iter().filter(|e| e["calendarDate"].as_str() == Some(date)).cloned().collect()))
}

fn backing_off(store: &Store, now: DateTime<Utc>) -> bool {
    store.cursor("backoff_until").and_then(|s| s.parse::<i64>().ok()).is_some_and(|t| t > now.timestamp())
}
fn weight_entry(now: DateTime<Utc>, path: &str, field: ChangedField, after_value: Option<Value>) -> Entry {
    Entry {
        at: now.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        resource: "Weight".into(),
        path: path.into(),
        fields: vec![Field { shown: field, before_value: None, after_value }],
        outcome: WriteOutcome::Failed,
        message: None,
        mismatches: vec![],
        device: None,
        download_baseline: None,
    }
}
/// The outcome when Garmin may have acted on a sent request.
fn sent_unconfirmed(store: &Store, error: garmin::Error, now: DateTime<Utc>) -> (WriteOutcome, Option<String>) {
    match error {
        e @ (garmin::Error::Network | garmin::Error::Http(500..)) => (
            WriteOutcome::Unconfirmed,
            Some(format!("{}; Garmin may or may not have saved it", not_saved(store, e, now).1.unwrap_or_default())),
        ),
        e => not_saved(store, e, now),
    }
}
fn read_back_failed(store: &Store, error: Option<garmin::Error>, now: DateTime<Utc>, what: &str) -> String {
    match error {
        Some(garmin::Error::RateLimited { retry_after }) => format!("{what}, but {}", rate_limited(store, retry_after, now).to_lowercase()),
        _ => format!("{what}, but Garmin's read-back failed"),
    }
}

/// Adds one weigh-in. Never resent automatically: a retry is the person's,
/// and its fresh read finds an earlier attempt at the same time first.
pub(crate) async fn create_weigh_in(store: &Store, client: &Client, w: NewWeighIn, now: DateTime<Utc>) -> (Entry, Option<Weighed>) {
    let _one = WRITES.lock().await;
    let date = w.at.date_naive().to_string();
    let sent_at = w.at.timestamp_millis();
    let body = serde_json::json!({
        "date": date,
        "dateTimestamp": timestamp(w.at.naive_local()),
        "gmtTimestamp": timestamp(w.at.naive_utc()),
        "unitKey": "kg",
        "sourceType": MANUAL,
        "value": w.grams / 1000.,
    });
    let shown = format!("{:.1} {} at {}", w.shown, unit_label(w.unit), w.at.naive_local().format("%H:%M on %-d %b"));
    let field = ChangedField { field: format!("weighIn:{sent_at}"), label: "Weigh-in added".into(), before: None, after: Some(shown) };
    let mut entry = weight_entry(now, ADD_WEIGH_IN, field, Some(body.clone()));
    if backing_off(store, now) {
        entry.message = Some("Garmin is limiting requests; try again later".into());
        return (entry, None);
    }
    let fresh = match day_weigh_ins(client, &date).await {
        Ok(Some(list)) => list,
        Ok(None) => {
            entry.message = Some("Garmin sent an unexpected response".into());
            return (entry, None);
        }
        Err(e) => {
            (entry.outcome, entry.message) = not_saved(store, e, now);
            return (entry, None);
        }
    };
    let same_time = fresh.iter().find(|e| e["timestampGMT"].as_i64() == Some(sent_at));
    let same_value = fresh.iter().find(|e| grams(e).is_some_and(|g| same_reading(g, w.grams, w.unit)));
    let weighed = |list: Vec<Value>| Some(Weighed { date: date.clone(), list, deleted: None });
    match (same_time, same_value) {
        (Some(e), _) if e["sourceType"] == MANUAL && grams(e).is_some_and(|g| same_reading(g, w.grams, w.unit)) => {
            entry.outcome = WriteOutcome::Applied;
            entry.message = Some("Already in Garmin from an earlier attempt; not sent again".into());
            return (entry, weighed(fresh));
        }
        // The read-back couldn't tell two weigh-ins at the same moment apart.
        (Some(e), _) => {
            entry.outcome = WriteOutcome::Conflict;
            entry.message = Some(format!("Garmin already has a weigh-in at that time: {}", weigh_in_text(e, w.unit)));
            return (entry, weighed(fresh));
        }
        (None, Some(e)) => {
            entry.outcome = WriteOutcome::Duplicate;
            entry.message = Some(format!("Garmin already has {} that day", weigh_in_text(e, w.unit)));
            return (entry, weighed(fresh));
        }
        (None, None) => {}
    }
    match client.send(&Explicit(()), reqwest::Method::POST, ADD_WEIGH_IN, Some(&body)).await {
        Ok(Reply::Accepted) => {}
        Ok(Reply::Rejected { message, .. }) => {
            entry.outcome = WriteOutcome::Rejected;
            entry.message = message;
            return (entry, weighed(fresh));
        }
        Err(e) => {
            (entry.outcome, entry.message) = sent_unconfirmed(store, e, now);
            let kept = (entry.outcome != WriteOutcome::Unconfirmed).then_some(fresh).and_then(weighed);
            return (entry, kept);
        }
    }
    let back = match day_weigh_ins(client, &date).await {
        Ok(Some(list)) => list,
        other => {
            entry.outcome = WriteOutcome::Unconfirmed;
            entry.message = Some(read_back_failed(store, other.err(), now, "Saved"));
            return (entry, None);
        }
    };
    let created: Vec<&Value> = back.iter().filter(|e| e["sourceType"] == MANUAL && e["timestampGMT"].as_i64() == Some(sent_at)).collect();
    match created.as_slice() {
        [one] => {
            let got = grams(one).unwrap_or_default();
            if same_reading(got, w.grams, w.unit) {
                entry.outcome = WriteOutcome::Applied;
            } else {
                entry.outcome = WriteOutcome::Differs;
                entry.mismatches.push(ChangedField {
                    field: "weight".into(),
                    label: "Weight".into(),
                    before: Some(reading(w.grams, w.unit)),
                    after: Some(reading(got, w.unit)),
                });
            }
        }
        _ => {
            entry.outcome = WriteOutcome::Unconfirmed;
            entry.message = Some("Garmin accepted it, but its weigh-ins don't show it yet".into());
        }
    }
    (entry, weighed(back))
}

/// Deletes the weigh-in the person saw, found again in a fresh read.
pub(crate) async fn delete_weigh_in(store: &Store, client: &Client, old: OldWeighIn, now: DateTime<Utc>) -> (Entry, Option<Weighed>) {
    let _one = WRITES.lock().await;
    let unit = weight::unit(store).unwrap_or(WeightUnit::Kg);
    let date = old.date.to_string();
    let stored = store.get("weight", &date).filter(|v| v["timestampGMT"].as_i64() == Some(old.timestamp_gmt));
    let field = ChangedField {
        field: format!("weighIn:{}", old.timestamp_gmt),
        label: "Weigh-in deleted".into(),
        before: stored.as_ref().map(|v| weigh_in_text(v, unit)),
        after: None,
    };
    let mut entry = weight_entry(now, DAY_WEIGH_INS, field, None);
    entry.fields[0].before_value = stored.as_ref().and_then(|v| v["weight"].as_f64()).map(Value::from);
    if backing_off(store, now) {
        entry.message = Some("Garmin is limiting requests; try again later".into());
        return (entry, None);
    }
    let fresh = match day_weigh_ins(client, &date).await {
        Ok(Some(list)) => list,
        Ok(None) => {
            entry.message = Some("Garmin sent an unexpected response".into());
            return (entry, None);
        }
        Err(e) => {
            (entry.outcome, entry.message) = not_saved(store, e, now);
            return (entry, None);
        }
    };
    let deleted = Some(old.timestamp_gmt);
    let weighed = |list: Vec<Value>| Some(Weighed { date: date.clone(), list, deleted });
    let Some(target) = fresh
        .iter()
        .find(|e| e["timestampGMT"].as_i64() == Some(old.timestamp_gmt) && old.sample_pk.is_none_or(|pk| e["samplePk"].as_u64() == Some(pk)))
        .cloned()
    else {
        entry.outcome = WriteOutcome::Conflict;
        entry.message = Some("Already gone from Garmin".into());
        return (entry, weighed(fresh));
    };
    entry.fields[0].shown.before = Some(weigh_in_text(&target, unit));
    let (Some(pk), Some(day)) = (target["samplePk"].as_u64(), target["calendarDate"].as_str()) else {
        entry.message = Some("Garmin sent an unexpected response".into());
        return (entry, None);
    };
    entry.path = format!("/weight-service/weight/{day}/byversion/{pk}");
    match client.send(&Explicit(()), reqwest::Method::DELETE, &entry.path, None).await {
        Ok(Reply::Accepted) => {}
        Ok(Reply::Rejected { message, .. }) => {
            entry.outcome = WriteOutcome::Rejected;
            entry.message = message;
            return (entry, None);
        }
        Err(e) => {
            (entry.outcome, entry.message) = sent_unconfirmed(store, e, now);
            return (entry, None);
        }
    }
    let back = match day_weigh_ins(client, &date).await {
        Ok(Some(list)) => list,
        other => {
            entry.outcome = WriteOutcome::Unconfirmed;
            entry.message = Some(read_back_failed(store, other.err(), now, "Garmin accepted the delete"));
            return (entry, None);
        }
    };
    if let Some(left) = back.iter().find(|e| e["samplePk"].as_u64() == Some(pk)) {
        entry.outcome = WriteOutcome::Differs;
        entry.mismatches.push(ChangedField { field: entry.fields[0].shown.field.clone(), label: "Weigh-in".into(), before: None, after: Some(weigh_in_text(left, unit)) });
        return (entry, Some(Weighed { date, list: back, deleted: None }));
    }
    entry.outcome = WriteOutcome::Applied;
    (entry, weighed(back))
}

/// Keeps the local copy of a day Garmin just answered for, as sync stores it:
/// the day's last weigh-in by Garmin's timestamp, or nothing.
pub(crate) fn keep_day(store: &Store, w: &Weighed, at: &str) -> rusqlite::Result<()> {
    match w.list.iter().max_by_key(|e| e["timestampGMT"].as_i64().unwrap_or(0)) {
        Some(row) => store.put("weight", &w.date, at, row)?,
        None => store.remove("weight", &w.date)?,
    }
    let latest_gone = w.deleted.is_some_and(|ts| store.get("weight_latest", "me").is_some_and(|v| v["timestampGMT"].as_i64() == Some(ts)));
    if latest_gone {
        // The next sync reads Garmin's latest again.
        store.remove("weight_latest", "me")?;
        store.set_cursor("weight_latest_checked", "0")?;
    }
    Ok(())
}

/// Entries that parse, and whether this build may rewrite the row.
fn read(raw: Option<String>) -> (Vec<Entry>, bool) {
    let Some(raw) = raw else { return (vec![], true) };
    let Ok(value) = serde_json::from_str::<Value>(&raw) else { return (vec![], false) };
    let entries = value["entries"]
        .as_array()
        .map(|a| a.iter().filter_map(|e| serde_json::from_value(e.clone()).ok()).collect::<Vec<Entry>>())
        .unwrap_or_default();
    let writable = value["schema"].as_u64() == Some(SCHEMA) && value["entries"].as_array().is_some_and(|a| a.len() == entries.len());
    (entries, writable)
}

/// Adds an entry, newest first; false if a newer or unreadable row must stay as it is.
pub(crate) fn record(store: &Store, entry: &Entry) -> Result<bool> {
    store.settings_tx(|tx| {
        let (mut entries, writable) = read(tx.get(KEY)?);
        if !writable {
            return Ok(false);
        }
        entries.insert(0, entry.clone());
        entries.truncate(LOG_LIMIT);
        let row = serde_json::json!({"schema": SCHEMA, "entries": entries});
        tx.set(KEY, &row.to_string())?;
        Ok::<_, Error>(true)
    })
}

/// The watch has downloaded since the write when the stored detail of the
/// same watch has a later `lastDownloadTimestamp` than the one read after it.
pub(crate) fn shown(store: &Store, entry: &Entry) -> GarminChange {
    let detail = store.get("device_detail", "selected").unwrap_or(Value::Null);
    let downloaded = entry.device.as_ref().and_then(|d| {
        (detail["deviceId"].as_u64().map(|n| n.to_string()).as_ref() == Some(d)).then(|| detail["lastDownloadTimestamp"].as_i64()).flatten()
    });
    let written = DateTime::parse_from_rfc3339(&entry.at).map(|t| t.timestamp_millis()).unwrap_or(i64::MAX);
    let threshold = entry.download_baseline.unwrap_or(written);
    GarminChange {
        at: entry.at.clone(),
        resource: entry.resource.clone(),
        fields: entry.fields.iter().map(|f| f.shown.clone()).collect(),
        outcome: entry.outcome,
        message: entry.message.clone(),
        mismatches: entry.mismatches.clone(),
        waiting_for_watch: entry.device.is_some()
            && matches!(entry.outcome, WriteOutcome::Applied | WriteOutcome::Differs)
            && !downloaded.is_some_and(|t| t > threshold),
    }
}

pub(crate) fn changes(store: &Store) -> Vec<GarminChange> {
    read(store.setting(KEY)).0.iter().map(|e| shown(store, e)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(at: &str) -> Entry {
        Entry {
            at: at.into(),
            resource: "Watch alarms".into(),
            path: "/x".into(),
            fields: vec![],
            outcome: WriteOutcome::Applied,
            message: None,
            mismatches: vec![],
            device: None,
            download_baseline: None,
        }
    }

    #[test]
    fn log_is_newest_first_and_bounded() {
        let store = Store::open(":memory:").unwrap();
        for i in 0..LOG_LIMIT + 5 {
            assert!(record(&store, &entry(&format!("2026-10-19T10:{:02}:00Z", i))).unwrap());
        }
        let shown = changes(&store);
        assert_eq!(shown.len(), LOG_LIMIT);
        assert_eq!(shown[0].at, format!("2026-10-19T10:{:02}:00Z", LOG_LIMIT + 4));
        assert_eq!(shown.last().unwrap().at, "2026-10-19T10:05:00Z");
        let row: Value = serde_json::from_str(&store.setting(KEY).unwrap()).unwrap();
        assert_eq!(row["schema"], SCHEMA);
    }

    #[test]
    fn a_newer_or_unreadable_row_is_shown_as_far_as_it_parses_and_never_rewritten() {
        let store = Store::open(":memory:").unwrap();
        let newer = serde_json::json!({"schema": 2, "entries": [serde_json::to_value(entry("2026-10-19T10:00:00Z")).unwrap(), {"outcome": "somethingNew"}]}).to_string();
        store.set_setting(KEY, &newer).unwrap();
        assert!(!record(&store, &entry("2026-10-19T11:00:00Z")).unwrap());
        assert_eq!(store.setting(KEY).unwrap(), newer);
        assert_eq!(changes(&store).len(), 1);
        store.set_setting(KEY, "not json").unwrap();
        assert!(!record(&store, &entry("2026-10-19T11:00:00Z")).unwrap());
        assert_eq!(store.setting(KEY).unwrap(), "not json");
        assert!(changes(&store).is_empty());
    }

    #[test]
    fn alarm_edits_are_validated_before_any_request() {
        let time = AlarmEdit::Time { alarm_id: 7, from: 390, to: 420 };
        assert!(alarms("3001", std::slice::from_ref(&time), false).is_ok());
        for (device, edits, confirm) in [
            ("", vec![time.clone()], false),
            ("../x", vec![time.clone()], false),
            ("3001", vec![], false),
            ("3001", vec![AlarmEdit::Time { alarm_id: 7, from: 390, to: 390 }], false),
            ("3001", vec![AlarmEdit::Time { alarm_id: 7, from: 390, to: 1440 }], false),
            ("3001", vec![AlarmEdit::On { alarm_id: 7, from: true, to: true }], false),
            ("3001", vec![AlarmEdit::Days { alarm_id: 7, from: vec!["ONCE".into()], to: vec![] }], false),
            ("3001", vec![AlarmEdit::Days { alarm_id: 7, from: vec!["ONCE".into()], to: vec!["monday".into()] }], false),
            ("3001", vec![time.clone(), time.clone()], false),
            ("3001", vec![AlarmEdit::Remove { alarm_id: 7 }], false),
            ("3001", vec![AlarmEdit::Remove { alarm_id: 7 }, time.clone()], true),
        ] {
            assert!(alarms(device, &edits, confirm).is_err(), "{device} {edits:?}");
        }
        assert!(alarms("3001", &[AlarmEdit::Remove { alarm_id: 7 }], true).is_ok());
    }

    #[test]
    fn lists_compare_as_sets_and_numbers_by_value() {
        assert!(same(Some(&serde_json::json!(["TUESDAY", "MONDAY"])), Some(&serde_json::json!(["MONDAY", "TUESDAY"]))));
        assert!(same(Some(&serde_json::json!({"a": 420})), Some(&serde_json::json!({"a": 420.0}))));
        assert!(!same(Some(&serde_json::json!(420)), None));
    }
}
