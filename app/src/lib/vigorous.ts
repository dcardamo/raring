// Words and labels for Fitness Age's vigorous window. The sums are the core's;
// these only round and phrase them (Rules: Fitness Age window).
import type {VigorousWindow} from './generated/VigorousWindow';
import {addDays} from './history';
import {num, shortMonth} from './format';

/** The window's first day: its last day and the 41 before it. */
export const windowStart = (w: VigorousWindow) => addDays(w.end, -41);

/** "8 Sept": a block's first day as an axis label. */
export const dayMonth = (date: string) => `${Number(date.slice(8, 10))} ${shortMonth(date)}`;

/** The six-week average before today's minutes and with them, in whole minutes;
 * `moved` only when the rounded figures differ. */
export function averages(w: VigorousWindow): {before: string, after: string, moved: boolean} | null {
  if (w.average == null) return null;
  const after = num(w.average), before = w.beforeToday == null ? after : num(w.beforeToday);
  return {before, after, moved: before !== after};
}

/** Where Garmin puts the driver's target, and what it gives for meeting it. */
export function targetLine(w: VigorousWindow, age: number | null): string {
  if (w.average != null && w.average >= w.target) return `Garmin’s ${num(w.target)} a week met${age != null ? ` · Fitness Age ${num(age)}` : ''}`;
  if (w.ageAtTarget != null) return `At ${num(w.target)} a week, Garmin puts you at ${num(w.ageAtTarget, 1)}${age != null ? ` · now ${num(age)}` : ''}`;
  return `Garmin’s target is ${num(w.target)} a week`;
}

/** The chart's spoken summary: every block, oldest first, and the target. */
export function describe(w: VigorousWindow, live = true): string {
  const blocks = w.weeks.map(v => v == null ? 'not known' : num(v)).join(', ');
  const today = live && w.today != null && w.today > 0 ? `, ${num(w.today)} of them today` : '';
  return `Vigorous minutes in six seven-day blocks from ${dayMonth(windowStart(w))}: ${blocks}; the last 7 days ${num(w.weeks.at(-1))}${today}. Garmin’s target ${num(w.target)} a week`;
}
