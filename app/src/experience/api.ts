// The prototype's stand-in for `lib/api`, swapped in by vite.experience.config.ts.
// Shipped components call these functions unchanged; every value is synthetic.
// Nothing here makes a network request or touches Tauri.
import type {AppStatus} from '../lib/generated/AppStatus';
import type {TodayView} from '../lib/generated/TodayView';
import type {SleepView} from '../lib/generated/SleepView';
import type {TrainingView} from '../lib/generated/TrainingView';
import type {WorkoutView} from '../lib/generated/WorkoutView';
import type {StrengthView} from '../lib/generated/StrengthView';
import type {HealthView} from '../lib/generated/HealthView';
import type {CheckinView} from '../lib/generated/CheckinView';
import type {HeartRateDay} from '../lib/generated/HeartRateDay';
import type {Settings} from '../lib/generated/Settings';
import type {TrendQuery} from '../lib/generated/TrendQuery';
import type {TrendView} from '../lib/generated/TrendView';
import type {TrendMetric} from '../lib/generated/TrendMetric';
import type {TrendSeries} from '../lib/generated/TrendSeries';
import type {PlanView} from '../lib/generated/PlanView';
import type {PlanUpdate} from '../lib/generated/PlanUpdate';
import samplePlan from '../lib/sample/plan.json';
import sampleCheckin from '../lib/sample/checkin.json';
import {period, spans, trendNames, bucketFor} from '../lib/trends';
import {store} from './lib/store.svelte';
import {acuteLoad, healthView, heartRateDay, sleepView, strengthView, todayView, trainingView, workoutView} from './lib/views';
import {activityById, bodyBatteryHigh, fitnessAgeFor, hrvFor, respirationFor, restingHrFor, skinTempFor, sleepHrFor, spo2For, stressAvgFor, vo2maxFor} from './lib/world';
import {addDays} from './lib/dates';

export type {Settings} from '../lib/generated/Settings';
export type SettingsPatch = Partial<Settings>;
export const supportsStatusSurfaces = false;
export const defaultSettings = {notifyMorningReadiness: true, notifyWeeklyCheckin: true, notifyMetricChanges: true, notifyVitals: true, notifyTrainingStatus: false, notifyBedtime: false, notifyStatus: false, statusOfferSeen: false, bedtimeLeadMin: 30, adaptiveToday: true, journalEnabled: false, planPrompt: true} satisfies Settings;
let settings: Settings = {...defaultSettings};

const ctx = () => store.ctx;
export const app_status = (): Promise<AppStatus> => Promise.resolve({signedIn: true, syncing: false, lastSync: `${ctx().upload.date}T${ctx().upload.at}:00`, historyDays: 400, historyFrom: ctx().world.historyStart, historyComplete: ctx().world.historyComplete, lastError: null});
export const today = (): Promise<TodayView> => Promise.resolve(todayView(ctx()));
export const sleep = (date?: string): Promise<SleepView> => Promise.resolve(sleepView(ctx(), date));
export const training = (): Promise<TrainingView> => Promise.resolve(trainingView(ctx()));
export const workout = (id: string): Promise<WorkoutView> => { const a = activityById(id); return a ? Promise.resolve(workoutView(a)) : Promise.reject(new Error('No prototype workout for this ID')); };
export const strength = (): Promise<StrengthView | null> => Promise.resolve(strengthView(ctx()));
export const health = (): Promise<HealthView> => Promise.resolve(healthView(ctx()));
export const heart_rate_day = (date: string): Promise<HeartRateDay> => Promise.resolve(heartRateDay(ctx(), date));
export const checkin = (): Promise<CheckinView> => Promise.resolve(structuredClone(sampleCheckin) as unknown as CheckinView);
export const plan = (): Promise<PlanView> => Promise.resolve(structuredClone(samplePlan) as unknown as PlanView);
export const set_plan = (_update: PlanUpdate): Promise<void> => Promise.resolve();
export const sync_now = (): Promise<void> => Promise.resolve();
export const open_garmin_connect = (): Promise<boolean> => Promise.resolve(false);
export const begin_sign_in = (): Promise<void> => Promise.resolve();
export const sign_out = (): Promise<void> => Promise.resolve();
export const export_archive = (): Promise<string> => Promise.resolve('Prototype: no file created');
export const request_notification_permission = (): Promise<boolean> => Promise.resolve(true);
export const get_settings = (): Promise<Settings> => Promise.resolve({...settings});
export const patch_settings = (patch: SettingsPatch): Promise<void> => { settings = {...settings, ...patch}; return Promise.resolve(); };
export const onStatusChanged = (): Promise<() => void> => Promise.resolve(() => {});

// Daily values from the synthetic world; gaps stay null, never zero.
function value(metric: TrendMetric, date: string): number | null {
  const c = ctx();
  if (date < c.world.historyStart || date > c.calendarDate || date > c.upload.date) return null;
  const night = c.nightOf(date);
  switch (metric) {
    case 'readiness': return c.readinessOf(date)?.value ?? null;
    case 'sleepScore': return night?.score ?? null;
    case 'sleepTime': return night?.asleep ?? null;
    case 'hrv': return night ? hrvFor(date) : null;
    case 'restingHr': return restingHrFor(date);
    case 'sleepHr': return night ? sleepHrFor(date) : null;
    case 'respiration': return night ? respirationFor(date) : null;
    case 'spo2': return night ? spo2For(date) : null;
    case 'skinTemp': return night ? skinTempFor(date) : null;
    case 'bodyBattery': return bodyBatteryHigh(date);
    case 'stress': return stressAvgFor(date);
    case 'vo2max': return vo2maxFor(date);
    case 'fitnessAge': return fitnessAgeFor(date);
    case 'acuteLoad': return c.world.activities.length ? acuteLoad(c, date) : null;
  }
}
const UNITS: Record<TrendMetric, string> = {readiness: '', sleepScore: '', sleepTime: 'min', hrv: 'ms', restingHr: 'bpm', sleepHr: 'bpm', respiration: 'br/min', spo2: '%', skinTemp: '°C', bodyBattery: '', stress: '', vo2max: '', acuteLoad: '', fitnessAge: 'years'};
function series(metric: TrendMetric, list: [string, string][]): TrendSeries {
  let total = 0, count = 0;
  const points = list.map(([start, end]) => {
    let sum = 0, days = 0;
    for (let d = start; d <= end; d = addDays(d, 1)) { const v = value(metric, d); if (v != null) { sum += v; days++; } }
    total += sum; count += days;
    return {start, end, value: days ? sum / days : null, days};
  });
  return {metric, name: trendNames[metric], unit: UNITS[metric], points, average: count ? total / count : null, days: count};
}
export const trend = (query: TrendQuery): Promise<TrendView> => Promise.resolve().then(() => {
  const today = ctx().calendarDate, first = ctx().world.historyStart;
  const validDate = (d: string) => /^\d{4}-\d{2}-\d{2}$/.test(d) && Number.isFinite(Date.parse(d + 'T12:00:00Z')) && new Date(d + 'T12:00:00Z').toISOString().slice(0,10) === d;
  for (const d of [query.start,query.end,query.comparisonStart]) {
    if (d == null) continue;
    if (!validDate(d) || d < '1990-01-01' || d > today) throw Error(`Choose a valid date from 1990-01-01 through ${today}`);
  }
  const end = query.end ?? today;
  if (query.range === 'custom' && (!query.start || query.start > end)) throw Error('Choose a block start on or before its end');
  const length = Math.round((Date.parse(end) - Date.parse(query.start ?? end)) / 86400000) + 1;
  const {from, bucket, shift} = query.range === 'custom' ? {from: query.start ?? end, bucket: bucketFor(length), shift: length} : period(query.range, end, first);
  if (query.previous && query.comparisonStart && shift && addDays(query.comparisonStart,shift-1) > today) throw Error('The comparison block runs past today');
  const list = spans(from, end, bucket);
  const now = series(query.metric, list);
  const offset = query.comparisonStart ? Math.round((Date.parse(from) - Date.parse(query.comparisonStart)) / 86400000) : (shift ?? 0);
  const back = (date: string) => addDays(date, -offset);
  const previous = query.previous && shift ? series(query.metric, list.map(([s, e]) => [back(s), back(e)])) : null;
  const overlay = query.overlay && query.overlay !== query.metric ? series(query.overlay, list) : null;
  return {range: query.range, bucket, from, to: end, series: now, overlay, previous, change: previous?.average != null && now.average != null ? now.average - previous.average : null,
    available: Object.keys(UNITS) as TrendMetric[], freshness: null};
});
