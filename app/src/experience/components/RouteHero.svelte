<script lang="ts">
 import type {HrSample} from '../../lib/generated/HrSample';
 import {colors} from '../../lib/charts';
 // A stylized, tile-free map: the recorded route over drawn streets and water,
 // coloured by heart-rate zone along the way. No network map tiles or places.
 let {route, samples = [], floors = [], w = 358, h = 190, compact = false, color = '#E4E8F0', label = 'Workout route'}: {
  route: [number, number][], samples?: HrSample[], floors?: number[], w?: number, h?: number, compact?: boolean, color?: string, label?: string,
 } = $props();
 const id = $props.id();
 let pts = $derived.by(() => {
  if (route.length < 2) return [] as [number, number][];
  const lat = route.reduce((a, p) => a + p[0], 0) / route.length, cos = Math.cos(lat * Math.PI / 180);
  const xs = route.map(p => p[1] * cos), ys = route.map(p => p[0]);
  const minX = Math.min(...xs), maxX = Math.max(...xs), minY = Math.min(...ys), maxY = Math.max(...ys);
  const pad = compact ? 10 : 26;
  const s = Math.min((w - pad * 2) / (maxX - minX || 1), (h - pad * 2) / (maxY - minY || 1));
  const ox = (w - (maxX - minX) * s) / 2, oy = (h - (maxY - minY) * s) / 2;
  return route.map((_, i) => [ox + (xs[i] - minX) * s, h - oy - (ys[i] - minY) * s] as [number, number]);
 });
 const zoneOf = (bpm: number) => { let z = 0; floors.forEach((f, i) => { if (bpm >= f) z = i + 1; }); return z; };
 // Split the path into runs of the same zone, by elapsed share of the route.
 let runs = $derived.by(() => {
  if (!pts.length) return [] as {d: string, c: string}[];
  if (!samples.length || floors.length !== 4) return [{d: pts.map((p, i) => `${i ? 'L' : 'M'}${p[0].toFixed(1)},${p[1].toFixed(1)}`).join(''), c: color}];
  const out: {d: string, c: string}[] = [];
  let cur = -1, d = '';
  pts.forEach((p, i) => {
   const s = samples[Math.min(samples.length - 1, Math.floor(i / pts.length * samples.length))];
   const z = zoneOf(s.bpm);
   if (z !== cur) { if (d) out.push({d: d + `L${p[0].toFixed(1)},${p[1].toFixed(1)}`, c: cur <= 0 ? color : colors.z[cur]}); d = `M${p[0].toFixed(1)},${p[1].toFixed(1)}`; cur = z; }
   else d += `L${p[0].toFixed(1)},${p[1].toFixed(1)}`;
  });
  if (d) out.push({d, c: cur <= 0 ? color : colors.z[cur]});
  return out;
 });
 let full = $derived(pts.map((p, i) => `${i ? 'L' : 'M'}${p[0].toFixed(1)},${p[1].toFixed(1)}`).join(''));
 // Deterministic decoration from the route's own shape.
 let seed = $derived(route.length ? Math.abs(Math.round(route[0][0] * 1e4 + route[Math.floor(route.length / 3)][1] * 1e4)) : 1);
 const rnd = (n: number) => ((Math.sin(seed * 9.13 + n * 77.7) + 1) / 2);
 let river = $derived(`M${-w},${h * (.55 + rnd(1) * .35)} C${w * .3},${h * (.3 + rnd(2) * .5)} ${w * .6},${h * (.6 + rnd(3) * .4)} ${w * 2},${h * (.35 + rnd(4) * .4)}`);
</script>
<svg class="routehero" class:compact viewBox={`0 0 ${w} ${h}`} role="img" aria-label={label} preserveAspectRatio="xMidYMid meet">
 <defs>
  <linearGradient id="fade{id}" x1="0" y1="0" x2="0" y2={h} gradientUnits="userSpaceOnUse"><stop offset=".55" stop-color="#0E1424" stop-opacity="0"/><stop offset="1" stop-color="#0E1424" stop-opacity={compact ? 0 : .85}/></linearGradient>
  <pattern id="grid{id}" width="34" height="34" patternUnits="userSpaceOnUse" patternTransform={`rotate(${-12 + rnd(5) * 24})`}><path d="M0 0H34M0 0V34" stroke="#B4BCCB" stroke-opacity=".07" stroke-width="1.2" fill="none"/><path d="M17 0V34" stroke="#B4BCCB" stroke-opacity=".035" stroke-width="1" fill="none"/></pattern>
 </defs>
 <rect x={-w} y={-h} width={w * 3} height={h * 3} fill="#0E1424"/>
 <rect x={-w} y={-h} width={w * 3} height={h * 3} fill="url(#grid{id})"/>
 <ellipse cx={w * (.15 + rnd(6) * .7)} cy={h * (.2 + rnd(7) * .5)} rx={w * .16} ry={h * .2} fill="#3DDC97" opacity=".05"/>
 <ellipse cx={w * (.2 + rnd(8) * .6)} cy={h * (.4 + rnd(9) * .5)} rx={w * .1} ry={h * .14} fill="#3DDC97" opacity=".04"/>
 <path d={river} stroke="#5CA8FF" stroke-opacity=".16" stroke-width={compact ? 10 : 18} fill="none" stroke-linecap="round"/>
 {#if pts.length}
  <path d={full} stroke="#0B0F1A" stroke-opacity=".9" stroke-width={compact ? 6 : 9} fill="none" stroke-linecap="round" stroke-linejoin="round"/>
  {#each runs as r}<path class="seg" d={r.d} stroke={r.c} stroke-width={compact ? 3.2 : 4.5} fill="none" stroke-linecap="round" stroke-linejoin="round"/>{/each}
  <circle cx={pts[0][0]} cy={pts[0][1]} r={compact ? 4 : 6.5} fill="#3DDC97" stroke="#0B0F1A" stroke-width="2.5"/>
  {#if !compact}<rect x={pts.at(-1)![0] - 5} y={pts.at(-1)![1] - 5} width="10" height="10" rx="2" fill="#EEF1F7" stroke="#0B0F1A" stroke-width="2"/>{/if}
 {:else}<text x={w / 2} y={h / 2} fill="#8E98AC" text-anchor="middle" font-size="12">No route recorded</text>{/if}
 <rect x={-w} y="0" width={w * 3} height={h * 2} fill="url(#fade{id})" pointer-events="none"/>
</svg>
<style>
 .routehero{display:block;width:100%;height:100%;background:#0E1424}
 .seg{animation:draw 1.1s ease-out both}
 @keyframes draw{from{opacity:0}}
 @media (prefers-reduced-motion:reduce){.seg{animation:none}}
</style>
