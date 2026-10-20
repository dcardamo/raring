import type {Tone} from './generated/Tone';
import type {TrendMetric} from './generated/TrendMetric';
import {colors,toneColor} from './charts';
import {readinessTone} from './history';

/**
 * How a chart colours a metric's readings. Garmin's levels (readiness, sleep score, stress, Body Battery's
 * display bucket) colour every reading; a usual range colours readings inside it good and outside it fair
 * (acute load: above Garmin's optimal range is poor); Fitness Age is good below your age and poor above it.
 * A metric without one of these, or a range metric whose band isn't shown, is neutral blue. Colour alone
 * carries it: charts draw no colour key.
 */
export type ChartScale = {
  tone: (value: number) => Tone;
  /** Values where the colour changes, low to high, for gradients. */
  edges: {value: number; below: Tone; above: Tone}[];
};

const NEUTRAL: ChartScale = {tone: () => 'neutral', edges: []};

function levels(edges: [number, number], tones: [Tone, Tone, Tone]): ChartScale {
  return {
    tone: v => v < edges[0] ? tones[0] : v < edges[1] ? tones[1] : tones[2],
    edges: [{value: edges[0], below: tones[0], above: tones[1]}, {value: edges[1], below: tones[1], above: tones[2]}],
  };
}

const RANGE_METRICS: TrendMetric[] = ['hrv', 'restingHr', 'sleepHr', 'respiration', 'spo2', 'skinTemp'];

export function chartScale(metric: TrendMetric | null | undefined, {band = null, age = null}: {band?: [number, number] | null; age?: number | null} = {}): ChartScale {
  switch (metric) {
    // Garmin Training Readiness levels; not Sleep Score thresholds.
    case 'readiness': return {...levels([50, 75], ['poor', 'fair', 'good']), tone: readinessTone};
    case 'sleepScore': return levels([60, 80], ['poor', 'fair', 'good']);
    // Garmin's stress categories: rest and low are good, medium fair, high poor.
    case 'stress': return levels([51, 76], ['good', 'fair', 'poor']);
    case 'bodyBattery': return levels([30, 70], ['poor', 'fair', 'good']);
    case 'acuteLoad':
      if (!band) return NEUTRAL;
      return {
        tone: v => v < band[0] ? 'fair' : v > band[1] ? 'poor' : 'good',
        edges: [{value: band[0], below: 'fair', above: 'good'}, {value: band[1], below: 'good', above: 'poor'}],
      };
    case 'fitnessAge':
      if (age == null) return NEUTRAL;
      return {
        tone: v => v < age ? 'good' : v > age ? 'poor' : 'fair',
        edges: [{value: age, below: 'good', above: 'poor'}],
      };
  }
  if (metric && RANGE_METRICS.includes(metric) && band) return {
    tone: v => v < band[0] || v > band[1] ? 'fair' : 'good',
    edges: [{value: band[0], below: 'fair', above: 'good'}, {value: band[1], below: 'good', above: 'fair'}],
  };
  return NEUTRAL;
}

/** A reading's chart colour: status colours, and neutral blue rather than ink for readings without a verdict. */
export const chartColor = (tone: Tone) => tone === 'neutral' ? colors.neutral : toneColor(tone);

/** Hard gradient stops down a plot whose domain is [low, high]: offset 0 is the top (high), 1 the bottom. */
export function gradientStops(scale: ChartScale, [low, high]: [number, number]): {offset: number; color: string}[] {
  if (!scale.edges.length || !(high > low)) return [];
  const at = (v: number) => (high - v) / (high - low);
  const stops = [{offset: 0, color: chartColor(scale.tone(high))}];
  // An edge on a bound still switches there, so the plot doesn't blend across it.
  for (const edge of [...scale.edges].sort((a, b) => b.value - a.value)) {
    if (edge.value > high || edge.value < low) continue;
    stops.push({offset: at(edge.value), color: chartColor(edge.above)}, {offset: at(edge.value), color: chartColor(edge.below)});
  }
  stops.push({offset: 1, color: chartColor(scale.tone(low))});
  return stops;
}
