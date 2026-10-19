<script lang="ts">
 import {store} from '../lib/store.svelte';
 import {dayRings} from '../lib/week';
 import {addDays, dayLabel, narrowDay} from '../lib/dates';
 import NestedRings from './NestedRings.svelte';
 // One cell per date: its number above nested goal rings (see the design's
 // "Day and week rings"). Tapping a day opens what was planned, recorded and left.
 let {selected, select, from, count = 7, lead = 0, size = 36}: {selected?: string, select?: (date: string) => void, from?: string, count?: number, lead?: number, size?: number} = $props();
 let start = $derived(from ?? store.ctx.weekFrom);
 let days = $derived(Array.from({length: count}, (_, i) => dayRings(store.ctx, addDays(start, i))));
 const click = (date: string) => select ? select(date) : store.openSheet('day', date);
 const MARK: Record<string, string> = {rest: 'rest day', ill: 'ill', paused: 'paused', unknown: 'awaiting data'};
 const describe = (d: ReturnType<typeof dayRings>) => [dayLabel(d.date), d.future ? 'future' : '', d.mark ? MARK[d.mark] : '',
  ...d.rings.filter(r => r.outline).map(r => `${r.name} ${d.future ? '' : `${r.recorded} `}${r.planned ? `${d.future ? '' : 'of '}${r.planned} planned` : 'recorded, unplanned'}`),
  d.garmin ? `${d.garmin} Garmin session${d.garmin === 1 ? '' : 's'}` : '', d.done ? 'all done' : d.missed ? 'a session not recorded' : ''].filter(Boolean).join(', ');
</script>
<div class="day-cells" style:--cell={size + 'px'}>
 {#if lead}{#each Array(lead) as _}<span></span>{/each}{/if}
 {#each days as d (d.date)}
  {@const outside = d.date < store.ctx.world.historyStart}
  <button class="dc" class:current={d.date === store.ctx.calendarDate} class:selected={selected === d.date} class:future={d.future} disabled={outside} onclick={() => click(d.date)} aria-label={describe(d)}>
   <span class="top"><span class="dow">{narrowDay(d.date)}</span><b class="n">{Number(d.date.slice(8))}</b>{#if d.garmin}<i class="gm" aria-hidden="true">◇</i>{/if}</span>
   {#if !outside}<NestedRings rings={d.rings} {size} mark={d.mark} future={d.future} done={d.done} passed={d.date < store.ctx.calendarDate}/>{/if}
  </button>
 {/each}
</div>
<style>
 .day-cells{display:grid;grid-template-columns:repeat(7,minmax(0,1fr));gap:3px}
 .dc{position:relative;min-width:0;padding:6px 0 7px;border:1px solid transparent;border-radius:14px;background:rgba(255,255,255,.03);color:var(--text);display:flex;flex-direction:column;align-items:center;gap:4px;cursor:pointer;min-height:44px}
 .dc:disabled{cursor:default;opacity:.3;background:none}
 .dc.current{background:rgba(255,255,255,.07);border-color:rgba(255,255,255,.24)}.dc.selected{border-color:var(--ink)}
 .dc.future{background:none}.dc.future b{color:var(--text3)}
 .top{display:flex;align-items:baseline;gap:3px;position:relative}.dow{font:600 9.5px Inter;color:var(--text3)}.top b{font-size:14px;font-weight:600}
 .gm{position:absolute;right:-11px;top:-1px;font-style:normal;font-size:9px;color:#BDAAFF}
 @container(max-width:330px){.day-cells{grid-template-columns:repeat(4,minmax(0,1fr));gap:6px}}
</style>
