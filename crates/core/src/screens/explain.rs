//! The "Why" sheets and Today's reconciliation line, built only from Garmin's
//! own factors, events and feedback. docs/spec.md lists the wording rules
//! under Rules ("Why sheets" and "Scores that differ").
use super::{hrv, humanize_code, label, load_device, num, readiness, string, training_device};
use crate::{rules, view::*};
use chrono::{Duration, NaiveDateTime};
use serde_json::Value;

/// Lowercase a name for the middle of a sentence, keeping acronyms ("HRV").
fn lower(name: &str) -> String {
    let mut chars = name.chars();
    match (chars.next(), chars.next()) {
        (Some(_), Some(second)) if second.is_ascii_uppercase() => name.to_owned(),
        _ => {
            let mut chars = name.chars();
            chars
                .next()
                .map(|c| c.to_lowercase().collect::<String>() + chars.as_str())
                .unwrap_or_default()
        }
    }
}
fn capitalize(text: &str) -> String {
    let mut chars = text.chars();
    chars
        .next()
        .map(|c| c.to_uppercase().collect::<String>() + chars.as_str())
        .unwrap_or_default()
}
fn and_list(names: &[String]) -> String {
    match names {
        [] => String::new(),
        [one] => one.clone(),
        [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
    }
}
fn verb(names: &[String]) -> &'static str {
    if names.len() == 1 {
        "is"
    } else {
        "are"
    }
}
/// Garmin's rating code in words; "NONE" means Garmin did not rate it.
fn rating(code: Option<&str>) -> Option<String> {
    code.filter(|c| !c.is_empty() && !c.eq_ignore_ascii_case("NONE"))
        .and_then(humanize_code)
}
fn article(word: &str) -> &'static str {
    if word.starts_with(['a', 'e', 'i', 'o', 'u', 'A', 'E', 'I', 'O', 'U']) {
        "an"
    } else {
        "a"
    }
}
fn signed(value: f64) -> String {
    if value > 0. {
        format!("+{value:.0}")
    } else if value < 0. {
        format!("−{:.0}", -value)
    } else {
        "0".into()
    }
}

// Readiness ------------------------------------------------------------------

fn readiness_input(key: &str) -> Option<(&'static str, &'static str)> {
    Some(match key {
        "sleep" => ("sleepScoreFactorFeedback", "Last night's sleep score."),
        "recovery" => (
            "recoveryTimeFactorFeedback",
            "Hours until Garmin expects you to be recovered from recent workouts.",
        ),
        "hrv" => (
            "hrvFactorFeedback",
            "Your 7-day HRV average against your baseline.",
        ),
        "acuteLoad" => (
            "acwrFactorFeedback",
            "Your last 7 days of training load against what you're used to.",
        ),
        "sleepHistory" => (
            "sleepHistoryFactorFeedback",
            "Your sleep over the last three nights against your need.",
        ),
        "stressHistory" => (
            "stressHistoryFactorFeedback",
            "Your stress over the last three days.",
        ),
        _ => return None,
    })
}

/// The factors that hold readiness back and the ones that help it, by
/// Garmin's ratings, at most two each: Poor ones hold it back (Moderate ones
/// too when none is Poor and readiness isn't High or Prime); Good ones help.
fn pulls(factors: &[Factor], level: Option<&str>) -> (Vec<String>, Vec<String>) {
    let high = level.is_some_and(|l| matches!(l.to_ascii_uppercase().as_str(), "HIGH" | "PRIME"));
    let rated = |tone: Tone| {
        let mut picked: Vec<&Factor> = factors
            .iter()
            .filter(|f| f.tone == tone && f.percent.is_some())
            .collect();
        picked.sort_by(|a, b| a.percent.unwrap_or(0.).total_cmp(&b.percent.unwrap_or(0.)));
        picked
    };
    let mut down = rated(Tone::Poor);
    if down.is_empty() && !high {
        down = rated(Tone::Fair);
    }
    let mut up = rated(Tone::Good);
    up.reverse();
    let names = |picked: Vec<&Factor>| picked.iter().take(2).map(|f| lower(&f.name)).collect();
    (names(down), names(up))
}

pub(super) fn readiness_why(r: &Value, factors: &[Factor], level: Option<&str>) -> Option<Why> {
    if factors.is_empty() {
        return None;
    }
    let snapshot = readiness(r);
    let inputs = factors
        .iter()
        .filter_map(|f| {
            let (feedback, detail) = readiness_input(&f.key)?;
            let rated = f.percent.is_some();
            let value = match f.key.as_str() {
                "sleep" => num(snapshot, &[&["sleepScore"]]).map(|v| format!("{v:.0}")),
                "recovery" => num(snapshot, &[&["recoveryTime"]])
                    .filter(|m| *m >= 0.)
                    .map(|m| format!("{:.0} h", m / 60.)),
                "hrv" => num(snapshot, &[&["hrvWeeklyAverage"]]).map(|v| format!("{v:.0} ms")),
                "acuteLoad" => num(snapshot, &[&["acuteLoad"]]).map(|v| format!("{v:.0}")),
                _ => None,
            };
            Some(WhyInput {
                key: f.key.clone(),
                name: f.name.clone(),
                value: value.filter(|_| rated),
                rating: rating(snapshot[feedback].as_str()),
                tone: f.tone,
                detail: if rated {
                    detail.into()
                } else {
                    format!("{detail} Garmin didn't rate it today, so it's left out.")
                },
            })
        })
        .collect();
    let (down, up) = pulls(factors, level);
    let summary = match (down.is_empty(), up.is_empty()) {
        (false, false) => format!(
            "{} {} holding readiness back; {} {} helping.",
            capitalize(&and_list(&down)),
            verb(&down),
            and_list(&up),
            verb(&up)
        ),
        (false, true) => format!(
            "{} {} holding readiness back.",
            capitalize(&and_list(&down)),
            verb(&down)
        ),
        (true, false) => format!(
            "{} {} helping readiness most.",
            capitalize(&and_list(&up)),
            verb(&up)
        ),
        (true, true) => "No single factor stands out today.".into(),
    };
    Some(Why {
        title: level
            .map(|l| format!("Why readiness is {}", label(Some(l.into()))))
            .unwrap_or_else(|| "Why readiness".into()),
        summary,
        feedback: string(snapshot, &[&["feedbackShort"]]).and_then(|c| rating(Some(&c))),
        inputs,
        source: "From Garmin's readiness factors, worst first. Raring adds no score.".into(),
    })
}

// Body Battery ---------------------------------------------------------------

struct BatteryEvent {
    kind: String,
    impact: f64,
    start: Option<NaiveDateTime>,
    minutes: Option<f64>,
    name: Option<String>,
    feedback: Option<String>,
}

fn battery_events(battery: &Value, summary: &Value) -> Vec<BatteryEvent> {
    // The daily summary's list names each activity; the report's does not.
    let list = summary["bodyBatteryActivityEventList"]
        .as_array()
        .filter(|a| !a.is_empty())
        .or_else(|| battery["bodyBatteryActivityEvent"].as_array());
    let mut events: Vec<BatteryEvent> = list
        .into_iter()
        .flatten()
        .filter_map(|e| {
            let impact = num(e, &[&["bodyBatteryImpact"]])?;
            // Garmin's event times are GMT with the wearer's offset alongside.
            let start = string(e, &[&["eventStartTimeGmt"]])
                .and_then(|s| NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S%.f").ok())
                .map(|t| {
                    t + Duration::milliseconds(num(e, &[&["timezoneOffset"]]).unwrap_or(0.) as i64)
                });
            Some(BatteryEvent {
                kind: string(e, &[&["eventType"]]).unwrap_or_default(),
                impact,
                start,
                minutes: num(e, &[&["durationInMilliseconds"]]).map(|ms| ms / 60_000.),
                name: string(e, &[&["activityName"]]).filter(|n| !n.trim().is_empty()),
                feedback: string(e, &[&["shortFeedback"]]),
            })
        })
        .collect();
    events.sort_by_key(|e| e.start);
    events
}

/// Garmin's dynamic Body Battery feedback ("DAY_STRESSFUL_AND_INACTIVE",
/// "SLEEP_PREPARATION_BALANCED_AND_ACTIVE_AND_EXERCISE_AND_BB_LOW") in words.
/// Codes outside the shapes seen on real accounts are left out.
fn battery_feedback(code: &str) -> Option<String> {
    let (rest, day) = match code.strip_prefix("DAY_") {
        Some(rest) => (rest, true),
        None => (
            code.strip_prefix("SLEEP_PREPARATION_")
                .or_else(|| code.strip_prefix("SLEEP_TIME_PASSED_"))?,
            false,
        ),
    };
    let mut stress = None;
    let mut activity = None;
    let mut exercise = None;
    for part in rest.split("_AND_") {
        match part {
            "STRESSFUL" => stress = Some("stressful"),
            "BALANCED" => stress = Some("balanced"),
            "RECOVERING" => stress = Some("restful"),
            "INACTIVE" => activity = Some("inactive"),
            "ACTIVE" => activity = Some("active"),
            "EXERCISE" => exercise = Some("with exercise"),
            "INTENSIVE_EXERCISE" => exercise = Some("with hard exercise"),
            "NOT_STRESS_DATA" | "BB_LOW" => {}
            _ => return None,
        }
    }
    let words: Vec<&str> = [stress, activity].into_iter().flatten().collect();
    if words.is_empty() && exercise.is_none() {
        return None;
    }
    let text = words.join(" and ");
    let noun = if day { "so far" } else { "day" };
    let lead = match (text.is_empty(), day) {
        (true, true) => "Today so far".to_owned(),
        (true, false) => "A day".to_owned(),
        (false, true) => format!("{} {noun}", capitalize(&text)),
        (false, false) => format!("{} {text} {noun}", capitalize(article(&text))),
    };
    Some(match exercise {
        Some(exercise) => format!("{lead}, {exercise}"),
        None => lead,
    })
}

pub(super) fn body_battery_why(
    battery: &Value,
    summary: &Value,
    now: Option<f64>,
    stress: &StressCard,
) -> Option<Why> {
    let events = battery_events(battery, summary);
    let sleep = events.iter().find(|e| e.kind == "SLEEP");
    let overnight = sleep
        .map(|e| e.impact)
        .or_else(|| num(summary, &[&["bodyBatteryDuringSleep"]]));
    let at_wake = num(summary, &[&["bodyBatteryAtWakeTime"]]).filter(|v| (0. ..=100.).contains(v));
    let clock = |t: NaiveDateTime| t.format("%H:%M").to_string();
    let mut inputs = vec![];
    if let Some(charge) = overnight {
        let window = sleep.and_then(|e| {
            let start = e.start?;
            Some(format!(
                ", {}–{}",
                clock(start),
                clock(start + Duration::minutes(e.minutes? as i64))
            ))
        });
        inputs.push(WhyInput {
            key: "overnight".into(),
            name: "Overnight".into(),
            value: Some(signed(charge)),
            rating: None,
            tone: at_wake.map(rules::battery).unwrap_or(Tone::Neutral),
            detail: format!(
                "Charge while you slept{}{}.",
                window.unwrap_or_default(),
                at_wake
                    .map(|w| format!("; {w:.0} at wake"))
                    .unwrap_or_default()
            ),
        });
    }
    if let Some(average) = stress.average {
        inputs.push(WhyInput {
            key: "stress".into(),
            name: "Stress".into(),
            value: Some(format!("{average:.0}")),
            rating: stress.qualifier.clone(),
            tone: stress.tone,
            detail: "Today's average. Stress drains Body Battery while you're awake and still."
                .into(),
        });
    }
    for e in events.iter().filter(|e| e.kind != "SLEEP") {
        let (key, name) = match e.kind.as_str() {
            "ACTIVITY" => (
                "activity",
                e.name.clone().unwrap_or_else(|| "Workout".into()),
            ),
            "NAP" => ("nap", "Nap".into()),
            "RECOVERY" => ("rest", "Restful period".into()),
            other => (
                "event",
                humanize_code(other).unwrap_or_else(|| "Event".into()),
            ),
        };
        let feedback = rating(e.feedback.as_deref())
            .filter(|f| !f.eq_ignore_ascii_case(&name) && !f.eq_ignore_ascii_case(&e.kind));
        inputs.push(WhyInput {
            key: key.into(),
            name,
            value: Some(signed(e.impact)),
            rating: feedback,
            tone: Tone::Neutral,
            detail: [e.start.map(clock), e.minutes.map(|m| format!("{m:.0} min"))]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
                .join(" · "),
        });
    }
    if inputs.is_empty() {
        return None;
    }
    let workouts: Vec<&BatteryEvent> = events
        .iter()
        .filter(|e| e.kind == "ACTIVITY" && e.impact < 0.)
        .collect();
    let mut drains = vec![];
    match workouts.as_slice() {
        [] => {}
        [one] => drains.push(one.name.clone().unwrap_or_else(|| "a workout".into())),
        many => drains.push(format!("{} workouts", many.len())),
    }
    if let Some(qualifier) = &stress.qualifier {
        let day = format!("{} day", lower(qualifier));
        drains.push(format!("{} {day}", article(&day)));
    }
    let mut summary_text = match overnight {
        Some(charge) if charge > 0. => format!(
            "Sleep charged it by {charge:.0}{}.",
            at_wake
                .map(|w| format!(", to {w:.0} at wake"))
                .unwrap_or_default()
        ),
        Some(_) => "Sleep didn't charge it.".into(),
        None => String::new(),
    };
    if !drains.is_empty() {
        let lead = if summary_text.is_empty() {
            "Today"
        } else {
            " Since then"
        };
        summary_text.push_str(&format!("{lead}: {}.", and_list(&drains)));
    }
    let feedback = string(
        summary,
        &[&["bodyBatteryDynamicFeedbackEvent", "feedbackShortType"]],
    )
    .or_else(|| {
        string(
            battery,
            &[&["bodyBatteryDynamicFeedbackEvent", "feedbackShortType"]],
        )
    })
    .and_then(|c| battery_feedback(&c));
    Some(Why {
        title: now
            .map(|n| format!("Why Body Battery is {n:.0}"))
            .unwrap_or_else(|| "Why Body Battery".into()),
        summary: if summary_text.is_empty() {
            "Garmin hasn't reported today's charge yet.".into()
        } else {
            summary_text
        },
        feedback,
        inputs,
        source: "From Garmin's Body Battery events and feedback.".into(),
    })
}

// Training status ------------------------------------------------------------

fn status_key(status: &str) -> String {
    label(Some(status.into()))
        .to_ascii_uppercase()
        .replace(' ', "_")
}

fn status_meaning(key: &str) -> Option<&'static str> {
    Some(match key {
        "PRODUCTIVE" => "Your load is building fitness.",
        "MAINTAINING" => "Your load is keeping your current fitness.",
        "RECOVERY" => "A lighter load is giving your body time to recover.",
        "PEAKING" => "Fitness is high and your load is tapering.",
        "UNPRODUCTIVE" => "Your load is steady, but fitness is slipping.",
        "DETRAINING" => "A week or more of light load is costing fitness.",
        "OVERREACHING" => "Your load is very high and may be working against you.",
        "STRAINED" => "Low HRV says your body is struggling to absorb training.",
        "NO_STATUS" => "Garmin needs more workouts to set a status.",
        "PAUSED" => "Training status is paused.",
        _ => return None,
    })
}

/// Garmin's load-balance phrase ("AEROBIC_HIGH_SHORTAGE") in words.
fn focus_phrase(code: &str) -> Option<(String, Tone)> {
    let code = code
        .replace("AEROBIC_HIGH", "HIGH_AEROBIC")
        .replace("AEROBIC_LOW", "LOW_AEROBIC");
    let tone = if code.contains("SHORTAGE") {
        Tone::Fair
    } else if code.contains("BALANCED") {
        Tone::Good
    } else {
        Tone::Neutral
    };
    Some((humanize_code(&code)?, tone))
}

pub(super) struct TrainingInputs<'a> {
    pub training: &'a Value,
    pub focus: &'a Value,
    pub hrv: &'a Value,
    pub status: Option<&'a str>,
    pub acute: Option<&'a Gauge>,
    pub ratio: Option<f64>,
    pub load_focus: &'a [LoadFocusBar],
    pub vo2max: Option<f64>,
    pub vo2max_change_30d: Option<f64>,
}

pub(super) fn training_why(t: TrainingInputs) -> Option<Why> {
    let device = training_device(t.training);
    let paused = device["trainingPaused"].as_bool() == Some(true);
    let status = t.status?;
    let key = if paused {
        "PAUSED".into()
    } else {
        status_key(status)
    };
    let mut inputs = vec![];
    let acwr = string(&device["acuteTrainingLoadDTO"], &[&["acwrStatus"]]);
    if t.acute.is_some() || acwr.is_some() {
        let tone = match acwr.as_deref() {
            Some("OPTIMAL") => Tone::Good,
            Some("LOW") => Tone::Fair,
            Some("HIGH" | "VERY_HIGH") => Tone::Poor,
            _ => t.acute.map(|g| g.tone).unwrap_or(Tone::Neutral),
        };
        let value = [
            t.acute.map(|g| format!("{:.0}", g.value)),
            t.ratio.map(|r| format!("ratio {r:.1}")),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(" · ");
        inputs.push(WhyInput {
            key: "acuteLoad".into(),
            name: "Acute load".into(),
            value: (!value.is_empty()).then_some(value),
            rating: rating(acwr.as_deref())
                .or_else(|| t.acute.map(|g| g.label.clone()).filter(|l| !l.is_empty())),
            tone,
            detail: "Your last 7 days of load against your 4-week load.".into(),
        });
    }
    let phrase = string(load_device(t.focus), &[&["trainingBalanceFeedbackPhrase"]])
        .or_else(|| {
            string(
                load_device(t.training),
                &[&["trainingBalanceFeedbackPhrase"]],
            )
        })
        .and_then(|p| focus_phrase(&p));
    let shortages: Vec<String> = t
        .load_focus
        .iter()
        .filter(|b| b.shortage)
        .map(|b| format!("{} shortage", b.name))
        .collect();
    let focus = phrase.or_else(|| {
        (!t.load_focus.is_empty()).then(|| {
            if shortages.is_empty() {
                ("In range".into(), Tone::Good)
            } else {
                (capitalize(&lower(&and_list(&shortages))), Tone::Fair)
            }
        })
    });
    if let Some((text, tone)) = focus {
        inputs.push(WhyInput {
            key: "loadFocus".into(),
            name: "Load focus".into(),
            value: None,
            rating: Some(text),
            tone,
            detail: "How your last 4 weeks split across low aerobic, high aerobic and anaerobic."
                .into(),
        });
    }
    let hs = hrv(t.hrv);
    if let Some(raw) = string(hs, &[&["status"]]) {
        inputs.push(WhyInput {
            key: "hrvStatus".into(),
            name: "HRV status".into(),
            value: num(hs, &[&["weeklyAvg"]]).map(|v| format!("{v:.0} ms")),
            rating: Some(label(Some(raw.clone()))),
            tone: rules::level(&raw),
            detail: "Your 7-day HRV average against your baseline.".into(),
        });
    }
    if let Some(vo2) = t.vo2max {
        let change = t.vo2max_change_30d.map(|d| {
            if d.abs() >= 1. {
                format!(" {} in 30 days.", signed(d))
            } else {
                " Steady over 30 days.".into()
            }
        });
        inputs.push(WhyInput {
            key: "vo2max".into(),
            name: "VO₂ max".into(),
            value: Some(format!("{vo2:.0}")),
            rating: None,
            tone: Tone::Neutral,
            detail: format!("Garmin's fitness estimate.{}", change.unwrap_or_default()),
        });
    }
    let rank = |tone: Tone| match tone {
        Tone::Poor => 0,
        Tone::Fair => 1,
        Tone::Good => 2,
        Tone::Neutral => 3,
    };
    inputs.sort_by_key(|i| rank(i.tone));
    let concerns: Vec<String> = inputs
        .iter()
        .filter(|i| matches!(i.tone, Tone::Poor | Tone::Fair))
        .filter_map(|i| {
            let rating = lower(i.rating.as_deref()?);
            Some(match i.key.as_str() {
                "loadFocus" => format!("load focus shows {} {rating}", article(&rating)),
                _ => format!("{} is {rating}", lower(&i.name)),
            })
        })
        .take(2)
        .collect();
    let mut summary = status_meaning(&key).map(str::to_owned).unwrap_or_default();
    if !concerns.is_empty() {
        let clause = capitalize(&concerns.join("; "));
        summary = [summary, format!("{clause}.")].join(" ").trim().to_owned();
    }
    if summary.is_empty() && inputs.is_empty() {
        return None;
    }
    let shown = label(Some(status.into()));
    Some(Why {
        title: format!("Why {}", if paused { "paused".into() } else { shown }),
        summary: if summary.is_empty() {
            "Garmin weighs these inputs to set your status.".into()
        } else {
            summary
        },
        feedback: None,
        inputs,
        source: "From Garmin's training status inputs. Raring adds no score.".into(),
    })
}

// Scores that differ ---------------------------------------------------------

pub(super) struct Scores<'a> {
    pub readiness: Option<&'a Gauge>,
    pub level: Option<&'a str>,
    pub factors: &'a [Factor],
    pub sleep: Option<&'a Gauge>,
    pub battery_at_wake: Option<f64>,
    pub hrv_status: Option<&'a str>,
    pub training_status: Option<&'a str>,
}

/// Readiness and another score "point different ways" when one is Good and
/// the other Poor; the first such score in this order is explained.
pub(super) fn reconcile(s: Scores) -> Option<Reconciliation> {
    let readiness = s.readiness?.tone;
    if !matches!(readiness, Tone::Good | Tone::Poor) {
        return None;
    }
    let mut others: Vec<(&str, &str, Tone, String)> = vec![];
    if let Some(sleep) = s.sleep {
        others.push((
            "sleep",
            "sleep",
            sleep.tone,
            format!("Sleep scored {:.0}", sleep.value),
        ));
    }
    if let Some(wake) = s.battery_at_wake {
        let tone = rules::battery(wake);
        let text = if tone == Tone::Good {
            format!("Body Battery charged to {wake:.0} overnight")
        } else {
            format!("Body Battery was {wake:.0} at wake")
        };
        others.push(("bodyBattery", "Body Battery", tone, text));
    }
    if let Some(raw) = s.hrv_status {
        others.push((
            "hrvStatus",
            "HRV status",
            rules::level(raw),
            format!("HRV status is {}", lower(&label(Some(raw.into())))),
        ));
    }
    if let Some(status) = s.training_status {
        let shown = label(Some(status.into()));
        others.push((
            "trainingStatus",
            "training status",
            rules::level(&shown),
            format!("Training status, built over weeks, is {}", lower(&shown)),
        ));
    }
    let opposite = if readiness == Tone::Good {
        Tone::Poor
    } else {
        Tone::Good
    };
    let (key, name, tone, text) = others.into_iter().find(|o| o.2 == opposite)?;
    let (down, up) = pulls(s.factors, s.level);
    let rated: Vec<&Factor> = s.factors.iter().filter(|f| f.percent.is_some()).collect();
    let line = if readiness == Tone::Poor {
        let names = if down.is_empty() {
            rated
                .first()
                .map(|f| vec![lower(&f.name)])
                .unwrap_or_default()
        } else {
            down
        };
        if names.is_empty() {
            format!("{text}, but readiness is low.")
        } else {
            format!(
                "{text}, but readiness is held back by {}.",
                and_list(&names)
            )
        }
    } else {
        let names = if up.is_empty() {
            rated
                .last()
                .map(|f| vec![lower(&f.name)])
                .unwrap_or_default()
        } else {
            up
        };
        if names.is_empty() {
            format!("{text}, but readiness is high.")
        } else {
            format!("{text}, but readiness is held up by {}.", and_list(&names))
        }
    };
    Some(Reconciliation {
        key: key.into(),
        label: format!("Why {name} and readiness differ"),
        tones: [tone, readiness],
        line,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn battery_feedback_phrases() {
        for (code, text) in [
            (
                "DAY_STRESSFUL_AND_INACTIVE",
                Some("Stressful and inactive so far"),
            ),
            (
                "DAY_STRESSFUL_AND_INACTIVE_AND_BB_LOW",
                Some("Stressful and inactive so far"),
            ),
            (
                "SLEEP_PREPARATION_STRESSFUL_AND_ACTIVE_AND_INTENSIVE_EXERCISE",
                Some("A stressful and active day, with hard exercise"),
            ),
            (
                "SLEEP_TIME_PASSED_NOT_STRESS_DATA_AND_INACTIVE",
                Some("An inactive day"),
            ),
            (
                "SLEEP_PREPARATION_RECOVERING_AND_EXERCISE",
                Some("A restful day, with exercise"),
            ),
            (
                "SLEEP_PREPARATION_NOT_STRESS_DATA_AND_INTENSIVE_EXERCISE",
                Some("A day, with hard exercise"),
            ),
            (
                "DAY_BALANCED_AND_ACTIVE_AND_EXERCISE",
                Some("Balanced and active so far, with exercise"),
            ),
            ("NO_DATA", None),
            ("DAY_SOMETHING_NEW", None),
        ] {
            assert_eq!(battery_feedback(code).as_deref(), text, "{code}");
        }
    }

    #[test]
    fn words() {
        assert_eq!(lower("HRV"), "HRV");
        assert_eq!(lower("Recovery time"), "recovery time");
        assert_eq!(and_list(&["a".into(), "b".into()]), "a and b");
        assert_eq!(signed(-7.), "−7");
        assert_eq!(
            focus_phrase("AEROBIC_HIGH_SHORTAGE"),
            Some(("High aerobic shortage".into(), Tone::Fair))
        );
        assert_eq!(status_key("PRODUCTIVE_6"), "PRODUCTIVE");
    }
}
