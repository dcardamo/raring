//! Tolerant projections of raw Garmin responses. Unknown and absent fields stay absent.
use crate::{phase, plan, planned, rules, store::Store, sync::selected_watch, view::*};
use chrono::{
    DateTime, Datelike, Duration, FixedOffset, Local, NaiveDate, NaiveDateTime, Timelike, Utc,
};
use serde_json::Value;
use std::collections::BTreeMap;

mod explain;
mod thresholds;

fn at<'a>(v: &'a Value, keys: &[&str]) -> &'a Value {
    keys.iter().fold(v, |v, k| &v[*k])
}
fn num(v: &Value, paths: &[&[&str]]) -> Option<f64> {
    paths.iter().find_map(|p| {
        let x = at(v, p);
        x.as_f64()
            .or_else(|| x.as_str().and_then(|s| s.parse().ok()))
    })
}
fn string(v: &Value, paths: &[&[&str]]) -> Option<String> {
    paths
        .iter()
        .find_map(|p| at(v, p).as_str().map(str::to_owned))
}
/// Garmin sends feedback as codes ("LET_YOUR_BODY_RECOVER"); show them as a
/// short sentence ("Let your body recover"). Text that already reads as prose
/// passes through unchanged.
pub(crate) fn humanize_code(code: &str) -> Option<String> {
    let code = code.trim();
    if code.is_empty() || code.len() > 60 {
        return None;
    }
    if code.chars().any(|c| c.is_ascii_lowercase()) {
        return Some(code.to_owned());
    }
    let words = code.to_ascii_lowercase().replace('_', " ");
    let mut chars = words.trim().chars();
    let first = chars.next()?;
    Some(first.to_ascii_uppercase().to_string() + chars.as_str())
}
pub(crate) fn local_time(v: &Value, keys: &[&str]) -> Option<String> {
    let value = at(v, keys);
    if let Some(s) = value.as_str() {
        return Some(s.replace(' ', "T"));
    }
    let number = value.as_i64()?;
    // Garmin's *Local epoch milliseconds encode wall time; don't apply the host timezone.
    let millis = if number.abs() < 100_000_000_000 {
        number * 1000
    } else {
        number
    };
    chrono::DateTime::from_timestamp_millis(millis)
        .map(|d| d.naive_utc().format("%Y-%m-%dT%H:%M:%S").to_string())
}
fn date_sub(date: &str, n: i64) -> String {
    NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map(|d| (d - Duration::days(n)).to_string())
        .unwrap_or_else(|_| date.to_owned())
}
fn snapshot_time(v: &Value, field: &str) -> Option<i64> {
    let time = local_time(v, &[field])?;
    chrono::DateTime::parse_from_rfc3339(&time)
        .map(|d| d.timestamp_millis())
        .ok()
        .or_else(|| {
            chrono::NaiveDateTime::parse_from_str(&time, "%Y-%m-%dT%H:%M:%S%.f")
                .ok()
                .map(|d| d.and_utc().timestamp_millis())
        })
}
pub(crate) fn readiness(v: &Value) -> &Value {
    let Some(snapshots) = v.as_array() else {
        return v;
    };
    let valid = || {
        snapshots
            .iter()
            .filter(|s| num(s, &[&["score"], &["trainingReadinessScore"]]).is_some())
    };
    // The wake-up snapshot is only the morning reading. Garmin also sends
    // post-exercise and realtime updates; array order is not a freshness rule.
    // Compare one clock at a time, never UTC timestamps against local wall time.
    ["timestamp", "timestampLocal"]
        .iter()
        .find_map(|field| {
            valid()
                .filter_map(|s| snapshot_time(s, field).map(|time| (time, s)))
                .max_by_key(|(time, _)| *time)
                .map(|(_, s)| s)
        })
        .or_else(|| valid().find(|s| s["inputContext"] == "AFTER_WAKEUP_RESET"))
        .or_else(|| valid().next())
        .unwrap_or(&Value::Null)
}
fn battery_latest(v: &Value) -> Option<f64> {
    v["bodyBatteryValuesArray"]
        .as_array()?
        .iter()
        .filter_map(|sample| {
            let time = sample.get(0)?.as_i64()?;
            let level = sample.get(1)?.as_f64()?;
            (0.0..=100.0).contains(&level).then_some((time, level))
        })
        .max_by_key(|(time, _)| *time)
        .map(|(_, level)| level)
}
pub(crate) fn sleep(v: &Value) -> &Value {
    if !v["dailySleepDTO"].is_null() {
        &v["dailySleepDTO"]
    } else {
        v
    }
}
/// A night's overnight averages. The heart rate is Garmin's average during
/// sleep, never resting heart rate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Night {
    HeartRate,
    Spo2,
    Respiration,
    SkinTemp,
}
/// Field names inside a sleep record: `dailySleepDTO` in a per-day row, the
/// item itself in a history row. First match wins.
fn night_fields(field: Night) -> &'static [&'static str] {
    match field {
        Night::HeartRate => &["avgHeartRate", "averageHeartRate"],
        Night::Spo2 => &["averageSpO2Value", "avgSpO2"],
        Night::Respiration => &["averageRespirationValue"],
        Night::SkinTemp => &["skinTempDeviation"],
    }
}
/// A per-day sleep row carries skin temperature beside the record, and that
/// value wins over the record's own.
const ROW_SKIN_TEMP: &str = "avgSkinTempDeviationC";
pub(crate) fn night_valid(field: Night, n: f64) -> bool {
    n.is_finite()
        && match field {
            Night::HeartRate => n > 0. && n < 300.,
            Night::Spo2 => n > 0. && n <= 100.,
            Night::Respiration => n > 0. && n < 100.,
            Night::SkinTemp => n.abs() < 10.,
        }
}
/// One overnight average from a sleep row or history item.
pub(crate) fn night_value(raw: &Value, field: Night) -> Option<f64> {
    let outer = (field == Night::SkinTemp)
        .then(|| num(raw, &[&[ROW_SKIN_TEMP]]))
        .flatten();
    outer
        .or_else(|| {
            let record = sleep(raw);
            night_fields(field)
                .iter()
                .find_map(|f| num(record, &[&[f]]))
        })
        .filter(|n| night_valid(field, *n))
}
/// The same fields as `night_value`, as SQLite JSON paths in the same order,
/// so long trends read them without parsing whole nights.
pub(crate) fn night_paths(field: Night) -> Vec<String> {
    let fields = night_fields(field);
    (field == Night::SkinTemp)
        .then(|| format!("$.{ROW_SKIN_TEMP}"))
        .into_iter()
        .chain(fields.iter().map(|f| format!("$.dailySleepDTO.{f}")))
        .chain(fields.iter().map(|f| format!("$.{f}")))
        .collect()
}
fn sleep_need_minutes(need: &Value) -> Option<f64> {
    // Connect's adjusted "actual" need is in minutes; the older explicit
    // seconds fields need conversion. "baseline" is not the adjusted need.
    // See garth/data/daily_sleep_data.py's sleep_need_minutes projection.
    num(need, &[&["actual"]])
        .filter(|n| n.is_finite() && *n > 0.)
        .or_else(|| {
            ["totalSleepNeedInSeconds", "totalSleepNeedSeconds"]
                .iter()
                .find_map(|key| {
                    num(need, &[&[key]])
                        .filter(|n| n.is_finite() && *n > 0.)
                        .map(|seconds| seconds / 60.)
                })
        })
}
fn sleep_need(v: &Value) -> Option<f64> {
    sleep_need_minutes(&sleep(v)["sleepNeed"])
}
fn parse_local(s: &str) -> Option<NaiveDateTime> {
    NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.f").ok()
}
fn iso(t: NaiveDateTime) -> String {
    t.format("%Y-%m-%dT%H:%M:%S").to_string()
}
fn midnight(date: NaiveDate) -> NaiveDateTime {
    date.and_hms_opt(0, 0, 0).unwrap_or_default()
}
/// A sleep record with no data counts as no record.
fn sleep_record(store: &Store, date: &str) -> Option<Value> {
    store
        .get("sleep", date)
        .filter(|v| !v.is_null() && !sleep(v).is_null())
}
/// Garmin answers with an empty or all-null record before the watch has
/// uploaded a night, so only a duration or a recorded start or end counts.
pub(crate) fn recorded_sleep(raw: &Value) -> bool {
    let s = sleep(raw);
    num(s, &[&["sleepTimeSeconds"]]).is_some_and(|t| t > 0.)
        || ["sleepStartTimestampLocal", "sleepEndTimestampLocal"]
            .iter()
            .any(|k| local_time(s, &[k]).is_some())
}
fn need_detail(factor: &str, code: &str) -> String {
    match (factor, code) {
        (_, "HIGH_ACWR") => "high training load".into(),
        (_, "DAILY_ACTIVITY_AND_TODAYS_LOAD") => "today's activity and load".into(),
        (_, "INCREASING_LOW_QUALITY") => "recent low-quality sleep".into(),
        _ => humanize_code(code)
            .map(|s| s.to_lowercase())
            .unwrap_or_else(|| code.to_lowercase()),
    }
}
/// Garmin's named reasons behind tonight's need; "NO_CHANGE" codes are left out.
fn need_adjustments(need: &Value) -> Vec<NeedAdjustment> {
    [
        ("Training", "trainingFeedback"),
        ("Sleep history", "sleepHistoryAdjustment"),
        ("HRV", "hrvAdjustment"),
        ("Naps", "napAdjustment"),
    ]
    .iter()
    .filter_map(|(factor, key)| {
        let code = need[*key].as_str()?.trim();
        (!code.is_empty() && !code.starts_with("NO_CHANGE")).then(|| NeedAdjustment {
            factor: (*factor).into(),
            detail: need_detail(factor, code),
        })
    })
    .collect()
}
fn minutes_text(minutes: f64) -> String {
    let m = minutes.abs().round() as i64;
    if m >= 60 {
        format!("{}h {:02}m", m / 60, m % 60)
    } else {
        format!("{m} min")
    }
}
fn need_reason(need: f64, baseline: Option<f64>, adjustments: &[NeedAdjustment]) -> Option<String> {
    let baseline = baseline.filter(|b| b.is_finite() && *b > 0.)?;
    let details = adjustments
        .iter()
        .map(|a| a.detail.as_str())
        .collect::<Vec<_>>();
    let because = match details.as_slice() {
        [] => String::new(),
        [one] => format!(" for {one}"),
        [rest @ .., last] => format!(" for {} and {last}", rest.join(", ")),
    };
    let diff = need - baseline;
    Some(if diff.abs() < 1. {
        "Your usual need; Garmin made no adjustment tonight.".into()
    } else if diff > 0. {
        format!("Garmin added {}{because}.", minutes_text(diff))
    } else {
        format!("Garmin took off {}{because}.", minutes_text(diff))
    })
}
/// Recorded wake times of the 14 nights ending on `date`, hours after midnight.
fn wake_hours(nights: &[Value]) -> Vec<f64> {
    nights
        .iter()
        .filter_map(|night| local_time(sleep(night), &["sleepEndTimestampLocal"]))
        .filter_map(|time| hour(&time))
        .filter(|hour| hour.is_finite() && (0.0..24.0).contains(hour))
        .collect()
}
fn recent_nights(store: &Store, date: &str) -> Vec<Value> {
    (0..14)
        .filter_map(|i| store.get("sleep", &date_sub(date, i)))
        .collect()
}
/// Minutes between the phone's UTC offset now and the latest recorded night's.
fn zone_shift(nights: &[Value], now: DateTime<FixedOffset>) -> Option<i32> {
    nights.iter().find_map(|night| {
        let s = sleep(night);
        let local = s["sleepEndTimestampLocal"].as_i64()?;
        let gmt = s["sleepEndTimestampGMT"].as_i64()?;
        let night_offset = ((local - gmt) as f64 / 60_000.).round() as i32;
        Some(now.offset().local_minus_utc() / 60 - night_offset)
    })
}
pub(crate) fn sleep_plan(store: &Store, date: &str) -> Option<SleepPlan> {
    let nights = recent_nights(store, date);
    let wakes = wake_hours(&nights);
    let record = sleep_record(store, date);
    let current = record.clone().unwrap_or(Value::Null);
    let next = &sleep(&current)["nextSleepNeed"];
    let upcoming_need = sleep_need_minutes(next);
    let historical: Vec<f64> = nights.iter().filter_map(sleep_need).collect();
    let (need_min, source) = if let Some(need) = upcoming_need {
        (need, SleepNeedSource::Tonight)
    } else if let Some(need) = sleep_need(&current) {
        (need, SleepNeedSource::Latest)
    } else if !historical.is_empty() {
        (
            historical.iter().sum::<f64>() / historical.len() as f64,
            SleepNeedSource::Average,
        )
    } else {
        return None;
    };
    if !need_min.is_finite() || need_min <= 0. {
        return None;
    }
    let base = NaiveDate::parse_from_str(date, "%Y-%m-%d").ok()?;
    let tonight = source == SleepNeedSource::Tonight;
    // Garmin files tonight's need under the morning it ends.
    let night = next["calendarDate"]
        .as_str()
        .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .filter(|_| tonight)
        .unwrap_or(base + Duration::days(1));
    let evening = midnight(night - Duration::days(1));
    // Garmin's window is for the night its need describes, so only tonight's
    // need carries one. Its minutes count from the evening's midnight; its end
    // plus the need lands on Garmin's expected wake.
    let window = if tonight {
        garmin_bedtime_window(next)
    } else {
        None
    };
    let (start_at, bedtime_at, bedtime_source, night) = match window {
        Some((start, end)) => (
            Some(evening + Duration::minutes(start)),
            evening + Duration::minutes(end),
            BedtimeSource::Garmin,
            night,
        ),
        // The reminder key and bedtime must describe the same selected wake.
        None => {
            let wake = phase::next_wake(
                base,
                recorded_wake(store, date),
                rules::wake_clock(&wakes).as_ref(),
            )?;
            (
                None,
                wake - Duration::minutes(need_min.round() as i64),
                BedtimeSource::WakeHistory,
                wake.date(),
            )
        }
    };
    let lead = store.settings().bedtime_lead_min;
    let wind_down_at = start_at.unwrap_or(bedtime_at) - Duration::minutes(lead.into());
    let wake_time = rules::bedtime(&wakes, 0.);
    let baseline_min = num(next, &[&["baseline"]]).filter(|_| tonight);
    let adjustments = if tonight {
        need_adjustments(next)
    } else {
        vec![]
    };
    let alignment = sleep(&current)["sleepAlignment"]["status"].as_str();
    Some(SleepPlan {
        need_read_at: tonight.then(|| store.fetched_at("sleep", date)).flatten().filter(|t| DateTime::parse_from_rfc3339(t).is_ok()),
        night: night.to_string(),
        bedtime: bedtime_at.format("%H:%M").to_string(),
        bedtime_start: start_at.map(|t| t.format("%H:%M").to_string()),
        bedtime_source,
        wind_down: wind_down_at.format("%H:%M").to_string(),
        wind_down_lead_min: lead,
        wind_down_at: iso(wind_down_at),
        bedtime_start_at: start_at.map(iso),
        bedtime_at: iso(bedtime_at),
        tip: rules::steady_tip(alignment, wake_time.as_deref()),
        wake_time,
        need_min,
        source,
        reason: need_reason(need_min, baseline_min, &adjustments),
        baseline_min,
        adjustments,
        wake_nights: wakes.len() as u32,
        stale: !record.as_ref().is_some_and(recorded_sleep),
    })
}
fn garmin_bedtime_window(need: &Value) -> Option<(i64, i64)> {
    let minutes = |key: &str| {
        num(need, &[&[key]])
            .filter(|n| n.is_finite())
            .map(|n| n.round() as i64)
    };
    Some((
        minutes("recommendedBedtimeStartMins")?,
        minutes("recommendedBedtimeEndMins")?,
    ))
}
fn alignment_label(code: &str) -> (String, Tone) {
    match code {
        "ALIGNED" => ("Aligned".into(), Tone::Good),
        "AHEAD" => ("Ahead".into(), Tone::Fair),
        "BEHIND" => ("Behind".into(), Tone::Fair),
        "JET_LAG" => ("Jet lag".into(), Tone::Neutral),
        other => (
            humanize_code(other).unwrap_or_else(|| other.into()),
            Tone::Neutral,
        ),
    }
}
fn calibrating(code: &str) -> bool {
    ["CALIBRAT", "NOT_ENOUGH", "INSUFFICIENT", "LEARNING"]
        .iter()
        .any(|k| code.contains(k))
}
fn alignment_night(date: &str, raw: &Value) -> AlignmentNight {
    let s = sleep(raw);
    let a = &s["sleepAlignment"];
    let (status, tone) = match a["status"].as_str() {
        Some(code) => {
            let (label, tone) = alignment_label(code);
            (Some(label), tone)
        }
        None => (None, Tone::Neutral),
    };
    let window = num(a, &[&["optimalSleepWindowStartMins"]])
        .zip(num(a, &[&["optimalSleepWindowEndMins"]]))
        .map(|(start, end)| [start, end]);
    let origin = NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .ok()
        .map(midnight);
    let offset = |key: &str| {
        let time = parse_local(&local_time(s, &[key])?)?;
        Some((time - origin?).num_minutes() as f64)
    };
    AlignmentNight {
        date: date.into(),
        status,
        tone,
        window,
        sleep: offset("sleepStartTimestampLocal")
            .zip(offset("sleepEndTimestampLocal"))
            .filter(|(start, end)| end > start)
            .map(|(start, end)| [start, end]),
    }
}
fn short_date(date: &str) -> String {
    NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map(|d| d.format("%a %-d %b").to_string())
        .unwrap_or_else(|_| date.into())
}
pub(crate) fn alignment(store: &Store, date: &str) -> Option<SleepAlignment> {
    let records: Vec<(String, Value)> = (0..14)
        .filter_map(|i| {
            let day = date_sub(date, i);
            sleep_record(store, &day).map(|v| (day, v))
        })
        .collect();
    if records.is_empty() {
        return None;
    }
    let history = records
        .iter()
        .filter(|(day, _)| day.as_str() > date_sub(date, 7).as_str())
        .rev()
        .map(|(day, v)| alignment_night(day, v))
        .collect();
    let rated = records
        .iter()
        .find(|(_, v)| sleep(v)["sleepAlignment"]["status"].is_string());
    let Some((day, raw)) = rated else {
        return Some(SleepAlignment {
            state: AlignmentState::Unavailable,
            latest: None,
            detail: "Garmin sent no sleep alignment for recent nights. Not every watch reports it."
                .into(),
            history,
        });
    };
    let code = sleep(raw)["sleepAlignment"]["status"]
        .as_str()
        .unwrap_or_default();
    let latest = alignment_night(day, raw);
    let (state, detail) = if calibrating(code) {
        (
            AlignmentState::Calibrating,
            "Garmin is still learning your rhythm.".to_owned(),
        )
    } else if day != date {
        (
            AlignmentState::Stale,
            format!(
                "No alignment for last night yet. Showing {}.",
                short_date(day)
            ),
        )
    } else {
        let detail = match code {
            "ALIGNED" => "Last night lined up with Garmin's optimal window.".to_owned(),
            "AHEAD" => "Last night ran earlier than Garmin's optimal window.".into(),
            "BEHIND" => "Last night ran later than Garmin's optimal window.".into(),
            "JET_LAG" if latest.window.is_none() => {
                "Garmin reports jet lag and has no optimal window for this night.".into()
            }
            _ => format!(
                "Garmin's status: {}.",
                latest.status.clone().unwrap_or_default()
            ),
        };
        (AlignmentState::Current, detail)
    };
    Some(SleepAlignment {
        state,
        latest: Some(latest),
        detail,
        history,
    })
}
/// Garmin's averages from the seven-day sleep stats ending on `date`.
fn garmin_averages(store: &Store, date: &str) -> Option<SleepAverages> {
    let v = store.get("sleep_stats", date)?;
    let overall = &v["overallStats"];
    let clock_of = |key: &str| {
        num(overall, &[&[key]])
            .filter(|s| s.is_finite())
            .map(|seconds| rules::clock((seconds / 60.).round() as i64))
    };
    let nights: Vec<&str> = v["individualStats"]
        .as_array()?
        .iter()
        .filter(|n| !n["values"]["localSleepStartTimeInMillis"].is_null())
        .filter_map(|n| n["calendarDate"].as_str())
        .collect();
    Some(SleepAverages {
        bedtime: clock_of("averageLocalSleepStartTime")?,
        wake_time: clock_of("averageLocalSleepEndTime")?,
        nights: nights.len() as u32,
        through: nights.iter().max().map(|d| (*d).to_owned())?,
    })
}
fn first(v: &Value) -> &Value {
    v.as_array().and_then(|a| a.first()).unwrap_or(v)
}
pub(crate) fn vo2(v: &Value) -> Option<f64> {
    num(
        first(v),
        &[
            &["generic", "vo2MaxValue"],
            &["vo2MaxValue"],
            &["generic", "vo2MaxPreciseValue"],
        ],
    )
}
fn historical_vo2(store: &Store, date: &str) -> Option<f64> {
    store
        .history("max_metrics", date, 1)
        .pop()
        .and_then(|(_, v)| vo2(&v))
        .or_else(|| {
            store
                .history("training", date, 1)
                .pop()
                .and_then(|(_, v)| vo2(&v["mostRecentVO2Max"]))
        })
}
fn sleep_stage_time(v: &Value, gmt: &str, timestamp: &str) -> Option<f64> {
    let raw = &v[gmt];
    if let Some(s) = raw.as_str() {
        if let Ok(t) = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.f") {
            return Some(t.and_utc().timestamp_millis() as f64);
        }
    }
    num(v, &[&[gmt], &[timestamp]]).map(|n| {
        if n.abs() < 100_000_000_000. {
            n * 1000.
        } else {
            n
        }
    })
}
fn overnight_samples(raw: &Value, kind: OvernightKind) -> Vec<Option<f64>> {
    let (root, field, fallback) = match kind {
        OvernightKind::Hrv => ("hrvData", "value", "hrv"),
        OvernightKind::HeartRate => ("sleepHeartRate", "value", "heartRate"),
        OvernightKind::Respiration => (
            "wellnessEpochRespirationDataDTOList",
            "respirationValue",
            "respiration",
        ),
        OvernightKind::Spo2 => ("wellnessEpochSPO2DataDTOList", "spo2Reading", "spo2"),
    };
    let valid = |n: f64| {
        n.is_finite()
            && match kind {
                OvernightKind::Hrv => n > 0.,
                OvernightKind::HeartRate => n > 0. && n < 300.,
                OvernightKind::Respiration => n > 0. && n < 100.,
                OvernightKind::Spo2 => n > 0. && n <= 100.,
            }
    };
    if let Some(samples) = raw[root].as_array() {
        samples
            .iter()
            .map(|sample| num(sample, &[&[field]]).filter(|n| valid(*n)))
            .collect()
    } else {
        raw["overnightSeries"][fallback]
            .as_array()
            .map(|a| {
                a.iter()
                    .map(|sample| num(sample, &[&[]]).filter(|n| valid(*n)))
                    .collect()
            })
            .unwrap_or_default()
    }
}
pub(crate) fn sscore(v: &Value) -> Option<f64> {
    let s = sleep(v);
    num(
        s,
        &[
            &["sleepScores", "overall", "value"],
            &["sleepScores", "overallScore"],
            &["sleepScore"],
            &["score"],
        ],
    )
}
fn current_rscore(v: &Value) -> Option<f64> {
    num(readiness(v), &[&["score"], &["trainingReadinessScore"]])
}
pub(crate) fn rscore(v: &Value) -> Option<f64> {
    // Historical days and weekly comparisons need a consistent morning baseline,
    // not a post-workout dip that depends on when that day's last sync happened.
    let morning = v
        .as_array()
        .and_then(|a| {
            a.iter()
                .find(|s| s["inputContext"] == "AFTER_WAKEUP_RESET")
                .or_else(|| a.first())
        })
        .unwrap_or(v);
    num(morning, &[&["score"], &["trainingReadinessScore"]])
}
fn hrv(v: &Value) -> &Value {
    if v["hrvSummary"].is_object() {
        &v["hrvSummary"]
    } else {
        v
    }
}
fn gauge(value: f64, max: f64, label: &str, tone: Tone, band: Option<[f64; 2]>) -> Gauge {
    Gauge {
        value,
        max,
        label: label.into(),
        tone,
        band,
    }
}
fn label(v: Option<String>) -> String {
    let code = v.unwrap_or_default();
    let code = code
        .rsplit_once('_')
        .filter(|(prefix, suffix)| {
            !prefix.is_empty()
                && prefix
                    .chars()
                    .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
                && !suffix.is_empty()
                && suffix.chars().all(|c| c.is_ascii_digit())
        })
        .map(|(prefix, _)| prefix)
        .unwrap_or(&code);
    code.replace('_', " ")
        .to_ascii_lowercase()
        .split_whitespace()
        .map(|s| {
            let mut c = s.chars();
            format!(
                "{}{}",
                c.next().unwrap_or(' ').to_ascii_uppercase(),
                c.as_str()
            )
        })
        .collect::<Vec<_>>()
        .join(" ")
}
fn device(v: &Value) -> &Value {
    v.as_object()
        .and_then(|map| {
            map.values()
                .find(|value| value["primaryTrainingDevice"].as_bool() == Some(true))
                .or_else(|| map.values().find(|value| value.is_object()))
        })
        .unwrap_or(&Value::Null)
}
fn training_device(v: &Value) -> &Value {
    let nested = &v["mostRecentTrainingStatus"]["latestTrainingStatusData"];
    device(if nested.is_object() {
        nested
    } else {
        &v["latestTrainingStatusData"]
    })
}
fn load_device(v: &Value) -> &Value {
    let nested = &v["mostRecentTrainingLoadBalance"]["metricsTrainingLoadBalanceDTOMap"];
    device(if nested.is_object() {
        nested
    } else {
        &v["metricsTrainingLoadBalanceDTOMap"]
    })
}
fn ratio(v: &Value) -> Option<f64> {
    num(v, &[&["acuteChronicWorkloadRatio"]]).or_else(|| {
        num(
            training_device(v),
            &[
                &["acuteChronicWorkloadRatio"],
                &["acuteTrainingLoadDTO", "acuteChronicWorkloadRatio"],
                &["acuteTrainingLoadDTO", "dailyAcuteChronicWorkloadRatio"],
            ],
        )
    })
}
pub(crate) fn status(v: &Value) -> Option<String> {
    string(
        training_device(v),
        &[
            &["trainingStatus"],
            &["trainingStatusFeedbackPhrase"],
            &["feedbackPhrase"],
        ],
    )
    .or_else(|| {
        string(
            v,
            &[
                &["trainingStatus"],
                &["trainingStatusFeedbackPhrase"],
                &["trainingStatusData", "trainingStatus"],
                &["latestTrainingStatusData", "trainingStatus"],
            ],
        )
    })
}
pub(crate) fn acute(v: &Value) -> Option<f64> {
    num(
        training_device(v),
        &[
            &["acuteTrainingLoadDTO", "dailyTrainingLoadAcute"],
            &["acuteTrainingLoadDTO", "acuteTrainingLoad"],
            &["acuteTrainingLoad"],
        ],
    )
    .or_else(|| {
        num(
            v,
            &[
                &["acuteTrainingLoad"],
                &["acuteLoad"],
                &["trainingStatusData", "acuteTrainingLoad"],
                &["latestTrainingStatusData", "acuteTrainingLoad"],
            ],
        )
    })
}
fn band(v: &Value) -> Option<[f64; 2]> {
    let nested = &training_device(v)["acuteTrainingLoadDTO"];
    let low = num(nested, &[&["minOptimalLoad"], &["optimalRangeMin"]]).or_else(|| {
        num(
            v,
            &[
                &["optimalRangeMin"],
                &["minOptimalLoad"],
                &["trainingStatusData", "optimalRangeMin"],
            ],
        )
    })?;
    let high = num(nested, &[&["maxOptimalLoad"], &["optimalRangeMax"]]).or_else(|| {
        num(
            v,
            &[
                &["optimalRangeMax"],
                &["maxOptimalLoad"],
                &["trainingStatusData", "optimalRangeMax"],
            ],
        )
    })?;
    Some([low, high])
}
/// Garmin's rating as a tone. Without a rating the factor stays neutral:
/// Raring doesn't rate Garmin's percentages itself.
fn factor_tone(feedback: Option<&str>) -> Tone {
    let feedback = feedback.unwrap_or("").to_ascii_uppercase();
    let words: Vec<&str> = feedback.split(|c: char| !c.is_ascii_alphabetic()).collect();
    if words.iter().any(|w| {
        matches!(
            *w,
            "POOR" | "LOW" | "NEGATIVE" | "UNBALANCED" | "INSUFFICIENT" | "HIGH"
        )
    }) {
        Tone::Poor
    } else if words
        .iter()
        .any(|w| matches!(*w, "FAIR" | "MODERATE" | "AVERAGE"))
    {
        Tone::Fair
    } else if words.iter().any(|w| {
        matches!(
            *w,
            "GOOD" | "EXCELLENT" | "OPTIMAL" | "BALANCED" | "POSITIVE" | "IDEAL"
        )
    }) {
        Tone::Good
    } else {
        Tone::Neutral
    }
}
// Detail fields live in summaryDTO; the list record can fill gaps in a partial detail.
struct ActivityPayload {
    detail: Value,
    listed: Value,
}
impl ActivityPayload {
    fn get(&self, key: &str) -> &Value {
        [
            &self.detail["summaryDTO"][key],
            &self.detail[key],
            &self.listed[key],
        ]
        .into_iter()
        .find(|value| !value.is_null())
        .unwrap_or(&Value::Null)
    }
    fn number(&self, keys: &[&str]) -> Option<f64> {
        keys.iter().find_map(|key| {
            let value = self.get(key);
            value.as_f64().or_else(|| value.as_str()?.parse().ok())
        })
    }
    fn text(&self, keys: &[&str]) -> Option<String> {
        keys.iter()
            .find_map(|key| self.get(key).as_str().map(str::to_owned))
    }
    fn kind(&self) -> String {
        string(
            &self.detail,
            &[
                &["activityTypeDTO", "typeKey"],
                &["activityType", "typeKey"],
            ],
        )
        .or_else(|| string(&self.listed, &[&["activityType", "typeKey"]]))
        .unwrap_or_default()
        .to_ascii_lowercase()
    }
}
fn activity_payload(store: &Store, id: &str) -> Option<ActivityPayload> {
    let detail = store.get("activity", id).unwrap_or(Value::Null);
    let listed = store
        .listed_activity(id)
        .or_else(|| store.get("listed", id))
        .unwrap_or(Value::Null);
    (!detail.is_null() || !listed.is_null()).then_some(ActivityPayload { detail, listed })
}
/// Type key, minutes and kilometres. The stored detail answers first, as in
/// `ActivityPayload::get`; the list pages are read only when it lacks a field.
pub(crate) fn activity_brief(store: &Store, id: &str) -> Option<(String, f64, Option<f64>)> {
    if let Some(detail) = store.get("activity", id) {
        let payload = ActivityPayload { detail, listed: Value::Null };
        let kind = payload.kind();
        let has = |key: &str| !payload.get(key).is_null();
        if !kind.is_empty() && has("duration") && has("distance") {
            let km = payload.number(&["distance"]).filter(|m| *m > 0.).map(|m| m / 1000.);
            return Some((kind, payload.number(&["duration"]).unwrap_or(0.) / 60., km));
        }
    }
    let payload = activity_payload(store, id)?;
    let km = payload.number(&["distance"]).filter(|m| *m > 0.).map(|m| m / 1000.);
    Some((payload.kind(), payload.number(&["duration"]).unwrap_or(0.) / 60., km))
}
pub(crate) fn activity_kind(store: &Store, id: &str) -> Option<String> {
    activity_payload(store, id)
        .map(|p| p.kind())
        .filter(|k| !k.is_empty())
}

pub(crate) fn activity(store: &Store, id: &str) -> Option<ActivityCard> {
    let v = activity_payload(store, id)?;
    let kind = v.kind();
    let icon = if kind.contains("run") {
        Icon::Run
    } else if kind.contains("cycl") || kind.contains("bike") {
        Icon::Bike
    } else if kind.contains("swim") {
        Icon::Swim
    } else if kind.contains("walk") {
        Icon::Walk
    } else if kind.contains("hik") {
        Icon::Hike
    } else if kind.contains("strength") {
        Icon::Strength
    } else {
        Icon::Other
    };
    let zones = store.get("zones", id).unwrap_or(Value::Null);
    Some(ActivityCard {
        id: id.into(),
        icon,
        name: v
            .text(&["activityName"])
            .unwrap_or_else(|| "Workout".into()),
        start_local: v.text(&["startTimeLocal"]).unwrap_or_default(),
        duration_s: v.number(&["duration"]).unwrap_or(0.),
        distance_km: v
            .number(&["distance"])
            .filter(|m| *m > 0.)
            .map(|m| m / 1000.),
        load: v.number(&["activityTrainingLoad", "trainingLoad"]),
        zone_seconds: zones
            .as_array()
            .map(|a| {
                if a.iter().any(|z| z["zoneNumber"].as_u64().is_some()) {
                    let mut seconds = vec![0.; 5];
                    for z in a {
                        if let Some(n @ 1..=5) = z["zoneNumber"].as_u64() {
                            seconds[n as usize - 1] =
                                num(z, &[&["secsInZone"], &["timeInZone"]]).unwrap_or(0.);
                        }
                    }
                    seconds
                } else {
                    a.iter()
                        .map(|z| num(z, &[&["secsInZone"], &["timeInZone"]]).unwrap_or(0.))
                        .collect()
                }
            })
            .unwrap_or_default(),
        aerobic_te: v.number(&["trainingEffect", "aerobicTrainingEffect"]),
        aerobic_label: v
            .text(&["aerobicTrainingEffectMessage"])
            .map(|s| label(Some(s))),
        anaerobic_te: v.number(&["anaerobicTrainingEffect"]),
        anaerobic_label: v
            .text(&["anaerobicTrainingEffectMessage"])
            .map(|s| label(Some(s))),
        benefit: v
            .text(&["trainingEffectLabel", "primaryBenefit"])
            .map(|s| label(Some(s))),
        avg_hr: v.number(&["averageHR", "avgHR"]).filter(|n| *n > 0.),
        intensity_min: v
            .number(&["moderateIntensityMinutes"])
            .zip(v.number(&["vigorousIntensityMinutes"]))
            .filter(|(m, g)| *m >= 0. && *g >= 0.)
            .map(|(m, g)| m + 2. * g),
    })
}
/// Stored recordings older than one already shown, newest first. The sport
/// filter applies before the page limit, so other sports can't hide older
/// ones; no Garmin request is made.
pub fn activity_page(store: &Store, before_start: &str, before_id: &str, icon: Option<Icon>, limit: usize) -> ActivityPage {
    let (mut start, mut id) = (before_start.to_owned(), before_id.to_owned());
    let mut activities = vec![];
    loop {
        let batch = store.activities_before(&start, &id, 100);
        if batch.is_empty() {
            return ActivityPage { activities, more: false };
        }
        for (next_id, next_start) in batch {
            if activities.len() == limit {
                return ActivityPage { activities, more: true };
            }
            if let Some(card) = activity(store, &next_id).filter(|c| icon.is_none_or(|i| c.icon == i)) {
                activities.push(card);
            }
            (start, id) = (next_start, next_id);
        }
    }
}
fn activities(store: &Store, since: &str) -> Vec<ActivityCard> {
    store
        .activity_ids(Some(since), 200)
        .iter()
        .filter_map(|id| activity(store, id))
        .collect()
}
pub(crate) fn age(store: &Store, date: &str) -> FitnessAgeCard {
    let (recorded_on, v) = store
        .get("fitness_age", date)
        .map(|v| (Some(date.to_owned()), v))
        .or_else(|| store.history("fitness_age", date, 1).pop().map(|(d, v)| (Some(d), v)))
        .unwrap_or((None, Value::Null));
    let fitness_age = num(&v, &[&["fitnessAge"], &["fitnessAgeData", "fitnessAge"]]);
    let trend: Vec<Option<f64>> = (0..12)
        .rev()
        .map(|n| {
            let month = NaiveDate::parse_from_str(date, "%Y-%m-%d")
                .ok()?
                .checked_sub_months(chrono::Months::new(n))?;
            store
                .history("fitness_age", &month.to_string(), 1)
                .pop()
                .and_then(|(_, v)| num(&v, &[&["fitnessAge"], &["fitnessAgeData", "fitnessAge"]]))
        })
        .collect();
    let change_12m = fitness_age.zip(trend[0]).map(|(a, b)| a - b);
    FitnessAgeCard {
        fitness_age,
        chronological_age: num(&v, &[&["chronologicalAge"]]),
        trend,
        change_12m,
        recorded_on: recorded_on.filter(|_| fitness_age.is_some()),
    }
}
fn vital_value(store: &Store, kind: VitalKind, date: &str) -> Option<f64> {
    let (source, paths): (&str, &[&[&str]]) = match kind {
        VitalKind::RestingHr => (
            "summary",
            &[&["restingHeartRate"], &["restingHeartRateInBeatsPerMinute"]],
        ),
        VitalKind::Hrv => ("hrv", &[&["hrvSummary", "lastNightAvg"], &["lastNightAvg"]]),
        VitalKind::Respiration => (
            "respiration",
            &[&["sleepingRespirationValue"], &["avgSleepRespirationValue"]],
        ),
        VitalKind::Spo2 => ("spo2", &[&["averageSpO2"], &["sleepSpO2"]]),
        VitalKind::SkinTemp => {
            return store
                .get("sleep", date)
                .and_then(|v| night_value(&v, Night::SkinTemp))
        }
    };
    store.get(source, date).and_then(|v| num(&v, paths))
}
fn health_metric(store: &Store, kind: VitalKind, date: &str) -> Option<Value> {
    let name = match kind {
        VitalKind::RestingHr => "HR", // Garmin's overnight average, not resting HR.
        VitalKind::Hrv => "HRV",
        VitalKind::Respiration => "RESPIRATION",
        VitalKind::Spo2 => "SPO2",
        VitalKind::SkinTemp => "SKIN_TEMP_C",
    };
    store.get("health_status", date)?["metrics"]
        .as_array()?
        .iter()
        .find(|m| m["type"] == name && (m["status"].is_string() || garmin_range(m).is_some()))
        .cloned()
}
fn finite_number(v: &Value) -> Option<f64> {
    v.as_f64().filter(|n| n.is_finite())
}
fn garmin_range(metric: &Value) -> Option<[f64; 2]> {
    let lo = finite_number(&metric["baselineLowerLimit"])?;
    let hi = finite_number(&metric["baselineUpperLimit"])?;
    (lo <= hi).then_some([lo, hi])
}
fn garmin_tone(status: &str) -> Tone {
    match status {
        "IN_RANGE" => Tone::Good,
        "ABOVE_RANGE" | "BELOW_RANGE" | "OUT_OF_RANGE" => Tone::Poor,
        _ => Tone::Neutral,
    }
}
pub fn vitals(store: &Store, date: &str) -> VitalsCard {
    let mut readings = vec![];
    for (kind, unit) in [
        (VitalKind::RestingHr, "bpm"),
        (VitalKind::Hrv, "ms"),
        (VitalKind::Respiration, "br/min"),
        (VitalKind::Spo2, "%"),
        (VitalKind::SkinTemp, "°C"),
    ] {
        let metric = health_metric(store, kind, date);
        let status = metric
            .as_ref()
            .and_then(|m| m["status"].as_str())
            .map(str::to_owned);
        let graph_value = metric.as_ref().and_then(|m| finite_number(&m["value"]));
        let trend: Vec<Option<f64>> = (0..30)
            .rev()
            .map(|i| {
                let day = date_sub(date, i);
                if metric.is_some() {
                    health_metric(store, kind, &day).and_then(|m| finite_number(&m["value"]))
                } else {
                    vital_value(store, kind, &day)
                }
            })
            .collect();
        let value = if metric.is_some() {
            graph_value
        } else {
            vital_value(store, kind, date)
        };
        let fallback = if kind == VitalKind::Hrv {
            store.get("hrv", date).and_then(|v| {
                let h = hrv(&v);
                Some([
                    num(
                        h,
                        &[
                            &["baseline", "balancedLow"],
                            &["baseline", "lowUpper"],
                            &["baseline", "lower"],
                        ],
                    )?,
                    num(
                        h,
                        &[
                            &["baseline", "highUpper"],
                            &["baseline", "balancedUpper"],
                            &["baseline", "upper"],
                        ],
                    )?,
                ])
            })
        } else {
            rules::normal(&trend.iter().flatten().copied().collect::<Vec<_>>())
        };
        let garmin = metric.as_ref().and_then(garmin_range);
        // An onboarding baseline is not calibrated, even when bounds have numbers.
        let onboarding = status.as_deref() == Some("ONBOARDING");
        let (normal, range_source) = if onboarding {
            (None, VitalRangeSource::None)
        } else if let Some(range) = garmin {
            (Some(range), VitalRangeSource::Garmin)
        } else if metric.is_none() || (kind == VitalKind::Hrv && status.is_none()) {
            (
                fallback,
                if fallback.is_some() {
                    if kind == VitalKind::Hrv {
                        VitalRangeSource::Garmin
                    } else {
                        VitalRangeSource::Raring
                    }
                } else {
                    VitalRangeSource::None
                },
            )
        } else {
            (None, VitalRangeSource::None)
        };
        let tone = if value.is_none() || onboarding {
            Tone::Neutral
        } else if let Some(status) = status.as_deref() {
            garmin_tone(status)
        } else {
            match (value, normal) {
                (Some(v), Some([lo, hi])) if v < lo || v > hi => Tone::Poor,
                (Some(_), Some(_)) => Tone::Good,
                _ => Tone::Neutral,
            }
        };
        let lo = normal
            .map(|r| r[0])
            .or(value)
            .unwrap_or(0.)
            .min(value.unwrap_or(f64::INFINITY));
        let hi = normal
            .map(|r| r[1])
            .or(value)
            .unwrap_or(1.)
            .max(value.unwrap_or(f64::NEG_INFINITY));
        let span = (hi - lo).abs().max(1.);
        readings.push(VitalReading {
            kind,
            value,
            unit: unit.into(),
            tone,
            status,
            range_source,
            normal,
            scale: [lo - span, hi + span],
            trend,
        });
    }
    let off_count = readings.iter().filter(|r| r.tone == Tone::Poor).count() as u32;
    let baseline_ready = readings
        .iter()
        .any(|r| r.value.is_some() && r.normal.is_some() && r.tone != Tone::Neutral);
    VitalsCard {
        readings,
        off_count,
        baseline_ready,
    }
}
fn stress_average(store: &Store, date: &str) -> Option<f64> {
    // Garmin reports -1 for a day without enough readings.
    store
        .get("stress", date)
        .and_then(|v| num(&v, &[&["avgStressLevel"]]))
        .or_else(|| {
            store
                .get("summary", date)
                .and_then(|v| num(&v, &[&["averageStressLevel"]]))
        })
        .filter(|v| (0. ..=100.).contains(v))
}
fn stress_level(value: f64) -> StressLevel {
    if value < 26. {
        StressLevel::Rest
    } else if value < 51. {
        StressLevel::Low
    } else if value < 76. {
        StressLevel::Medium
    } else {
        StressLevel::High
    }
}
/// Garmin samples every three minutes and marks unmeasurable readings with
/// negative values: -1 off-wrist or too little data, -2 too much motion.
fn stress_timeline(v: &Value) -> (Vec<Option<f64>>, Vec<[f64; 2]>) {
    let start = string(v, &[&["startTimestampGMT"]])
        .and_then(|s| NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S%.f").ok())
        .map(|d| d.and_utc().timestamp_millis());
    let (Some(start), Some(samples)) = (start, v["stressValuesArray"].as_array()) else {
        return (vec![], vec![]);
    };
    let mut slots: Vec<Option<f64>> = vec![];
    let mut active: Vec<[f64; 2]> = vec![];
    for sample in samples {
        let (Some(time), Some(value)) = (
            sample.get(0).and_then(Value::as_i64),
            sample.get(1).and_then(Value::as_f64),
        ) else {
            continue;
        };
        let minute = (time - start) as f64 / 60_000.;
        if !(0. ..1440.).contains(&minute) {
            continue;
        }
        let slot = (minute / 3.) as usize;
        if slots.len() <= slot {
            slots.resize(slot + 1, None);
        }
        let from = slot as f64 * 3.;
        if (0. ..=100.).contains(&value) {
            slots[slot] = Some(value);
        } else if value == -2. {
            match active.last_mut() {
                Some([_, end]) if *end >= from => *end = from + 3.,
                _ => active.push([from, from + 3.]),
            }
        }
    }
    (slots, active)
}
/// Garmin's intraday Body Battery from the daily stress payload, in the same
/// three-minute slots from the day's start as stress. Samples without a level
/// are gaps; the column order comes from Garmin's descriptors.
pub(crate) fn battery_timeline(v: &Value) -> Vec<Option<f64>> {
    battery_slots(v, false)
}
/// Garmin's levels in three-minute slots; `measured_only` drops samples whose
/// status says Garmin modelled them (a sample without a status counts).
pub(crate) fn battery_slots(v: &Value, measured_only: bool) -> Vec<Option<f64>> {
    let start = string(v, &[&["startTimestampGMT"]])
        .and_then(|s| NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S%.f").ok())
        .map(|d| d.and_utc().timestamp_millis());
    let column = |key: &str, fallback: usize| {
        v["bodyBatteryValueDescriptorsDTOList"]
            .as_array()
            .and_then(|list| list.iter().find(|d| d["bodyBatteryValueDescriptorKey"] == key))
            .and_then(|d| d["bodyBatteryValueDescriptorIndex"].as_u64())
            .map_or(fallback, |i| i as usize)
    };
    let (time_at, status_at, level_at) = (column("timestamp", 0), column("bodyBatteryStatus", 1), column("bodyBatteryLevel", 2));
    let (Some(start), Some(samples)) = (start, v["bodyBatteryValuesArray"].as_array()) else {
        return vec![];
    };
    let mut slots: Vec<Option<f64>> = vec![];
    for sample in samples {
        let (Some(time), Some(level)) = (
            sample.get(time_at).and_then(Value::as_i64),
            sample.get(level_at).and_then(Value::as_f64),
        ) else {
            continue;
        };
        if measured_only && sample.get(status_at).and_then(Value::as_str).is_some_and(|s| s != "MEASURED") {
            continue;
        }
        let minute = (time - start) as f64 / 60_000.;
        if !(0. ..1500.).contains(&minute) || !(0. ..=100.).contains(&level) {
            continue;
        }
        let slot = (minute / 3.) as usize;
        if slots.len() <= slot {
            slots.resize(slot + 1, None);
        }
        slots[slot] = Some(level);
    }
    slots
}
fn utc_millis(value: &Value) -> Option<i64> {
    if let Some(n) = value.as_i64() {
        return Some(if n.abs() < 100_000_000_000 { n * 1000 } else { n });
    }
    let text = value.as_str()?;
    chrono::DateTime::parse_from_rfc3339(text)
        .map(|d| d.timestamp_millis())
        .ok()
        .or_else(|| NaiveDateTime::parse_from_str(text, "%Y-%m-%dT%H:%M:%S%.f")
            .ok().map(|d| d.and_utc().timestamp_millis()))
}
fn local_millis(v: &Value, field: &str) -> Option<i64> {
    let local = local_time(v, &[field])?;
    NaiveDateTime::parse_from_str(&local, "%Y-%m-%dT%H:%M:%S%.f")
        .ok().map(|d| d.and_utc().timestamp_millis())
}
fn clip_elapsed(from: i64, to: i64, start: i64, duration_min: f64) -> Option<[f64; 2]> {
    let left = ((from - start) as f64 / 60_000.).max(0.);
    let right = ((to - start) as f64 / 60_000.).min(duration_min);
    (right > left).then_some([left, right])
}

pub fn heart_rate_day(store: &Store, date: &str) -> HeartRateDay {
    let raw = store.get("daily_hr", date).unwrap_or(Value::Null);
    let start = utc_millis(&raw["startTimestampGMT"]);
    let index = |key: &str, fallback: usize| {
        raw["heartRateValueDescriptors"]
            .as_array()
            .and_then(|fields| fields.iter().find(|field| field["key"] == key))
            .and_then(|field| field["index"].as_u64())
            .map(|i| i as usize)
            .unwrap_or(fallback)
    };
    let timestamp_index = index("timestamp", 0);
    let value_index = index("heartrate", 1);
    let samples = raw["heartRateValues"].as_array();
    let end = utc_millis(&raw["endTimestampGMT"]);
    let date_end = NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .ok().and_then(|d| (d + Duration::days(1)).and_hms_opt(0, 0, 0))
        .map(|d| d.and_utc().timestamp_millis());
    // Garmin's GMT end is the last upload on a partial day, even when its
    // "local end" claims next midnight. Only trust a completed past day's
    // GMT end as the actual boundary (23/24/25 elapsed hours).
    let completed = NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .is_ok_and(|day| day < Local::now().date_naive());
    let full_end = start.zip(end).filter(|(start, end)| {
        completed
            && date_end == local_millis(&raw, "endTimestampLocal")
            && (1380 * 60_000..=1500 * 60_000).contains(&(end - start))
    });
    let mut duration_min = full_end.map(|(start, end)| (end - start) as f64 / 60_000.).unwrap_or(1440.);
    if let (Some(start), Some(samples)) = (start, samples) {
        if let Some(last) = samples.iter().filter_map(|s| s.get(timestamp_index).and_then(Value::as_i64))
            .filter(|t| *t >= start && *t < start + 1560 * 60_000).max() {
            duration_min = duration_min.max((((last - start) / 120_000 + 1) * 2) as f64);
        }
    }
    // Only a complete, unchanged source offset can map local-only activity
    // or sleep fields onto the elapsed UTC axis. Across DST/travel, use GMT
    // timestamps or leave the overlay unknown rather than shift it by an hour.
    let offset = full_end.and_then(|(start, end)| {
        let first = local_millis(&raw, "startTimestampLocal")? - start;
        let last = local_millis(&raw, "endTimestampLocal")? - end;
        (first == last).then_some(first)
    });
    let mut timeline = Vec::new();
    if let (Some(start), Some(samples)) = (start, samples) {
        for sample in samples {
            let (Some(timestamp), Some(value)) =
                (sample.get(timestamp_index).and_then(Value::as_i64), sample.get(value_index))
            else { continue };
            if timestamp < start { continue }
            let minute = (timestamp - start) / 60_000;
            if minute as f64 >= duration_min { continue }
            let slot = minute as usize / 2;
            if timeline.len() <= slot { timeline.resize(slot + 1, None); }
            timeline[slot] = value.as_f64().filter(|n| (1. ..300.).contains(n));
        }
    }
    let mut sleep_intervals = Vec::new();
    let mut activities = Vec::new();
    if let Some(start) = start {
        for wake_day in [date.to_owned(), date_sub(date, -1)] {
            let Some(record) = store.get("sleep", &wake_day) else { continue };
            let night = sleep(&record);
            let field = |kind: &str| utc_millis(&night[format!("sleep{kind}TimestampGMT")])
                .or_else(|| offset.and_then(|zone| local_millis(night, &format!("sleep{kind}TimestampLocal")).map(|t| t-zone)));
            if let Some(range) = field("Start").zip(field("End"))
                .and_then(|(from,to)| clip_elapsed(from,to,start,duration_min)) {
                sleep_intervals.push(range);
            }
        }
        for (id, _) in store.activities_between(&date_sub(date, 1), date) {
            let Some(a) = activity(store, &id) else { continue };
            let payload = activity_payload(store, &id);
            let from = payload.as_ref().and_then(|p| {
                [&p.detail["summaryDTO"]["startTimeGMT"], &p.detail["startTimeGMT"],
                 &p.listed["summaryDTO"]["startTimeGMT"], &p.listed["startTimeGMT"]]
                    .iter().find_map(|v| utc_millis(v))
            }).or_else(|| offset.and_then(|zone| NaiveDateTime::parse_from_str(&a.start_local, "%Y-%m-%dT%H:%M:%S%.f")
                .ok().map(|t| t.and_utc().timestamp_millis()-zone)));
            if let Some(range) = from.and_then(|from| clip_elapsed(from, from + (a.duration_s * 1000.) as i64, start, duration_min)) {
                activities.push(range);
            }
        }
    }
    HeartRateDay {
        date: date.into(),
        resting: num(&raw, &[&["restingHeartRate"]]).filter(|n| *n > 0.),
        minimum: num(&raw, &[&["minHeartRate"]]).filter(|n| *n > 0.),
        maximum: num(&raw, &[&["maxHeartRate"]]).filter(|n| *n > 0.),
        timeline,
        duration_min,
        sleep: sleep_intervals,
        activities,
    }
}
pub fn stress(store: &Store, date: &str) -> StressCard {
    let raw = store.get("stress", date).unwrap_or(Value::Null);
    let summary = store.get("summary", date).unwrap_or(Value::Null);
    let average = stress_average(store, date);
    let level = average.map(stress_level);
    let qualifier =
        string(&summary, &[&["stressQualifier"]]).filter(|q| !q.is_empty() && q != "UNKNOWN");
    let tone = match qualifier.as_deref() {
        Some("CALM" | "BALANCED") => Tone::Good,
        Some("STRESSFUL") => Tone::Fair,
        Some("VERY_STRESSFUL") => Tone::Poor,
        _ => match level {
            Some(StressLevel::Rest | StressLevel::Low) => Tone::Good,
            Some(StressLevel::Medium) => Tone::Fair,
            Some(StressLevel::High) => Tone::Poor,
            None => Tone::Neutral,
        },
    };
    let (timeline, active) = stress_timeline(&raw);
    let midnight = NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .ok()
        .and_then(|d| d.and_hms_opt(0, 0, 0));
    let minute = |field: &str| {
        let time = local_time(sleep(&store.get("sleep", date)?), &[field])?;
        let time = NaiveDateTime::parse_from_str(&time, "%Y-%m-%dT%H:%M:%S%.f").ok()?;
        Some((time - midnight?).num_minutes() as f64)
    };
    let asleep = minute("sleepStartTimestampLocal")
        .zip(minute("sleepEndTimestampLocal"))
        .filter(|(_, end)| *end > 0.)
        .map(|(start, end)| [start.max(0.), end.min(1440.)]);
    let time: Vec<StressTime> = [
        (StressLevel::Rest, "restStressDuration"),
        (StressLevel::Low, "lowStressDuration"),
        (StressLevel::Medium, "mediumStressDuration"),
        (StressLevel::High, "highStressDuration"),
    ]
    .iter()
    .filter_map(|(level, field)| {
        num(&summary, &[&[*field]])
            .filter(|s| *s >= 0.)
            .map(|seconds| StressTime {
                level: *level,
                seconds,
            })
    })
    .collect();
    StressCard {
        average,
        max: num(&raw, &[&["maxStressLevel"]])
            .or_else(|| num(&summary, &[&["maxStressLevel"]]))
            .filter(|v| (0. ..=100.).contains(v)),
        level,
        qualifier: qualifier.map(|q| label(Some(q))),
        tone,
        timeline,
        active,
        sleep: asleep,
        active_s: num(&summary, &[&["activityStressDuration"]]).filter(|s| *s >= 0.),
        time,
        trend: (0..30)
            .rev()
            .map(|i| stress_average(store, &date_sub(date, i)))
            .collect(),
    }
}
fn has_night(store: &Store, date: &str) -> bool {
    store
        .get("readiness", date)
        .is_some_and(|r| current_rscore(&r).is_some())
        || store.get("sleep", date).is_some_and(|s| recorded_sleep(&s))
}
/// The day the screens show. Garmin files a night under the day you wake, so
/// the calendar day is empty until you wake and the watch syncs; until then
/// the latest earlier night stays current up to its handover (three hours
/// after its credible expected wake, otherwise noon). A midnight waker's
/// filing can skip a date, so two days back are checked.
pub fn current_date(store: &Store, now: NaiveDateTime) -> NaiveDate {
    let today = now.date();
    if has_night(store, &today.to_string()) {
        return today;
    }
    for back in 1..=2 {
        let date = today - Duration::days(back);
        let key = date.to_string();
        if has_night(store, &key) {
            let usual = rules::wake_clock(&wake_hours(&recent_nights(store, &key)));
            return if now < phase::handover(date, recorded_wake(store, &key), usual.as_ref()) {
                date
            } else {
                today
            };
        }
    }
    today
}
/// The recorded wake of the night filed under `date`, when it has synced.
fn recorded_wake(store: &Store, date: &str) -> Option<NaiveDateTime> {
    let raw = sleep_record(store, date).filter(recorded_sleep)?;
    local_time(sleep(&raw), &["sleepEndTimestampLocal"]).and_then(|t| parse_local(&t))
}
pub(crate) fn watch_info(store: &Store) -> (Value, Option<DateTime<Utc>>) {
    let list = store.get("devices", "me").unwrap_or(Value::Null);
    let watch = selected_watch(&list).cloned().unwrap_or(Value::Null);
    let detail = store
        .get("device_detail", "selected")
        .unwrap_or(Value::Null);
    let matching = watch["deviceId"]
        .as_u64()
        .is_some_and(|id| detail["deviceId"].as_u64() == Some(id));
    let uploaded = matching
        .then(|| detail["lastUploadTimestamp"].as_i64())
        .flatten()
        .and_then(DateTime::from_timestamp_millis);
    (watch, uploaded)
}

fn missing_metric_reasons(
    watch: &Value,
    s: &Value,
    r: &Value,
    h: &Value,
) -> BTreeMap<String, String> {
    let mut reasons = BTreeMap::new();
    for (key, field, missing, reason) in [
        (
            "sleep",
            "sleepScoreCapable",
            sscore(s).is_none(),
            "Your watch doesn't record sleep scores",
        ),
        (
            "readiness",
            "trainingReadinessCapable",
            current_rscore(r).is_none(),
            "Your watch doesn't record training readiness",
        ),
        (
            "hrv",
            "hrvStatusCapable",
            num(hrv(h), &[&["lastNightAvg"]]).is_none(),
            "Your watch doesn't record HRV status",
        ),
        (
            "skinTemp",
            "sleepSkinTemperatureCapable",
            night_value(s, Night::SkinTemp).is_none(),
            "Your watch doesn't record skin temperature",
        ),
    ] {
        if missing && watch[field].as_bool() == Some(false) {
            reasons.insert(key.into(), reason.into());
        }
    }
    reasons
}

/// Missing sleep alone is not evidence of a missed upload. Require yesterday's
/// recorded wake, a known old upload, and a mid-morning clock on today's date.
fn sleep_upload_stale(
    store: &Store,
    date: &str,
    now: DateTime<Local>,
    uploaded: Option<DateTime<Utc>>,
) -> bool {
    if date != now.date_naive().to_string()
        || now.hour() < 10
        || store
            .get("sleep", date)
            .is_some_and(|s| num(sleep(&s), &[&["sleepTimeSeconds"]]).is_some_and(|n| n > 0.))
    {
        return false;
    }
    let Some(uploaded) = uploaded else {
        return false;
    };
    let yesterday = store
        .get("sleep", &date_sub(date, 1))
        .unwrap_or(Value::Null);
    let Some(wake) = local_time(sleep(&yesterday), &["sleepEndTimestampLocal"])
        .and_then(|s| NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S%.f").ok())
    else {
        return false;
    };
    let expected = now.date_naive().and_time(wake.time());
    // A yesterday-only upload cannot explain today's missing night. Compare
    // local wall-clock timestamps in the phone's timezone, including DST.
    now.naive_local() >= expected && uploaded.with_timezone(&Local).naive_local() < expected
}

#[cfg(test)]
mod watch_tests {
    use super::*;
    use chrono::TimeZone;
    use serde_json::json;

    #[test]
    fn absent_sleep_only_signals_stale_after_expected_wake_and_old_upload() {
        let store = Store::open(":memory:").unwrap();
        let now = Local
            .with_ymd_and_hms(2025, 2, 4, 11, 0, 0)
            .single()
            .unwrap();
        let yesterday = (now.date_naive() - Duration::days(1)).to_string();
        let date = now.date_naive().to_string();
        store
            .put(
                "sleep",
                &yesterday,
                "synthetic",
                &json!({
                    "dailySleepDTO": {"sleepEndTimestampLocal": format!("{yesterday}T07:20:00")}
                }),
            )
            .unwrap();
        let local = |hour| {
            Local
                .with_ymd_and_hms(2025, 2, 4, hour, 0, 0)
                .single()
                .unwrap()
                .with_timezone(&Utc)
        };
        assert!(sleep_upload_stale(&store, &date, now, Some(local(6))));
        assert!(!sleep_upload_stale(&store, &date, now, Some(local(8))));
        assert!(!sleep_upload_stale(&store, &date, now, None));
        assert!(!sleep_upload_stale(
            &store,
            &date,
            now - Duration::hours(2),
            Some(local(6))
        ));
        store
            .put(
                "sleep",
                &date,
                "synthetic",
                &json!({"dailySleepDTO":{"sleepTimeSeconds":27000}}),
            )
            .unwrap();
        assert!(!sleep_upload_stale(&store, &date, now, Some(local(6))));
    }
}

pub fn today(store: &Store, date: &str) -> TodayView {
    today_at(store, date, Local::now().fixed_offset())
}
/// Today as of `now`, which sets the time-of-day phase.
pub fn today_at(store: &Store, date: &str, now: DateTime<FixedOffset>) -> TodayView {
    let calendar = NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap_or(now.date_naive());
    today_at_calendar(store, date, now, calendar)
}
/// Keep the sleep/readiness display day separate from the actual local calendar day.
/// Explicit historical reads use `today_at`, which anchors both to `date`.
pub fn today_at_calendar(store: &Store, date: &str, now: DateTime<FixedOffset>, calendar: NaiveDate) -> TodayView {
    let r = store.get("readiness", date).unwrap_or(Value::Null);
    let s = store.get("sleep", date).unwrap_or(Value::Null);
    let t = store.get("training", date).unwrap_or(Value::Null);
    let h = store.get("hrv", date).unwrap_or(Value::Null);
    let hs = hrv(&h);
    let (watch, uploaded) = watch_info(store);
    let missing_metric_reasons = missing_metric_reasons(&watch, &s, &r, &h);
    // Before noon Today can still show yesterday's recorded night; check the
    // current calendar day's missing upload rather than the displayed day.
    let local_now = now.with_timezone(&Local);
    let watch_sleep_upload_stale = sleep_upload_stale(
        store,
        &local_now.date_naive().to_string(),
        local_now,
        uploaded,
    );
    let rl = string(readiness(&r), &[&["level"], &["trainingReadinessLevel"]]);
    let ratio = ratio(&t).map(|v| if v > 10. { v / 100. } else { v });
    let call = rl.as_deref().map(|s| rules::effort(s, ratio));
    let factors = [
        (
            "sleep",
            "Sleep",
            "sleepScoreFactorPercent",
            "sleepScoreFactorFeedback",
        ),
        (
            "recovery",
            "Recovery time",
            "recoveryTimeFactorPercent",
            "recoveryTimeFactorFeedback",
        ),
        ("hrv", "HRV", "hrvFactorPercent", "hrvFactorFeedback"),
        (
            "acuteLoad",
            "Acute load",
            "acwrFactorPercent",
            "acwrFactorFeedback",
        ),
        (
            "sleepHistory",
            "Sleep history",
            "sleepHistoryFactorPercent",
            "sleepHistoryFactorFeedback",
        ),
        (
            "stressHistory",
            "Stress history",
            "stressHistoryFactorPercent",
            "stressHistoryFactorFeedback",
        ),
    ];
    let mut factors: Vec<Factor> = factors
        .iter()
        .filter_map(|(key, name, field, feedback_field)| {
            let snapshot = readiness(&r);
            let feedback = snapshot[*feedback_field].as_str().filter(|s| !s.is_empty());
            // Garmin sends NONE with a 0% when it couldn't rate a factor
            // (no valid sleep, too little stress data): a gap, not a zero.
            let unrated = feedback.is_some_and(|f| f.eq_ignore_ascii_case("NONE"));
            let feedback = feedback.filter(|_| !unrated);
            let percent = num(snapshot, &[&[*field]]).filter(|_| !unrated);
            (percent.is_some() || feedback.is_some() || unrated).then(|| Factor {
                key: (*key).into(),
                name: (*name).into(),
                percent,
                tone: if unrated {
                    Tone::Neutral
                } else {
                    factor_tone(feedback)
                },
                detail: feedback
                    .map(|s| label(Some(s.into())))
                    .or_else(|| percent.map(|p| format!("Garmin rates your {name} at {p:.0}%.")))
                    .unwrap_or_else(|| "Garmin didn't rate this today.".into()),
            })
        })
        .collect();
    factors.sort_by(|a, b| {
        a.percent
            .unwrap_or(100.)
            .total_cmp(&b.percent.unwrap_or(100.))
    });
    // The factor Garmin rates worst, its percentage breaking ties; unrated
    // factors never count.
    let biggest_drag = rules::limiting_factor(&factors).map(|f| f.name.clone());
    let mut chips = vec![];
    if let Some(v) = num(sleep(&s), &[&["sleepTimeSeconds"]]) {
        chips.push(Chip {
            icon: Icon::Moon,
            value: Some(format!(
                "{}:{:02}",
                (v / 3600.).floor() as u32,
                (v as u32 % 3600) / 60
            )),
            text: None,
            tone: sscore(&s).map(rules::score).unwrap_or(Tone::Neutral),
        });
    }
    if let Some(v) = num(hs, &[&["lastNightAvg"]]) {
        chips.push(Chip {
            icon: Icon::Hrv,
            value: Some(format!("{v:.0}")),
            text: Some("ms".into()),
            tone: rules::level(hs["status"].as_str().unwrap_or("")),
        });
    }
    let load = acute(&t);
    if let Some(v) = load {
        let (text, tone) = match band(&t) {
            Some([lo, _]) if v < lo => ("Room to build", Tone::Fair),
            Some([_, hi]) if v > hi => ("Load high", Tone::Poor),
            Some(_) => ("On track", Tone::Good),
            None => ("Load", Tone::Neutral),
        };
        chips.push(Chip {
            icon: Icon::Flame,
            value: None,
            text: Some(text.into()),
            tone,
        });
    }
    let b = store.get("battery", date).unwrap_or(Value::Null);
    let b = b.as_array().and_then(|a| a.first()).unwrap_or(&b);
    let summary = store.get("summary", date).unwrap_or(Value::Null);
    let hstatus = string(hs, &[&["status"]]);
    let plan = sleep_plan(store, date);
    let vitals = vitals(store, date);
    let nights = recent_nights(store, date);
    let recorded = recorded_sleep(&s).then(|| sleep(&s));
    let woke_at = recorded_wake(store, date);
    let mut focus = phase::focus(&phase::PhaseInput {
        now: now.naive_local(),
        date: NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap_or(now.date_naive()),
        plan: plan.as_ref(),
        wakes: &wake_hours(&nights),
        last_night: recorded.map(|s| LastNight {
            asleep_min: num(s, &[&["sleepTimeSeconds"]]).map(|t| t / 60.),
            need_min: sleep_need_minutes(&s["sleepNeed"]),
            woke: woke_at.map(|t| t.format("%H:%M").to_string()),
        }),
        woke_at,
        adaptive: store.settings().adaptive_today,
        // Garmin has sent JET_LAG for a night without recorded sleep, which
        // is a missing night, not travel.
        jet_lag: recorded
            .is_some_and(|s| s["sleepAlignment"]["status"].as_str() == Some("JET_LAG")),
        zone_shift_min: zone_shift(&nights, now),
        vitals_off: vitals.off_count,
    });
    let body_battery = BodyBatteryCard {
        // Daily stats can be more recent than the sparsely sampled report.
        now: num(&summary, &[&["bodyBatteryMostRecentValue"]])
            .or_else(|| num(b, &[&["bodyBatteryValue"], &["currentValue"]]))
            .or_else(|| battery_latest(b)),
        charged: num(b, &[&["charged"], &["chargedValue"]])
            .or_else(|| num(&summary, &[&["bodyBatteryChargedValue"]])),
        drained: num(b, &[&["drained"], &["drainedValue"]])
            .or_else(|| num(&summary, &[&["bodyBatteryDrainedValue"]])),
        timeline: store.get("stress", date).map(|v| battery_timeline(&v)).unwrap_or_default(),
    };
    let stress = stress(store, date);
    let activities = activities(store, date);
    // "Right now" needs something synced today to show.
    if body_battery.now.is_none()
        && stress.average.is_none()
        && !activities.iter().any(|a| a.start_local.starts_with(date))
    {
        focus.cards.retain(|c| *c != TodayCard::Now);
    }
    focus.recovery_hours = num(readiness(&r), &[&["recoveryTime"]])
        .filter(|m| m.is_finite() && *m > 0.)
        .map(|m| m / 60.);
    let headline = phase::headline(
        &focus,
        plan.as_ref(),
        match call {
            Some(Effort::Push) => "Ready to push",
            Some(Effort::Steady) => "Room to build",
            Some(Effort::Easy) | Some(Effort::Rest) => "Take it easy",
            None => "Waiting for readiness",
        },
    );
    let mut view = TodayView {
        date: date.into(),
        watch_last_upload: uploaded.map(|t| t.to_rfc3339_opts(chrono::SecondsFormat::Millis, true)),
        watch_sleep_upload_stale,
        missing_metric_reasons,
        sleep_plan: plan,
        readiness_trend: (0..30)
            .rev()
            .map(|i| {
                store.get("readiness", &date_sub(date, i)).and_then(|v| {
                    if i == 0 {
                        current_rscore(&v)
                    } else {
                        rscore(&v)
                    }
                })
            })
            .collect(),
        readiness: current_rscore(&r).map(|v| {
            gauge(
                v,
                100.,
                &label(rl.clone()),
                rl.as_deref().map(rules::level).unwrap_or(Tone::Neutral),
                None,
            )
        }),
        sleep: sscore(&s).map(|v| {
            gauge(
                v,
                100.,
                if v >= 80. {
                    "Good"
                } else if v >= 60. {
                    "Fair"
                } else {
                    "Poor"
                },
                rules::score(v),
                None,
            )
        }),
        load: load.map(|v| {
            gauge(
                v,
                (v * 1.3).max(band(&t).map(|b| b[1] * 1.2).unwrap_or(1000.)),
                "Acute load",
                match band(&t) {
                    Some([lo, hi]) if v >= lo && v <= hi => Tone::Good,
                    Some([_, hi]) if v > hi => Tone::Poor,
                    Some(_) => Tone::Fair,
                    None => Tone::Neutral,
                },
                band(&t),
            )
        }),
        headline,
        chips,
        factors,
        readiness_why: None,
        body_battery_why: None,
        reconcile: None,
        call,
        // A finished workout comes first, then Garmin's own readiness feedback
        // (sent as codes such as LET_YOUR_BODY_RECOVER), then the biggest drag.
        call_note: activities
            .first()
            .filter(|a| a.start_local.starts_with(date))
            .map(|a| {
                format!(
                    "{} done · {} load",
                    a.name,
                    a.load
                        .map(|v| format!("{v:.0}"))
                        .unwrap_or_else(|| "—".into())
                )
            })
            .or_else(|| {
                string(
                    readiness(&r),
                    &[&["feedbackShort"], &["feedbackLong"], &["feedbackPhrase"]],
                )
                .and_then(|code| humanize_code(&code))
            })
            .or_else(|| biggest_drag.map(|d| format!("{d} is your biggest drag today."))),
        body_battery,
        stress,
        heart_rate_day: heart_rate_day(store, date),
        hrv: HrvCard {
            last_night: num(hs, &[&["lastNightAvg"]]),
            status: hstatus.clone().map(|s| label(Some(s))),
            tone: hstatus
                .as_deref()
                .map(rules::level)
                .unwrap_or(Tone::Neutral),
            baseline: vitals
                .readings
                .iter()
                .find(|v| v.kind == VitalKind::Hrv)
                .and_then(|v| v.normal),
            week: (0..7)
                .rev()
                .map(|i| vital_value(store, VitalKind::Hrv, &date_sub(date, i)))
                .collect(),
        },
        vitals,
        activities,
        fitness_age: age(store, date),
        checkin_ready: NaiveDate::parse_from_str(date, "%Y-%m-%d")
            .is_ok_and(|d| d.weekday() == chrono::Weekday::Mon)
            && store.get("summary", &date_sub(date, 1)).is_some(),
        focus,
        planned: planned::session(store, &calendar.to_string(), now.with_timezone(&Utc)),
        plan: plan::this_week(store, date),
        lead: None,
        adapt: None,
        day: None,
    };
    view.readiness_why = explain::readiness_why(&r, &view.factors, rl.as_deref());
    view.body_battery_why =
        explain::body_battery_why(b, &summary, view.body_battery.now, &view.stress);
    view.reconcile = explain::reconcile(explain::Scores {
        readiness: view.readiness.as_ref(),
        level: rl.as_deref(),
        factors: &view.factors,
        sleep: view.sleep.as_ref(),
        battery_at_wake: num(&summary, &[&["bodyBatteryAtWakeTime"]])
            .filter(|v| (0. ..=100.).contains(v)),
        hrv_status: hstatus.as_deref(),
        training_status: status(&t).as_deref(),
    });
    view
}
pub fn sleep_view(store: &Store, date: &str) -> SleepView {
    let raw = store.get("sleep", date).unwrap_or(Value::Null);
    let v = sleep(&raw);
    let score = sscore(&raw);
    let asleep = num(v, &[&["sleepTimeSeconds"]]).map(|s| s / 60.);
    let need = sleep_need(&raw);
    let (naps, naps_state) = crate::naps_journal::naps(store, date);
    let stage_history: Vec<Value> = (0..30)
        .filter_map(|i| store.get("sleep", &date_sub(date, i)))
        .collect();
    let nights: Vec<(String, Value)> = (0..14)
        .rev()
        .filter_map(|i| {
            let day = date_sub(date, i);
            store.get("sleep", &day).map(|v| (day, v))
        })
        .collect();
    let windows: Vec<NightWindow> = nights
        .iter()
        .filter_map(|(day, v)| {
            let s = sleep(v);
            Some(NightWindow {
                date: day.clone(),
                bed_hour: hour(&local_time(s, &["sleepStartTimestampLocal"])?)?,
                wake_hour: hour(&local_time(s, &["sleepEndTimestampLocal"])?)?,
            })
        })
        .collect();
    let wakes: Vec<f64> = windows.iter().map(|w| w.wake_hour).collect();
    let headline = match (asleep, need) {
        (Some(a), Some(n)) if a >= n => format!("{:.0} min over your need", a - n),
        (Some(a), Some(n)) => format!("{:.0} min short of your need", n - a),
        _ => "Sleep at a glance".into(),
    };
    SleepView {
        date: date.into(),
        naps,
        naps_state,
        start_local: local_time(v, &["sleepStartTimestampLocal"]),
        end_local: local_time(v, &["sleepEndTimestampLocal"]),
        score: score.map(|v| {
            gauge(
                v,
                100.,
                if v >= 80. {
                    "Good"
                } else if v >= 60. {
                    "Fair"
                } else {
                    "Poor"
                },
                rules::score(v),
                None,
            )
        }),
        asleep_min: asleep,
        need_min: need,
        headline,
        stages: raw["sleepLevels"]
            .as_array()
            .map(|levels| {
                levels
                    .iter()
                    .filter_map(|item| {
                        let stage = match item["activityLevel"].as_f64()? as u32 {
                            0 => Stage::Deep,
                            1 => Stage::Light,
                            2 => Stage::Rem,
                            3 => Stage::Awake,
                            _ => return None,
                        };
                        let start = sleep_stage_time(item, "startGMT", "startTimestamp")?;
                        let end = sleep_stage_time(item, "endGMT", "endTimestamp")?;
                        let first =
                            sleep_stage_time(levels.first()?, "startGMT", "startTimestamp")?;
                        let divisor = 60_000.;
                        Some(StageSegment {
                            stage,
                            start_min: (start - first) / divisor,
                            minutes: (end - start) / divisor,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default(),
        totals: [
            (Stage::Awake, "awakeSleepSeconds"),
            (Stage::Rem, "remSleepSeconds"),
            (Stage::Light, "lightSleepSeconds"),
            (Stage::Deep, "deepSleepSeconds"),
        ]
        .iter()
        .filter_map(|(stage, key)| {
            num(v, &[&[*key]]).map(|s| {
                let history: Vec<f64> = stage_history
                    .iter()
                    .filter_map(|night| num(sleep(night), &[&[*key]]))
                    .collect();
                let average = (!history.is_empty())
                    .then(|| history.iter().sum::<f64>() / history.len() as f64);
                StageTotal {
                    stage: *stage,
                    minutes: s / 60.,
                    usual_delta_min: average.map(|avg| (s - avg) / 60.),
                }
            })
        })
        .collect(),
        overnight: [
            (
                OvernightKind::Hrv,
                "hrv",
                "ms",
                num(
                    &store.get("hrv", date).unwrap_or(Value::Null),
                    &[&["hrvSummary", "lastNightAvg"]],
                ),
            ),
            (
                OvernightKind::HeartRate,
                "heart rate",
                "bpm",
                night_value(v, Night::HeartRate),
            ),
            (
                OvernightKind::Respiration,
                "respiration",
                "br/min",
                night_value(v, Night::Respiration),
            ),
            (
                OvernightKind::Spo2,
                "SpO2",
                "%",
                night_value(v, Night::Spo2),
            ),
        ]
        .iter()
        .filter_map(|(kind, _, unit, value)| {
            value.map(|n| OvernightTile {
                kind: *kind,
                value: Some(n),
                unit: (*unit).into(),
                series: overnight_samples(&raw, *kind),
            })
        })
        .collect(),
        consistency_spread_min: rules::normal(
            &windows
                .iter()
                .map(|w| (w.bed_hour + if w.bed_hour < 12. { 24. } else { 0. }) * 60.)
                .collect::<Vec<_>>(),
        )
        .map(|r| (r[1] - r[0]) / 2.),
        bed_window: rules::normal(
            &windows
                .iter()
                .map(|w| {
                    if w.bed_hour < 12. {
                        w.bed_hour + 24.
                    } else {
                        w.bed_hour
                    }
                })
                .collect::<Vec<_>>(),
        )
        .map(|r| [r[0] % 24., r[1] % 24.]),
        wake_window: rules::normal(&wakes),
        consistency: windows,
        garmin_averages: garmin_averages(store, date),
        alignment: alignment(store, date),
        sleep_plan: sleep_plan(store, date),
        recent: (0..7)
            .rev()
            .map(|i| {
                let day = date_sub(date, i);
                let raw = store.get("sleep", &day);
                NightNeed {
                    asleep_min: raw.as_ref().and_then(|r| num(sleep(r), &[&["sleepTimeSeconds"]])).map(|s| s / 60.),
                    need_min: raw.as_ref().and_then(sleep_need),
                    date: day,
                }
            })
            .collect(),
        tonight_first: false,
        wound_down: false,
    }
}
fn hour(s: &str) -> Option<f64> {
    let time = s.split('T').nth(1).or_else(|| s.split(' ').nth(1))?;
    let mut x = time.split(':');
    Some(x.next()?.parse::<f64>().ok()? + x.next()?.parse::<f64>().ok()? / 60.)
}
pub fn training(store: &Store, date: &str) -> TrainingView {
    let calendar = NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap_or(NaiveDate::MIN);
    training_at_calendar(store, date, calendar, Utc::now())
}
/// Historical Training uses its requested date; live Training receives the
/// calendar day independently of the displayed sleep/readiness day.
pub fn training_at_calendar(store: &Store, date: &str, calendar: NaiveDate, now: DateTime<Utc>) -> TrainingView {
    let v = store.get("training", date).unwrap_or(Value::Null);
    let load = acute(&v);
    let status = status(&v);
    let workouts = activities(store, &date_sub(date, 30));
    let strength = crate::strength::week(
        store,
        date,
        &workouts
            .iter()
            .filter(|a| a.icon == Icon::Strength)
            .map(|a| a.start_local.clone())
            .collect::<Vec<_>>(),
    );
    let week = (0..7)
        .rev()
        .filter_map(|i| {
            let date = date_sub(date, i);
            store
                .get("training_daily", &date)
                .and_then(|v| num(&v, &[&["dailyTrainingLoad"]]))
                .or_else(|| {
                    store
                        .get("training", &date)
                        .and_then(|v| num(&v, &[&["dailyTrainingLoad"]]))
                })
                .map(|load| DayLoad { date, load })
        })
        .collect();
    let m = store.get("max_metrics", date).unwrap_or(Value::Null);
    let focus = store.get("load_focus", date).unwrap_or(Value::Null);
    let load_focus = [
        (
            "lowAerobic",
            "Low aerobic",
            "monthlyLoadAerobicLow",
            "monthlyLoadAerobicLowTargetMin",
            "monthlyLoadAerobicLowTargetMax",
        ),
        (
            "highAerobic",
            "High aerobic",
            "monthlyLoadAerobicHigh",
            "monthlyLoadAerobicHighTargetMin",
            "monthlyLoadAerobicHighTargetMax",
        ),
        (
            "anaerobic",
            "Anaerobic",
            "monthlyLoadAnaerobic",
            "monthlyLoadAnaerobicTargetMin",
            "monthlyLoadAnaerobicTargetMax",
        ),
    ]
    .iter()
    .filter_map(|(key, name, value_key, min_key, max_key)| {
        let nested = load_device(&focus);
        let data = if nested[*key].is_object() {
            &nested[*key]
        } else {
            &focus[*key]
        };
        num(nested, &[&[*value_key]])
            .or_else(|| num(data, &[&["value"], &["load"]]))
            .map(|value| {
                let target = match (
                    num(nested, &[&[*min_key]]).or_else(|| num(data, &[&["optimalMin"]])),
                    num(nested, &[&[*max_key]]).or_else(|| num(data, &[&["optimalMax"]])),
                ) {
                    (Some(a), Some(b)) => Some([a, b]),
                    _ => None,
                };
                LoadFocusBar {
                    key: (*key).into(),
                    name: (*name).into(),
                    value,
                    target,
                    shortage: target.is_some_and(|t| value < t[0]),
                }
            })
    })
    .collect();
    let vo2max = vo2(&m).or_else(|| vo2(&v["mostRecentVO2Max"]));
    let vo2max_trend: Vec<Option<f64>> = (0..5)
        .rev()
        .map(|week| historical_vo2(store, &date_sub(date, week * 7)))
        .collect();
    let vo2max_change_30d = vo2max.zip(vo2max_trend[0]).map(|(a, b)| a - b);
    let races_raw = store.get("races", date).unwrap_or(Value::Null);
    let races = [
        ("5K", "time5K"),
        ("10K", "time10K"),
        ("Half", "timeHalfMarathon"),
        ("Full", "timeMarathon"),
    ]
    .iter()
    .filter_map(|(label, key)| {
        num(&races_raw, &[&[*key]]).map(|seconds| RacePrediction {
            label: (*label).into(),
            seconds,
        })
    })
    .collect();
    let (race_state, race) = planned::race(store, &calendar.to_string(), now);
    let mut view = TrainingView {
        acute: load.map(|amount| {
            gauge(
                amount,
                (amount * 1.3).max(band(&v).map(|b| b[1] * 1.2).unwrap_or(1000.)),
                match band(&v) {
                    Some([lo, _]) if amount < lo => "Low",
                    Some([_, hi]) if amount > hi => "High",
                    Some(_) => "Optimal",
                    None => "",
                },
                match band(&v) {
                    Some([lo, hi]) if amount >= lo && amount <= hi => Tone::Good,
                    Some([_, hi]) if amount > hi => Tone::Poor,
                    Some(_) => Tone::Fair,
                    None => Tone::Neutral,
                },
                band(&v),
            )
        }),
        status: status.clone().map(|s| label(Some(s))),
        status_tone: status
            .as_ref()
            .map(|s| rules::level(&label(Some(s.clone()))))
            .unwrap_or(Tone::Neutral),
        status_why: None,
        headline: match (load, band(&v)) {
            (Some(amount), Some([lo, _])) if amount < lo => "Room to build".into(),
            (Some(amount), Some([_, hi])) if amount > hi => "Load high".into(),
            (Some(_), Some(_)) => "In your optimal range".into(),
            _ => status
                .map(|s| label(Some(s)))
                .unwrap_or_else(|| "Training at a glance".into()),
        },
        chips: [
            ratio(&v).map(|ratio| Chip {
                icon: Icon::Flame,
                value: Some(format!("{ratio:.1}")),
                text: Some("Load ratio".into()),
                tone: Tone::Fair,
            }),
            num(
                readiness(&store.get("readiness", date).unwrap_or(Value::Null)),
                &[&["recoveryTime"]],
            )
            .map(|minutes| Chip {
                icon: Icon::Clock,
                value: Some(format!("{:.0}h", minutes / 60.)),
                text: Some("Recovery".into()),
                tone: Tone::Neutral,
            }),
        ]
        .into_iter()
        .flatten()
        .collect(),
        week,
        load_focus,
        vo2max,
        vo2max_change_30d,
        vo2max_trend,
        races,
        endurance_score: store
            .get("endurance", date)
            .and_then(|v| num(&v, &[&["overallScore"], &["score"]])),
        hill_score: store
            .get("hill", date)
            .and_then(|v| num(&v, &[&["overallScore"], &["score"]])),
        thresholds: thresholds::from_store(store, date),
        running_tolerance_km: store.get("tolerance", date).and_then(|v| {
            // The daily endpoint returns an array and reports acuteTolerance in metres.
            num(first(&v), &[&["acuteTolerance"]])
                .map(|metres| metres / 1000.)
                .or_else(|| num(&v, &[&["runningTolerance"], &["tolerance"]]))
        }),
        workouts,
        gear: crate::gear::view(store),
        planned: planned::session(store, &calendar.to_string(), now),
        race,
        race_state,
        strength,
        lanes: None,
    };
    view.status_why = explain::training_why(explain::TrainingInputs {
        training: &v,
        focus: &focus,
        hrv: &store.get("hrv", date).unwrap_or(Value::Null),
        status: self::status(&v).as_deref(),
        acute: view.acute.as_ref(),
        ratio: ratio(&v),
        load_focus: &view.load_focus,
        vo2max: view.vo2max,
        vo2max_change_30d: view.vo2max_change_30d,
    });
    view
}
pub fn workout(store: &Store, id: &str) -> Option<WorkoutView> {
    let summary = activity(store, id)?;
    let v = activity_payload(store, id)?;
    let details = store.get("details", id).unwrap_or(Value::Null);
    let metrics = details["metricDescriptors"].as_array();
    let samples = details["activityDetailMetrics"].as_array();
    let index = |key: &str| {
        metrics.and_then(|m| {
            m.iter()
                .enumerate()
                .find(|(_, x)| x["key"] == key)
                .map(|(position, descriptor)| {
                    descriptor["metricsIndex"]
                        .as_u64()
                        .map(|n| n as usize)
                        .unwrap_or(position)
                })
        })
    };
    // Cadence: both feet for runs and walks, rpm for rides; never single-leg.
    let stream_keys: [(StreamKind, &str, &[&str]); 8] = [
        (StreamKind::Speed, "km/h", &["directSpeed"]),
        (StreamKind::Elevation, "m", &["directElevation"]),
        (StreamKind::Cadence, if summary.icon == Icon::Bike { "rpm" } else { "spm" }, &["directDoubleCadence", "directBikeCadence"]),
        (StreamKind::Power, "W", &["directPower"]),
        (StreamKind::Respiration, "brpm", &["directRespirationRate"]),
        (StreamKind::Stamina, "%", &["directAvailableStamina"]),
        (StreamKind::PerformanceCondition, "points", &["directPerformanceCondition"]),
        (StreamKind::Temperature, "°C", &["directAirTemperature"]),
    ];
    // A stream is used only when its descriptor names the unit Raring expects.
    let unit_of = |key: &str| {
        metrics.and_then(|m| m.iter().find(|x| x["key"] == key)).and_then(|d| d["unit"]["key"].as_str())
    };
    let stream_index: Vec<(StreamKind, &str, Option<usize>)> = stream_keys
        .iter()
        .map(|(k, u, keys)| (*k, *u, keys.iter().filter(|key| stream_unit_ok(key, unit_of(key))).find_map(|key| index(key))))
        .collect();
    let mut along_t: Vec<f64> = vec![];
    let mut along_v: Vec<Vec<Option<f64>>> = vec![vec![]; stream_index.len()];
    let hr = index("directHeartRate");
    let ts = index("directTimestamp");
    let lat = index("directLatitude");
    let lon = index("directLongitude");
    let points = samples.map(|v| v.as_slice()).unwrap_or(&[]);
    let mut heart_rate = vec![];
    let mut route = vec![];
    let start_ts = points
        .iter()
        .filter_map(|p| ts.and_then(|i| p["metrics"][i].as_f64()))
        .find(|t| t.is_finite());
    // Epoch milliseconds and epoch seconds must be distinguished by their absolute
    // origin, not by elapsed time (which can be under 100 seconds).
    let timestamp_scale = if start_ts.is_some_and(|t| t.abs() >= 100_000_000_000.) {
        1000.
    } else {
        1.
    };
    let stride = points.len().div_ceil(300).max(1);
    let last = points
        .last()
        .filter(|_| !(points.len() - 1).is_multiple_of(stride));
    for p in points.iter().step_by(stride).chain(last) {
        let a = &p["metrics"];
        if let (Some(h), Some(t), Some(origin)) = (
            hr.and_then(|i| a[i].as_f64()),
            ts.and_then(|i| a[i].as_f64()),
            start_ts,
        ) {
            let elapsed = (t - origin) / timestamp_scale;
            if elapsed.is_finite()
                && elapsed >= 0.
                && h.is_finite()
                && heart_rate
                    .last()
                    .is_none_or(|prev: &HrSample| elapsed >= prev.t_s)
            {
                heart_rate.push(HrSample {
                    t_s: elapsed,
                    bpm: h,
                });
            }
        }
        if let (Some(t), Some(origin)) = (ts.and_then(|i| a[i].as_f64()), start_ts) {
            let elapsed = (t - origin) / timestamp_scale;
            if elapsed.is_finite() && elapsed >= 0. && along_t.last().is_none_or(|prev| elapsed >= *prev) {
                along_t.push(elapsed);
                for (n, (kind, _, i)) in stream_index.iter().enumerate() {
                    let raw = i.and_then(|i| a[i].as_f64()).filter(|v| v.is_finite());
                    along_v[n].push(raw.map(|v| if *kind == StreamKind::Speed { v * 3.6 } else { v }));
                }
            }
        }
        if let (Some(y), Some(x)) = (
            lat.and_then(|i| a[i].as_f64()),
            lon.and_then(|i| a[i].as_f64()),
        ) {
            route.push([y, x]);
        }
    }
    let splits = store.get("splits", id).unwrap_or(Value::Null);
    let mut stats: Vec<Stat> = [
        ("distance", "Distance", "km", 1000.),
        ("duration", "Time", "s", 1.),
        ("averageHR", "Avg HR", "bpm", 1.),
        ("maxHR", "Max HR", "bpm", 1.),
        ("calories", "Calories", "kcal", 1.),
        ("elevationGain", "Ascent", "m", 1.),
        ("averagePower", "Avg power", "W", 1.),
        ("averageBikeCadence", "Cadence", "rpm", 1.),
        ("averageRunCadence", "Cadence", "spm", 1.),
    ]
    .iter()
    .filter_map(|(key, label, unit, div)| {
        v.number(&[*key])
            .filter(|v| *key != "distance" || *v > 0.)
            .map(|v| Stat {
                key: (*key).into(),
                label: (*label).into(),
                value: v / div,
                unit: (*unit).into(),
            })
    })
    .collect();
    if summary.icon == Icon::Bike {
        if let Some(speed) = v.number(&["averageSpeed"]) {
            stats.insert(
                2.min(stats.len()),
                Stat {
                    key: "speed".into(),
                    label: "Avg speed".into(),
                    value: speed * 3.6,
                    unit: "km/h".into(),
                },
            );
        }
    } else if let Some(distance) = summary.distance_km.filter(|d| *d > 0.) {
        if summary.duration_s > 0. {
            stats.insert(
                2.min(stats.len()),
                Stat {
                    key: "pace".into(),
                    label: "Pace".into(),
                    value: summary.duration_s / distance,
                    unit: "s/km".into(),
                },
            );
        }
    }
    let series: Vec<StreamSeries> = stream_index
        .iter()
        .zip(along_v)
        .filter(|(_, values)| values.iter().any(Option::is_some))
        .map(|((kind, unit, _), values)| StreamSeries { kind: *kind, unit: (*unit).into(), values })
        .collect();
    let along = (!series.is_empty()).then_some(AlongTheWay { t_s: along_t, series });
    let more: Vec<Stat> = [
        ("movingDuration", "Moving time", "s"),
        ("moderateIntensityMinutes", "Moderate intensity", "min"),
        ("vigorousIntensityMinutes", "Vigorous intensity", "min"),
        ("differenceBodyBattery", "Body Battery", "points"),
        ("beginPotentialStamina", "Stamina at start", "%"),
        ("endPotentialStamina", "Stamina at end", "%"),
        ("elevationLoss", "Descent", "m"),
        ("normalizedPower", "Normalized power", "W"),
        ("intensityFactor", "Intensity factor", ""),
        ("maxPowerTwentyMinutes", "Best 20 min power", "W"),
        ("functionalThresholdPower", "FTP", "W"),
        ("strideLength", "Stride length", "cm"),
        ("avgRespirationRate", "Breathing rate", "brpm"),
        ("recoveryHeartRate", "Recovery heart rate", "bpm"),
        ("averageTemperature", "Watch temperature, average", "°C"),
        ("minTemperature", "Watch temperature, lowest", "°C"),
        ("maxTemperature", "Watch temperature, highest", "°C"),
    ]
    .iter()
    .filter_map(|(key, label, unit)| {
        v.number(&[*key]).filter(|n| n.is_finite()).map(|value| Stat { key: (*key).into(), label: (*label).into(), value, unit: (*unit).into() })
    })
    .collect();
    let effects: Vec<EffectLine> = [
        (EffectKind::Aerobic, summary.aerobic_te, summary.aerobic_label.clone()),
        (EffectKind::Anaerobic, summary.anaerobic_te, summary.anaerobic_label.clone()),
    ]
    .into_iter()
    .filter_map(|(kind, value, garmin_label)| {
        let value = value.filter(|v| (0. ..=5.).contains(v))?;
        let (band, line) = effect_band(kind, value);
        Some(EffectLine { kind, value, band: band.into(), line: line.into(), garmin_label })
    })
    .collect();
    let day = summary.start_local.get(..10).unwrap_or_default().to_owned();
    let vo2max_new = store
        .get("max_metrics", &day)
        .and_then(|m| vo2(&m))
        .filter(|today| historical_vo2(store, &date_sub(&day, 1)) != Some(*today));
    Some(WorkoutView {
        counted: None,
        along,
        more,
        effects,
        vo2max_new,
        summary,
        recovery_hours: v.number(&["recoveryTime"]).map(|s| s / 3600.),
        stats,
        route,
        heart_rate,
        zone_floors: store
            .get("zones", id)
            .and_then(|v| v.as_array().cloned())
            .map(|zones| {
                let mut floors: Vec<_> = zones
                    .iter()
                    .filter_map(|z| {
                        Some((
                            z["zoneNumber"].as_u64().filter(|n| (2..=5).contains(n))?,
                            num(
                                z,
                                &[&["zoneLowBoundary"], &["minHeartRate"], &["lowerBound"]],
                            )?,
                        ))
                    })
                    .collect();
                floors.sort_by_key(|(number, _)| *number);
                floors.into_iter().map(|(_, floor)| floor).collect()
            })
            .unwrap_or_default(),
        splits: splits["lapDTOs"]
            .as_array()
            .or_else(|| splits.as_array())
            .map(|a| {
                a.iter()
                    .enumerate()
                    .filter_map(|(i, v)| {
                        Some(Split {
                            index: i as u32 + 1,
                            distance_km: num(v, &[&["distance"]])? / 1000.,
                            seconds: num(v, &[&["duration"]])?,
                            avg_hr: num(v, &[&["averageHR"]]),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default(),
        strength: crate::strength::session(store, id),
    })
}
/// The unit key each stored stream must carry. Values are already in that
/// base unit: the descriptor's `factor` is not applied (checked against the
/// summary's distance, duration and elevation).
pub fn stream_unit_ok(key: &str, unit: Option<&str>) -> bool {
    let expected: &[&str] = match key {
        "directSpeed" => &["mps"],
        "directElevation" => &["meter"],
        "directDoubleCadence" => &["stepsPerMinute"],
        "directBikeCadence" => &["rpm"],
        "directPower" => &["watt"],
        "directRespirationRate" => &["breathsPerMinute"],
        "directAvailableStamina" | "directPerformanceCondition" => &["dimensionless"],
        // Garmin spells it "celcius".
        "directAirTemperature" => &["celcius", "celsius"],
        _ => &[],
    };
    unit.is_some_and(|u| expected.contains(&u))
}
/// Garmin's training-effect bands (0–0.9, 1–1.9, 2–2.9, 3–3.9, 4–4.9, 5)
/// and one plain line for each, by kind.
pub fn effect_band(kind: EffectKind, v: f64) -> (&'static str, &'static str) {
    let band = if v >= 5. {
        "Overreaching"
    } else if v >= 4. {
        "Highly improving"
    } else if v >= 3. {
        "Improving"
    } else if v >= 2. {
        "Maintaining"
    } else if v >= 1. {
        "Minor benefit"
    } else {
        "No benefit"
    };
    let line = match (kind, v) {
        (_, v) if v >= 5. => "more than your body can absorb right away; recovery comes first",
        (EffectKind::Aerobic, v) if v >= 3. => "builds your aerobic base",
        (EffectKind::Aerobic, v) if v >= 2. => "keeps your aerobic base",
        (EffectKind::Aerobic, _) => "little aerobic stimulus",
        (EffectKind::Anaerobic, v) if v >= 3. => "builds capacity for hard, fast efforts",
        (EffectKind::Anaerobic, v) if v >= 2. => "keeps some capacity for hard efforts",
        (EffectKind::Anaerobic, _) => "little sprint or hard work",
    };
    (band, line)
}
/// Strength history; none without any strength activity with sets.
pub fn strength(store: &Store, date: &str) -> Option<StrengthView> {
    let since = crate::strength::history_start(store, date)?;
    let before = NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .ok()?
        .succ_opt()?;
    let workouts = store
        .strength_activity_ids(&since.to_string(), &before.to_string())
        .iter()
        .filter_map(|id| activity(store, id))
        .collect();
    crate::strength::history(store, date, workouts)
}
fn body_weight(store: &Store, date: &str) -> Option<BodyWeightCard> {
    let end = NaiveDate::parse_from_str(date, "%Y-%m-%d").ok()?;
    let start = end - Duration::days(83);
    let rows = store.history("weight", date, 1000);
    let latest = store.get("weight_latest", "me").unwrap_or(Value::Null);
    let latest_day = latest["date"]
        .as_i64()
        .and_then(DateTime::from_timestamp_millis)
        .map(|time| time.date_naive());
    let latest_row = latest_day
        .filter(|d| *d <= end)
        .map(|d| (d.to_string(), &latest));
    let mut entries: BTreeMap<String, &Value> = rows.iter().map(|(d, v)| (d.clone(), v)).collect();
    if let Some((day, row)) = latest_row {
        let older = entries
            .get(&day)
            .and_then(|v| v["timestampGMT"].as_i64())
            .unwrap_or(0);
        if row["timestampGMT"].as_i64().unwrap_or(0) >= older {
            entries.insert(day, row);
        }
    }
    let (measured_on, current) = entries
        .iter()
        .rev()
        .find(|(_, row)| num(row, &[&["weight"]]).is_some_and(|v| v.is_finite() && v > 0.))?;
    // Do not assume metric while the user's measurement setting is still unknown.
    let settings = store.get("profile_settings", "me")?;
    let system = settings["measurementSystem"].as_str()?;
    if system != "metric" && !system.starts_with("statute") { return None; }
    let unit = crate::strength::weight_unit(store);
    let value = crate::strength::weight(num(current, &[&["weight"]]), unit)?;
    let mut trend = vec![None; 12];
    for (day, row) in &entries {
        let Some(day) = NaiveDate::parse_from_str(day, "%Y-%m-%d").ok() else {
            continue;
        };
        if day < start || day > end {
            continue;
        }
        let index = ((day - start).num_days() / 7) as usize;
        if let Some(v) = crate::strength::weight(num(row, &[&["weight"]]).filter(|n| n.is_finite() && *n > 0.), unit) {
            trend[index] = Some(v);
        }
    }
    Some(BodyWeightCard {
        value,
        unit,
        measured_on: measured_on.clone(),
        bmi: num(current, &[&["bmi"]]).filter(|n| n.is_finite() && *n > 0.),
        body_fat: num(current, &[&["bodyFat"]])
            .filter(|n| n.is_finite() && (0. ..=100.).contains(n)),
        trend,
    })
}

pub fn health(store: &Store, date: &str) -> HealthView {
    health_at_calendar(store, date, date)
}
/// Live Health retains its sleep-day vitals while the private journal follows
/// the actual local calendar day, as planned Training does.
pub fn health_at_calendar(store: &Store, date: &str, calendar: &str) -> HealthView {
    let body_weight = body_weight(store, date);
    let v = store.get("fitness_age", date).unwrap_or(Value::Null);
    let drivers: Vec<Driver> = [
        ("vo2max", "VO2max", "ml/kg/min"),
        ("restingHr", "Resting HR", "bpm"),
        ("bmi", "BMI", ""),
        ("bodyFat", "Body fat", "%"),
        ("vigorousMinutes", "Vigorous minutes", "min"),
        ("vigorousDays", "Vigorous days", "days/week"),
    ]
    .iter()
    .filter_map(|(key, name, unit)| {
        let component = match *key {
            "restingHr" => "rhr",
            "vigorousMinutes" => "vigorousMinutesAvg",
            "vigorousDays" => "vigorousDaysAvg",
            _ => key,
        };
        let data = if v["components"][component].is_object() {
            &v["components"][component]
        } else {
            &v["fitnessAgeData"][*key]
        };
        let value = num(data, &[&["value"]])
            .filter(|n| n.is_finite())
            .or_else(|| {
                // A weigh-in can supply the displayed driver, not Garmin's target or age estimate.
                // An old reading stays on the dated tile, not a current driver.
                let recent = body_weight.as_ref().filter(|w| {
                    NaiveDate::parse_from_str(date, "%Y-%m-%d").ok()
                        .zip(NaiveDate::parse_from_str(&w.measured_on, "%Y-%m-%d").ok())
                        .is_some_and(|(today, measured)| (0..=30).contains(&(today - measured).num_days()))
                });
                match *key {
                    "bmi" => recent.and_then(|w| w.bmi),
                    "bodyFat" => recent.and_then(|w| w.body_fat),
                    _ => None,
                }
            });
        value.map(|value| {
            let target =
                num(data, &[&["targetValue"], &["target"]]).filter(|n| n.is_finite() && *n > 0.);
            let met = target.is_some_and(|t| {
                if matches!(*key, "restingHr" | "bmi" | "bodyFat") {
                    value <= t
                } else {
                    value >= t
                }
            });
            Driver {
                key: (*key).into(),
                name: (*name).into(),
                value: Some(value),
                unit: (*unit).into(),
                target,
                met,
                tone: if target.is_none() {
                    Tone::Neutral
                } else if met {
                    Tone::Good
                } else {
                    Tone::Fair
                },
            }
        })
    })
    .collect();
    let lever = drivers
        .iter()
        .filter_map(|d: &Driver| {
            let (Some(current), Some(target)) = (d.value, d.target) else {
                return None;
            };
            if target <= 0. || d.met {
                return None;
            }
            let gap = if matches!(d.key.as_str(), "restingHr" | "bmi" | "bodyFat") {
                (current - target) / target
            } else {
                (target - current) / target
            };
            if !gap.is_finite() || gap <= 0. {
                return None;
            }
            let component = match d.key.as_str() {
                "restingHr" => "rhr",
                "vigorousMinutes" => "vigorousMinutesAvg",
                "vigorousDays" => "vigorousDaysAvg",
                key => key,
            };
            Some((
                gap,
                Lever {
                    key: d.key.clone(),
                    name: d.name.clone(),
                    current,
                    target,
                    unit: d.unit.clone(),
                    age_if_met: num(&v["components"][component], &[&["potentialAge"]])
                        .or_else(|| num(&v["fitnessAgeData"][&d.key], &[&["ageIfMet"]])),
                },
            ))
        })
        .max_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, lever)| lever);
    let journal_enabled = store.settings().journal_enabled;
    HealthView {
        date: date.into(),
        journal_enabled,
        journal: if journal_enabled { crate::naps_journal::journal(store, calendar) } else { vec![] },
        fitness_age: age(store, date),
        lever,
        drivers,
        vitals: vitals(store, date),
        stress: stress(store, date),
        heart_rate_day: heart_rate_day(store, date),
        body_weight,
        intensity: NaiveDate::parse_from_str(date, "%Y-%m-%d").ok().and_then(|d| {
            plan::intensity(store, plan::week_start(d, plan::week_starts_on(store)), d)
        }),
    }
}
pub fn checkin(store: &Store, monday: &str) -> CheckinView {
    let previous = date_sub(monday, 7);
    let mut cards = vec![];
    for (area, source, measure, unit) in [
        (CheckinArea::Readiness, "readiness", 0, ""),
        (CheckinArea::Sleep, "sleep", 1, ""),
        (CheckinArea::Training, "training", 2, "load"),
        (CheckinArea::Health, "fitness_age", 3, "years"),
    ] {
        let series: Vec<Option<f64>> = (1..=7)
            .rev()
            .map(|i| {
                let d = date_sub(monday, i);
                store.get(source, &d).and_then(|v| match measure {
                    0 => rscore(&v),
                    1 => sscore(&v),
                    2 => acute(&v),
                    _ => num(&v, &[&["fitnessAge"]]),
                })
            })
            .collect();
        let before: Vec<f64> = (8..=14)
            .rev()
            .filter_map(|i| {
                let d = date_sub(monday, i);
                store.get(source, &d).and_then(|v| match measure {
                    0 => rscore(&v),
                    1 => sscore(&v),
                    2 => acute(&v),
                    _ => num(&v, &[&["fitnessAge"]]),
                })
            })
            .collect();
        let current: Vec<f64> = series.iter().flatten().copied().collect();
        let value =
            (!current.is_empty()).then(|| current.iter().sum::<f64>() / current.len() as f64);
        let previous =
            (!before.is_empty()).then(|| before.iter().sum::<f64>() / before.len() as f64);
        cards.push(CheckinCard {
            area,
            headline: match area {
                CheckinArea::Readiness => "Readiness",
                CheckinArea::Sleep => "Sleep",
                CheckinArea::Training => "Training",
                CheckinArea::Health => "Fitness age",
            }
            .into(),
            value,
            unit: unit.into(),
            previous,
            tone: value.map(rules::score).unwrap_or(Tone::Neutral),
            series,
        });
    }
    let win = cards
        .iter()
        .find(|c| match c.area {
            CheckinArea::Readiness | CheckinArea::Sleep => {
                c.value.zip(c.previous).is_some_and(|(a, b)| a > b)
            }
            CheckinArea::Health => c.value.zip(c.previous).is_some_and(|(a, b)| a < b),
            _ => false,
        })
        .map(|c| format!("{} improved", c.headline));
    let last = date_sub(monday, 1);
    let sleep_gaps: Vec<f64> = (1..=7)
        .filter_map(|i| {
            let s = sleep_view(store, &date_sub(monday, i));
            s.asleep_min
                .zip(s.need_min)
                .filter(|(_, n)| *n > 0.)
                .map(|(a, n)| ((n - a) / n).max(0.))
        })
        .collect();
    let sleep_gap =
        (!sleep_gaps.is_empty()).then(|| sleep_gaps.iter().sum::<f64>() / sleep_gaps.len() as f64);
    let training = training(store, &last);
    let load_gap = training
        .load_focus
        .iter()
        .filter_map(|bar| {
            bar.target
                .filter(|r| r[0] > 0.)
                .map(|r| ((r[0] - bar.value) / r[0]).max(0.))
        })
        .reduce(f64::max);
    let readiness_drop = cards[0]
        .value
        .zip(cards[0].previous)
        .map(|(a, b)| ((b - a) / 100.).max(0.));
    let plan_week = NaiveDate::parse_from_str(monday, "%Y-%m-%d")
        .ok()
        .map(|d| (d, plan::last_full_week(store, d)));
    let intensity = plan_week.and_then(|(d, start)| plan::intensity(store, start, d));
    // Garmin's weekly intensity minutes against its goal; without them, the
    // Fitness Age vigorous-minutes average against its target.
    let intensity_gap = intensity
        .as_ref()
        .and_then(|i| i.goal.map(|g| (i.total, g)))
        .or_else(|| {
            health(store, &last)
                .drivers
                .iter()
                .find(|d| d.key == "vigorousMinutes")
                .and_then(|d| d.value.zip(d.target))
        })
        .filter(|(_, t)| *t > 0.)
        .map(|(v, t)| ((t - v) / t).max(0.));
    let focus =
        rules::weekly_focus(sleep_gap, load_gap, readiness_drop, intensity_gap).map(str::to_owned);
    let vo2max_change = historical_vo2(store, &last)
        .zip(historical_vo2(store, &date_sub(monday, 8)))
        .map(|(a, b)| a - b);
    let fitness_age_change = cards[3].value.zip(cards[3].previous).map(|(a, b)| a - b);
    CheckinView {
        timing: timing_review(store, monday),
        week_start: previous,
        cards,
        vo2max_change,
        fitness_age_change,
        win,
        focus,
        plan: plan_week.and_then(|(d, _)| plan::checkin(store, d)),
        intensity,
    }
}

fn week_nights(store: &Store, monday: &str, weeks_back: i64) -> Vec<(NightWindow, Option<String>)> {
    (1..=7)
        .rev()
        .filter_map(|i| {
            let day = date_sub(monday, i + 7 * weeks_back);
            let raw = sleep_record(store, &day)?;
            let s = sleep(&raw);
            let window = NightWindow {
                date: day,
                bed_hour: hour(&local_time(s, &["sleepStartTimestampLocal"])?)?,
                wake_hour: hour(&local_time(s, &["sleepEndTimestampLocal"])?)?,
            };
            Some((
                window,
                s["sleepAlignment"]["status"].as_str().map(str::to_owned),
            ))
        })
        .collect()
}
fn spread_min(hours: &[f64]) -> Option<f64> {
    let low = hours.iter().copied().reduce(f64::min)?;
    let high = hours.iter().copied().reduce(f64::max)?;
    Some(((high - low) * 60.).round())
}
/// The week's bed and wake times and Garmin's alignment ratings, for the check-in.
fn timing_review(store: &Store, monday: &str) -> Option<TimingReview> {
    let week = week_nights(store, monday, 0);
    if week.is_empty() {
        return None;
    }
    // Bedtimes before noon belong to the evening before.
    let beds: Vec<f64> = week
        .iter()
        .map(|(n, _)| {
            if n.bed_hour < 12. {
                n.bed_hour + 24.
            } else {
                n.bed_hour
            }
        })
        .collect();
    // Compare both weeks on their own circular wake axis, including midnight.
    let unwrap_wakes = |hours: Vec<f64>| {
        let Some(clock) = rules::wake_clock(&hours) else {
            return hours;
        };
        hours
            .iter()
            .map(|h| clock.median - 12. + (h - clock.median + 12.).rem_euclid(24.))
            .collect()
    };
    let wakes = unwrap_wakes(week.iter().map(|(n, _)| n.wake_hour).collect());
    let range = |hours: &[f64]| {
        let low = hours.iter().copied().reduce(f64::min)?;
        let high = hours.iter().copied().reduce(f64::max)?;
        Some([low, high].map(|h| rules::clock((h * 60.).round() as i64)))
    };
    let count = |code: &str| {
        week.iter()
            .filter(|(_, s)| s.as_deref() == Some(code))
            .count() as u32
    };
    let (aligned, ahead, behind) = (count("ALIGNED"), count("AHEAD"), count("BEHIND"));
    let wake_spread_min = spread_min(&wakes);
    let previous = unwrap_wakes(
        week_nights(store, monday, 1)
            .iter()
            .map(|(n, _)| n.wake_hour)
            .collect(),
    );
    let usual = rules::bedtime(&wakes, 0.);
    Some(TimingReview {
        bed_range: range(&beds),
        wake_range: range(&wakes),
        wake_spread_min,
        previous_wake_spread_min: spread_min(&previous),
        aligned,
        ahead,
        behind,
        focus: rules::timing_focus(aligned, ahead, behind, wake_spread_min, usual.as_deref())?,
        nights: week.into_iter().map(|(n, _)| n).collect(),
    })
}

#[cfg(test)]
mod humanize_tests {
    use super::humanize_code;

    #[test]
    fn garmin_feedback_codes_read_as_sentences() {
        assert_eq!(
            humanize_code("LET_YOUR_BODY_RECOVER").as_deref(),
            Some("Let your body recover")
        );
        assert_eq!(
            humanize_code("GOOD_SLEEP_HISTORY").as_deref(),
            Some("Good sleep history")
        );
        assert_eq!(
            humanize_code("Already prose").as_deref(),
            Some("Already prose")
        );
        assert_eq!(humanize_code("  "), None);
    }
}
