//! Which part of the person's own day Today is in, and so which advice card
//! leads. Phases follow the recorded wake time and tonight's sleep plan, never
//! fixed clock hours; without a credible schedule Today keeps its fixed
//! layout. The boundaries and fallbacks are listed under Rules in docs/spec.md.
use crate::{
    rules::{self, WakeClock},
    view::{DayPhase, LastNight, SleepPlan, TodayCard, TodayFocus},
};
use chrono::{Duration, NaiveDate, NaiveDateTime, NaiveTime};

/// How long the morning view lasts after waking.
pub const WAKE_SPAN_MIN: i64 = 180;
/// Tonight's plan leads this long before Garmin's window (or Raring's
/// asleep-by time) starts.
pub const PROMOTE_MIN: i64 = 120;
/// Fewer recorded wake times than this in 14 days is no schedule.
pub const MIN_WAKE_NIGHTS: usize = 4;
/// Wake times (on a clock circle) whose middle half spreads wider than this
/// are irregular.
pub const IRREGULAR_SPREAD_MIN: f64 = 180.;
/// A UTC offset this far from the latest recorded night's means travel; DST
/// shifts by an hour and keeps the schedule.
pub const TRAVEL_SHIFT_MIN: i32 = 60;
/// Two outside-range vitals put the vitals card above the leading card.
pub const VITALS_FIRST: u32 = 2;

/// Everything the phase depends on, read from local data.
pub struct PhaseInput<'a> {
    pub now: NaiveDateTime,
    /// The day Today shows: the morning of the latest synced night, kept
    /// until `handover` while the next night hasn't synced.
    pub date: NaiveDate,
    pub plan: Option<&'a SleepPlan>,
    /// Recorded wake times of the last 14 nights, hours after local midnight.
    pub wakes: &'a [f64],
    /// The wake time of Garmin's sleep schedule for `date`, hours after
    /// local midnight, unless it's Garmin's untouched default.
    pub schedule_wake: Option<f64>,
    /// The displayed night's recorded sleep, when it has synced.
    pub last_night: Option<LastNight>,
    /// The displayed night's recorded wake; used only with `last_night`.
    pub woke_at: Option<NaiveDateTime>,
    pub adaptive: bool,
    /// Garmin rated the displayed night as jet lag.
    pub jet_lag: bool,
    /// Minutes between the phone's UTC offset now and the latest recorded
    /// night's.
    pub zone_shift_min: Option<i32>,
    pub vitals_off: u32,
}

fn parse(s: &str) -> Option<NaiveDateTime> {
    NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S").ok()
}
fn iso(t: NaiveDateTime) -> String {
    t.format("%Y-%m-%dT%H:%M:%S").to_string()
}
fn midnight(date: NaiveDate) -> NaiveDateTime {
    date.and_time(NaiveTime::MIN)
}

/// The usual wake the phases use: recorded wake times once there are
/// enough of them, otherwise the wake time of Garmin's sleep schedule, which
/// counts as a credible schedule of its own. Returns whether it is Garmin's.
pub fn usual(wakes: &[f64], schedule_wake: Option<f64>) -> (Option<WakeClock>, bool) {
    let recorded = rules::wake_clock(wakes);
    match schedule_wake {
        Some(h) if recorded.as_ref().is_none_or(|u| u.count < MIN_WAKE_NIGHTS) => {
            (Some(WakeClock { median: h, spread: 0., count: MIN_WAKE_NIGHTS }), true)
        }
        _ => (recorded, false),
    }
}
/// Enough recorded wake times, close enough together, to anchor the day.
pub fn credible(usual: Option<&WakeClock>) -> bool {
    usual.is_some_and(|u| u.count >= MIN_WAKE_NIGHTS && u.spread * 60. <= IRREGULAR_SPREAD_MIN)
}
/// The usual wake on the morning a night is filed under (Garmin files a
/// night under the day it ends); a late-evening usual wake is the evening
/// before.
pub fn expected_wake(morning: NaiveDate, usual: &WakeClock) -> NaiveDateTime {
    midnight(morning) + Duration::minutes((usual.median * 60.).round() as i64)
}
/// When the night after the one shown on `date` should end: the first usual
/// wake more than 12 hours after that day's recorded wake (or its expected
/// wake without one). A pre-midnight wake filed under `date` then counts as
/// that day's night, not the next one.
pub fn next_wake(
    date: NaiveDate,
    woke: Option<NaiveDateTime>,
    usual: Option<&WakeClock>,
) -> Option<NaiveDateTime> {
    let usual = usual?;
    let after = woke.unwrap_or(expected_wake(date, usual)) + Duration::hours(12);
    (0..=2)
        .map(|k| expected_wake(date + Duration::days(k), usual))
        .find(|t| *t > after)
}
/// Until when Today keeps showing `date` while the next night hasn't synced:
/// three hours after a credible expected wake, otherwise noon of the next
/// calendar day. `screens::current_date` and the waiting state share it.
pub fn handover(
    date: NaiveDate,
    woke: Option<NaiveDateTime>,
    usual: Option<&WakeClock>,
) -> NaiveDateTime {
    next_wake(date, woke, usual)
        .filter(|_| credible(usual))
        .map(|t| t + Duration::minutes(WAKE_SPAN_MIN))
        .unwrap_or(midnight(date + Duration::days(1)) + Duration::hours(12))
}

/// Tonight's plan reduced to the times the phases need.
struct Night {
    promote: NaiveDateTime,
    wind_down: NaiveDateTime,
    end: NaiveDateTime,
    /// When tonight's sleep should end.
    wake: NaiveDateTime,
}

pub fn focus(input: &PhaseInput) -> TodayFocus {
    let (usual, garmin) = usual(input.wakes, input.schedule_wake);
    let usual = usual.as_ref();
    let woke_at = input.woke_at.filter(|_| input.last_night.is_some());
    let next = next_wake(input.date, woke_at, usual);
    // The displayed day is stale once its next night should have ended: at
    // the credible expected wake, otherwise once the calendar day has moved
    // on. This holds in every layout, so no fallback brings back yesterday's
    // effort call.
    let stale_from = next
        .filter(|_| credible(usual))
        .unwrap_or(midnight(input.date + Duration::days(1)));
    let handover = handover(input.date, woke_at, usual);
    let cards = |lead: &[TodayCard]| {
        let mut cards = lead.to_vec();
        if input.vitals_off >= VITALS_FIRST {
            cards.insert(0, TodayCard::Vitals);
        }
        cards
    };
    let focus = |phase, lead: &[TodayCard], until: Option<NaiveDateTime>| TodayFocus {
        phase,
        cards: cards(lead),
        until: until.filter(|t| *t > input.now).map(iso),
        effort: true,
        plan_open: false,
        sleep_missing: input.last_night.is_none(),
        last_night: input.last_night.clone(),
        note: None,
        recovery_hours: None,
        schedule: None,
    };
    // Waiting for a night that hasn't synced: neutral, with neither the
    // previous day's plan nor its effort call.
    let waiting = |until| TodayFocus {
        effort: false,
        sleep_missing: true,
        last_night: None,
        ..focus(DayPhase::Wake, &[TodayCard::Focus, TodayCard::Day], until)
    };
    if input.now >= stale_from {
        return waiting(Some(handover));
    }
    let overview = |note: Option<&str>| TodayFocus {
        plan_open: true,
        note: note.map(str::to_owned),
        ..focus(
            DayPhase::Overview,
            &[TodayCard::Focus, TodayCard::Day, TodayCard::SleepPlan],
            Some(stale_from),
        )
    };
    if !input.adaptive {
        return overview(None);
    }
    if usual.is_none_or(|u| u.count < MIN_WAKE_NIGHTS) {
        return overview(Some("Fixed layout until a few more nights are recorded."));
    }
    if input.jet_lag
        || input
            .zone_shift_min
            .is_some_and(|shift| shift.abs() > TRAVEL_SHIFT_MIN)
    {
        return overview(Some("Fixed layout while your sleep settles into a new time zone."));
    }
    if !credible(usual) {
        return overview(Some("Fixed layout: your wake times vary too much to plan the day around."));
    }
    let (Some(usual), Some(next)) = (usual, next) else {
        return overview(None);
    };
    let night = input.plan.and_then(|plan| {
        let start = plan.bedtime_start_at.as_deref().unwrap_or(&plan.bedtime_at);
        Some(Night {
            promote: parse(start)? - Duration::minutes(PROMOTE_MIN),
            wind_down: parse(&plan.wind_down_at)?,
            end: parse(&plan.bedtime_at)?,
            wake: next,
        })
    });
    let Some(night) = night else {
        return overview(Some("Fixed layout until there's a sleep plan for tonight."));
    };
    let woke = woke_at.unwrap_or(expected_wake(input.date, usual));
    // The displayed day has no recorded night and its usual wake (a day
    // sleeper's, say) hasn't come yet.
    if input.last_night.is_none() && input.now < woke {
        return waiting(Some(woke));
    }
    if !(woke < night.wind_down && night.wind_down <= night.end && night.end < night.wake) {
        return overview(Some("Fixed layout: tonight's plan doesn't fit your usual wake time."));
    }
    let wake_end = (woke + Duration::minutes(WAKE_SPAN_MIN)).min(night.promote);
    let now = input.now;
    let focus = |phase, lead: &[TodayCard], until| TodayFocus {
        schedule: garmin.then(|| "Garmin sleep schedule".to_owned()),
        ..focus(phase, lead, until)
    };
    use TodayCard::*;
    if now < wake_end {
        focus(DayPhase::Wake, &[Focus, Day, SleepPlan], Some(wake_end))
    } else if now < night.promote {
        focus(DayPhase::Day, &[TodayCard::Day, Now, Focus, SleepPlan], Some(night.promote))
    } else if now < night.wind_down {
        // From the evening on nothing should invite a workout.
        TodayFocus {
            plan_open: true,
            effort: false,
            ..focus(DayPhase::Evening, &[SleepPlan, Day, Focus], Some(night.wind_down))
        }
    } else {
        let (phase, until) = if now < night.end {
            (DayPhase::WindDown, night.end)
        } else {
            (DayPhase::LateNight, night.wake)
        };
        TodayFocus {
            plan_open: true,
            effort: false,
            ..focus(phase, &[SleepPlan, Day, Focus], Some(until))
        }
    }
}

/// Today's headline for the phase; the readiness headline otherwise.
pub fn headline(focus: &TodayFocus, plan: Option<&SleepPlan>, readiness: &str) -> String {
    match (focus.phase, plan) {
        (DayPhase::Evening, Some(plan)) => format!("Wind down at {}", plan.wind_down),
        (DayPhase::WindDown, _) => "Time to wind down".into(),
        (DayPhase::LateNight, _) => "Rest when you're ready".into(),
        (DayPhase::Wake, _) if focus.sleep_missing && !focus.effort => {
            "Waiting for last night's sleep".into()
        }
        _ => readiness.into(),
    }
}
