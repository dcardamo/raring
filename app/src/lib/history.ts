import type {Tone} from './generated/Tone';

export type HistoryPoint = {date: string; value: number | null};

// Calendar arithmetic in UTC keeps local Garmin dates intact across DST changes.
export function historyPoints(values: (number | null)[], endDate: string, days: number): HistoryPoint[] {
  const end = new Date(endDate + 'T12:00:00Z');
  return Array.from({length: days}, (_, i) => {
    const day = new Date(end);
    day.setUTCDate(day.getUTCDate() - (days - 1 - i));
    const value = values[values.length - days + i] ?? null;
    return {date: day.toISOString().slice(0, 10), value: value != null && Number.isFinite(value) ? value : null};
  });
}

export function historyScale(values: (number | null)[], band: [number, number] | null): [number, number] {
  const present = [...values, ...(band ?? [])].filter((v): v is number => v != null && Number.isFinite(v));
  if (!present.length) return [0, 100];
  const low = Math.min(...present), high = Math.max(...present);
  const padding = Math.max((high - low) * .15, high === low ? 1 : .1);
  return [low - padding, high + padding];
}

/** The scale widened to round steps (whole hours for minutes) and its 2–5 ticks, low to high. */
export function historyAxis(values: (number | null)[], band: [number, number] | null, unit = ''): {scale: [number, number], ticks: number[]} {
  const [low, high] = historyScale(values, band);
  const steps = unit === 'min' ? [15, 30, 60, 120, 180, 240, 360, 720] : [1, 2, 2.5, 5].flatMap(m => [-2, -1, 0, 1, 2, 3, 4].map(e => m * 10 ** e)).sort((a, b) => a - b);
  const step = steps.find(s => Math.ceil(high / s - 1e-9) - Math.floor(low / s + 1e-9) <= 4) ?? steps.at(-1)!;
  const lo = Math.floor(low / step + 1e-9) * step, hi = Math.ceil(high / step - 1e-9) * step;
  const ticks = Array.from({length: Math.round((hi - lo) / step) + 1}, (_, i) => +(lo + i * step).toFixed(6));
  return {scale: [lo, hi], ticks};
}

// Same levels as Garmin Training Readiness; not Sleep Score thresholds.
export const readinessTone = (value: number): Tone => value >= 75 ? 'good' : value >= 50 ? 'fair' : 'poor';

// Calendar days after `date`, in UTC like historyPoints.
export function addDays(date: string, days: number): string {
  const day = new Date(date + 'T12:00:00Z');
  day.setUTCDate(day.getUTCDate() + days);
  return day.toISOString().slice(0, 10);
}
