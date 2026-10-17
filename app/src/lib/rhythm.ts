import type {AlignmentNight} from './generated/AlignmentNight';

// Alignment minutes count from the wake day's midnight; negative is the evening before.
export const minuteClock = (minutes: number) => {
  const m = ((Math.round(minutes) % 1440) + 1440) % 1440;
  return `${String(Math.floor(m / 60)).padStart(2, '0')}:${String(m % 60).padStart(2, '0')}`;
};

// An overnight axis on whole hours that holds every window and night, at least 22:00–08:00.
export function rhythmAxis(nights: AlignmentNight[]): [number, number] {
  const values = nights.flatMap(n => [...(n.window ?? []), ...(n.sleep ?? [])]).filter(Number.isFinite);
  const low = Math.min(-120, ...values.map(v => v - 30));
  const high = Math.max(480, ...values.map(v => v + 30));
  return [Math.floor(low / 60) * 60, Math.ceil(high / 60) * 60];
}

// Hour ticks every two, three or four hours, so labels never touch.
export function rhythmTicks([low, high]: [number, number]): number[] {
  const span = high - low;
  const step = span > 1080 ? 240 : span > 660 ? 180 : 120;
  const first = Math.ceil(low / step) * step;
  return Array.from({length: Math.floor((high - first) / step) + 1}, (_, i) => first + i * step);
}
