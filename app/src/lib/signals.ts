import type {Factor} from './generated/Factor';
import type {VitalReading} from './generated/VitalReading';
import type {VitalsCard} from './generated/VitalsCard';
import type {Tone} from './generated/Tone';

export function rangeStatus(value: number | null, range: [number, number] | null, kind?: VitalReading['kind']): {label: string; tone: Tone} {
  if (value == null) return {label: 'Not recorded', tone: 'neutral'};
  if (!range) return {label: 'Building baseline', tone: 'neutral'};
  if (value < range[0]) return {label: kind==='hrv'?'Below baseline':'Below usual', tone: 'poor'};
  if (value > range[1]) return {label: kind==='hrv'?'Above baseline':'Above usual', tone: 'poor'};
  return {label: 'Within range', tone: 'good'};
}

export function vitalStatus(v: VitalReading): {label:string; tone:Tone} {
  if (v.value == null) return {label:'Not recorded',tone:'neutral'};
  if (v.status === 'ONBOARDING') return {label:'Garmin baseline building',tone:'neutral'};
  if (v.status) {
    const label = ({IN_RANGE:'Within Garmin range',BELOW_RANGE:'Below Garmin range',ABOVE_RANGE:'Above Garmin range',OUT_OF_RANGE:'Outside Garmin range'} as Record<string,string>)[v.status];
    return label ? {label,tone:v.tone} : {label:'Garmin status unavailable',tone:'neutral'};
  }
  return rangeStatus(v.value,v.normal,v.kind);
}

export const vitalName = (v: VitalReading) => v.kind==='restingHr' && (v.status || v.rangeSource==='garmin') ? 'Sleep HR' : shortVitalNames[v.kind];
export const rangeName = (v: VitalReading) => v.rangeSource==='garmin' || v.status ? 'Garmin baseline' : v.rangeSource==='raring' ? 'Raring usual range' : 'Personal range';

export function vitalSummary(vitals: VitalsCard) {
  const recorded = vitals.readings.filter(v => v.value != null);
  const compared = recorded.filter(v => vitalStatus(v).tone !== 'neutral');
  const outside = recorded.filter(v => vitalStatus(v).tone === 'poor').length;
  const complete = compared.length === vitals.readings.length && compared.length > 0;
  const headline = outside
    ? `${outside} outside your ranges`
    : complete ? 'Within your ranges'
    : compared.length ? `${compared.length} ${compared.length===1?'reading':'readings'} in range`
    : recorded.length ? 'Getting to know your usual' : 'Waiting for readings';
  const detail = `${recorded.length} of ${vitals.readings.length} recorded · ${compared.length} comparable`;
  const tone: Tone = outside ? 'fair' : complete ? 'good' : 'neutral';
  return {headline, detail, tone, outside};
}

const ratingRank = {poor: 0, fair: 1, good: 2} as Record<string, number>;

/** The factor Garmin rates worst, its percentage breaking ties; unrated factors never count. */
export function limitingFactor(factors: Factor[]) {
  return factors.filter(f => f.tone in ratingRank)
    .sort((a, b) => ratingRank[a.tone] - ratingRank[b.tone] || (a.percent ?? Infinity) - (b.percent ?? Infinity))[0] ?? null;
}

export const vitalDecimals = (kind: VitalReading['kind']) => kind === 'skinTemp' || kind === 'respiration' ? 1 : 0;
export const vitalIcons = {restingHr: 'heart', hrv: 'hrv', respiration: 'lungs', spo2: 'drop', skinTemp: 'temp'} as const;
export const shortVitalNames = {restingHr: 'Resting HR', hrv: 'HRV', respiration: 'Respiration', spo2: 'SpO₂', skinTemp: 'Skin temp'} as const;
