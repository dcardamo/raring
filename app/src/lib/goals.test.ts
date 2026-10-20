import {describe, expect, it} from 'vitest';
import {backlogDraft, itemIcon} from './goals';

describe('backlogDraft', () => {
 const left = (kind: 'intensity' | 'zone2', amount: number) => backlogDraft({kind, amount, sport: null}, '2025-10-16').allocation;
 it('takes a share of what is left', () => expect(left('zone2', 120)).toEqual({zone2Min: 45}));
 it('takes the whole remainder rather than leaving a crumb', () => {
  expect(left('intensity', 32)).toEqual({intensityMin: 32});
  expect(left('intensity', 40)).toEqual({intensityMin: 30});
  expect(left('intensity', 20)).toEqual({intensityMin: 20});
  expect(left('intensity', 45)).toEqual({intensityMin: 30});
 });
});

describe('itemIcon', () => {
 it('gives Garmin Coach cardio a neutral cardio icon', () => {
  expect(itemIcon({intention: {sport: 'other'}, coach: {kind: 'cardio'}})).toBe('pulse');
  expect(itemIcon({intention: {sport: 'running'}, coach: null})).toBe('run');
 });
});
