<script lang="ts">
 import type {Gauge} from '../generated/Gauge';
 import {num} from '../format';
 import {toneColor} from '../charts';
 import Icon from './Icon.svelte';
 // Garmin's acute load as a dot on a bar, with Garmin's optimal range as a
 // neutral band behind it. With `open` the whole bar opens the load's history.
 let {load, open}: {load: Gauge, open?: () => void} = $props();
 // Garmin sends the range's bounds with decimals; they print whole.
 let band = $derived(load.band ? [Math.round(load.band[0]), Math.round(load.band[1])] : null);
 let range = $derived(band ? `; Garmin's optimal range ${band[0]}–${band[1]}` : '');
 const at = (v: number) => Math.min(100, v / load.max * 100);
</script>
{#snippet bar()}
 <div><small>Acute load · Garmin</small><b class="n">{num(load.value)}</b><span style:color={toneColor(load.tone)}>{load.label}</span>{#if open}<i class="go"><Icon name="chevron" size={14}/></i>{/if}</div>
 <div class="lbar" role="img" aria-label={`Acute load ${num(load.value)}${range}; scale to ${load.max}`}>
  {#if load.band}<i class="band" style:left={at(load.band[0]) + '%'} style:width={at(load.band[1]) - at(load.band[0]) + '%'}></i>{/if}<u style:left={at(load.value) + '%'} style:background={toneColor(load.tone)}></u></div>
 <div class="lticks"><span>0</span>{#if band}<span>Optimal {band[0]}–{band[1]}</span>{/if}<span>{load.max}</span></div>
{/snippet}
{#if open}<button class="load tap" onclick={open} aria-label={`Acute load ${num(load.value)}${load.label ? `, ${load.label}` : ''}${range}. Open its history`}>{@render bar()}</button>
{:else}<div class="load">{@render bar()}</div>{/if}
<style>
 .load{display:grid;gap:6px;width:100%}.load>div:first-child{display:flex;gap:10px;align-items:baseline}.load small{font-size:12px;color:var(--text2)}.load b{font-size:24px}.load span{font:600 13px Inter}
 .tap{min-height:44px;padding:0;border:0;background:none;color:var(--text);text-align:left}
 .go{display:inline-flex;margin-left:auto;align-self:center;color:var(--text2)}
 .lbar{position:relative;height:10px;border-radius:5px;background:rgba(255,255,255,.07)}.lbar .band{position:absolute;top:0;bottom:0;background:rgba(228,232,240,.22);border-radius:5px}
 .lbar u{position:absolute;top:50%;width:14px;height:14px;margin:-7px 0 0 -7px;border:2px solid var(--bg);border-radius:50%}
 .lticks{display:flex;flex-wrap:wrap;gap:0 8px;justify-content:space-between;font-size:11px;color:var(--text2)}
</style>
