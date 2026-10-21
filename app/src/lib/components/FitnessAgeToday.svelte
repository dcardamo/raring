<script lang="ts">
 import type {TodayFitness} from '../generated/TodayFitness';
 import Card from './Card.svelte';
 import Icon from './Icon.svelte';
 import VigorousBars from './VigorousBars.svelte';
 import {num} from '../format';
 import {averages, targetLine} from '../vigorous';
 // Today's Fitness Age card, on a day that moved it: the vigorous minutes
 // Garmin's driver averages, with today's in them, or Garmin's new lower
 // estimate. It never credits the estimate to a workout.
 let {fitness: f, open}: {fitness: TodayFitness, open: () => void} = $props();
 let avg = $derived(averages(f.window));
</script>
<Card title="Fitness Age" onclick={open} extra="fa-today">
 {#if f.droppedFrom != null && f.age != null}<p class="fa-new"><Icon name="trend-down" size={15}/><span><b>New Garmin estimate: {num(f.age)}</b>, down from {num(f.droppedFrom)}</span></p>{/if}
 <div class="fa-row">
  <div class="fa-num">
   {#if avg}<div class="fa-big n">{#if avg.moved}<span class="was">{avg.before}</span><span class="arrow" aria-label="to">→</span>{/if}<b>{avg.after}</b><small>min/wk</small></div>{/if}
   <p>Vigorous minutes, six-week average{avg?.moved ? ', with today' : ''}</p>
  </div>
  <div class="fa-chart"><VigorousBars window={f.window} compact/></div>
 </div>
 <p class="fa-foot">{targetLine(f.window, f.age)}</p>
</Card>
<style>
 .fa-new{display:flex;gap:8px;align-items:flex-start;margin:-4px 0 10px;padding:8px 10px;border-radius:var(--radius-control);background:color-mix(in srgb,var(--good) 12%,transparent);color:var(--good);font-size:13px;line-height:1.35}
 .fa-new b{color:var(--text);font-weight:600}.fa-new span{color:var(--text2)}.fa-new :global(svg){flex:none;margin-top:1px}
 .fa-row{display:flex;flex-wrap:wrap;align-items:flex-end;gap:8px 14px;margin-top:-6px}
 .fa-num{flex:1 1 9.5em;min-width:0}
 .fa-big{display:flex;align-items:baseline;flex-wrap:wrap;gap:0 5px;font:600 28px/1.05 Outfit,Inter;color:var(--text)}
 .fa-big .was{color:var(--text2)}.fa-big .arrow{font-size:20px;color:var(--text3)}.fa-big b{font-weight:600}
 .fa-big small{font:500 13px Inter;color:var(--text3)}
 .fa-num p{margin-top:4px;font-size:12px;line-height:1.35;color:var(--text2)}
 .fa-chart{flex:1 1 8.5em;min-width:0;max-width:220px}
 .fa-foot{margin-top:10px;font-size:12.5px;line-height:1.4;color:var(--text2)}
</style>
