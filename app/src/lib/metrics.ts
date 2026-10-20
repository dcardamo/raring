export const vitalNames: Record<string, string> = {
  restingHr: 'Resting heart rate', hrv: 'HRV', respiration: 'Respiration',
  spo2: 'SpO₂', skinTemp: 'Skin temperature',
};

export const metricNames: Record<string, string> = {
  ...vitalNames, bodyBattery: 'Body Battery', stress: 'Stress', heartRate: 'Heart rate', vo2max: 'VO₂ max',
  endurance: 'Endurance', hill: 'Hill score', runningTolerance: 'Running tolerance',
};

export const metricDescriptions: Record<string, string> = {
  bodyBattery: 'Garmin’s estimate of your energy reserves, from 0 to 100. Rest and sleep recharge it; activity and stress drain it.',
  stress: 'Garmin estimates stress from your heart rate and heart rate variability while you are still, on a scale of 0 to 100: 0–25 is rest, 26–50 low, 51–75 medium and 76–100 high. It cannot rate stress while you are moving or not wearing the watch; those times are gaps, never zero.',
  restingHr: 'Garmin’s daily resting heart rate. The shaded band is your usual range over the last 30 days.',
  heartRate: 'Garmin’s all-day heart-rate readings in beats per minute. Missing or off-wrist readings are gaps, not zero. The daily resting HR is Garmin’s value, distinct from the overnight Sleep HR in Health Status.',
  hrv: 'Heart rate variability measures the variation between heartbeats during sleep. This is your nightly average against Garmin’s baseline, not your HRV status. Garmin’s Balanced status uses your seven-day average, so a single night can fall outside the baseline while your status stays Balanced.',
  respiration: 'Breaths per minute during sleep, as reported by Garmin. The shaded band shows your usual overnight range.',
  spo2: 'Garmin’s overnight blood oxygen estimate. Watch fit and movement can affect it.',
  skinTemp: 'Garmin’s overnight skin-temperature change, not your absolute body temperature. Compare it with your usual range.',
  vo2max: 'Garmin’s estimate of the oxygen your body can use during exercise. The trend shows how your estimate has changed over time.',
  endurance: 'Garmin’s endurance score describes your ability to sustain prolonged effort across activities.',
  hill: 'Garmin’s hill score describes your ability to run uphill, based on your training and fitness.',
  runningTolerance: 'Garmin’s running tolerance is the running distance your body is currently prepared to handle. It is not a Raring training target.',
};

export function metricSource(key: string): 'today' | 'health' | 'training' {
  if (key === 'bodyBattery' || key === 'stress') return 'today';
  if (['vo2max', 'endurance', 'hill', 'runningTolerance'].includes(key)) return 'training';
  return 'health';
}
