import {describe, expect, it} from 'vitest';
import {render} from 'svelte/server';
import RouteHero from './RouteHero.svelte';
import type {HrSample} from '../../lib/generated/HrSample';

// WorkoutView has geographic points but no position timestamps. Even arrays
// with equal lengths cannot establish that a heart-rate sample belongs there.
const route: [number, number][] = [[47, 8], [47.001, 8.007], [47.009, 8.008], [47.01, 8], [47, 8]];
const color = '#36C9C6';
const samples = (length: number): HrSample[] => Array.from({length}, (_, i) => ({tS: i * i * 7, bpm: i % 2 ? 180 : 90}));
const trace = (html: string) => {
  const paths = [...html.matchAll(/<path\b[^>]*>/g)].map(m => m[0]);
  return paths.filter(p => /class="[^"]*\bseg\b/.test(p));
};

describe('recorded route evidence', () => {
  for (const compact of [false, true]) {
    it(`renders one ${compact ? 'compact' : 'full'} sport-coloured outline regardless of unaligned HR arrays`, () => {
      const baseline = trace(render(RouteHero, {props: {route, color, compact}}).body);
      expect(baseline).toHaveLength(1);
      expect(baseline[0]).toContain(`stroke="${color}"`);
      for (const length of [0, 1, route.length, 91]) {
        // Extra runtime props simulate an old caller. The component deliberately
        // accepts neither HR nor zone boundaries in its typed interface.
        const legacyInputs = {samples: samples(length), floors: [118, 136, 150, 163]};
        const html = render(RouteHero, {props: {route, color, compact, ...legacyInputs}}).body;
        expect(trace(html)).toEqual(baseline);
        expect(html).not.toMatch(/heart.rate|zone/i);
        const reversedInputs = {...legacyInputs, samples: [...legacyInputs.samples].reverse()};
        const reversed = render(RouteHero, {props: {route, color, compact, ...reversedInputs}}).body;
        expect(trace(reversed)).toEqual(baseline);
      }
    });
  }
  it('keeps the no-route state instead of fabricating a position from HR', () => {
    const legacyInputs = {samples: samples(30), floors: [118, 136, 150, 163]};
    const html = render(RouteHero, {props: {route: [], ...legacyInputs}}).body;
    expect(trace(html)).toHaveLength(0);
    expect(html).toContain('No route recorded');
  });
});
