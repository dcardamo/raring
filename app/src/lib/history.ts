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

// Same levels as Garmin Training Readiness; not Sleep Score thresholds.
export const readinessTone = (value: number): Tone => value >= 75 ? 'good' : value >= 50 ? 'fair' : 'poor';
