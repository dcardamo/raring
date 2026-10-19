//! Training's week lanes (docs/spec.md, Rules: Week lanes): cumulative work
//! per goal kind this plan week against the goal and the usual week.
use crate::{ledger, local::LocalRecords, review, store::Store, view::*};
use chrono::{DateTime, Duration, NaiveDate, Utc};

/// Linear-interpolated percentile of a non-empty list.
pub fn percentile(xs: &[f64], p: f64) -> f64 {
    let mut s = xs.to_vec();
    s.sort_by(f64::total_cmp);
    let i = (s.len() - 1) as f64 * p;
    let (lo, hi) = (i.floor() as usize, i.ceil() as usize);
    s[lo] + (s[hi] - s[lo]) * (i - lo as f64)
}

/// One day's amount for a lane; None when that day's evidence is unknown.
fn day_amount(store: &Store, kind: LaneKind, recs: &[&Recording], date: NaiveDate) -> Option<f64> {
    let sum = |f: fn(&Recording) -> Option<f64>| recs.iter().map(|r| f(r)).sum::<Option<f64>>();
    match kind {
        LaneKind::Cardio => Some(recs.iter().filter(|r| !matches!(r.sport, Sport::Strength | Sport::Other)).map(|r| r.duration_min).sum()),
        LaneKind::Zone2 => sum(|r| r.zone2_min),
        LaneKind::HighIntensity => sum(|r| r.high_intensity_min),
        LaneKind::Intensity => ledger::daily_intensity(store, date),
        LaneKind::Strength => Some(recs.iter().filter(|r| r.sport == Sport::Strength).count() as f64),
    }
}

/// Cumulative Monday → `until` (inclusive); later days None, and None from
/// the first unknown day.
fn cumulative(store: &Store, kind: LaneKind, board: &PlanBoard, start: NaiveDate, until: NaiveDate) -> Vec<Option<f64>> {
    let mut total = Some(0.);
    (0..7)
        .map(|i| {
            let d = start + Duration::days(i);
            if d > until {
                return None;
            }
            let day = d.to_string();
            let recs: Vec<&Recording> = board.recordings.iter().filter(|r| r.date == day).collect();
            // Recorded lanes need the day's activity list; intensity is Garmin's own daily total.
            let known = kind == LaneKind::Intensity || crate::sync::activities_listed(store, d, d);
            total = total.zip(day_amount(store, kind, &recs, d).filter(|_| known)).map(|(a, b)| a + b);
            total
        })
        .collect()
}

pub fn week(store: &Store, local: &LocalRecords, today: NaiveDate, now: DateTime<Utc>) -> TrainingWeek {
    let board = ledger::board_with(store, local, today, today, now);
    let start = NaiveDate::parse_from_str(&board.start, "%Y-%m-%d").unwrap_or(today);
    let t = board.goals.as_ref().map(|g| g.targets.clone());
    let target = |k: LaneKind| -> f64 {
        t.as_ref().map_or(0., |t| match k {
            LaneKind::Zone2 => t.zone2_min as f64,
            LaneKind::HighIntensity => t.high_intensity_min as f64,
            LaneKind::Intensity => {
                if t.intensity_garmin_goal { board.garmin_intensity_goal.unwrap_or(0.) } else { t.intensity_min as f64 }
            }
            LaneKind::Strength => t.strength_sessions as f64,
            LaneKind::Cardio => 0.,
        })
    };
    let mut kinds: Vec<LaneKind> = [LaneKind::Zone2, LaneKind::HighIntensity, LaneKind::Intensity].into_iter().filter(|k| target(*k) > 0.).collect();
    if kinds.is_empty() {
        kinds.push(LaneKind::Cardio);
    }
    let recent_strength = store.activities_between(&(today - Duration::days(56)).to_string(), &today.to_string()).iter().any(|(id, _)| {
        crate::local::sport_of(&crate::screens::activity_kind(store, id).unwrap_or_default()) == Sport::Strength
    });
    if target(LaneKind::Strength) > 0. || recent_strength {
        kinds.push(LaneKind::Strength);
    }
    // Up to eight earlier weeks, each counted by the week review's rule.
    let past: Vec<(NaiveDate, PlanBoard)> = (1..=8)
        .map(|i| start - Duration::days(7 * i))
        .filter(|w| review::week(store, local, *w, today).review.counted)
        .map(|w| (w, ledger::board_with(store, local, w, today, now)))
        .collect();
    let lanes = kinds
        .into_iter()
        .map(|kind| {
            let weeks: Vec<Vec<f64>> = past
                .iter()
                .filter_map(|(w, b)| cumulative(store, kind, b, *w, *w + Duration::days(6)).into_iter().collect::<Option<Vec<f64>>>())
                .collect();
            let band = |p: f64| (0..7).map(|i| percentile(&weeks.iter().map(|w| w[i]).collect::<Vec<_>>(), p)).collect();
            WeekLane {
                kind,
                goal: Some(target(kind)).filter(|g| *g > 0.),
                cumulative: cumulative(store, kind, &board, start, today),
                usual: (weeks.len() >= 3).then(|| UsualBand { low: band(0.25), median: band(0.5), high: band(0.75) }),
                weeks: weeks.len() as u32,
            }
        })
        .collect();
    let mut by_sport: Vec<SportMinutes> = vec![];
    for r in board.recordings.iter().filter(|r| r.date.as_str() <= board.today.as_str()) {
        match by_sport.iter_mut().find(|m| m.sport == r.sport) {
            Some(m) => m.minutes += r.duration_min,
            None => by_sport.push(SportMinutes { sport: r.sport, minutes: r.duration_min }),
        }
    }
    by_sport.sort_by(|a, b| b.minutes.total_cmp(&a.minutes));
    TrainingWeek { start: board.start.clone(), today: today.to_string(), lanes, by_sport }
}
