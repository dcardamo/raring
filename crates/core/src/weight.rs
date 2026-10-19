//! Weight and reading freshness (spec Rules). Local stored measurements only;
//! smoothing and comparisons are Raring's, not Garmin estimates or targets.
use crate::{store::Store, view::*, Error, Result};
use chrono::{DateTime, Duration, NaiveDate};
use serde_json::Value;
use std::collections::BTreeMap;

const FIRST: &str = "1990-01-01";
const WINDOWS: [u32; 6] = [1, 3, 7, 14, 30, 90];
const KG_PER_LB: f64 = 0.45359237;

pub fn freshness(
    recorded: Option<NaiveDate>,
    as_of: NaiveDate,
    threshold_days: u32,
) -> ReadingFreshness {
    let recorded = recorded.filter(|d| *d <= as_of);
    let age = recorded.map(|d| (as_of - d).num_days() as u32);
    ReadingFreshness {
        recorded_on: recorded.map(|d| d.to_string()),
        age_days: age,
        stale: age.is_some_and(|d| d > threshold_days),
        threshold_days,
    }
}

pub fn steady_threshold(days: u32) -> f64 {
    0.2_f64.max(0.05 * f64::from(days) / 7.)
}

fn kilos(v: &Value) -> Option<f64> {
    v["weight"]
        .as_f64()
        .filter(|w| w.is_finite() && *w > 0.)
        .map(|w| w / 1000.)
}
fn valid_date(d: &str) -> Option<NaiveDate> {
    (d.len() == 10 && d >= FIRST)
        .then(|| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .flatten()
}

/// Range backfill already keeps the latest measurement of each Garmin day.
/// Merge the latest endpoint only when it is newer and a valid measurement.
fn measurements(store: &Store, end: NaiveDate) -> BTreeMap<NaiveDate, Value> {
    let mut rows: BTreeMap<NaiveDate, Value> = store
        .days_between("weight", FIRST, &end.to_string())
        .into_iter()
        .filter_map(|(d, v)| {
            Some((
                valid_date(&d).filter(|d| *d <= end && kilos(&v).is_some())?,
                v,
            ))
        })
        .collect();
    if let Some(v) = store
        .get("weight_latest", "me")
        .filter(|v| kilos(v).is_some())
    {
        if let Some(day) = v["date"]
            .as_i64()
            .and_then(DateTime::from_timestamp_millis)
            .map(|t| t.date_naive())
            .filter(|d| d.to_string().as_str() >= FIRST && *d <= end)
        {
            let timestamp = v["timestampGMT"].as_i64().unwrap_or(0);
            if rows
                .get(&day)
                .is_none_or(|r| r["timestampGMT"].as_i64().unwrap_or(0) <= timestamp)
            {
                rows.insert(day, v);
            }
        }
    }
    rows
}

/// Daily trend through the last actual measurement, retaining full precision.
/// Missing days hold the preceding trend; no projection after the final date.
fn smooth(rows: &BTreeMap<NaiveDate, Value>) -> BTreeMap<NaiveDate, f64> {
    let mut result = BTreeMap::new();
    let mut previous: Option<(NaiveDate, f64)> = None;
    for (&day, v) in rows {
        let Some(w) = kilos(v) else { continue };
        let trend = if let Some((last, t)) = previous {
            for d in last.iter_days().skip(1).take_while(|d| *d < day) {
                result.insert(d, t);
            }
            let gap = (day - last).num_days() as f64;
            t + (1. - 0.9_f64.powf(gap)) * (w - t)
        } else {
            w
        };
        result.insert(day, trend);
        previous = Some((day, trend));
    }
    result
}

pub fn view(
    store: &Store,
    range: TrendRange,
    end: NaiveDate,
    today: NaiveDate,
) -> Result<WeightView> {
    if end.to_string().as_str() < FIRST || end > today || range == TrendRange::Custom {
        return Err(Error::InvalidLocal("weight range or date"));
    }
    let settings = store.get("profile_settings", "me").unwrap_or(Value::Null);
    let unit = match settings["measurementSystem"].as_str() {
        Some("metric") => Some(WeightUnit::Kg),
        Some(s) if s.starts_with("statute") => Some(WeightUnit::Lb),
        _ => None,
    };
    let rows = measurements(store, end);
    let latest = rows.last_key_value();
    let mut view = WeightView {
        range,
        unit,
        as_of: end.to_string(),
        from: None,
        to: None,
        freshness: freshness(latest.map(|(d, _)| *d), end, 7),
        scale: None,
        trend: None,
        average: None,
        change: None,
        bmi: None,
        body_fat: None,
        points: vec![],
        changes: vec![],
        note: None,
    };
    let Some(unit) = unit else {
        view.note = Some("Waiting for Garmin's measurement units".into());
        return Ok(view);
    };
    let Some((&last, row)) = latest else {
        view.note = Some("No recorded weigh-ins".into());
        return Ok(view);
    };
    let convert = |kg: f64| {
        if unit == WeightUnit::Kg {
            kg
        } else {
            kg / KG_PER_LB
        }
    };
    let series = smooth(&rows);
    let first = *series.first_key_value().unwrap().0;
    let days = match range {
        TrendRange::Days7 => 7,
        TrendRange::Days14 => 14,
        TrendRange::Days30 => 30,
        TrendRange::Days90 => 90,
        TrendRange::Year => 365,
        TrendRange::All => (last - first).num_days() + 1,
        TrendRange::Custom => unreachable!(),
    };
    let from = (last - Duration::days(days - 1)).max(first);
    let trend = *series.get(&last).unwrap();
    let before = if range == TrendRange::All {
        Some(first)
    } else {
        Some(last - Duration::days(days))
    };
    view.from = Some(from.to_string());
    view.to = Some(last.to_string());
    view.scale = kilos(row).map(convert);
    view.trend = Some(convert(trend));
    let values: Vec<_> = series.range(from..=last).map(|(_, v)| *v).collect();
    view.average =
        (!values.is_empty()).then(|| convert(values.iter().sum::<f64>() / values.len() as f64));
    view.change = before
        .and_then(|d| series.get(&d))
        .map(|v| convert(trend - v));
    view.bmi = row["bmi"].as_f64().filter(|v| v.is_finite() && *v > 0.);
    view.body_fat = row["bodyFat"]
        .as_f64()
        .filter(|v| v.is_finite() && (0. ..=100.).contains(v));
    // The core carries exact dates and scale-gap semantics into the chart.
    // Long ranges take the latest weigh-in per step, never average scale weight.
    let step = if days > 400 {
        14
    } else if days > 90 {
        7
    } else {
        1
    };
    let mut start = from;
    let mut previous_scale: Option<NaiveDate> = None;
    while start <= last {
        let stop = (start + Duration::days(step - 1)).min(last);
        let measure = rows.range(start..=stop).next_back();
        let date = measure.map_or(stop, |(d, _)| *d);
        let scale = measure.and_then(|(_, v)| kilos(v)).map(convert);
        let break_scale = scale.is_some()
            && previous_scale.is_some_and(|p| {
                let dates: Vec<_> = rows.range(p..=date).map(|(d, _)| *d).collect();
                dates.windows(2).any(|w| (w[1] - w[0]).num_days() > 2)
            });
        if scale.is_some() {
            previous_scale = Some(date);
        }
        view.points.push(WeightPoint {
            date: date.to_string(),
            scale,
            trend: series.get(&date).copied().map(convert),
            break_scale,
        });
        start = stop + Duration::days(1);
    }
    view.changes = WINDOWS
        .iter()
        .map(|&n| {
            let start = last - Duration::days(i64::from(n));
            let delta = series.get(&start).map(|v| trend - v);
            let direction = delta.map(|d| {
                if d.abs() < steady_threshold(n) {
                    WeightDirection::Steady
                } else if d < 0. {
                    WeightDirection::Losing
                } else {
                    WeightDirection::Gaining
                }
            });
            WeightChange {
                days: n,
                change: delta.map(convert),
                per_week: delta
                    .filter(|_| n >= 7)
                    .map(|d| convert(d * 7. / f64::from(n))),
                direction,
                spark: (0..=n)
                    .map(|i| {
                        series
                            .get(&(start + Duration::days(i64::from(i))))
                            .copied()
                            .map(convert)
                    })
                    .collect(),
            }
        })
        .collect();
    Ok(view)
}
