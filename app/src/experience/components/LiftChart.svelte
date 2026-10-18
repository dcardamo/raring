<script lang="ts">
 import {dayMonth} from '../lib/dates';
 // Heaviest recorded set per session for one exercise; best points are starred.
 let {points, name, best, color = '#FF7DB8'}: {points: {date: string, weight: number}[], name: string, best: number, color?: string} = $props();
 const W = 300, H = 74;
 let lo = $derived(Math.min(...points.map(p => p.weight)) - 2), hi = $derived(Math.max(...points.map(p => p.weight)) + 2);
 const x = (i: number) => 6 + i / Math.max(1, points.length - 1) * (W - 12);
 const y = (v: number) => H - 8 - (v - lo) / (hi - lo || 1) * (H - 18);
 let d = $derived(points.map((p, i) => `${i ? 'L' : 'M'}${x(i).toFixed(1)},${y(p.weight).toFixed(1)}`).join(''));
</script>
<svg viewBox={`0 0 ${W} ${H + 14}`} class="lift" role="img" aria-label={`${name}: heaviest set per session from ${points[0].weight} to ${points.at(-1)!.weight} kg`}>
 <path d={`${d}L${x(points.length - 1)},${H}L${x(0)},${H}Z`} fill={color} opacity=".12"/>
 <path class="ln" {d} fill="none" stroke={color} stroke-width="2.2" stroke-linejoin="round"/>
 {#each points as p, i}{#if p.weight === best && (i === 0 || points[i - 1].weight < best)}<g transform={`translate(${x(i)} ${y(p.weight)})`}><circle r="6.5" fill="#FFD666" stroke="#0B0F1A" stroke-width="2"/><path d="M0-3.6l1.1 2.3 2.5.3-1.8 1.7.5 2.5L0 2l-2.3 1.2.5-2.5-1.8-1.7 2.5-.3z" fill="#0B0F1A"/></g>{:else}<circle cx={x(i)} cy={y(p.weight)} r="2.6" fill={color}/>{/if}{/each}
 <text x="4" y={H + 12} font-size="10" fill="#8E98AC">{dayMonth(points[0].date)}</text>
 <text x={W - 4} y={H + 12} font-size="10" fill="#8E98AC" text-anchor="end">{dayMonth(points.at(-1)!.date)}</text>
</svg>
<style>.lift{display:block;width:100%}.ln{stroke-dasharray:600;animation:draw 1s ease-out both}@keyframes draw{from{stroke-dashoffset:600}to{stroke-dashoffset:0}}@media (prefers-reduced-motion:reduce){.ln{animation:none}}</style>
