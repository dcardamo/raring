<script lang="ts">
 import type {Snippet} from 'svelte';
 import type {Ring, DayMark} from '../lib/week';
 // Concentric goal rings, outermost first. A thick arc is recorded progress; a
 // thin line is the planned remainder; no ring where nothing was planned.
 let {rings, size = 40, stroke = 3.2, gap = 1.4, mark = null, future = false, children, label}: {
  rings: Ring[], size?: number, stroke?: number, gap?: number, mark?: DayMark, future?: boolean, children?: Snippet, label?: string,
 } = $props();
 const id = $props.id();
 const c = $derived(size / 2);
 const rOf = (i: number) => c - stroke / 2 - .5 - i * (stroke + gap);
 let inner = $derived(Math.max(4, rOf(rings.length - 1) - stroke / 2 - 1));
</script>
<span class="nr" style:width={size + 'px'} style:height={size + 'px'} role={label ? 'img' : undefined} aria-label={label}>
 <svg width={size} height={size} viewBox={`0 0 ${size} ${size}`} aria-hidden="true">
  <defs><pattern id="h{id}" width="4" height="4" patternUnits="userSpaceOnUse" patternTransform="rotate(45)"><rect width="1.3" height="4" fill="#A9C8FF" opacity=".55"/></pattern></defs>
  {#if mark === 'ill' || mark === 'paused'}<circle cx={c} cy={c} r={c - .5} fill={`url(#h${id})`}/>{/if}
  {#if mark === 'unknown'}<circle cx={c} cy={c} r={c - .7} fill="none" stroke="#6B7689" stroke-width="1" stroke-dasharray="1.5 2.5"/>{/if}
  {#each rings as r, i}
   {#if r.outline > 0}
    <circle cx={c} cy={c} r={rOf(i)} fill="none" stroke={r.color} stroke-opacity=".55" stroke-width={Math.max(1, stroke * .38)} pathLength="100"
     stroke-dasharray={`${r.outline * 100} 100`} transform={`rotate(-90 ${c} ${c})`}/>
    {#if r.filled > 0 && !future}<circle cx={c} cy={c} r={rOf(i)} fill="none" stroke={r.color} stroke-width={stroke} stroke-linecap={r.filled < 1 ? 'round' : 'butt'} pathLength="100"
     stroke-dasharray={`${Math.max(2, r.filled * 100)} 100`} transform={`rotate(-90 ${c} ${c})`}/>{/if}
   {/if}
  {/each}
  {#if mark === 'rest'}<circle cx={c} cy={c} r={inner} fill="rgba(169,200,255,.14)"/>{/if}
 </svg>
 <span class="nc" style:width={inner * 2 + 'px'}>{#if mark === 'rest'}<svg width={Math.min(14, inner * 1.3)} height={Math.min(14, inner * 1.3)} viewBox="0 0 24 24" fill="#A9C8FF" aria-hidden="true"><path d="M20 14.5A8 8 0 0 1 9.5 4a8 8 0 1 0 10.5 10.5z"/></svg>{:else if children}{@render children()}{/if}</span>
</span>
<style>
 .nr{position:relative;display:inline-grid;place-items:center;flex:none}
 .nr>svg{position:absolute;inset:0}
 .nc{position:relative;display:grid;place-items:center;line-height:1;text-align:center}
</style>
