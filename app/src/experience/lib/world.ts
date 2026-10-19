// Independently synthetic world for the experience prototype. No value here comes
// from a real Garmin account. Every number is deterministic, so screenshots and
// browser checks are repeatable. Names, routes and readings are illustrative.
import {addDays, between, daysBetween, hm, range, toMin, weekStart, weekday} from './dates';
import type {Sport} from './sports';

/** The prototype's interrupted week, Monday to Sunday. */
export const DAYS = ['2026-11-16', '2026-11-17', '2026-11-18', '2026-11-19', '2026-11-20', '2026-11-21', '2026-11-22'];
/** The start of downloaded history for the complete profile. */
export const HISTORY_START = '2025-09-01';
export const PARTIAL_HISTORY_START = '2026-09-01';

/** A stable pseudo-random number in [0, 1) for a key. */
export function rand(key: string) {
  let h = 2166136261;
  for (const c of key) h = Math.imul(h ^ c.charCodeAt(0), 16777619);
  h ^= h >>> 13; h = Math.imul(h, 0x5bd1e995); h ^= h >>> 15;
  return (h >>> 0) / 4294967296;
}
const pick = <T>(key: string, list: T[]) => list[Math.floor(rand(key) * list.length)];
const round1 = (n: number) => Math.round(n * 10) / 10;

export type SetRow = [reps: number | null, weight: number | null, mark?: 'record' | 'flag' | 'edited'];
export type ExerciseLog = {exercise: string, key: string | null, muscles: string[], sets: SetRow[], flagged?: string};
export type Activity = {
  id: string, date: string, start: string, sport: Sport, name: string, min: number, km?: number,
  im: number, load: number, benefit: string, aerobic: number, anaerobic: number,
  avgHr: number, maxHr: number, zones: number[], calories: number, ascent?: number,
  timerOnly?: boolean, record?: string, lateUpload?: string, routeKind?: number, generated?: boolean,
};
export type Night = {asleep: number, need: number, score: number, bed: string, wake: string, note?: string} | null;
export type Readiness = {value: number, level: 'Low' | 'Moderate' | 'High', limit: string};

const BENEFIT: Record<Sport, string[]> = {
  running: ['Base', 'Tempo', 'Threshold', 'Base', 'Recovery'], cycling: ['Base', 'Base', 'Tempo'], strength: ['Base', 'Anaerobic'],
  walking: ['Recovery'], hiking: ['Base'], hiit: ['Anaerobic', 'VO2 Max'], swimming: ['Base', 'Tempo'], yoga: ['Recovery'], other: ['Base'],
};

// Zone seconds shaped by sport and benefit, summing to the duration.
function zonesFor(sport: Sport, benefit: string, min: number, key: string): number[] {
  const shape: Record<string, number[]> = {
    Recovery: [.55, .38, .07, 0, 0], Base: [.12, .48, .32, .08, 0], Tempo: [.06, .2, .38, .32, .04], Threshold: [.05, .15, .25, .45, .1],
    Anaerobic: [.18, .25, .25, .2, .12], 'VO2 Max': [.1, .2, .25, .3, .15],
  };
  const s = (sport === 'strength' ? [.3, .42, .2, .07, .01] : sport === 'walking' ? [.62, .33, .05, 0, 0] : sport === 'yoga' ? [.8, .2, 0, 0, 0] : shape[benefit] ?? shape.Base)
    .map((v, i) => Math.max(0, v + (rand(key + 'z' + i) - .5) * .06));
  const total = s.reduce((a, b) => a + b, 0);
  return s.map(v => Math.round(v / total * min * 60));
}

function make(id: string, date: string, start: string, sport: Sport, o: Partial<Activity> = {}): Activity {
  const k = id;
  const r = (s: string) => rand(k + s);
  const defaults: Record<Sport, [min: number, kmPerMin: number, hr: number, loadPerMin: number, ae: number]> = {
    running: [34 + r('m') * 22, .175 + r('p') * .025, 148, 2.3, 3], cycling: [55 + r('m') * 50, .42 + r('p') * .08, 136, 1.7, 2.8],
    strength: [38 + r('m') * 18, 0, 112, 1.3, 2], walking: [26 + r('m') * 30, .085 + r('p') * .012, 96, .25, .9],
    hiking: [110 + r('m') * 80, .062, 118, .9, 2.4], hiit: [24 + r('m') * 10, 0, 152, 2.9, 2.6], swimming: [35 + r('m') * 15, .03, 128, 1.5, 2.3],
    yoga: [40 + r('m') * 20, 0, 84, .15, .6], other: [30, 0, 105, .8, 1.5],
  };
  const [m0, kpm, hr, lpm, ae] = defaults[sport];
  const min = Math.round(o.min ?? m0);
  const benefit = o.benefit ?? pick(k + 'b', BENEFIT[sport]);
  const km = o.km ?? (kpm ? round1(min * kpm) : undefined);
  const avgHr = o.avgHr ?? Math.round(hr + (benefit === 'Tempo' ? 8 : benefit === 'Threshold' ? 12 : benefit === 'Recovery' ? -10 : 0) + (r('h') - .5) * 8);
  const load = o.load ?? Math.round(min * lpm * (benefit === 'Recovery' ? .6 : benefit === 'Base' ? 1 : 1.3) * (.9 + r('l') * .2));
  const aerobic = o.aerobic ?? round1(Math.max(.4, Math.min(5, ae + (benefit === 'Recovery' ? -.3 : benefit === 'Base' ? 0 : .6) + (r('a') - .5) * .4)));
  const im = o.im ?? (sport === 'walking' || sport === 'yoga' ? (r('i') > .7 ? Math.round(min * .25) : 0) : Math.round(min * (benefit === 'Recovery' ? .3 : benefit === 'Base' ? .8 : 1.2)));
  const names: Partial<Record<Sport, string[]>> = {
    running: ['Morning Run', 'Run', 'River Run', 'Easy Run', 'Tempo Run'], cycling: ['Ride', 'Evening Ride', 'Commute Ride', 'Long Ride'],
    strength: ['Strength', 'Upper body', 'Lower body', 'Full body'], walking: ['Walk', 'Lunch Walk', 'Evening Walk'], hiking: ['Hike'],
    hiit: ['HIIT'], swimming: ['Pool Swim'], yoga: ['Yoga'],
  };
  return {
    id, date, start, sport, name: o.name ?? pick(k + 'n', names[sport] ?? ['Activity']), min, km, im, load, benefit, aerobic,
    anaerobic: o.anaerobic ?? round1(sport === 'hiit' ? 2.4 + r('an') : sport === 'strength' ? 1.2 + r('an') * .8 : benefit === 'Threshold' || benefit === 'Tempo' ? 1 + r('an') * .8 : r('an') * .5),
    avgHr, maxHr: o.maxHr ?? Math.round(avgHr + 14 + r('x') * 14), zones: o.zones ?? zonesFor(sport, benefit, min, k),
    calories: o.calories ?? Math.round(min * (avgHr - 40) * .085), ascent: km ? Math.round(km * (6 + r('e') * 9)) : undefined,
    routeKind: Math.floor(r('route') * 3), ...o,
  };
}

// The handcrafted recent activities carry the journey cases' evidence.
const BASE: Activity[] = [
  make('a-1102', '2026-11-09', '07:02', 'strength', {name: 'Strength', min: 44, im: 22, load: 61, benefit: 'Base', aerobic: 2.1}),
  make('a-1104', '2026-11-10', '18:05', 'running', {name: 'Run', min: 32, km: 5.6, im: 38, load: 74, benefit: 'Tempo', aerobic: 3.0}),
  make('a-1107', '2026-11-12', '07:10', 'strength', {name: 'Strength', min: 28, im: 8, load: 30, benefit: 'Recovery', aerobic: 1.3, timerOnly: true}),
  make('a-1110', '2026-11-14', '09:30', 'hiit', {name: 'HIIT', min: 30, im: 46, load: 88, benefit: 'Anaerobic', aerobic: 2.6}),
  make('a-1111', '2026-11-15', '10:00', 'cycling', {name: 'Ride', min: 70, km: 31.2, im: 64, load: 112, benefit: 'Base', aerobic: 2.9}),
  make('a-1112', '2026-11-15', '16:20', 'walking', {name: 'Walk', min: 36, km: 3.1, im: 0, load: 9, benefit: 'Recovery', aerobic: .8, lateUpload: '2026-11-16'}),
  make('a-1201', '2026-11-16', '07:05', 'strength', {name: 'Strength', min: 42, im: 18, load: 58, benefit: 'Base', aerobic: 2.0, record: 'Dumbbell bench press 22 kg × 8'}),
  make('a-1202', '2026-11-16', '17:40', 'cycling', {name: 'Ride', min: 25, km: 9.4, im: 20, load: 31, benefit: 'Base', aerobic: 1.8}),
  make('a-1203', '2026-11-17', '18:10', 'running', {name: 'Run', min: 35, km: 6.1, im: 40, load: 79, benefit: 'Tempo', aerobic: 3.1, avgHr: 154, maxHr: 172}),
  make('a-1204', '2026-11-18', '12:30', 'walking', {name: 'Walk', min: 24, km: 2.2, im: 0, load: 6, benefit: 'Recovery', aerobic: .6, avgHr: 94, maxHr: 108}),
  make('a-1205', '2026-11-22', '10:15', 'walking', {name: 'Walk', min: 40, km: 3.6, im: 10, load: 14, benefit: 'Recovery', aerobic: 1.0}),
];
/** Only used by the "target completed early" case; it shares the normal evidence path. */
export const EXTRA_ACTIVITY = make('a-1206', '2026-11-18', '07:00', 'strength', {name: 'Strength', min: 38, im: 16, load: 52, benefit: 'Base', aerobic: 1.9});

type Recipe = [day: number, start: string, sport: Sport, o?: Partial<Activity>][];
// Plan weeks before the interrupted week, matching the commitment history.
const RECIPES: Record<string, {recipe: Recipe, im?: number}> = {
  '2026-09-28': {recipe: [[1, '18:20', 'running'], [3, '12:40', 'walking'], [5, '09:10', 'cycling']]},
  '2026-10-05': {im: 168, recipe: [[0, '07:05', 'strength'], [1, '18:00', 'running', {benefit: 'Tempo'}], [3, '07:10', 'strength'], [4, '12:30', 'walking'], [5, '09:30', 'cycling', {min: 82}]]},
  '2026-10-12': {im: 172, recipe: [[0, '07:00', 'strength'], [2, '18:10', 'running', {benefit: 'Threshold'}], [4, '17:50', 'hiit'], [5, '08:45', 'cycling', {min: 95}], [6, '11:00', 'walking']]},
  '2026-10-19': {recipe: [[2, '13:00', 'walking'], [5, '10:30', 'walking']]},
  '2026-10-26': {im: 62, recipe: [[0, '07:05', 'strength'], [1, '18:15', 'running']]},
  '2026-11-02': {im: 151, recipe: [[0, '07:00', 'strength'], [1, '12:20', 'walking'], [2, '07:05', 'strength'], [3, '18:00', 'running', {benefit: 'Base'}], [5, '08:30', 'strength'], [6, '10:00', 'walking']]},
};
// Weeks with few or no activities: a winter cold and a summer holiday of hikes.
const QUIET: Record<string, Recipe> = {
  '2026-02-09': [], '2026-02-16': [[5, '11:00', 'walking']],
  '2026-08-10': [[1, '09:00', 'hiking'], [3, '08:30', 'hiking'], [5, '10:00', 'walking']],
};

function generatedWeek(ws: string): Activity[] {
  if (QUIET[ws]) return QUIET[ws].map(([d, start, sport, o], i) => make(`h-${ws}-${i}`, addDays(ws, d), start, sport, {...o, generated: true}));
  const month = Number(ws.slice(5, 7));
  const summer = month >= 5 && month <= 9;
  const r = (s: string) => rand(ws + s);
  const plan: Recipe = [];
  // A steady base: runs and walks most weeks, strength from late autumn 2025, rides in summer.
  const runs = r('runs') < .15 ? 0 : r('runs') < .6 ? 1 : 2;
  const strength = ws < '2025-11-03' ? (r('st') < .5 ? 1 : 0) : r('st') < .15 ? 1 : r('st') < .8 ? 2 : 3;
  const rides = summer ? (r('rd') < .3 ? 1 : 2) : r('rd') < .35 ? 1 : 0;
  const walks = 1 + Math.floor(r('wk') * 3);
  const days = [0, 1, 2, 3, 4, 5, 6].sort((a, b) => rand(ws + 'd' + a) - rand(ws + 'd' + b));
  let di = 0;
  const next = () => days[di++ % 7];
  for (let i = 0; i < strength; i++) plan.push([next(), pick(ws + 'st' + i, ['07:00', '07:10', '17:45', '18:05']), 'strength']);
  for (let i = 0; i < runs; i++) plan.push([next(), pick(ws + 'rn' + i, ['06:40', '18:10', '18:30', '07:15']), 'running']);
  for (let i = 0; i < rides; i++) plan.push([next(), pick(ws + 'rd' + i, ['09:00', '17:30', '10:15']), 'cycling']);
  for (let i = 0; i < walks; i++) plan.push([days[(di + i * 2) % 7], pick(ws + 'wk' + i, ['12:30', '12:45', '19:30', '16:10']), 'walking']);
  if (r('hiit') > .78) plan.push([next(), '17:50', 'hiit']);
  if (r('swim') > .85) plan.push([next(), '07:30', 'swimming']);
  if (r('yoga') > .82) plan.push([next(), '20:00', 'yoga']);
  return plan.map(([d, start, sport, o], i) => make(`h-${ws}-${i}`, addDays(ws, d), start, sport, {...o, generated: true}));
}

function buildActivities(): Activity[] {
  const out: Activity[] = [];
  for (let ws = HISTORY_START; ws < '2026-11-09'; ws = addDays(ws, 7)) {
    const fixed = RECIPES[ws];
    if (!fixed) { out.push(...generatedWeek(ws)); continue; }
    const week = fixed.recipe.map(([d, start, sport, o], i) => make(`h-${ws}-${i}`, addDays(ws, d), start, sport, {...o, generated: true}));
    // Recorded intensity minutes add up to the week's documented total.
    if (fixed.im != null) {
      const last = week.findLast(a => a.sport !== 'walking') ?? week[week.length - 1];
      last.im = Math.max(0, last.im + fixed.im - week.reduce((n, a) => n + a.im, 0));
    }
    out.push(...week);
  }
  out.push(...BASE);
  return out.sort((a, b) => (a.date + a.start).localeCompare(b.date + b.start));
}
export const ACTIVITIES = buildActivities();
export const activityById = (id: string) => ACTIVITIES.find(a => a.id === id) ?? (id === EXTRA_ACTIVITY.id ? EXTRA_ACTIVITY : undefined);

// ---------- strength sets ----------
type Lift = {name: string, key: string, muscles: string[], start: number, end: number, reps: number, step: number};
const LIFTS: Record<string, Lift> = {
  bench: {name: 'Dumbbell bench press', key: 'BENCH_PRESS/DUMBBELL_BENCH_PRESS', muscles: ['chest', 'arms', 'shoulders'], start: 14, end: 20, reps: 10, step: 2},
  row: {name: 'One-arm row', key: 'ROW/ONE_ARM_DUMBBELL_ROW', muscles: ['back', 'arms'], start: 16, end: 24, reps: 10, step: 2},
  goblet: {name: 'Goblet squat', key: 'SQUAT/GOBLET_SQUAT', muscles: ['quads', 'glutes'], start: 12, end: 18, reps: 10, step: 2},
  rdl: {name: 'Romanian deadlift', key: 'DEADLIFT/ROMANIAN_DEADLIFT', muscles: ['hamstrings', 'glutes', 'back'], start: 30, end: 46, reps: 8, step: 2},
  press: {name: 'Shoulder press', key: 'SHOULDER_PRESS/DUMBBELL_SHOULDER_PRESS', muscles: ['shoulders', 'arms'], start: 8, end: 14, reps: 10, step: 2},
  lunge: {name: 'Walking lunge', key: 'LUNGE/DUMBBELL_WALKING_LUNGE', muscles: ['quads', 'glutes', 'hamstrings'], start: 8, end: 14, reps: 12, step: 2},
  calf: {name: 'Standing calf raise', key: 'CALF_RAISE/STANDING_CALF_RAISE', muscles: ['calves'], start: 16, end: 24, reps: 15, step: 2},
  plank: {name: 'Plank', key: 'PLANK/PLANK', muscles: ['core'], start: 0, end: 0, reps: 1, step: 0},
};
export const EXERCISES = Object.entries(LIFTS).map(([id, l]) => ({id: l.name.toLowerCase().replace(/[^a-z]+/g, '-'), lift: id, ...l}));

const FIXED_SETS: Record<string, ExerciseLog[]> = {
  'a-1201': [
    {exercise: 'Dumbbell bench press', key: LIFTS.bench.key, muscles: LIFTS.bench.muscles, sets: [[10, 20], [8, 22, 'record'], [8, 22]]},
    {exercise: 'One-arm row', key: LIFTS.row.key, muscles: LIFTS.row.muscles, sets: [[10, 24], [10, 24], [9, 24]]},
    {exercise: 'Not recognised', key: null, muscles: [], sets: [[12, null], [12, null]]},
    {exercise: 'Plank', key: LIFTS.plank.key, muscles: ['core'], sets: [[1, null, 'flag']], flagged: 'One set looks off: 1 rep recorded for a timed hold'},
  ],
  'a-1102': [
    {exercise: 'Dumbbell bench press', key: LIFTS.bench.key, muscles: LIFTS.bench.muscles, sets: [[10, 20], [10, 20], [8, 20]]},
    {exercise: 'Goblet squat', key: LIFTS.goblet.key, muscles: LIFTS.goblet.muscles, sets: [[10, 20, 'record'], [10, 20]]},
  ],
  'a-1206': [{exercise: 'Goblet squat', key: LIFTS.goblet.key, muscles: LIFTS.goblet.muscles, sets: [[10, 20], [10, 22], [8, 22]]}],
};
/** The edited set behind the corrected best (U63): Garmin Connect now says 14 kg. */
export const CORRECTED_SESSION = 'h-2026-06-15-0';

export function setsFor(a: Activity): ExerciseLog[] | null {
  if (a.sport !== 'strength' || a.timerOnly) return null;
  if (FIXED_SETS[a.id]) return FIXED_SETS[a.id];
  const t = Math.max(0, Math.min(1, daysBetween(HISTORY_START, a.date) / daysBetween(HISTORY_START, '2026-11-08')));
  const upper = rand(a.id + 'split') < .5;
  const chosen = upper ? ['bench', 'row', 'press'] : ['goblet', 'rdl', 'lunge', 'calf'];
  if (a.id === CORRECTED_SESSION && !chosen.includes('goblet')) chosen.unshift('goblet');
  return chosen.map(k => {
    const l = LIFTS[k];
    const w = Math.round((l.start + (l.end - l.start) * t + (rand(a.id + k) - .6) * l.step) / l.step) * l.step;
    const n = 3;
    const sets: SetRow[] = Array.from({length: n}, (_, i) => [l.reps - (i === n - 1 && rand(a.id + k + i) > .5 ? 2 : 0), Math.max(l.step, w)]);
    if (a.id === CORRECTED_SESSION && k === 'goblet') sets[0] = [10, 14, 'edited'];
    return {exercise: l.name, key: l.key, muscles: l.muscles, sets};
  });
}

// ---------- sleep ----------
const FIXED_NIGHTS: Record<string, Night> = {
  '2026-11-10': {asleep: 452, need: 470, score: 79, bed: '22:58', wake: '06:46'},
  '2026-11-11': {asleep: 431, need: 465, score: 74, bed: '23:20', wake: '06:52'},
  '2026-11-12': {asleep: 470, need: 470, score: 82, bed: '22:49', wake: '06:44'},
  '2026-11-13': null,
  '2026-11-14': {asleep: 495, need: 475, score: 85, bed: '23:05', wake: '07:40'},
  '2026-11-15': {asleep: 468, need: 470, score: 80, bed: '23:12', wake: '07:30'},
  '2026-11-16': {asleep: 446, need: 470, score: 77, bed: '22:56', wake: '06:45'},
  '2026-11-17': {asleep: 425, need: 480, score: 72, bed: '23:18', wake: '06:48'},
  '2026-11-18': {asleep: 310, need: 490, score: 48, bed: '00:52', wake: '06:40', note: 'Woke often between 02:00 and 04:00'},
  '2026-11-19': {asleep: 402, need: 495, score: 63, bed: '23:31', wake: '06:58'},
  '2026-11-20': {asleep: 488, need: 485, score: 84, bed: '22:40', wake: '07:15'},
  '2026-11-21': {asleep: 470, need: 475, score: 81, bed: '22:55', wake: '07:35'},
  '2026-11-22': {asleep: 455, need: 470, score: 78, bed: '23:02', wake: '07:20'},
};
const SLEEP_FIRST: Record<string, Night> = {
  '2026-11-16': {asleep: 380, need: 485, score: 64, bed: '00:10', wake: '06:30'},
  '2026-11-17': {asleep: 362, need: 495, score: 60, bed: '00:25', wake: '06:27'},
  '2026-11-18': {asleep: 345, need: 500, score: 57, bed: '00:41', wake: '06:26'},
};
/** Nights are filed under the morning they end, as Garmin does. */
export function nightFor(date: string, sleepFirst = false): Night {
  if (sleepFirst && date in SLEEP_FIRST) return SLEEP_FIRST[date];
  if (date in FIXED_NIGHTS) return FIXED_NIGHTS[date];
  if (date < HISTORY_START || date > DAYS[6] || rand(date + 'gap') < .045) return null;
  const weekend = [0, 6].includes(weekday(date));
  const late = sleepFirst ? 50 : 0;
  const bedMin = 22 * 60 + 50 + late + (rand(date + 'bed') - .5) * 70 + (weekend ? 30 : 0);
  const wakeMin = 6 * 60 + 45 + (rand(date + 'wake') - .5) * 30 + (weekend ? 50 : 0);
  const inBed = wakeMin + 1440 - bedMin;
  const asleep = Math.round(inBed - 14 - rand(date + 'aw') * 26 - (sleepFirst ? 25 : 0));
  const need = Math.round(465 + (rand(date + 'need') - .4) * 30);
  const score = Math.max(35, Math.min(95, Math.round(80 - (need - asleep) * .22 + (rand(date + 'sc') - .5) * 12)));
  return {asleep, need, score, bed: hm(bedMin), wake: hm(wakeMin)};
}

/** A coherent synthetic stage sequence for a night: deep early, REM later. */
export function stagesFor(date: string, asleep: number) {
  const out: {stage: 'deep' | 'light' | 'rem' | 'awake', startMin: number, minutes: number}[] = [];
  let t = 0, cycle = 0;
  const total = asleep + 18;
  while (t < total - 5) {
    const r = (s: string) => rand(`${date}${cycle}${s}`);
    const early = cycle < 2;
    const seq: [typeof out[0]['stage'], number][] = [['light', 14 + r('a') * 18], ['deep', early ? 30 + r('b') * 18 : 6 + r('b') * 10], ['light', 12 + r('c') * 14], ['rem', early ? 10 + r('d') * 8 : 20 + r('d') * 14]];
    if (r('w') > .55 || cycle === 2) seq.push(['awake', 3 + r('e') * 5]);
    for (const [stage, m] of seq) {
      const minutes = Math.round(Math.min(m, total - t));
      if (minutes <= 0) break;
      out.push({stage, startMin: t, minutes});
      t += minutes;
    }
    cycle++;
  }
  // Preserve the recorded duration and endpoints: the drawing must agree with
  // the headline and stage totals, rather than rounding into extra sleep.
  const regular = nightFor(date);
  const night = regular?.asleep === asleep ? regular : nightFor(date, true);
  const elapsed = night ? (toMin(night.wake) - toMin(night.bed) + 1440) % 1440 : asleep + 18;
  const awake = Math.max(0, elapsed - asleep);
  if (awake && !out.some(s => s.stage === 'awake')) out.push({stage:'awake',startMin:0,minutes:1});
  for (const [isAwake, target] of [[false, asleep], [true, awake]] as const) {
    const selected = out.filter(s => (s.stage === 'awake') === isAwake);
    const sum = selected.reduce((n,s) => n + s.minutes,0);
    let remaining = target;
    selected.forEach((s,i) => {s.minutes = i === selected.length - 1 ? remaining : Math.floor(s.minutes / sum * target); remaining -= s.minutes;});
  }
  let startMin = 0;
  return out.filter(s => s.minutes > 0).map(s => {const segment={...s,startMin};startMin+=s.minutes;return segment;});
}

// ---------- readiness, energy, vitals ----------
const FIXED_READINESS: Record<string, Readiness> = {
  '2026-11-16': {value: 72, level: 'Moderate', limit: 'Sleep history'},
  '2026-11-17': {value: 66, level: 'Moderate', limit: 'Recovery time'},
  '2026-11-18': {value: 38, level: 'Low', limit: 'Sleep score'},
  '2026-11-19': {value: 52, level: 'Moderate', limit: 'Sleep history'},
  '2026-11-20': {value: 84, level: 'High', limit: 'Acute load'},
  '2026-11-21': {value: 81, level: 'High', limit: 'Acute load'},
  '2026-11-22': {value: 77, level: 'High', limit: 'Sleep history'},
};
export function readinessFor(date: string): Readiness | null {
  if (date in FIXED_READINESS) return FIXED_READINESS[date];
  const n = nightFor(date);
  if (!n || date < HISTORY_START || date > DAYS[6]) return null;
  const value = Math.max(20, Math.min(96, Math.round(52 + n.score * .32 + (rand(date + 'rd') - .5) * 22)));
  return {value, level: value >= 75 ? 'High' : value >= 50 ? 'Moderate' : 'Low', limit: pick(date + 'lim', ['Sleep history', 'Acute load', 'Recovery time', 'HRV status'])};
}
const FIXED_BB: Record<string, number> = {'2026-11-16': 71, '2026-11-17': 64, '2026-11-18': 32, '2026-11-19': 48, '2026-11-20': 79, '2026-11-21': 76, '2026-11-22': 70};
/** Body Battery at the last upload of the day (current) or its daily high (history). */
export const bodyBatteryFor = (date: string) => FIXED_BB[date] ?? Math.round(58 + rand(date + 'bb') * 34);
export const bodyBatteryHigh = (date: string) => Math.min(100, bodyBatteryFor(date) + 12 + Math.round(rand(date + 'bbh') * 8));

const smooth = (date: string, base: number, swing: number, noise: number, salt: string) => {
  const t = daysBetween(HISTORY_START, date);
  return base + Math.sin(t / 47 + salt.length) * swing + (rand(date + salt) - .5) * noise;
};
export const restingHrFor = (date: string) => Math.round(smooth(date, 54, 1.6, 3, 'rhr') - Math.min(2, daysBetween(HISTORY_START, date) / 220));
export const sleepHrFor = (date: string) => Math.round(smooth(date, 52, 1.5, 3, 'shr'));
export const hrvFor = (date: string) => date === '2026-11-18' ? 38 : Math.round(smooth(date, 47, 3, 8, 'hrv'));
export const respirationFor = (date: string) => round1(smooth(date, 14.2, .3, .6, 'resp'));
export const spo2For = (date: string) => Math.round(smooth(date, 95.5, .6, 1.6, 'spo2'));
export const skinTempFor = (date: string) => round1(smooth(date, 0, .15, .5, 'skin'));
export const stressAvgFor = (date: string) => date === '2026-11-18' ? 41 : Math.round(smooth(date, 30, 4, 12, 'stress'));
/** Garmin VO₂ max: sparse, stepping as new estimates are recorded. */
export function vo2maxFor(date: string): number | null {
  if (date < HISTORY_START || rand(date + 'vo2') > .16 && date !== '2026-11-03') return null;
  return date >= '2026-11-03' ? 47 : date >= '2026-03-20' ? 46 : 45;
}
/** Garmin Fitness Age: flat for the last 8 weeks. */
export function fitnessAgeFor(date: string): number | null {
  if (date < HISTORY_START || rand(date + 'fa') > .2 && date !== '2026-11-03') return null;
  return date >= '2026-09-07' ? 41 : date >= '2026-02-23' ? 42 : 43;
}
export const CHRONOLOGICAL_AGE = 44;
/** Garmin endurance and hill scores: occasional estimates, not daily readings. */
export function enduranceFor(date: string): number | null {
  if (date < HISTORY_START || rand(date + 'end') > .1) return null;
  return Math.round(5600 + daysBetween(HISTORY_START, date) / 2 + (rand(date + 'en') - .5) * 80);
}
export function hillFor(date: string): number | null {
  if (date < HISTORY_START || rand(date + 'hill') > .07) return null;
  return date >= '2026-08-01' ? 61 : 58;
}
/** Scale weigh-ins: most mornings, with day-to-day water swings and a slow loss lately. */
export function weightFor(date: string): number | null {
  if (date < HISTORY_START || date > '2026-11-22' || rand(date + 'wt') > .62) return null;
  const days = daysBetween(HISTORY_START, date), recent = Math.max(0, daysBetween('2026-09-01', date));
  return round1(75.6 - days / 900 - recent * .018 + (rand(date + 'w') - .5) * 1.4 + (weekday(date) === 1 ? .35 : 0));
}

// ---------- intraday timelines (existing view fields) ----------
function asleepRanges(date: string, sleepFirst: boolean, untilMin: number): [number, number][] {
  const out: [number, number][] = [];
  const last = nightFor(date, sleepFirst);
  if (last) { const b = toMin(last.bed), w = toMin(last.wake); out.push([b > 12 * 60 ? 0 : b, w]); }
  const tonight = nightFor(addDays(date, 1), sleepFirst);
  if (tonight && toMin(tonight.bed) > 12 * 60 && toMin(tonight.bed) < untilMin) out.push([toMin(tonight.bed), Math.min(1440, untilMin)]);
  return out;
}
export function dayTimelines(date: string, acts: Activity[], sleepFirst: boolean, untilMin = 1440) {
  const sleep = asleepRanges(date, sleepFirst, untilMin);
  const active = acts.filter(a => a.date === date).map(a => [toMin(a.start), Math.min(1440, toMin(a.start) + a.min)] as [number, number]);
  const inside = (m: number, list: [number, number][]) => list.some(([a, b]) => m >= a && m < b);
  const stress = Array.from({length: 480}, (_, i) => {
    const m = i * 3;
    if (m >= untilMin) return null;
    if (inside(m, active)) return null;
    if (rand(date + 'off' + Math.floor(m / 45)) < .03) return null;
    if (inside(m, sleep)) return Math.round(12 + rand(date + 's' + i) * 10);
    const busy = Math.sin((m - 540) / 140) * 12 + (m > 600 && m < 1020 ? 14 : 0);
    return Math.max(4, Math.min(96, Math.round(stressAvgFor(date) + busy + (rand(date + 'st' + i) - .5) * 22)));
  });
  const hr = Array.from({length: 720}, (_, i) => {
    const m = i * 2;
    if (m >= untilMin || rand(date + 'hoff' + Math.floor(m / 40)) < .02) return null;
    const act = acts.find(a => a.date === date && m >= toMin(a.start) && m < toMin(a.start) + a.min);
    if (act) return Math.round(act.avgHr + Math.sin((m - toMin(act.start)) / 3) * 8 + (rand(date + 'h' + i) - .5) * 6);
    if (inside(m, sleep)) return Math.round(sleepHrFor(date) - 2 + (rand(date + 'hs' + i) - .5) * 5);
    return Math.round(68 + Math.sin(m / 90) * 6 + (rand(date + 'hd' + i) - .5) * 10);
  });
  return {stress, hr, sleep, active};
}

// ---------- routes, HR traces and splits ----------
const CENTER: [number, number] = [47.02, 8.03];
/** A synthetic loop or out-and-back; three recurring shapes, like favourite routes. */
export function routeFor(a: Activity): [number, number][] {
  if (!a.km || !['running', 'cycling', 'walking', 'hiking'].includes(a.sport)) return [];
  const n = 90;
  const kind = a.routeKind ?? 0;
  const scale = Math.sqrt(a.km) * .0042;
  const phase = rand(a.id + 'ph') * Math.PI * 2;
  const pts: [number, number][] = [];
  for (let i = 0; i <= n; i++) {
    const t = i / n * Math.PI * 2;
    let x: number, y: number;
    if (kind === 0) { x = Math.cos(t) * 1.2 + Math.cos(2 * t + phase) * .25; y = Math.sin(t) * .8 + Math.sin(3 * t + phase) * .12; }
    else if (kind === 1) { const u = Math.sin(t / 2); x = u * 2 + Math.sin(t * 3) * .05; y = Math.sin(u * 3 + phase) * .35 + (i > n / 2 ? .08 : 0); }
    else { x = Math.sin(t) * 1.1; y = Math.sin(t) * Math.cos(t) * 1.1 + Math.cos(3 * t + phase) * .1; }
    const wob = (rand(a.id + 'w' + i) - .5) * .03;
    pts.push([CENTER[0] + (y + wob) * scale, CENTER[1] + (x + wob) * scale * 1.45]);
  }
  return pts;
}
export function hrTrace(a: Activity) {
  const s = a.min * 60, step = Math.max(10, Math.ceil(s / 280));
  const out: {tS: number, bpm: number}[] = [];
  for (let t = 0; t <= s; t += step) {
    const warm = Math.min(1, t / 420);
    const intervals = a.sport === 'hiit' || a.benefit === 'Threshold' ? (Math.floor(t / 180) % 2 ? 12 : -6) : a.benefit === 'Tempo' ? (t > s * .25 && t < s * .8 ? 8 : -4) : 0;
    const strength = a.sport === 'strength' ? (Math.floor(t / 90) % 2 ? 16 : -10) : 0;
    const drift = t / s * 6;
    out.push({tS: t, bpm: Math.round(78 + (a.avgHr - 78) * warm + intervals * warm + strength + drift + (rand(a.id + 'hr' + t) - .5) * 6)});
  }
  return out;
}
export function splitsFor(a: Activity) {
  if (!a.km || !['running', 'walking', 'cycling', 'hiking'].includes(a.sport)) return [];
  const lap = a.sport === 'cycling' ? 5 : 1;
  const laps = Math.floor(a.km / lap);
  const per = a.min * 60 / a.km;
  const out = Array.from({length: laps}, (_, i) => ({index: i + 1, distanceKm: lap, seconds: Math.round(per * lap * (1 + (rand(a.id + 'sp' + i) - .5) * .08 - (i === laps - 2 ? .03 : 0))), avgHr: Math.round(a.avgHr - 6 + i / Math.max(1, laps) * 10)}));
  const rest = round1(a.km - laps * lap);
  if (rest >= .1) out.push({index: laps + 1, distanceKm: rest, seconds: Math.round(per * rest), avgHr: a.avgHr + 3});
  return out;
}

// ---------- plan context ----------
export const SCHEDULE = [
  {id: 'g-int', date: '2026-11-18', name: 'Intervals 6 × 3 min', sport: 'running' as Sport, source: 'garmin', routine: false, min: 45, plan: {hi: 18, im: 45} as Allocation, steps: 'Warm-up 10 min · 6 × 3 min hard, 2 min easy · cool-down 5 min'},
  {id: 'g-tempo', date: '2026-11-19', name: 'Tempo run 40 min', sport: 'running' as Sport, source: 'garmin', routine: false, min: 40, plan: {im: 40} as Allocation, steps: 'Warm-up 10 min · 20 min steady-hard · cool-down 10 min'},
  {id: 'g-legs', date: '2026-11-21', name: 'Leg day A', sport: 'strength' as Sport, source: 'garmin', routine: true, min: 45, plan: {strength: 1} as Allocation, steps: 'Exercises not visible to Raring'},
];
/** What a Garmin workout plans toward goal kinds, read from its steps' durations and zone targets (synthetic; step targets unverified). */
export type Allocation = {strength?: number, zone2?: number, hi?: number, im?: number, sleep?: number};
export type Schedule = typeof SCHEDULE[number];
/** An active Garmin Coach plan is context only: Garmin doesn't say which schedule items belong to it. */
export const COACH = {name: '10K plan', since: '2026-11-02'};
export const COMMITMENT_VERSIONS = [
  {from: '2026-W41', strength: 2, im: 150, note: 'First saved plan'},
  {from: '2026-W45', strength: 3, im: 150, note: 'Raised strength'},
  {from: '2026-W47', strength: 2, im: 150, note: 'Lowered strength before the week began'},
];
export const PLAN_WEEKS = [
  {id: '2026-W40', from: '2026-09-28', state: 'none', label: 'No plan yet', detail: '3 recorded activities'},
  {id: '2026-W41', from: '2026-10-05', state: 'reached', label: 'Reached', detail: 'Strength 2 of 2 · 168 of 150 min', version: '2 strength · 150 min', strength: [2, 2], im: [168, 150]},
  {id: '2026-W42', from: '2026-10-12', state: 'partial', label: '1 of 2 reached', detail: 'Strength 1 of 2 · 172 of 150 min', version: '2 strength · 150 min', strength: [1, 2], im: [172, 150]},
  {id: '2026-W43', from: '2026-10-19', state: 'paused', label: 'Paused', detail: 'Whole week paused · 2 activities recorded', version: '2 strength · 150 min', strength: [0, 2], im: [0, 150]},
  {id: '2026-W44', from: '2026-10-26', state: 'unknown', label: 'Awaiting evidence', detail: 'Activity list incomplete · 2 known workouts', version: '2 strength · 150 min', strength: [1, 2], im: [62, 150]},
  {id: '2026-W45', from: '2026-11-02', state: 'reached', label: 'Reached', detail: 'Strength 3 of 3 · 151 of 150 min', version: '3 strength · 150 min', strength: [3, 3], im: [151, 150]},
  {id: '2026-W46', from: '2026-11-09', state: 'partial', label: '1 of 2 reached', detail: 'Strength 2 of 3 · 178 of 150 min', version: '3 strength · 150 min', strength: [2, 3], im: [178, 150]},
] as {id: string, from: string, state: string, label: string, detail: string, version?: string, strength?: [number, number], im?: [number, number]}[];
export const MILESTONES = [
  {id: 'm-bench', date: '2026-11-16', kind: 'best', title: 'Dumbbell bench press', value: '22 kg × 8', note: 'Best in available history', route: 'workout/a-1201'},
  {id: 'm-goblet', date: '2026-11-09', kind: 'best', title: 'Goblet squat', value: '20 kg × 10', note: 'Best in available history · corrected once', corrected: 'An earlier 24 kg set was edited in Garmin Connect to 14 kg, so it is no longer a best.', route: 'workout/a-1102'},
  {id: 'm-vo2', date: '2026-11-03', kind: 'estimate', title: 'VO₂ max', value: '47', note: 'Garmin estimate · first at this value', route: 'metric/vo2max'},
  {id: 'm-week', date: '2026-10-11', kind: 'plan', title: 'First plan week', value: 'Every commitment met', note: '2 of 2 strength · 168 of 150 min', route: 'review/2026-W41'},
];
export const GEAR = {unit: 'km', fetchedAt: '2026-11-18T07:12:00Z', items: [
  {id: 'g-road', name: 'Road shoes', kind: 'Shoes', status: 'Active', distance: 512, threshold: 700},
  {id: 'g-bike', name: 'Commuter bike', kind: 'Bike', status: 'Active', distance: 1840, threshold: null},
  {id: 'g-trail', name: 'Old trail shoes', kind: 'Shoes', status: 'Retired', distance: 804, threshold: 800},
]};
export const NAPS = {
  same: [{start: ['2026-11-18', '13:10'], end: ['2026-11-18', '13:36'], min: 26}],
  cross: [{start: ['2026-11-17', '23:40'], end: ['2026-11-18', '00:20'], min: 40}, {start: ['2026-11-18', '15:05'], end: null, min: 22}],
} as Record<string, {start: [string, string], end: [string, string] | null, min: number}[]>;
export const JOURNAL = [
  {date: '2026-11-18', state: 'available', entries: [['Caffeine', '2 cups'], ['Alcohol', 'No'], ['Late meal', null]]},
  {date: '2026-11-17', state: 'empty', entries: []},
  {date: '2026-11-16', state: 'available', entries: [['Caffeine', '1 cup'], ['Alcohol', 'Yes']]},
  {date: '2026-11-15', state: 'unavailable', entries: []},
] as {date: string, state: 'available' | 'empty' | 'unavailable', entries: [string, string | null][]}[];

export const inWeek = (date: string, of: string) => between(date, weekStart(of), addDays(weekStart(of), 6));
export const allDays = (from: string, to: string) => range(from, to);
