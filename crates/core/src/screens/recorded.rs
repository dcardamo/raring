//! A workout's Garmin summary fields in groups, what it did beyond training
//! effect, and the self-evaluation (docs/spec.md, Rules: Workout detail).
use super::ActivityPayload;
use serde_json::Value;
use crate::view::{ActivityCard, DistanceUnit, Feel, FieldGroup, Icon, RecordedGroup, SelfEvaluation, Stat};

pub(super) fn any(_: f64) -> bool {
    true
}
/// Garmin sends 0 for what a sport or sensor didn't record (an average
/// cadence or power of 0), so 0 is not a reading for these fields.
pub(super) fn positive(v: f64) -> bool {
    v > 0.
}
/// Garmin's 0–100 stress level; negative codes mean it couldn't measure.
fn level(v: f64) -> bool {
    (0. ..=100.).contains(&v)
}

/// A summary field under its activity-detail name, with the activity list's
/// names for it and the factor from each into `unit`.
struct Field {
    group: FieldGroup,
    key: &'static str,
    label: &'static str,
    unit: &'static str,
    listed: &'static [(&'static str, f64)],
    keep: fn(f64) -> bool,
}
const fn field(group: FieldGroup, key: &'static str, label: &'static str, unit: &'static str, listed: &'static [(&'static str, f64)], keep: fn(f64) -> bool) -> Field {
    Field { group, key, label, unit, listed, keep }
}
use FieldGroup::*;
/// Garmin's fields beyond the stats grid and What it did, in display order.
/// The list's names were read from real list pages; its stroke distance is
/// in centimetres where the detail's is in metres.
const FIELDS: &[Field] = &[
    field(Time, "movingDuration", "Moving time", "s", &[], positive),
    field(Time, "elapsedDuration", "Elapsed time", "s", &[], positive),
    field(Form, "steps", "Steps", "", &[], positive),
    field(Form, "maxRunCadence", "Max cadence", "spm", &[("maxRunningCadenceInStepsPerMinute", 1.)], positive),
    field(Form, "maxBikeCadence", "Max cadence", "rpm", &[("maxBikingCadenceInRevPerMinute", 1.)], positive),
    field(Form, "strideLength", "Stride length", "cm", &[("avgStrideLength", 1.)], positive),
    field(Form, "groundContactTime", "Ground contact time", "ms", &[("avgGroundContactTime", 1.)], positive),
    field(Form, "verticalOscillation", "Vertical oscillation", "cm", &[("avgVerticalOscillation", 1.)], positive),
    field(Form, "verticalRatio", "Vertical ratio", "%", &[("avgVerticalRatio", 1.)], positive),
    field(Form, "avgStepSpeedLoss", "Step speed loss", "cm/s", &[], positive),
    field(Form, "avgStepSpeedLossPercent", "Step speed loss, % of speed", "%", &[], positive),
    field(Power, "maxPower", "Max power", "W", &[], positive),
    field(Power, "normalizedPower", "Normalized power", "W", &[("normPower", 1.)], positive),
    field(Power, "intensityFactor", "Intensity factor", "", &[], positive),
    field(Power, "trainingStressScore", "Training Stress Score", "", &[], positive),
    field(Power, "maxPowerTwentyMinutes", "Best 20 min power", "W", &[("max20MinPower", 1.)], positive),
    field(Power, "functionalThresholdPower", "FTP", "W", &[], positive),
    field(Power, "leftBalance", "Left leg", "%", &[("avgLeftBalance", 1.)], positive),
    field(Power, "rightBalance", "Right leg", "%", &[], positive),
    field(Strokes, "activeLengths", "Lengths", "", &[], positive),
    field(Strokes, "averageSwolf", "Avg SWOLF", "", &[], positive),
    field(Strokes, "averageStrokes", "Strokes per length", "", &[("avgStrokes", 1.)], positive),
    // Swims and paddles name the same stroke rate differently.
    field(Strokes, "averageStrokeCadence", "Stroke rate", "strokes/min", &[("averageSwimCadenceInStrokesPerMinute", 1.)], positive),
    field(Strokes, "maxStrokeCadence", "Max stroke rate", "strokes/min", &[("maxSwimCadenceInStrokesPerMinute", 1.)], positive),
    field(Strokes, "averageStrokeDistance", "Distance per stroke", "m", &[("avgStrokeDistance", 0.01)], positive),
    field(Strokes, "totalNumberOfStrokes", "Strokes", "", &[("strokes", 1.)], positive),
    field(Elevation, "elevationLoss", "Descent", "m", &[], any),
    field(Elevation, "minElevation", "Lowest", "m", &[], any),
    field(Elevation, "maxElevation", "Highest", "m", &[], any),
    field(HeartRate, "recoveryHeartRate", "Recovery heart rate", "bpm", &[], positive),
    field(Breathing, "avgRespirationRate", "Average", "brpm", &[], positive),
    field(Breathing, "minRespirationRate", "Lowest", "brpm", &[], positive),
    field(Breathing, "maxRespirationRate", "Highest", "brpm", &[], positive),
    field(Stress, "startStress", "At start", "", &[], level),
    field(Stress, "endStress", "At end", "", &[], level),
    field(Stress, "avgStress", "Average", "", &[], level),
    field(Stress, "maxStress", "Highest", "", &[], level),
    field(Temperature, "averageTemperature", "Average", "°C", &[], any),
    field(Temperature, "minTemperature", "Lowest", "°C", &[], any),
    field(Temperature, "maxTemperature", "Highest", "°C", &[], any),
    field(Energy, "bmrCalories", "Resting calories", "kcal", &[], positive),
    field(Energy, "waterEstimated", "Estimated sweat loss", "ml", &[], positive),
    field(Energy, "caloriesConsumed", "Calories consumed", "kcal", &[], positive),
    field(Energy, "waterConsumed", "Fluid consumed", "ml", &[], positive),
];

fn number(x: &Value) -> Option<f64> {
    x.as_f64().or_else(|| x.as_str()?.parse().ok()).filter(|n| n.is_finite())
}
/// The first reading `keep` accepts: the activity detail's before the
/// list's, each under any of the field's names, so a list's 0 never hides
/// the detail's reading and a detail's 0 never hides the list's.
pub(super) fn reading(v: &ActivityPayload, key: &str, listed: &[(&str, f64)], keep: fn(f64) -> bool) -> Option<f64> {
    let names = || std::iter::once((key, 1.)).chain(listed.iter().copied());
    [&v.detail["summaryDTO"], &v.detail, &v.listed]
        .into_iter()
        .find_map(|source| names().find_map(|(name, factor)| number(&source[name]).map(|n| n * factor).filter(|n| keep(*n))))
}
fn value(v: &ActivityPayload, key: &str, listed: &[(&str, f64)]) -> Option<f64> {
    reading(v, key, listed, any)
}

pub(super) fn groups(v: &ActivityPayload, summary: &ActivityCard) -> Vec<RecordedGroup> {
    let ride = summary.icon == Icon::Bike;
    let mut stats: Vec<(FieldGroup, Stat)> = vec![];
    // Rides read speed, everything else pace, as the stats grid does; Garmin
    // sends no grade-adjusted speed for rides.
    let speeds = [
        ("averageMovingSpeed", if ride { "Moving speed" } else { "Moving pace" }),
        ("maxSpeed", if ride { "Max speed" } else { "Best pace" }),
        ("avgGradeAdjustedSpeed", "Grade-adjusted pace"),
    ];
    for (key, label) in speeds {
        if summary.distance_km.is_none() || ride && key == "avgGradeAdjustedSpeed" {
            continue;
        }
        if let Some(kmh) = reading(v, key, &[], positive).map(|mps| mps * 3.6) {
            let (value, unit) = if ride { (kmh, "km/h") } else { (3600. / kmh, "s/km") };
            stats.push((Pace, Stat { key: key.into(), label: label.into(), value, unit: unit.into() }));
        }
    }
    // Garmin stores the pool length in centimetres with the factor to the
    // pool's own unit (100 for metres); both come from the same record.
    let pool = [&v.detail["summaryDTO"], &v.detail, &v.listed].into_iter().find_map(|source| {
        let factor = number(&source["unitOfPoolLength"]["factor"]).filter(|f| *f > 0.).unwrap_or(1.);
        number(&source["poolLength"]).map(|l| l / factor).filter(|l| *l > 0.)
    });
    if let Some(length) = pool {
        let unit = if summary.distance_unit == DistanceUnit::Yd { "yd" } else { "m" };
        stats.push((Strokes, Stat { key: "poolLength".into(), label: "Pool length".into(), value: length, unit: unit.into() }));
    }
    for f in FIELDS {
        let skip = match f.key {
            // Equal to the time shown above when the timer never paused.
            "elapsedDuration" => value(v, "duration", &[]).zip(value(v, f.key, &[])).is_some_and(|(time, all)| (all - time).abs() < 1.),
            // Steps on foot; strength and indoor sessions without distance count stray steps.
            "steps" => summary.distance_km.is_none(),
            // A ride's strokes are pedal strokes.
            "totalNumberOfStrokes" => ride,
            _ => false,
        };
        if skip {
            continue;
        }
        if let Some(n) = reading(v, f.key, f.listed, f.keep) {
            stats.push((f.group, Stat { key: f.key.into(), label: f.label.into(), value: n, unit: f.unit.into() }));
        }
    }
    [Time, Pace, Form, Power, Strokes, Elevation, HeartRate, Breathing, Stress, Temperature, Energy]
        .into_iter()
        .filter_map(|group| {
            let fields: Vec<Stat> = stats.iter().filter(|(g, _)| *g == group).map(|(_, s)| s.clone()).collect();
            (!fields.is_empty()).then_some(RecordedGroup { group, fields })
        })
        .collect()
}

/// What it did beyond training effect, as Garmin sent it.
pub(super) fn did(v: &ActivityPayload) -> Vec<Stat> {
    [
        ("moderateIntensityMinutes", "Moderate intensity", "min", any as fn(f64) -> bool),
        ("vigorousIntensityMinutes", "Vigorous intensity", "min", any),
        ("differenceBodyBattery", "Body Battery", "points", any),
        ("beginPotentialStamina", "Stamina at start", "%", any),
        ("endPotentialStamina", "Stamina at end", "%", any),
        ("minAvailableStamina", "Lowest available stamina", "%", any),
    ]
    .into_iter()
    .filter_map(|(key, label, unit, keep)| {
        reading(v, key, &[], keep).map(|value| Stat { key: key.into(), label: label.into(), value, unit: unit.into() })
    })
    .collect()
}

/// Garmin's feel is 0–100 in five steps and perceived effort 1–10 stored ten
/// times over (the FIT session's `workout_feel` and `workout_rpe`).
pub(super) fn self_evaluation(v: &ActivityPayload) -> Option<SelfEvaluation> {
    let feel = value(v, "directWorkoutFeel", &[]).filter(|n| (0. ..=100.).contains(n)).map(|n| match (n / 25.).round() as u8 {
        0 => Feel::VeryWeak,
        1 => Feel::Weak,
        2 => Feel::Normal,
        3 => Feel::Strong,
        _ => Feel::VeryStrong,
    });
    // 0 is Garmin's "None": no effort was chosen.
    let effort = value(v, "directWorkoutRpe", &[]).filter(|n| (10. ..=100.).contains(n)).map(|n| (n / 10.).round() as u8);
    (feel.is_some() || effort.is_some()).then_some(SelfEvaluation { feel, effort })
}
