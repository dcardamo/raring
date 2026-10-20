//! Today's leading message (docs/spec.md, Rules: Today's lead, Adapt): hard
//! gates first (privacy, evidence, circumstances, decisions, Garmin guidance,
//! time), then one supported message. No ranking score.
use crate::{
    ledger,
    local::{self, LocalRecords},
    store::Store,
    view::*,
};
use chrono::{DateTime, Duration, FixedOffset, NaiveDate};

const GOAL_SPORTS: [(Sport, &str); 7] = [
    (Sport::Running, "Run"),
    (Sport::Cycling, "Ride"),
    (Sport::Walking, "Walk"),
    (Sport::Hiking, "Hike"),
    (Sport::Swimming, "Swim"),
    (Sport::Strength, "Strength"),
    (Sport::Other, "Session"),
];

pub(crate) fn sport_name(s: Sport) -> &'static str {
    GOAL_SPORTS.iter().find(|(k, _)| *k == s).map_or("Session", |(_, n)| n)
}

fn day_label(d: &str) -> String {
    NaiveDate::parse_from_str(d, "%Y-%m-%d").map_or_else(|_| d.to_owned(), |d| d.format("%a %-d %b").to_string())
}

fn hm(minutes: f64) -> String {
    let m = minutes.round() as i64;
    format!("{}:{:02}", m / 60, m % 60)
}

/// Today's circumstances. Yesterday's rest carries through the night until
/// the next waking day.
pub fn adapt(local: &LocalRecords, today: NaiveDate, overnight: bool) -> AdaptState {
    let day = today.to_string();
    let yesterday = (today - Duration::days(1)).to_string();
    let rest = local.circumstances.rest.iter().any(|r| r.date == day || (overnight && r.date == yesterday));
    let pause = local
        .circumstances
        .pauses
        .iter()
        .find(|p| p.since <= day && p.until.as_deref().is_none_or(|u| day.as_str() <= u))
        .cloned();
    let limits = local.circumstances.limits.clone();
    let limited = !limits.excluded.is_empty() || !limits.note.is_empty();
    let label = match &pause {
        Some(p) if p.reason == PauseReason::Ill => "Ill",
        Some(_) => "Paused",
        None if rest => "Rest day",
        None if limited => "Limits on",
        None => "Adapt today",
    };
    AdaptState {
        rest,
        review_due: pause.as_ref().is_some_and(|p| p.until.is_none() && p.review.as_deref().is_some_and(|r| r <= day.as_str())),
        blocked: rest || pause.is_some(),
        pause,
        limits,
        label: label.to_owned(),
        routines: local
            .routines
            .items
            .iter()
            .map(|item| RoutineToday { done: local.routines.reports.iter().any(|x| x.id == item.id && x.date == day), item: item.clone() })
            .collect(),
    }
}

struct Lead {
    reasons: Vec<LeadReason>,
}

impl Lead {
    fn gate(&mut self, gate: &str, result: impl Into<String>) {
        self.reasons.push(LeadReason { gate: gate.into(), result: result.into() });
    }
    #[allow(clippy::too_many_arguments)]
    fn done(
        self,
        id: String,
        kind: LeadKind,
        headline: impl Into<String>,
        line: Option<String>,
        action: Option<(&str, Option<String>, bool)>,
        dismissable: bool,
    ) -> TodayLead {
        TodayLead {
            id,
            kind,
            headline: headline.into(),
            line,
            action: action.map(|(label, route, adapt)| LeadAction { label: label.into(), route, adapt }),
            dismissable,
            kept: false,
            garmin: None,
            activity: None,
            intention: None,
            reasons: self.reasons,
        }
    }
}

fn route(r: &str) -> Option<String> {
    Some(r.to_owned())
}

/// The displayed night hasn't synced: a day without readiness, or the
/// phase's neutral wait, which keeps the previous day's readiness on screen
/// but no effort call (`phase::focus` turns effort off only for that and the
/// sleep phases). Read before `hold_back` removes effort for other reasons.
fn waiting(view: &TodayView) -> bool {
    let sleep_phase = matches!(view.focus.phase, DayPhase::Evening | DayPhase::WindDown | DayPhase::LateNight);
    view.focus.sleep_missing && (view.readiness.is_none() || !view.focus.effort) && !sleep_phase
}

/// What every surface must respect today: rest, illness, a pause, limits and
/// a dismissed training suggestion. Removes the effort call when any of the
/// first four apply. Returns the state and whether the suggestion was dismissed.
pub fn hold_back(local: &LocalRecords, view: &mut TodayView, today: NaiveDate) -> (AdaptState, bool) {
    let phase = view.focus.phase;
    let state = adapt(local, today, phase == DayPhase::LateNight || waiting(view));
    if state.blocked || limited(&state) {
        view.call = None;
        view.call_note = None;
        view.focus.effort = false;
    }
    let id = format!("suggest:{today}");
    let dismissed = local.circumstances.dismissals.iter().any(|d| d.date == today.to_string() && d.id == id);
    if dismissed {
        view.call = None;
        view.call_note = None;
        view.focus.effort = false;
    }
    (state, dismissed)
}

pub fn limited(state: &AdaptState) -> bool {
    !state.limits.excluded.is_empty() || !state.limits.note.is_empty()
}

/// A short, private line for lock-screen surfaces. Never the limits text.
pub fn circumstance_line(state: &AdaptState) -> Option<&'static str> {
    match &state.pause {
        Some(p) if p.reason == PauseReason::Ill => Some("Ill · goals paused"),
        Some(_) => Some("Training paused"),
        None if state.rest => Some("Rest day"),
        None if limited(state) => Some("Training suggestions limited"),
        None => None,
    }
}

/// The live Today: choose the lead and keep exercise prompts out while
/// resting, ill, paused or limited. Historical reads never call this.
pub fn apply(store: &Store, view: &mut TodayView, today: NaiveDate, now: DateTime<FixedOffset>) {
    let local = local::load(store);
    let sleep_phase = matches!(view.focus.phase, DayPhase::Evening | DayPhase::WindDown | DayPhase::LateNight);
    let waiting = waiting(view);
    let (state, _) = hold_back(&local, view, today);
    let board = ledger::board(store, today, today, now.to_utc());
    let lead = choose(view, &local, &state, &board, today, waiting, sleep_phase);
    // A finished workout lets the day finish, and tonight's sleep invites
    // no effort: no effort scale beside either.
    if matches!(lead.kind, LeadKind::Done | LeadKind::Sleep) {
        view.focus.effort = false;
    }
    view.lead = Some(lead);
    view.adapt = Some(state);
}

fn choose(
    view: &TodayView,
    local: &LocalRecords,
    state: &AdaptState,
    board: &PlanBoard,
    today: NaiveDate,
    waiting: bool,
    sleep_phase: bool,
) -> TodayLead {
    let day = today.to_string();
    let mut l = Lead { reasons: vec![] };
    l.gate("Privacy", "Journal entries never feed Today.");
    if waiting {
        l.gate("Evidence", "Last night hasn't synced; an older night's readiness isn't today's.");
        return l.done(format!("wait:{day}"), LeadKind::Waiting, "Last night hasn't synced yet", None, None, false);
    }
    let night = view.focus.last_night.as_ref();
    l.gate(
        "Evidence",
        match (&view.readiness, night.and_then(|n| n.asleep_min)) {
            (Some(r), Some(a)) => format!("Readiness {} · last night {} asleep.", r.value, hm(a)),
            (Some(r), None) => format!("Readiness {} · no recorded night.", r.value),
            (None, Some(a)) => format!("No readiness · last night {} asleep.", hm(a)),
            (None, None) => "No readiness or recorded night.".into(),
        },
    );
    if sleep_phase {
        l.gate("Time", "Evening: sleep preparation leads, with no exercise prompt.");
        let plan = view.sleep_plan.as_ref();
        let night_id = plan.map_or(day.clone(), |p| p.night.clone());
        let sleep = route("sleep/tonight");
        return match view.focus.phase {
            DayPhase::LateNight => l.done(format!("late:{night_id}"), LeadKind::Sleep, "Rest when you're ready", None, Some(("Tonight's sleep", sleep, false)), false),
            DayPhase::WindDown => {
                let line = plan.map(|p| match (&p.bedtime_start, p.bedtime_source) {
                    (Some(start), BedtimeSource::Garmin) => format!("Garmin's window {start}–{}", p.bedtime),
                    _ => format!("Asleep by {}", p.bedtime),
                });
                l.done(format!("wind:{night_id}"), LeadKind::Sleep, "Time to wind down", line, Some(("Tonight's plan", sleep, false)), false)
            }
            _ => l.done(
                format!("evening:{night_id}"),
                LeadKind::Sleep,
                "Make room for sleep",
                plan.map(|p| format!("Wind down from {}", p.wind_down)),
                Some(("Tonight's plan", sleep, false)),
                false,
            ),
        };
    }
    if let Some(p) = &state.pause {
        l.gate("Circumstances", format!("{} since {}: no exercise prompts, nothing to catch up.", if p.reason == PauseReason::Ill { "Ill" } else { "Paused" }, day_label(&p.since)));
        let (kind, headline) = if p.reason == PauseReason::Ill { (LeadKind::Ill, "Ill · goals paused") } else { (LeadKind::Paused, "Training is paused") };
        return l.done(format!("paused:{day}"), kind, headline, Some(format!("Since {}", day_label(&p.since))), Some(("Resume when ready", None, true)), false);
    }
    if state.rest {
        l.gate("Circumstances", "Rest day: no exercise prompts today or tonight; your plan stays as it is.");
        return l.done(format!("rest:{day}"), LeadKind::Rest, "Rest day", Some("Through tonight".into()), Some(("View plan", route("plan"), false)), false);
    }
    if view.focus.phase == DayPhase::Overview {
        if let Some(note) = &view.focus.note {
            l.gate("Time", "Today keeps its fixed order.");
            return l.done(format!("fixed:{day}"), LeadKind::Fixed, "Your recorded day", Some(note.clone()), Some(("Last night's sleep", route("sleep/last"), false)), false);
        }
    }
    let excluded = |s: Sport| state.limits.excluded.contains(&s);
    let note = !state.limits.note.is_empty();
    // Recognition: a recording today leads and creates no new task.
    let recorded: Vec<&Recording> = board
        .recordings
        .iter()
        .filter(|r| r.date == day && (r.sport != Sport::Walking || board.goals.is_none()))
        .collect();
    if let Some(r) = recorded.last() {
        l.gate("Decisions", format!("{} recorded at {}; recognition leads.", r.name, r.start_local.get(11..16).unwrap_or("")));
        let goal = |k: GoalKind| board.ledger.iter().find(|g| g.kind == k);
        let line = if let (Sport::Strength, Some(g)) = (r.sport, goal(GoalKind::Strength)) {
            let n = board.recordings.iter().filter(|x| x.sport == Sport::Strength && x.start_local <= r.start_local).count();
            Some(format!("{n} of {} strength this week", g.target))
        } else if let (Some(m), Some(_)) = (r.intensity_min, goal(GoalKind::Intensity)) {
            Some(format!("+{m} intensity min (Garmin)"))
        } else {
            None
        };
        let mut lead = l.done(format!("done:{}", r.id), LeadKind::Done, "Done today", line, Some(("Open workout", route(&format!("workout/{}", r.id)), false)), false);
        lead.activity = Some(r.id.clone());
        return lead;
    }
    let dismissed = |id: &str| local.circumstances.dismissals.iter().any(|d| d.date == day && d.id == id);
    let low = matches!(view.call, Some(Effort::Rest | Effort::Easy));
    // A session the person kept today leads over other plans for the day.
    let kept = local.circumstances.today_choice.as_ref().filter(|c| c.date == day).map(|c| c.plan_id.as_str());
    let own = board.items.iter().filter(|p| p.intention.date == day && p.state == ItemState::Planned && p.intention.kind == IntentionKind::Workout);
    let own_kept = kept.and_then(|k| own.clone().find(|p| p.intention.id == k));
    let garmin_today = || board.garmin.iter().filter(|g| g.date == day && !dismissed(&format!("garmin:{day}:{}", g.name)));
    let garmin_kept = kept.and_then(|k| garmin_today().find(|g| format!("garmin:{day}:{}", g.name) == k));
    let garmin_lead = if own_kept.is_some() { None } else { garmin_kept.or_else(|| garmin_today().next()) };
    if let Some(g) = garmin_lead {
        let id = format!("garmin:{day}:{}", g.name);
        l.gate("Decisions", format!("Garmin's schedule: {} today (read-only).", g.name));
        let is_kept = garmin_kept.is_some_and(|k| std::ptr::eq(k, g));
        let mut lead = if !state.limits.excluded.is_empty() || note {
            l.gate("Circumstances", "Your limits apply and Raring can't see this workout's sport or movements.");
            l.done(id.clone(), LeadKind::Review, "Review needed", Some(format!("{} · contents unknown to Raring", g.name)), Some(("Review your plan", route("plan"), false)), true)
        } else if low {
            l.gate("Garmin guidance", "Garmin suggests an easier day; the session isn't cancelled.");
            l.done(id.clone(), LeadKind::Garmin, "An easier day may fit", Some(format!("Garmin: {}", g.name)), Some(("Review your plan", route("plan"), false)), true)
        } else {
            l.done(id.clone(), LeadKind::Garmin, g.name.clone(), Some("Garmin schedule".into()), Some(("Your plan", route("plan"), false)), true)
        };
        lead.kept = is_kept;
        lead.garmin = Some(id);
        return lead;
    }
    if let Some(p) = own_kept.or_else(|| own.clone().next()) {
        let i = &p.intention;
        let name = i.name.clone().unwrap_or_else(|| sport_name(i.sport).to_owned());
        let mut lead = if note {
            l.gate("Circumstances", "Your movement note applies; Raring can't check a session's movements.");
            l.done(format!("review:{}", i.id), LeadKind::Review, "Review your session", Some(format!("{name} · your movement note applies")), Some(("Your plan", route("plan"), false)), false)
        } else {
            l.gate("Decisions", "Your own session for today leads.");
            let headline = match &i.time { Some(t) => format!("{name} at {t}"), None => format!("{name} today") };
            let line = match (p.coach.is_some(), excluded(i.sport)) {
                (true, true) => "Garmin Coach · excluded in your limits",
                (true, false) => "Garmin Coach",
                (false, true) => "Your plan · excluded in your limits",
                (false, false) => "Your plan",
            };
            l.done(format!("session:{}", i.id), LeadKind::Session, headline, Some(line.into()), Some(("Your plan", route("plan"), false)), false)
        };
        lead.intention = Some(i.id.clone());
        lead.kept = kept == Some(i.id.as_str());
        return lead;
    }
    l.gate("Decisions", "Nothing scheduled, planned or recorded yet today.");
    let Some(readiness) = &view.readiness else {
        l.gate("Garmin guidance", "No readiness, so no effort suggestion; sleep leads.");
        let (headline, line) = match night.and_then(|n| n.asleep_min) {
            Some(a) => (
                format!("You slept {}", hm(a)),
                night.and_then(|n| n.need_min).map(|need| if a < need { format!("{} below last night's need", hm(need - a)) } else { "Met last night's need".into() }),
            ),
            None => ("No sleep recorded".to_owned(), None),
        };
        return l.done(format!("sleep:{day}"), LeadKind::Summary, headline, line, Some(("Last night's sleep", route("sleep/last"), false)), false);
    };
    if !state.limits.excluded.is_empty() || note {
        l.gate("Circumstances", "Your limits constrain training suggestions.");
        let names: Vec<&str> = state.limits.excluded.iter().map(|s| sport_name(*s)).collect();
        let (headline, line) = if note {
            ("Your movement limits apply", "Workouts need your review".to_owned())
        } else {
            ("Your activity limits apply", format!("{} excluded", names.join(", ")))
        };
        return l.done(format!("limits:{day}"), LeadKind::Limits, headline, Some(line), Some(("Review limits", None, true)), false);
    }
    let id = format!("suggest:{day}");
    if dismissed(&id) {
        l.gate("Relevance", "You chose \"Not today\" for training ideas; it stays dismissed after sync and resume.");
        return l.done("quiet".into(), LeadKind::Quiet, "Nothing else needs you today", Some("Training ideas hidden for today".into()), Some(("Tonight's sleep", route("sleep/tonight"), false)), false);
    }
    match view.call {
        Some(Effort::Push) => {
            l.gate("Garmin guidance", format!("Readiness {} {}: Garmin's guidance allows hard effort if it fits.", readiness.value, readiness.label));
            l.gate("Relevance", "No session declared: an option to plan, not a required load.");
            let line = night.and_then(|n| n.asleep_min).map(|a| format!("After {} of sleep", hm(a)));
            l.done(id, LeadKind::Suggest, "Training is an option today", line, Some(("Plan a session", route("plan"), false)), true)
        }
        call => {
            l.gate("Garmin guidance", format!("Readiness {} {}: Garmin suggests {}.", readiness.value, readiness.label, if call == Some(Effort::Steady) { "a steady day" } else { "an easier day" }));
            let headline = if call == Some(Effort::Steady) { "A steady day" } else { "An easier day may fit" };
            let worst = view
                .factors
                .iter()
                .filter(|f| matches!(f.tone, Tone::Poor | Tone::Fair))
                .min_by(|a, b| (a.tone != Tone::Poor, a.percent.unwrap_or(100.)).partial_cmp(&(b.tone != Tone::Poor, b.percent.unwrap_or(100.))).unwrap_or(std::cmp::Ordering::Equal))
                .map(|f| format!("{} holds it back most", f.name));
            l.done(id, LeadKind::Easy, headline, worst, Some(("Readiness", route("readiness"), false)), true)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::screens;
    use chrono::{Local, TimeZone, Weekday};
    use serde_json::json;

    const DAY: &str = "2025-03-13";
    fn today() -> NaiveDate {
        NaiveDate::from_ymd_opt(2025, 3, 13).unwrap()
    }
    fn at(day: u32, hour: u32) -> DateTime<Local> {
        Local.with_ymd_and_hms(2025, 3, day, hour, 0, 0).unwrap()
    }
    fn fixture(level: &str, score: u32) -> Store {
        let store = Store::open(":memory:").unwrap();
        store
            .put("readiness", DAY, "now", &json!({"inputContext": "AFTER_WAKEUP_RESET", "timestamp": "2025-03-13T07:00:00.0",
                "score": score, "level": level, "sleepScoreFactorPercent": 40, "sleepScoreFactorFeedback": "POOR"}))
            .unwrap();
        store.set_cursor("listed:lo", "2025-03-01").unwrap();
        store.set_cursor("listed:hi", DAY).unwrap();
        store
    }
    fn act(store: &Store, action: LocalAction, when: DateTime<Local>) {
        local::apply(store, &action, when, Weekday::Mon, &[], None).unwrap();
    }
    fn view(store: &Store, phase: DayPhase) -> TodayView {
        let mut v = screens::today_at(store, DAY, at(13, 12).fixed_offset());
        v.focus.phase = phase;
        v.focus.sleep_missing = false;
        v.focus.note = None;
        v
    }
    fn lead(store: &Store, phase: DayPhase) -> (TodayView, TodayLead) {
        let mut v = view(store, phase);
        apply(store, &mut v, today(), at(13, 12).fixed_offset());
        let l = v.lead.clone().unwrap();
        (v, l)
    }
    fn recording(store: &Store, id: u64, kind: &str, hour: u32) {
        store
            .put("activity", &id.to_string(), "now", &json!({"activityId": id, "activityName": format!("Morning {kind}"),
                "activityTypeDTO": {"typeKey": kind}, "summaryDTO": {"startTimeLocal": format!("{DAY} {hour:02}:00:00"), "duration": 1800}}))
            .unwrap();
    }

    #[test]
    fn a_night_that_hasnt_synced_waits_without_advice() {
        let store = Store::open(":memory:").unwrap();
        let mut v = view(&store, DayPhase::Wake);
        v.focus.sleep_missing = true;
        apply(&store, &mut v, today(), at(13, 7).fixed_offset());
        let l = v.lead.unwrap();
        assert_eq!((l.kind, l.headline.as_str()), (LeadKind::Waiting, "Last night hasn't synced yet"));
        assert!(l.action.is_none() && !l.dismissable);
        // Past the usual wake the previous day's readiness stays on screen,
        // but its verdict doesn't lead.
        let store = fixture("LOW", 30);
        let mut v = view(&store, DayPhase::Wake);
        (v.focus.sleep_missing, v.focus.effort, v.focus.last_night) = (true, false, None);
        assert!(v.readiness.is_some());
        apply(&store, &mut v, today(), at(13, 8).fixed_offset());
        assert_eq!(v.lead.unwrap().kind, LeadKind::Waiting);
    }

    #[test]
    fn evening_and_late_night_lead_with_sleep_and_no_exercise() {
        let store = fixture("HIGH", 85);
        assert_eq!(lead(&store, DayPhase::Evening).1.headline, "Make room for sleep");
        assert_eq!(lead(&store, DayPhase::WindDown).1.headline, "Time to wind down");
        let (_, late) = lead(&store, DayPhase::LateNight);
        assert_eq!((late.kind, late.headline.as_str()), (LeadKind::Sleep, "Rest when you're ready"));
    }

    #[test]
    fn high_readiness_is_an_option_and_not_today_keeps_it_hidden_for_the_day() {
        let store = fixture("HIGH", 85);
        let (v, l) = lead(&store, DayPhase::Day);
        assert_eq!((l.kind, l.headline.as_str(), l.dismissable), (LeadKind::Suggest, "Training is an option today", true));
        assert_eq!(v.call, Some(Effort::Push));
        act(&store, LocalAction::Dismiss { id: l.id.clone() }, at(13, 12));
        let (_, quiet) = lead(&store, DayPhase::Day);
        assert_eq!(quiet.kind, LeadKind::Quiet, "dismissed after sync and resume alike");
        assert!(quiet.reasons.iter().any(|r| r.gate == "Relevance"));
    }

    #[test]
    fn rest_outranks_readiness_and_lasts_through_the_night() {
        let store = fixture("HIGH", 85);
        act(&store, LocalAction::RestToday, at(13, 9));
        let (v, l) = lead(&store, DayPhase::Day);
        assert_eq!((l.kind, l.headline.as_str()), (LeadKind::Rest, "Rest day"));
        assert_eq!((v.call, v.focus.effort), (None, false), "no effort call on a rest day");
        assert_eq!(v.adapt.as_ref().unwrap().label, "Rest day");
        let local = local::load(&store);
        let tomorrow = today() + Duration::days(1);
        assert!(adapt(&local, tomorrow, true).rest, "after midnight, before waking");
        assert!(!adapt(&local, tomorrow, false).rest, "the next waking day prompts again");
    }

    #[test]
    fn illness_pauses_prompts_until_the_person_resumes() {
        let store = fixture("PRIME", 95);
        act(&store, LocalAction::Pause { reason: PauseReason::Ill, review: Some("2025-03-13".into()) }.clone(), at(10, 9));
        let (v, l) = lead(&store, DayPhase::Day);
        assert_eq!((l.kind, l.headline.as_str()), (LeadKind::Ill, "Ill · goals paused"));
        assert!(l.action.as_ref().is_some_and(|a| a.adapt));
        assert_eq!(v.call, None);
        let state = v.adapt.unwrap();
        assert_eq!((state.label.as_str(), state.blocked), ("Ill", true));
    }

    #[test]
    fn a_recording_today_leads_with_its_contribution() {
        let store = fixture("HIGH", 85);
        act(&store, LocalAction::SetGoals { targets: GoalTargets { strength_sessions: 2, ..Default::default() } }, at(10, 8));
        recording(&store, 7, "strength_training", 7);
        let (_, l) = lead(&store, DayPhase::Day);
        assert_eq!((l.kind, l.activity.as_deref(), l.line.as_deref()), (LeadKind::Done, Some("7"), Some("1 of 2 strength this week")));
        assert_eq!(l.headline, "Done today", "the recording below names it");
        assert_eq!(l.action.unwrap().route.as_deref(), Some("workout/7"));
    }

    #[test]
    fn a_walk_doesnt_lead_once_goals_exist() {
        let store = fixture("HIGH", 85);
        recording(&store, 8, "walking", 8);
        assert_eq!(lead(&store, DayPhase::Day).1.kind, LeadKind::Done);
        act(&store, LocalAction::SetGoals { targets: GoalTargets { strength_sessions: 2, ..Default::default() } }, at(10, 8));
        assert_eq!(lead(&store, DayPhase::Day).1.kind, LeadKind::Suggest);
    }

    #[test]
    fn your_session_leads_and_a_movement_note_asks_for_review() {
        let store = fixture("HIGH", 85);
        act(&store, LocalAction::AddIntention { date: DAY.into(), time: Some("18:30".into()), sport: Sport::Strength, type_key: None, name: None,
            duration_min: None, allocation: Allocation::default(), kind: IntentionKind::Workout, garmin_id: None }, at(13, 8));
        let (_, l) = lead(&store, DayPhase::Day);
        assert_eq!((l.kind, l.headline.as_str()), (LeadKind::Session, "Strength at 18:30"));
        act(&store, LocalAction::SetLimits { limits: Limits { excluded: vec![], note: "No overhead pressing".into() } }, at(13, 9));
        let (v, l) = lead(&store, DayPhase::Day);
        assert_eq!((l.kind, l.headline.as_str()), (LeadKind::Review, "Review your session"));
        assert_eq!(v.call, None, "limits hold back the effort call");
    }

    #[test]
    fn limits_replace_the_training_suggestion() {
        let store = fixture("HIGH", 85);
        act(&store, LocalAction::SetLimits { limits: Limits { excluded: vec![Sport::Running], note: String::new() } }, at(13, 9));
        let (v, l) = lead(&store, DayPhase::Day);
        assert_eq!((l.kind, l.line.as_deref()), (LeadKind::Limits, Some("Run excluded")));
        assert_eq!(v.adapt.unwrap().label, "Limits on");
    }

    #[test]
    fn low_readiness_suggests_an_easier_day_with_garmins_worst_factor_and_garmins_schedule_stays() {
        let store = fixture("LOW", 32);
        let (_, l) = lead(&store, DayPhase::Day);
        assert_eq!((l.kind, l.headline.as_str(), l.line.as_deref()), (LeadKind::Easy, "An easier day may fit", Some("Sleep holds it back most")));
        store.put("scheduled_workouts", "current", &at(13, 6).to_rfc3339(), &json!({"startDate": "2025-03-10", "endDate": "2025-03-23",
            "entries": [{"calendarDate": DAY, "workoutName": "Threshold intervals"}]})).unwrap();
        let (_, l) = lead(&store, DayPhase::Day);
        assert_eq!((l.kind, l.headline.as_str(), l.line.as_deref()), (LeadKind::Garmin, "An easier day may fit", Some("Garmin: Threshold intervals")));
    }

    #[test]
    fn a_garmin_coach_session_leads_with_its_attribution() {
        let store = fixture("HIGH", 85);
        let fetched = at(13, 8).to_rfc3339();
        store.put("coach_plans", "current", &fetched, &json!({"trainingPlanList": [{"trainingPlanId": 42, "trainingPlanCategory": "FBT_ADAPTIVE", "trainingStatus": {"statusKey": "Scheduled"}}]})).unwrap();
        store.put("coach_adaptive", "current", &fetched, &serde_json::from_str::<serde_json::Value>(include_str!("../tests/fixtures/coach_adaptive.json")).unwrap()).unwrap();
        let (_, l) = lead(&store, DayPhase::Day);
        assert_eq!((l.kind, l.headline.as_str(), l.line.as_deref(), l.intention.as_deref()), (LeadKind::Session, "Threshold today", Some("Garmin Coach"), Some("coach:2025-03-13:1")));
    }

    #[test]
    fn a_kept_session_leads_and_only_todays_sessions_can_be_kept() {
        let store = fixture("HIGH", 85);
        let add = |sport, time: &str| {
            act(&store, LocalAction::AddIntention { date: DAY.into(), time: Some(time.into()), sport, type_key: None, name: None,
                duration_min: None, allocation: Allocation::default(), kind: IntentionKind::Workout, garmin_id: None }, at(13, 8));
            local::load(&store).intentions.items.last().unwrap().id.clone()
        };
        add(Sport::Running, "07:00");
        let lift = add(Sport::Strength, "18:00");
        let (_, l) = lead(&store, DayPhase::Day);
        assert_eq!((l.headline.as_str(), l.kept), ("Run at 07:00", false));
        act(&store, LocalAction::KeepToday { plan_id: lift.clone() }, at(13, 9));
        let (_, l) = lead(&store, DayPhase::Day);
        assert_eq!((l.headline.as_str(), l.kept, l.intention.as_deref()), ("Strength at 18:00", true, Some(lift.as_str())));
        assert!(local::apply(&store, &LocalAction::KeepToday { plan_id: "garmin:2025-03-14:Intervals".into() }, at(13, 9), Weekday::Mon, &[], None).is_err());
        assert!(local::apply(&store, &LocalAction::KeepToday { plan_id: "i-none".into() }, at(13, 9), Weekday::Mon, &[], None).is_err());
        // Keeping Garmin's session puts it ahead of your own.
        store.put("scheduled_workouts", "current", &at(13, 6).to_rfc3339(), &json!({"startDate": "2025-03-10", "endDate": "2025-03-23",
            "entries": [{"calendarDate": DAY, "workoutName": "Tempo"}]})).unwrap();
        act(&store, LocalAction::KeepToday { plan_id: format!("garmin:{DAY}:Tempo") }, at(13, 9));
        let (_, l) = lead(&store, DayPhase::Day);
        assert_eq!((l.kind, l.kept, l.garmin.as_deref()), (LeadKind::Garmin, true, Some("garmin:2025-03-13:Tempo")));
        let tomorrow = NaiveDate::from_ymd_opt(2025, 3, 14).unwrap();
        let mut v = view(&store, DayPhase::Day);
        apply(&store, &mut v, tomorrow, at(14, 9).fixed_offset());
        assert!(!v.lead.unwrap().kept, "a choice lasts its day");
    }

    #[test]
    fn without_readiness_last_night_leads() {
        let store = Store::open(":memory:").unwrap();
        let mut v = view(&store, DayPhase::Day);
        v.focus.last_night = Some(LastNight { asleep_min: Some(400.), need_min: Some(480.), woke: None });
        apply(&store, &mut v, today(), at(13, 12).fixed_offset());
        let l = v.lead.unwrap();
        assert_eq!((l.kind, l.headline.as_str(), l.line.as_deref()), (LeadKind::Summary, "You slept 6:40", Some("1:20 below last night's need")));
    }
}
