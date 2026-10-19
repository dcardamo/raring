//! Long-range trends. Each metric has one value per day, read from the per-day
//! rows of the recent window and the history rows behind them; long ranges
//! average those days into weeks or months (docs/spec.md, Rules: Trends).
use crate::{
    screens::{self, Night},
    store::Store,
    view::*,
};
use chrono::{Datelike, Duration, NaiveDate};
use serde_json::Value;
use std::collections::BTreeMap;

pub const METRICS: [TrendMetric; 14] = [
    TrendMetric::Readiness,
    TrendMetric::SleepScore,
    TrendMetric::SleepTime,
    TrendMetric::Hrv,
    TrendMetric::RestingHr,
    TrendMetric::SleepHr,
    TrendMetric::Respiration,
    TrendMetric::Spo2,
    TrendMetric::SkinTemp,
    TrendMetric::BodyBattery,
    TrendMetric::Stress,
    TrendMetric::Vo2max,
    TrendMetric::FitnessAge,
    TrendMetric::AcuteLoad,
];

/// Where a daily value lives. A metric lists its sources most trusted first.
enum Source {
    /// A number at the first of these SQLite JSON paths that has one.
    Path(&'static str, &'static [&'static str]),
    /// A value that needs the whole (small) row.
    Row(&'static str, fn(&Value) -> Option<f64>),
    /// An overnight average, at the same fields the Sleep and Health screens read.
    Nightly(&'static str, Night),
}
use Source::{Nightly, Path, Row};

/// Lower bound for every trend date: older than any Garmin Connect account.
const FIRST_DAY: (i32, u32, u32) = (1990, 1, 1);
fn first_day() -> NaiveDate {
    NaiveDate::from_ymd_opt(FIRST_DAY.0, FIRST_DAY.1, FIRST_DAY.2).unwrap_or_default()
}

fn health_status(v: &Value, kind: &str) -> Option<f64> {
    v["metrics"]
        .as_array()?
        .iter()
        .find(|m| m["type"] == kind)?["value"]
        .as_f64()
}
fn status_hrv(v: &Value) -> Option<f64> {
    health_status(v, "HRV")
}
fn status_hr(v: &Value) -> Option<f64> {
    health_status(v, "HR")
}
fn status_respiration(v: &Value) -> Option<f64> {
    health_status(v, "RESPIRATION")
}
fn status_spo2(v: &Value) -> Option<f64> {
    health_status(v, "SPO2")
}
fn status_skin(v: &Value) -> Option<f64> {
    health_status(v, "SKIN_TEMP_C")
}
/// The day's last readiness snapshot carries the end-of-day acute load, the
/// value Training Status reports for that day.
fn acute_load(v: &Value) -> Option<f64> {
    screens::readiness(v)["acuteLoad"].as_f64()
}

/// Health Status matches the nightly records for HR, HRV, SpO2 and skin
/// temperature; its respiration can read slightly higher. Health Status
/// wins where Garmin sent it, as on the vitals cards.
fn sources(metric: TrendMetric) -> &'static [Source] {
    use TrendMetric::*;
    match metric {
        Readiness => &[Row("readiness", screens::rscore)],
        SleepScore => &[
            Path("sleep", &["$.dailySleepDTO.sleepScores.overall.value"]),
            Path("sleep_summary", &["$.sleepScores.overall.value"]),
        ],
        SleepTime => &[
            Path("sleep", &["$.dailySleepDTO.sleepTimeSeconds"]),
            Path("sleep_summary", &["$.sleepTimeSeconds"]),
        ],
        Hrv => &[
            Row("health_status", status_hrv),
            Path("hrv", &["$.hrvSummary.lastNightAvg"]),
            Path("hrv_summary", &["$.lastNightAvg"]),
        ],
        RestingHr => &[
            Path("summary", &["$.restingHeartRate"]),
            Path("daily_summary", &["$.heartRate.restingValue"]),
        ],
        SleepHr => &[
            Row("health_status", status_hr),
            Nightly("sleep", Night::HeartRate),
            Nightly("sleep_summary", Night::HeartRate),
        ],
        Respiration => &[
            Row("health_status", status_respiration),
            Nightly("sleep", Night::Respiration),
            Nightly("sleep_summary", Night::Respiration),
        ],
        Spo2 => &[
            Row("health_status", status_spo2),
            Nightly("sleep", Night::Spo2),
            Nightly("sleep_summary", Night::Spo2),
        ],
        SkinTemp => &[
            Row("health_status", status_skin),
            Nightly("sleep", Night::SkinTemp),
            Nightly("sleep_summary", Night::SkinTemp),
        ],
        BodyBattery => &[
            Path("summary", &["$.bodyBatteryHighestValue"]),
            Path("daily_summary", &["$.bodyBattery.maxValue"]),
        ],
        Stress => &[
            Path("stress", &["$.avgStressLevel"]),
            Path("summary", &["$.averageStressLevel"]),
            Path("daily_summary", &["$.stress.avgLevel"]),
        ],
        FitnessAge => &[Path(
            "fitness_age",
            &["$.fitnessAge", "$.fitnessAgeData.fitnessAge"],
        )],
        Vo2max => &[Row("max_metrics", screens::vo2)],
        AcuteLoad => &[Row("readiness", acute_load)],
    }
}

/// Garmin marks unmeasured days with negative or zero placeholders.
fn valid(metric: TrendMetric, v: f64) -> bool {
    use TrendMetric::*;
    v.is_finite()
        && match metric {
            Readiness | SleepScore | BodyBattery | Stress => (0. ..=100.).contains(&v),
            SleepTime | Hrv | Vo2max | FitnessAge => v > 0.,
            RestingHr => (20. ..=250.).contains(&v),
            SleepHr => screens::night_valid(Night::HeartRate, v),
            Respiration => screens::night_valid(Night::Respiration, v),
            Spo2 => screens::night_valid(Night::Spo2, v),
            SkinTemp => screens::night_valid(Night::SkinTemp, v),
            AcuteLoad => v >= 0.,
        }
}

pub fn name_unit(metric: TrendMetric) -> (&'static str, &'static str) {
    use TrendMetric::*;
    match metric {
        Readiness => ("Readiness", ""),
        SleepScore => ("Sleep score", ""),
        SleepTime => ("Time asleep", "min"),
        Hrv => ("HRV", "ms"),
        RestingHr => ("Resting HR", "bpm"),
        SleepHr => ("Sleep HR", "bpm"),
        Respiration => ("Respiration", "br/min"),
        Spo2 => ("SpO₂", "%"),
        SkinTemp => ("Skin temp", "°C"),
        BodyBattery => ("Body Battery high", ""),
        Stress => ("Stress", ""),
        Vo2max => ("VO₂ max", ""),
        FitnessAge => ("Fitness Age", "years"),
        AcuteLoad => ("Acute load", ""),
    }
}

fn day(s: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(s.get(..10)?, "%Y-%m-%d").ok()
}

/// Recorded days of a metric from `from` to `to`.
pub fn daily(
    store: &Store,
    metric: TrendMetric,
    from: NaiveDate,
    to: NaiveDate,
) -> BTreeMap<NaiveDate, f64> {
    let (from, to) = (from.to_string(), to.to_string());
    let mut days = BTreeMap::new();
    // Least trusted first, so a better source overwrites it; an invalid
    // reading never hides a valid one.
    for source in sources(metric).iter().rev() {
        let values: Vec<(String, f64)> = match source {
            Path(endpoint, paths) => store.day_numbers(endpoint, paths, &from, &to),
            Nightly(endpoint, field) => {
                let paths = screens::night_paths(*field);
                let paths: Vec<&str> = paths.iter().map(String::as_str).collect();
                store.day_numbers(endpoint, &paths, &from, &to)
            }
            Row(endpoint, read) => store
                .days_between(endpoint, &from, &to)
                .into_iter()
                .filter_map(|(key, row)| read(&row).map(|v| (key, v)))
                .collect(),
        };
        for (key, value) in values {
            let value = if metric == TrendMetric::SleepTime {
                value / 60.
            } else {
                value
            };
            if let Some(date) = day(&key).filter(|_| valid(metric, value)) {
                days.insert(date, value);
            }
        }
    }
    days
}

fn monday(d: NaiveDate) -> NaiveDate {
    d - Duration::days(d.weekday().num_days_from_monday() as i64)
}
fn month_start(d: NaiveDate) -> NaiveDate {
    d.with_day(1).unwrap_or(d)
}
fn next_month(d: NaiveDate) -> NaiveDate {
    let (y, m) = if d.month() == 12 {
        (d.year() + 1, 1)
    } else {
        (d.year(), d.month() + 1)
    };
    NaiveDate::from_ymd_opt(y, m, 1).unwrap_or(d + Duration::days(31))
}

/// One point per day up to 90 days, weekly averages up to two years, monthly
/// beyond.
pub fn bucket_for(days: i64) -> TrendBucket {
    if days <= 90 {
        TrendBucket::Day
    } else if days <= 731 {
        TrendBucket::Week
    } else {
        TrendBucket::Month
    }
}

/// The first day, bucket size and, where there is one, the shift back to the
/// previous period. All starts at the first recorded day.
fn period(
    range: TrendRange,
    end: NaiveDate,
    first: Option<NaiveDate>,
) -> (NaiveDate, TrendBucket, Option<i64>) {
    let days = |n: i64| (end - Duration::days(n - 1), TrendBucket::Day, Some(n));
    match range {
        TrendRange::Days7 => days(7),
        TrendRange::Days14 => days(14),
        TrendRange::Days30 => days(30),
        TrendRange::Days90 => days(90),
        TrendRange::Custom => unreachable!("custom spans are validated by trend"),
        // 52 whole weeks, the last one running to today.
        TrendRange::Year => (
            monday(end) - Duration::weeks(51),
            TrendBucket::Week,
            Some(364),
        ),
        TrendRange::All => {
            let first = first.unwrap_or(end).min(end);
            let bucket = bucket_for((end - first).num_days() + 1);
            let from = match bucket {
                TrendBucket::Day => first,
                TrendBucket::Week => monday(first),
                TrendBucket::Month => month_start(first),
            };
            (from, bucket, None)
        }
    }
}

fn buckets(from: NaiveDate, to: NaiveDate, bucket: TrendBucket) -> Vec<(NaiveDate, NaiveDate)> {
    let mut out = vec![];
    let mut start = from;
    while start <= to {
        let next = match bucket {
            TrendBucket::Day => start + Duration::days(1),
            TrendBucket::Week => start + Duration::days(7),
            TrendBucket::Month => next_month(start),
        };
        out.push((start, (next - Duration::days(1)).min(to)));
        start = next;
    }
    out
}

fn mean(values: impl Iterator<Item = f64>) -> (Option<f64>, u32) {
    let (sum, n) = values.fold((0., 0u32), |(s, n), v| (s + v, n + 1));
    ((n > 0).then(|| sum / n as f64), n)
}

fn series(
    metric: TrendMetric,
    days: &BTreeMap<NaiveDate, f64>,
    spans: &[(NaiveDate, NaiveDate)],
) -> TrendSeries {
    let (name, unit) = name_unit(metric);
    let points = spans
        .iter()
        .map(|(start, end)| {
            let (value, days) = mean(days.range(start..=end).map(|(_, v)| *v));
            TrendPoint {
                start: start.to_string(),
                end: end.to_string(),
                value,
                days,
            }
        })
        .collect();
    let (average, count) = match (spans.first(), spans.last()) {
        (Some((from, _)), Some((_, to))) => mean(days.range(from..=to).map(|(_, v)| *v)),
        _ => (None, 0),
    };
    TrendSeries {
        metric,
        name: name.into(),
        unit: unit.into(),
        points,
        average,
        days: count,
    }
}

pub fn trend(store: &Store, query: &TrendQuery, today: &str) -> Result<TrendView, String> {
    let parse = |s: &str| {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").map_err(|_| format!("Invalid trend date: {s}"))
    };
    let today = parse(today)?;
    // Every supplied date lies between the first possible day and today, so a
    // chart never silently draws an empty future or pre-Garmin range.
    let check = |s: &str| {
        let date = parse(s)?;
        if date > today {
            Err(format!("Choose a date no later than today ({today})"))
        } else if date < first_day() {
            Err(format!("Choose a date from {} on", first_day()))
        } else {
            Ok(date)
        }
    };
    let end = query.end.as_deref().map(check).transpose()?.unwrap_or(today);
    let start = query.start.as_deref().map(check).transpose()?;
    let comparison = query.comparison_start.as_deref().map(check).transpose()?;
    let custom = if query.range == TrendRange::Custom {
        let from = start.ok_or("Choose a block start date")?;
        if from > end {
            return Err("Choose a block start on or before its end".into());
        }
        let length = (end - from).num_days() + 1;
        Some((from, bucket_for(length), Some(length)))
    } else {
        None
    };
    // All reads the whole history to find its first day; other ranges read
    // their own period and, when compared, the one before it.
    let (days, (from, bucket, shift)) = if query.range == TrendRange::All {
        let days = daily(store, query.metric, first_day(), end);
        let period = period(query.range, end, days.keys().next().copied());
        (days, period)
    } else {
        let (from, bucket, shift) = custom.unwrap_or_else(|| period(query.range, end, None));
        let back = shift.filter(|_| query.previous).unwrap_or(0);
        let days = daily(store, query.metric, from - Duration::days(back), end);
        (days, (from, bucket, shift))
    };
    if let (Some(start), Some(length)) = (comparison.filter(|_| query.previous), shift) {
        if start + Duration::days(length - 1) > today {
            return Err(format!(
                "The comparison block runs past today ({today}); choose an earlier start"
            ));
        }
    }
    let spans = buckets(from, end, bucket);
    let series_now = series(query.metric, &days, &spans);
    let previous = shift.filter(|_| query.previous).map(|shift| {
        let back = Duration::days(comparison.map_or(shift, |start| (from - start).num_days()));
        let spans: Vec<_> = spans.iter().map(|(s, e)| (*s - back, *e - back)).collect();
        if comparison.is_some() {
            let recorded = daily(store, query.metric, spans[0].0, spans.last().unwrap().1);
            series(query.metric, &recorded, &spans)
        } else {
            series(query.metric, &days, &spans)
        }
    });
    let overlay = query
        .overlay
        .filter(|m| *m != query.metric)
        .map(|metric| series(metric, &daily(store, metric, from, end), &spans));
    let change = previous
        .as_ref()
        .and_then(|p| Some(series_now.average? - p.average?));
    Ok(TrendView {
        range: query.range,
        bucket,
        from: from.to_string(),
        to: end.to_string(),
        series: series_now,
        overlay,
        previous,
        change,
        available: METRICS.to_vec(),
        freshness: (end == today && !matches!(query.metric, TrendMetric::Vo2max | TrendMetric::FitnessAge)).then(|| {
            let latest = daily(store, query.metric, today - Duration::days(400), today).into_keys().next_back();
            crate::weight::freshness(latest, today, 2)
        }),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn d(s: &str) -> NaiveDate {
        day(s).unwrap()
    }
    #[test]
    fn buckets_follow_the_span() {
        assert_eq!(bucket_for(90), TrendBucket::Day);
        assert_eq!(bucket_for(91), TrendBucket::Week);
        assert_eq!(bucket_for(731), TrendBucket::Week);
        assert_eq!(bucket_for(732), TrendBucket::Month);
    }
    #[test]
    fn a_year_is_52_monday_weeks_ending_today() {
        // 2026-10-15 is a Thursday.
        let (from, bucket, shift) = period(TrendRange::Year, d("2026-10-15"), None);
        assert_eq!(bucket, TrendBucket::Week);
        assert_eq!(from.weekday(), chrono::Weekday::Mon);
        assert_eq!(shift, Some(364));
        let spans = buckets(from, d("2026-10-15"), bucket);
        assert_eq!(spans.len(), 52);
        assert_eq!(spans[51], (d("2026-10-12"), d("2026-10-15")));
    }
    #[test]
    fn months_are_calendar_months_clipped_to_today() {
        let spans = buckets(d("2024-11-01"), d("2025-02-10"), TrendBucket::Month);
        assert_eq!(
            spans,
            vec![
                (d("2024-11-01"), d("2024-11-30")),
                (d("2024-12-01"), d("2024-12-31")),
                (d("2025-01-01"), d("2025-01-31")),
                (d("2025-02-01"), d("2025-02-10")),
            ]
        );
    }
}
