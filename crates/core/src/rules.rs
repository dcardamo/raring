//! Deliberately small set of derived decisions; all scores remain Garmin's.
use crate::view::{Effort, Tone};

pub fn effort(level: &str, ratio: Option<f64>) -> Effort {
    let value = match level.to_ascii_uppercase().as_str() {
        "PRIME" | "HIGH" => Effort::Push,
        "MODERATE" => Effort::Steady,
        "LOW" => Effort::Easy,
        "POOR" => Effort::Rest,
        _ => return Effort::Steady,
    };
    if ratio.is_some_and(|r| r > 1.3) && value == Effort::Push {
        Effort::Steady
    } else {
        value
    }
}
pub fn level(level: &str) -> Tone {
    match level.to_ascii_uppercase().as_str() {
        "PRIME" | "HIGH" | "BALANCED" | "PRODUCTIVE" | "MAINTAINING" | "OPTIMAL" => Tone::Good,
        "MODERATE" | "UNBALANCED" | "RECOVERY" | "PEAKING" => Tone::Fair,
        "LOW" | "POOR" | "STRAINED" | "UNPRODUCTIVE" | "OVERREACHING" => Tone::Poor,
        _ => Tone::Neutral,
    }
}
pub fn score(score: f64) -> Tone {
    if score >= 80.0 {
        Tone::Good
    } else if score >= 60.0 {
        Tone::Fair
    } else {
        Tone::Poor
    }
}
pub fn normal(values: &[f64]) -> Option<[f64; 2]> {
    if values.len() < 14 {
        return None;
    }
    let mut v: Vec<f64> = values.iter().copied().filter(|x| x.is_finite()).collect();
    if v.len() < 14 {
        return None;
    }
    v.sort_by(f64::total_cmp);
    let quantile = |p: f64| {
        let index = (v.len() - 1) as f64 * p;
        let lo = index.floor() as usize;
        let hi = index.ceil() as usize;
        v[lo] + (v[hi] - v[lo]) * (index - lo as f64)
    };
    Some([quantile(0.1), quantile(0.9)])
}
pub fn bedtime(wakes: &[f64], need: f64) -> Option<String> {
    if wakes.is_empty() || !need.is_finite() {
        return None;
    }
    let mut v: Vec<f64> = wakes.iter().copied().filter(|x| x.is_finite()).collect();
    if v.is_empty() {
        return None;
    }
    v.sort_by(f64::total_cmp);
    let middle = (v[(v.len() - 1) / 2] + v[v.len() / 2]) / 2.0;
    let minute = ((middle * 60.0 - need).round() as i64).rem_euclid(1440);
    Some(format!("{:02}:{:02}", minute / 60, minute % 60))
}
pub fn biggest_drag<'a>(factors: &'a [(&'a str, f64)]) -> Option<&'a str> {
    factors
        .iter()
        .filter(|(_, v)| v.is_finite())
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(name, _)| *name)
}
pub fn weekly_focus(
    sleep_gap: Option<f64>,
    load_gap: Option<f64>,
    readiness_drop: Option<f64>,
    vigorous_gap: Option<f64>,
) -> Option<&'static str> {
    [
        ("Sleep", sleep_gap),
        ("Load focus", load_gap),
        ("Readiness", readiness_drop),
        ("Vigorous minutes", vigorous_gap),
    ]
    .into_iter()
    .filter_map(|(name, v)| v.filter(|x| *x > 0.0).map(|x| (name, x)))
    .max_by(|a, b| a.1.total_cmp(&b.1))
    .map(|(name, _)| name)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn effort_table() {
        for (s, r, e) in [
            ("PRIME", None, Effort::Push),
            ("HIGH", Some(1.4), Effort::Steady),
            ("HIGH", Some(1.2), Effort::Push),
            ("MODERATE", None, Effort::Steady),
            ("LOW", None, Effort::Easy),
            ("POOR", None, Effort::Rest),
        ] {
            assert_eq!(effort(s, r), e)
        }
    }
    #[test]
    fn ranges() {
        assert_eq!(normal(&[2.; 13]), None);
        let range = normal(&(0..14).map(f64::from).collect::<Vec<_>>()).unwrap();
        assert!((range[0] - 1.3).abs() < 1e-9 && (range[1] - 11.7).abs() < 1e-9);
        assert_eq!(
            normal(&(0..30).map(f64::from).collect::<Vec<_>>()),
            Some([2.9000000000000004, 26.1])
        );
        assert_eq!(bedtime(&[7., 8., 9.], 480.), Some("00:00".into()));
        assert_eq!(bedtime(&[7., 8.], 450.), Some("00:00".into()));
    }
    #[test]
    fn decisions() {
        assert_eq!(biggest_drag(&[("sleep", 70.), ("load", 34.)]), Some("load"));
        assert_eq!(
            weekly_focus(Some(0.2), Some(0.1), None, None),
            Some("Sleep")
        );
        assert_eq!(
            weekly_focus(Some(0.1), Some(0.5), Some(0.2), Some(0.3)),
            Some("Load focus")
        );
        assert_eq!(weekly_focus(None, None, None, None), None);
    }
}
