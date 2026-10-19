<script lang="ts">
 import type {Snippet} from 'svelte';
 import type {Ring, DayMark} from '../lib/week';
 // Concentric goal rings, outermost first. A solid arc is recorded progress; a
 // dashed arc is planned work not recorded yet. Nothing is drawn for the rest,
 // so an empty remainder is empty on every ring.
 let {rings, size = 40, stroke = 3.2, gap = 1.4, mark = null, future = false, done = false, passed = false, children, label}: {
  rings: Ring[], size?: number, stroke?: number, gap?: number, mark?: DayMark, future?: boolean, done?: boolean, passed?: boolean, children?: Snippet, label?: string,
 } = $props();
 const id = $props.id();
 const c = $derived(size / 2);
 const rOf = (i: number) => c - stroke / 2 - .5 - i * (stroke + gap);
 let inner = $derived(Math.max(4, rOf(rings.length - 1) - stroke / 2 - 1));
 // Dashes along [from, to] of the ring (in percent of the circle).
 const dashes = (from: number, to: number, r: number) => {
  const per = Math.max(3, 340 / (2 * Math.PI * r)), out: [number, number][] = [];
  for (let a = from + per * .5; a < to - per * .3; a += per * 2) out.push([a, Math.min(per, to - a)]);
  return out;
 };
</script>
<span class="nr" style:width={size + 'px'} style:height={size + 'px'} role={label ? 'img' : undefined} aria-label={label}>
 <svg width={size} height={size} viewBox={`0 0 ${size} ${size}`} aria-hidden="true">
  <defs><pattern id="h{id}" width="4" height="4" patternUnits="userSpaceOnUse" patternTransform="rotate(45)"><rect width="1.3" height="4" fill="#A9C8FF" opacity=".55"/></pattern></defs>
  {#if mark === 'ill' || mark === 'paused'}<circle cx={c} cy={c} r={c - .5} fill={`url(#h${id})`}/>{/if}
  {#if mark === 'unknown'}<circle cx={c} cy={c} r={c - .7} fill="none" stroke="#6B7689" stroke-width="1" stroke-dasharray="1.5 2.5"/>{/if}
  {#each rings as r, i}
   {@const f = future ? 0 : Math.min(1, r.filled) * 100}{@const o = Math.min(1, r.outline) * 100}
   {#if f > 0}<circle class="solid" cx={c} cy={c} r={rOf(i)} fill="none" stroke={r.color} stroke-width={stroke} stroke-linecap={f < 100 ? 'round' : 'butt'} pathLength="100"
    stroke-dasharray={`${Math.max(2, f)} 100`} transform={`rotate(-90 ${c} ${c})`}/>{/if}
   {#if o > f + 1}{#each dashes(f, o, rOf(i)) as [s, len]}<circle class="planned" class:passed cx={c} cy={c} r={rOf(i)} fill="none" stroke={r.color} stroke-width={Math.max(1.2, stroke * .7)} pathLength="100"
    stroke-dasharray={`0 ${s} ${len} 100`} transform={`rotate(-90 ${c} ${c})`}/>{/each}{/if}
  {/each}
  {#if mark === 'rest'}<circle cx={c} cy={c} r={inner} fill="rgba(169,200,255,.14)"/>{/if}
 </svg>
 <span class="nc" style:width={inner * 2 + 'px'}>{#if mark === 'rest'}<svg width={Math.min(14, inner * 1.3)} height={Math.min(14, inner * 1.3)} viewBox="0 0 24 24" fill="#A9C8FF" aria-hidden="true"><path d="M20 14.5A8 8 0 0 1 9.5 4a8 8 0 1 0 10.5 10.5z"/></svg>{:else if done && !children}<svg width={Math.min(13, inner * 1.4)} height={Math.min(13, inner * 1.4)} viewBox="0 0 24 24" fill="none" stroke="#3DDC97" stroke-width="3.4" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M5 12.5l4.5 4.5L19 7.5"/></svg>{:else if children}{@render children()}{/if}</span>
</span>
<style>
 .nr{position:relative;display:inline-grid;place-items:center;flex:none}
 .nr>svg{position:absolute;inset:0}
 .nc{position:relative;display:grid;place-items:center;line-height:1;text-align:center}
 .planned{stroke-opacity:.8}.planned.passed{stroke-opacity:.3}
</style>
