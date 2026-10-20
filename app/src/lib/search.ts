import {metricNames} from './metrics';

/** A search result: a page, or a card on one (`anchor` is the card's element id). */
export type SearchEntry = {name: string; route: string; anchor?: string; also?: string[]};

/** Lowercase ASCII-ish words: NFKD turns SpO₂ into spo2 and drops accents. */
export const normalize = (text: string) => text.normalize('NFKD').replace(/\p{M}/gu, '').toLowerCase()
  .replace(/[’']/g, '').replace(/[^\p{L}\p{N}]+/gu, ' ').trim();

const PAGES: SearchEntry[] = [
  {name: 'Today', route: 'today'}, {name: 'Plan', route: 'plan'}, {name: 'Training', route: 'training'},
  {name: 'Health', route: 'health'}, {name: 'Progress', route: 'progress'},
  {name: 'Last night’s sleep', route: 'sleep/last'}, {name: 'Readiness', route: 'readiness'},
  {name: 'Weekly goals', route: 'plan/commitments', also: ['targets']}, {name: 'Zone 2', route: 'plan'},
  {name: 'Intensity minutes', route: 'health', anchor: 'intensity', also: ['vigorous', 'moderate']},
  {name: 'Fitness Age', route: 'health', anchor: 'fitness-age'},
  {name: 'Garmin journal', route: 'health', anchor: 'journal', also: ['lifestyle logs']},
  {name: 'Strength', route: 'strength', also: ['lifting', 'sets']}, {name: 'Activities', route: 'training', anchor: 'training-sections', also: ['workouts']},
  {name: 'Gear', route: 'gear', also: ['shoes']},
  {name: 'Weight', route: 'health/weight', also: ['body weight', 'scale']},
  {name: 'Body fat', route: 'health/weight'}, {name: 'BMI', route: 'health/weight', also: ['body mass index']},
  {name: 'Weekly review', route: 'review/latest', also: ['check in']},
  {name: 'Settings', route: 'settings'}, {name: 'Notifications', route: 'settings', anchor: 'notifications'},
  {name: 'Export your data', route: 'settings', anchor: 'export', also: ['backup']},
  {name: 'Import settings', route: 'settings', anchor: 'import', also: ['restore', 'new phone']},
  {name: 'Help', route: 'help'}, {name: 'Profile', route: 'profile'},
];

const ALSO: Record<string, string[]> = {
  hrv: ['heart rate variability'], respiration: ['breathing', 'breaths'], spo2: ['blood oxygen', 'oxygen saturation', 'pulse ox'],
  skinTemp: ['temperature', 'skin temp'], vo2max: ['aerobic fitness'], stress: [], bodyBattery: ['energy'],
};

/**
 * Pages and metrics by name. `sleepHr` says what metric/restingHr currently shows: Health Status's
 * overnight Sleep HR, Garmin's resting HR, or unknown (null) until Health has been read. Garmin's
 * daily resting HR always lives on the heart-rate detail.
 */
export function searchIndex(sleepHr: boolean | null): SearchEntry[] {
  const metrics: SearchEntry[] = Object.entries(metricNames)
    .filter(([id]) => id !== 'restingHr' && id !== 'heartRate')
    .map(([id, name]) => ({name, route: 'metric/' + id, also: ALSO[id]}));
  const heart: SearchEntry[] = sleepHr === false
    ? [{name: 'Resting heart rate', route: 'metric/restingHr', also: ['rhr']}]
    : [{name: 'Resting heart rate (daily)', route: 'metric/heartRate', also: ['rhr']},
      ...(sleepHr ? [{name: 'Sleep HR', route: 'metric/restingHr', also: ['overnight heart rate', 'sleeping heart rate']}] : [])];
  return [...PAGES, ...heart, {name: 'Heart rate', route: 'metric/heartRate', also: ['all-day heart rate', 'hr', 'pulse']}, ...metrics];
}

/** Every word of the query must start a word of the name or a synonym. */
export function searchHits(query: string, index: SearchEntry[]): SearchEntry[] {
  const words = normalize(query).split(' ').filter(Boolean);
  if (!words.length) return index;
  return index.filter(entry => [entry.name, ...(entry.also ?? [])].some(text => {
    const have = normalize(text).split(' ');
    return words.every(w => have.some(h => h.startsWith(w)));
  }));
}
