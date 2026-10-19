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
use chrono::{Datelike, Duration, Months, NaiveDate};
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
            let card = screens::activity(store, &id)?;
            let sport = crate::local::sport_of(&screens::activity_kind(store, &id).unwrap_or_default());
            Some(Activity { id, date, sport, minutes: card.duration_s / 60., km: card.distance_km })
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
        let Some(raw) = store.get("sleep", &d.to_string()) else { continue };
        let s = screens::sleep(&raw);
        let asleep = number(s, "sleepTimeSeconds").map(|v| v / 60.);
        let need = s["sleepNeed"]["actual"].as_f64().filter(|n| *n > 0.);
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
fn evidence(store: &Store, local: &LocalRecords, today: NaiveDate, history: Option<NaiveDate>) -> Vec<MilestoneRecord> {
    let mut records = vec![];
    for b in strength::all_time_bests(store, &today.to_string()) {
        let id = format!("best:{}:{}", b.key, b.reps);
        records.push(MilestoneRecord {
            id, value: b.grams, date: b.date, kind: Some(MilestoneKind::Best),
            title: format!("{} · best for {} reps", b.name, b.reps),
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
            let week = review::week(store, local, w, today);
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

pub fn progress(store: &Store, period: Period, anchor: NaiveDate, today: NaiveDate) -> crate::Result<ProgressView> {
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
                let week = review::week(store, &local, w, end);
                GoalWeek {
                    start: w.to_string(),
                    rings: week.rings,
                    counted: week.review.counted,
                    met: week.goals.iter().filter(|g| g.state == OutcomeState::Met).count() as u32,
                    goals: week.goals.iter().filter(|g| g.state != OutcomeState::Observed).count() as u32,
                    reasons: week.review.reasons,
                }
            })
            .collect()
    };
    let evidence = evidence(store, &local, end, history);
    let records = if current { local::sync_milestones(store, &evidence, today)? } else { evidence };
    let mut milestones = present(store, records, current);
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
        ring_days: if matches!(period, Period::Week | Period::Month) { ring_days(store, &local, from, to, today) } else { vec![] },
        totals: totals_now,
    })
}

fn ring_days(store: &Store, local: &LocalRecords, from: NaiveDate, to: NaiveDate, today: NaiveDate) -> Vec<DayCell> {
    let now = chrono::Utc::now();
    let mut out = vec![];
    let mut week = plan::week_start(from, plan::week_starts_on(store));
    while week <= to {
        out.extend(ledger::board_with(store, local, week, today, now).days);
        week += Duration::days(7);
    }
    out
}

/// Milestones as shown, newest first; `current` marks unseen ones new.
fn present(store: &Store, records: Vec<MilestoneRecord>, current: bool) -> Vec<Milestone> {
    let unit = strength::weight_unit(store);
    let mut out: Vec<Milestone> = records.into_iter().filter_map(|m| {
        let kind = m.kind?;
        let value = if m.withdrawn { "Evidence no longer available".into() } else {
            match kind {
                MilestoneKind::Best => format!("{:.1} {}", strength::weight(Some(m.value), unit)?, if unit == WeightUnit::Lb { "lb" } else { "kg" }).replace(".0 ", " "),
                MilestoneKind::Longest => format!("{:.1} km", m.value),
                MilestoneKind::GoalWeek => format!("Week ending {}", m.date),
            }
        };
        Some(Milestone { id: m.id, kind, title: m.title, value, date: m.date, route: m.route, new: current && !m.seen, corrected: m.corrected })
    }).collect();
    out.sort_by(|a, b| b.date.cmp(&a.date).then(a.id.cmp(&b.id)));
    out
}

/// Every milestone in available history, newest first: the same awards as
/// Progress, never a second award.
pub fn milestone_archive(store: &Store, today: NaiveDate) -> crate::Result<Vec<Milestone>> {
    let local = local::load(store);
    let history = store.cursor("listed:lo").and_then(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok());
    let evidence = evidence(store, &local, today, history);
    Ok(present(store, local::sync_milestones(store, &evidence, today)?, true))
}

const DOMAIN_WEEKS: i64 = 12;

/// Weeks, bests and months for Progress's Cardio, Strength and Long-term.
pub fn domains(store: &Store, today: NaiveDate) -> ProgressDomains {
    let local = local::load(store);
    let now = chrono::Utc::now();
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
            DomainWeek {
                start: w.to_string(),
                current: w == this_week,
                mark: if ill >= 2 { Some(DayMark::Ill) } else if paused >= 2 { Some(DayMark::Paused) } else if !complete { Some(DayMark::Unknown) } else { None },
                zone2_min: sum(|r| r.zone2_min),
                high_intensity_min: sum(|r| r.high_intensity_min),
                intensity_min: plan::intensity(store, w, today).map(|c| c.total),
                strength: complete.then(|| recs.iter().filter(|r| r.sport == Sport::Strength).count() as f64),
                vo2max: vo2.range(w..=end).next_back().map(|(_, v)| *v),
            }
        })
        .collect();
    let unit = strength::weight_unit(store);
    let mut heaviest: BTreeMap<String, strength::AllTimeBest> = BTreeMap::new();
    for b in strength::all_time_bests(store, &today.to_string()) {
        match heaviest.get(&b.key) {
            Some(h) if h.grams > b.grams || (h.grams == b.grams && h.reps >= b.reps) => {}
            _ => {
                heaviest.insert(b.key.clone(), b);
            }
        }
    }
    let mut bests: Vec<DomainBest> = heaviest
        .into_values()
        .map(|b| DomainBest { weight: strength::weight(Some(b.grams), unit), unit, key: b.key, name: b.name, reps: b.reps, date: b.date, activity: b.activity })
        .collect();
    bests.sort_by(|a, b| b.weight.unwrap_or(0.).total_cmp(&a.weight.unwrap_or(0.)));
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
        bests,
        months,
    }
}
