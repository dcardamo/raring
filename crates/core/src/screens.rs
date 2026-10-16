//! Tolerant projections of raw Garmin responses. Unknown and absent fields stay absent.
use crate::{rules, store::Store, view::*};
use chrono::{Datelike, Duration, NaiveDate};
use serde_json::Value;

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
fn readiness(v: &Value) -> &Value {
    if let Some(a) = v.as_array() {
        a.iter()
            .find(|x| x["inputContext"] == "AFTER_WAKEUP_RESET")
            .or_else(|| a.first())
            .unwrap_or(&Value::Null)
    } else {
        v
    }
}
fn sleep(v: &Value) -> &Value {
    if !v["dailySleepDTO"].is_null() {
        &v["dailySleepDTO"]
    } else {
        v
    }
}
fn first(v: &Value) -> &Value {
    v.as_array().and_then(|a| a.first()).unwrap_or(v)
}
fn vo2(v: &Value) -> Option<f64> {
    num(
        first(v),
        &[
            &["generic", "vo2MaxValue"],
            &["vo2MaxValue"],
            &["generic", "vo2MaxPreciseValue"],
        ],
    )
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
fn rscore(v: &Value) -> Option<f64> {
    num(readiness(v), &[&["score"], &["trainingReadinessScore"]])
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
    v.unwrap_or_default()
        .replace('_', " ")
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
fn status(v: &Value) -> Option<String> {
    string(
        v,
        &[
            &["trainingStatus"],
            &["trainingStatusFeedbackPhrase"],
            &["trainingStatusData", "trainingStatus"],
            &["latestTrainingStatusData", "trainingStatus"],
        ],
    )
}
fn acute(v: &Value) -> Option<f64> {
    num(
        v,
        &[
            &["acuteTrainingLoad"],
            &["acuteLoad"],
            &["trainingStatusData", "acuteTrainingLoad"],
            &["latestTrainingStatusData", "acuteTrainingLoad"],
        ],
    )
}
fn band(v: &Value) -> Option<[f64; 2]> {
    Some([
        num(
            v,
            &[
                &["optimalRangeMin"],
                &["minOptimalLoad"],
                &["trainingStatusData", "optimalRangeMin"],
            ],
        )?,
        num(
            v,
            &[
                &["optimalRangeMax"],
                &["maxOptimalLoad"],
                &["trainingStatusData", "optimalRangeMax"],
            ],
        )?,
    ])
}
fn activity(store: &Store, id: &str) -> Option<ActivityCard> {
    let v = store.get("activity", id).or_else(|| {
        store
            .history("activities", "999999", 100)
            .into_iter()
            .flat_map(|(_, v)| v.as_array().cloned().unwrap_or_default())
            .find(|a| a["activityId"].to_string().trim_matches('"') == id)
    })?;
    let kind = string(&v, &[&["activityType", "typeKey"]])
        .unwrap_or_default()
        .to_ascii_lowercase();
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
        name: string(&v, &[&["activityName"]]).unwrap_or_else(|| "Workout".into()),
        start_local: string(&v, &[&["startTimeLocal"]]).unwrap_or_default(),
        duration_s: num(&v, &[&["duration"]]).unwrap_or(0.),
        distance_km: num(&v, &[&["distance"]]).map(|m| m / 1000.),
        load: num(&v, &[&["activityTrainingLoad"], &["trainingLoad"]]),
        zone_seconds: zones
            .as_array()
            .map(|a| {
                a.iter()
                    .map(|z| num(z, &[&["secsInZone"], &["timeInZone"]]).unwrap_or(0.))
                    .collect()
            })
            .unwrap_or_default(),
        aerobic_te: num(&v, &[&["aerobicTrainingEffect"]]),
        aerobic_label: string(&v, &[&["aerobicTrainingEffectMessage"]]).map(|s| label(Some(s))),
        anaerobic_te: num(&v, &[&["anaerobicTrainingEffect"]]),
        anaerobic_label: string(&v, &[&["anaerobicTrainingEffectMessage"]]).map(|s| label(Some(s))),
        benefit: string(&v, &[&["trainingEffectLabel"], &["primaryBenefit"]])
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
        VitalKind::SkinTemp => ("sleep", &[&["dailySleepDTO", "skinTempDeviation"]]),
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
        let lo = normal.map(|r| r[0]).or(value).unwrap_or(0.);
        let hi = normal.map(|r| r[1]).or(value).unwrap_or(1.);
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
pub fn today(store: &Store, date: &str) -> TodayView {
    let r = store.get("readiness", date).unwrap_or(Value::Null);
    let s = store.get("sleep", date).unwrap_or(Value::Null);
    let t = store.get("training", date).unwrap_or(Value::Null);
    let h = store.get("hrv", date).unwrap_or(Value::Null);
    let hs = hrv(&h);
    let rl = string(readiness(&r), &[&["level"], &["trainingReadinessLevel"]]);
    let ratio = num(&t, &[&["acuteChronicWorkloadRatio"], &["acwrPercent"]]).map(|v| {
        if v > 10. {
            v / 100.
        } else {
            v
        }
    });
    let call = rl.as_deref().map(|s| rules::effort(s, ratio));
    let factors = [
        ("sleep", "Sleep", Icon::Moon, "sleepScoreFactorPercent"),
        (
            "recovery",
            "Recovery",
            Icon::Clock,
            "recoveryTimeFactorPercent",
        ),
        ("hrv", "HRV", Icon::Hrv, "hrvFactorPercent"),
        ("acuteLoad", "Load", Icon::Flame, "acwrFactorPercent"),
        (
            "sleepHistory",
            "Sleep history",
            Icon::Moon,
            "sleepHistoryFactorPercent",
        ),
        (
            "stressHistory",
            "Stress",
            Icon::Heart,
            "stressHistoryFactorPercent",
        ),
    ];
    let mut factors: Vec<Factor> = factors
        .iter()
        .filter_map(|(key, name, _, field)| {
            num(readiness(&r), &[&[*field]]).map(|p| Factor {
                key: (*key).into(),
                name: (*name).into(),
                percent: Some(p),
                tone: rules::score(p),
                detail: format!("Garmin rates your {name} at {p:.0}%."),
            })
        })
        .collect();
    factors.sort_by(|a, b| {
        a.percent
            .unwrap_or(100.)
            .total_cmp(&b.percent.unwrap_or(100.))
    });
    let mut chips = vec![];
    if let Some(v) = num(sleep(&s), &[&["sleepTimeSeconds"]]) {
        chips.push(Chip {
            icon: Icon::Moon,
            value: Some(format!("{:.1}h", v / 3600.)),
            text: Some("sleep".into()),
            tone: sscore(&s).map(rules::score).unwrap_or(Tone::Neutral),
        });
    }
    if let Some(v) = num(hs, &[&["lastNightAvg"]]) {
        chips.push(Chip {
            icon: Icon::Hrv,
            value: Some(format!("{v:.0} ms")),
            text: Some("HRV".into()),
            tone: rules::level(hs["status"].as_str().unwrap_or("")),
        });
    }
    let load = acute(&t);
    if let Some(v) = load {
        chips.push(Chip {
            icon: Icon::Flame,
            value: Some(format!("{v:.0}")),
            text: Some("load".into()),
            tone: Tone::Neutral,
        });
    }
    let b = store.get("battery", date).unwrap_or(Value::Null);
    let b = b.as_array().and_then(|a| a.first()).unwrap_or(&b);
    let hstatus = string(hs, &[&["status"]]);
    TodayView {
        date: date.into(),
        readiness: rscore(&r).map(|v| {
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
                Tone::Fair,
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
        call_note: None,
        body_battery: BodyBatteryCard {
            now: num(b, &[&["bodyBatteryValue"], &["currentValue"]]),
            charged: num(b, &[&["charged"], &["chargedValue"]]),
            drained: num(b, &[&["drained"], &["drainedValue"]]),
        },
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
    let need = num(
        v,
        &[
            &["sleepNeed", "totalSleepNeedInSeconds"],
            &["sleepNeed", "totalSleepNeedSeconds"],
        ],
    )
    .map(|s| s / 60.);
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
    let tonight_need = need.or_else(|| {
        let ns: Vec<f64> = nights
            .iter()
            .filter_map(|(_, v)| {
                num(sleep(v), &[&["sleepNeed", "totalSleepNeedInSeconds"]]).map(|v| v / 60.)
            })
            .collect();
        (!ns.is_empty()).then(|| ns.iter().sum::<f64>() / ns.len() as f64)
    });
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
                        let start = num(item, &[&["startGMT"], &["startTimestamp"]])?;
                        let end = num(item, &[&["endGMT"], &["endTimestamp"]])?;
                        let first = num(levels.first()?, &[&["startGMT"], &["startTimestamp"]])?;
                        let divisor = if end - start > 100_000. { 60_000. } else { 60. };
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
                let history: Vec<f64> = nights
                    .iter()
                    .filter_map(|(_, night)| num(sleep(night), &[&[*key]]))
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
                num(v, &[&["averageHeartRate"]]),
            ),
            (
                OvernightKind::Respiration,
                "respiration",
                "br/min",
                num(v, &[&["averageRespirationValue"]]),
            ),
            (OvernightKind::Spo2, "SpO2", "%", num(v, &[&["avgSpO2"]])),
        ]
        .iter()
        .filter_map(|(kind, _, unit, value)| {
            value.map(|n| OvernightTile {
                kind: *kind,
                value: Some(n),
                unit: (*unit).into(),
                series: vec![],
            })
        })
        .collect(),
        consistency_spread_min: rules::normal(
            &windows.iter().map(|w| w.bed_hour * 60.).collect::<Vec<_>>(),
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
        tonight_bedtime: tonight_need.and_then(|n| rules::bedtime(&wakes, n)),
        tonight_need_min: tonight_need,
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
            acute(&store.get("training", &date)?).map(|load| DayLoad { date, load })
        })
        .collect();
    let m = store.get("max_metrics", date).unwrap_or(Value::Null);
    let focus = store.get("load_focus", date).unwrap_or(Value::Null);
    let load_focus = [
        ("lowAerobic", "Low aerobic"),
        ("highAerobic", "High aerobic"),
        ("anaerobic", "Anaerobic"),
    ]
    .iter()
    .filter_map(|(key, name)| {
        let data = &focus[*key];
        num(data, &[&["value"], &["load"]]).map(|value| {
            let target = match (num(data, &[&["optimalMin"]]), num(data, &[&["optimalMax"]])) {
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
    let vo2max = vo2(&m);
    let vo2max_trend: Vec<Option<f64>> = (0..5)
        .rev()
        .map(|week| {
            store
                .history("max_metrics", &date_sub(date, week * 7), 1)
                .pop()
                .and_then(|(_, v)| vo2(&v))
        })
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
                "Acute load",
                Tone::Fair,
                band(&v),
            )
        }),
        status: status.clone().map(|s| label(Some(s))),
        status_tone: status.as_deref().map(rules::level).unwrap_or(Tone::Neutral),
        headline: status
            .map(|s| label(Some(s)))
            .unwrap_or_else(|| "Training at a glance".into()),
        chips: vec![],
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
        running_tolerance_km: store
            .get("tolerance", date)
            .and_then(|v| num(&v, &[&["runningTolerance"], &["tolerance"]])),
        workouts,
    }
}
pub fn workout(store: &Store, id: &str) -> Option<WorkoutView> {
    let summary = activity(store, id)?;
    let v = store.get("activity", id).unwrap_or(Value::Null);
    let details = store.get("details", id).unwrap_or(Value::Null);
    let metrics = details["metricDescriptors"].as_array();
    let samples = details["activityDetailMetrics"].as_array();
    let index = |key: &str| metrics.and_then(|m| m.iter().position(|x| x["key"] == key));
    let hr = index("directHeartRate");
    let ts = index("directTimestamp");
    let lat = index("directLatitude");
    let lon = index("directLongitude");
    let points = samples.map(|v| v.as_slice()).unwrap_or(&[]);
    let mut heart_rate = vec![];
    let mut route = vec![];
    let stride = (points.len() / 300).max(1);
    for p in points.iter().step_by(stride) {
        let a = &p["metrics"];
        if let (Some(h), Some(t)) = (
            hr.and_then(|i| a[i].as_f64()),
            ts.and_then(|i| a[i].as_f64()),
        ) {
            heart_rate.push(HrSample { t_s: t, bpm: h });
        }
        if let (Some(y), Some(x)) = (
            lat.and_then(|i| a[i].as_f64()),
            lon.and_then(|i| a[i].as_f64()),
        ) {
            route.push([y, x]);
        }
    }
    let splits = store.get("splits", id).unwrap_or(Value::Null);
    Some(WorkoutView {
        summary,
        recovery_hours: num(&v, &[&["recoveryTime"]]).map(|s| s / 3600.),
        stats: [
            ("distance", "Distance", "km", 1000.),
            ("duration", "Time", "s", 1.),
            ("averageHR", "Avg HR", "bpm", 1.),
            ("calories", "Calories", "kcal", 1.),
            ("elevationGain", "Ascent", "m", 1.),
        ]
        .iter()
        .filter_map(|(key, label, unit, div)| {
            num(&v, &[&[*key]]).map(|v| Stat {
                key: (*key).into(),
                label: (*label).into(),
                value: v / div,
                unit: (*unit).into(),
            })
        })
        .collect(),
        route,
        heart_rate,
        zone_floors: vec![],
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
    let drivers = [
        ("vo2max", "VO2max", "ml/kg/min"),
        ("restingHr", "Resting HR", "bpm"),
        ("bmi", "BMI", ""),
        ("bodyFat", "Body fat", "%"),
        ("vigorousMinutes", "Vigorous minutes", "min"),
    ]
    .iter()
    .filter_map(|(key, name, unit)| {
        let data = &v["fitnessAgeData"][*key];
        let value = num(data, &[&["value"]]);
        value.map(|value| Driver {
            key: (*key).into(),
            name: (*name).into(),
            value: Some(value),
            unit: (*unit).into(),
            target: num(data, &[&["target"]]),
            met: num(data, &[&["target"]]).is_some_and(|t| value >= t),
            tone: Tone::Neutral,
        })
    })
    .collect();
    HealthView {
        fitness_age: age(store, date),
        lever: None,
        drivers,
        vitals: vitals(store, date),
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
    let sleep = sleep_view(store, &last);
    let sleep_gap = sleep
        .asleep_min
        .zip(sleep.need_min)
        .filter(|(_, need)| *need > 0.)
        .map(|(a, n)| ((n - a) / n).max(0.));
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
    let vo2max_change = training.vo2max_change_30d;
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
