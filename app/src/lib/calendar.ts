// Months as their first day (YYYY-MM-01), in UTC like history.ts so local
// Garmin dates never shift across a DST change.

/** The first day of the month `months` after (or before) `month`'s. */
export function shiftMonth(month: string, months: number): string {
  const day = new Date(month.slice(0, 7) + '-01T12:00:00Z');
  day.setUTCMonth(day.getUTCMonth() + months);
  return day.toISOString().slice(0, 10);
}

/** Every date of `month`'s month, first to last. */
export function monthDays(month: string): string[] {
  const [year, number] = month.split('-').map(Number);
  const count = new Date(Date.UTC(year, number, 0)).getUTCDate();
  return Array.from({length: count}, (_, i) => `${month.slice(0, 7)}-${String(i + 1).padStart(2, '0')}`);
}
