<script lang="ts">
 import type {DayCell} from '../generated/DayCell';
 import {GOALS} from '../goals';
 import NestedRings from './NestedRings.svelte';
 // One cell per date: weekday and number above nested goal rings, then a row
 // of marks: outlined dots for sessions still planned, a muted hollow dot for a
 // session not recorded, ◇ for Garmin's and ☾ for a Garmin Coach rest day.
 // Tapping a day shows what was planned, recorded and left.
 // `within` dims days outside a period, such as the edges of a month's weeks.
 let {days, selected, select, size = 36, within, coachRest = []}: {days: DayCell[], selected?: string, select: (date: string) => void, size?: number, within?: [string, string], coachRest?: string[]} = $props();
 const weekday = (d: string) => new Date(d + 'T12:00:00Z').toLocaleDateString('en-GB', {weekday: 'short', timeZone: 'UTC'});
 const MARK: Record<string, string> = {rest: 'rest day', ill: 'ill', paused: 'paused', unknown: 'awaiting data'};
 const describe = (d: DayCell) => [new Date(d.date + 'T12:00:00Z').toLocaleDateString('en-GB', {weekday: 'long', day: 'numeric', month: 'short', timeZone: 'UTC'}),
  d.today ? 'today' : '', d.mark ? MARK[d.mark] : '',
  ...d.rings.filter(r => r.recorded || r.planned).map(r => `${GOALS[r.kind].name} ${d.future ? '' : `${r.recorded} `}${r.planned ? `${d.future ? '' : 'of '}${r.planned} planned` : 'recorded'}`),
  d.planned ? `${d.planned} session${d.planned === 1 ? '' : 's'} planned` : '',
  d.garmin ? `${d.garmin} Garmin session${d.garmin === 1 ? '' : 's'}` : '', coachRest.includes(d.date) ? 'Garmin Coach rest day' : '', d.done ? 'all done' : d.notRecorded ? 'a session not recorded' : ''].filter(Boolean).join(', ');
</script>
<div class="day-cells" style:--cell={size + 'px'}>
 {#each days as d (d.date)}
  <button class="dc" class:outside={!!within && (d.date < within[0] || d.date > within[1])} class:current={d.today} class:selected={selected === d.date} class:future={d.future} onclick={() => select(d.date)} aria-label={describe(d)} aria-pressed={selected === d.date}>
   <span class="top"><span class="dow">{weekday(d.date).slice(0, 1)}</span><b class="n">{Number(d.date.slice(8))}</b></span>
   <NestedRings rings={d.rings} {size} mark={d.mark} done={d.done}/>
   <span class="marks" aria-hidden="true">{#each Array(Math.min(3, d.planned ?? 0)) as _}<i class="pl"></i>{/each}{#if d.notRecorded}<i class="nr"></i>{/if}{#if d.garmin}<i class="gm">◇</i>{/if}{#if coachRest.includes(d.date)}<i class="gm">☾</i>{/if}</span>
  </button>
 {/each}
</div>
<style>
 .day-cells{display:grid;grid-template-columns:repeat(7,minmax(0,1fr));gap:3px}
 .dc{position:relative;min-width:0;padding:6px 0 7px;border:1px solid transparent;border-radius:14px;background:rgba(255,255,255,.03);color:var(--text);display:flex;flex-direction:column;align-items:center;gap:4px;min-height:44px}
 .dc.current{background:rgba(255,255,255,.07);border-color:rgba(255,255,255,.24)}.dc.selected{border-color:var(--ink)}
 .dc.outside{background:none}.dc.outside b{color:var(--text2)}.dc.outside :global(svg){opacity:.35}
 .dc.future{background:none}.dc.future b{color:var(--text3)}
 .top{display:flex;flex-direction:column;align-items:center;line-height:1.1}.dow{font:600 11px Inter;color:var(--text2)}.top b{font-size:14px;font-weight:600}
 .marks{display:flex;align-items:center;justify-content:center;gap:2px;height:11px;line-height:1}
 .marks i{display:block;width:6px;height:6px;border-radius:50%;box-sizing:border-box}.pl{border:1.5px solid var(--ink)}.nr{border:1.5px solid var(--text3);opacity:.8}
 .marks .gm{width:auto;height:auto;font-style:normal;font-size:11px;color:#BDAAFF}
</style>
