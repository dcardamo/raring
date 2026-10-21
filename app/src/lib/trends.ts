import type {TrendBucket} from './generated/TrendBucket';
import type {TrendMetric} from './generated/TrendMetric';
import type {TrendPoint} from './generated/TrendPoint';
import type {TrendRange} from './generated/TrendRange';
import {addDays, historyPoints} from './history';
import {duration, num} from './format';

export const ranges: {range: TrendRange; label: string; days?: number}[] = [
  {range: 'days7', label: '7D', days: 7}, {range: 'days14', label: '14D', days: 14}, {range: 'days30', label: '30D', days: 30},
  {range: 'days90', label: '90D', days: 90}, {range: 'year', label: '1Y'}, {range: 'all', label: 'Max'}, {range:'custom',label:'Dates'},
];

export const trendNames: Record<TrendMetric, string> = {
  readiness: 'Readiness', sleepScore: 'Sleep score', sleepTime: 'Time asleep', hrv: 'HRV', restingHr: 'Resting HR',
  sleepHr: 'Sleep HR', respiration: 'Respiration', spo2: 'SpO₂', skinTemp: 'Skin temp', bodyBattery: 'Body Battery high',
  stress: 'Stress', vo2max: 'VO₂ max', acuteLoad: 'Acute load', fitnessAge:'Fitness Age',
};

/** The core's lower bound for every trend date. */
export const firstDay = '1990-01-01';

export const trendDecimals = (metric: TrendMetric) => metric === 'respiration' || metric === 'skinTemp' ? 1 : 0;

// A sensible first pairing when someone turns the overlay on.
export const defaultOverlay = (metric: TrendMetric): TrendMetric =>
  metric === 'readiness' ? 'sleepTime' : metric === 'hrv' ? 'acuteLoad' : 'readiness';

/** Minutes read as hours and minutes; everything else as a number. */
export const trendValue = (value: number | null | undefined, unit: string, decimals = 0) =>
  unit === 'min' ? duration(value) : num(value, decimals);

/** The per-day values a screen already holds, as one-day points ending on `endDate`. */
export function localPoints(values: (number | null)[], endDate: string, days: number): TrendPoint[] {
  return historyPoints(values, endDate, days).map(p => ({start: p.date, end: p.date, value: p.value, days: p.value == null ? 0 : 1, partial: null}));
}

const parse = (date: string) => new Date(date + 'T12:00:00Z');
export function monday(date: string): string {
  return addDays(date, -((parse(date).getUTCDay() + 6) % 7));
}
const nextMonth = (date: string) => {
  const d = parse(date.slice(0, 7) + '-01');
  d.setUTCMonth(d.getUTCMonth() + 1);
  return d.toISOString().slice(0, 10);
};
const dayCount = (from: string, to: string) => Math.round((parse(to).getTime() - parse(from).getTime()) / 86_400_000) + 1;

// Same bucket rule as the Rust core: days up to 90, weeks up to two years, months beyond.
export const bucketFor = (days: number): TrendBucket => days <= 90 ? 'day' : days <= 731 ? 'week' : 'month';

export function spans(from: string, to: string, bucket: TrendBucket): [string, string][] {
  const out: [string, string][] = [];
  for (let start = from; start <= to;) {
    const next = bucket === 'day' ? addDays(start, 1) : bucket === 'week' ? addDays(start, 7) : nextMonth(start);
    const end = addDays(next, -1);
    out.push([start, end < to ? end : to]);
    start = next;
  }
  return out;
}

/** The first day and bucket of a range ending on `end`; All starts at `first`. */
export function period(range: TrendRange, end: string, first: string | null): {from: string; bucket: TrendBucket; shift: number | null} {
  const days = ranges.find(r => r.range === range)?.days;
  if (days) return {from: addDays(end, 1 - days), bucket: 'day', shift: days};
  if (range === 'year') return {from: addDays(monday(end), -51 * 7), bucket: 'week', shift: 364};
  const start = first && first < end ? first : end, bucket = bucketFor(dayCount(start, end));
  return {from: bucket === 'week' ? monday(start) : bucket === 'month' ? start.slice(0, 8) + '01' : start, bucket, shift: null};
}

const short = (date: string, options: Intl.DateTimeFormatOptions) => parse(date).toLocaleDateString('en-GB', {timeZone: 'UTC', ...options});

/** The selected point's date: a day, "Week of 22 Sep" or "July 2026". */
export function pointLabel(point: TrendPoint, bucket: TrendBucket): string {
  if (bucket === 'week') return 'Week of ' + short(point.start, {day: 'numeric', month: 'short',year:'numeric'});
  if (bucket === 'month') return short(point.start, {month: 'long', year: 'numeric'});
  return short(point.start, {weekday: 'short', day: 'numeric', month: 'short'});
}

/** Axis labels for the first and last point. */
export function axisLabel(point: TrendPoint, bucket: TrendBucket): string {
  return bucket === 'day' ? short(point.start, {day: 'numeric', month: 'short'}) : short(point.start, {month: 'short', year: 'numeric'});
}

export function changeValue(value:number|null, unit:string, decimals:number):string {
  if(value==null)return '—';
  const magnitude=Math.abs(value);
  if(magnitude<1e-9)return '0';
  const sign=value<0?'−':'+';
  if(unit==='min')return sign+(magnitude<1?'<1 min':duration(magnitude));
  return sign+(magnitude<.1?'<0.1':num(magnitude,Math.max(1,decimals)));
}

export const bucketDays = (point: TrendPoint) => dayCount(point.start, point.end);
