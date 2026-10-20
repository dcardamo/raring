//! Progress for a period (docs/spec.md, Rules: Progress periods to
//! Milestones): known totals, honest comparisons, coverage, goals over time,
//! bests in available history and milestones awarded once.
use crate::{
    local::{self, LocalRecords, MilestoneRecord},
    ledger, plan, review, screens, strength, trends,
    store::Store,
    sync::activities_listed,
    view::*,
};
use chrono::{DateTime, Datelike, Duration, Months, NaiveDate, Utc};
use serde_json::Value;
use std::collections::BTreeMap;

const GOAL_WEEKS: i64 = 12;

fn month_start(d: NaiveDate) -> NaiveDate {
    d.with_day(1).unwrap_or(d)
}
fn month_end(d: NaiveDate) -> NaiveDate {
    month_start(d) + Months::new(1) - Duration::days(1)
}

/// First and last day of the period containing `anchor`.
pub fn bounds(store: &Store, period: Period, anchor: NaiveDate) -> (NaiveDate, NaiveDate) {
    match period {
        Period::Week => {
            let start = plan::week_start(anchor, plan::week_starts_on(store));
            (start, start + Duration::days(6))
        }
        Period::Month => (month_start(anchor), month_end(anchor)),
        Period::Quarter => (month_start(anchor) - Months::new(2), month_end(anchor)),
        Period::Year => (month_start(anchor) - Months::new(11), month_end(anchor)),
    }
}

fn previous_start(store: &Store, period: Period, from: NaiveDate) -> NaiveDate {
    match period {
        Period::Week => plan::week_start(from - Duration::days(7), plan::week_starts_on(store)),
        Period::Month => from - Months::new(1),
        Period::Quarter => from - Months::new(3),
        Period::Year => from - Months::new(12),
    }
}

struct Activity {
    id: String,
    date: NaiveDate,
    sport: Sport,
    minutes: f64,
    km: Option<f64>,
}

fn activities(store: &Store, from: NaiveDate, to: NaiveDate) -> Vec<Activity> {
    store
        .activities_between(&from.to_string(), &to.to_string())
        .into_iter()
        .filter_map(|(id, start)| {
            let date = NaiveDate::parse_from_str(start.get(..10)?, "%Y-%m-%d").ok()?;
            let (kind, minutes, km) = screens::activity_brief(store, &id)?;
            Some(Activity { id, date, sport: crate::local::sport_of(&kind), minutes, km })
        })
        .collect()
}

fn days(from: NaiveDate, to: NaiveDate) -> impl Iterator<Item = NaiveDate> {
    from.iter_days().take_while(move |d| *d <= to)
}

fn number(v: &Value, key: &str) -> Option<f64> {
    v[key].as_f64().filter(|n| n.is_finite() && *n >= 0.)
}

fn totals(store: &Store, acts: &[Activity], from: NaiveDate, end: NaiveDate) -> PeriodTotals {
    let inside: Vec<&Activity> = acts.iter().filter(|a| a.date >= from && a.date <= end).collect();
    let covered: Vec<NaiveDate> = days(from, end).filter(|d| activities_listed(store, *d, *d)).collect();
    let intensity: Vec<f64> = days(from, end)
        .filter_map(|d| {
            let s = store.get("summary", &d.to_string())?;
            Some(number(&s, "moderateIntensityMinutes")? + 2. * number(&s, "vigorousIntensityMinutes")?)
        })
        .collect();
    PeriodTotals {
        activities: inside.len() as u32,
        active_min: inside.iter().map(|a| a.minutes).sum::<f64>().round(),
        distance_km: (inside.iter().filter_map(|a| a.km).sum::<f64>() * 10.).round() / 10.,
        intensity_min: (!intensity.is_empty()).then(|| intensity.iter().sum()),
        intensity_days: intensity.len() as u32,
        active_days: covered.iter().filter(|d| inside.iter().any(|a| a.date == **d)).count() as u32,
        covered_days: covered.len() as u32,
        days: days(from, end).count() as u32,
    }
}

fn bucket(acts: &[Activity], store: &Store, from: NaiveDate, to: NaiveDate, end: NaiveDate) -> PeriodBucket {
    let mut by: BTreeMap<Sport, f64> = BTreeMap::new();
    for a in acts.iter().filter(|a| a.date >= from && a.date <= to.min(end)) {
        *by.entry(a.sport).or_default() += a.minutes;
    }
    let mut minutes: Vec<SportMinutes> = by.into_iter().map(|(sport, m)| SportMinutes { sport, minutes: m }).collect();
    minutes.sort_by(|a, b| b.minutes.total_cmp(&a.minutes));
    PeriodBucket {
        from: from.to_string(),
        to: to.to_string(),
        total: minutes.iter().map(|m| m.minutes).sum(),
        minutes,
        known: from <= end && days(from, to.min(end)).any(|d| activities_listed(store, d, d)),
        future: from > end,
    }
}

fn buckets(store: &Store, acts: &[Activity], period: Period, from: NaiveDate, to: NaiveDate, end: NaiveDate) -> Vec<PeriodBucket> {
    match period {
        Period::Week | Period::Month => days(from, to).map(|d| bucket(acts, store, d, d, end)).collect(),
        Period::Quarter => {
            let first = plan::week_starts_on(store);
            let mut out = vec![];
            let mut w = plan::week_start(from, first);
            while w <= to {
                out.push(bucket(acts, store, w.max(from), (w + Duration::days(6)).min(to), end));
                w += Duration::days(7);
            }
            out
        }
        Period::Year => {
            let mut out = vec![];
            let mut m = from;
            while m <= to {
                out.push(bucket(acts, store, m, month_end(m), end));
                m = m + Months::new(1);
            }
            out
        }
    }
}

fn calendar(store: &Store, local: &LocalRecords, acts: &[Activity], from: NaiveDate, to: NaiveDate, today: NaiveDate, history: Option<NaiveDate>) -> Vec<CalendarDay> {
    days(from, to)
        .map(|d| {
            let ds = d.to_string();
            let count = acts.iter().filter(|a| a.date == d).count() as u32;
            let covered = d <= today && activities_listed(store, d, d);
            let pause = local.circumstances.pauses.iter().find(|p| p.since <= ds && p.until.as_deref().is_none_or(|u| ds.as_str() <= u));
            let state = if d > today {
                CalendarState::Future
            } else if history.is_none_or(|h| d < h) && count == 0 {
                CalendarState::Before
            } else if !covered && count == 0 {
                CalendarState::Unknown
            } else if count > 0 {
                CalendarState::Active
            } else if local.circumstances.rest.iter().any(|r| r.date == ds) {
                CalendarState::Rest
            } else if let Some(p) = pause {
                if p.reason == PauseReason::Ill { CalendarState::Ill } else { CalendarState::Paused }
            } else {
                CalendarState::None
            };
            CalendarDay { date: ds, state, activities: count, covered }
        })
        .collect()
}

fn sleep_need(store: &Store, from: NaiveDate, end: NaiveDate) -> Option<SleepNeedSummary> {
    let mut nights = 0;
    let mut below = 0;
    for d in days(from, end) {
        let Some(raw) = screens::sleep_row(store, &d.to_string()) else { continue };
        let s = screens::sleep(&raw);
        let asleep = number(s, "sleepTimeSeconds").map(|v| v / 60.);
        let need = screens::sleep_need(&raw);
        if let (Some(a), Some(n)) = (asleep, need) {
            nights += 1;
            if a <= n - 60. {
                below += 1;
            }
        }
    }
    (nights > 0).then_some(SleepNeedSummary { nights, below_hour: below })
}

fn sport_name(s: Sport) -> &'static str {
    match s {
        Sport::Running => "run",
        Sport::Cycling => "ride",
        Sport::Walking => "walk",
        Sport::Hiking => "hike",
        Sport::Swimming => "swim",
        Sport::Strength => "strength session",
        Sport::Other => "activity",
    }
}

/// Evidence bounded by the selected day, in canonical units.
fn evidence(store: &Store, local: &LocalRecords, today: NaiveDate, history: Option<NaiveDate>, now: DateTime<Utc>) -> Vec<MilestoneRecord> {
    let mut records = vec![];
    for b in strength::all_time_bests(store, &today.to_string()) {
        let id = format!("best:{}:{}", b.key, b.reps);
        records.push(MilestoneRecord {
            id, value: b.grams, date: b.date, kind: Some(MilestoneKind::Best),
            title: format!("{} · best for {} reps", b.name, b.reps),
            route: Some(format!("workout/{}", b.activity)), activity: Some(b.activity), set_reps: Some(b.set_reps), ..Default::default()
        });
    }
    for b in strength::all_time_reps_bests(store, &today.to_string()) {
        let id = format!("reps:{}", b.key);
        records.push(MilestoneRecord {
            id, value: f64::from(b.reps), date: b.date, kind: Some(MilestoneKind::Reps),
            title: format!("{} · most reps in a set", b.name),
            route: Some(format!("workout/{}", b.activity)), activity: Some(b.activity), ..Default::default()
        });
    }
    if let Some(h) = history {
        let all = activities(store, h, today);
        let mut longest: BTreeMap<Sport, (&Activity, u32)> = BTreeMap::new();
        for a in all.iter().filter(|a| a.km.is_some() && !matches!(a.sport, Sport::Strength | Sport::Other)) {
            let e = longest.entry(a.sport).or_insert((a, 0));
            e.1 += 1;
            if a.km > e.0.km {
                e.0 = a;
            }
        }
        // A sport recorded a few times has a longest worth naming.
        for (sport, (a, _)) in longest.into_iter().filter(|(_, (_, n))| *n >= 3) {
            let id = format!("longest:{sport:?}").to_lowercase();
            let km = a.km.unwrap_or(0.);
            records.push(MilestoneRecord {
                id, value: km, date: a.date.to_string(), activity: Some(a.id.clone()),
                kind: Some(MilestoneKind::Longest), title: format!("Longest {}", sport_name(sport)),
                route: Some(format!("workout/{}", a.id)), ..Default::default()
            });
        }
    }
    // The first counted week with every goal met.
    if let Some(first) = local.goals.versions.first().and_then(|v| NaiveDate::parse_from_str(&v.from, "%Y-%m-%d").ok()) {
        let wk = plan::week_starts_on(store);
        let mut w = plan::week_start(first, wk);
        let this = plan::week_start(today, wk);
        while w < this {
            let week = review::week(store, local, w, today, now);
            let end = w + Duration::days(6);
            // No award from lowering a target or pausing. Sleep observations
            // alone are not an action-completion achievement.
            let revised = local.goals.versions.iter().any(|v| v.source == GoalSource::Revision && v.from >= w.to_string() && v.from <= end.to_string());
            let measured: Vec<_> = week.goals.iter().filter(|g| g.state != OutcomeState::Observed).collect();
            if !revised && week.review.ill_days == 0 && week.review.paused_days == 0 && week.review.counted
                && !measured.is_empty() && measured.iter().all(|g| g.state == OutcomeState::Met) {
                records.push(MilestoneRecord {
                    id: "goalweek:first".into(), value: 1., date: week.review.end.clone(),
                    kind: Some(MilestoneKind::GoalWeek), title: "First week with every goal met".into(),
                    route: Some(format!("review/{w}")), ..Default::default()
                });
                break;
            }
            w += Duration::days(7);
        }
    }
    records
}

pub fn progress(store: &Store, period: Period, anchor: NaiveDate, today: NaiveDate, now: DateTime<Utc>) -> crate::Result<ProgressView> {
    let earliest = NaiveDate::from_ymd_opt(1990, 1, 1).unwrap();
    if anchor < earliest || anchor > today || today.year() > 9999 {
        return Err(crate::Error::InvalidLocal("progress date must be between 1990 and today"));
    }
    let local = local::load(store);
    let (from, to) = bounds(store, period, anchor);
    let end = to.min(today);
    let history = store.cursor("listed:lo").and_then(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok());
    let prev_from = previous_start(store, period, from);
    let current = to >= today;
    let elapsed = (end - from).num_days();
    // The same elapsed days, unless a shorter previous period can't hold them.
    let prev_end = if current { prev_from + Duration::days(elapsed) } else { from - Duration::days(1) };
    let comparable = !current || prev_end < from;
    let acts = activities(store, prev_from.min(from), end);
    let totals_now = totals(store, &acts, from, end);
    let partial = totals_now.covered_days < totals_now.days;
    let previous = (comparable && !partial && end >= from)
        .then(|| totals(store, &acts, prev_from, prev_end))
        .filter(|p| p.covered_days == p.days && p.days > 0);
    let calendar = if matches!(period, Period::Week | Period::Month) { calendar(store, &local, &acts, from, to, today, history) } else { vec![] };
    let wk = plan::week_starts_on(store);
    let last_week = plan::week_start(end, wk);
    let goal_weeks = if local.goals.versions.is_empty() {
        vec![]
    } else {
        let first_goal = local.goals.versions[0].from.clone();
        (0..GOAL_WEEKS)
            .rev()
            .map(|i| last_week - Duration::days(7 * i))
            .filter(|w| (*w + Duration::days(6)).to_string() >= first_goal)
            .map(|w| {
                let week = review::week(store, &local, w, end, now);
                let current = w + Duration::days(6) >= today;
                let (ill, paused) = (week.review.ill_days, week.review.paused_days);
                let mark = (!current && !week.review.counted).then(|| {
                    if ill + paused >= 2 {
                        if ill >= paused { DayMark::Ill } else { DayMark::Paused }
                    } else {
                        DayMark::Unknown
                    }
                });
                GoalWeek {
                    start: w.to_string(),
                    rings: week.rings,
                    current,
                    mark,
                    counted: week.review.counted,
                    met: week.goals.iter().filter(|g| g.state == OutcomeState::Met).count() as u32,
                    goals: week.goals.iter().filter(|g| g.state != OutcomeState::Observed).count() as u32,
                    reasons: week.review.reasons,
                }
            })
            .collect()
    };
    let evidence = evidence(store, &local, end, history, now);
    let records = if current { local::sync_milestones(store, &evidence, today)? } else { evidence.clone() };
    let mut milestones = present(store, records, current, &set_reps(&evidence));
    let milestones_total = milestones.len() as u32;
    milestones.truncate(12);
    Ok(ProgressView {
        today: today.to_string(),
        period,
        from: from.to_string(),
        to: to.to_string(),
        end: end.to_string(),
        current,
        history_start: history.map(|h| h.to_string()),
        previous_from: previous.as_ref().map(|_| prev_from.to_string()),
        previous_to: previous.as_ref().map(|_| prev_end.to_string()),
        previous,
        partial,
        buckets: buckets(store, &acts, period, from, to, end),
        calendar,
        goal_weeks,
        milestones,
        sleep: sleep_need(store, from, end),
        milestones_total,
        ring_days: if matches!(period, Period::Week | Period::Month) { ring_days(store, &local, from, to, today, now) } else { vec![] },
        totals: totals_now,
    })
}

fn ring_days(store: &Store, local: &LocalRecords, from: NaiveDate, to: NaiveDate, today: NaiveDate, now: DateTime<Utc>) -> Vec<DayCell> {
    let mut out = vec![];
    let mut week = plan::week_start(from, plan::week_starts_on(store));
    while week <= to {
        out.extend(ledger::board_with(store, local, week, today, now).days);
        week += Duration::days(7);
    }
    out
}

fn set_reps(evidence: &[MilestoneRecord]) -> BTreeMap<String, u32> {
    evidence.iter().filter_map(|e| Some((e.id.clone(), e.set_reps?))).collect()
}

/// "1–10 reps" for a run of rep counts, "1 rep", or "1, 5 reps".
fn rep_counts(n: &[u32]) -> String {
    const ALL: [u32; 5] = [1, 3, 5, 8, 10];
    let unit = if n == [1] { "rep" } else { "reps" };
    let run = ALL.iter().position(|r| Some(r) == n.first()).is_some_and(|i| ALL.get(i..i + n.len()) == Some(n));
    match n {
        [one] => format!("{one} {unit}"),
        [first, .., last] if run => format!("{first}–{last} {unit}"),
        _ => format!("{} {unit}", n.iter().map(u32::to_string).collect::<Vec<_>>().join(", ")),
    }
}

/// Milestones as shown, newest first; `current` marks unseen ones new. A
/// set that is the heaviest for several rep counts is one row naming its reps.
fn present(store: &Store, records: Vec<MilestoneRecord>, current: bool, reps: &BTreeMap<String, u32>) -> Vec<Milestone> {
    let unit = strength::weight_unit(store);
    let kg = |g: f64| Some(format!("{:.1} {}", strength::weight(Some(g), unit)?, if unit == WeightUnit::Lb { "lb" } else { "kg" }).replace(".0 ", " "));
    // Bests by exercise, set (activity, weight, date) and withdrawal, each with its rep counts.
    type SetKey = (String, Option<String>, u64, String, bool);
    let mut sets: BTreeMap<SetKey, Vec<(u32, MilestoneRecord)>> = BTreeMap::new();
    let mut out: Vec<Milestone> = vec![];
    for m in records {
        let Some(kind) = m.kind else { continue };
        if kind == MilestoneKind::Best {
            let mut parts = m.id.rsplitn(2, ':');
            let n = parts.next().and_then(|n| n.parse::<u32>().ok()).unwrap_or(0);
            let exercise = parts.next().unwrap_or_default().to_owned();
            sets.entry((exercise, m.activity.clone(), m.value.to_bits(), m.date.clone(), m.withdrawn)).or_default().push((n, m));
            continue;
        }
        let value = if m.withdrawn { "Evidence no longer available".into() } else {
            match kind {
                MilestoneKind::Reps => format!("{} reps", m.value as u32),
                MilestoneKind::Longest => format!("{:.1} km", m.value),
                MilestoneKind::Best | MilestoneKind::GoalWeek => String::new(),
            }
        };
        out.push(Milestone { id: m.id, kind, title: m.title, value, date: m.date, route: m.route, new: current && !m.seen, corrected: m.corrected });
    }
    for (_, mut group) in sets {
        group.sort_by_key(|(n, _)| *n);
        let counts: Vec<u32> = group.iter().map(|(n, _)| *n).collect();
        let set_reps = group.iter().find_map(|(_, m)| reps.get(&m.id).copied());
        let new = current && group.iter().any(|(_, m)| !m.seen);
        let corrected = group.iter().any(|(_, m)| m.corrected);
        let m = group.swap_remove(0).1;
        let name = m.title.split(" · ").next().unwrap_or_default().to_owned();
        let heaviest = format!("heaviest for {}", rep_counts(&counts));
        let value = if m.withdrawn {
            format!("Evidence no longer available · {heaviest}")
        } else {
            let Some(weight) = kg(m.value) else { continue };
            match set_reps {
                Some(r) => format!("{weight} × {r} · {heaviest}"),
                None => format!("{weight} · {heaviest}"),
            }
        };
        out.push(Milestone { id: m.id, kind: MilestoneKind::Best, title: name, value, date: m.date, route: m.route, new, corrected });
    }
    out.sort_by(|a, b| b.date.cmp(&a.date).then(a.id.cmp(&b.id)));
    out
}

/// Every milestone in available history, newest first: the same awards as
/// Progress, never a second award.
pub fn milestone_archive(store: &Store, today: NaiveDate, now: DateTime<Utc>) -> crate::Result<Vec<Milestone>> {
    let local = local::load(store);
    let history = store.cursor("listed:lo").and_then(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok());
    let evidence = evidence(store, &local, today, history, now);
    Ok(present(store, local::sync_milestones(store, &evidence, today)?, true, &set_reps(&evidence)))
}

const DOMAIN_WEEKS: i64 = 12;

/// Weeks and months for Progress's Cardio, Strength and Long-term.
pub fn domains(store: &Store, today: NaiveDate, now: DateTime<Utc>) -> ProgressDomains {
    let local = local::load(store);
    let wk = plan::week_starts_on(store);
    let this_week = plan::week_start(today, wk);
    let history = store.cursor("listed:lo").and_then(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok());
    let vo2 = trends::daily(store, TrendMetric::Vo2max, this_week - Duration::days(7 * DOMAIN_WEEKS) - Duration::days(400), today);
    let weeks = (0..DOMAIN_WEEKS)
        .rev()
        .map(|i| this_week - Duration::days(7 * i))
        .filter(|w| history.is_some_and(|h| *w + Duration::days(6) >= h))
        .map(|w| {
            let board = ledger::board_with(store, &local, w, today, now);
            let complete = board.coverage_complete;
            let recs = &board.recordings;
            let sum = |f: fn(&Recording) -> Option<f64>| complete.then(|| recs.iter().map(f).sum::<Option<f64>>()).flatten();
            let ill = review::days_with(&local, w, Some(PauseReason::Ill));
            let paused = review::days_with(&local, w, None) - ill;
            let end = (w + Duration::days(6)).min(today);
            let goal = |kind| board.ledger.iter().find(|l| l.kind == kind && !l.observation).map(|l| l.target);
            DomainWeek {
                start: w.to_string(),
                current: w == this_week,
                mark: if ill >= 2 { Some(DayMark::Ill) } else if paused >= 2 { Some(DayMark::Paused) } else if !complete { Some(DayMark::Unknown) } else { None },
                zone2_min: sum(|r| r.zone2_min),
                high_intensity_min: sum(|r| r.high_intensity_min),
                intensity_min: plan::intensity(store, w, today).map(|c| c.total),
                strength: complete.then(|| recs.iter().filter(|r| r.sport == Sport::Strength).count() as f64),
                vo2max: vo2.range(w..=end).next_back().map(|(_, v)| *v),
                zone2_goal: goal(GoalKind::Zone2),
                high_intensity_goal: goal(GoalKind::HighIntensity),
                intensity_goal: goal(GoalKind::Intensity),
                strength_goal: goal(GoalKind::Strength),
            }
        })
        .collect();
    let first_month = month_start(today) - Months::new(11);
    let span_from = first_month;
    let acts = activities(store, span_from, today);
    let read = |m| trends::daily(store, m, span_from, today);
    let (rhr, hrv, fa) = (read(TrendMetric::RestingHr), read(TrendMetric::Hrv), read(TrendMetric::FitnessAge));
    let avg = |series: &BTreeMap<NaiveDate, f64>, a: NaiveDate, b: NaiveDate| {
        let v: Vec<f64> = series.range(a..=b).map(|(_, v)| *v).collect();
        (!v.is_empty()).then(|| v.iter().sum::<f64>() / v.len() as f64)
    };
    let months = (0..12)
        .map(|i| {
            let m = first_month + Months::new(i);
            let end = month_end(m).min(today);
            let covered = history.is_some_and(|h| h <= m) && activities_listed(store, m, end);
            DomainMonth {
                start: m.to_string(),
                minutes: covered.then(|| acts.iter().filter(|a| a.date >= m && a.date <= end).map(|a| a.minutes).sum()),
                resting_hr: avg(&rhr, m, end),
                hrv: avg(&hrv, m, end),
                vo2max: vo2.range(m..=end).next_back().map(|(_, v)| *v),
                fitness_age: fa.range(m..=end).next_back().map(|(_, v)| *v),
            }
        })
        .collect();
    ProgressDomains {
        today: today.to_string(),
        goals: ledger::board_with(store, &local, today, today, now).goals.map(|g| g.targets),
        weeks,
        months,
    }
}
