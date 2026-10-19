<script lang="ts">
 import {smoothPath} from '../lib/curve';
 // A small filled sparkline for a table row; no axes, so it sits next to its number.
 let {values, color = '#7FA8FF', w = 64, h = 22, label}: {values: (number | null)[], color?: string, w?: number, h?: number, label: string} = $props();
 let vs = $derived(values.filter((v): v is number => v != null));
 let lo = $derived(Math.min(...vs)), hi = $derived(Math.max(...vs));
 const y = (v: number) => hi === lo ? h / 2 : 2 + (1 - (v - lo) / (hi - lo)) * (h - 4);
 let pts = $derived(values.map((v, i) => [i / Math.max(1, values.length - 1) * w, v == null ? null : y(v)] as [number, number | null]));
 let d = $derived(smoothPath(pts, true));
 let first = $derived(pts.find(p => p[1] != null)), last = $derived(pts.findLast(p => p[1] != null));
</script>
{#if vs.length >= 2}<svg width={w} height={h} role="img" aria-label={label}><path d={`${d}L${last![0]},${h}L${first![0]},${h}Z`} fill={color} opacity=".18"/><path {d} fill="none" stroke={color} stroke-width="1.8"/></svg>{:else}<span class="nos" style:width={w + 'px'}></span>{/if}
<style>svg{display:block;flex:none}.nos{display:block}</style>
