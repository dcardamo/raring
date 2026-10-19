// Trend weight: an exponentially smoothed average of scale weigh-ins, so
// water and food swings don't read as real change. Proposed rule (design,
// "Proposed rules: trend weight"); new Raring math, not a Garmin figure.
import {addDays} from './dates';
import {weightFor} from './world';
import type {Ctx} from './rules';

export const SMOOTHING = .1;
/** The last weigh-in Raring can see; the sparse profile stopped weighing in. */
export function weighIn(ctx: Ctx, date: string): number | null {
  if (date < ctx.world.historyStart || date > ctx.upload.date || ctx.state.profile === 'sparse' && date > '2026-10-30') return null;
  return weightFor(date);
}
export function lastWeighIn(ctx: Ctx): string | null {
  for (let d = ctx.upload.date; d >= ctx.world.historyStart; d = addDays(d, -1)) if (weighIn(ctx, d) != null) return d;
  return null;
}
/** Trend per day from the first weigh-in to `to`. A day without a weigh-in keeps
 *  the trend; a weigh-in after a gap of g days moves it by 1 − 0.9^g. */
export function trendSeries(ctx: Ctx, to: string): Map<string, number> {
  const out = new Map<string, number>();
  let t: number | null = null, gap = 0;
  for (let d = ctx.world.historyStart; d <= to; d = addDays(d, 1)) {
    const w = weighIn(ctx, d);
    gap++;
    if (w != null) { t = t == null ? w : t + (1 - (1 - SMOOTHING) ** gap) * (w - t); gap = 0; }
    if (t != null) out.set(d, Math.round(t * 100) / 100);
  }
  return out;
}
export type Change = {days: number, kg: number | null, perWeek: number | null, word: 'Losing' | 'Gaining' | 'Steady' | null};
/** Trend change over N days ending at the last weigh-in; never extrapolated past it. */
export function changes(ctx: Ctx, spans = [1, 3, 7, 14, 30, 90]): {last: string | null, trend: number | null, rows: Change[]} {
  const last = lastWeighIn(ctx);
  if (!last) return {last, trend: null, rows: spans.map(days => ({days, kg: null, perWeek: null, word: null}))};
  const series = trendSeries(ctx, last), end = series.get(last)!;
  return {last, trend: end, rows: spans.map(days => {
    const start = series.get(addDays(last, -days));
    if (start == null) return {days, kg: null, perWeek: null, word: null};
    const kg = Math.round((end - start) * 10) / 10;
    return {days, kg, perWeek: Math.round(kg / days * 7 * 100) / 100, word: Math.abs(kg) < .1 ? 'Steady' : kg < 0 ? 'Losing' : 'Gaining'};
  })};
}
export const ageDays = (from: string | null, to: string) => from == null ? null : Math.round((Date.parse(to) - Date.parse(from)) / 864e5);
