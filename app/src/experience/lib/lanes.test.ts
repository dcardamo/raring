import {describe, expect, it} from 'vitest';
import {derive} from './rules';
import {fresh} from './store.svelte';
import {liftSessions, weekLanes, usualWeeks} from './lanes';
import {setsFor} from './world';
import {rangeDays} from '../components/RangePicker.svelte';
const goals = (t: object) => [{from: '2025-09-01', targets: {strength: 0, zone2: 0, hi: 0, im: 0, sleep: 0, sleepScore: 0, sleepTime: '22:15', sleepKind: 'winddown' as const, ...t}}];
const at = (extra = {}) => derive(fresh('mixed', {day: '2026-11-18', clock: 'day', ...extra}));
describe('lift volume', () => {
 it('sums weight × reps over weighted sets only and counts the rest as excluded', () => {
  const c = at(), a = c.world.activities.filter(x => x.sport === 'strength').at(-1)!;
  for (const log of setsFor(a)!) {
   const p = liftSessions(c, log.exercise, a.date).find(x => x.date === a.date)!;
   const weighted = log.sets.filter(([r, w]) => (w ?? 0) > 0 && (r ?? 0) > 0);
   expect(p.volume).toBe(weighted.length ? weighted.reduce((t, [r, w]) => t + r! * w!, 0) : null);
   expect(p.max).toBe(weighted.length ? Math.max(...weighted.map(([, w]) => w!)) : null);
   expect(p.excluded).toBe(log.sets.length - weighted.length);
  }
 });
});
describe('week lanes', () => {
 it('shows one lane per cardio goal kind, or plain cardio minutes, plus strength', () => {
  expect(weekLanes(at({goalVersions: goals({strength: 2})})).map(l => l.key)).toEqual(['cardio', 'strength']);
  expect(weekLanes(at({goalVersions: goals({zone2: 90, hi: 20, strength: 2})})).map(l => l.key)).toEqual(['zone2', 'hi', 'strength']);
 });
 it('draws nothing after today and a usual band only from complete weeks', () => {
  const c = at({goalVersions: goals({im: 150})}), l = weekLanes(c)[0];
  expect(l.cum.slice(3).every(v => v == null)).toBe(true);
  expect(usualWeeks(c)).not.toContain('2026-10-26');
  expect(usualWeeks(c)).not.toContain('2026-10-19');
  for (let i = 0; i < 7; i++) expect(l.usual!.lo[i]).toBeLessThanOrEqual(l.usual!.hi[i]);
 });
});
describe('range picker', () => {
 it('Max covers all downloaded history', () => {
  expect(rangeDays(0, '2026-11-18', '2025-09-01')).toBe(444);
  expect(rangeDays(14, '2026-11-18', '2025-09-01')).toBe(14);
 });
});
