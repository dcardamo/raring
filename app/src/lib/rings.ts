// Nested goal rings must all fit: the stroke and gap shrink together with the
// number of rings so the innermost ring keeps a radius of at least MIN_RADIUS.
export const MIN_RADIUS = 3;
export function ringRadii(size: number, count: number, stroke: number, gap: number): {stroke: number, gap: number, radii: number[]} {
 const n = Math.max(1, count);
 const ratio = Math.min(gap / stroke, .5);
 // Innermost centre: size/2 − .5 − s/2 − (n−1)·s·(1 + ratio) ≥ MIN_RADIUS.
 const fit = (size / 2 - .5 - MIN_RADIUS) / (n - .5 + (n - 1) * ratio);
 const s = Math.min(stroke, fit);
 const g = s * ratio;
 const radii = Array.from({length: count}, (_, i) => size / 2 - s / 2 - .5 - i * (s + g));
 return {stroke: s, gap: g, radii};
}
