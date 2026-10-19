import {describe, expect, it} from 'vitest';
import {derive} from './rules';
import {fresh} from './store.svelte';
import {changes, lastWeighIn, steadyWithin, trendSeries, weighIn} from './weight';
import {bodyBatteryFor, bodyBatteryHigh, nightFor, readinessFor, spo2For, stressAvgFor} from './world';
import {addDays} from './dates';
const at = (profile: 'mixed' | 'sparse' = 'mixed') => derive(fresh(profile, {day: '2026-11-18', clock: 'day'}));
describe('steady threshold', () => {
 it('reads moves inside scale noise as Steady, per window', () => {
  expect(steadyWithin(1)).toBe(.2); expect(steadyWithin(14)).toBe(.2);
  expect(steadyWithin(30)).toBeCloseTo(.214, 2); expect(steadyWithin(90)).toBeCloseTo(.643, 2);
  const c = changes(at());
  const one = c.rows.find(r => r.days === 1)!;
  expect(one.word).toBe('Steady');
  for (const r of c.rows) if (r.kg != null && Math.abs(r.kg) >= steadyWithin(r.days) + .05) expect(r.word).not.toBe('Steady');
 });
});
describe('bounded synthetic metrics', () => {
 it('stay inside their scales across history', () => {
  for (let d = '2025-09-01'; d <= '2026-11-22'; d = addDays(d, 1)) {
   for (const v of [bodyBatteryFor(d), bodyBatteryHigh(d), stressAvgFor(d)]) { expect(v).toBeGreaterThanOrEqual(0); expect(v).toBeLessThanOrEqual(100); }
   const n = nightFor(d); if (n) { expect(n.score).toBeLessThanOrEqual(100); expect(n.score).toBeGreaterThanOrEqual(0); }
   const r = readinessFor(d); if (r) expect(r.value).toBeLessThanOrEqual(100);
   expect(spo2For(d)).toBeLessThanOrEqual(100);
  }
 });
});
describe('trend weight', () => {
 it('moves a tenth of the way per daily weigh-in and holds between them', () => {
  const c = at(), s = trendSeries(c, '2026-11-18');
  let prev: string | null = null;
  for (let d = '2026-10-01'; d <= '2026-11-18'; d = addDays(d, 1)) {
   const w = weighIn(c, d), t = s.get(d)!, y = s.get(addDays(d, -1))!;
   if (w == null) expect(t).toBe(y);
   else if (prev === addDays(d, -1)) expect(t).toBeCloseTo(y + .1 * (w - y), 1);
   if (w != null) prev = d;
  }
 });
 it('damps a single heavy morning', () => {
  const c = at(), last = lastWeighIn(c)!, s = trendSeries(c, last);
  const jump = Math.abs(weighIn(c, last)! - s.get(addDays(last, -1))!);
  expect(Math.abs(s.get(last)! - s.get(addDays(last, -1))!)).toBeLessThanOrEqual(jump * .1 + .01);
 });
 it('measures changes up to the last weigh-in and never extrapolates', () => {
  const c = at('sparse'), r = changes(c);
  expect(r.last! <= '2026-10-30').toBe(true);
  expect(r.trend).toBe(trendSeries(c, r.last!).get(r.last!));
  const m = changes(at());
  for (const row of m.rows) if (row.kg != null && Math.abs(row.kg) <= .1) expect(row.word).toBe('Steady');
 });
 it('shows no change where the trend did not exist yet', () => {
  const c = derive(fresh('simple', {day: '2026-11-18', clock: 'day'}));
  const r = changes(c, [365]);
  expect(r.rows[0].kg).toBeNull();
 });
});
