<script lang="ts" module>
 export type Point = {x: string, v: number | null, parts?: {v: number, color: string}[], mark?: 'ill' | 'paused' | 'unknown' | 'future', note?: string, current?: boolean};
</script>
<script lang="ts">
 // A readable chart: y-axis with units, x labels, labelled values, and an
 // optional goal line (one value, labelled, or each point's own as a stepped
 // line the caller names),
 // usual range band and hatched spans (ill, paused, awaiting data). Gaps stay
 // gaps; nothing is drawn between missing points. A current (unfinished)
 // point is outlined; a measured zero is labelled at the baseline. The band's
 // label sits at the left, away from the last value's label at the right.

 let {points, kind = 'bars', unit = '', goal = null, goalLabel = 'goal', band = null, bandLabel = 'usual', height = 150, color = 'var(--amount)', fmt = (v: number) => String(Math.round(v)), values = kind === 'bars', yMin, yMax, label, every = 0, line = null, lineColor = 'var(--ink)', cap, goals = null, integer = false}: {
  points: Point[], kind?: 'bars' | 'line' | 'dots', unit?: string, goal?: number | null, goalLabel?: string, band?: [number, number] | null, bandLabel?: string,
  height?: number, color?: string, fmt?: (v: number) => string, values?: boolean, yMin?: number, yMax?: number, label: string, every?: number, line?: (number | null)[] | null, lineColor?: string, cap?: number, goals?: (number | null)[] | null, integer?: boolean,
 } = $props();
 import {declutter} from '../declutter';
 import {plotAxis} from '../plot';
 const patternId=$props.id();
 let w = $state(320);
 const L = 34, R = 8, T = 16, B = 20;
 let stepped = $derived((goals ?? []).filter((v): v is number => v != null));
 let vals = $derived([...points.map(p => p.v), ...(line ?? [])].filter((v): v is number => v != null));
 let axis = $derived(plotAxis([...vals, ...(band ?? []), ...(goal != null ? [goal] : []), ...stepped], {zero:kind==='bars',integer,min:yMin,max:yMax,cap}));
 let lo = $derived(axis.scale[0]);
 let step = $derived((w - L - R) / Math.max(1, points.length));
 const y = (v: number) => T + (1 - (v - lo) / Math.max(1e-6, top - lo)) * (height - T - B);
 const x = (i: number) => L + step * i + step / 2;
 let top = $derived(axis.scale[1]);
 // With nothing to plot the scale means nothing: the chart keeps its baseline and says so.
 let empty = $derived(!vals.length && !stepped.length && goal == null && !band);
 let ticks = $derived(empty ? [lo] : axis.ticks);
 let n = $derived(every || (points.length <= 8 ? 1 : Math.ceil(points.length / 5)));
 let path = $derived.by(() => { let d = '', pen = false; points.forEach((p, i) => { if (p.v == null) { pen = false; return; } d += `${pen ? 'L' : 'M'}${x(i).toFixed(1)},${y(p.v).toFixed(1)}`; pen = true; }); return d; });
 let last = $derived(points.findLastIndex(p => p.v != null));
 // An overlaid series (for example trend weight over scale weigh-ins).
 // Each point's own goal across its slot, stepping where it changes.
 let goalPath = $derived.by(() => { if (!goals) return ''; let d = '', prev: number | null = null; goals.forEach((g, i) => { if (g == null) { prev = null; return; } const a = x(i) - step / 2, b = x(i) + step / 2; d += prev == null ? `M${a.toFixed(1)},${y(g).toFixed(1)}` : `L${a.toFixed(1)},${y(prev).toFixed(1)}L${a.toFixed(1)},${y(g).toFixed(1)}`; d += `L${b.toFixed(1)},${y(g).toFixed(1)}`; prev = g; }); return d; });
 let line2 = $derived.by(() => { if (!line) return ''; let d = '', pen = false; line.forEach((v, i) => { if (v == null) { pen = false; return; } d += `${pen ? 'L' : 'M'}${x(i).toFixed(1)},${y(v).toFixed(1)}`; pen = true; }); return d; });
</script>
<div class="ax" bind:clientWidth={w} data-chart="axis" {@attach declutter}>
 {#if unit}<span class="yu">{unit}</span>{/if}
 <svg width={w} {height} role="img" aria-label={label}>
  <defs><pattern id={patternId} width="5" height="5" patternUnits="userSpaceOnUse" patternTransform="rotate(45)"><rect width="1.5" height="5" fill="var(--goal-sleep)" opacity=".35"/></pattern></defs>
  {#each ticks as t}<line x1={L} x2={w - R} y1={y(t)} y2={y(t)} stroke="var(--chart-grid)" stroke-width="var(--stroke-grid)"/>{#if !empty}<text class="yt" data-thin="2" x={L - 5} y={y(t) + 3.5} text-anchor="end">{fmt(t)}</text>{/if}{/each}
  {#if !vals.length}<text class="em" x={(L + w - R) / 2} y={(T + height - B) / 2} text-anchor="middle">No readings</text>{/if}
  {#if band}<rect x={L} width={w - L - R} y={y(band[1])} height={Math.max(1, y(band[0]) - y(band[1]))} fill="var(--chart-band)"/><text class="bl" x={L + 4} y={y(band[1]) + 10}>{bandLabel} {fmt(band[0])}–{fmt(band[1])}</text>{/if}
  {#each points as p, i}
   {#if p.mark === 'ill' || p.mark === 'paused'}<rect x={x(i) - step / 2 + 1} y={T} width={step - 2} height={height - T - B} fill={`url(#${patternId})`}/>{/if}
   {#if p.mark === 'unknown'}<rect x={x(i) - step / 2 + 1} y={T} width={step - 2} height={height - T - B} fill="none" stroke="#6B7689" stroke-dasharray="2 3"/>{/if}
   {#if kind === 'bars' && p.v != null}
    {#if p.parts}{#each p.parts as part, k}{@const below = p.parts.slice(0, k).reduce((s, q) => s + q.v, 0)}<rect x={x(i) - Math.min(14, step * .32)} width={Math.min(28, step * .64)} y={y(below + part.v)} height={Math.max(0, y(below) - y(below + part.v))} fill={part.color} opacity={p.current ? .55 : 1}/>{/each}
    {:else}<rect x={x(i) - Math.min(14, step * .32)} width={Math.min(28, step * .64)} y={y(p.v)} height={Math.max(p.v > 0 ? 2 : 0, y(lo) - y(p.v))} rx="3" fill={p.current ? 'none' : color} stroke={p.current ? color : undefined} stroke-width={p.current ? 1.5 : undefined} stroke-dasharray={p.current ? '3 2' : undefined}/>{/if}
    {#if values && p.v >= 0}<text class="vl" data-thin x={x(i)} y={y(p.v) - 4} text-anchor="middle">{fmt(Math.abs(p.v))}</text>{/if}
   {/if}
   {#if kind === 'dots' && p.v != null}<circle cx={x(i)} cy={y(p.v)} r={line ? 2.6 : 3.5} fill={color} opacity={line ? .55 : 1}/>{/if}
   {#if (points.length - 1 - i) % n === 0}<text class="xt" data-thin="1" x={i === points.length - 1 && x(i) > w - R - 22 ? w - 1 : x(i)} y={height - 5} text-anchor={i === points.length - 1 && x(i) > w - R - 22 ? 'end' : 'middle'}>{p.x}</text>{/if}
  {/each}
  {#if kind === 'line'}<path d={path} fill="none" stroke={color} stroke-width="var(--stroke-chart)"/>{#if last >= 0}<circle cx={x(last)} cy={y(points[last].v!)} r="3.5" fill={color}/>{/if}{/if}
  {#if (kind !== 'bars') && last >= 0 && !line}<text class="vl" x={Math.min(w - R - 2, x(last))} y={y(points[last].v!) - 7} text-anchor={x(last) > w - 40 ? 'end' : 'middle'}>{fmt(points[last].v!)}</text>{/if}
  {#if line2}<path d={line2} fill="none" stroke={lineColor} stroke-width="var(--stroke-chart)" stroke-linejoin="round"/>{/if}
  {#if goalPath}<path d={goalPath} fill="none" stroke="var(--ink)" stroke-dasharray="4 3" stroke-width="1.2"/>{/if}
  {#if goal != null}<line x1={L} x2={w - R} y1={y(goal)} y2={y(goal)} stroke="var(--ink)" stroke-dasharray="4 3" stroke-width="1.2"/><text class="gl" x={w - R - 2} y={y(goal) - 4} text-anchor="end" paint-order="stroke" stroke="var(--card)" stroke-width="3">{goalLabel} {fmt(goal)}</text>{/if}
 </svg>
</div>
<style>
 .ax{width:100%;min-width:0}.ax svg{display:block}
 text{font-family:Inter,sans-serif}
 .yt,.xt{font-size:11px;fill:var(--text2)}.yu{display:block;margin-bottom:2px;font-size:12px;color:var(--text2)}
 .vl{font:500 12px Outfit,Inter;fill:var(--text)}.em{font:500 12px Inter;fill:var(--text2)}.gl{font:600 12px Inter;fill:var(--text2)}.bl{font:500 12px Inter;fill:var(--text2)}
</style>
