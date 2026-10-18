import type {TrendMetric} from '../../lib/generated/TrendMetric';

// Every metric has one canonical detail route; `parent` is its cold-link Back target.
export const METRICS: Record<string, {name: string, parent: string, unit: string, source: string, trend: TrendMetric | null, shipped: boolean, area: string}> = {
  readiness: {name: 'Training Readiness', parent: 'readiness', unit: '', source: 'Garmin', trend: 'readiness', shipped: false, area: 'Readiness'},
  sleepScore: {name: 'Sleep score', parent: 'sleep', unit: '', source: 'Garmin', trend: 'sleepScore', shipped: false, area: 'Sleep'},
  sleepTime: {name: 'Time asleep', parent: 'sleep', unit: 'h', source: 'Garmin overnight duration', trend: 'sleepTime', shipped: false, area: 'Sleep'},
  sleepHr: {name: 'Sleep HR', parent: 'health', unit: 'bpm', source: 'Garmin overnight average', trend: 'sleepHr', shipped: false, area: 'Health'},
  bodyBattery: {name: 'Body Battery', parent: 'health', unit: '', source: 'Garmin · at last upload', trend: 'bodyBattery', shipped: true, area: 'Health'},
  stress: {name: 'Stress', parent: 'health', unit: '', source: 'Garmin daily average', trend: 'stress', shipped: true, area: 'Health'},
  heartRate: {name: 'All-day heart rate', parent: 'health', unit: 'bpm', source: 'Garmin resting HR (daily)', trend: 'restingHr', shipped: true, area: 'Health'},
  restingHr: {name: 'Resting heart rate', parent: 'health', unit: 'bpm', source: 'Garmin daily resting value', trend: 'restingHr', shipped: true, area: 'Health'},
  hrv: {name: 'HRV', parent: 'health', unit: 'ms', source: 'Garmin nightly average', trend: 'hrv', shipped: true, area: 'Health'},
  respiration: {name: 'Respiration', parent: 'health', unit: 'brpm', source: 'Garmin overnight', trend: 'respiration', shipped: true, area: 'Health'},
  spo2: {name: 'Pulse Ox', parent: 'health', unit: '%', source: 'Garmin overnight', trend: 'spo2', shipped: true, area: 'Health'},
  skinTemp: {name: 'Skin temperature', parent: 'health', unit: '°C', source: 'Garmin overnight change', trend: 'skinTemp', shipped: true, area: 'Health'},
  vo2max: {name: 'VO₂ max', parent: 'training', unit: '', source: 'Garmin estimate', trend: 'vo2max', shipped: true, area: 'Training'},
  endurance: {name: 'Endurance score', parent: 'training', unit: '', source: 'Garmin', trend: null, shipped: true, area: 'Training'},
  hill: {name: 'Hill score', parent: 'training', unit: '', source: 'Garmin', trend: null, shipped: true, area: 'Training'},
  runningTolerance: {name: 'Running tolerance', parent: 'training', unit: 'km', source: 'Garmin weekly', trend: null, shipped: true, area: 'Training'},
  acuteLoad: {name: 'Acute load', parent: 'training', unit: '', source: 'Garmin 7-day weighted load', trend: 'acuteLoad', shipped: false, area: 'Training'},
  fitnessAge: {name: 'Fitness Age', parent: 'health', unit: 'years', source: 'Garmin estimate', trend: 'fitnessAge', shipped: false, area: 'Health'},
  weight: {name: 'Weight', parent: 'health', unit: 'kg', source: 'Garmin weigh-ins', trend: null, shipped: false, area: 'Health'},
};
