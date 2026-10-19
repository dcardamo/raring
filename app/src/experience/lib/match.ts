// Plan items (your sessions and Garmin's scheduled workouts) and how each is
// matched to recordings. Rule (design, "Proposed rules: matching"): a plan and
// a recording on the same day and sport match automatically when each is the
// other's only candidate; the person can unlink. Otherwise Raring asks. A
// recording counts once toward each goal however it is matched.
import type {Activity, Schedule} from './world';
import type {Allocation, Ctx, Intention} from './rules';
import {weekId} from './dates';

export type PlanItem = {
  id: string, source: 'local' | 'garmin', date: string, sport: Intention['sport'], name: string | null, time: string | null,
  min: number | null, allocation: Allocation, kind: 'workout' | 'sleep', intention?: Intention, schedule?: Schedule,
};
export type ItemState = 'done' | 'review' | 'planned' | 'not-recorded' | 'awaiting' | 'released' | 'missed' | 'sleep-observation';
export type Match = {state: ItemState, activity: Activity | null, auto: boolean, candidates: Activity[]};

const localAllocation = (i: Intention): Allocation => i.allocation ?? (i.sport === 'strength' && i.kind !== 'sleep' ? {strength: 1} : {});
export function localItem(i: Intention): PlanItem {
  return {id: i.id, source: 'local', date: i.date, sport: i.sport, name: i.name ?? null, time: i.time, min: i.durationMin ?? null, allocation: localAllocation(i), kind: i.kind === 'sleep' ? 'sleep' : 'workout', intention: i};
}
export function garminItem(s: Schedule): PlanItem {
  return {id: s.id, source: 'garmin', date: s.date, sport: s.sport, name: s.name, time: null, min: s.min, allocation: s.plan ?? {}, kind: 'workout', schedule: s};
}
/** Every plan item in a date range. Older local allocations made for a Garmin session replace it, so nothing is planned twice. */
export function planItems(ctx: Ctx, from: string, to: string): PlanItem[] {
  const local = ctx.state.intentions.filter(i => i.date >= from && i.date <= to);
  const overlays = new Set(local.filter(i => i.garminId && (!i.status || i.status === 'open')).map(i => i.garminId));
  const garmin = ctx.world.schedule.filter(s => s.date >= from && s.date <= to && !overlays.has(s.id));
  return [...local.map(localItem), ...garmin.map(garminItem)];
}
export const itemById = (ctx: Ctx, id: string) => planItems(ctx, '0000-00-00', '9999-12-31').find(p => p.id === id);
const closed = (ctx: Ctx, p: PlanItem) => p.intention?.status === 'released' || p.intention?.status === 'missed' || p.source === 'garmin' && !!ctx.state.dismissed[`sched:${p.id}:${p.date}`];

/** Matches for one day: explicit links first, then unambiguous same-sport pairs. */
export function dayMatches(ctx: Ctx, date: string): Map<string, Match> {
  const items = planItems(ctx, date, date).filter(p => p.kind === 'workout' && !closed(ctx, p));
  const acts = ctx.world.activities.filter(a => a.date === date);
  const out = new Map<string, Match>();
  const taken = new Set<string>();
  for (const p of items) {
    const id = p.intention?.link ?? ctx.state.glinks?.[p.id];
    const a = id ? acts.find(x => x.id === id) : undefined;
    if (a) { out.set(p.id, {state: 'done', activity: a, auto: false, candidates: [a]}); taken.add(a.id); }
  }
  const open = items.filter(p => !out.has(p.id));
  const free = acts.filter(a => !taken.has(a.id));
  const cands = (p: PlanItem) => free.filter(a => a.sport === p.sport);
  const plansFor = (a: Activity) => open.filter(p => p.sport === a.sport);
  for (const p of open) {
    const c = cands(p);
    if (c.length === 1 && plansFor(c[0]).length === 1 && !ctx.state.unlinked?.[p.id]) { out.set(p.id, {state: 'done', activity: c[0], auto: true, candidates: c}); taken.add(c[0].id); }
  }
  const leftover = acts.filter(a => !taken.has(a.id));
  const unmatched = open.filter(p => !out.has(p.id));
  for (const p of unmatched) {
    const c = cands(p).filter(a => !taken.has(a.id));
    // Ask only when it is genuinely ambiguous: several candidates either way, an
    // unlinked auto-match, or on a passed day one endurance recording of another
    // sport and similar length as the only possible match for the only open plan.
    const swap = leftover.filter(a => a.sport !== p.sport && ENDURANCE.has(a.sport) && ENDURANCE.has(p.sport) && (!p.min || a.min >= p.min * .5 && a.min <= p.min * 2));
    const otherSport = date < ctx.calendarDate && !c.length && swap.length === 1 && unmatched.length === 1 && leftover.length === 1;
    if (date <= ctx.calendarDate && (c.length || otherSport)) out.set(p.id, {state: 'review', activity: null, auto: false, candidates: c.length ? c : swap});
  }
  return out;
}

const ENDURANCE = new Set<Intention['sport']>(['running', 'cycling', 'swimming', 'hiking']);

export function itemMatch(ctx: Ctx, p: PlanItem): Match {
  const none = (state: ItemState): Match => ({state, activity: null, auto: false, candidates: []});
  if (p.intention?.status === 'released' || p.intention?.status === 'missed') return none(p.intention.status);
  if (p.source === 'garmin' && ctx.state.dismissed[`sched:${p.id}:${p.date}`]) return none('released');
  if (p.kind === 'sleep') return none(p.date < ctx.calendarDate ? 'sleep-observation' : 'planned');
  const m = dayMatches(ctx, p.date).get(p.id);
  if (m) return m;
  if (p.date >= ctx.calendarDate) return none('planned');
  return none(ctx.world.coverageIncompleteFrom && p.date >= ctx.world.coverageIncompleteFrom || ctx.world.coverageGaps.includes(weekId(p.date)) ? 'awaiting' : 'not-recorded');
}
/** The one consistent word for each state. */
export const STATE_WORD: Record<ItemState, string> = {done: 'Done', review: 'Needs review', planned: 'Planned', 'not-recorded': 'Not recorded', awaiting: 'Awaiting data', released: 'Let go', missed: 'Marked missed', 'sleep-observation': 'Observed in Sleep'};
