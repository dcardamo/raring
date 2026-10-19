// The weekly loop: day and week rings, the backlog of what's left, the
// week-end outcome and next week's proposed goals. Prototype rules; the
// design's "Day and week rings" and "Proposed rules: next week's goals".
import {addDays, between, toMin, weekId, weekStart} from './dates';
import {PLAN_WEEKS} from './world';
import {SPORTS, type Sport} from './sports';
import {allocationFor, candidates, GOAL_INFO, goalsFor, intentionState, isOpen, recordedFor, type GoalKey} from './allocation';
import type {Ctx, GoalTargets, Intention, Pause} from './rules';
import {itemMatch, planItems, type PlanItem} from './match';

export const RING_ORDER: GoalKey[] = ['strength', 'zone2', 'hi', 'im', 'sleep'];
export const activeKeys = (g: GoalTargets) => RING_ORDER.filter(k => g[k] > 0);

export type DayMark = 'rest' | 'ill' | 'paused' | 'unknown' | null;
/** One ring: `filled` is recorded progress, `outline` how much of the ring is drawn at all (0 = no ring). */
export type Ring = {key: GoalKey, color: string, name: string, unit: string, recorded: number, planned: number, filled: number, outline: number};

const pausesOf = (ctx: Ctx): Pause[] => [...ctx.state.pauses, ...(ctx.state.pause ? [ctx.state.pause] : [])];
export function pauseOn(ctx: Ctx, date: string): Pause | null {
  return pausesOf(ctx).find(p => date >= p.since && (!p.until || date <= p.until)) ?? null;
}
export function dayMark(ctx: Ctx, date: string): DayMark {
  if (ctx.state.rest?.day === date) return 'rest';
  const p = pauseOn(ctx, date);
  if (p) return p.reason === 'ill' ? 'ill' : 'paused';
  if (date <= ctx.calendarDate && unknownDay(ctx, date)) return 'unknown';
  return null;
}
export const unknownDay = (ctx: Ctx, date: string) => date > ctx.upload.date || !!ctx.world.coverageIncompleteFrom && date >= ctx.world.coverageIncompleteFrom || ctx.world.coverageGaps.includes(weekId(date));

// A sleep-timing night is the evening of its date; it is met when the person
// marked a wind-down, or (bedtime goals) Garmin's sleep start is by the chosen time.
export function sleepMet(ctx: Ctx, date: string, g = goalsFor(ctx, date)): boolean | null {
  if (g.sleepKind === 'winddown') return ctx.state.winddown[date] ? true : date < ctx.calendarDate ? false : null;
  const n = ctx.nightOf(addDays(date, 1));
  if (!n) return null;
  const at = (t: string) => { const m = toMin(t); return m < 720 ? m + 1440 : m; };
  return at(n.bed) <= at(g.sleepTime) + 15;
}
const live = (ctx: Ctx, p: PlanItem) => !['released', 'missed'].includes(itemMatch(ctx, p).state);
const plannedOn = (ctx: Ctx, date: string, key: GoalKey) => planItems(ctx, date, date).filter(p => live(ctx, p)).reduce((n, p) => n + (p.allocation[key] ?? 0), 0);
/** Planned work still to do from today to `to`: not done, not under review, not passed. */
const stillPlanned = (ctx: Ctx, to: string, key: GoalKey) => planItems(ctx, ctx.calendarDate, to).filter(p => itemMatch(ctx, p).state === 'planned').reduce((n, p) => n + (p.allocation[key] ?? 0), 0);

export function dayRings(ctx: Ctx, date: string): {date: string, rings: Ring[], mark: DayMark, future: boolean, garmin: number, done: boolean, missed: boolean} {
  const g = goalsFor(ctx, date), future = date > ctx.calendarDate, mark = dayMark(ctx, date);
  const acts = ctx.world.activities.filter(a => a.date === date);
  const rings = activeKeys(g).map(key => {
    const planned = plannedOn(ctx, date, key);
    const recorded = future || mark === 'unknown' ? 0 : key === 'sleep' ? (planned && sleepMet(ctx, date, g) ? 1 : 0) : recordedFor(acts, key);
    const target = Math.max(planned, recorded);
    return {key, ...GOAL_INFO[key], recorded, planned, filled: target ? recorded / target : 0, outline: target ? 1 : 0};
  });
  const items = planItems(ctx, date, date).filter(p => p.kind === 'workout' && live(ctx, p));
  const states = items.map(p => itemMatch(ctx, p).state);
  return {date, rings, mark, future, garmin: items.filter(p => p.source === 'garmin').length,
   done: items.length > 0 && states.every(s => s === 'done'), missed: states.some(s => s === 'not-recorded')};
}

export function weekRings(ctx: Ctx, from = ctx.weekFrom) {
  const to = addDays(from, 6), end = to < ctx.calendarDate ? to : ctx.calendarDate, g = goalsFor(ctx, end);
  const acts = ctx.world.activities.filter(a => between(a.date, from, end));
  const rings: Ring[] = activeKeys(g).map(key => {
    const target = g[key];
    const recorded = key === 'sleep' ? days(from).filter(d => d < ctx.calendarDate && plannedOn(ctx, d, 'sleep') && sleepMet(ctx, d, g)).length : recordedFor(acts, key);
    const planned = stillPlanned(ctx, to, key);
    const filled = Math.min(1, recorded / target);
    return {key, ...GOAL_INFO[key], recorded, planned, filled, outline: Math.min(1, filled + planned / target)};
  });
  const scores = days(from).filter(d => d <= end).map(d => ctx.nightOf(d)?.score).filter((v): v is number => v != null);
  const score = g.sleepScore > 0 ? {target: g.sleepScore, average: scores.length ? Math.round(scores.reduce((a, b) => a + b, 0) / scores.length) : null, nights: scores.length} : null;
  return {rings, score, goals: g};
}
const days = (from: string) => Array.from({length: 7}, (_, i) => addDays(from, i));

// ---------- backlog ----------
export type BacklogItem = {key: GoalKey, amount: number, label: string, sport: Sport};
const CHUNK: Partial<Record<GoalKey, number>> = {zone2: 45, hi: 20, im: 30};
export function cardioSport(ctx: Ctx): Sport {
  const n = new Map<Sport, number>();
  for (const a of ctx.world.activities.slice(-40)) if (['running', 'cycling', 'walking', 'swimming', 'hiking'].includes(a.sport)) n.set(a.sport, (n.get(a.sport) ?? 0) + 1);
  return [...n.entries()].sort((a, b) => b[1] - a[1])[0]?.[0] ?? 'walking';
}
/** What's left of this week's goals once recordings and placed sessions are counted. */
export function backlog(ctx: Ctx): {items: BacklogItem[], paused: boolean, awaiting: boolean} {
  const g = goalsFor(ctx), paused = !!ctx.paused || dayMark(ctx, ctx.calendarDate) === 'ill';
  const acts = ctx.world.activities.filter(a => between(a.date, ctx.weekFrom, ctx.calendarDate));
  const awaiting = days(ctx.weekFrom).some(d => d <= ctx.calendarDate && unknownDay(ctx, d) && d <= ctx.upload.date) || !!ctx.world.coverageIncompleteFrom && ctx.calendarDate >= ctx.world.coverageIncompleteFrom;
  const items: BacklogItem[] = [];
  for (const key of activeKeys(g)) {
    const placed = stillPlanned(ctx, ctx.weekTo, key);
    if (key === 'sleep') {
      const nights = ctx.state.intentions.filter(i => isOpen(i) && i.kind === 'sleep' && between(i.date, ctx.weekFrom, ctx.weekTo)).length;
      const left = Math.max(0, g.sleep - nights);
      if (left) items.push({key, amount: left, label: `${left} sleep-timing night${left === 1 ? '' : 's'}`, sport: 'other'});
      continue;
    }
    const left = Math.max(0, g[key] - recordedFor(acts, key) - placed);
    if (!left) continue;
    if (key === 'strength') for (let i = 0; i < left; i++) items.push({key, amount: 1, label: 'Strength session', sport: 'strength'});
    else items.push({key, amount: left, label: `${left} min ${GOAL_INFO[key].name}`, sport: cardioSport(ctx)});
  }
  return {items, paused, awaiting};
}
/** The intention a backlog item becomes when taken for a day. */
export function takeItem(ctx: Ctx, item: BacklogItem, date: string): Omit<Intention, 'id' | 'link' | 'history'> {
  const g = goalsFor(ctx);
  if (item.key === 'sleep') return {sport: 'other', date, time: g.sleepTime, kind: 'sleep', name: `${g.sleepKind === 'winddown' ? 'Wind down' : 'Asleep by'} ${g.sleepTime}`, allocation: {sleep: 1}};
  const amount = item.key === 'strength' ? 1 : Math.min(item.amount, CHUNK[item.key] ?? item.amount);
  return {sport: item.sport, date, time: null, kind: 'workout', name: item.key === 'strength' ? undefined : `${SPORTS[item.sport].name} · ${amount} min ${GOAL_INFO[item.key].name}`, allocation: {[item.key]: amount}};
}

// ---------- week outcome and next week's proposal ----------
export type Outcome = {key: GoalKey, name: string, unit: string, target: number, recorded: number, state: 'met' | 'partly' | 'none' | 'not-counted'};
export type WeekOutcome = {from: string, counted: boolean, reasons: string[], ill: number, paused: number, rest: number, notRecorded: number, goals: Outcome[]};
export function weekOutcome(ctx: Ctx, from: string): WeekOutcome {
  const to = addDays(from, 6), g = goalsFor(ctx, to < ctx.calendarDate ? to : ctx.calendarDate);
  const marks = days(from).map(d => dayMark(ctx, d));
  const ill = marks.filter(m => m === 'ill').length, paused = marks.filter(m => m === 'paused').length, rest = marks.filter(m => m === 'rest').length;
  const gap = ctx.world.coverageGaps.includes(weekId(from)) || PLAN_WEEKS.some(w => w.from === from && w.state === 'unknown') || !!ctx.world.coverageIncompleteFrom && to >= ctx.world.coverageIncompleteFrom;
  const counted = !gap && ill + paused <= 1;
  const reasons = [gap && 'Garmin activity list incomplete', ill && `${ill} ill day${ill === 1 ? '' : 's'}`, paused && `paused ${paused} day${paused === 1 ? '' : 's'}`, rest && `${rest} rest day${rest === 1 ? '' : 's'}`].filter(Boolean) as string[];
  const acts = ctx.world.activities.filter(a => between(a.date, from, to));
  const notRecorded = planItems(ctx, from, to).filter(p => itemMatch(ctx, p).state === 'not-recorded').length;
  const goals = activeKeys(g).map(key => {
    const recorded = key === 'sleep' ? days(from).filter(d => plannedOn(ctx, d, 'sleep') && sleepMet(ctx, d, g)).length : recordedFor(acts, key);
    const state: Outcome['state'] = !counted ? 'not-counted' : recorded >= g[key] ? 'met' : recorded > 0 ? 'partly' : 'none';
    return {key, ...GOAL_INFO[key], target: g[key], recorded, state};
  });
  return {from, counted, reasons, ill, paused, rest, notRecorded, goals};
}

export type Proposal = {key: GoalKey, name: string, unit: string, current: number, proposed: number, reason: string};
const round5 = (n: number) => Math.round(n / 5) * 5;
export function proposeGoals(ctx: Ctx, nextFrom: string): Proposal[] {
  const lastFrom = addDays(nextFrom, -7), g = goalsFor(ctx, addDays(nextFrom, -1));
  const history = Array.from({length: 8}, (_, i) => weekOutcome(ctx, addDays(lastFrom, -7 * i))).filter(w => w.from >= weekStart(ctx.world.historyStart));
  const last = history[0];
  return activeKeys(g).map(key => {
    const current = g[key], minutes = key !== 'strength' && key !== 'sleep';
    const runs = history.filter(w => w.counted).map(w => w.goals.find(o => o.key === key)).filter((o): o is Outcome => !!o && o.target > 0);
    const out = (proposed: number, reason: string): Proposal => ({key, ...GOAL_INFO[key], current, proposed, reason});
    if (last && last.ill >= 2) return out(minutes ? Math.max(5, round5(current * .9)) : Math.max(1, current - 1), `Easing back after ${last.ill} ill days`);
    if (runs.length < 2) return out(current, 'Not enough complete weeks to suggest a change');
    const met = (o: Outcome) => o.recorded >= o.target;
    const comfortable = minutes ? runs[0].recorded >= runs[0].target * 1.1 : runs.slice(0, 3).length === 3 && runs.slice(0, 3).every(met);
    if (met(runs[0]) && met(runs[1]) && comfortable) {
      const up = minutes ? Math.min(current + 20, round5(current * 1.1)) : Math.min(5, current + 1);
      if (up > current) return out(up, minutes ? `Met in your last two complete weeks, ${runs[0].recorded} of ${runs[0].target} min last week` : 'Met in your last three complete weeks');
    }
    const low = runs.slice(0, 3).filter(o => minutes ? o.recorded < o.target * .7 : o.recorded < o.target).length;
    if (low >= 2) return out(minutes ? Math.max(5, round5(current * .9)) : Math.max(1, current - 1), `Under target in ${low} of your last ${Math.min(3, runs.length)} complete weeks`);
    return out(current, met(runs[0]) ? 'Met last week; keep it steady' : 'Close enough to keep');
  });
}

// ---------- the Plan your week prompt ----------
export function weekHasPlan(ctx: Ctx) {
  return activeKeys(goalsFor(ctx)).length > 0 && planItems(ctx, ctx.weekFrom, ctx.weekTo).some(p => live(ctx, p));
}
export function planPrompt(ctx: Ctx): null | {review: string | null} {
  const wk = weekId(ctx.calendarDate);
  if (ctx.state.planReminder === false || ctx.state.planWeeks?.[wk] || ['winddown', 'late', 'waiting'].includes(ctx.phase) || ctx.state.signedOut) return null;
  const lastFrom = addDays(ctx.weekFrom, -7), hadGoals = activeKeys(goalsFor(ctx, addDays(ctx.weekFrom, -1))).length > 0;
  const startOfWeek = ctx.calendarDate <= addDays(ctx.weekFrom, 1);
  if (weekHasPlan(ctx) && !startOfWeek) return null;
  if (weekHasPlan(ctx) && !hadGoals) return null;
  return {review: hadGoals && startOfWeek ? lastFrom : null};
}
export {candidates};

// ---------- an event in the week ----------
/** A dated event (Garmin race or local) and the sessions this week that serve it: Garmin's
 *  scheduled sessions and your sessions of the event's sport. Each still counts once per goal. */
export function eventWeek(ctx: Ctx) {
  const ev = [...ctx.world.ambitions, ...ctx.state.ambitionsAdded].filter(a => a.date && a.date >= ctx.calendarDate).sort((a, b) => a.date!.localeCompare(b.date!))[0];
  if (!ev) return null;
  const sport = ((ev as {sport?: Sport | null}).sport ?? 'running') as Sport;
  const items = planItems(ctx, ctx.weekFrom, ctx.weekTo).filter(p => p.kind === 'workout' && p.sport === sport && itemMatch(ctx, p).state !== 'released');
  const recordedMin = ctx.world.activities.filter(a => between(a.date, ctx.weekFrom, ctx.calendarDate) && a.sport === sport).reduce((t, a) => t + a.min, 0);
  const scheduledMin = items.filter(p => p.source === 'garmin').reduce((t, p) => t + (p.min ?? 0), 0);
  return {ev, sport, items, recordedMin, scheduledMin, days: Math.round((Date.parse(ev.date!) - Date.parse(ctx.calendarDate)) / 864e5)};
}
