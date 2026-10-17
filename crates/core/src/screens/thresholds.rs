use super::{num, RecordedTrainingMetric, Store, Value};
use chrono::NaiveDate;

fn reading(
    kind: &str,
    value: Option<f64>,
    unit: &str,
    row: &Value,
    date_field: &str,
    through: &str,
) -> Option<RecordedTrainingMetric> {
    let value = value.filter(|v| v.is_finite() && *v >= 0.)?;
    let day = row[date_field].as_str()?;
    let recorded_on = day.get(..10)?;
    let date = NaiveDate::parse_from_str(recorded_on, "%Y-%m-%d").ok()?;
    if date > NaiveDate::parse_from_str(through, "%Y-%m-%d").ok()? {
        return None;
    }
    Some(RecordedTrainingMetric {
        kind: kind.into(),
        value,
        unit: unit.into(),
        recorded_on: recorded_on.into(),
        stale: row["isStale"].as_bool().unwrap_or(false),
    })
}

/// No request date is used as a measurement date. Entries can be separately
/// dated and arrive in an arbitrary order; keep the newest for each metric.
pub(super) fn project(
    lt: &Value,
    power: &Value,
    ftp: &Value,
    acclimation: &Value,
    through: &str,
    system: Option<&str>,
) -> Vec<RecordedTrainingMetric> {
    let mut out = Vec::new();
    let mut add = |metric: Option<RecordedTrainingMetric>| {
        if let Some(metric) = metric {
            if let Some(existing) = out
                .iter_mut()
                .find(|m: &&mut RecordedTrainingMetric| m.kind == metric.kind)
            {
                if existing.recorded_on < metric.recorded_on {
                    *existing = metric;
                }
            } else {
                out.push(metric);
            }
        }
    };
    if let Some(rows) = lt.as_array() {
        for row in rows {
            add(reading(
                "runningHr",
                num(row, &[&["heartRate"], &["hearRate"]]).filter(|v| *v > 0.),
                "bpm",
                row,
                "calendarDate",
                through,
            ));
            // Garmin's biometric speed is seconds per meter, not m/s.
            let speed = num(row, &[&["speed"]]).filter(|v| *v > 0.);
            let distance = match system {
                Some("metric") => Some((1000., "s/km")),
                Some(s) if s.starts_with("statute") => Some((1609.344, "s/mi")),
                _ => None,
            };
            if let (Some(speed), Some((meters, unit))) = (speed, distance) {
                add(reading(
                    "runningPace",
                    Some(meters * speed),
                    unit,
                    row,
                    "calendarDate",
                    through,
                ));
            }
        }
    }
    let powers: Vec<&Value> = match power {
        Value::Array(rows) => rows.iter().collect(),
        Value::Object(_) => vec![power],
        _ => vec![],
    };
    for row in powers {
        if row["isStale"] == true {
            continue;
        }
        if row["sport"]
            .as_str()
            .is_some_and(|s| !s.eq_ignore_ascii_case("running"))
        {
            continue;
        }
        add(reading(
            "runningPower",
            num(row, &[&["functionalThresholdPower"]]).filter(|v| *v > 0.),
            "W",
            row,
            "calendarDate",
            through,
        ));
    }
    let ftps: Vec<&Value> = match ftp {
        Value::Array(rows) => rows.iter().collect(),
        Value::Object(_) => vec![ftp],
        _ => vec![],
    };
    for row in ftps {
        if row["isStale"] == true
            || row["sport"]
                .as_str()
                .is_some_and(|s| !s.eq_ignore_ascii_case("cycling"))
        {
            continue;
        }
        add(reading(
            "cyclingFtp",
            num(row, &[&["functionalThresholdPower"]]).filter(|v| *v > 0.),
            "W",
            row,
            "calendarDate",
            through,
        ));
    }
    add(reading(
        "heat",
        num(acclimation, &[&["heatAcclimationPercentage"]]),
        "%",
        acclimation,
        "heatAcclimationDate",
        through,
    ));
    // Garmin's altitudeAcclimation is an elevation in meters, not a percentage.
    let altitude = match system {
        Some("metric") => Some((1., "m")),
        Some(s) if s.starts_with("statute") => Some((3.28084, "ft")),
        _ => None,
    };
    if let Some((factor, unit)) = altitude {
        add(reading(
            "altitude",
            num(acclimation, &[&["altitudeAcclimation"]]).map(|m| m * factor),
            unit,
            acclimation,
            "altitudeAcclimationDate",
            through,
        ));
    }
    out.sort_by_key(|m| match m.kind.as_str() {
        "runningHr" => 0, "runningPace" => 1, "runningPower" => 2,
        "cyclingFtp" => 3, "heat" => 4, _ => 5,
    });
    out
}

pub(super) fn from_store(store: &Store, through: &str) -> Vec<RecordedTrainingMetric> {
    let get = |kind| store.get(kind, "latest").unwrap_or(Value::Null);
    let settings = get_settings(store);
    project(
        &get("lactate_threshold"),
        &get("running_power_threshold"),
        &get("cycling_ftp"),
        &get("acclimation"),
        through,
        settings.as_deref(),
    )
}
fn get_settings(store: &Store) -> Option<String> {
    let settings = store.get("profile_settings", "me")?;
    crate::strength::reduce_settings(&settings)["measurementSystem"]
        .as_str()
        .map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn absent_calibrating_and_known_zero() {
        let rows = project(
            &json!([]),
            &json!([{"calendarDate":"2025-10-02","functionalThresholdPower":null,"isStale":false}]),
            &Value::Null,
            &json!({"heatAcclimationDate":"2025-10-03","heatAcclimationPercentage":0,"altitudeAcclimationDate":null,"altitudeAcclimation":null}),
            "2025-10-04",
            Some("metric"),
        );
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].kind, "heat");
        assert_eq!(rows[0].value, 0.);
        assert!(project(
            &Value::Null,
            &Value::Null,
            &Value::Null,
            &Value::Null,
            "2025-10-04",
            None
        )
        .is_empty());
    }
    #[test]
    fn separate_dates_latest_and_units() {
        let lt = json!([
            {"calendarDate":"2025-10-01","hearRate":160,"speed":null},
            {"calendarDate":"2025-10-03","hearRate":164,"speed":null},
            {"calendarDate":"2025-10-02T12:45:00.000","speed":0.25},
            {"calendarDate":"2025-10-05","speed":0.20}
        ]);
        let power = json!([{"calendarDate":"2025-10-02","sport":"Running","functionalThresholdPower":280,"isStale":false}]);
        let ftp = json!({"calendarDate":"2025-10-01","sport":"CYCLING","functionalThresholdPower":230,"isStale":false});
        let metric = project(
            &lt,
            &power,
            &ftp,
            &Value::Null,
            "2025-10-04",
            Some("metric"),
        );
        assert_eq!(
            metric.iter().find(|m| m.kind == "runningHr").unwrap().value,
            164.
        );
        assert_eq!(
            metric
                .iter()
                .find(|m| m.kind == "runningPace")
                .unwrap()
                .value,
            250.
        );
        assert_eq!(
            metric
                .iter()
                .find(|m| m.kind == "runningPace")
                .unwrap()
                .recorded_on,
            "2025-10-02"
        );
        assert_eq!(
            metric
                .iter()
                .find(|m| m.kind == "cyclingFtp")
                .unwrap()
                .value,
            230.
        );
        let miles = project(
            &lt,
            &power,
            &ftp,
            &Value::Null,
            "2025-10-04",
            Some("statute_us"),
        );
        assert_eq!(
            miles
                .iter()
                .find(|m| m.kind == "runningPace")
                .unwrap()
                .value,
            402.336
        );
        assert!(
            !project(&lt, &power, &ftp, &Value::Null, "2025-10-04", None)
                .iter()
                .any(|m| m.kind == "runningPace")
        );
    }
    #[test]
    fn one_metric_and_stale_power() {
        let only_ftp = project(
            &Value::Null,
            &Value::Null,
            &json!({"calendarDate":"2025-10-01","functionalThresholdPower":210,"isStale":true}),
            &json!({"altitudeAcclimationDate":"2025-10-02","altitudeAcclimation":37}),
            "2025-10-03",
            Some("metric"),
        );
        assert_eq!(only_ftp.len(), 1);
        assert_eq!(only_ftp[0].kind, "altitude");
        assert_eq!(only_ftp[0].value, 37.);
        assert_eq!(only_ftp[0].unit, "m");
    }
}
