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
/// Body Battery's display bucket: Garmin rates no level, so only the color is ours.
pub fn battery(value: f64) -> Tone {
    if value >= 70.0 {
        Tone::Good
    } else if value >= 30.0 {
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
/// Recorded wake times read as clock times on a 24-hour circle, so a steady
/// 23:55/00:05 pattern is one cluster rather than a noon median.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WakeClock {
    /// The usual wake, hours from the wake day's midnight, between 18:00 the
    /// evening before (-6) and 18:00 (18): a midnight cluster stays on one
    /// side of the date line instead of flipping a whole day.
    pub median: f64,
    /// The middle half's width, hours.
    pub spread: f64,
    pub count: usize,
}
/// Each time is unwrapped to within 12 hours of the circular mean before the
/// median and quartiles are taken, which leaves ordinary mornings unchanged.
pub fn wake_clock(hours: &[f64]) -> Option<WakeClock> {
    let v: Vec<f64> = hours
        .iter()
        .copied()
        .filter(|h| h.is_finite())
        .map(|h| h.rem_euclid(24.))
        .collect();
    if v.is_empty() {
        return None;
    }
    let angle = |h: f64| h / 24. * std::f64::consts::TAU;
    let (sin, cos) = v
        .iter()
        .fold((0., 0.), |(s, c), h| (s + angle(*h).sin(), c + angle(*h).cos()));
    let mean = sin.atan2(cos).rem_euclid(std::f64::consts::TAU) / std::f64::consts::TAU * 24.;
    let mut unwrapped: Vec<f64> = v
        .iter()
        .map(|h| mean - 12. + (h - (mean - 12.)).rem_euclid(24.))
        .collect();
    unwrapped.sort_by(f64::total_cmp);
    let quantile = |p: f64| {
        let index = (unwrapped.len() - 1) as f64 * p;
        let (lo, hi) = (index.floor() as usize, index.ceil() as usize);
        unwrapped[lo] + (unwrapped[hi] - unwrapped[lo]) * (index - lo as f64)
    };
    Some(WakeClock {
        median: (quantile(0.5) + 6.).rem_euclid(24.) - 6.,
        spread: quantile(0.75) - quantile(0.25),
        count: v.len(),
    })
}
/// Minutes from the wake day's midnight to the usual wake time minus the
/// need; negative is the evening before.
pub fn bedtime_minutes(wakes: &[f64], need: f64) -> Option<i64> {
    if !need.is_finite() {
        return None;
    }
    let usual = wake_clock(wakes)?;
    Some((usual.median * 60.0 - need).round() as i64)
}
pub fn clock(minutes: i64) -> String {
    let minute = minutes.rem_euclid(1440);
    format!("{:02}:{:02}", minute / 60, minute % 60)
}
pub fn bedtime(wakes: &[f64], need: f64) -> Option<String> {
    bedtime_minutes(wakes, need).map(clock)
}
fn near(wake: Option<&str>) -> String {
    wake.map(|w| format!(" near {w}")).unwrap_or_default()
}
/// One line toward steadier timing for tonight's plan. Garmin's alignment
/// status leads; without it, the usual wake time does. Never just "go to bed
/// earlier".
pub fn steady_tip(status: Option<&str>, wake: Option<&str>) -> Option<String> {
    let near = near(wake);
    match status {
        Some("ALIGNED") => Some(format!(
            "Your sleep lines up with Garmin's optimal window. Keep waking{near}."
        )),
        Some("BEHIND") => Some(format!(
            "Sleep has run later than Garmin's optimal window. Shift gradually: keep waking{near} rather than forcing one early night."
        )),
        Some("AHEAD") => Some(format!(
            "Sleep has run earlier than Garmin's optimal window. Keep waking{near}."
        )),
        _ => wake.map(|_| format!("Keep waking{near}, weekends included.")),
    }
}
/// The weekly check-in's one timing focus: Garmin's alignment ratings when at
/// least three nights have one, otherwise how much wake times varied.
pub fn timing_focus(
    aligned: u32,
    ahead: u32,
    behind: u32,
    wake_spread_min: Option<f64>,
    wake: Option<&str>,
) -> Option<String> {
    let near = near(wake);
    let rated = aligned + ahead + behind;
    if rated >= 3 {
        return Some(if aligned * 2 >= rated {
            format!("Garmin rated most nights aligned. Keep waking{near}.")
        } else if behind >= ahead {
            format!("Most nights ran later than Garmin's optimal window. Keep a steady wake time{near}, weekends included, and let bedtime follow.")
        } else {
            format!("Most nights ran earlier than Garmin's optimal window. Keep a steady wake time{near}.")
        });
    }
    let spread = wake_spread_min?;
    Some(if spread >= 60. {
        let spread = spread.round() as i64;
        format!(
            "Wake times varied by {}h {:02}m. Aim for a steady wake{near}.",
            spread / 60,
            spread % 60
        )
    } else {
        format!("Wake times held steady. Keep them{near}.")
    })
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
    intensity_gap: Option<f64>,
) -> Option<&'static str> {
    [
        ("Sleep", sleep_gap),
        ("Load focus", load_gap),
        ("Readiness", readiness_drop),
        ("Intensity minutes", intensity_gap),
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
        assert_eq!(bedtime_minutes(&[7.25], 480.), Some(-45));
        assert_eq!(bedtime_minutes(&[10.5], 440.), Some(190));
        assert_eq!(clock(-45), "23:15");
        assert_eq!(clock(1450), "00:10");
    }
    #[test]
    fn wake_times_are_read_on_a_clock_circle() {
        let near = |a: f64, b: f64| (a - b).abs() < 1e-6;
        // An ordinary morning: the plain median and middle half.
        let morning = wake_clock(&[6.5, 7., 7.25, 7.5, 8.]).unwrap();
        assert!(near(morning.median, 7.25) && near(morning.spread, 0.5));
        // A steady pattern around midnight is one tight cluster.
        let midnight = wake_clock(&[23.9, 0.1, 23.95, 0.05, 0.]).unwrap();
        assert!(near(midnight.median, 0.) && midnight.spread < 0.2, "{midnight:?}");
        assert_eq!(bedtime(&[23.9, 0.1, 23.95, 0.05, 0.], 480.), Some("16:00".into()));
        assert_eq!(bedtime_minutes(&[23.9, 0.1, 23.95, 0.05, 0.], 480.), Some(-480));
        // Late-evening wakes sit on the evening before; afternoons stay on the day.
        assert!(near(wake_clock(&[23.5]).unwrap().median, -0.5));
        assert!(near(wake_clock(&[15.]).unwrap().median, 15.));
        // A truly broad spread stays broad.
        let broad = wake_clock(&[5., 5.5, 9., 13., 14., 6., 15., 10.]).unwrap();
        assert!(broad.spread > 3., "{broad:?}");
        assert_eq!(wake_clock(&[f64::NAN]), None);
    }
    #[test]
    fn timing_coaching_prefers_steady_wake() {
        assert_eq!(
            steady_tip(Some("BEHIND"), Some("07:00")).unwrap(),
            "Sleep has run later than Garmin's optimal window. Shift gradually: keep waking near 07:00 rather than forcing one early night."
        );
        assert_eq!(
            steady_tip(Some("ALIGNED"), None).unwrap(),
            "Your sleep lines up with Garmin's optimal window. Keep waking."
        );
        assert_eq!(
            steady_tip(None, Some("06:30")).unwrap(),
            "Keep waking near 06:30, weekends included."
        );
        assert_eq!(
            steady_tip(Some("JET_LAG"), Some("06:30")).unwrap(),
            "Keep waking near 06:30, weekends included."
        );
        assert_eq!(steady_tip(Some("JET_LAG"), None), None);
        assert_eq!(steady_tip(None, None), None);

        assert!(timing_focus(5, 0, 2, Some(200.), Some("07:00"))
            .unwrap()
            .starts_with("Garmin rated most nights aligned"));
        assert!(timing_focus(1, 1, 4, None, None)
            .unwrap()
            .starts_with("Most nights ran later"));
        assert!(timing_focus(1, 3, 0, None, None)
            .unwrap()
            .starts_with("Most nights ran earlier"));
        // Fewer than three rated nights fall back to wake-time spread.
        assert_eq!(
            timing_focus(1, 0, 1, Some(130.), Some("07:00")).unwrap(),
            "Wake times varied by 2h 10m. Aim for a steady wake near 07:00."
        );
        assert_eq!(
            timing_focus(0, 0, 0, Some(20.), Some("07:00")).unwrap(),
            "Wake times held steady. Keep them near 07:00."
        );
        assert_eq!(timing_focus(0, 0, 0, None, None), None);
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
