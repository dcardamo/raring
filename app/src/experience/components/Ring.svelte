<script lang="ts">
 import type {Snippet} from 'svelte';
 import Icon from '../../lib/components/Icon.svelte';
 // A commitment or count ring. States are told by shape as well as colour:
 // reached = closed ring with a check, paused = hatched, unknown = dotted.
 let {value, target, size = 56, stroke = 6, color = '#E4E8F0', state = 'open', label, children, center = true}: {
  value: number, target: number, size?: number, stroke?: number, color?: string, state?: string, label: string, children?: Snippet, center?: boolean,
 } = $props();
 const id = $props.id();
 let r = $derived(size / 2 - stroke / 2 - 1), c = $derived(2 * Math.PI * r);
 let frac = $derived(target > 0 ? Math.min(1, value / target) : 0);
 let reached = $derived(state === 'reached' || state === 'reached-adjusted');
 let col = $derived(reached ? '#3DDC97' : state === 'paused' || state === 'adjusted-paused' || state === 'unknown' ? '#8E98AC' : color);
</script>
<span class="ring" style:width={size + 'px'} style:height={size + 'px'} role="img" aria-label={label}>
 <svg width={size} height={size} viewBox={`0 0 ${size} ${size}`} aria-hidden="true">
  <defs><pattern id="rh{id}" width="4" height="4" patternUnits="userSpaceOnUse" patternTransform="rotate(45)"><rect width="1.4" height="4" fill="#8E98AC" opacity=".6"/></pattern></defs>
  {#if state === 'paused' || state === 'adjusted-paused'}<circle cx={size / 2} cy={size / 2} r={r} fill="none" stroke="url(#rh{id})" stroke-width={stroke}/>
  {:else}<circle cx={size / 2} cy={size / 2} r={r} fill="none" stroke="rgba(255,255,255,.09)" stroke-width={stroke} stroke-dasharray={state === 'unknown' ? '2 4' : undefined}/>{/if}
  {#if frac > 0 && state !== 'paused'}<circle class="arc" cx={size / 2} cy={size / 2} r={r} fill="none" stroke={col} stroke-width={stroke} stroke-linecap="round"
   stroke-dasharray={state === 'unknown' ? '2 4' : `${c * frac} ${c}`} transform={`rotate(-90 ${size / 2} ${size / 2})`} style:--len={c * frac}/>{/if}
 </svg>
 {#if center}<span class="rc">{#if children}{@render children()}{:else if reached}<Icon name="check" size={Math.round(size * .36)}/>{:else}<b class="n" style:font-size={Math.round(size * .3) + 'px'}>{value}</b>{/if}</span>{/if}
</span>
<style>
 .ring{position:relative;display:inline-grid;place-items:center;flex:none}
 .ring svg{position:absolute;inset:0}
 .rc{position:relative;display:grid;place-items:center;color:var(--text);line-height:1}
 .rc :global(svg){color:#3DDC97}
 .arc{animation:sweep .9s cubic-bezier(.3,.7,.3,1) both}
 @keyframes sweep{from{stroke-dasharray:0 1000}}
 @media (prefers-reduced-motion:reduce){.arc{animation:none}}
 :global(.quiet) .arc{animation:none}
</style>
