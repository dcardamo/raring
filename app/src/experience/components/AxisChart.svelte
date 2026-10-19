<script lang="ts" module>
 export type Point = {x: string, v: number | null, parts?: {v: number, color: string}[], mark?: 'ill' | 'paused' | 'unknown' | 'future', note?: string};
</script>
<script lang="ts">
 // A readable chart: y-axis with units, x labels, labelled values, and an
 // optional goal line, usual range band and hatched spans (ill, paused,
 // awaiting data). Gaps stay gaps; nothing is drawn between missing points.

 let {points, kind = 'bars', unit = '', goal = null, goalLabel = 'goal', band = null, bandLabel = 'usual', height = 150, color = '#5CA8FF', fmt = (v: number) => String(Math.round(v)), values = kind === 'bars', yMin, yMax, label, every = 0, line = null, lineColor = '#E4E8F0', cap}: {
  points: Point[], kind?: 'bars' | 'line' | 'dots', unit?: string, goal?: number | null, goalLabel?: string, band?: [number, number] | null, bandLabel?: string,
  height?: number, color?: string, fmt?: (v: number) => string, values?: boolean, yMin?: number, yMax?: number, label: string, every?: number, line?: (number | null)[] | null, lineColor?: string, cap?: number,
 } = $props();
 let w = $state(320);
 const L = 34, R = 8, T = 22, B = 20;
 let vals = $derived([...points.map(p => p.v), ...(line ?? [])].filter((v): v is number => v != null));
 let lo = $derived(yMin ?? (kind === 'bars' ? 0 : Math.floor(Math.min(...vals, ...(band ?? []), ...(goal != null ? [goal] : [])) * .97)));
 // Bounded scales (Body Battery, stress, scores, SpO₂) never run past their maximum.
 let hi = $derived(Math.min(cap ?? Infinity, yMax ?? Math.max(1, Math.ceil(Math.max(...vals, ...(band ?? []), ...(goal != null ? [goal] : [])) * 1.08))));
 let step = $derived((w - L - R) / Math.max(1, points.length));
 const y = (v: number) => T + (1 - (v - lo) / Math.max(1e-6, hi - lo)) * (height - T - B);
 const x = (i: number) => L + step * i + step / 2;
 let ticks = $derived([lo, lo + (hi - lo) / 2, hi]);
 let n = $derived(every || (points.length <= 8 ? 1 : Math.ceil(points.length / 5)));
 let path = $derived.by(() => { let d = '', pen = false; points.forEach((p, i) => { if (p.v == null) { pen = false; return; } d += `${pen ? 'L' : 'M'}${x(i).toFixed(1)},${y(p.v).toFixed(1)}`; pen = true; }); return d; });
 let last = $derived(points.findLastIndex(p => p.v != null));
 // An overlaid series (for example trend weight over scale weigh-ins).
 let line2 = $derived.by(() => { if (!line) return ''; let d = '', pen = false; line.forEach((v, i) => { if (v == null) { pen = false; return; } d += `${pen ? 'L' : 'M'}${x(i).toFixed(1)},${y(v).toFixed(1)}`; pen = true; }); return d; });
</script>
<div class="ax" bind:clientWidth={w}>
 <svg width={w} {height} role="img" aria-label={label}>
  <defs><pattern id="axh" width="5" height="5" patternUnits="userSpaceOnUse" patternTransform="rotate(45)"><rect width="1.5" height="5" fill="#A9C8FF" opacity=".35"/></pattern></defs>
  {#each ticks as t}<line x1={L} x2={w - R} y1={y(t)} y2={y(t)} stroke="rgba(255,255,255,.07)"/><text class="yt" x={L - 5} y={y(t) + 3.5} text-anchor="end">{fmt(t)}</text>{/each}
  <text class="yu" x="2" y="10">{unit}</text>
  {#if band}<rect x={L} width={w - L - R} y={y(band[1])} height={Math.max(1, y(band[0]) - y(band[1]))} fill="rgba(61,220,151,.09)"/><text class="bl" x={w - R - 2} y={y(band[1]) + 10} text-anchor="end">{bandLabel} {fmt(band[0])}–{fmt(band[1])}</text>{/if}
  {#each points as p, i}
   {#if p.mark === 'ill' || p.mark === 'paused'}<rect x={x(i) - step / 2 + 1} y={T} width={step - 2} height={height - T - B} fill="url(#axh)"/>{/if}
   {#if p.mark === 'unknown'}<rect x={x(i) - step / 2 + 1} y={T} width={step - 2} height={height - T - B} fill="none" stroke="#6B7689" stroke-dasharray="2 3"/>{/if}
   {#if kind === 'bars' && p.v != null}
    {#if p.parts}{#each p.parts as part, k}{@const below = p.parts.slice(0, k).reduce((s, q) => s + q.v, 0)}<rect x={x(i) - Math.min(14, step * .32)} width={Math.min(28, step * .64)} y={y(below + part.v)} height={Math.max(0, y(below) - y(below + part.v))} fill={part.color}/>{/each}
    {:else}<rect x={x(i) - Math.min(14, step * .32)} width={Math.min(28, step * .64)} y={y(p.v)} height={Math.max(p.v > 0 ? 2 : 0, y(lo) - y(p.v))} rx="3" fill={color}/>{/if}
    {#if values && p.v > 0}<text class="vl" x={x(i)} y={y(p.v) - 4} text-anchor="middle">{fmt(p.v)}</text>{/if}
   {/if}
   {#if kind === 'dots' && p.v != null}<circle cx={x(i)} cy={y(p.v)} r={line ? 2.6 : 3.5} fill={color} opacity={line ? .55 : 1}/>{/if}
   {#if i % n === 0 || i === points.length - 1}<text class="xt" x={i === points.length - 1 && x(i) > w - R - 22 ? w - 1 : x(i)} y={height - 5} text-anchor={i === points.length - 1 && x(i) > w - R - 22 ? 'end' : 'middle'}>{p.x}</text>{/if}
  {/each}
  {#if kind === 'line'}<path d={path} fill="none" stroke={color} stroke-width="2"/>{#if last >= 0}<circle cx={x(last)} cy={y(points[last].v!)} r="3.5" fill={color}/>{/if}{/if}
  {#if (kind !== 'bars') && last >= 0 && !line}<text class="vl" x={Math.min(w - R - 2, x(last))} y={y(points[last].v!) - 7} text-anchor={x(last) > w - 40 ? 'end' : 'middle'}>{fmt(points[last].v!)}</text>{/if}
  {#if line2}<path d={line2} fill="none" stroke={lineColor} stroke-width="2.4" stroke-linejoin="round"/>{/if}
  {#if goal != null}<line x1={L} x2={w - R} y1={y(goal)} y2={y(goal)} stroke="#E4E8F0" stroke-dasharray="4 3" stroke-width="1.2"/><text class="gl" x={w - R - 2} y={y(goal) - 4} text-anchor="end" paint-order="stroke" stroke="#0E1424" stroke-width="3">{goalLabel} {fmt(goal)}</text>{/if}
 </svg>
</div>
<style>
 .ax{width:100%;min-width:0}.ax svg{display:block}
 text{font-family:Inter,sans-serif}
 .yt,.xt{font-size:9.5px;fill:var(--text3)}.yu{font-size:9px;fill:var(--text3)}
 .vl{font:600 10px Outfit,Inter;fill:var(--text)}.gl{font:600 9.5px Inter;fill:var(--text2)}.bl{font:500 9px Inter;fill:#3DDC97}
</style>
