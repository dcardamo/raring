// What the header search can find: every metric and named detail, by area.
import {METRICS} from './metrics';
import {EXERCISES} from './world';

export type SearchEntry = {name: string, route: string, area: string};
export const SEARCH_INDEX: SearchEntry[] = [
  ...Object.entries(METRICS).filter(([id]) => id !== 'weight').map(([id, m]) => ({name: m.name, route: `metric/${id}`, area: m.area})),
  {name: 'Weight', route: 'health/weight', area: 'Health · private'},
  {name: 'Race predictions', route: 'training/performance', area: 'Training'}, {name: 'Thresholds (lactate, FTP)', route: 'training/performance', area: 'Training'},
  {name: 'Load focus', route: 'training', area: 'Training'}, {name: 'Gear (shoes, bikes)', route: 'gear', area: 'Training'},
  {name: 'Activities', route: 'activities', area: 'Training'}, {name: 'Muscle coverage', route: 'strength/muscles', area: 'Training · strength'},
  {name: 'Sleep', route: 'sleep', area: 'Health'}, {name: 'Naps', route: 'sleep/naps', area: 'Health · sleep'}, {name: 'Sleep stages', route: 'sleep/stages', area: 'Health · sleep'},
  {name: 'Sleep patterns', route: 'sleep/patterns', area: 'Health · sleep'}, {name: 'Sleep timing', route: 'sleep/timing', area: 'Health · sleep'},
  {name: 'Garmin journal', route: 'health/journal', area: 'Health · private'}, {name: 'Vitals', route: 'health/vitals', area: 'Health'},
  {name: 'Weekly goals', route: 'plan', area: 'Plan'}, {name: 'Weekly reviews', route: 'reviews', area: 'Progress'}, {name: 'Trends', route: 'trends', area: 'Progress'},
  {name: 'Export your data', route: 'settings/export', area: 'Settings'}, {name: 'Help', route: 'help', area: 'Settings'},
  ...EXERCISES.map(e => ({name: e.name, route: `strength/${e.id}`, area: 'Training · strength'})),
].sort((a, b) => a.name.localeCompare(b.name));
const SYNONYMS: Record<string, string> = {'pulse ox': 'spo2', spo2: 'pulse ox', vo2: 'vo₂', shoes: 'gear', bike: 'gear', hr: 'heart', temperature: 'skin', lifestyle: 'journal', backup: 'export', energy: 'body battery'};
export function search(q: string): SearchEntry[] {
  const t = q.trim().toLowerCase();
  if (!t) return [];
  const terms = [t, SYNONYMS[t]].filter(Boolean);
  return SEARCH_INDEX.filter(e => terms.some(x => `${e.name} ${e.area}`.toLowerCase().includes(x)));
}
