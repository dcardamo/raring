// Builds the real view-contract types from the synthetic world, so shipped
// components render prototype data unchanged. Templates come from the shipped
// sample fixtures; values and dates come from the prototype's world.
import type {ActivityCard} from '../../lib/generated/ActivityCard';
import type {WorkoutView} from '../../lib/generated/WorkoutView';
import type {SleepView} from '../../lib/generated/SleepView';
import type {TodayView} from '../../lib/generated/TodayView';
import type {HealthView} from '../../lib/generated/HealthView';
import type {TrainingView} from '../../lib/generated/TrainingView';
import type {StrengthView} from '../../lib/generated/StrengthView';
import type {StrengthSession} from '../../lib/generated/StrengthSession';
import type {StressCard} from '../../lib/generated/StressCard';
import type {HeartRateDay} from '../../lib/generated/HeartRateDay';
import type {Muscle} from '../../lib/generated/Muscle';
import type {MuscleSets} from '../../lib/generated/MuscleSets';
import type {Effort} from '../../lib/generated/Effort';
import type {SleepPlan} from '../../lib/generated/SleepPlan';
import type {Tone} from '../../lib/generated/Tone';
import sampleToday from '../../lib/sample/today.json';
import sampleHealth from '../../lib/sample/health.json';
import sampleTraining from '../../lib/sample/training.json';
import sampleSleep from '../../lib/sample/sleep.json';
import {addDays, between, daysBetween, dur, hm, range, toMin, weekStart} from './dates';
import {effortFor, type Ctx} from './rules';
import {SPORTS} from './sports';
import {
  ACTIVITIES, CHRONOLOGICAL_AGE, EXERCISES, GEAR, JOURNAL, bodyBatteryFor, dayTimelines, fitnessAgeFor, hrTrace, hrvFor, nightFor, rand, readinessFor,
  respirationFor, restingHrFor, routeFor, setsFor, skinTempFor, sleepHrFor, splitsFor, spo2For, stagesFor, stressAvgFor, vo2maxFor, weightFor, type Activity,
} from './world';

const clone = <T>(v: T): T => structuredClone(v);
const tone = (v: number, good: number, fair: number): Tone => v >= good ? 'good' : v >= fair ? 'fair' : 'poor';
export const MUSCLES: Muscle[] = ['chest', 'back', 'shoulders', 'arms', 'core', 'hips', 'quads', 'hamstrings', 'glutes', 'calves'];
const aerobicLabel = (te: number) => te < 1 ? 'No benefit' : te < 2 ? 'Minor' : te < 3 ? 'Maintaining' : te < 4 ? 'Improving' : te < 5 ? 'Highly improving' : 'Overreaching';

export function activityCard(a: Activity): ActivityCard {
  return {
    id: a.id, icon: SPORTS[a.sport].icon, name: a.name, startLocal: `${a.date}T${a.start}:00`, durationS: a.min * 60, distanceKm: a.km ?? null,
    load: a.load, zoneSeconds: a.zones, aerobicTe: a.aerobic, aerobicLabel: aerobicLabel(a.aerobic), anaerobicTe: a.anaerobic, anaerobicLabel: aerobicLabel(a.anaerobic),
    benefit: a.benefit,
  };
}

export function strengthSession(a: Activity): StrengthSession | null {
  if (a.sport !== 'strength') return null;
  const logs = setsFor(a);
  if (!logs) return {unit: 'kg', figure: 'female', sets: 1, reps: null, unknownRepSets: 1, restS: 0, records: 0, setsRecorded: false, zeroRepSets: 0, missingRepSets: 1, doubledSets: 0, unnamedSets: 0, muscles: MUSCLES.map(muscle => ({muscle, sets: 0})), exercises: []};
  const previous = (name: string) => {
    const earlier = ACTIVITIES.filter(x => x.sport === 'strength' && (x.date + x.start) < (a.date + a.start)).reverse();
    for (const x of earlier) { const log = setsFor(x)?.find(l => l.exercise === name); if (log) return {date: x.date, sets: log.sets}; }
    return null;
  };
  const counts: Record<string, number> = {};
  const exercises = logs.map(l => {
    for (const m of l.muscles) counts[m] = (counts[m] ?? 0) + l.sets.length;
    const prev = l.key ? previous(l.exercise) : null;
    const rec = l.sets.find(s => s[2] === 'record');
    return {
      key: l.key, name: l.key ? l.exercise : null, muscles: l.muscles as Muscle[],
      sets: l.sets.map(([reps, weight, mark]) => ({reps, weight, durationS: reps ? reps * 3 + 12 : 40, flags: !l.key ? ['unnamed' as const] : [], record: mark === 'record'})),
      lastDate: prev?.date ?? null, lastSets: prev?.sets.map(([reps, weight]) => ({reps, weight})) ?? [], record: rec ? {weight: rec[1], reps: rec[0]} : null,
    };
  });
  const sets = exercises.reduce((n, e) => n + e.sets.length, 0);
  return {
    unit: 'kg', figure: 'female', sets, reps: exercises.reduce((n, e) => n + e.sets.reduce((t, s) => t + (s.reps ?? 0), 0), 0), unknownRepSets: 0, restS: 600,
    records: exercises.filter(e => e.record).length, setsRecorded: true, zeroRepSets: 0, missingRepSets: 0, doubledSets: 0,
    unnamedSets: exercises.filter(e => !e.key).reduce((n, e) => n + e.sets.length, 0), muscles: MUSCLES.map(muscle => ({muscle, sets: counts[muscle] ?? 0})), exercises,
  };
}

export function workoutView(a: Activity): WorkoutView {
  const speed = a.sport === 'cycling';
  const stats = [
    ...(a.km ? [{key: 'distance', label: 'Distance', value: a.km, unit: 'km'}] : []),
    {key: 'duration', label: 'Time', value: a.min * 60, unit: 's'},
    ...(a.km ? [speed ? {key: 'speed', label: 'Speed', value: Math.round(a.km / (a.min / 60) * 10) / 10, unit: 'km/h'} : {key: 'pace', label: 'Pace', value: Math.round(a.min * 60 / a.km), unit: 's/km'}] : []),
    {key: 'avgHr', label: 'Avg HR', value: a.avgHr, unit: 'bpm'}, {key: 'maxHr', label: 'Max HR', value: a.maxHr, unit: 'bpm'},
    {key: 'calories', label: 'Calories', value: a.calories, unit: 'kcal'},
  ];
  return {
    summary: activityCard(a), recoveryHours: a.load > 40 ? Math.round(a.load / 6) : null, stats, route: routeFor(a), heartRate: hrTrace(a),
    zoneFloors: [118, 136, 150, 163], splits: splitsFor(a), strength: strengthSession(a),
  };
}

// ---------- sleep ----------
export function sleepPlanFor(ctx: Ctx): SleepPlan {
  const t = ctx.tonight;
  const base = clone(sampleSleep.sleepPlan) as SleepPlan;
  return {...base, night: addDays(t.nightOf, 1), bedtime: t.windowEnd, bedtimeStart: t.window ? t.windowStart : null, bedtimeSource: t.window ? 'garmin' : 'wakeHistory',
    windDown: t.windDown, windDownAt: `${t.nightOf}T${t.windDown}:00`, bedtimeStartAt: t.window ? `${t.nightOf}T${t.windowStart}:00` : null, bedtimeAt: `${t.nightOf}T${t.windowEnd}:00`,
    wakeTime: t.wake, needMin: t.need, baselineMin: 470, reason: t.reason, adjustments: [{factor: 'Sleep history', detail: 'recent short sleep'}], tip: null, stale: ctx.stale};
}
export function sleepView(ctx: Ctx, date = ctx.displayedDay): SleepView {
  const n = ctx.nightOf(date);
  const base = clone(sampleSleep) as unknown as SleepView;
  const start = n ? (toMin(n.bed) > 12 * 60 ? `${addDays(date, -1)}T${n.bed}:00` : `${date}T${n.bed}:00`) : null;
  const stages = n ? stagesFor(date, n.asleep) : [];
  const sum = (s: string) => stages.filter(x => x.stage === s).reduce((t, x) => t + x.minutes, 0);
  const nights14 = range(addDays(date, -13), date).map(d => [d, ctx.nightOf(d)] as const).filter(([, x]) => x);
  const hour = (t: string) => { const m = toMin(t); return Math.round(m / 6) / 10; };
  const naps = ctx.napList.map(x => ({startLocal: `${x.start[0]}T${x.start[1]}:00`, endLocal: x.end ? `${x.end[0]}T${x.end[1]}:00` : null, durationMin: x.min, feedback: null}));
  const napsState = ctx.state.profile !== 'sleep' && ctx.state.profile !== 'mixed' ? 'empty' : ctx.state.napCase === 'unavailable' ? 'unavailable' : ctx.state.napCase === 'stale' ? 'stale' : ctx.state.napCase === 'empty' || !naps.length ? 'empty' : 'available';
  return {
    ...base, date, startLocal: start, endLocal: n ? `${date}T${n.wake}:00` : null,
    score: n ? {value: n.score, max: 100, label: n.score >= 80 ? 'Good' : n.score >= 60 ? 'Fair' : 'Poor', tone: tone(n.score, 80, 60), band: null} : null,
    asleepMin: n?.asleep ?? null, needMin: n?.need ?? null, headline: n ? (n.score >= 80 ? 'A good night' : n.score >= 60 ? 'A fair night' : 'A short night') : 'No recorded night',
    stages, totals: n ? (['deep', 'light', 'rem', 'awake'] as const).map(stage => ({stage, minutes: sum(stage), usualDeltaMin: Math.round((rand(date + stage) - .5) * 20)})) : [],
    overnight: n ? [
      {kind: 'hrv', value: hrvFor(date), unit: 'ms', series: Array.from({length: 8}, (_, i) => hrvFor(date) - 4 + Math.round(rand(date + 'hv' + i) * 8))},
      {kind: 'heartRate', value: sleepHrFor(date), unit: 'bpm', series: Array.from({length: 12}, (_, i) => sleepHrFor(date) + Math.round(4 - i / 3 + rand(date + 'sh' + i) * 3))},
      {kind: 'respiration', value: respirationFor(date), unit: 'brpm', series: Array.from({length: 10}, (_, i) => Math.round((respirationFor(date) + (rand(date + 'rs' + i) - .5) * .6) * 10) / 10)},
      {kind: 'spo2', value: spo2For(date), unit: '%', series: Array.from({length: 10}, (_, i) => spo2For(date) + Math.round((rand(date + 'o' + i) - .5) * 2))},
    ] : [],
    consistency: nights14.map(([d, x]) => ({date: d, bedHour: hour(x!.bed), wakeHour: hour(x!.wake)})),
    consistencySpreadMin: 22, bedWindow: [22.75, 23.25], wakeWindow: [6.66, 7.16],
    garminAverages: {bedtime: '23:14', wakeTime: '06:58', nights: 7, through: date},
    alignment: {state: 'current', latest: n ? {date, status: toMin(n.bed) < 12 * 60 ? 'Behind' : 'Aligned', tone: toMin(n.bed) < 12 * 60 ? 'fair' : 'good', window: [-75, 375], sleep: [toMin(n.bed) > 720 ? toMin(n.bed) - 1440 : toMin(n.bed), toMin(n.wake)]} : null,
      detail: n && toMin(n.bed) < 12 * 60 ? 'Last night started after Garmin’s optimal window.' : 'Last night lined up with Garmin’s optimal window.',
      history: nights14.slice(-7).map(([d, x]) => ({date: d, status: toMin(x!.bed) < 12 * 60 ? 'Behind' : 'Aligned', tone: toMin(x!.bed) < 12 * 60 ? 'fair' as Tone : 'good' as Tone, window: [-75, 375] as [number, number], sleep: [toMin(x!.bed) > 720 ? toMin(x!.bed) - 1440 : toMin(x!.bed), toMin(x!.wake)] as [number, number]}))},
    sleepPlan: ctx.phase === 'waiting' ? null : sleepPlanFor(ctx), naps, napsState,
  };
}

// ---------- day timelines ----------
const untilFor = (ctx: Ctx, date: string) => date === ctx.upload.date ? toMin(ctx.upload.at) : date > ctx.upload.date ? 0 : 1440;
export function timelines(ctx: Ctx, date = ctx.calendarDate) {
  return dayTimelines(date, ctx.world.activities, ctx.state.profile === 'sleep', untilFor(ctx, date));
}
/**
 * Garmin's intraday Body Battery, shaped like the stored dailyStress
 * bodyBatteryValuesArray: charge while asleep, drain with stress and activity.
 * Today's curve ends at the value of the last upload. Minutes run from 22:00
 * the evening before (−120) to the given end.
 */
export function bodyBatteryDay(ctx: Ctx, date = ctx.calendarDate, end?: number) {
  const night = ctx.nightOf(date);
  const until = Math.min(end ?? 1440, untilFor(ctx, date));
  const t = timelines(ctx, date);
  const bed = night ? (toMin(night.bed) > 720 ? toMin(night.bed) - 1440 : toMin(night.bed)) : null;
  const wake = night ? toMin(night.wake) : null;
  const target = date === ctx.upload.date ? bodyBatteryFor(date) : null;
  const charge = night ? Math.round(night.asleep / 8) : 0;
  let low = Math.round(16 + rand(date + 'bblow') * 14);
  if (target != null) low = Math.max(5, Math.min(low, 100 - charge), Math.min(100, target + 8) - charge);
  const high = Math.min(100, low + charge);
  // Unscaled drain per 3-minute step after waking; scaled so today ends at the upload value.
  const steps: number[] = [];
  for (let m = wake ?? 0; m < Math.min(until, 1440); m += 3) {
    const s = t.stress[Math.floor(m / 3)];
    const act = ctx.world.activities.find(a => a.date === date && m >= toMin(a.start) && m < toMin(a.start) + a.min);
    steps.push(act ? act.load / act.min * .55 : s == null ? .08 : .04 + s / 900);
  }
  const raw = steps.reduce((a, b) => a + b, 0);
  const scale = target != null && raw > 0 ? Math.max(0, high - target) / raw : 1;
  const points: [number, number][] = [];
  let v = low + 4, k = 0;
  for (let m = -120; m <= Math.min(until, 1500); m += 3) {
    if (bed != null && wake != null && m >= bed && m < wake) v = low + charge * Math.sqrt((m - bed) / (wake - bed));
    else if (bed != null && m < bed) v = low + 4 * (bed - m) / Math.max(1, bed + 120);
    else if (wake != null && m >= wake && m < 1440) v = Math.max(5, v - (steps[k++] ?? 0) * scale);
    else if (m >= 1440) v = Math.max(5, v - .03);
    points.push([m, Math.round(v)]);
  }
  return {points, charged: high - low, high, low, now: points.at(-1)?.[1] ?? null, until};
}

export function stressCard(ctx: Ctx, date = ctx.calendarDate): StressCard {
  const t = timelines(ctx, date);
  const vals = t.stress.filter((v): v is number => v != null);
  const average = vals.length ? Math.round(vals.reduce((a, b) => a + b, 0) / vals.length) : null;
  const count = (lo: number, hi: number) => vals.filter(v => v >= lo && v < hi).length * 180;
  return {
    average, max: vals.length ? Math.max(...vals) : null, level: average == null ? null : average < 26 ? 'rest' : average < 51 ? 'low' : average < 76 ? 'medium' : 'high',
    qualifier: average == null ? null : average < 26 ? 'Calm' : average < 40 ? 'Balanced' : 'Stressful', tone: average == null ? 'neutral' : average < 40 ? 'good' : 'fair',
    timeline: t.stress, active: t.active, sleep: t.sleep[0] ?? null, time: [['rest', count(0, 26)], ['low', count(26, 51)], ['medium', count(51, 76)], ['high', count(76, 101)]].map(([level, seconds]) => ({level: level as 'rest', seconds: seconds as number})),
    activeS: t.active.reduce((n, [a, b]) => n + (b - a) * 60, 0), trend: range(addDays(date, -29), date).map(d => d < ctx.world.historyStart ? null : stressAvgFor(d)),
  };
}
export function heartRateDay(ctx: Ctx, date = ctx.calendarDate): HeartRateDay {
  const t = timelines(ctx, date);
  const vals = t.hr.filter((v): v is number => v != null);
  return {date, resting: vals.length ? restingHrFor(date) : null, minimum: vals.length ? Math.min(...vals) : null, maximum: vals.length ? Math.max(...vals) : null, timeline: t.hr, durationMin: 1440, sleep: t.sleep, activities: t.active};
}

// ---------- today, health, training, strength ----------
function vitalsFor(ctx: Ctx) {
  const d = ctx.displayedDay;
  const off = ctx.state.vitalsException;
  const series = (f: (d: string) => number) => range(addDays(d, -29), d).map(x => x < ctx.world.historyStart || !nightFor(x) ? null : f(x));
  const read = (kind: string, value: number, unit: string, scale: [number, number], normal: [number, number], f: (d: string) => number) => {
    const out = off && (kind === 'hrv' || kind === 'restingHr');
    const v = out ? (kind === 'hrv' ? 31 : 61) : value;
    return {kind, value: ctx.night ? v : null, unit, scale, normal, trend: series(f), tone: out ? 'fair' : 'good', status: out ? (kind === 'hrv' ? 'BELOW_RANGE' : 'ABOVE_RANGE') : kind === 'skinTemp' ? null : 'IN_RANGE', rangeSource: kind === 'skinTemp' ? 'raring' : 'garmin'};
  };
  const readings = [read('restingHr', sleepHrFor(d), 'bpm', [44, 64], [48, 57], sleepHrFor), read('hrv', hrvFor(d), 'ms', [28, 66], [38, 56], hrvFor),
    read('respiration', respirationFor(d), 'brpm', [12, 16.5], [13.2, 15.2], respirationFor), read('spo2', spo2For(d), '%', [90, 100], [93, 98], spo2For),
    read('skinTemp', skinTempFor(d), '°C', [-1, 1], [-.5, .5], skinTempFor)];
  return {readings, offCount: off ? 2 : 0, baselineReady: true} as TodayView['vitals'];
}
function fitnessAgeCard(ctx: Ctx) {
  const pts = range(addDays(ctx.calendarDate, -364), ctx.calendarDate).filter(d => d.slice(8) === '01' || d.slice(8) === '15').map(d => {
    let v: number | null = null; for (let x = d; x >= addDays(d, -40) && v == null; x = addDays(x, -1)) v = fitnessAgeFor(x); return v;
  });
  return {fitnessAge: 41, chronologicalAge: CHRONOLOGICAL_AGE, trend: pts.slice(-12), change12m: -2};
}
export function todayView(ctx: Ctx): TodayView {
  const base = clone(sampleToday) as unknown as TodayView;
  const r = ctx.readiness, n = ctx.night;
  const acute = acuteLoad(ctx, ctx.calendarDate);
  const factors = r ? [
    {key: 'sleep', name: 'Sleep', percent: n ? n.score : null, tone: tone(n?.score ?? 0, 80, 60), detail: 'Last night’s sleep score.'},
    {key: 'recovery', name: 'Recovery time', percent: Math.max(20, 100 - Math.round(acute / 9)), tone: tone(100 - acute / 9, 70, 45), detail: 'Hours until Garmin expects you to be recovered.'},
    {key: 'hrv', name: 'HRV', percent: Math.min(100, hrvFor(ctx.displayedDay) * 2 - 10), tone: tone(hrvFor(ctx.displayedDay), 42, 36), detail: 'Your 7-day HRV average against your baseline.'},
    {key: 'acuteLoad', name: 'Acute load', percent: Math.max(30, 100 - Math.round(acute / 8)), tone: tone(100 - acute / 8, 60, 40), detail: 'Your last 7 days of load against what you’re used to.'},
    {key: 'sleepHistory', name: 'Sleep history', percent: r.limit === 'Sleep history' ? 46 : 74, tone: r.limit === 'Sleep history' ? 'fair' : 'good', detail: 'Your sleep over the last three nights against your need.'},
    {key: 'stressHistory', name: 'Stress history', percent: null, tone: 'neutral', detail: 'Not rated by Garmin today.'},
  ] as TodayView['factors'] : [];
  return {
    ...base, date: ctx.displayedDay, watchLastUpload: null, readiness: r ? {value: r.value, max: 100, label: r.level, tone: tone(r.value, 75, 50), band: null} : null,
    readinessTrend: range(addDays(ctx.displayedDay, -29), ctx.displayedDay).map(d => ctx.readinessOf(d)?.value ?? null),
    sleep: n ? {value: n.score, max: 100, label: n.score >= 80 ? 'Good' : n.score >= 60 ? 'Fair' : 'Poor', tone: tone(n.score, 80, 60), band: null} : null,
    load: {value: acute, max: 1000, label: acute > 560 ? 'High' : acute < 260 ? 'Low' : 'Optimal', tone: acute > 560 ? 'fair' : 'good', band: [300, 560]},
    headline: ctx.stale || ctx.phase === 'waiting' ? 'Waiting for your watch' : ctx.paused ? 'Training is paused' : ctx.restActive ? 'Rest day' : ctx.hasLimits ? 'Your limits apply' : r ? (r.value >= 75 ? 'Ready when you are' : r.value >= 50 ? 'A steady day' : 'Take it easier') : 'Your day is taking shape',
    factors, call: ctx.paused || ctx.restActive || ctx.hasLimits || ctx.stale || ctx.phase === 'waiting' ? null : effortFor(r) as Effort | null, callNote: null, chips: [],
    readinessWhy: r ? {title: `Why readiness is ${r.level}`, summary: `${r.limit} holds readiness back most.`, feedback: null, source: 'Garmin readiness factors', inputs: factors.map(f => ({key:f.key,name:f.name,value:f.key === 'sleep' ? String(n?.score ?? '—') : f.key === 'hrv' ? `${hrvFor(ctx.displayedDay)} ms` : f.key === 'acuteLoad' ? String(acute) : f.key === 'recovery' ? `${Math.round(acute / 30)} h` : null,rating:f.percent == null ? null : f.tone === 'good' ? 'Good' : f.tone === 'fair' ? 'Fair' : 'Poor',tone:f.tone,detail:f.detail}))} : null,
    bodyBattery: {now: ctx.bodyBattery, charged: n ? bodyBatteryDay(ctx, ctx.displayedDay).charged : null, drained: 22},
    bodyBatteryWhy: ctx.bodyBattery != null ? {title: `Why Body Battery is ${ctx.bodyBattery}`, summary: n ? `Sleep charged it by ${bodyBatteryDay(ctx, ctx.displayedDay).charged}.` : 'No recorded night.', feedback:null,source:'Garmin Body Battery events and feedback',inputs:[{key:'overnight',name:'Overnight',value:n ? `+${bodyBatteryDay(ctx, ctx.displayedDay).charged}` : null,rating:null,tone:'neutral',detail:n ? `${n.bed}–${n.wake}` : 'No recorded night'},{key:'stress',name:'Stress',value:String(stressCard(ctx).average ?? '—'),rating:stressCard(ctx).qualifier,tone:stressCard(ctx).tone,detail:'Average through the day at the last upload'}]} : null,
    stress: stressCard(ctx), heartRateDay: heartRateDay(ctx), hrv: {lastNight: n ? hrvFor(ctx.displayedDay) : null, status: 'Balanced', tone: 'good', baseline: [38, 56], week: range(addDays(ctx.displayedDay, -6), ctx.displayedDay).map(d => nightFor(d) ? hrvFor(d) : null)},
    vitals: vitalsFor(ctx), activities: ctx.actsToday.map(activityCard), fitnessAge: fitnessAgeCard(ctx), checkinReady: false,
    focus: {...base.focus, lastNight: n ? {asleepMin: n.asleep, needMin: n.need, woke: n.wake} : null}, plan: null, sleepPlan: sleepPlanFor(ctx),
    planned: {state: 'empty', session: null, coach: null, coverageStart: ctx.calendarDate, coverageEnd: addDays(ctx.calendarDate, 14)},
    reconcile: null, missingMetricReasons: ctx.world.readinessSupported ? {} : {readiness: 'Your watch doesn’t record Training Readiness'},
  };
}
export function healthView(ctx: Ctx): HealthView {
  const base = clone(sampleHealth) as unknown as HealthView;
  const weights = range(addDays(ctx.calendarDate, -83), ctx.calendarDate).filter(d => d.slice(0, 10) && new Date(d + 'T12:00:00Z').getUTCDay() === 6).map(weightFor);
  const vig = ctx.world.activities.filter(a => between(a.date, addDays(ctx.calendarDate, -27), ctx.calendarDate)).reduce((t, a) => t + Math.round((a.zones[3] + a.zones[4]) / 60), 0) / 4;
  const weekIm = ctx.world.activities.filter(a => between(a.date, ctx.weekFrom, ctx.calendarDate)).reduce((t, a) => t + a.im, 0);
  return {
    ...base, date: ctx.displayedDay, journalEnabled: ctx.state.journal.consent,
    journal: JOURNAL.filter(d => d.date <= ctx.calendarDate && d.date >= addDays(ctx.calendarDate, -6)).map(d => ({date: d.date, state: d.state, entries: d.entries.map(([name, value]) => ({name, value}))})),
    fitnessAge: fitnessAgeCard(ctx), lever: {key: 'vigorousMinutes', name: 'Vigorous minutes', current: Math.round(vig), target: 75, unit: 'min/wk', ageIfMet: null},
    drivers: [
      {key: 'vo2max', name: 'VO₂ max', value: 47, unit: '', target: 45, met: true, tone: 'good'},
      {key: 'restingHr', name: 'Resting HR', value: restingHrFor(ctx.displayedDay), unit: 'bpm', target: 60, met: true, tone: 'good'},
      {key: 'bodyFat', name: 'Body fat', value: 19.8, unit: '%', target: 22, met: true, tone: 'good'},
      {key: 'vigorousMinutes', name: 'Vigorous', value: Math.round(vig), unit: 'min/wk', target: 75, met: vig >= 75, tone: vig >= 75 ? 'good' : 'fair'},
    ],
    vitals: vitalsFor(ctx), stress: stressCard(ctx), heartRateDay: heartRateDay(ctx),
    bodyWeight: {value: 74.2, unit: 'kg', measuredOn: '2026-11-14', bmi: 23.4, bodyFat: 19.8, trend: weights.slice(-12)},
    intensity: {start: ctx.weekFrom, end: ctx.weekTo, total: weekIm, moderate: Math.round(weekIm * .45), vigorous: Math.round(weekIm * .275), goal: 150},
  };
}
/** Garmin acute load stand-in: a 7-day exponentially weighted sum of recorded loads. */
export function acuteLoad(ctx: Ctx, date: string) {
  let sum = 0;
  for (const a of ctx.world.activities) {
    const age = daysBetween(a.date, date);
    if (age >= 0 && age < 28) sum += a.load * Math.exp(-age / 7);
  }
  return Math.round(sum * 1.05);
}
export function trainingView(ctx: Ctx): TrainingView {
  const base = clone(sampleTraining) as unknown as TrainingView;
  const acute = acuteLoad(ctx, ctx.calendarDate);
  const acts = ctx.world.activities.filter(a => a.date <= ctx.calendarDate);
  const recent = acts.slice(-6).reverse();
  const strength = acts.filter(a => a.sport === 'strength' && a.date >= ctx.weekFrom);
  const sv = strengthView(ctx);
  const running = acts.some(a => a.sport === 'running');
  const vo2 = range(addDays(ctx.calendarDate, -55), ctx.calendarDate).filter((_, i) => i % 7 === 6).map(d => { for (let x = d; x > addDays(d, -60); x = addDays(x, -1)) { const v = vo2maxFor(x); if (v) return v; } return null; });
  return {
    ...base, acute: {value: acute, max: 1000, label: acute > 560 ? 'High' : acute < 260 ? 'Low' : 'Optimal', tone: acute > 560 ? 'fair' : 'good', band: [300, 560]},
    status: 'Productive', statusTone: 'good', headline: acute < 260 ? 'Room to build' : 'Building steadily',
    statusWhy: {title:'Why Productive',summary:'Garmin’s training status and load inputs.',feedback:null,source:'Garmin training status',inputs:[{key:'acuteLoad',name:'Acute load',value:String(acute),rating:acute > 560 ? 'High' : acute < 260 ? 'Low' : 'Optimal',tone:acute > 560 ? 'fair' : 'good',detail:'Weighted recent exercise load'},{key:'vo2max',name:'VO₂ max',value:'47',rating:null,tone:'neutral',detail:'Latest recorded Garmin estimate'}]},
    chips: [{icon: 'flame', value: '1.0', text: 'Load ratio', tone: 'good'}, {icon: 'clock', value: `${Math.round(acute / 30)}h`, text: 'Recovery', tone: 'neutral'}],
    week: range(addDays(ctx.calendarDate, -6), ctx.calendarDate).map(d => ({date: d, load: acts.filter(a => a.date === d).reduce((t, a) => t + a.load, 0)})),
    loadFocus: [
      {key: 'lowAerobic', name: 'Low aerobic', value: Math.round(acute * .55), target: [200, 420], shortage: false},
      {key: 'highAerobic', name: 'High aerobic', value: Math.round(acute * .33), target: [150, 360], shortage: false},
      {key: 'anaerobic', name: 'Anaerobic', value: Math.round(acute * .12), target: [60, 180], shortage: acute * .12 < 60},
    ],
    vo2max: 47, vo2maxChange30d: 1, vo2maxTrend: vo2, races: running ? [{label: '5K', seconds: 1420}, {label: '10K', seconds: 2980}, {label: 'Half', seconds: 6660}, {label: 'Full', seconds: 14700}] : [],
    enduranceScore: 5870, hillScore: 61, runningToleranceKm: running ? 31 : null,
    thresholds: [
      ...(running ? [{kind: 'runningHr', value: 164, unit: 'bpm', recordedOn: '2026-10-28', stale: false}, {kind: 'runningPace', value: 298, unit: 's/km', recordedOn: '2026-10-28', stale: false}, {kind: 'runningPower', value: 286, unit: 'W', recordedOn: '2026-10-28', stale: false}] : []),
      {kind: 'cyclingFtp', value: 238, unit: 'W', recordedOn: '2026-10-12', stale: false}, {kind: 'heat', value: 0, unit: '%', recordedOn: '2026-11-17', stale: false},
    ] as TrainingView['thresholds'],
    workouts: recent.map(activityCard), gear: GEAR as TrainingView['gear'],
    planned: ctx.world.schedule.length ? {state: 'available', session: {name: ctx.world.schedule.find(s => s.date >= ctx.calendarDate)?.name ?? 'Leg day A', date: ctx.world.schedule.find(s => s.date >= ctx.calendarDate)?.date ?? '2026-11-21'}, coach: null, coverageStart: ctx.calendarDate, coverageEnd: addDays(ctx.calendarDate, 14)} : {state: 'empty', session: null, coach: null, coverageStart: ctx.calendarDate, coverageEnd: addDays(ctx.calendarDate, 14)},
    race: ctx.world.ambitions.find(a => a.id === 'event') ? {name: 'Winter 10K', date: '2026-12-12', daysUntil: daysBetween(ctx.calendarDate, '2026-12-12')} : null,
    raceState: ctx.world.ambitions.find(a => a.id === 'event') ? 'available' : 'empty',
    strength: strength.length && sv ? {figure: 'female', weekStart: ctx.weekFrom, sets: strength.reduce((n, a) => n + (setsFor(a) ?? []).reduce((t, e) => t + e.sets.length, 0), 0), sessionsWithSets: strength.filter(a => !a.timerOnly).length, strengthActivities: strength.length, records: strength.filter(a => a.record).length, muscles: sv.thisWeek} : null,
  };
}

function muscleTotals(acts: Activity[]): MuscleSets[] {
  const counts: Record<string, number> = {};
  for (const a of acts) for (const l of setsFor(a) ?? []) for (const m of l.muscles) counts[m] = (counts[m] ?? 0) + l.sets.length;
  return MUSCLES.map(muscle => ({muscle, sets: counts[muscle] ?? 0}));
}
export function musclesBetween(ctx: Ctx, from: string, to: string) {
  return muscleTotals(ctx.world.activities.filter(a => a.sport === 'strength' && between(a.date, from, to)));
}
export function strengthView(ctx: Ctx): StrengthView | null {
  const acts = ctx.world.activities.filter(a => a.sport === 'strength' && a.date <= ctx.calendarDate);
  if (!acts.length) return null;
  const ws = weekStart(ctx.calendarDate);
  const weeks = Array.from({length: 12}, (_, i) => addDays(ws, (i - 11) * 7));
  const recent = acts.filter(a => a.date >= weeks[0]);
  const exercises = EXERCISES.map(e => {
    const sessions = recent.flatMap(a => { const l = setsFor(a)?.find(x => x.exercise === e.name); if (!l) return []; const top = l.sets.reduce((b, s) => (s[1] ?? 0) > (b[1] ?? 0) ? s : b, l.sets[0]); return [{activityId: a.id, date: a.date, top: {weight: top[1], reps: top[0]}, reps: l.sets.map(s => s[0]), record: l.sets.some(s => s[2] === 'record')}]; }).reverse();
    const best = (reps: number) => {
      let w: number | null = null, date: string | null = null;
      for (const a of acts) for (const l of setsFor(a) ?? []) if (l.exercise === e.name) for (const s of l.sets) if ((s[0] ?? 0) >= reps && (s[1] ?? 0) > (w ?? 0)) { w = s[1]; date = a.date; }
      return {reps, weight: w, date};
    };
    return {key: e.key, name: e.name, muscles: e.muscles as Muscle[], sessions, bests: [1, 3, 5, 8, 10].map(best)};
  }).filter(e => e.sessions.length);
  const avg = muscleTotals(acts.filter(a => between(a.date, addDays(ws, -28), addDays(ws, -1)))).map(m => ({...m, sets: Math.round(m.sets / 4 * 10) / 10}));
  return {
    unit: 'kg', figure: 'female', date: ctx.calendarDate, weekStart: ws, thisWeek: musclesBetween(ctx, ws, ctx.calendarDate), average4w: avg,
    last4w: musclesBetween(ctx, addDays(ws, -21), ctx.calendarDate), weeks: weeks.map(w => ({weekStart: w, muscles: musclesBetween(ctx, w, addDays(w, 6))})),
    sessionsWithSetsThisWeek: acts.filter(a => a.date >= ws && !a.timerOnly).length, exercises,
    workouts: recent.slice().reverse().map(a => ({activity: activityCard(a), setsRecorded: !a.timerOnly})),
  };
}

export const sleepLine = (n: {asleep: number, need: number} | null) => !n ? 'No recorded night' : n.asleep >= n.need ? `Met need ${dur(n.need)}` : `${dur(n.need - n.asleep)} below need`;
export const readinessAt = readinessFor;
export {hm};
