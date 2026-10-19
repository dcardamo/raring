//! The week review (docs/spec.md, Rules: Week review): each week measured
//! against its own goals, why a week doesn't count, and next week's goals
//! proposed from counted weeks. Proposals change nothing until accepted.
use crate::{
    ledger,
    local::{self, LocalRecords},
    store::Store,
    view::*,
};
use chrono::{Duration, NaiveDate, Utc};

const HISTORY_WEEKS: i64 = 8;

pub(crate) struct Week {
    pub review: WeekReview,
    pub goals: Vec<GoalOutcome>,
    pub rings: Vec<Ring>,
}

fn days_with(local: &LocalRecords, start: NaiveDate, reason: Option<PauseReason>) -> u32 {
    (0..7)
        .filter(|i| {
            let d = (start + Duration::days(*i)).to_string();
            local.circumstances.pauses.iter().any(|p| {
                reason.is_none_or(|r| p.reason == r) && p.since <= d && p.until.as_deref().is_none_or(|u| d.as_str() <= u)
            })
        })
        .count() as u32
}

pub(crate) fn week(store: &Store, local: &LocalRecords, start: NaiveDate, today: NaiveDate) -> Week {
    let board = ledger::board_with(store, local, start, today, Utc::now());
    let ill = days_with(local, start, Some(PauseReason::Ill));
    let paused = days_with(local, start, None) - ill;
    let rest = board.days.iter().filter(|d| local.circumstances.rest.iter().any(|r| r.date == d.date)).count() as u32;
    let counted = start + Duration::days(6) <= today && board.coverage_complete && ill + paused <= 1;
    let mut reasons = vec![];
    if start + Duration::days(6) > today { reasons.push("Week still in progress".into()); }
    if !board.coverage_complete {
        reasons.push("Garmin's activity list is incomplete".to_owned());
    }
    let plural = |n: u32, one: &str, many: &str| if n == 1 { format!("1 {one}") } else { format!("{n} {many}") };
    if ill > 0 {
        reasons.push(plural(ill, "ill day", "ill days"));
    }
    if paused > 0 {
        reasons.push(format!("paused {}", plural(paused, "day", "days")));
    }
    if rest > 0 {
        reasons.push(plural(rest, "rest day", "rest days"));
    }
    let goals: Vec<GoalOutcome> = board
        .ledger
        .iter()
        .map(|l| GoalOutcome {
            kind: l.kind,
            target: l.target,
            recorded: l.recorded,
            state: if l.observation {
                OutcomeState::Observed
            } else if !counted || l.recorded.is_none() {
                OutcomeState::NotCounted
            } else if l.recorded.unwrap_or(0.) >= l.target {
                OutcomeState::Met
            } else if l.recorded.unwrap_or(0.) > 0. {
                OutcomeState::Partly
            } else {
                OutcomeState::None
            },
        })
        .collect();
    let next = start + Duration::days(7);
    Week {
        review: WeekReview {
            start: board.start.clone(),
            end: board.end.clone(),
            counted,
            reasons,
            ill_days: ill,
            paused_days: paused,
            rest_days: rest,
            not_recorded: board.items.iter().filter(|p| p.state == ItemState::NotRecorded).count() as u32,
            goals: goals.clone(),
            proposals: vec![],
            next_start: next.to_string(),
            decision: local.reviews.decisions.iter().find(|d| d.week == board.start).cloned(),
        },
        goals,
        rings: board.week_rings,
    }
}

fn round5(n: f64) -> f64 {
    (n / 5.).round() * 5.
}

/// Next week's proposals after reviewing the week starting `start`.
fn propose(store: &Store, local: &LocalRecords, start: NaiveDate, today: NaiveDate, last: &Week) -> Vec<GoalProposal> {
    let Some(version) = local::goals_in_week(&local.goals, start, start + Duration::days(6)) else {
        return vec![];
    };
    let first_goal = local.goals.versions.first().map(|v| v.from.clone()).unwrap_or_default();
    let history: Vec<Week> = (0..HISTORY_WEEKS)
        .map(|i| start - Duration::days(7 * i))
        .filter(|w| (*w + Duration::days(6)).to_string() >= first_goal)
        .map(|w| if w == start { Week { review: last.review.clone(), goals: last.goals.clone(), rings: last.rings.clone() } } else { week(store, local, w, today) })
        .collect();
    let t = &version.targets;
    let kinds: Vec<(GoalKind, f64, bool, f64)> = [
        (GoalKind::Strength, t.strength_sessions, false, 5.),
        (GoalKind::Zone2, t.zone2_min, true, 3000.),
        (GoalKind::HighIntensity, t.high_intensity_min, true, 3000.),
        (GoalKind::Intensity, if t.intensity_garmin_goal { 0 } else { t.intensity_min }, true, 3000.),
        (GoalKind::SleepTiming, if t.sleep_kind == SleepTimingKind::SavedPlan { 0 } else { t.sleep_nights }, false, 7.),
    ]
    .into_iter()
    .filter(|(_, n, _, _)| *n > 0)
    .map(|(k, n, minutes, max)| (k, f64::from(n), minutes, max))
    .collect();
    kinds
        .into_iter()
        .map(|(kind, current, minutes, max)| {
            let out = |proposed: f64, reason: String| GoalProposal { kind, current, proposed, reason };
            let ease = if minutes { (round5(current * 0.9)).max(5.) } else { (current - 1.).max(1.) };
            if last.review.ill_days >= 2 {
                return out(ease, format!("Easing back after {} ill days", last.review.ill_days));
            }
            let runs: Vec<&GoalOutcome> = history
                .iter()
                .filter(|w| w.review.counted)
                .filter_map(|w| w.goals.iter().find(|g| g.kind == kind && g.recorded.is_some()))
                .collect();
            if runs.len() < 2 {
                return out(current, "Not enough complete weeks to suggest a change".into());
            }
            let met = |g: &GoalOutcome| g.recorded.unwrap_or(0.) >= g.target;
            let comfortable = if minutes { runs[0].recorded.unwrap_or(0.) >= runs[0].target * 1.1 } else { runs.len() >= 3 && runs[..3].iter().all(|g| met(g)) };
            if met(runs[0]) && met(runs[1]) && comfortable {
                let up = if minutes { (current + 20.).min(round5(current * 1.1)) } else { (current + 1.).min(max) };
                if up > current {
                    let reason = if minutes {
                        format!("Met in your last two complete weeks, {:.0} of {:.0} min last week", runs[0].recorded.unwrap_or(0.), runs[0].target)
                    } else {
                        "Met in your last three complete weeks".into()
                    };
                    return out(up, reason);
                }
            }
            let window = &runs[..runs.len().min(3)];
            let low = window.iter().filter(|g| if minutes { g.recorded.unwrap_or(0.) < g.target * 0.7 } else { !met(g) }).count();
            if low >= 2 && ease < current {
                return out(ease, format!("Under target in {low} of your last {} complete weeks", window.len()));
            }
            out(current, if met(runs[0]) { "Met last week; keep it steady".into() } else { "Close enough to keep".into() })
        })
        .collect()
}

/// The review of the plan week containing `anchor`.
pub fn week_review(store: &Store, anchor: NaiveDate, today: NaiveDate) -> WeekReview {
    let local = local::load(store);
    let start = crate::plan::week_start(anchor, crate::plan::week_starts_on(store));
    let last = week(store, &local, start, today);
    let proposals = propose(store, &local, start, today, &last);
    WeekReview { proposals, ..last.review }
}

/// The proposals a decision on the week starting `start` may accept.
pub fn proposals(store: &Store, start: NaiveDate, today: NaiveDate) -> Vec<GoalProposal> {
    week_review(store, start, today).proposals
}

/// Past weeks with goals, newest first, for the review list.
pub fn list(store: &Store, today: NaiveDate, weeks: i64) -> Vec<ReviewRow> {
    let local = local::load(store);
    let first = crate::plan::week_starts_on(store);
    let this = crate::plan::week_start(today, first);
    let first_goal = local.goals.versions.first().map(|v| v.from.clone());
    (1..=weeks)
        .map(|i| this - Duration::days(7 * i))
        .filter(|w| first_goal.as_deref().is_some_and(|g| (*w + Duration::days(6)).to_string().as_str() >= g))
        .map(|w| {
            let wk = week(store, &local, w, today);
            ReviewRow {
                start: wk.review.start,
                counted: wk.review.counted,
                met: wk.goals.iter().filter(|g| g.state == OutcomeState::Met).count() as u32,
                goals: wk.goals.iter().filter(|g| g.state != OutcomeState::Observed).count() as u32,
                reasons: wk.review.reasons,
            }
        })
        .collect()
}
