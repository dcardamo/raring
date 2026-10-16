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

export function vitalSummary(vitals: VitalsCard) {
  const recorded = vitals.readings.filter(v => v.value != null);
  const compared = recorded.filter(v => v.normal != null);
  const outside = compared.filter(v => rangeStatus(v.value, v.normal).tone === 'poor').length;
  const complete = compared.length === vitals.readings.length && compared.length > 0;
  const headline = outside
    ? `${outside} outside your usual range`
    : complete ? 'Within your usual ranges'
    : compared.length ? `${compared.length} ${compared.length===1?'reading':'readings'} in range`
    : recorded.length ? 'Getting to know your usual' : 'Waiting for readings';
  const detail = `${recorded.length} of ${vitals.readings.length} recorded · ${compared.length} with a range`;
  const tone: Tone = outside ? 'fair' : complete ? 'good' : 'neutral';
  return {headline, detail, tone, outside};
}

export function limitingFactor(factors: Factor[]) {
  return factors.filter(f => f.percent != null && Number.isFinite(f.percent))
    .sort((a, b) => a.percent! - b.percent!)[0] ?? null;
}

export const vitalDecimals = (kind: VitalReading['kind']) => kind === 'skinTemp' || kind === 'respiration' ? 1 : 0;
export const vitalIcons = {restingHr: 'heart', hrv: 'hrv', respiration: 'lungs', spo2: 'drop', skinTemp: 'temp'} as const;
export const shortVitalNames = {restingHr: 'Resting HR', hrv: 'HRV', respiration: 'Respiration', spo2: 'SpO₂', skinTemp: 'Skin temp'} as const;
