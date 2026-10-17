// Synthetic long-range trends for the browser preview. The app reads them from the Rust core.
import type {TrendMetric} from '../generated/TrendMetric';
import type {TrendQuery} from '../generated/TrendQuery';
import type {TrendSeries} from '../generated/TrendSeries';
import type {TrendView} from '../generated/TrendView';
import {period, spans, trendNames, bucketFor, firstDay} from '../trends';
import today from './today.json';
import health from './health.json';

const SAMPLE_TODAY = '2025-10-16';
const shape: Record<TrendMetric, [base: number, swing: number, noise: number, unit: string, first: string]> = {
  readiness: [72, 8, 22, '', '2024-02-05'], sleepScore: [78, 6, 16, '', '2024-02-05'], sleepTime: [440, 25, 70, 'min', '2024-02-05'],
  hrv: [56, 4, 10, 'ms', '2024-02-05'], restingHr: [48, 3, 4, 'bpm', '2021-04-12'], sleepHr: [51, 3, 4, 'bpm', '2024-02-05'],
  respiration: [14.1, .4, .8, 'br/min', '2024-02-05'], spo2: [96, 1, 2, '%', '2024-02-05'], skinTemp: [0, .2, .5, '°C', '2024-02-05'],
  bodyBattery: [82, 6, 18, '', '2021-04-12'], stress: [30, 5, 12, '', '2021-04-12'], vo2max: [50, 2, 1, '', '2019-03-02'],
  acuteLoad: [560, 120, 160, '', '2024-02-05'], fitnessAge:[39,2,1,'years','2024-02-05'],
};
const allMetrics = Object.keys(shape) as TrendMetric[];

function hash(text: string) {
  let h = 2166136261;
  for (const c of text) h = Math.imul(h ^ c.charCodeAt(0), 16777619);
  return (h >>> 0) / 4294967296;
}

// A seasonal swing plus noise; every eleventh day and a few weeks in 2022 are missing.
function value(metric: TrendMetric, date: string): number | null {
  const [base, swing, noise, , first] = shape[metric];
  const age=Math.round((Date.parse(SAMPLE_TODAY)-Date.parse(date))/86400000);
  const local=metric==='readiness'?today.readinessTrend:metric==='stress'?today.stress.trend:health.vitals.readings.find(v=>v.kind===(metric==='sleepHr'?'restingHr':metric))?.trend;
  if(local && age>=0 && age<local.length)return local[local.length-1-age];
  if (date < first || date > SAMPLE_TODAY || hash(metric + date) < 1 / 11 || (date >= '2022-06-01' && date < '2022-07-15')) return null;
  if (metric === 'vo2max' && hash(date) > .12) return null;
  const t = Date.parse(date) / 86_400_000;
  const v = base + Math.sin(t / 58) * swing + (hash(date + metric) - .5) * noise;
  return metric === 'respiration' || metric === 'skinTemp' ? Math.round(v * 10) / 10 : Math.round(v);
}

function series(metric: TrendMetric, list: [string, string][]): TrendSeries {
  let total = 0, count = 0;
  const points = list.map(([start, end]) => {
    let sum = 0, days = 0;
    for (let d = start; d <= end; d = new Date(Date.parse(d + 'T12:00:00Z') + 86_400_000).toISOString().slice(0, 10)) {
      const v = value(metric, d);
      if (v != null) {sum += v; days++}
    }
    total += sum; count += days;
    return {start, end, value: days ? sum / days : null, days};
  });
  return {metric, name: trendNames[metric], unit: shape[metric][3], points, average: count ? total / count : null, days: count};
}

export function sampleTrend(query: TrendQuery): TrendView {
  const end = query.end ?? SAMPLE_TODAY;
  const validDate=(s:string)=>/^\d{4}-\d{2}-\d{2}$/.test(s) && Number.isFinite(Date.parse(s+'T12:00:00Z')) && new Date(s+'T12:00:00Z').toISOString().slice(0,10)===s;
  // The same bounds as the core: every supplied date from firstDay to today.
  for(const date of [query.end,query.start,query.comparisonStart]){
   if(date==null)continue;
   if(!validDate(date))throw Error('Invalid trend date: '+date);
   if(date>SAMPLE_TODAY)throw Error(`Choose a date no later than today (${SAMPLE_TODAY})`);
   if(date<firstDay)throw Error(`Choose a date from ${firstDay} on`);
  }
  if(query.range==='custom' && (!query.start || query.start>end))throw Error('Choose a block start on or before its end');
  const length=Math.round((Date.parse(end)-Date.parse(query.start ?? end))/86400000)+1;
  const {from, bucket, shift} = query.range==='custom'?{from:query.start??end,bucket:bucketFor(length),shift:length}:period(query.range, end, shape[query.metric][4]);
  if(query.previous && query.comparisonStart && shift && Date.parse(query.comparisonStart)+(shift-1)*86400000>Date.parse(SAMPLE_TODAY))throw Error(`The comparison block runs past today (${SAMPLE_TODAY}); choose an earlier start`);
  const list = spans(from, end, bucket);
  const now = series(query.metric, list);
  const offset=query.comparisonStart?Math.round((Date.parse(from)-Date.parse(query.comparisonStart))/86400000):(shift??0);
  const back = (date: string) => new Date(Date.parse(date + 'T12:00:00Z') - offset * 86_400_000).toISOString().slice(0, 10);
  const previous = query.previous && shift ? series(query.metric, list.map(([s, e]) => [back(s), back(e)])) : null;
  const overlay = query.overlay && query.overlay !== query.metric ? series(query.overlay, list) : null;
  return {
    range: query.range, bucket, from, to: end, series: now, overlay, previous,
    change: previous?.average != null && now.average != null ? now.average - previous.average : null,
    available: allMetrics,
  };
}
