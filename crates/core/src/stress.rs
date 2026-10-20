//! Garmin's word for each day's stress. It decides which days the stress
//! trend leaves out of its averages (docs/spec.md, Rules: Trends).
use crate::{store::Store, view::Tone};
use chrono::NaiveDate;
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
