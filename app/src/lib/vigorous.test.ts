import {describe, expect, it} from 'vitest';
import type {VigorousWindow} from './generated/VigorousWindow';
import {averages, dayMonth, describe as spoken, targetLine, windowStart} from './vigorous';

const w = (over: Partial<VigorousWindow> = {}): VigorousWindow => ({end: '2025-10-16', weeks: [14, 31, 6, 27, 20, 30], today: 24, average: 128 / 6, beforeToday: 104 / 6, target: 75, ageAtTarget: 37.8, ...over});

describe('vigorous window words', () => {
  it('starts 41 days before its last day', () => {
    expect(windowStart(w())).toBe('2025-09-05');
    expect(dayMonth('2025-09-05')).toBe('5 Sept');
  });
  it('shows today moving the average only when the whole minutes differ', () => {
    expect(averages(w())).toEqual({before: '17', after: '21', moved: true});
    expect(averages(w({today: 1, beforeToday: 21.2}))).toEqual({before: '21', after: '21', moved: false});
    expect(averages(w({average: null, beforeToday: null}))).toBeNull();
  });
  it('names Garmin’s target and the age it gives there, or that it is met', () => {
    expect(targetLine(w(), 39)).toBe('At 75 a week, Garmin puts you at 37.8 · now 39');
    expect(targetLine(w({average: 80}), 39)).toBe('Garmin’s 75 a week met · Fitness Age 39');
    expect(targetLine(w({ageAtTarget: null}), null)).toBe('Garmin’s target is 75 a week');
  });
  it('speaks every block, gaps included', () => {
    expect(spoken(w({weeks: [14, null, 6, 27, 20, 30]}))).toBe('Vigorous minutes in six seven-day blocks from 5 Sept: 14, not known, 6, 27, 20, 30; the last 7 days 30, 24 of them today. Garmin’s target 75 a week');
    expect(spoken(w(), false)).not.toContain('today');
  });
});
