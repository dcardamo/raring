// Progress period summaries. These are the proposed derived rules written up in
// docs/experience-design.md ("Proposed rules for Progress"); nothing here ships
// until that section is in spec.md and tested in the Rust core.
import {addDays, addMonths, between, daysBetween, dayMonth, monthEnd, monthName, monthStart, monthYear, range, weekId, weekRange, weekStart} from './dates';
import {versionFor, type Ctx} from './rules';
import {SPORTS, type Sport} from './sports';
import {PLAN_WEEKS, fitnessAgeFor, hrvFor, restingHrFor, vo2maxFor, bodyBatteryHigh, stressAvgFor, setsFor, EXERCISES, type Activity} from './world';
import {acuteLoad} from './views';

export type Period = 'week' | 'month' | 'quarter' | 'year';
export const PERIODS: {key: Period, label: string}[] = [{key: 'week', label: 'Week'}, {key: 'month', label: 'Month'}, {key: 'quarter', label: '3 months'}, {key: 'year', label: 'Year'}];

/** Bounds of the period containing `anchor`; the end never passes today. */
export function bounds(period: Period, anchor: string, today: string) {
  let from: string, to: string;
  if (period === 'week') { from = weekStart(anchor); to = addDays(from, 6); }
  else if (period === 'month') { from = monthStart(anchor); to = monthEnd(anchor); }
  else if (period === 'quarter') { from = addMonths(anchor, -2); to = monthEnd(anchor); }
  else { from = addMonths(anchor, -11); to = monthEnd(anchor); }
  const end = to > today ? today : to;
  const label = period === 'week' ? weekRange(from) : period === 'month' ? monthYear(from) : period === 'quarter' ? `${monthName(from)}–${monthName(to)} ${to.slice(0, 4)}` : `${monthName(from)} ${from.slice(0, 4)}–${monthName(to)} ${to.slice(0, 4)}`;
  return {from, to, end, label, current: to >= today, elapsed: daysBetween(from, end) + 1, length: daysBetween(from, to) + 1};
}
export function shift(period: Period, anchor: string, n: number) {
  if (period === 'week') return addDays(anchor, 7 * n);
  if (period === 'month') return addMonths(anchor, n);
  if (period === 'quarter') return addMonths(anchor, 3 * n);
  return addMonths(anchor, 12 * n);
}

/** Days we can speak about: downloaded and not inside a coverage gap. */
export function covered(ctx: Ctx, d: string) {
  return d >= ctx.world.historyStart && d <= ctx.calendarDate && !ctx.world.coverageGaps.includes(weekId(d)) && !(ctx.world.coverageIncompleteFrom && d >= ctx.world.coverageIncompleteFrom);
}

export type Totals = {activities: number, minutes: number, km: number, im: number, activeDays: number, coveredDays: number, days: number};
export function totals(ctx: Ctx, from: string, to: string): Totals {
  const acts = ctx.world.activities.filter(a => between(a.date, from, to));
  const days = range(from, to);
  return {
    activities: acts.length, minutes: acts.reduce((t, a) => t + a.min, 0), km: Math.round(acts.reduce((t, a) => t + (a.km ?? 0), 0) * 10) / 10,
    im: acts.reduce((t, a) => t + a.im, 0), activeDays: new Set(acts.map(a => a.date)).size, coveredDays: days.filter(d => covered(ctx, d)).length, days: days.length,
  };
}

export function summary(ctx: Ctx, period: Period, anchor: string) {
  const b = bounds(period, anchor, ctx.calendarDate);
  const now = totals(ctx, b.from, b.end);
  // Compare to date: the previous period's first equal number of days.
  const prevFrom = shift(period, b.from, -1);
  const prevEnd = b.current ? addDays(prevFrom, b.elapsed - 1) : addDays(b.from, -1);
  const prev = prevFrom >= ctx.world.historyStart ? totals(ctx, prevFrom, prevEnd) : null;
  return {...b, now, prev, prevFrom, prevEnd};
}

/** Minutes by sport for each bucket of the period (days, weeks or months). */
export function buckets(ctx: Ctx, period: Period, from: string, to: string, end: string) {
  const list: {from: string, to: string, label: string, short: string}[] = [];
  if (period === 'week' || period === 'month') for (const d of range(from, to)) list.push({from: d, to: d, label: dayMonth(d), short: period === 'week' ? 'MTWTFSS'[(new Date(d + 'T12:00:00Z').getUTCDay() + 6) % 7] : String(Number(d.slice(8)))});
  else if (period === 'quarter') for (let w = weekStart(from); w <= to; w = addDays(w, 7)) list.push({from: w < from ? from : w, to: addDays(w, 6) > to ? to : addDays(w, 6), label: weekRange(w), short: dayMonth(w)});
  else for (let m = from; m <= to; m = addMonths(m, 1)) list.push({from: m, to: monthEnd(m), label: monthYear(m), short: monthName(m)[0]});
  return list.map(x => {
    const acts = ctx.world.activities.filter(a => between(a.date, x.from, x.to));
    const bySport = new Map<Sport, number>();
    for (const a of acts) bySport.set(a.sport, (bySport.get(a.sport) ?? 0) + a.min);
    const future = x.from > end;
    const known = range(x.from, x.to > end ? end : x.to).some(d => covered(ctx, d));
    return {...x, future, known, total: acts.reduce((t, a) => t + a.min, 0), sports: [...bySport.entries()].sort((a, b) => b[1] - a[1])};
  });
}
export function sportMix(ctx: Ctx, from: string, to: string) {
  const m = new Map<Sport, {min: number, count: number}>();
  for (const a of ctx.world.activities.filter(x => between(x.date, from, to))) {
    const v = m.get(a.sport) ?? {min: 0, count: 0}; v.min += a.min; v.count++; m.set(a.sport, v);
  }
  return [...m.entries()].sort((a, b) => b[1].min - a[1].min).map(([sport, v]) => ({sport, ...v, name: SPORTS[sport].plural, color: SPORTS[sport].color}));
}

/** One calendar month for the activity calendar. */
export function calendarMonth(ctx: Ctx, month: string) {
  const from = monthStart(month), to = monthEnd(month);
  const lead = (new Date(from + 'T12:00:00Z').getUTCDay() + 6) % 7;
  return {from, to, lead, days: range(from, to).map(d => {
    const acts = ctx.world.activities.filter(a => a.date === d);
    const rest = ctx.state.rest?.day === d;
    const paused = [...ctx.state.pauses, ...(ctx.state.pause ? [ctx.state.pause] : [])].some(p => d >= p.since && (!p.until || d <= p.until)) || PLAN_WEEKS.some(w => w.state === 'paused' && between(d, w.from, addDays(w.from, 6))) && ['mixed', 'sparse'].includes(ctx.state.profile);
    const state = d > ctx.calendarDate ? 'future' : d < ctx.world.historyStart ? 'before' : !covered(ctx, d) && !acts.length ? 'unknown' : acts.length ? 'active' : rest ? 'rest' : paused ? 'paused' : 'none';
    return {date: d, acts, rest, paused, state, today: d === ctx.calendarDate};
  })};
}

// ---------- commitments over time ----------
export type WeekBar = {id: string, from: string, label: string, state: string, value: number | null, target: number | null, current?: boolean};
export function commitmentWeeks(ctx: Ctx, key: 'strength' | 'im'): WeekBar[] {
  if (!['mixed', 'sparse'].includes(ctx.state.profile) && !ctx.state.ownCommitments) return [];
  const rows: WeekBar[] = PLAN_WEEKS.map(w => {
    const pair = key === 'strength' ? w.strength : w.im;
    return {id: w.id, from: w.from, label: dayMonth(w.from), state: w.state === 'partial' ? (pair && pair[0] >= pair[1] ? 'reached' : 'open') : w.state, value: w.state === 'none' ? (key === 'strength' ? countSport(ctx.world.activities, 'strength', w.from) : sumIm(ctx.world.activities, w.from)) : pair?.[0] ?? null, target: pair?.[1] ?? null};
  });
  if (!['mixed', 'sparse'].includes(ctx.state.profile)) rows.length = 0;
  const p = ctx.plan;
  if (p) {
    const t = p.tracks.find(x => x.key === key);
    rows.push({id: p.wk, from: ctx.weekFrom, label: 'Now', state: t?.state ?? 'open', value: t?.have ?? null, target: t?.target ?? null, current: true});
  }
  return rows.filter(r => r.from >= ctx.world.historyStart || r.current).slice(-10);
}
const countSport = (acts: Activity[], s: Sport, from: string) => acts.filter(a => a.sport === s && between(a.date, from, addDays(from, 6))).length;
const sumIm = (acts: Activity[], from: string) => acts.filter(a => between(a.date, from, addDays(from, 6))).reduce((t, a) => t + a.im, 0);
export const versionsFor = (ctx: Ctx) => versionFor(ctx).versions;

// ---------- trends ----------
type Series = (d: string) => number | null;
export const TREND_TILES: {id: string, name: string, unit: string, f: (ctx: Ctx) => Series, decimals?: number, lowerIsCalmer?: boolean, sparse?: boolean, needs?: (ctx: Ctx) => boolean}[] = [
  {id: 'vo2max', name: 'VO₂ max', unit: '', f: () => vo2maxFor, sparse: true, needs: c => c.world.readinessSupported},
  {id: 'fitnessAge', name: 'Fitness Age', unit: 'yrs', f: () => fitnessAgeFor, sparse: true},
  {id: 'restingHr', name: 'Resting HR', unit: 'bpm', f: () => restingHrFor},
  {id: 'hrv', name: 'HRV', unit: 'ms', f: c => d => c.nightOf(d) ? hrvFor(d) : null},
  {id: 'sleepTime', name: 'Time asleep', unit: 'h', f: c => d => c.nightOf(d)?.asleep ?? null},
  {id: 'bodyBattery', name: 'Body Battery high', unit: '', f: () => bodyBatteryHigh},
  {id: 'stress', name: 'Stress', unit: 'avg', f: () => stressAvgFor},
  {id: 'acuteLoad', name: 'Acute load', unit: '', f: c => d => acuteLoad(c, d), needs: c => c.world.activities.length > 0},
];
export function trendTile(ctx: Ctx, tile: typeof TREND_TILES[number], from: string, end: string, prevFrom: string, prevEnd: string) {
  const f = tile.f(ctx);
  const pick = (a: string, b: string) => range(a, b).filter(d => d >= ctx.world.historyStart && d <= ctx.calendarDate).map(d => [d, f(d)] as const);
  const now = pick(from, end), prev = pick(prevFrom, prevEnd);
  const vals = now.map(([, v]) => v).filter((v): v is number => v != null);
  const pv = prev.map(([, v]) => v).filter((v): v is number => v != null);
  const avg = (l: number[]) => l.length ? l.reduce((a, b) => a + b, 0) / l.length : null;
  // Sparse estimates show their latest recorded value, not an average.
  const latest = (list: readonly (readonly [string, number | null])[]) => { for (let i = list.length - 1; i >= 0; i--) if (list[i][1] != null) return list[i]; return null; };
  const value = tile.sparse ? latest(now)?.[1] ?? latestBefore(ctx, f, from) : avg(vals);
  const before = tile.sparse ? latest(prev)?.[1] ?? latestBefore(ctx, f, prevFrom) : avg(pv);
  const step = Math.max(1, Math.round(now.length / 40));
  return {value, before, change: value != null && before != null ? value - before : null, points: now.filter((_, i) => i % step === 0 || i === now.length - 1).map(([, v]) => v), count: vals.length, latestDate: latest(now)?.[0] ?? null};
}
function latestBefore(ctx: Ctx, f: Series, d: string) { for (let x = addDays(d, -1); x >= ctx.world.historyStart; x = addDays(x, -1)) { const v = f(x); if (v != null) return v; } return null; }

// ---------- strength ----------
export function liftProgress(ctx: Ctx, from: string, end: string) {
  const acts = ctx.world.activities.filter(a => a.sport === 'strength' && a.date <= end);
  return EXERCISES.map(e => {
    const pts = acts.flatMap(a => { const l = setsFor(a)?.find(x => x.exercise === e.name); if (!l) return []; const w = Math.max(...l.sets.map(s => s[1] ?? 0)); return w ? [{date: a.date, weight: w, id: a.id}] : []; });
    const inPeriod = pts.filter(p => p.date >= from);
    const before = pts.filter(p => p.date < from);
    const best = Math.max(0, ...pts.map(p => p.weight));
    return {exercise: e, points: inPeriod, best, start: before.at(-1)?.weight ?? inPeriod[0]?.weight ?? null, end: inPeriod.at(-1)?.weight ?? null};
  }).filter(x => x.points.length >= 2).sort((a, b) => b.points.length - a.points.length).slice(0, 3);
}
export function strengthWeeks(ctx: Ctx, from: string, end: string) {
  const out: {from: string, sets: number, sessions: number, timerOnly: number}[] = [];
  for (let w = weekStart(from); w <= end; w = addDays(w, 7)) {
    const acts = ctx.world.activities.filter(a => a.sport === 'strength' && between(a.date, w, addDays(w, 6)));
    out.push({from: w, sets: acts.reduce((t, a) => t + (setsFor(a) ?? []).reduce((n, l) => n + l.sets.length, 0), 0), sessions: acts.length, timerOnly: acts.filter(a => a.timerOnly).length});
  }
  return out;
}

// ---------- records ----------
/** Longest recorded distance and time per sport in available history (proposed rule). */
export function records(ctx: Ctx) {
  const out: {id: string, sport: Sport, title: string, value: string, date: string, route: string}[] = [];
  const by = new Map<Sport, Activity[]>();
  for (const a of ctx.world.activities) { const l = by.get(a.sport) ?? []; l.push(a); by.set(a.sport, l); }
  for (const [sport, list] of by) {
    const far = list.filter(a => a.km).sort((a, b) => (b.km ?? 0) - (a.km ?? 0))[0];
    if (far && list.length >= 3) out.push({id: `far-${sport}`, sport, title: `Longest ${SPORTS[sport].name.toLowerCase()}`, value: `${far.km} km`, date: far.date, route: `workout/${far.id}`});
  }
  return out.sort((a, b) => b.date.localeCompare(a.date));
}
export {weekId, monthStart};
