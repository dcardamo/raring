// Calendar helpers for the prototype's synthetic world. Dates are local calendar
// days (YYYY-MM-DD); weeks start on Monday, as Garmin's intensity week does here.
const WD = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];
const MO = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];

export const parse = (iso: string) => { const [y, m, d] = iso.split('-').map(Number); return new Date(Date.UTC(y, m - 1, d)); };
export const iso = (dt: Date) => dt.toISOString().slice(0, 10);
export const addDays = (s: string, n: number) => { const dt = parse(s); dt.setUTCDate(dt.getUTCDate() + n); return iso(dt); };
export const daysBetween = (a: string, b: string) => Math.round((parse(b).getTime() - parse(a).getTime()) / 86400000);
export const weekday = (s: string) => parse(s).getUTCDay();
export const dayLabel = (s: string) => { const dt = parse(s); return `${WD[dt.getUTCDay()]} ${dt.getUTCDate()} ${MO[dt.getUTCMonth()]}`; };
export const shortDay = (s: string) => WD[parse(s).getUTCDay()];
export const narrowDay = (s: string) => WD[parse(s).getUTCDay()][0];
export const dayMonth = (s: string) => { const dt = parse(s); return `${dt.getUTCDate()} ${MO[dt.getUTCMonth()]}`; };
export const monthName = (s: string, long = false) => { const dt = parse(s); return long ? dt.toLocaleDateString('en-GB', {month: 'long', timeZone: 'UTC'}) : MO[dt.getUTCMonth()]; };
export const monthYear = (s: string) => { const dt = parse(s); return `${dt.toLocaleDateString('en-GB', {month: 'long', timeZone: 'UTC'})} ${dt.getUTCFullYear()}`; };
export const dur = (min: number | null | undefined) => min == null ? '—' : `${Math.floor(min / 60)}h ${String(Math.round(min) % 60).padStart(2, '0')}`;
export const hm = (min: number) => { const m = ((Math.round(min) % 1440) + 1440) % 1440; return `${String(Math.floor(m / 60)).padStart(2, '0')}:${String(m % 60).padStart(2, '0')}`; };
export const toMin = (t: string) => { const [h, m] = t.split(':').map(Number); return h * 60 + m; };
export const weekStart = (s: string) => { const dt = parse(s); const wd = (dt.getUTCDay() + 6) % 7; dt.setUTCDate(dt.getUTCDate() - wd); return iso(dt); };
export const monthStart = (s: string) => `${s.slice(0, 7)}-01`;
export const addMonths = (s: string, n: number) => { const dt = parse(monthStart(s)); dt.setUTCMonth(dt.getUTCMonth() + n); return iso(dt); };
export const monthEnd = (s: string) => addDays(addMonths(s, 1), -1);
export const weekId = (s: string) => {
  const dt = parse(s); const day = (dt.getUTCDay() + 6) % 7; dt.setUTCDate(dt.getUTCDate() - day + 3);
  const first = new Date(Date.UTC(dt.getUTCFullYear(), 0, 4));
  const n = 1 + Math.round(((dt.getTime() - first.getTime()) / 86400000 - 3 + ((first.getUTCDay() + 6) % 7)) / 7);
  return `${dt.getUTCFullYear()}-W${String(n).padStart(2, '0')}`;
};
export const weekRange = (from: string) => {
  const to = addDays(from, 6);
  return from.slice(5, 7) === to.slice(5, 7) ? `${Number(from.slice(8))}–${dayMonth(to)}` : `${dayMonth(from)}–${dayMonth(to)}`;
};
export const between = (d: string, a: string, b: string) => d >= a && d <= b;
export const range = (from: string, to: string) => { const out: string[] = []; for (let d = from; d <= to; d = addDays(d, 1)) out.push(d); return out; };
