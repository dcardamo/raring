import {historyAxis} from './history';
import type {TrendMetric} from './generated/TrendMetric';

export function metricAxis(metric:TrendMetric|null|undefined,values:(number|null)[],band:[number,number]|null,unit='') {
  return ['readiness','sleepScore','stress','bodyBattery'].includes(metric??'')
    ? {scale:[0,100] as [number,number],ticks:[0,50,100]}
    : historyAxis(values,band,unit);
}

/** Plot geometry only: round ticks, a real zero for amounts, no data-derived verdict. */
export function plotAxis(values: (number | null)[], {zero = false, integer = false, min, max, cap, unit = ''}: {
  zero?: boolean; integer?: boolean; min?: number; max?: number; cap?: number; unit?: string;
} = {}): {scale: [number, number]; ticks: number[]} {
  const known = values.filter((v): v is number => v != null && Number.isFinite(v));
  let axis = historyAxis(known.length ? known : [0, 1], null, unit);
  if (zero) {
    // Padding below zero would force an unnecessarily tall amount domain.
    const required = Math.max(1, ...known);
    const steps = unit === 'min' ? [15,30,60,120,180,240,360,720,1440,2880]
      : [1,2,2.5,5].flatMap(m=>[-2,-1,0,1,2,3,4,5,6].map(e=>m*10**e)).sort((a,b)=>a-b);
    const step = steps.find(s=>(!integer||s>=1) && Math.ceil(required/s)<=4) ?? steps.at(-1)!;
    const high = Math.ceil(required/step)*step;
    axis = {scale:[0,high],ticks:Array.from({length:Math.round(high/step)+1},(_,i)=>+(i*step).toFixed(6))};
  }
  let lo = min ?? (zero ? 0 : axis.scale[0]);
  let hi = Math.min(cap ?? Infinity, max ?? axis.scale[1]);
  if (integer) {
    lo = Math.floor(lo);
    hi = hi <= 4 ? Math.ceil(hi) : Math.ceil(hi / 2) * 2;
  }
  if (hi <= lo) hi = lo + 1;
  const ticks = integer ? (hi - lo <= 4 ? Array.from({length: hi - lo + 1}, (_, i) => lo + i) : [lo, Math.round((lo + hi) / 2), hi])
    : [...new Set([lo, ...axis.ticks.filter(t => t > lo && t < hi), hi])];
  return {scale: [lo, hi], ticks};
}
