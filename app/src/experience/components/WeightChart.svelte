<script lang="ts">
 import {smoothPath} from '../lib/curve';
 // Scale weight as a faint line through weigh-ins, trend weight as the bold
 // line with a marker on each weigh-in; y axis in kg, dates below.
 let {points, color, fmt, label}: {points: {x: string, scale: number | null, trend: number | null}[], color: string, fmt: (v: number) => string, label: string} = $props();
 let w = $state(320);
 const H = 170, L = 36, R = 8, T = 10, B = 20;
 let vals = $derived(points.flatMap(p => [p.scale, p.trend]).filter((v): v is number => v != null));
 let lo = $derived(vals.length ? Math.floor((Math.min(...vals) - .3) * 2) / 2 : 0), hi = $derived(vals.length ? Math.ceil((Math.max(...vals) + .3) * 2) / 2 : 1);
 const x = (i: number) => L + (w - L - R) * (points.length > 1 ? i / (points.length - 1) : .5);
 const y = (v: number) => T + (1 - (v - lo) / Math.max(1e-6, hi - lo)) * (H - T - B);
 let ticks = $derived([lo, (lo + hi) / 2, hi]);
 // Join weigh-ins up to two steps apart; a longer gap breaks the scale line
 // rather than inventing a shape where nobody weighed in.
 let scaleD = $derived.by(() => {
  const out: [number, number | null][] = [];
  let run: number[] = [];
  const flush = () => { if (run.length > 2) out.push([0, null]); run = []; };
  points.forEach((p, i) => { if (p.scale == null) { run.push(i); return; } flush(); out.push([x(i), y(p.scale)]); });
  return smoothPath(out, false);
 });
 let trendD = $derived(smoothPath(points.map((p, i) => [x(i), p.trend == null ? null : y(p.trend)]), true));
 let marks = $derived(points.length <= 45);
 let every = $derived(Math.max(1, Math.ceil(points.length / 5)));
</script>
<div class="wch" bind:clientWidth={w}>
 <svg width={w} height={H} role="img" aria-label={label}>
  {#each ticks as t}<line x1={L} x2={w - R} y1={y(t)} y2={y(t)} stroke="rgba(255,255,255,.08)" stroke-dasharray="3 4"/><text class="yt" x={L - 5} y={y(t) + 3.5} text-anchor="end">{fmt(t)}</text>{/each}
  <path d={scaleD} fill="none" stroke={color} stroke-opacity=".45" stroke-width="1.8"/>
  <path d={trendD} fill="none" stroke={color} stroke-width="2.6"/>
  {#if marks}{#each points as p, i}{#if p.scale != null && p.trend != null}<circle cx={x(i)} cy={y(p.trend)} r="3.2" fill="#121827" stroke={color} stroke-width="1.8"/>{/if}{/each}{/if}
  {#each points as p, i}{#if i % every === 0 || i === points.length - 1}<text class="xt" x={x(i)} y={H - 4} text-anchor={i === points.length - 1 ? 'end' : i === 0 ? 'start' : 'middle'}>{p.x}</text>{/if}{/each}
 </svg>
</div>
<style>.wch{width:100%;min-width:0}svg{display:block}text{font-family:Inter,sans-serif}.yt,.xt{font-size:9.5px;fill:var(--text3)}</style>
