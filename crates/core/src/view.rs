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

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct BodyBatteryCard {
    pub now: Option<f64>,
    pub charged: Option<f64>,
    pub drained: Option<f64>,
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
    /// Garmin's time in each category today; empty until reported.
    pub time: Vec<StressTime>,
    /// Time Garmin could not rate because you were moving, seconds.
    pub active_s: Option<f64>,
    /// Daily averages for the last 30 days, oldest first.
    pub trend: Vec<Option<f64>>,
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

/// One overnight vital against your normal range.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct VitalReading {
    pub kind: VitalKind,
    pub value: Option<f64>,
    pub unit: String,
    pub tone: Tone,
    /// Your normal range (Garmin's baseline for HRV). None until there is
    /// enough history.
    pub normal: Option<[f64; 2]>,
    /// The bar's scale, wide enough to show value and range.
    pub scale: [f64; 2],
    /// Last 30 nights, oldest first.
    pub trend: Vec<Option<f64>>,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct VitalsCard {
    pub readings: Vec<VitalReading>,
    /// How many readings are outside their normal range.
    pub off_count: u32,
    /// False until 14 nights exist; the UI then shows values without flags.
    pub baseline_ready: bool,
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
    /// Seconds in heart-rate zones 1–5.
    pub zone_seconds: Vec<f64>,
    pub aerobic_te: Option<f64>,
    /// Garmin's label for the aerobic effect: Minor, Maintaining, Improving…
    pub aerobic_label: Option<String>,
    pub anaerobic_te: Option<f64>,
    pub anaerobic_label: Option<String>,
    /// Garmin's primary benefit: Base, Tempo, Threshold, VO2 Max…
    pub benefit: Option<String>,
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
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TodayView {
    /// Local date, YYYY-MM-DD.
    pub date: String,
    pub readiness: Option<Gauge>,
    /// Readiness scores for the last 30 days, oldest first (the Readiness
    /// detail chart).
    pub readiness_trend: Vec<Option<f64>>,
    pub sleep: Option<Gauge>,
    pub sleep_plan: Option<SleepPlan>,
    pub load: Option<Gauge>,
    /// A few words: "Ready to push", "Take it easy".
    pub headline: String,
    pub chips: Vec<Chip>,
    pub factors: Vec<Factor>,
    pub call: Option<Effort>,
    /// For example "Morning Run done · 186 load".
    pub call_note: Option<String>,
    pub body_battery: BodyBatteryCard,
    pub stress: StressCard,
    pub hrv: HrvCard,
    pub vitals: VitalsCard,
    pub activities: Vec<ActivityCard>,
    pub fitness_age: FitnessAgeCard,
    /// True on Mondays once the weekly check-in exists.
    pub checkin_ready: bool,
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
    /// Against your 30-night average; positive means more.
    pub usual_delta_min: Option<f64>,
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
    /// Time to start winding down, HH:MM: the lead before the window starts
    /// (or before the asleep-by time without Garmin's window).
    pub wind_down: String,
    pub wind_down_lead_min: u32,
    /// Local wall-clock times, ISO 8601 without offset, so screens and
    /// reminders agree on the date across midnight.
    pub wind_down_at: String,
    pub bedtime_start_at: Option<String>,
    pub bedtime_at: String,
    /// Median local wake time, HH:MM, when any wake time is recorded.
    pub wake_time: Option<String>,
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
    /// No sleep has synced for the current day, so the plan rests on older
    /// nights; reminders skip it.
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

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SleepView {
    /// The morning the night ended, YYYY-MM-DD.
    pub date: String,
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

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TrainingView {
    /// Acute load on a scale, with Garmin's optimal range as the band.
    pub acute: Option<Gauge>,
    /// Garmin's training status: Productive, Maintaining, Recovery…
    pub status: Option<String>,
    pub status_tone: Tone,
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
    pub workouts: Vec<ActivityCard>,
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
pub struct HealthView {
    /// Local date, YYYY-MM-DD.
    pub date: String,
    pub fitness_age: FitnessAgeCard,
    pub lever: Option<Lever>,
    pub drivers: Vec<Driver>,
    pub vitals: VitalsCard,
    pub stress: StressCard,
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
    /// Minutes between the wind-down reminder and the bedtime window.
    #[serde(default = "default_bedtime_lead")]
    pub bedtime_lead_min: u32,
}

fn default_bedtime_lead() -> u32 {
    30
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
            bedtime_lead_min: default_bedtime_lead(),
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
