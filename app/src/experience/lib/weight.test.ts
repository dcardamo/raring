import {describe, expect, it} from 'vitest';
import {derive} from './rules';
import {fresh} from './store.svelte';
import {changes, lastWeighIn, trendSeries, weighIn} from './weight';
import {addDays} from './dates';
const at = (profile: 'mixed' | 'sparse' = 'mixed') => derive(fresh(profile, {day: '2026-11-18', clock: 'day'}));
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
  for (const row of m.rows) if (row.kg != null) expect(row.word).toBe(Math.abs(row.kg) < .1 ? 'Steady' : row.kg < 0 ? 'Losing' : 'Gaining');
 });
 it('shows no change where the trend did not exist yet', () => {
  const c = derive(fresh('simple', {day: '2026-11-18', clock: 'day'}));
  const r = changes(c, [365]);
  expect(r.rows[0].kg).toBeNull();
 });
});
