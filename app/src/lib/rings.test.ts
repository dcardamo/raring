import {describe, expect, it} from 'vitest';
import {MIN_RADIUS, ringRadii} from './rings';

describe('ringRadii', () => {
 it('keeps the requested stroke when the rings fit', () => {
  const {stroke, radii} = ringRadii(92, 5, 6.5, 2);
  expect(stroke).toBe(6.5);
  expect(radii[0]).toBeCloseTo(42.25);
 });
 it.each([[28, 5], [30, 5], [34, 5], [36, 4], [36, 5]])('fits every ring at %i px with %i goals', (size, n) => {
  const {stroke, gap, radii} = ringRadii(size, n, 3.2, 1.4);
  expect(radii).toHaveLength(n);
  expect(Math.min(...radii)).toBeGreaterThanOrEqual(MIN_RADIUS - 1e-9);
  expect(stroke).toBeGreaterThanOrEqual(1);
  // Neighbouring rings never overlap.
  for (let i = 1; i < n; i++) expect(radii[i - 1] - radii[i]).toBeCloseTo(stroke + gap);
  expect(radii[0] + stroke / 2).toBeLessThanOrEqual(size / 2);
 });
});
