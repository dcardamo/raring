//! Garmin's word for each day's stress. It colours the stress calendar and
//! decides which days the stress trend leaves out of its averages
//! (docs/spec.md, Rules: Trends and Stress calendar).
use crate::{
    store::Store,
    view::{NightStress, StressDay, StressDayKind, StressMonth, Tone, TrendMetric},
};
use chrono::{Datelike, Duration, Months, NaiveDate};
use std::collections::{BTreeMap, BTreeSet};

/// Garmin's word for a day.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Word {
    Calm,
    Balanced,
    Stressful,
    VeryStressful,
}

impl Word {
    pub fn label(self) -> &'static str {
        match self {
            Word::Calm => "Calm",
            Word::Balanced => "Balanced",
            Word::Stressful => "Stressful",
            Word::VeryStressful => "Very Stressful",
        }
    }
    pub fn tone(self) -> Tone {
        match self {
            Word::Calm | Word::Balanced => Tone::Good,
            Word::Stressful => Tone::Fair,
            Word::VeryStressful => Tone::Poor,
        }
    }
}

/// How much of the day Garmin's word covers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Coverage {
    /// The whole day, sleep included.
    Day(Word),
    /// Only the time awake: Garmin measured no sleep (`BALANCED_AWAKE`).
    AwakeOnly(Word),
    /// Garmin couldn't rate the day (`UNKNOWN`): the watch was off for much of it.
    LowWear,
}

impl Coverage {
    /// Garmin rated only part of the day, so its average isn't comparable
    /// with whole days.
    pub fn partial(self) -> bool {
        !matches!(self, Coverage::Day(_))
    }
}

/// Garmin's qualifier, as the daily summary (`STRESSFUL`) or a history row
/// (`stressful`) spells it. A missing or unfamiliar word makes no claim.
pub fn coverage(qualifier: &str) -> Option<Coverage> {
    let code = qualifier.trim().to_ascii_uppercase();
    if code == "UNKNOWN" {
        return Some(Coverage::LowWear);
    }
    let (base, awake) = match code.strip_suffix("_AWAKE") {
        Some(base) => (base, true),
        None => (code.as_str(), false),
    };
    let word = match base {
        "CALM" => Word::Calm,
        "BALANCED" => Word::Balanced,
        "STRESSFUL" => Word::Stressful,
        "VERY_STRESSFUL" => Word::VeryStressful,
        _ => return None,
    };
    Some(if awake {
        Coverage::AwakeOnly(word)
    } else {
        Coverage::Day(word)
    })
}

/// Each stored day's coverage from `from` to `to` (YYYY-MM-DD). The daily
/// summary wins over the history row behind it.
pub fn coverage_between(store: &Store, from: &str, to: &str) -> BTreeMap<NaiveDate, Coverage> {
    let mut days = BTreeMap::new();
    for (endpoint, path) in [
        ("daily_summary", "$.stress.qualifier"),
        ("summary", "$.stressQualifier"),
    ] {
        for (key, qualifier) in store.day_strings(endpoint, path, from, to) {
            let date = key
                .get(..10)
                .and_then(|k| NaiveDate::parse_from_str(k, "%Y-%m-%d").ok());
            if let (Some(date), Some(coverage)) = (date, coverage(&qualifier)) {
                days.insert(date, coverage);
            }
        }
    }
    days
}

/// Days Garmin rated only in part: too little wear, or awake only.
pub fn partial_days(store: &Store, from: &str, to: &str) -> BTreeSet<NaiveDate> {
    coverage_between(store, from, to)
        .into_iter()
        .filter(|(_, coverage)| coverage.partial())
        .map(|(date, _)| date)
        .collect()
}

/// The calendar month holding `month` (YYYY-MM or a date in it), read from
/// local rows only; days after `today` are future.
pub fn month(store: &Store, month: &str, today: &str) -> Result<StressMonth, String> {
    let date = |s: &str| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok();
    let today = date(today).ok_or_else(|| format!("Invalid date: {today}"))?;
    let first = month
        .get(..7)
        .and_then(|m| date(&format!("{m}-01")))
        .ok_or_else(|| format!("Invalid month: {month}"))?;
    if first > today {
        return Err(format!("Choose a month no later than this one ({today})"));
    }
    if first < NaiveDate::from_ymd_opt(1990, 1, 1).unwrap_or_default() {
        return Err("Choose a month from January 1990 on".into());
    }
    let last = (first + Months::new(1) - Duration::days(1)).min(today);
    let (from, to) = (first.to_string(), last.to_string());
    let coverage = coverage_between(store, &from, &to);
    let averages = crate::trends::readings(store, TrendMetric::Stress, first, last);
    let nights = nights(store, &from, &to);
    let mut workouts: BTreeMap<String, u32> = BTreeMap::new();
    for (id, start) in store.activities_between(&from, &to) {
        if let (Some(day), Some(_)) = (start.get(..10), crate::screens::activity_brief(store, &id)) {
            *workouts.entry(day.to_owned()).or_default() += 1;
        }
    }
    let days = first
        .iter_days()
        .take_while(|d| d.month() == first.month())
        .map(|day| {
            let average = averages.get(&day).copied().filter(|_| day <= today);
            let (kind, word, tone) = match (coverage.get(&day), average) {
                _ if day > today => (StressDayKind::Future, None, Tone::Neutral),
                (Some(Coverage::Day(word)), _) => (StressDayKind::Rated, Some(word), word.tone()),
                (Some(Coverage::AwakeOnly(word)), _) => (StressDayKind::AwakeOnly, Some(word), Tone::Neutral),
                (Some(Coverage::LowWear), Some(_)) => (StressDayKind::LowWear, None, Tone::Neutral),
                (_, Some(_)) => (StressDayKind::Unrated, None, Tone::Neutral),
                _ => (StressDayKind::None, None, Tone::Neutral),
            };
            let date = day.to_string();
            StressDay {
                workouts: workouts.get(&date).copied().unwrap_or(0),
                night: nights.get(&date).cloned(),
                date,
                kind,
                word: word.map(|w| w.label().into()),
                tone,
                average,
            }
        })
        .collect();
    Ok(StressMonth {
        month: from,
        today: today.to_string(),
        days,
        first: store
            .first_key(&["stress", "summary", "daily_summary"])
            .and_then(|key| key.get(..7).map(|m| format!("{m}-01"))),
    })
}

/// Garmin's rating of each night's stress from its sleep score, keyed by the
/// morning the night ended; the night's own record wins over history.
fn nights(store: &Store, from: &str, to: &str) -> BTreeMap<String, NightStress> {
    let mut nights: BTreeMap<String, NightStress> = BTreeMap::new();
    for (endpoint, prefix) in [("sleep_summary", "$"), ("sleep", "$.dailySleepDTO")] {
        let ratings: BTreeMap<String, String> = store
            .day_strings(endpoint, &format!("{prefix}.sleepScores.stress.qualifierKey"), from, to)
            .into_iter()
            .collect();
        let averages: BTreeMap<String, f64> = store
            .day_numbers(endpoint, &[&format!("{prefix}.avgSleepStress")], from, to)
            .into_iter()
            .filter(|(_, v)| (0. ..=100.).contains(v))
            .collect();
        for key in ratings.keys().chain(averages.keys()) {
            let Some(date) = key.get(..10) else { continue };
            let rating = ratings.get(key).map(|r| r.trim().to_ascii_uppercase()).filter(|r| !r.is_empty());
            nights.insert(
                date.to_owned(),
                NightStress {
                    tone: match rating.as_deref() {
                        Some("EXCELLENT" | "GOOD") => Tone::Good,
                        Some("FAIR") => Tone::Fair,
                        Some("POOR") => Tone::Poor,
                        _ => Tone::Neutral,
                    },
                    rating: rating.map(|r| {
                        let mut chars = r.chars();
                        chars.next().map(String::from).unwrap_or_default() + &chars.as_str().to_ascii_lowercase()
                    }),
                    average: averages.get(key).copied(),
                },
            );
        }
    }
    nights
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn garmin_words_in_either_spelling() {
        assert_eq!(coverage("BALANCED"), Some(Coverage::Day(Word::Balanced)));
        assert_eq!(coverage("very_stressful"), Some(Coverage::Day(Word::VeryStressful)));
        assert_eq!(coverage("VERY_STRESSFUL_AWAKE"), Some(Coverage::AwakeOnly(Word::VeryStressful)));
        assert_eq!(coverage("calm_awake"), Some(Coverage::AwakeOnly(Word::Calm)));
        assert_eq!(coverage("UNKNOWN"), Some(Coverage::LowWear));
        assert_eq!(coverage("unknown"), Some(Coverage::LowWear));
        assert_eq!(coverage(""), None);
        assert_eq!(coverage("RELAXED"), None);
        assert!(Coverage::LowWear.partial() && Coverage::AwakeOnly(Word::Calm).partial());
        assert!(!Coverage::Day(Word::Stressful).partial());
    }
}
