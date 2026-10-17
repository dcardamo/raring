//! Tolerant projections of raw Garmin responses. Unknown and absent fields stay absent.
use crate::{rules, store::Store, sync::selected_watch, view::*};
use chrono::{DateTime, Datelike, Duration, Local, NaiveDate, NaiveDateTime, Timelike, Utc};
use serde_json::Value;
use std::collections::BTreeMap;

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
fn local_time(v: &Value, keys: &[&str]) -> Option<String> {
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
fn readiness(v: &Value) -> &Value {
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
fn sleep(v: &Value) -> &Value {
    if !v["dailySleepDTO"].is_null() {
        &v["dailySleepDTO"]
    } else {
        v
    }
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
fn sleep_plan(store: &Store, date: &str) -> Option<SleepPlan> {
    let nights: Vec<Value> = (0..14)
        .filter_map(|i| store.get("sleep", &date_sub(date, i)))
        .collect();
    let wakes: Vec<f64> = nights
        .iter()
        .filter_map(|night| local_time(sleep(night), &["sleepEndTimestampLocal"]))
        .filter_map(|time| hour(&time))
        .filter(|hour| hour.is_finite() && (0.0..24.0).contains(hour))
        .collect();
    let current = store.get("sleep", date).unwrap_or(Value::Null);
    let upcoming_need = sleep_need_minutes(&sleep(&current)["nextSleepNeed"]);
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
    // Garmin's window is for the night its need describes, so only tonight's
    // need carries one. Its end plus the need lands on Garmin's expected wake.
    let window = if source == SleepNeedSource::Tonight {
        garmin_bedtime_window(&sleep(&current)["nextSleepNeed"])
    } else {
        None
    };
    let (bedtime, bedtime_start) = match window {
        Some((start, end)) => (end, Some(start)),
        None => (rules::bedtime(&wakes, need_min)?, None),
    };
    Some(SleepPlan {
        bedtime,
        bedtime_start,
        wake_time: rules::bedtime(&wakes, 0.),
        need_min,
        source,
        wake_nights: wakes.len() as u32,
    })
}
fn garmin_bedtime_window(need: &Value) -> Option<(String, String)> {
    let clock = |key: &str| {
        let minutes = num(need, &[&[key]]).filter(|n| n.is_finite())?;
        let minute = (minutes.round() as i64).rem_euclid(1440);
        Some(format!("{:02}:{:02}", minute / 60, minute % 60))
    };
    Some((
        clock("recommendedBedtimeStartMins")?,
        clock("recommendedBedtimeEndMins")?,
    ))
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
fn sscore(v: &Value) -> Option<f64> {
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
fn rscore(v: &Value) -> Option<f64> {
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
fn acute(v: &Value) -> Option<f64> {
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
fn factor_tone(feedback: Option<&str>, percent: Option<f64>) -> Tone {
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
        percent.map(rules::score).unwrap_or(Tone::Neutral)
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
        .history("activities", "999999", 100)
        .into_iter()
        .flat_map(|(_, v)| v.as_array().cloned().unwrap_or_default())
        .find(|a| a["activityId"].to_string().trim_matches('"') == id)
        .or_else(|| store.get("listed", id))
        .unwrap_or(Value::Null);
    (!detail.is_null() || !listed.is_null()).then_some(ActivityPayload { detail, listed })
}
fn activity(store: &Store, id: &str) -> Option<ActivityCard> {
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
    })
}
fn activities(store: &Store, since: &str) -> Vec<ActivityCard> {
    store
        .activity_ids(Some(since), 200)
        .iter()
        .filter_map(|id| activity(store, id))
        .collect()
}
fn age(store: &Store, date: &str) -> FitnessAgeCard {
    let v = store
        .get("fitness_age", date)
        .or_else(|| store.history("fitness_age", date, 1).pop().map(|(_, v)| v))
        .unwrap_or(Value::Null);
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
        VitalKind::SkinTemp => (
            "sleep",
            &[
                &["avgSkinTempDeviationC"],
                &["dailySleepDTO", "skinTempDeviation"],
            ],
        ),
    };
    store.get(source, date).and_then(|v| num(&v, paths))
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
        let trend: Vec<Option<f64>> = (0..30)
            .rev()
            .map(|i| vital_value(store, kind, &date_sub(date, i)))
            .collect();
        let value = vital_value(store, kind, date);
        let normal = if kind == VitalKind::Hrv {
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
        let tone = match (value, normal) {
            (Some(v), Some([lo, hi])) if v < lo || v > hi => Tone::Poor,
            (Some(_), Some(_)) => Tone::Good,
            _ => Tone::Neutral,
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
            normal,
            scale: [lo - span, hi + span],
            trend,
        });
    }
    let off_count = readings.iter().filter(|r| r.tone == Tone::Poor).count() as u32;
    let baseline_ready = readings
        .iter()
        .filter(|r| r.kind != VitalKind::Hrv)
        .any(|r| r.normal.is_some());
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
        || store
            .get("sleep", date)
            .is_some_and(|s| num(sleep(&s), &[&["sleepTimeSeconds"]]).is_some_and(|t| t > 0.))
}
/// The day the screens show. Garmin files a night under the day you wake, so
/// after midnight the calendar day is empty until you wake and the watch syncs;
/// until then (or noon) the previous day stays current.
pub fn current_date(store: &Store, now: NaiveDateTime) -> NaiveDate {
    let today = now.date();
    let yesterday = today - Duration::days(1);
    if now.hour() < 12
        && !has_night(store, &today.to_string())
        && has_night(store, &yesterday.to_string())
    {
        yesterday
    } else {
        today
    }
}
fn watch_info(store: &Store) -> (Value, Option<DateTime<Utc>>) {
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
            num(
                s,
                &[
                    &["avgSkinTempDeviationC"],
                    &["dailySleepDTO", "skinTempDeviation"],
                ],
            )
            .is_none(),
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
    let r = store.get("readiness", date).unwrap_or(Value::Null);
    let s = store.get("sleep", date).unwrap_or(Value::Null);
    let t = store.get("training", date).unwrap_or(Value::Null);
    let h = store.get("hrv", date).unwrap_or(Value::Null);
    let hs = hrv(&h);
    let (watch, uploaded) = watch_info(store);
    let missing_metric_reasons = missing_metric_reasons(&watch, &s, &r, &h);
    // Before noon Today can still show yesterday's recorded night; check the
    // current calendar day's missing upload rather than the displayed day.
    let watch_sleep_upload_stale = sleep_upload_stale(
        store,
        &Local::now().date_naive().to_string(),
        Local::now(),
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
            "Recovery",
            "recoveryTimeFactorPercent",
            "recoveryTimeFactorFeedback",
        ),
        ("hrv", "HRV", "hrvFactorPercent", "hrvFactorFeedback"),
        (
            "acuteLoad",
            "Load",
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
            "Stress",
            "stressHistoryFactorPercent",
            "stressHistoryFactorFeedback",
        ),
    ];
    let mut factors: Vec<Factor> = factors
        .iter()
        .filter_map(|(key, name, field, feedback_field)| {
            let snapshot = readiness(&r);
            let percent = num(snapshot, &[&[*field]]);
            let feedback = snapshot[*feedback_field].as_str().filter(|s| !s.is_empty());
            (percent.is_some() || feedback.is_some()).then(|| Factor {
                key: (*key).into(),
                name: (*name).into(),
                percent,
                tone: factor_tone(feedback, percent),
                detail: feedback
                    .map(|s| label(Some(s.into())))
                    .or_else(|| percent.map(|p| format!("Garmin rates your {name} at {p:.0}%.")))
                    .unwrap_or_default(),
            })
        })
        .collect();
    factors.sort_by(|a, b| {
        a.percent
            .unwrap_or(100.)
            .total_cmp(&b.percent.unwrap_or(100.))
    });
    let drag = factors
        .iter()
        .filter_map(|f| f.percent.map(|p| (f.name.as_str(), p)))
        .collect::<Vec<_>>();
    let biggest_drag = rules::biggest_drag(&drag).map(str::to_owned);
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
    TodayView {
        date: date.into(),
        watch_last_upload: uploaded.map(|t| t.to_rfc3339_opts(chrono::SecondsFormat::Millis, true)),
        watch_sleep_upload_stale,
        missing_metric_reasons,
        sleep_plan: sleep_plan(store, date),
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
        headline: match call {
            Some(Effort::Push) => "Ready to push",
            Some(Effort::Steady) => "Room to build",
            Some(Effort::Easy) | Some(Effort::Rest) => "Take it easy",
            None => "Waiting for readiness",
        }
        .into(),
        chips,
        factors,
        call,
        // A finished workout comes first, then Garmin's own readiness feedback
        // (sent as codes such as LET_YOUR_BODY_RECOVER), then the biggest drag.
        call_note: activities(store, date)
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
        body_battery: BodyBatteryCard {
            // Daily stats can be more recent than the sparsely sampled report.
            now: num(&summary, &[&["bodyBatteryMostRecentValue"]])
                .or_else(|| num(b, &[&["bodyBatteryValue"], &["currentValue"]]))
                .or_else(|| battery_latest(b)),
            charged: num(b, &[&["charged"], &["chargedValue"]])
                .or_else(|| num(&summary, &[&["bodyBatteryChargedValue"]])),
            drained: num(b, &[&["drained"], &["drainedValue"]])
                .or_else(|| num(&summary, &[&["bodyBatteryDrainedValue"]])),
        },
        stress: stress(store, date),
        hrv: HrvCard {
            last_night: num(hs, &[&["lastNightAvg"]]),
            status: hstatus.clone().map(|s| label(Some(s))),
            tone: hstatus
                .as_deref()
                .map(rules::level)
                .unwrap_or(Tone::Neutral),
            baseline: vitals(store, date)
                .readings
                .iter()
                .find(|v| v.kind == VitalKind::Hrv)
                .and_then(|v| v.normal),
            week: (0..7)
                .rev()
                .map(|i| vital_value(store, VitalKind::Hrv, &date_sub(date, i)))
                .collect(),
        },
        vitals: vitals(store, date),
        activities: activities(store, date),
        fitness_age: age(store, date),
        checkin_ready: NaiveDate::parse_from_str(date, "%Y-%m-%d")
            .is_ok_and(|d| d.weekday() == chrono::Weekday::Mon)
            && store.get("summary", &date_sub(date, 1)).is_some(),
    }
}
pub fn sleep_view(store: &Store, date: &str) -> SleepView {
    let raw = store.get("sleep", date).unwrap_or(Value::Null);
    let v = sleep(&raw);
    let score = sscore(&raw);
    let asleep = num(v, &[&["sleepTimeSeconds"]]).map(|s| s / 60.);
    let need = sleep_need(&raw);
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
                num(v, &[&["avgHeartRate"], &["averageHeartRate"]])
                    .filter(|n| *n > 0. && *n < 300.),
            ),
            (
                OvernightKind::Respiration,
                "respiration",
                "br/min",
                num(v, &[&["averageRespirationValue"]]).filter(|n| *n > 0. && *n < 100.),
            ),
            (
                OvernightKind::Spo2,
                "SpO2",
                "%",
                num(v, &[&["averageSpO2Value"], &["avgSpO2"]]).filter(|n| *n > 0. && *n <= 100.),
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
        sleep_plan: sleep_plan(store, date),
    }
}
fn hour(s: &str) -> Option<f64> {
    let time = s.split('T').nth(1).or_else(|| s.split(' ').nth(1))?;
    let mut x = time.split(':');
    Some(x.next()?.parse::<f64>().ok()? + x.next()?.parse::<f64>().ok()? / 60.)
}
pub fn training(store: &Store, date: &str) -> TrainingView {
    let v = store.get("training", date).unwrap_or(Value::Null);
    let load = acute(&v);
    let status = status(&v);
    let workouts = activities(store, &date_sub(date, 30));
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
    TrainingView {
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
        headline: match (load, band(&v)) {
            (Some(amount), Some([lo, _])) if amount < lo => "Room to build".into(),
            (Some(amount), Some([_, hi])) if amount > hi => "Load high".into(),
            (Some(_), Some(_)) => "Room to build".into(),
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
        running_tolerance_km: store.get("tolerance", date).and_then(|v| {
            // The daily endpoint returns an array and reports acuteTolerance in metres.
            num(first(&v), &[&["acuteTolerance"]])
                .map(|metres| metres / 1000.)
                .or_else(|| num(&v, &[&["runningTolerance"], &["tolerance"]]))
        }),
        workouts,
    }
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
    Some(WorkoutView {
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
    })
}
pub fn health(store: &Store, date: &str) -> HealthView {
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
        let value = num(data, &[&["value"]]).filter(|n| n.is_finite());
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
    HealthView {
        date: date.into(),
        fitness_age: age(store, date),
        lever,
        drivers,
        vitals: vitals(store, date),
        stress: stress(store, date),
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
    let vigorous_gap = health(store, &last)
        .drivers
        .iter()
        .find(|d| d.key == "vigorousMinutes")
        .and_then(|d| d.value.zip(d.target))
        .filter(|(_, t)| *t > 0.)
        .map(|(v, t)| ((t - v) / t).max(0.));
    let focus =
        rules::weekly_focus(sleep_gap, load_gap, readiness_drop, vigorous_gap).map(str::to_owned);
    let vo2max_change = historical_vo2(store, &last)
        .zip(historical_vo2(store, &date_sub(monday, 8)))
        .map(|(a, b)| a - b);
    let fitness_age_change = cards[3].value.zip(cards[3].previous).map(|(a, b)| a - b);
    CheckinView {
        week_start: previous,
        cards,
        vo2max_change,
        fitness_age_change,
        win,
        focus,
    }
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
