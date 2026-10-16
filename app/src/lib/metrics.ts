export const vitalNames: Record<string, string> = {
  restingHr: 'Resting heart rate', hrv: 'HRV', respiration: 'Respiration',
  spo2: 'SpO₂', skinTemp: 'Skin temperature',
};

export const metricNames: Record<string, string> = {
  ...vitalNames, bodyBattery: 'Body Battery', vo2max: 'VO₂ max',
  endurance: 'Endurance', hill: 'Hill score', runningTolerance: 'Running tolerance',
};

export const metricDescriptions: Record<string, string> = {
  bodyBattery: 'Garmin’s estimate of your energy reserves, from 0 to 100. Rest and sleep recharge it; activity and stress drain it. This is the latest synced reading, not a live watch connection.',
  restingHr: 'Your resting heart rate from Garmin. The shaded band shows your usual range, not a medical target.',
  hrv: 'Heart rate variability measures the variation between heartbeats during sleep. This is your nightly average against Garmin’s baseline, not your HRV status. Garmin’s Balanced status uses your seven-day average, so a single night can fall outside the baseline while your status stays Balanced.',
  respiration: 'Breaths per minute during sleep, as reported by Garmin. The shaded band shows your usual overnight range.',
  spo2: 'Garmin’s overnight blood oxygen estimate. Watch readings are not a medical diagnosis; fit and movement can affect them.',
  skinTemp: 'Garmin’s overnight skin-temperature change, not your absolute body temperature. Compare it with your usual range.',
  vo2max: 'Garmin’s estimate of the oxygen your body can use during exercise. The trend shows how your estimate has changed over time.',
  endurance: 'Garmin’s endurance score describes your ability to sustain prolonged effort across activities.',
  hill: 'Garmin’s hill score describes your ability to run uphill, based on your training and fitness.',
  runningTolerance: 'Garmin’s running tolerance is the running distance your body is currently prepared to handle. It is not a Pepometer training target.',
};

export function metricSource(key: string): 'today' | 'health' | 'training' {
  if (key === 'bodyBattery') return 'today';
  if (['vo2max', 'endurance', 'hill', 'runningTolerance'].includes(key)) return 'training';
  return 'health';
}
