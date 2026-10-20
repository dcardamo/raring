<script lang="ts">
 import type {Snippet} from 'svelte';
 import type {Ring} from '../generated/Ring';
 import type {DayMark} from '../generated/DayMark';
 import {GOALS} from '../goals';
 import {ringRadii} from '../rings';
 // Concentric goal rings, outermost first. Only what was recorded is drawn;
 // planned work lives in the text and agenda, and an empty remainder is empty.
 // Marks (hatching, dots) sit over the rings; `dim` fades rings that didn't count.
 let {rings, size = 40, stroke = 3.2, gap = 1.4, mark = null, done = false, dim = false, children, label}: {
  rings: Ring[], size?: number, stroke?: number, gap?: number, mark?: DayMark | null, done?: boolean, dim?: boolean, children?: Snippet, label?: string,
 } = $props();
 const id = $props.id();
 const c = $derived(size / 2);
 let geo = $derived(ringRadii(size, rings.length, stroke, gap));
 let inner = $derived(Math.max(4, (geo.radii.at(-1) ?? c) - geo.stroke / 2 - 1));
</script>
<span class="nr" style:width={size + 'px'} style:height={size + 'px'} role={label ? 'img' : undefined} aria-label={label}>
 <svg width={size} height={size} viewBox={`0 0 ${size} ${size}`} aria-hidden="true">
  <defs><pattern id="h{id}" width="4" height="4" patternUnits="userSpaceOnUse" patternTransform="rotate(45)"><rect width="1.3" height="4" fill="#A9C8FF" opacity=".55"/></pattern></defs>
  <g opacity={dim ? .4 : 1}>{#each rings as r, i}
   {@const f = Math.min(1, r.fill) * 100}
   {#if f > 0}<circle cx={c} cy={c} r={geo.radii[i]} fill="none" stroke={GOALS[r.kind].color} stroke-width={geo.stroke} stroke-linecap={f < 100 ? 'round' : 'butt'} pathLength="100"
    stroke-dasharray={`${Math.max(2, f)} 100`} transform={`rotate(-90 ${c} ${c})`}/>{/if}
  {/each}</g>
  {#if mark === 'ill' || mark === 'paused'}<circle cx={c} cy={c} r={c - .5} fill={`url(#h${id})`}/>{/if}
  {#if mark === 'unknown'}<circle cx={c} cy={c} r={c - .7} fill="none" stroke="#9AA4B5" stroke-width="1.2" stroke-dasharray="1.5 2.5"/>{/if}
  {#if mark === 'rest'}<circle cx={c} cy={c} r={inner} fill="rgba(169,200,255,.14)"/>{/if}
 </svg>
 <span class="nc" style:width={inner * 2 + 'px'}>{#if mark === 'rest'}<svg width={Math.min(14, inner * 1.3)} height={Math.min(14, inner * 1.3)} viewBox="0 0 24 24" fill="#A9C8FF" aria-hidden="true"><path d="M20 14.5A8 8 0 0 1 9.5 4a8 8 0 1 0 10.5 10.5z"/></svg>{:else if done && !children}<svg width={Math.min(13, inner * 1.4)} height={Math.min(13, inner * 1.4)} viewBox="0 0 24 24" fill="none" stroke="#3DDC97" stroke-width="3.4" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M5 12.5l4.5 4.5L19 7.5"/></svg>{:else if children}{@render children()}{/if}</span>
</span>
<style>
 .nr{position:relative;display:inline-grid;place-items:center;flex:none}
 .nr>svg{position:absolute;inset:0}
 .nc{position:relative;display:grid;place-items:center;line-height:1;text-align:center}
</style>
