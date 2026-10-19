// Synthetic stand-ins for fields already stored in Garmin's activity payloads
// but not exposed by the core today (design: data provenance). Which fields
// exist depends on sport and watch; absent fields are omitted, never zero.
import {hrTrace, rand, vo2maxFor, type Activity} from './world';
import {addDays} from './dates';

export type Summary = {
  moderate: number, vigorous: number, intensityMin: number, bodyBatteryDiff: number | null,
  staminaStart: number | null, staminaEnd: number | null, recoveryHr: number | null, temperature: number | null,
  performanceCondition: number | null, elevationGain: number | null, elevationLoss: number | null,
  cadence: number | null, cadenceUnit: string, strideM: number | null, gctMs: number | null, verticalOscCm: number | null,
  avgPower: number | null, np: number | null, intensityFactor: number | null, ftp: number | null, maxPower20: number | null,
  avgRespiration: number | null, movingMin: number, vo2Estimate: number | null,
};
const endurance = (a: Activity) => ['running', 'cycling', 'walking', 'hiking', 'swimming'].includes(a.sport);

export function summaryFor(a: Activity): Summary {
  const r = (k: string) => rand(a.id + k);
  const vigorous = Math.round(a.im * (a.benefit === 'Recovery' ? 0 : a.anaerobic > 1.5 ? .35 : .2) / 2);
  const moderate = a.im - 2 * vigorous;
  const run = a.sport === 'running', ride = a.sport === 'cycling';
  const vo2Today = vo2maxFor(a.date), vo2Before = vo2maxFor(addDays(a.date, -1));
  return {
    moderate: Math.max(0, moderate), vigorous, intensityMin: a.im,
    bodyBatteryDiff: a.timerOnly ? null : -Math.max(2, Math.round(a.load / 9)),
    staminaStart: run || ride ? Math.round(88 + r('st') * 10) : null,
    staminaEnd: run || ride ? Math.round(40 + r('se') * 30 - a.load / 12) : null,
    recoveryHr: endurance(a) && a.sport !== 'walking' ? Math.round(18 + r('rhr') * 14) : null,
    temperature: a.sport === 'strength' ? null : Math.round(4 + r('tmp') * 9),
    performanceCondition: run || ride ? Math.round((r('pc') - .4) * 8) : null,
    elevationGain: a.km && a.sport !== 'swimming' ? Math.round(a.km * (6 + r('eg') * 14)) : null,
    elevationLoss: a.km && a.sport !== 'swimming' ? Math.round(a.km * (6 + r('el') * 14)) : null,
    cadence: run ? Math.round(164 + r('cad') * 14) : ride ? Math.round(82 + r('cad') * 10) : a.sport === 'walking' ? Math.round(108 + r('cad') * 10) : null,
    cadenceUnit: ride ? 'rpm' : 'spm',
    strideM: run ? Math.round((1 + r('str') * .25) * 100) / 100 : null,
    gctMs: run ? Math.round(238 + r('gct') * 30) : null,
    verticalOscCm: run ? Math.round((7.6 + r('vo') * 2) * 10) / 10 : null,
    avgPower: ride ? Math.round(160 + r('pw') * 60) : run ? Math.round(250 + r('pw') * 40) : null,
    np: ride ? Math.round(178 + r('np') * 60) : null,
    intensityFactor: ride ? Math.round((.68 + r('if') * .2) * 100) / 100 : null,
    ftp: ride ? 238 : null,
    maxPower20: ride ? Math.round(205 + r('m20') * 40) : null,
    avgRespiration: a.timerOnly ? null : Math.round(22 + r('resp') * 14),
    movingMin: Math.round(a.min * (a.sport === 'walking' || a.sport === 'hiking' ? .93 : .98)),
    vo2Estimate: (run || ride) && vo2Today != null && vo2Today !== vo2Before ? vo2Today : null,
  };
}

/** Time-aligned samples on one elapsed-time axis (the stored streams share timestamps). */
export function streamsFor(a: Activity) {
  if (!endurance(a) || !a.km) return null;
  const hr = hrTrace(a), speedKmh = a.km / (a.min / 60);
  return hr.map(({tS, bpm}, i) => {
    const t = tS / 60;
    const v = speedKmh * (1 + (rand(a.id + 'v' + i) - .5) * .12 + (bpm - a.avgHr) / 400);
    const elev = 420 + Math.sin(tS / 600 + rand(a.id + 'e') * 6) * 22 + Math.sin(tS / 170) * 6;
    return {t, bpm, speed: Math.max(1, Math.round(v * 10) / 10), elev: Math.round(elev)};
  });
}

/** Garmin's training-effect bands, with one plain line each. */
export function effectLine(kind: 'aerobic' | 'anaerobic', v: number) {
  const band = v >= 5 ? 'Overreaching' : v >= 4 ? 'Highly improving' : v >= 3 ? 'Improving' : v >= 2 ? 'Maintaining' : v >= 1 ? 'Some benefit' : 'No benefit';
  const what = kind === 'aerobic'
    ? (v >= 3 ? 'builds your aerobic base' : v >= 2 ? 'keeps your aerobic base' : 'little aerobic stimulus')
    : (v >= 3 ? 'builds hard-effort capacity' : v >= 2 ? 'keeps some hard-effort capacity' : 'little sprint or hard work');
  return {band, what};
}
