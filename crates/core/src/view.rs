//! View models: exactly what each screen renders.
//!
//! These types are the contract between the Rust core and the Svelte UI.
//! `cargo test -p raring-core` regenerates their TypeScript twins in
//! `app/src/lib/generated/`. Values are raw (numbers, seconds, local ISO
//! timestamps) and the UI formats them; labels that come from Garmin enums
//! (levels, statuses) and every headline are produced here so they can be
//! tested.
//!
//! Every Garmin-derived field is optional: a watch may not report it, or the
//! day may not have synced yet. Screens hide what is missing.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use ts_rs::TS;

/// The color meaning shared by all screens: good is mint, fair amber,
/// poor coral, neutral gray.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum Tone {
    Good,
    Fair,
    Poor,
    Neutral,
}

/// Small icons used by chips, tiles and activity rows.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum Icon {
    Moon,
    Hrv,
    Flame,
    Heart,
    Lungs,
    Drop,
    Temp,
    Clock,
    Bolt,
    Run,
    Bike,
    Swim,
    Walk,
    Hike,
    Strength,
    Other,
}

/// Today's suggested effort, from the rules in docs/spec.md.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum Effort {
    Rest,
    Easy,
    Steady,
    Push,
}

/// An arc gauge. `band` marks a target range on the same scale (Garmin's
/// optimal acute-load range); `label` is the word under the number
/// ("High", "Good", "Optimal").
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Gauge {
    pub value: f64,
    pub max: f64,
    pub label: String,
    pub tone: Tone,
    pub band: Option<[f64; 2]>,
}

/// A pill under a headline: an icon, a value and/or a few words.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Chip {
    pub icon: Icon,
    pub value: Option<String>,
    pub text: Option<String>,
    pub tone: Tone,
}

/// One of Garmin's Training Readiness factors.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Factor {
    /// Stable key: sleep, recovery, hrv, acuteLoad, sleepHistory, stressHistory.
    pub key: String,
    pub name: String,
    /// Garmin's factor percentage, 0–100.
    pub percent: Option<f64>,
    pub tone: Tone,
    /// One sentence shown when the factor is tapped.
    pub detail: String,
}

/// One input behind a score, named from Garmin's own factors, events and
/// feedback.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct WhyInput {
    /// Stable key: a readiness factor key, overnight, stress, activity, nap,
    /// rest, acuteLoad, loadFocus, hrvStatus or vo2max.
    pub key: String,
    pub name: String,
    /// The value Garmin used, ready to show ("18 h", "+34"), when it sends one.
    pub value: Option<String>,
    /// Garmin's rating or label in words ("Very good", "Optimal"); None when
    /// Garmin gave none.
    pub rating: Option<String>,
    pub tone: Tone,
    /// What the input is, in one short sentence.
    pub detail: String,
}

/// The "Why" sheet behind a score: the inputs Garmin used, worst first.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Why {
    pub title: String,
    /// One line naming what helps the score and what holds it back.
    pub summary: String,
    /// Garmin's own feedback, rephrased, when it sends some.
    pub feedback: Option<String>,
    pub inputs: Vec<WhyInput>,
    /// Where the inputs come from.
    pub source: String,
}

/// Readiness and another score pointing different ways, explained in one
/// line (the spec's "Scores that differ" rule).
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Reconciliation {
    /// The other score: sleep, bodyBattery, hrvStatus or trainingStatus.
    pub key: String,
    /// The tap target: "Why Body Battery and readiness differ".
    pub label: String,
    /// The other score's tone, then readiness's.
    pub tones: [Tone; 2],
    pub line: String,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct BodyBatteryCard {
    pub now: Option<f64>,
    pub charged: Option<f64>,
    pub drained: Option<f64>,
    /// Garmin's levels in three-minute slots from the day's start, as the
    /// stress timeline; gaps are not zero.
    #[serde(default)]
    pub timeline: Vec<Option<f64>>,
}

/// Garmin's stress categories: rest 0–25, low 26–50, medium 51–75, high 76–100.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum StressLevel {
    Rest,
    Low,
    Medium,
    High,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct StressTime {
    pub level: StressLevel,
    pub seconds: f64,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct StressCard {
    /// Garmin's average for the day so far, 0–100.
    pub average: Option<f64>,
    pub max: Option<f64>,
    /// The category the average falls in.
    pub level: Option<StressLevel>,
    /// Garmin's word for the day: Calm, Balanced, Stressful, Very Stressful.
    pub qualifier: Option<String>,
    pub tone: Tone,
    /// Readings in three-minute slots from local midnight (Garmin's sampling
    /// interval). Off-wrist and unmeasurable readings are gaps.
    pub timeline: Vec<Option<f64>>,
    /// Minute ranges from midnight where Garmin could not measure stress
    /// because you were moving.
    pub active: Vec<[f64; 2]>,
    /// Last night's sleep, as minutes from midnight (clipped at midnight).
    pub sleep: Option<[f64; 2]>,
    /// Recorded workouts, as minute ranges from midnight clipped to the day.
    #[serde(default)]
    pub workouts: Vec<[f64; 2]>,
    /// Garmin's time in each category today; empty until reported.
    pub time: Vec<StressTime>,
    /// Time Garmin could not rate because you were moving, seconds.
    pub active_s: Option<f64>,
    /// Daily averages for the last 30 days, oldest first.
    pub trend: Vec<Option<f64>>,
}

/// How the stress calendar shows a day.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum StressDayKind {
    /// Garmin's word covers the whole day.
    Rated,
    /// Garmin's word covers only the time awake: it measured no sleep.
    AwakeOnly,
    /// Garmin couldn't rate the day: the watch was off for much of it.
    LowWear,
    /// Readings without a word from Garmin.
    Unrated,
    /// Nothing stored for the day.
    None,
    Future,
}

/// One day on the stress calendar.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct StressDay {
    pub date: String,
    pub kind: StressDayKind,
    /// Garmin's word ("Very Stressful"), also on an awake-only day.
    pub word: Option<String>,
    /// The word's colour on a rated day; neutral on every other.
    pub tone: Tone,
    /// Garmin's average for the day, 0–100.
    pub average: Option<f64>,
    /// Recorded workouts that started on the day.
    pub workouts: u32,
    /// The night before: the sleep that ended that morning.
    #[serde(default)]
    pub night: Option<NightStress>,
}

/// Garmin's rating of a night's stress, from its sleep score.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct NightStress {
    /// Garmin's word: Excellent, Good, Fair or Poor.
    pub rating: Option<String>,
    pub tone: Tone,
    /// Garmin's average stress while asleep, 0–100.
    pub average: Option<f64>,
}

/// A calendar month of Garmin's stress words, from local data.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct StressMonth {
    /// The month's first day, YYYY-MM-DD.
    pub month: String,
    /// The local day it was read on; later days are future.
    pub today: String,
    /// Every day of the month, oldest first.
    pub days: Vec<StressDay>,
    /// The first day of the earliest month with stress on this phone.
    pub first: Option<String>,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct HeartRateDay {
    pub date: String,
    /// Garmin's daily resting value; not Health Status's overnight mean.
    pub resting: Option<f64>,
    pub minimum: Option<f64>,
    pub maximum: Option<f64>,
    /// Two-minute elapsed slots from the source day's UTC start. Gaps are not zero.
    pub timeline: Vec<Option<f64>>,
    /// Actual length of the source day, in elapsed minutes (23, 24 or 25 hours).
    pub duration_min: f64,
    /// Recorded sleep intervals clipped to this day, in elapsed minutes.
    pub sleep: Vec<[f64; 2]>,
    /// Recorded activities clipped to this day, in elapsed minutes.
    pub activities: Vec<[f64; 2]>,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct HrvCard {
    /// Last night's average, ms.
    pub last_night: Option<f64>,
    /// Garmin's status word: Balanced, Unbalanced, Low, Poor.
    pub status: Option<String>,
    pub tone: Tone,
    /// Garmin's balanced baseline range, ms.
    pub baseline: Option<[f64; 2]>,
    /// Nightly averages for the last 7 nights, oldest first.
    pub week: Vec<Option<f64>>,
}

#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum VitalKind {
    RestingHr,
    Hrv,
    Respiration,
    Spo2,
    SkinTemp,
}

#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum VitalRangeSource {
    Garmin,
    Raring,
    None,
}

/// One overnight vital. Garmin's status takes precedence over range math;
/// the range source distinguishes Garmin's baseline from Raring's fallback.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct VitalReading {
    pub kind: VitalKind,
    pub value: Option<f64>,
    pub unit: String,
    pub tone: Tone,
    /// Garmin's baseline, or Raring's fallback where Garmin provides none.
    pub normal: Option<[f64; 2]>,
    /// The bar's scale, wide enough to show value and range.
    pub scale: [f64; 2],
    /// Last 30 nights, oldest first.
    pub trend: Vec<Option<f64>>,
    pub status: Option<String>,
    pub range_source: VitalRangeSource,
}

/// A night's reading against its own usual range (Rules: Night vitals on Today).
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum UsualState {
    Within,
    Above,
    Below,
    /// No usual range yet (too few nights, or none from Garmin).
    Unset,
    /// Garmin is still building the baseline.
    Calibrating,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct NightReading {
    pub value: Option<f64>,
    pub unit: String,
    pub usual: Option<[f64; 2]>,
    pub range_source: VitalRangeSource,
    pub state: UsualState,
    /// Outside the range: the value minus the range's middle, whole units.
    pub delta: Option<f64>,
    /// Good inside the range, fair outside it either way (attention, not a
    /// rating); neutral without a range.
    pub tone: Tone,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct NightVitals {
    /// Garmin's daily resting HR for the day the night ends, against the
    /// resting-HR chart's usual range. Never Health Status's HR, which is the
    /// overnight average.
    pub resting_hr: NightReading,
    /// Garmin's overnight average heart rate (Health Status HR when sent).
    pub sleep_hr: NightReading,
    /// Last night's average HRV against Garmin's baseline.
    pub hrv: NightReading,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct VitalsCard {
    pub readings: Vec<VitalReading>,
    /// Garmin outliers plus readings outside a fallback range.
    pub off_count: u32,
    /// At least one reading has a comparable range.
    pub baseline_ready: bool,
}

/// How an activity's distance reads: kilometres or miles by Garmin's
/// measurement setting, and metres or yards (the pool's unit) for swims.
/// Pace, speed and elevation follow: statute units for `Mi` and `Yd`.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum DistanceUnit {
    #[default]
    Km,
    Mi,
    M,
    Yd,
}

/// A page of older stored recordings for Training → Activities.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ActivityPage {
    pub activities: Vec<ActivityCard>,
    /// Older recordings may remain.
    pub more: bool,
}

/// One month of stored recordings for Training → Activities' calendar.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ActivityMonth {
    /// The month's first day.
    pub month: String,
    /// The calendar's first cell: the start of the week holding the 1st, on
    /// the weekday Garmin's weeks start.
    pub grid_start: String,
    /// The local date; later days have nothing recorded yet.
    pub today: String,
    /// The oldest stored recording's local date; no earlier month opens.
    pub first: Option<String>,
    /// The month's recordings, newest first.
    pub activities: Vec<ActivityCard>,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ActivityCard {
    /// Garmin activity id as a string (it exceeds JavaScript's safe integers).
    pub id: String,
    pub icon: Icon,
    pub name: String,
    /// Local start time, ISO 8601 without offset.
    pub start_local: String,
    pub duration_s: f64,
    pub distance_km: Option<f64>,
    pub load: Option<f64>,
    /// Seconds in heart-rate zones 1–5. Missing/invalid durations are unknown,
    /// distinct from an explicitly recorded zero.
    pub zone_seconds: Vec<Option<f64>>,
    pub aerobic_te: Option<f64>,
    /// Garmin's label for the aerobic effect: Minor, Maintaining, Improving…
    pub aerobic_label: Option<String>,
    pub anaerobic_te: Option<f64>,
    pub anaerobic_label: Option<String>,
    /// Garmin's primary benefit: Base, Tempo, Threshold, VO2 Max…
    pub benefit: Option<String>,
    /// Average heart rate, bpm.
    #[serde(default)]
    pub avg_hr: Option<f64>,
    /// Garmin's moderate plus twice vigorous minutes for this activity.
    #[serde(default)]
    pub intensity_min: Option<f64>,
    /// Garmin's vigorous intensity minutes for this activity.
    #[serde(default)]
    pub vigorous_min: Option<f64>,
    #[serde(default)]
    pub distance_unit: DistanceUnit,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct FitnessAgeCard {
    pub fitness_age: Option<f64>,
    pub chronological_age: Option<f64>,
    /// Monthly values for the last 12 months, oldest first.
    pub trend: Vec<Option<f64>>,
    /// Change over the last 12 months (negative is younger).
    pub change_12m: Option<f64>,
    /// The day of Garmin's estimate shown.
    #[serde(default)]
    pub recorded_on: Option<String>,
}

/// Fitness Age's vigorous-minutes driver, live from Garmin's daily summaries:
/// the 42 days ending on `end` that Garmin averages per week (Rules: Fitness
/// Age window).
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct VigorousWindow {
    /// The window's last day, YYYY-MM-DD.
    pub end: String,
    /// Six seven-day blocks ending on `end`, oldest first: Garmin's daily
    /// vigorous minutes summed. A block with a day missing is unknown.
    pub weeks: Vec<Option<f64>>,
    /// Garmin's vigorous minutes on `end` itself.
    pub today: Option<f64>,
    /// The 42 days' minutes ÷ 6, Garmin's weekly average; unknown unless every
    /// day has Garmin's figure.
    pub average: Option<f64>,
    /// The same average without `end`'s minutes.
    pub before_today: Option<f64>,
    /// Garmin's weekly target for the driver, minutes.
    pub target: f64,
    /// The Fitness Age Garmin gives for reaching the target, when it says.
    pub age_at_target: Option<f64>,
}

/// Where Today shows its Fitness Age card.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum FitnessPlace {
    /// Under what was recorded today: Garmin's estimate dropped today.
    Hero,
    /// After This week: vigorous minutes recorded today.
    AfterWeek,
}

/// Today's link to Fitness Age: the vigorous window, a new lower estimate, and
/// on Garmin's push call before any vigorous minutes, a line naming the lever.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TodayFitness {
    pub window: VigorousWindow,
    /// Garmin's latest Fitness Age estimate.
    pub age: Option<f64>,
    /// The previous estimate, when today's is lower in whole years.
    pub dropped_from: Option<f64>,
    /// Where the card goes; None on a day that moved neither.
    pub card: Option<FitnessPlace>,
    /// Name vigorous minutes as the lever under Garmin's push call.
    pub lever_line: bool,
}

/// Where the person is in their own day, anchored to their wake time and
/// tonight's sleep plan. `Overview` is the fixed layout.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum DayPhase {
    Overview,
    Wake,
    Day,
    Evening,
    WindDown,
    LateNight,
}

/// Today's advice cards and where Your day sits among them. Metric cards
/// follow them in a fixed order.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum TodayCard {
    /// Today's focus: effort and the limiting readiness factor.
    Focus,
    SleepPlan,
    /// Vitals, moved up when two or more are outside their range.
    Vitals,
    /// Your day and recorded today. Before any other card (bar the focus
    /// and vitals) it stays with the hero; after tonight's plan it follows
    /// it in a compact form.
    Day,
}

/// The displayed night's recorded sleep.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct LastNight {
    pub asleep_min: Option<f64>,
    /// Garmin's need for that night.
    pub need_min: Option<f64>,
    /// Recorded wake time, HH:MM.
    pub woke: Option<String>,
}

/// What leads Today right now.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TodayFocus {
    pub phase: DayPhase,
    /// Today's cards in order; the first advice card leads.
    pub cards: Vec<TodayCard>,
    /// When the phase next changes, local ISO 8601 without offset; Today is
    /// read again then.
    pub until: Option<String>,
    /// Show the effort call. Off once winding down, and before a night that
    /// hasn't synced.
    pub effort: bool,
    /// Tonight's plan in full; otherwise folded to one line.
    pub plan_open: bool,
    /// The displayed night has no recorded sleep. Neutral, never a verdict.
    pub sleep_missing: bool,
    pub last_night: Option<LastNight>,
    /// Why Today keeps its fixed layout, one line.
    pub note: Option<String>,
    /// Garmin's recovery time at the latest readiness, hours.
    pub recovery_hours: Option<f64>,
    /// Where the phases' wake time comes from when it isn't recorded nights:
    /// "Garmin sleep schedule".
    #[serde(default)]
    pub schedule: Option<String>,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TodayView {
    /// Local date, YYYY-MM-DD.
    pub date: String,
    /// Selected watch's last Garmin Connect upload, UTC ISO 8601.
    pub watch_last_upload: Option<String>,
    /// True only when today's sleep is absent after a known wake time and
    /// the watch has not uploaded since that expected wake.
    pub watch_sleep_upload_stale: bool,
    /// Missing metric keys and explanations; absent means capability unknown.
    pub missing_metric_reasons: BTreeMap<String, String>,
    pub readiness: Option<Gauge>,
    /// Readiness scores for the last 30 days, oldest first (the Readiness
    /// detail chart).
    pub readiness_trend: Vec<Option<f64>>,
    pub sleep: Option<Gauge>,
    pub sleep_plan: Option<SleepPlan>,
    pub load: Option<Gauge>,
    /// A few words: "Ready to push", "Take it easy".
    pub headline: String,
    pub factors: Vec<Factor>,
    pub readiness_why: Option<Why>,
    pub body_battery_why: Option<Why>,
    /// Shown on Today behind a tap when two scores point different ways.
    pub reconcile: Option<Reconciliation>,
    pub call: Option<Effort>,
    /// For example "Morning Run done · 186 load".
    pub call_note: Option<String>,
    pub body_battery: BodyBatteryCard,
    pub stress: StressCard,
    pub heart_rate_day: HeartRateDay,
    pub hrv: HrvCard,
    pub vitals: VitalsCard,
    pub activities: Vec<ActivityCard>,
    pub fitness_age: FitnessAgeCard,
    /// True on Mondays once the weekly check-in exists.
    pub checkin_ready: bool,
    pub focus: TodayFocus,
    pub planned: PlannedTraining,
    /// This week's plan; None until targets are set.
    pub plan: Option<PlanWeek>,
    /// What leads Today now; only for the live day, never a historical read.
    #[serde(default)]
    pub lead: Option<TodayLead>,
    /// Today's circumstances; only for the live day.
    #[serde(default)]
    pub adapt: Option<AdaptState>,
    /// The calendar day on one axis; only for the live day.
    #[serde(default)]
    pub day: Option<DayTimeline>,
    /// Last night's resting HR, Sleep HR and HRV against their usual ranges,
    /// for the morning and the day; none from the evening on or before the
    /// night syncs.
    #[serde(default)]
    pub night_vitals: Option<NightVitals>,
    /// Fitness Age's vigorous driver on the calendar day; only for the live
    /// day, while Garmin's estimate has a vigorous-minutes target.
    #[serde(default)]
    pub fitness: Option<TodayFitness>,
}

#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum Stage {
    Awake,
    Rem,
    Light,
    Deep,
}

/// A run of one sleep stage. Minutes are counted from `SleepView::start_local`.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct StageSegment {
    pub stage: Stage,
    pub start_min: f64,
    pub minutes: f64,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct StageTotal {
    pub stage: Stage,
    pub minutes: f64,
    /// Against this stage's recorded mean in the 30 calendar days ending
    /// on the selected night (including it); positive means more.
    pub usual_delta_min: Option<f64>,
    /// That mean, in minutes; never a target or a rating.
    #[serde(default)]
    pub comparison_min: Option<f64>,
    /// This stage's denominator. Missing/invalid values do not count; zero does.
    #[serde(default)]
    pub comparison_nights: u32,
}

#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum OvernightKind {
    Hrv,
    HeartRate,
    Respiration,
    Spo2,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct OvernightTile {
    pub kind: OvernightKind,
    pub value: Option<f64>,
    pub unit: String,
    /// Samples across the night, evenly spaced; invalid or missing readings remain gaps.
    pub series: Vec<Option<f64>>,
}

#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum SleepNeedSource {
    Tonight,
    Latest,
    Average,
}

/// Who chose tonight's bedtime: Garmin's recommended window, or Raring's
/// fallback from wake history and the need.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum BedtimeSource {
    Garmin,
    WakeHistory,
}

/// One of Garmin's reasons for changing tonight's sleep need.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct NeedAdjustment {
    /// Training, Sleep history, HRV or Naps.
    pub factor: String,
    /// Garmin's code in a few words: "high training load".
    pub detail: String,
}

/// Garmin's sleep need for tonight, with Garmin's bedtime window when it sends
/// one, otherwise a bedtime derived from the last 14 nights' wake times. The
/// one plan behind Today, Sleep and the bedtime reminder.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SleepPlan {
    /// The morning this night ends, YYYY-MM-DD; one reminder per night.
    pub night: String,
    /// Suggested time to be asleep tonight, HH:MM: the end of Garmin's window,
    /// or the median wake time minus the need.
    pub bedtime: String,
    /// Start of Garmin's recommended bedtime window, HH:MM, when Garmin sends one.
    pub bedtime_start: Option<String>,
    pub bedtime_source: BedtimeSource,
    /// Time to be in bed, HH:MM: the asleep-by time minus the time to fall
    /// asleep (Raring's, from Settings).
    #[serde(default)]
    pub in_bed: String,
    #[serde(default)]
    pub in_bed_at: String,
    /// The time to fall asleep the plan allows, minutes.
    #[serde(default)]
    pub fall_asleep_min: u32,
    /// The need plus the time to fall asleep: time in bed tonight, minutes.
    #[serde(default)]
    pub in_bed_min: f64,
    /// When tonight's sleep should end: the asleep-by time plus the need.
    #[serde(default)]
    pub wake_at: String,
    /// Time to start winding down, HH:MM: the lead before the earlier of
    /// Garmin's window start and the in-bed time.
    pub wind_down: String,
    pub wind_down_lead_min: u32,
    /// Local wall-clock times, ISO 8601 without offset, so screens and
    /// reminders agree on the date across midnight.
    pub wind_down_at: String,
    pub bedtime_start_at: Option<String>,
    pub bedtime_at: String,
    /// Median local wake time, HH:MM, when any wake time is recorded.
    pub wake_time: Option<String>,
    /// When Raring last read Garmin's need for tonight (it can change after
    /// a nap or a workout); none for older sources.
    #[serde(default)]
    pub need_read_at: Option<String>,
    pub need_min: f64,
    pub source: SleepNeedSource,
    /// Garmin's unadjusted need, minutes, sent with tonight's need.
    pub baseline_min: Option<f64>,
    /// Garmin's reasons for tonight's need; empty when unchanged or unknown.
    pub adjustments: Vec<NeedAdjustment>,
    /// One line on why the need is what it is, from Garmin's feedback.
    pub reason: Option<String>,
    /// One line toward steadier timing, from Garmin's alignment status when
    /// current, otherwise from wake history.
    pub tip: Option<String>,
    /// Recorded wake times in the 14-day window; sparse history stays explicit.
    pub wake_nights: u32,
    /// No recorded sleep has synced for the current day, so the plan rests
    /// on older nights; reminders skip it.
    pub stale: bool,
}

/// Where Garmin's Sleep Alignment stands for the latest night.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum AlignmentState {
    /// Garmin rated last night.
    Current,
    /// Last night has no rating; the latest earlier one is shown.
    Stale,
    /// Garmin says it is still learning the rhythm.
    Calibrating,
    /// Garmin sent no alignment for recent nights.
    Unavailable,
}

/// One night against Garmin's optimal sleep window. Minutes count from local
/// midnight of the morning the night ends; negative is the evening before.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AlignmentNight {
    pub date: String,
    /// Garmin's status in a word: Aligned, Ahead, Behind, Jet lag.
    pub status: Option<String>,
    pub tone: Tone,
    /// Garmin's optimal sleep window.
    pub window: Option<[f64; 2]>,
    /// Recorded sleep that night.
    pub sleep: Option<[f64; 2]>,
}

/// Garmin's Sleep Alignment: sleep timing against the window its rhythm
/// model suggests. Not tonight's recommended bedtime, and not Raring's usual
/// windows.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SleepAlignment {
    pub state: AlignmentState,
    /// The night shown in detail; None when unavailable.
    pub latest: Option<AlignmentNight>,
    /// One sentence for the status or state.
    pub detail: String,
    /// The last 7 nights, oldest first, including nights without a rating.
    pub history: Vec<AlignmentNight>,
}

/// Garmin's own averages over the last seven days (Sleep Consistency).
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SleepAverages {
    /// HH:MM.
    pub bedtime: String,
    pub wake_time: String,
    pub nights: u32,
    /// The newest night included, YYYY-MM-DD.
    pub through: String,
}

/// One night's bed and wake time, as hours on a 24h clock (23.5 = 23:30).
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct NightWindow {
    pub date: String,
    pub bed_hour: f64,
    pub wake_hour: f64,
}

#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum ReadState {
    #[default]
    Unknown,
    Unavailable,
    Stale,
    Empty,
    Available,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Nap {
    pub start_local: Option<String>,
    pub end_local: Option<String>,
    pub duration_min: Option<f64>,
    pub feedback: Option<String>,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct JournalEntry {
    pub name: String,
    pub value: Option<String>,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct JournalDay {
    pub date: String,
    pub state: ReadState,
    pub entries: Vec<JournalEntry>,
}

/// One page's per-day detail that an explicit open can still fetch.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum DayDetailPart {
    Sleep,
    Stress,
    Vitals,
    /// Garmin's Fitness Age estimate as of that day.
    FitnessAge,
    Workout,
}

/// Per-day reads this phone has stored or still needs for an opened day.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DayDetail {
    /// Parts with a per-day read this phone has never stored.
    #[serde(default)]
    pub missing: Vec<DayDetailPart>,
    /// Parts Garmin answered with nothing for this day.
    #[serde(default)]
    pub unavailable: Vec<DayDetailPart>,
}

/// One night's recorded sleep against Garmin's need for it.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct NightNeed {
    /// The morning the night ended.
    pub date: String,
    pub asleep_min: Option<f64>,
    pub need_min: Option<f64>,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SleepView {
    /// The morning the night ended, YYYY-MM-DD.
    pub date: String,
    #[serde(default)]
    pub naps: Vec<Nap>,
    #[serde(default)]
    pub naps_state: ReadState,
    pub start_local: Option<String>,
    pub end_local: Option<String>,
    pub score: Option<Gauge>,
    pub asleep_min: Option<f64>,
    /// Garmin's sleep need, when the watch reports it.
    pub need_min: Option<f64>,
    pub headline: String,
    pub stages: Vec<StageSegment>,
    pub totals: Vec<StageTotal>,
    pub overnight: Vec<OvernightTile>,
    /// The last 14 nights, oldest first.
    pub consistency: Vec<NightWindow>,
    /// Typical spread of bedtimes, minutes (the "±18 min" pill).
    pub consistency_spread_min: Option<f64>,
    pub bed_window: Option<[f64; 2]>,
    pub wake_window: Option<[f64; 2]>,
    /// Garmin's seven-day average bed and wake times, when synced.
    pub garmin_averages: Option<SleepAverages>,
    /// Garmin's Sleep Alignment; None without any recent sleep.
    pub alignment: Option<SleepAlignment>,
    pub sleep_plan: Option<SleepPlan>,
    /// The seven nights ending this one, oldest first; a missing night is
    /// all `None`.
    #[serde(default)]
    pub recent: Vec<NightNeed>,
    /// Live read in the evening, wind-down or late night: Tonight opens first.
    #[serde(default)]
    pub tonight_first: bool,
    /// Live read: a wind-down was reported for the evening of the night
    /// Tonight shows.
    #[serde(default)]
    pub wound_down: bool,
    /// Live read: that evening's wind-down can be reported (from two hours
    /// before the night's window).
    #[serde(default)]
    pub wind_down_open: bool,
    /// A past night's detail an explicit open can fetch.
    #[serde(default)]
    pub detail: DayDetail,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DayLoad {
    /// Local date, YYYY-MM-DD.
    pub date: String,
    pub load: f64,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct LoadFocusBar {
    /// lowAerobic, highAerobic or anaerobic.
    pub key: String,
    pub name: String,
    pub value: f64,
    pub target: Option<[f64; 2]>,
    pub shortage: bool,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RacePrediction {
    /// 5K, 10K, Half, Full.
    pub label: String,
    pub seconds: f64,
}

#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum PlannedState {
    Unknown,
    Stale,
    Partial,
    Empty,
    Available,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct PlannedSession {
    pub name: String,
    /// Garmin calendar date, not a UTC instant.
    pub date: String,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct PlannedTraining {
    pub state: PlannedState,
    pub session: Option<PlannedSession>,
    pub coach: Option<String>,
    /// Inclusive Garmin calendar dates from the cached schedule request.
    pub coverage_start: Option<String>,
    pub coverage_end: Option<String>,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UpcomingRace {
    pub name: String,
    /// The all-day calendar date is never converted through a timezone.
    pub date: String,
    pub days_until: i32,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct GearItem {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub status: String,
    pub distance: Option<f64>,
    pub threshold: Option<f64>,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct GearView {
    pub items: Vec<GearItem>,
    pub unit: String,
    /// API fetch time, not a watch measurement or activity time.
    pub fetched_at: String,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RecordedTrainingMetric {
    /// runningHr, runningPace, runningPower, cyclingFtp, heat or altitude.
    pub kind: String,
    pub value: f64,
    /// bpm, s/km, s/mi, W, %, m or ft.
    pub unit: String,
    pub recorded_on: String,
    pub stale: bool,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TrainingView {
    /// Acute load on a scale, with Garmin's optimal range as the band.
    pub acute: Option<Gauge>,
    /// Garmin's training status: Productive, Maintaining, Recovery…
    pub status: Option<String>,
    pub status_tone: Tone,
    pub status_why: Option<Why>,
    pub headline: String,
    pub chips: Vec<Chip>,
    /// The last 7 days, oldest first.
    pub week: Vec<DayLoad>,
    pub load_focus: Vec<LoadFocusBar>,
    pub vo2max: Option<f64>,
    pub vo2max_change_30d: Option<f64>,
    /// Weekly values, oldest first.
    pub vo2max_trend: Vec<Option<f64>>,
    pub races: Vec<RacePrediction>,
    pub endurance_score: Option<f64>,
    pub hill_score: Option<f64>,
    pub running_tolerance_km: Option<f64>,
    /// Garmin's latest recorded readings; absent metrics are not zero.
    pub thresholds: Vec<RecordedTrainingMetric>,
    pub workouts: Vec<ActivityCard>,
    pub gear: Option<GearView>,
    pub planned: PlannedTraining,
    pub race: Option<UpcomingRace>,
    pub race_state: PlannedState,
    /// This week's strength sets, when any strength session has sets in
    /// the last 12 weeks.
    pub strength: Option<StrengthWeek>,
    /// This plan week's lanes against goals and the usual week; live only.
    #[serde(default)]
    pub lanes: Option<TrainingWeek>,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Stat {
    /// distance, time, pace, avgHr, maxHr, calories, power, cadence, ascent…
    pub key: String,
    pub label: String,
    pub value: f64,
    pub unit: String,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct HrSample {
    pub t_s: f64,
    pub bpm: f64,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Split {
    pub index: u32,
    pub distance_km: f64,
    pub seconds: f64,
    pub avg_hr: Option<f64>,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct WorkoutView {
    pub summary: ActivityCard,
    pub recovery_hours: Option<f64>,
    pub stats: Vec<Stat>,
    /// Simplified route as [lat, lon] pairs; empty without GPS.
    pub route: Vec<[f64; 2]>,
    /// Heart rate downsampled to at most ~300 points.
    pub heart_rate: Vec<HrSample>,
    /// Lower bounds of zones 2–5, bpm.
    pub zone_floors: Vec<f64>,
    pub splits: Vec<Split>,
    /// Garmin's recorded sets, for strength activities that have them.
    pub strength: Option<StrengthSession>,
    /// Stored streams beyond heart rate, on one elapsed-time axis; only the
    /// ones this activity recorded.
    #[serde(default)]
    pub along: Option<AlongTheWay>,
    /// What it did beyond training effect: intensity minutes, Body Battery,
    /// stamina and impact load, as Garmin sent them.
    #[serde(default)]
    pub more: Vec<Stat>,
    /// Garmin's other summary fields this activity has, beyond `stats` and
    /// `more`, in groups; a group without fields is left out.
    #[serde(default)]
    pub recorded: Vec<RecordedGroup>,
    /// The person's own rating on the watch after the activity, when given.
    #[serde(default)]
    pub self_evaluation: Option<SelfEvaluation>,
    /// Garmin's training effect in its bands, with one plain line each.
    #[serde(default)]
    pub effects: Vec<EffectLine>,
    /// Garmin's VO₂ max estimate, when a new one was stored that day.
    #[serde(default)]
    pub vo2max_new: Option<f64>,
    /// How it counted toward the week's goals; live reads only.
    #[serde(default)]
    pub counted: Option<WorkoutCount>,
    /// Per-activity reads this phone has never stored for the workout.
    #[serde(default)]
    pub detail: DayDetail,
    /// The route card's street map (Rules: Street maps); none without a
    /// route or with street maps off.
    #[serde(default)]
    pub map: Option<RouteMapView>,
}

/// Streets, water and parks under a workout's route, drawn by the core from
/// OpenStreetMap tiles stored on the phone.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RouteMapView {
    pub frame: MapFrame,
    pub state: RouteMapState,
    /// Drawn bottom to top; empty until `ready`.
    pub layers: Vec<MapLayer>,
}

/// Places Web Mercator coordinates (0–1 across the world, y down) on the
/// 358×176 route card: `x = (mercatorX − x0) × scale`, likewise for y.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MapFrame {
    pub x0: f64,
    pub y0: f64,
    pub scale: f64,
}

#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum RouteMapState {
    /// Drawn from map data on the phone.
    Ready,
    /// This area's map data isn't on the phone yet; the route card asks for it.
    Missing,
}

/// One kind of map feature as SVG path data in card coordinates.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MapLayer {
    pub kind: MapLayerKind,
    pub path: String,
}

/// Areas are filled; the rest are lines.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum MapLayerKind {
    /// Parks, grass and woods.
    Green,
    Water,
    Waterway,
    /// Footways, cycleways, trails and piers.
    Path,
    /// Residential and service streets and tracks.
    Minor,
    /// Secondary and tertiary roads.
    Road,
    /// Motorways, trunk and primary roads.
    Major,
}

/// Weights follow Garmin's measurement setting.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum WeightUnit {
    Kg,
    Lb,
}

/// The body map's figure. Defaults from the Garmin profile.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum Figure {
    Female,
    Male,
}

/// Strength's optional body-map figure override; `Profile` follows Garmin.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum FigureChoice {
    #[default]
    Profile,
    Female,
    Male,
}

/// Raring's muscle groups: Garmin's catalog muscles, grouped for the body map.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum Muscle {
    Chest,
    Back,
    Shoulders,
    Arms,
    Core,
    Hips,
    Quads,
    Hamstrings,
    Glutes,
    Calves,
}

/// Why a set may be miscounted (docs/spec.md, Rules). Flags never change
/// Garmin's numbers.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum SetFlag {
    /// Garmin recorded 0 reps.
    ZeroReps,
    /// Garmin has no rep count for the set.
    MissingReps,
    /// At least 1.8 times the median of the exercise's other sets.
    Doubled,
    /// Garmin couldn't name the exercise.
    Unnamed,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MuscleSets {
    pub muscle: Muscle,
    /// Sets, or an average of weekly sets.
    pub sets: f64,
}

/// One set as Garmin recorded it.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct StrengthSet {
    pub reps: Option<u32>,
    /// In `unit`; none for bodyweight sets.
    pub weight: Option<f64>,
    pub duration_s: Option<f64>,
    pub flags: Vec<SetFlag>,
    /// No earlier set of this exercise was as heavy for as many reps.
    pub record: bool,
}

/// A weight and rep count, such as a set from last time or a record.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Lift {
    pub weight: Option<f64>,
    pub reps: Option<u32>,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct StrengthExercise {
    /// Garmin's exercise key, "CATEGORY/NAME" or "CATEGORY"; none when
    /// Garmin couldn't name it.
    pub key: Option<String>,
    pub name: Option<String>,
    pub muscles: Vec<Muscle>,
    pub sets: Vec<StrengthSet>,
    /// The previous session with this exercise.
    pub last_date: Option<String>,
    pub last_sets: Vec<Lift>,
    /// The heaviest record set this session.
    pub record: Option<Lift>,
}

/// A strength activity's sets, grouped by exercise in the order done.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct StrengthSession {
    pub unit: WeightUnit,
    pub figure: Figure,
    /// When Garmin's sets were last read, RFC 3339; missing when never read.
    pub checked_at: Option<String>,
    /// Garmin's active sets; rests are not sets.
    pub sets: u32,
    /// The sum of the known rep counts; none when no set has a count.
    pub reps: Option<u32>,
    /// Active sets without a rep count, left out of `reps`.
    pub unknown_rep_sets: u32,
    pub rest_s: f64,
    pub records: u32,
    /// False when no set has reps or a named exercise: the watch timed the
    /// session without recording sets.
    pub sets_recorded: bool,
    pub zero_rep_sets: u32,
    pub missing_rep_sets: u32,
    pub doubled_sets: u32,
    pub unnamed_sets: u32,
    pub muscles: Vec<MuscleSets>,
    pub exercises: Vec<StrengthExercise>,
}

/// The Training card: this week's strength.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct StrengthWeek {
    pub figure: Figure,
    /// First day of this week, following Garmin's week-start setting.
    pub week_start: String,
    /// Sets, records and muscles count only sessions with sets.
    pub sets: u32,
    pub sessions_with_sets: u32,
    /// Every Garmin strength activity this week, including ones timed
    /// without sets. The weekly plan counts these.
    pub strength_activities: u32,
    pub records: u32,
    pub muscles: Vec<MuscleSets>,
}

/// A strength activity in the history list.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct StrengthActivity {
    pub activity: ActivityCard,
    /// False for sessions timed without sets, or with no sets from Garmin.
    pub sets_recorded: bool,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct WeekMuscles {
    pub week_start: String,
    pub muscles: Vec<MuscleSets>,
}

/// One session of one exercise.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ExerciseSession {
    pub activity_id: String,
    pub date: String,
    /// The heaviest unflagged set.
    pub top: Option<Lift>,
    /// Reps of every set, in order.
    pub reps: Vec<Option<u32>>,
    pub record: bool,
}

/// The heaviest weight lifted for at least `reps` reps.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RepBest {
    pub reps: u32,
    pub weight: Option<f64>,
    pub date: Option<String>,
}

/// The most reps in one set of a bodyweight exercise (no weight to compare).
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RepCountBest {
    pub reps: u32,
    pub date: String,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ExerciseHistory {
    pub key: String,
    pub name: String,
    pub muscles: Vec<Muscle>,
    /// The last 12 weeks, newest first.
    pub sessions: Vec<ExerciseSession>,
    /// Across all stored sessions.
    pub bests: Vec<RepBest>,
    /// Bodyweight work has no weight to compare; its best is the most reps in
    /// one counted set, across all stored sessions.
    pub reps_best: Option<RepCountBest>,
}

/// Strength history: the last 12 weeks.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct StrengthView {
    pub unit: WeightUnit,
    pub figure: Figure,
    pub date: String,
    pub week_start: String,
    pub this_week: Vec<MuscleSets>,
    /// Weekly average over the four weeks before this one.
    pub average_4w: Vec<MuscleSets>,
    /// Total sets over the four weeks ending with this one.
    pub last_4w: Vec<MuscleSets>,
    /// Twelve weeks, oldest first, ending with this week.
    pub weeks: Vec<WeekMuscles>,
    pub sessions_with_sets_this_week: u32,
    /// Exercises done in the last 12 weeks, most recent first.
    pub exercises: Vec<ExerciseHistory>,
    /// Strength activities in the last 12 weeks, newest first, including
    /// ones without sets.
    pub workouts: Vec<StrengthActivity>,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Driver {
    /// vo2max, restingHr, bmi, bodyFat, vigorousMinutes.
    pub key: String,
    pub name: String,
    pub value: Option<f64>,
    pub unit: String,
    /// Garmin's target for this driver, when it gives one.
    pub target: Option<f64>,
    pub met: bool,
    pub tone: Tone,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Lever {
    pub key: String,
    pub name: String,
    pub current: f64,
    pub target: f64,
    pub unit: String,
    /// The Fitness Age reached by hitting the target, when Garmin says.
    pub age_if_met: Option<f64>,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct BodyWeightCard {
    pub value: f64,
    pub unit: WeightUnit,
    pub measured_on: String,
    pub bmi: Option<f64>,
    pub body_fat: Option<f64>,
    /// Last 12 weeks, oldest first; gaps are not zero.
    pub trend: Vec<Option<f64>>,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct HealthView {
    /// Local date, YYYY-MM-DD.
    pub date: String,
    #[serde(default)]
    pub journal: Vec<JournalDay>,
    #[serde(default)]
    pub journal_enabled: bool,
    pub fitness_age: FitnessAgeCard,
    pub lever: Option<Lever>,
    pub drivers: Vec<Driver>,
    /// The six weeks of vigorous minutes Garmin averages, ending on the
    /// calendar day (the requested day for a past one).
    #[serde(default)]
    pub vigorous: Option<VigorousWindow>,
    pub vitals: VitalsCard,
    pub stress: StressCard,
    pub heart_rate_day: HeartRateDay,
    pub body_weight: Option<BodyWeightCard>,
    /// This week's intensity minutes, when Garmin reported any day of it.
    pub intensity: Option<IntensityCard>,
    /// Raring's usual range for Garmin's daily resting HR over the last 30
    /// days (Rules: Normal range), the band on the resting-HR chart. Never
    /// Health Status's Sleep HR baseline, which measures something else.
    #[serde(default)]
    pub resting_hr_range: Option<[f64; 2]>,
    /// That night's resting HR, Sleep HR and HRV against their usual ranges,
    /// as on Today.
    #[serde(default)]
    pub night_vitals: Option<NightVitals>,
    /// A past day's detail an explicit open can fetch.
    #[serde(default)]
    pub detail: DayDetail,
}

#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum CheckinArea {
    Readiness,
    Sleep,
    Training,
    Health,
}

/// One swipeable card of the weekly check-in.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CheckinCard {
    pub area: CheckinArea,
    pub headline: String,
    pub value: Option<f64>,
    pub unit: String,
    pub previous: Option<f64>,
    pub tone: Tone,
    /// Daily values for the week, Monday first.
    pub series: Vec<Option<f64>>,
}

/// The week's sleep timing: what happened, with one focus. Descriptive only.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TimingReview {
    /// Recorded nights of the week, oldest first.
    pub nights: Vec<NightWindow>,
    /// Earliest and latest bedtime, HH:MM.
    pub bed_range: Option<[String; 2]>,
    pub wake_range: Option<[String; 2]>,
    /// Latest minus earliest wake time, minutes.
    pub wake_spread_min: Option<f64>,
    pub previous_wake_spread_min: Option<f64>,
    /// Nights Garmin rated against its optimal window.
    pub aligned: u32,
    pub ahead: u32,
    pub behind: u32,
    pub focus: String,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CheckinView {
    /// Monday, YYYY-MM-DD.
    pub week_start: String,
    pub cards: Vec<CheckinCard>,
    pub timing: Option<TimingReview>,
    pub vo2max_change: Option<f64>,
    pub fitness_age_change: Option<f64>,
    pub win: Option<String>,
    pub focus: Option<String>,
    /// The plan week that ended most recently; None until targets are set.
    pub plan: Option<PlanCheckin>,
    /// Intensity minutes for that plan week.
    pub intensity: Option<IntensityCard>,
}

/// Notification switches (Settings screen). Defaults follow the spec:
/// training-status and bedtime notifications start off.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Settings {
    pub notify_morning_readiness: bool,
    pub notify_weekly_checkin: bool,
    /// VO2max and Fitness Age changes.
    pub notify_metric_changes: bool,
    pub notify_vitals: bool,
    pub notify_training_status: bool,
    pub notify_bedtime: bool,
    #[serde(default)]
    pub notify_status: bool,
    #[serde(default)]
    pub status_offer_seen: bool,
    /// Minutes between the wind-down reminder and the bedtime window.
    #[serde(default = "default_bedtime_lead")]
    pub bedtime_lead_min: u32,
    /// Time to fall asleep, 0–60 in 5-minute steps: in bed this long before
    /// the asleep-by time.
    #[serde(default = "default_fall_asleep")]
    pub fall_asleep_min: u32,
    /// Let Today's leading card follow the time of day; off for shift work.
    #[serde(default = "default_adaptive_today")]
    pub adaptive_today: bool,
    /// Local consent for read-only lifestyle log downloads; never used in notifications.
    #[serde(default)]
    pub journal_enabled: bool,
    /// Offer to plan the week at its start; off hides the prompt everywhere.
    #[serde(default = "default_plan_prompt")]
    pub plan_prompt: bool,
    /// Strength's body-map figure; Profile follows Garmin's profile gender.
    #[serde(default)]
    pub body_figure: FigureChoice,
    /// Draw streets under workout routes, downloading each area's map once.
    #[serde(default = "default_street_maps")]
    pub street_maps: bool,
}

/// How a Garmin write ended (Rules: Garmin writes).
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum WriteOutcome {
    /// Garmin accepted it and the read-back matches.
    Applied,
    /// Garmin accepted it but the read-back shows other values.
    Differs,
    /// Garmin accepted it; the read-back failed, so it is unconfirmed.
    Unconfirmed,
    /// Not sent: Garmin's value changed since the person saw it.
    Conflict,
    /// Garmin refused it, with its message when it gave one.
    Rejected,
    /// Not saved: Garmin couldn't be reached, failed or was rate limiting.
    Failed,
    /// Not sent: Garmin needs a new sign-in.
    NeedsSignIn,
}

/// One field of a Garmin write, as display text: the person's new value,
/// or for a read-back difference what was sent and what Garmin returned.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ChangedField {
    /// Stable key, such as `alarm:7.alarmTime`.
    pub field: String,
    pub label: String,
    /// Absent for an added item.
    pub before: Option<String>,
    /// Absent for a removed item.
    pub after: Option<String>,
}

/// One entry of Settings' recent Garmin changes, newest first.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct GarminChange {
    /// When the person asked for it, RFC 3339 UTC.
    pub at: String,
    /// What was written, such as "Watch alarms".
    pub resource: String,
    pub fields: Vec<ChangedField>,
    pub outcome: WriteOutcome,
    /// Garmin's rejection message or the failure, when known.
    pub message: Option<String>,
    /// Fields that read back differently from what was sent.
    pub mismatches: Vec<ChangedField>,
    /// Garmin has it, but the watch hasn't downloaded since.
    pub waiting_for_watch: bool,
}

fn default_bedtime_lead() -> u32 {
    30
}
fn default_fall_asleep() -> u32 {
    15
}
fn default_adaptive_today() -> bool {
    true
}
fn default_plan_prompt() -> bool {
    true
}
fn default_street_maps() -> bool {
    true
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            notify_morning_readiness: true,
            notify_weekly_checkin: true,
            notify_metric_changes: true,
            notify_vitals: true,
            notify_training_status: false,
            notify_bedtime: false,
            notify_status: false,
            status_offer_seen: false,
            bedtime_lead_min: default_bedtime_lead(),
            fall_asleep_min: default_fall_asleep(),
            adaptive_today: default_adaptive_today(),
            journal_enabled: false,
            plan_prompt: default_plan_prompt(),
            body_figure: FigureChoice::Profile,
            street_maps: default_street_maps(),
        }
    }
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AppStatus {
    pub signed_in: bool,
    pub syncing: bool,
    /// Local time of the last successful sync, ISO 8601.
    pub last_sync: Option<String>,
    /// Days of history stored so far.
    pub history_days: u32,
    /// The oldest day with Garmin data on the phone, YYYY-MM-DD.
    pub history_from: Option<String>,
    /// False while older history is still downloading.
    pub history_complete: bool,
    pub last_error: Option<String>,
}

/// A metric with a daily value that trend charts can show and overlay.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum TrendMetric {
    /// Garmin's morning Training Readiness score.
    Readiness,
    SleepScore,
    /// Minutes asleep.
    SleepTime,
    /// Nightly average HRV, ms.
    Hrv,
    RestingHr,
    /// Garmin's average heart rate during sleep (Health Status "HR").
    SleepHr,
    Respiration,
    Spo2,
    SkinTemp,
    /// The day's highest Body Battery.
    BodyBattery,
    /// Garmin's average stress for the day.
    Stress,
    Vo2max,
    FitnessAge,
    /// Garmin's acute (7-day) training load at the end of the day.
    AcuteLoad,
}

#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum TrendRange {
    Days7,
    Days14,
    Days30,
    Days90,
    Year,
    All,
    Custom,
}

/// How many days one point covers. Long ranges average weeks or months.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum TrendBucket {
    Day,
    Week,
    Month,
}

/// What a trend chart asks for: one metric over a range, optionally with a
/// second metric overlaid or the period before it.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TrendQuery {
    pub metric: TrendMetric,
    pub range: TrendRange,
    /// The last day, YYYY-MM-DD; the current day when absent.
    #[serde(default)]
    pub end: Option<String>,
    #[serde(default)]
    pub overlay: Option<TrendMetric>,
    #[serde(default)]
    pub previous: bool,
    /// Inclusive first day for a custom training block.
    #[serde(default)]
    pub start: Option<String>,
    /// First day of an equally long comparison block; absent means the previous period.
    #[serde(default)]
    pub comparison_start: Option<String>,
}

/// One day, week or month. The value is the mean of the recorded days in it;
/// None when no day in it was recorded.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TrendPoint {
    /// Garmin's chronological age on this estimate's day, never inferred.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub chronological_age: Option<f64>,
    /// First and last day covered, YYYY-MM-DD.
    pub start: String,
    pub end: String,
    pub value: Option<f64>,
    /// Recorded days behind the value.
    pub days: u32,
    /// Garmin's average on a stress day it rated only in part (too little
    /// wear, or awake only), on daily points. Never in `value`, `days` or an
    /// average.
    #[serde(default)]
    pub partial: Option<f64>,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TrendSeries {
    pub metric: TrendMetric,
    pub name: String,
    pub unit: String,
    /// Oldest first.
    pub points: Vec<TrendPoint>,
    /// Mean of every recorded day in the period.
    pub average: Option<f64>,
    pub days: u32,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TrendView {
    pub range: TrendRange,
    pub bucket: TrendBucket,
    /// First and last day of the period, YYYY-MM-DD; for All, from the
    /// first recorded day.
    pub from: String,
    pub to: String,
    pub series: TrendSeries,
    /// A second metric over the same points.
    pub overlay: Option<TrendSeries>,
    /// The same-length period just before, point for point.
    pub previous: Option<TrendSeries>,
    /// This period's average minus the previous one's.
    pub change: Option<f64>,
    /// Supported metrics for the picker; unavailable periods stay empty.
    pub available: Vec<TrendMetric>,
    /// The newest reading's age on today's chart (2-day threshold); none for
    /// a past end or for sparse estimates (VO₂ max, Fitness Age).
    #[serde(default)]
    pub freshness: Option<ReadingFreshness>,
}

/// The four weekly plan targets.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum PlanTargetKind {
    Strength,
    Intensity,
    Zone2,
    AsleepOnTime,
}

/// Behind pace is amber, done is green, on track is ink.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum PlanState {
    OnTrack,
    Behind,
    Done,
}

/// The starting point that set the targets; any change makes them Custom.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum PlanPreset {
    Strength,
    Endurance,
    Sleep,
    Custom,
}

/// Weekly plan targets: a local setting, never sent anywhere. A target of
/// zero is off.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct PlanTargets {
    pub preset: PlanPreset,
    pub strength_sessions: u32,
    /// Weekly intensity minutes; None follows Garmin's weekly goal.
    pub intensity_min: Option<u32>,
    pub zone2_min: u32,
    pub asleep_nights: u32,
    /// Weekdays with nothing due, 0 = Monday … 6 = Sunday.
    pub rest_days: Vec<u8>,
}

impl PlanTargets {
    pub fn preset(preset: PlanPreset) -> Self {
        let (strength_sessions, zone2_min, asleep_nights) = match preset {
            PlanPreset::Strength | PlanPreset::Custom => (3, 90, 5),
            PlanPreset::Endurance => (1, 150, 5),
            PlanPreset::Sleep => (2, 60, 6),
        };
        Self {
            preset,
            strength_sessions,
            intensity_min: None,
            zone2_min,
            asleep_nights,
            rest_days: vec![],
        }
    }
}

/// One target's progress for a week.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct PlanProgress {
    pub kind: PlanTargetKind,
    /// Sessions, minutes or nights so far.
    pub value: f64,
    pub target: f64,
    /// value / target, capped at 1: partial credit.
    pub completion: f64,
    /// Share of the week's training days (nights, for sleep) that have passed.
    pub pace: f64,
    pub state: PlanState,
    /// A few words under the bar: the days that counted, or what is left.
    pub note: String,
    /// Why this target can't be measured; it is then left out of every count.
    pub unavailable: Option<String>,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct PlanWeek {
    /// First and last day, YYYY-MM-DD, starting on Garmin's first day of the week.
    pub start: String,
    pub end: String,
    pub paused: bool,
    /// Whole days after today in this week; 0 for past weeks and on the last day.
    pub days_left: u32,
    /// Targets that are on (above zero), in a fixed order.
    pub items: Vec<PlanProgress>,
    /// Measurable targets.
    pub total: u32,
    /// Measurable targets not behind pace.
    pub on_track: u32,
    pub reached: u32,
    /// Honest summary of reached/partial/none/unknown/paused states.
    pub headline: String,
    /// "3 days left · Zone 2 is behind".
    pub detail: String,
    /// Average completion of the measurable targets; None when there are
    /// none or the week has no data.
    pub completion: Option<f64>,
}

/// A history bar. `partial` means an enabled target lacks evidence, so 100%
/// of known targets must not be displayed as all reached.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct PlanHistoryWeek {
    pub start: String,
    pub completion: Option<f64>,
    pub partial: bool,
    pub paused: bool,
    pub current: bool,
}

/// The weekly plan screen and its setup.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct PlanView {
    /// False until targets are saved once.
    pub configured: bool,
    /// This week's targets (the Strength preset before setup).
    pub targets: PlanTargets,
    /// Garmin's weekly intensity-minutes goal, the default intensity target.
    pub garmin_intensity_goal: Option<f64>,
    /// Garmin's first day of the week, 0 = Monday … 6 = Sunday.
    pub week_starts_on: u8,
    pub week: PlanWeek,
    /// Last week measured against this week's targets.
    pub last_week: PlanWeek,
    /// Eight weeks ending with this one, oldest first.
    pub history: Vec<PlanHistoryWeek>,
    /// Starting points for setup: Strength, Endurance and Sleep.
    pub presets: Vec<PlanTargets>,
}

/// A target change from the setup screen.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct PlanUpdate {
    pub targets: PlanTargets,
    /// Leave this week out of history.
    pub paused: bool,
}

/// The plan card of the weekly check-in: the week that just ended.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct PlanCheckin {
    pub week: PlanWeek,
    /// Each item's value the week before, in the same order.
    pub previous: Vec<Option<f64>>,
    /// "Same targets · rest Mon and Fri".
    pub next: String,
}

/// Garmin's intensity minutes for one week.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct IntensityCard {
    pub start: String,
    pub end: String,
    /// The last day the total covers: the day asked for, or the week's end
    /// when it is Garmin's weekly row.
    pub through: String,
    /// Garmin's weekly total: moderate minutes plus twice the vigorous ones.
    pub total: f64,
    pub moderate: f64,
    pub vigorous: f64,
    /// Garmin's weekly goal.
    pub goal: Option<f64>,
}

// ---------------------------------------------------------------------------
// Local records: goals, your sessions and circumstances. Stored on the phone
// only (docs/spec.md, Rules: Local records), never sent to Garmin.

/// A sport family from Garmin's activity type key. Mixed and unfamiliar types
/// stay `Other`, with Garmin's own key kept beside them.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum Sport {
    Running,
    Cycling,
    Walking,
    Hiking,
    Swimming,
    Strength,
    Other,
}

/// Weekly goal kinds, in ring order from the outside. Sleep Score is a
/// Garmin reading shown against its target, not a ring.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum GoalKind {
    Strength,
    Zone2,
    /// Minutes in heart-rate zones 4 and 5.
    HighIntensity,
    Intensity,
    SleepTiming,
    SleepScore,
}

/// How a sleep-timing night is met.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum SleepTimingKind {
    /// Garmin's recorded sleep start by the chosen time.
    #[default]
    AsleepBy,
    /// A wind-down the person reports from the chosen time.
    WindDown,
    /// Migrated asleep-on-time target: observed against each night's saved
    /// plan, shown neutrally, never a verdict.
    SavedPlan,
}

/// One week's targets. Zero turns a target off.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase", default)]
#[ts(export)]
pub struct GoalTargets {
    pub strength_sessions: u32,
    pub zone2_min: u32,
    /// Minutes in heart-rate zones 4 and 5.
    pub high_intensity_min: u32,
    pub intensity_min: u32,
    /// Follow Garmin's weekly intensity goal instead of `intensity_min`.
    pub intensity_garmin_goal: bool,
    pub sleep_nights: u32,
    /// HH:MM, for asleep-by and wind-down nights.
    pub sleep_time: Option<String>,
    pub sleep_kind: SleepTimingKind,
    /// Average Garmin Sleep Score target for the week.
    pub sleep_score: u32,
    /// Usual rest weekdays, 0 = Monday … 6 = Sunday.
    pub rest_days: Vec<u8>,
}

/// Why a goal version exists.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum GoalSource {
    /// First targets, applying from the start of that week.
    Setup,
    /// Changed during a week, from that day.
    Revision,
    /// An accepted week-review proposal, from the next week.
    Proposal,
    /// Carried over from the earlier weekly plan.
    Migrated,
}

/// Targets in effect from `from` (YYYY-MM-DD) until the next version.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct GoalVersion {
    pub from: String,
    pub targets: GoalTargets,
    pub source: GoalSource,
    /// Local time of the change, ISO 8601 without offset.
    pub saved_at: String,
}

/// How much of each goal a planned session is meant to cover.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase", default)]
#[ts(export)]
pub struct Allocation {
    pub strength_sessions: u32,
    pub zone2_min: u32,
    pub high_intensity_min: u32,
    pub intensity_min: u32,
    pub sleep_nights: u32,
}

#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum IntentionKind {
    #[default]
    Workout,
    WindDown,
}

/// Open until the person lets it go or marks it missed. A passed open
/// session without a recording is "Not recorded", never missed.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum IntentionStatus {
    #[default]
    Open,
    LetGo,
    Missed,
}

#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum IntentionChange {
    Added,
    Moved,
    Edited,
    LetGo,
    Missed,
    Reopened,
    /// A date or sport change removed a link that no longer fits.
    LinkCleared,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct IntentionEvent {
    /// Local time, ISO 8601 without offset.
    pub at: String,
    pub change: IntentionChange,
    pub from: Option<String>,
    pub to: Option<String>,
}

/// A session the person placed on a day. Not proof of completion.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Intention {
    pub id: String,
    /// Local calendar date, YYYY-MM-DD.
    pub date: String,
    /// HH:MM, optional.
    pub time: Option<String>,
    pub sport: Sport,
    /// Garmin's type key when the person chose one of their recorded types.
    pub type_key: Option<String>,
    pub name: Option<String>,
    pub duration_min: Option<u32>,
    #[serde(default)]
    pub allocation: Allocation,
    #[serde(default)]
    pub kind: IntentionKind,
    /// The Garmin scheduled workout this session stands in for, if any.
    pub garmin_id: Option<String>,
    #[serde(default)]
    pub status: IntentionStatus,
    #[serde(default)]
    pub history: Vec<IntentionEvent>,
}

/// A link the person made between a plan item and a recording, or an
/// automatic pair they took apart.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct PlanLink {
    pub plan_id: String,
    pub activity_id: String,
    pub at: String,
}

#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum PauseReason {
    #[default]
    Pause,
    Ill,
}

/// Training expectations paused from `since` through `until` (open until
/// the person resumes). Never ends by itself.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Pause {
    pub id: String,
    pub since: String,
    pub until: Option<String>,
    #[serde(default)]
    pub reason: PauseReason,
    /// A date to review the pause; it prompts a review, never a resume.
    pub review: Option<String>,
}

/// Activities to avoid and a free-text movement note. The note is shown,
/// never interpreted, and never leaves the app except in a user's export.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase", default)]
#[ts(export)]
pub struct Limits {
    pub excluded: Vec<Sport>,
    pub note: String,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RestDay {
    pub date: String,
    pub at: String,
}

/// "Not today" for one suggestion on one local day.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Dismissal {
    pub id: String,
    pub date: String,
}

/// A self-reported wind-down, keyed by the evening it began.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct WindDownReport {
    pub date: String,
    pub at: String,
}

/// The session kept for one local day.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TodayChoice {
    pub date: String,
    pub plan_id: String,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct LocalEvent {
    pub id: String,
    pub name: String,
    pub date: String,
    pub sport: Option<Sport>,
}

/// The person's answer to the plan-your-week prompt for one week.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum PromptChoice {
    Planned,
    Dismissed,
}

/// Every local change goes through one command, under the account barrier.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(tag = "action", rename_all = "camelCase", rename_all_fields = "camelCase")]
#[ts(export)]
pub enum LocalAction {
    /// The first targets apply from this week's start; later changes from today.
    SetGoals { targets: GoalTargets },
    AddIntention {
        date: String,
        time: Option<String>,
        sport: Sport,
        type_key: Option<String>,
        name: Option<String>,
        duration_min: Option<u32>,
        allocation: Allocation,
        kind: IntentionKind,
        garmin_id: Option<String>,
    },
    /// Time, name, duration and allocation; keeps its day and links.
    EditIntention {
        id: String,
        time: Option<String>,
        name: Option<String>,
        duration_min: Option<u32>,
        allocation: Allocation,
    },
    /// To today or a later day. Clears its link; no catch-up is added.
    MoveIntention { id: String, date: String },
    SetIntentionStatus { id: String, status: IntentionStatus },
    RemoveIntention { id: String },
    Link { plan_id: String, activity_id: String },
    Unlink { plan_id: String, activity_id: String },
    RestToday,
    CancelRest,
    Pause { reason: PauseReason, review: Option<String> },
    Resume,
    SetLimits { limits: Limits },
    Dismiss { id: String },
    ReportWindDown,
    SetPromptWeek { week: String, choice: PromptChoice },
    AddEvent { name: String, date: String, sport: Option<Sport> },
    RemoveEvent { id: String },
    /// Plan has explained the migrated sleep-timing observation.
    ObservationExplained,
    /// Accept some of a week's proposals (none keeps every goal). Accepted
    /// ones apply from the next week.
    DecideReview { week: String, accepted: Vec<GoalKind> },
    /// Progress showed the new milestones.
    MilestonesSeen,
    /// Keep a session (yours, or `garmin:DATE:NAME`) as today's choice.
    KeepToday { plan_id: String },
    /// Halve this week's training targets from today; next week returns to the
    /// current targets.
    LighterWeek,
    /// Add an exercise to your own checklist.
    AddRoutine { name: String, how: Option<String> },
    RemoveRoutine { id: String },
    /// Self-report an exercise done (or not) today.
    ReportRoutine { id: String, done: bool },
}

// ---------------------------------------------------------------------------
// The plan board: goals, the week's plan items and recordings, the goal
// ledger, rings and backlog (docs/spec.md, Rules: Plan matching, Goal
// ledger, Rings, Backlog).

/// A recorded activity as the plan rules count it.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Recording {
    pub id: String,
    /// Local start date, YYYY-MM-DD.
    pub date: String,
    pub start_local: String,
    pub name: String,
    pub icon: Icon,
    pub sport: Sport,
    /// Garmin's activity type key.
    pub type_key: String,
    pub duration_min: f64,
    /// Whole minutes in Garmin's zone 2; None when Garmin sent no zones.
    pub zone2_min: Option<f64>,
    /// Whole minutes in Garmin's zones 4 and 5; None when unknown.
    pub high_intensity_min: Option<f64>,
    /// Garmin's moderate plus twice vigorous minutes for this activity.
    pub intensity_min: Option<f64>,
}

#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum ItemState {
    /// Matched to a recording (or a reported wind-down).
    Done,
    /// Raring can't tell which recording, if any, it was.
    Review,
    /// Today or later, not yet recorded.
    Planned,
    /// Passed without a matching recording. Never "missed".
    NotRecorded,
    /// Passed, but the activity list for that day is incomplete.
    Awaiting,
    LetGo,
    Missed,
}

#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum MatchKind {
    /// The person linked it.
    Linked,
    /// Same day and sport, each the other's only candidate.
    Automatic,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct PlanItem {
    pub intention: Intention,
    pub state: ItemState,
    /// The recording that completed it.
    pub recording: Option<String>,
    pub matched_by: Option<MatchKind>,
    /// Recordings it might be, when it needs review.
    pub candidates: Vec<String>,
    /// A Garmin Coach session: read-only, from Garmin's adaptive plan.
    #[serde(default)]
    pub coach: Option<CoachSession>,
}

/// A Garmin scheduled workout: read-only, by name and date. Its sport,
/// duration and targets are unverified, so it is not counted or matched.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct GarminSession {
    pub date: String,
    pub name: String,
}

/// One goal's week: what was recorded, what is placed, what is left.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct GoalLedger {
    pub kind: GoalKind,
    /// Sessions, minutes or nights in effect now (the week's end, for a past week).
    pub target: f64,
    /// The week's starting target, when it was changed during the week.
    pub original: Option<f64>,
    pub revised_on: Option<String>,
    /// None while the evidence is incomplete; never shown as zero.
    pub recorded: Option<f64>,
    /// Placed on today or later and not yet recorded.
    pub planned: f64,
    /// Target minus recorded and planned; None when unknown or paused.
    pub remaining: Option<f64>,
    pub reached: bool,
    /// Recordings to review for this goal: each counted once, however many
    /// of its sessions could be it.
    pub review: u32,
    /// Why `recorded` is unknown.
    pub unavailable: Option<String>,
    /// The migrated sleep-timing observation: nights are shown, without a verdict.
    pub observation: bool,
}

#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum DayMark {
    Rest,
    Ill,
    Paused,
    /// The activity list for this passed day is incomplete.
    Unknown,
}

/// A ring draws only recorded work: `fill` of the ring, 0 draws nothing.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Ring {
    pub kind: GoalKind,
    pub recorded: f64,
    pub planned: f64,
    pub fill: f64,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DayCell {
    pub date: String,
    pub today: bool,
    pub future: bool,
    pub mark: Option<DayMark>,
    /// One per active ring goal, in ring order.
    pub rings: Vec<Ring>,
    /// Everything planned that day is done.
    pub done: bool,
    pub not_recorded: bool,
    /// Sessions still planned that day (today or later).
    pub planned: u32,
    pub recordings: Vec<String>,
    pub items: Vec<String>,
    pub garmin: u32,
    /// Garmin's Sleep Score for the night ending this morning.
    pub sleep_score: Option<f64>,
}

/// Average Garmin Sleep Score this week against the target: a reading.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SleepScoreGoal {
    pub target: f64,
    pub average: Option<f64>,
    pub nights: u32,
}

/// What is left of a goal once recordings and placed sessions count.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct BacklogItem {
    pub kind: GoalKind,
    /// One session, or minutes or nights.
    pub amount: f64,
    /// The cardio sport recorded most in the last eight weeks, as a default.
    pub sport: Option<Sport>,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct WeekPause {
    pub since: String,
    pub until: Option<String>,
    pub reason: PauseReason,
    /// The pause covers every day of the week.
    pub whole_week: bool,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct PlanBoard {
    /// The phone's local calendar date.
    pub today: String,
    /// The plan week shown, first and last day.
    pub start: String,
    pub end: String,
    /// Targets in effect today (the week's end, for a past week).
    pub goals: Option<GoalVersion>,
    /// Every goal version, oldest first.
    pub versions: Vec<GoalVersion>,
    pub garmin_intensity_goal: Option<f64>,
    pub ledger: Vec<GoalLedger>,
    pub week_rings: Vec<Ring>,
    pub sleep_score: Option<SleepScoreGoal>,
    pub days: Vec<DayCell>,
    pub items: Vec<PlanItem>,
    pub recordings: Vec<Recording>,
    pub garmin: Vec<GarminSession>,
    pub garmin_state: PlannedState,
    pub backlog: Vec<BacklogItem>,
    /// Paused or ill today: nothing is left to catch up.
    pub backlog_paused: bool,
    pub pause: Option<WeekPause>,
    /// The activity list covers the week through today.
    pub coverage_complete: bool,
    /// The migrated sleep-timing observation hasn't been explained yet.
    pub explain_observation: bool,
    /// Rows written by a newer Raring are shown but can't be changed here.
    pub read_only: bool,
    pub prompt: Option<PlanPrompt>,
    /// Garmin's next race and your local events, soonest first.
    pub events: Vec<EventLine>,
    /// The soonest event with a sport that this week's sessions serve.
    #[serde(default)]
    pub event_week: Option<EventWeek>,
    /// This week of an adaptive Garmin Coach plan, when one is active.
    #[serde(default)]
    pub coach: Option<CoachWeek>,
    /// This week's vigorous minutes against Fitness Age's weekly target, read-only.
    #[serde(default)]
    pub fitness: Option<PlanFitness>,
}

/// Garmin's vigorous minutes this plan week against the weekly target of
/// Garmin's Fitness Age driver. A Garmin reading, not one of your goals.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct PlanFitness {
    /// Unknown while a day this week lacks Garmin's daily figure.
    pub vigorous: Option<f64>,
    pub target: f64,
}

/// This week's work toward an event: your sessions of its sport and what
/// was recorded. Each still counts toward its weekly goals as usual.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct EventWeek {
    pub event: EventLine,
    pub sport: Sport,
    /// Your sessions this week of the event's sport, not let go.
    pub items: Vec<String>,
    /// Recorded minutes of that sport this week, through today.
    pub recorded_min: f64,
}

// ---------------------------------------------------------------------------
// Today's leading message and the circumstances that gate it (docs/spec.md,
// Rules: Today's lead, Adapt).

#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum LeadKind {
    /// The displayed night hasn't synced; nothing current to suggest.
    Waiting,
    /// Follow my day is off or timing isn't credible: the recorded day.
    Fixed,
    /// Evening, wind-down or late night: tonight's sleep leads.
    Sleep,
    Ill,
    Paused,
    Rest,
    /// A recording today leads; the day can finish.
    Done,
    /// Garmin's scheduled workout today (read-only).
    Garmin,
    /// Your session today.
    Session,
    /// A session or schedule whose movements your limits ask you to review.
    Review,
    /// No readiness: last night's sleep leads.
    Summary,
    /// Your activity limits apply; no training suggestion.
    Limits,
    /// Training ideas dismissed for today.
    Quiet,
    /// Garmin's guidance allows hard effort: training is an option.
    Suggest,
    /// Garmin suggests an easier or steady day.
    Easy,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct LeadAction {
    pub label: String,
    /// A canonical route to open.
    pub route: Option<String>,
    /// Open Adapt instead of a page.
    pub adapt: bool,
}

/// One gate the lead passed, in order: the actual reason, for "Why this?".
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct LeadReason {
    pub gate: String,
    pub result: String,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TodayLead {
    /// Stable for the moment it describes; dismissals key on it.
    pub id: String,
    pub kind: LeadKind,
    pub headline: String,
    pub line: Option<String>,
    pub action: Option<LeadAction>,
    /// "Not today" may hide it for the local day.
    pub dismissable: bool,
    /// The person kept this session for today.
    #[serde(default)]
    pub kept: bool,
    /// Garmin's scheduled workout it concerns, `garmin:DATE:NAME`.
    #[serde(default)]
    pub garmin: Option<String>,
    /// The recording it recognises.
    pub activity: Option<String>,
    /// Your session it concerns.
    pub intention: Option<String>,
    pub reasons: Vec<LeadReason>,
    /// Today's session, described below the day's headline (Rules: Today's
    /// session). Keep and Choose another act on it.
    #[serde(default)]
    pub session: Option<TodaySession>,
}

#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum SessionSource {
    /// Garmin Coach's adaptive plan.
    Coach,
    /// Garmin's calendar (a scheduled workout).
    Garmin,
    /// Your plan.
    Own,
}

/// What today's session is, in plain words from verified fields only.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TodaySession {
    pub name: String,
    pub source: SessionSource,
    /// "Garmin Coach · week 1 of 4", "Garmin calendar" or "Your plan".
    pub source_label: String,
    /// Planned time, HH:MM.
    pub time: Option<String>,
    /// Kind, equipment, duration and target: "Strength · gym · about 60 min".
    pub what: Option<String>,
    /// Garmin's training effect in plain words.
    pub effect: Option<String>,
    /// Garmin's own label for that effect.
    pub garmin_effect: Option<String>,
    /// Garmin Coach's own reason for the session, rephrased.
    pub note: Option<String>,
    /// Against today's suggested effort; none when it fits or is unknown.
    pub fit: Option<String>,
}

/// Today's circumstances, for the Adapt control on every root page.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AdaptState {
    /// Rest today, or yesterday's rest carried through the night.
    pub rest: bool,
    /// The pause covering today.
    pub pause: Option<Pause>,
    pub limits: Limits,
    /// "Adapt today", "Rest day", "Ill", "Paused" or "Limits on".
    pub label: String,
    /// A pause's review date has come.
    pub review_due: bool,
    /// Exercise prompts are off today: rest, illness or a pause.
    pub blocked: bool,
    /// Your own checklist (for example from a clinician), with today's
    /// self-reports.
    #[serde(default)]
    pub routines: Vec<RoutineToday>,
}

/// An exercise on the checklist the person supplied. Raring adds no
/// instructions, load or progression.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RoutineItem {
    pub id: String,
    pub name: String,
    pub how: Option<String>,
}

/// A self-reported completion for one local day.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RoutineReport {
    pub id: String,
    pub date: String,
    pub at: String,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RoutineToday {
    pub item: RoutineItem,
    /// Self-reported done today.
    pub done: bool,
}

// ---------------------------------------------------------------------------
// The week review: what was met, why weeks don't count, and next week's goals
// proposed from history (docs/spec.md, Rules: Week review).

#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum OutcomeState {
    Met,
    Partly,
    None,
    /// The week had incomplete coverage or more than one paused or ill day.
    NotCounted,
    /// The migrated sleep-timing observation: shown without a verdict.
    Observed,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct GoalOutcome {
    pub kind: GoalKind,
    pub target: f64,
    pub recorded: Option<f64>,
    pub state: OutcomeState,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct GoalProposal {
    pub kind: GoalKind,
    pub current: f64,
    pub proposed: f64,
    pub reason: String,
}

/// A decision on a week's proposals; nothing changes without one.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ReviewDecision {
    pub week: String,
    pub at: String,
    pub proposals: Vec<GoalProposal>,
    pub accepted: Vec<GoalKind>,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct WeekReview {
    pub start: String,
    pub end: String,
    /// Counts toward proposals: complete coverage, at most one paused or ill day.
    pub counted: bool,
    /// Why the week doesn't count, or what shaped it (rest days).
    pub reasons: Vec<String>,
    pub ill_days: u32,
    pub paused_days: u32,
    pub rest_days: u32,
    pub not_recorded: u32,
    pub goals: Vec<GoalOutcome>,
    /// For the week after this one; empty without goals.
    pub proposals: Vec<GoalProposal>,
    pub next_start: String,
    /// When an accepted change would take effect: next week's start, or today
    /// once that week has begun.
    pub applies_from: String,
    /// Only the latest completed week's review, while undecided, can change goals.
    pub decidable: bool,
    pub decision: Option<ReviewDecision>,
}

/// One past week in the review list.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ReviewRow {
    pub start: String,
    pub counted: bool,
    pub met: u32,
    pub goals: u32,
    pub reasons: Vec<String>,
}

/// The plan-your-week prompt, when it applies (never in wind-down or late night).
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct PlanPrompt {
    /// Last week, to review first, at the start of a week after a week with goals.
    pub review: Option<String>,
}

#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum EventSource {
    Garmin,
    Local,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct EventLine {
    /// The local event's id; None for Garmin's race.
    pub id: Option<String>,
    pub name: String,
    pub date: String,
    pub days_until: i32,
    pub sport: Option<Sport>,
    pub source: EventSource,
    /// Garmin's page was full: later races may exist.
    pub partial: bool,
}

// ---------------------------------------------------------------------------
// Progress for a period (docs/spec.md, Rules: Progress periods to Milestones).

#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum Period {
    Week,
    Month,
    Quarter,
    Year,
}

/// Known totals for a window: activities with a local start date in it.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct PeriodTotals {
    pub activities: u32,
    pub active_min: f64,
    /// Activities without a distance add nothing; they aren't zero-distance.
    pub distance_km: f64,
    /// Garmin's daily moderate plus twice vigorous minutes over the days that have them.
    pub intensity_min: Option<f64>,
    pub intensity_days: u32,
    /// Covered days with at least one activity.
    pub active_days: u32,
    /// Days through today covered by a complete activity list.
    pub covered_days: u32,
    pub days: u32,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SportMinutes {
    pub sport: Sport,
    pub minutes: f64,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct PeriodBucket {
    pub from: String,
    pub to: String,
    pub minutes: Vec<SportMinutes>,
    pub total: f64,
    /// Some day in it is covered; otherwise no claim is made.
    pub known: bool,
    pub future: bool,
}

#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum CalendarState {
    Active,
    Rest,
    Ill,
    Paused,
    None,
    /// Inside a gap in the activity list: never drawn as zero.
    Unknown,
    Future,
    /// Before downloaded history.
    Before,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CalendarDay {
    pub date: String,
    pub state: CalendarState,
    pub activities: u32,
    /// Known workouts on an incomplete day keep their mark with this false.
    pub covered: bool,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct GoalWeek {
    pub start: String,
    pub rings: Vec<Ring>,
    /// The week in progress: never counted yet, never marked.
    pub current: bool,
    /// Why a passed week didn't count: ill or paused (two or more such days,
    /// the larger), otherwise unknown (an incomplete activity list).
    pub mark: Option<DayMark>,
    pub counted: bool,
    pub met: u32,
    pub goals: u32,
    pub reasons: Vec<String>,
}

#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum MilestoneKind {
    /// The heaviest set at a rep count for one exercise.
    Best,
    /// The most reps in one bodyweight set for one exercise.
    Reps,
    /// The longest recorded distance for a sport.
    Longest,
    /// The first counted week with every goal met.
    GoalWeek,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Milestone {
    /// Stable evidence identity; a milestone is awarded once.
    pub id: String,
    pub kind: MilestoneKind,
    pub title: String,
    pub value: String,
    pub date: String,
    pub route: Option<String>,
    /// Not seen yet on Progress.
    pub new: bool,
    /// A Garmin edit moved it; corrected without a new award.
    pub corrected: bool,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SleepNeedSummary {
    /// Nights with both Garmin's recorded sleep and need.
    pub nights: u32,
    /// Of those, nights an hour or more below need. No running total.
    pub below_hour: u32,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ProgressView {
    pub period: Period,
    pub from: String,
    pub to: String,
    /// The last day counted: the period's end, or today.
    pub end: String,
    pub current: bool,
    /// The first day of downloaded activity history.
    pub history_start: Option<String>,
    pub totals: PeriodTotals,
    /// The same elapsed days of the previous period; None when not comparable.
    pub previous: Option<PeriodTotals>,
    pub previous_from: Option<String>,
    pub previous_to: Option<String>,
    /// Some day so far isn't covered: totals are known totals, not complete ones.
    pub partial: bool,
    pub buckets: Vec<PeriodBucket>,
    /// Days of a week or month period.
    pub calendar: Vec<CalendarDay>,
    /// Twelve plan weeks ending with this period's last week, oldest first.
    pub goal_weeks: Vec<GoalWeek>,
    /// Newest first, in available history; at most 12 (the archive has all).
    pub milestones: Vec<Milestone>,
    /// All milestones in available history.
    #[serde(default)]
    pub milestones_total: u32,
    pub sleep: Option<SleepNeedSummary>,
    /// Week and month periods: the plan weeks covering the period, one cell
    /// per day with its goal rings (days outside the period included).
    #[serde(default)]
    pub ring_days: Vec<DayCell>,
    // Local calendar date, independent of the selected historical period.
    pub today: String,
}

/// Date of the reading, never the API fetch date. Future/unknown dates are unknown.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ReadingFreshness {
    pub recorded_on: Option<String>,
    pub age_days: Option<u32>,
    pub stale: bool,
    pub threshold_days: u32,
}

#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum WeightDirection {
    Losing,
    Gaining,
    Steady,
}

/// A dated plotted point; a scale gap never becomes an invented weigh-in.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct WeightPoint {
    pub date: String,
    pub scale: Option<f64>,
    pub trend: Option<f64>,
    pub break_scale: bool,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct WeightChange {
    pub days: u32,
    pub change: Option<f64>,
    pub per_week: Option<f64>,
    pub direction: Option<WeightDirection>,
    pub spark: Vec<Option<f64>>,
}

/// Garmin scale measurements and explicitly labelled Raring trend arithmetic.
/// All values use the account's units; no numbers are exposed before units sync.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct WeightView {
    pub range: TrendRange,
    pub unit: Option<WeightUnit>,
    pub as_of: String,
    pub from: Option<String>,
    pub to: Option<String>,
    pub freshness: ReadingFreshness,
    pub scale: Option<f64>,
    pub trend: Option<f64>,
    pub average: Option<f64>,
    pub change: Option<f64>,
    pub bmi: Option<f64>,
    pub body_fat: Option<f64>,
    pub points: Vec<WeightPoint>,
    pub changes: Vec<WeightChange>,
    pub note: Option<String>,
}

// ---------------------------------------------------------------------------
// Your day: the local calendar day on one midnight-to-midnight axis
// (docs/spec.md, Rules: Your day).

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct StageSpan {
    pub stage: Stage,
    /// Minutes from local midnight of the day, clipped to it.
    pub start_min: f64,
    pub end_min: f64,
}

#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum DayEventKind {
    Activity,
    Nap,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DayEvent {
    pub kind: DayEventKind,
    /// Activity id; none for naps.
    pub id: Option<String>,
    pub name: String,
    pub icon: Icon,
    /// Minutes from local midnight; a missing endpoint stays missing.
    pub start_min: Option<f64>,
    pub end_min: Option<f64>,
    pub duration_min: Option<f64>,
    /// Naps: Garmin's level after the nap minus before it, from the intraday
    /// curve, when both sides were measured.
    pub body_battery_change: Option<f64>,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DayTimeline {
    /// The local calendar day, YYYY-MM-DD.
    pub date: String,
    /// The phone's time now, minutes from midnight.
    pub now_min: f64,
    /// The watch's last upload on this day; nothing is drawn after it.
    pub upload_min: Option<f64>,
    /// The night ending this morning, at its clock times.
    pub sleep: Vec<StageSpan>,
    pub asleep_min: Option<f64>,
    /// Three-minute slots from midnight, blank after the upload or now; gaps
    /// are not zero.
    pub body_battery: Vec<Option<f64>>,
    /// The row's value: the last drawn level, otherwise the calendar day's
    /// daily-stats latest when Garmin sent no intraday levels.
    pub body_battery_now: Option<f64>,
    pub stress: Vec<Option<f64>>,
    pub stress_average: Option<f64>,
    /// Two-minute slots.
    pub heart_rate: Vec<Option<f64>>,
    /// Today isn't 24 elapsed hours (a clock change): Body Battery, stress and
    /// heart rate are left to their details, since their slots aren't
    /// wall-clock time.
    pub clock_change: bool,
    pub resting_hr: Option<f64>,
    pub events: Vec<DayEvent>,
}

// ---------------------------------------------------------------------------
// Profile: what Raring holds from Garmin, read from local data only.

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ProfileView {
    /// Garmin's age, sent with Fitness Age.
    pub age: Option<f64>,
    /// Garmin's measurement setting: `metric`, `statute_us` or `statute_uk`.
    pub units: Option<String>,
    /// The latest weigh-in in the display unit, with its date and age.
    pub weight: Option<f64>,
    pub weight_unit: Option<WeightUnit>,
    pub weight_freshness: ReadingFreshness,
    /// The primary watch's last upload.
    pub watch_last_upload: Option<String>,
    pub gear: Option<GearView>,
    pub limits: Limits,
}

// ---------------------------------------------------------------------------
// Training's week lanes (docs/spec.md, Rules: Week lanes).

#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum LaneKind {
    /// Recorded endurance and walking minutes, when no cardio goal is set.
    Cardio,
    Zone2,
    HighIntensity,
    /// Garmin's daily moderate + 2 × vigorous minutes.
    Intensity,
    Strength,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UsualBand {
    /// Per weekday, Monday first: the 25th, 50th and 75th percentile of the
    /// counted weeks' cumulative values.
    pub low: Vec<f64>,
    pub median: Vec<f64>,
    pub high: Vec<f64>,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct WeekLane {
    pub kind: LaneKind,
    /// The weekly target; none for Cardio.
    pub goal: Option<f64>,
    /// Cumulative per day of the plan week; none after today or once a day's
    /// evidence is unknown.
    pub cumulative: Vec<Option<f64>>,
    /// Needs at least three counted weeks.
    pub usual: Option<UsualBand>,
    /// Counted weeks behind the band.
    pub weeks: u32,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TrainingWeek {
    /// The plan week's first day.
    pub start: String,
    pub today: String,
    pub lanes: Vec<WeekLane>,
    /// Recorded minutes per sport this week through today, most first.
    #[serde(default)]
    pub by_sport: Vec<SportMinutes>,
}

// ---------------------------------------------------------------------------
// One exercise over a range (docs/spec.md, Rules: Lift volume).

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct LiftPoint {
    pub date: String,
    pub activity_id: String,
    /// The heaviest counted set, in the display unit.
    pub max: Option<f64>,
    /// Σ weight × reps over counted sets, in the display unit.
    pub volume: Option<f64>,
    /// Sets of this exercise left out: bodyweight, unweighted, without reps,
    /// or flagged as doubled.
    pub excluded: u32,
    /// Most reps in one counted bodyweight set (Rules: Records and bests).
    pub most_reps: Option<u32>,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct LiftSeries {
    pub key: String,
    pub name: String,
    pub unit: WeightUnit,
    pub range: TrendRange,
    /// The range's first day (the first session for Max) and today: the
    /// chart's date axis.
    pub from: String,
    pub to: String,
    /// Sessions in the range, oldest first.
    pub points: Vec<LiftPoint>,
}

/// Sets per muscle group over a range (docs/spec.md, Rules: Muscle gaps).
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MuscleRange {
    pub range: TrendRange,
    pub from: String,
    pub to: String,
    /// Days covered, from the first stored session for Max.
    pub days: u32,
    pub figure: Figure,
    pub muscles: Vec<MuscleSets>,
    /// Groups with at most `gap_at` sets in the range, fewest first.
    pub gaps: Vec<Muscle>,
    pub gap_at: f64,
}

// ---------------------------------------------------------------------------
// Workout detail beyond heart rate (docs/spec.md, Rules: Workout detail).

#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum StreamKind {
    /// km/h.
    Speed,
    /// Metres.
    Elevation,
    /// Steps per minute (both feet) or, for rides, rpm.
    Cadence,
    /// Centimetres.
    StrideLength,
    /// Milliseconds.
    GroundContactTime,
    /// Centimetres.
    VerticalOscillation,
    /// Percent: vertical oscillation over stride length.
    VerticalRatio,
    /// Paddle strokes per minute.
    StrokeRate,
    /// Metres per paddle stroke.
    StrokeDistance,
    /// Watts.
    Power,
    /// Breaths per minute.
    Respiration,
    /// Garmin's available stamina, percent.
    Stamina,
    /// Garmin's performance condition, points.
    PerformanceCondition,
    /// The watch's own sensor, °C; reads warm on the wrist, not the weather.
    Temperature,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct StreamSeries {
    pub kind: StreamKind,
    pub unit: String,
    /// One per `AlongTheWay.t_s`; a missing sample stays missing.
    pub values: Vec<Option<f64>>,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AlongTheWay {
    /// Seconds from the first sample, at most ~300 points.
    pub t_s: Vec<f64>,
    pub series: Vec<StreamSeries>,
}

/// Garmin's five feel answers (`directWorkoutFeel` 0, 25, 50, 75, 100).
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum Feel {
    VeryWeak,
    Weak,
    Normal,
    Strong,
    VeryStrong,
}

/// The self-evaluation the watch asks for after an activity. Either answer
/// can be skipped.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SelfEvaluation {
    pub feel: Option<Feel>,
    /// Perceived effort, 1–10.
    pub effort: Option<u8>,
}

/// Groups of Garmin's summary fields, in display order.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum FieldGroup {
    Time,
    /// Pace, or speed for rides.
    Pace,
    /// Running form; steps and cadence on foot; cadence on a ride.
    Form,
    Power,
    /// Swim and paddle strokes.
    Strokes,
    Elevation,
    HeartRate,
    Breathing,
    Stress,
    Temperature,
    /// Calories and fluid.
    Energy,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RecordedGroup {
    pub group: FieldGroup,
    pub fields: Vec<Stat>,
}

#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum EffectKind {
    Aerobic,
    Anaerobic,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct EffectLine {
    pub kind: EffectKind,
    /// Garmin's 0–5 value.
    pub value: f64,
    /// Garmin's band for the value.
    pub band: String,
    /// One plain line: what this band means for this kind of effect.
    pub line: String,
    /// Garmin's own label for the activity, when it sent one.
    pub garmin_label: Option<String>,
}

// ---------------------------------------------------------------------------
// Progress by domain (docs/spec.md, Rules: Progress domains).

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DomainWeek {
    /// The plan week's first day.
    pub start: String,
    /// Still in progress.
    pub current: bool,
    /// Two or more ill or paused days, or an incomplete activity list.
    pub mark: Option<DayMark>,
    /// Recorded Zone 2 and Zone 4–5 minutes; None while any activity lacks
    /// zones or the list is incomplete.
    pub zone2_min: Option<f64>,
    pub high_intensity_min: Option<f64>,
    /// Garmin's weekly intensity minutes (the Plan progress rule).
    pub intensity_min: Option<f64>,
    /// Garmin strength activities; None while the list is incomplete.
    pub strength: Option<f64>,
    /// Garmin's latest VO₂ max estimate dated in the week.
    pub vo2max: Option<f64>,
    /// That week's own targets (Goal ledger); None before goals or while a
    /// goal was off. Intensity following Garmin's goal is Garmin's weekly goal.
    pub zone2_goal: Option<f64>,
    pub high_intensity_goal: Option<f64>,
    pub intensity_goal: Option<f64>,
    pub strength_goal: Option<f64>,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DomainMonth {
    /// The month's first day.
    pub start: String,
    /// Recorded minutes; None before history or while a day is uncovered.
    pub minutes: Option<f64>,
    /// Averages of Garmin's daily readings in the month.
    pub resting_hr: Option<f64>,
    pub hrv: Option<f64>,
    /// Garmin's latest estimate dated in the month.
    pub vo2max: Option<f64>,
    pub fitness_age: Option<f64>,
}

/// The plan weeks (Cardio, Strength) and calendar months (Long-term) that
/// Progress's domain sections show.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DomainQuery {
    /// How many plan weeks, ending with the week holding `weeks_end`.
    pub weeks: u32,
    /// A day in the last week shown; this week when absent.
    pub weeks_end: Option<String>,
    /// How many calendar months, ending with the month holding `months_end`.
    pub months: u32,
    /// A day in the last month shown; this month when absent.
    pub months_end: Option<String>,
}

impl Default for DomainQuery {
    /// What Progress first shows: twelve weeks and twelve months ending now.
    fn default() -> Self {
        Self { weeks: 12, weeks_end: None, months: 12, months_end: None }
    }
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ProgressDomains {
    pub today: String,
    /// Targets in effect today.
    pub goals: Option<GoalTargets>,
    /// The chosen plan weeks' first and last day.
    pub weeks_from: String,
    pub weeks_to: String,
    /// The chosen plan weeks, oldest first, from downloaded history.
    pub weeks: Vec<DomainWeek>,
    /// The chosen months' first and last day.
    pub months_from: String,
    pub months_to: String,
    /// The chosen calendar months, oldest first.
    pub months: Vec<DomainMonth>,
}

// ---------------------------------------------------------------------------
// Garmin Coach (docs/spec.md, Rules: Garmin Coach).

#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum CoachKind {
    /// Garmin's `strength_training`.
    Strength,
    /// Garmin's `other`: a cardio session with no sport chosen yet.
    Cardio,
    /// A sport Raring hasn't seen from Garmin Coach: shown, never counted.
    Unknown,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CoachSession {
    /// `coach:DATE:ORDER`; plan links and Keep use it.
    pub id: String,
    pub date: String,
    pub name: String,
    /// None on a rest day.
    pub kind: Option<CoachKind>,
    /// Garmin's estimate.
    pub duration_min: Option<u32>,
    /// Garmin's training-effect label, when it is one Raring knows.
    pub effect: Option<String>,
    pub rest_day: bool,
    /// Garmin's own status, shown as Garmin's; never taken as completion.
    pub garmin_status: Option<String>,
    /// Garmin's `workoutPhrase` code, such as `BASE` or
    /// `STRENGTH_BASE_BUILDER_13_GENERIC_GYM`.
    #[serde(default)]
    pub phrase: Option<String>,
    /// Target heart rate, when Garmin's `workoutDescription` is "<n>bpm".
    #[serde(default)]
    pub target_hr: Option<u32>,
    /// Garmin's plan week (`weekId`), from 1.
    #[serde(default)]
    pub week: Option<u32>,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CoachWeek {
    pub name: String,
    pub state: PlannedState,
    /// This plan week's Coach sessions and rest days, by date.
    pub sessions: Vec<CoachSession>,
    /// Garmin's `durationInWeeks`.
    #[serde(default)]
    pub weeks: Option<u32>,
}

/// One goal's share from a workout (docs/spec.md, Rules: How it counted).
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CountLine {
    pub kind: GoalKind,
    /// This workout's share: 1 strength session, or its zone minutes.
    pub amount: f64,
    /// Strength: this session's place among the week's strength recordings.
    pub position: Option<u32>,
    /// Recorded toward the goal this week so far.
    pub recorded: Option<f64>,
    pub target: f64,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct WorkoutCount {
    /// The plan week's first day.
    pub week: String,
    /// Whether any weekly goal is on that week; without one, nothing counts.
    pub goals: bool,
    pub lines: Vec<CountLine>,
    /// Your session (or Garmin Coach's) this recording completed.
    pub plan_item: Option<String>,
    /// Milestones awarded for this recording, in available history.
    pub bests: Vec<String>,
    /// Its vigorous minutes toward Fitness Age, while they're in Garmin's window.
    #[serde(default)]
    pub fitness: Option<FitnessCount>,
}

/// A recording's vigorous minutes in Fitness Age's six-week window.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct FitnessCount {
    /// Garmin's vigorous minutes for this recording.
    pub minutes: f64,
    /// Garmin's vigorous minutes in its plan week so far, when every day is known.
    pub week: Option<f64>,
    /// The plan week is the current one.
    pub this_week: bool,
    /// Garmin's weekly target for the driver.
    pub target: f64,
    /// The last day these minutes are inside Garmin's six-week average.
    pub until: String,
}

/// What importing a chosen export would change, by record kind. Counts are
/// rows in the file: added, replaced (the file's copy is newer) and kept
/// (this phone already has it or has a newer copy).
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ImportRow {
    pub kind: String,
    pub added: u32,
    pub replaced: u32,
    pub kept: u32,
}

/// Something in the export this import deliberately does not bring over.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ImportSkip {
    pub what: String,
    pub reason: String,
}

/// A validated import, before or after applying it (§ Rules: Import).
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ImportPreview {
    /// When the export was made.
    pub exported_at: Option<String>,
    /// The version of Raring that made the export.
    pub app_version: Option<String>,
    /// The earliest and latest dates the imported settings cover.
    pub from: Option<String>,
    pub to: Option<String>,
    pub rows: Vec<ImportRow>,
    pub skipped: Vec<ImportSkip>,
}
