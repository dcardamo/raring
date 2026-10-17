import {describe, expect, it} from 'vitest';
import {minuteClock, rhythmAxis, rhythmTicks} from './rhythm';

const night = (window: [number, number] | null, sleep: [number, number] | null) => ({date: '2025-03-12', status: null, tone: 'neutral' as const, window, sleep});

describe('rhythm timeline', () => {
  it('formats minutes around midnight', () => {
    expect(minuteClock(-90)).toBe('22:30');
    expect(minuteClock(0)).toBe('00:00');
    expect(minuteClock(1450)).toBe('00:10');
  });
  it('keeps a default overnight span and grows on whole hours', () => {
    expect(rhythmAxis([])).toEqual([-120, 480]);
    expect(rhythmAxis([night([-90, 360], [45, 435])])).toEqual([-120, 480]);
    expect(rhythmAxis([night(null, [-250, 600])])).toEqual([-300, 660]);
  });
  it('ticks every two hours, or wider on long nights', () => {
    expect(rhythmTicks([-120, 480])).toEqual([-120, 0, 120, 240, 360, 480]);
    expect(rhythmTicks([-180, 600])).toEqual([-180, 0, 180, 360, 540]);
    expect(rhythmTicks([-300, 840])).toEqual([-240, 0, 240, 480, 720]);
  });
});
