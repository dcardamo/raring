// Training's week lanes and lift series. Proposed rules: design, "Proposed
// rules: week lanes and lift volume".
import {addDays, between} from './dates';
import {EXERCISES, setsFor, type Activity} from './world';
import {GOAL_INFO, goalsFor, recordedFor} from './allocation';
import {weekOutcome} from './week';
import type {Ctx} from './rules';

export type LaneKey = 'cardio' | 'zone2' | 'hi' | 'im' | 'strength';
export type Lane = {key: LaneKey, name: string, unit: string, color: string, goal: number | null, cum: (number | null)[], usual: {lo: number[], mid: number[], hi: number[]} | null, weeks: number};
const measure = (key: LaneKey, acts: Activity[]) => key === 'cardio' ? acts.filter(a => a.sport !== 'strength' && a.sport !== 'yoga').reduce((t, a) => t + a.min, 0) : recordedFor(acts, key);
const pct = (xs: number[], p: number) => { const s = [...xs].sort((a, b) => a - b); const i = (s.length - 1) * p, lo = Math.floor(i); return s.length ? s[lo] + (s[Math.ceil(i)] - s[lo]) * (i - lo) : 0; };
/** Cumulative Monday→Sunday for one week; days after `until` are null. */
const cumulative = (ctx: Ctx, key: LaneKey, from: string, until: string) => {
  let t = 0;
  return Array.from({length: 7}, (_, i) => { const d = addDays(from, i); if (d > until) return null; t += measure(key, ctx.world.activities.filter(a => a.date === d)); return t; });
};
/** Usual week: the recent complete weeks (counted: full coverage, at most one ill or paused day). */
export function usualWeeks(ctx: Ctx, n = 8) {
  return Array.from({length: n}, (_, i) => addDays(ctx.weekFrom, -7 * (i + 1))).filter(w => w >= ctx.world.historyStart && weekOutcome(ctx, w).counted);
}
export function weekLanes(ctx: Ctx): Lane[] {
  const g = goalsFor(ctx), weeks = usualWeeks(ctx);
  const keys: LaneKey[] = [...(['zone2', 'hi', 'im'] as const).filter(k => g[k] > 0)];
  if (!keys.length) keys.push('cardio');
  if (g.strength > 0 || ctx.world.activities.some(a => a.sport === 'strength' && a.date >= addDays(ctx.weekFrom, -56))) keys.push('strength');
  return keys.map(key => {
    const past = weeks.map(w => cumulative(ctx, key, w, addDays(w, 6)) as number[]);
    const usual = past.length >= 3 ? {lo: [0, 1, 2, 3, 4, 5, 6].map(i => pct(past.map(p => p[i]), .25)), mid: [0, 1, 2, 3, 4, 5, 6].map(i => pct(past.map(p => p[i]), .5)), hi: [0, 1, 2, 3, 4, 5, 6].map(i => pct(past.map(p => p[i]), .75))} : null;
    const info = key === 'cardio' ? {name: 'Cardio', unit: 'min', color: '#36C9C6'} : key === 'strength' ? {name: 'Strength', unit: 'sessions', color: GOAL_INFO.strength.color} : GOAL_INFO[key];
    return {key, name: info.name, unit: info.unit, color: info.color, goal: key === 'cardio' ? null : g[key] || null, cum: cumulative(ctx, key, ctx.weekFrom, ctx.calendarDate), usual, weeks: past.length};
  });
}

export type LiftPoint = {date: string, max: number | null, volume: number | null, excluded: number};
/** Per session: heaviest weighted set, and volume = Σ weight × reps over sets with both
 *  recorded. Bodyweight or unweighted sets are excluded from both and counted. */
export function liftSessions(ctx: Ctx, exercise: string, from: string): LiftPoint[] {
  return ctx.world.activities.filter(a => a.sport === 'strength' && between(a.date, from, ctx.calendarDate)).flatMap(a => {
    const log = setsFor(a)?.find(l => l.exercise === exercise);
    if (!log) return [];
    const weighted = log.sets.filter(([r, w]) => (w ?? 0) > 0 && (r ?? 0) > 0);
    return [{date: a.date, max: weighted.length ? Math.max(...weighted.map(([, w]) => w!)) : null, volume: weighted.length ? weighted.reduce((t, [r, w]) => t + r! * w!, 0) : null, excluded: log.sets.length - weighted.length}];
  });
}
export function topLifts(ctx: Ctx, from: string, n = 3) {
  return EXERCISES.map(e => ({name: e.name, id: e.id, points: liftSessions(ctx, e.name, from)})).filter(x => x.points.filter(p => p.max != null).length >= 2).sort((a, b) => b.points.length - a.points.length).slice(0, n);
}
