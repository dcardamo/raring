<script lang="ts">
 import Icon from '../../lib/components/Icon.svelte';
 import {SPORTS} from '../lib/sports';
 import {dayLabel} from '../lib/dates';
 import type {calendarMonth} from '../lib/progress';
 // A month of sport-coloured marks. Chosen rest is rest, paused days are hatched,
 // unknown coverage is dotted; days without a record are just quiet.
 let {month, onDay}: {month: ReturnType<typeof calendarMonth>, onDay: (d: string) => void} = $props();
</script>
<div class="cal" role="grid" aria-label="Activity calendar">
 <div class="row hdr" role="row">{#each ['M', 'T', 'W', 'T', 'F', 'S', 'S'] as d}<span role="columnheader">{d}</span>{/each}</div>
 <div class="grid">
  {#each Array(month.lead) as _}<span></span>{/each}
  {#each month.days as d}
   <button class="day s-{d.state}" class:today={d.today} disabled={d.state === 'future' || d.state === 'before'} onclick={() => onDay(d.date)}
    aria-label={`${dayLabel(d.date)}: ${d.acts.length ? d.acts.map(a => SPORTS[a.sport].name).join(', ') : d.state === 'rest' ? 'rest day' : d.state === 'paused' ? 'paused' : d.state === 'unknown' ? 'awaiting evidence' : d.state === 'before' ? 'before downloaded history' : 'nothing recorded'}`}>
    <span class="num n">{Number(d.date.slice(8))}</span>
    {#if d.acts.length}
     <span class="dots">{#each d.acts.slice(0, 3) as a}<i style:background={SPORTS[a.sport].color}></i>{/each}</span>
    {:else if d.state === 'rest'}<span class="moon"><Icon name="moon" size={11}/></span>{/if}
   </button>
  {/each}
 </div>
</div>
<style>
 .row{display:grid;grid-template-columns:repeat(7,1fr);text-align:center;font:600 10px Inter;color:var(--text3);margin-bottom:4px}
 .grid{display:grid;grid-template-columns:repeat(7,1fr);gap:4px}
 .day{aspect-ratio:1;border-radius:10px;border:1px solid transparent;background:rgba(255,255,255,.03);display:flex;flex-direction:column;align-items:center;justify-content:center;gap:3px;color:var(--text2);padding:0;min-height:38px}
 .day .num{font-size:12px}
 .s-active{background:rgba(255,255,255,.07);color:var(--text)}
 .s-paused{background-image:repeating-linear-gradient(45deg,rgba(169,200,255,.14) 0 3px,transparent 3px 7px)}
 .s-unknown{background:none;border:1.5px dotted rgba(142,152,172,.55)}
 .s-rest{background:rgba(169,200,255,.08)}
 .s-future,.s-before{background:none;color:rgba(142,152,172,.4)}
 .today{border-color:var(--ink)}
 .dots{display:flex;gap:2px}
 .dots i{width:6px;height:6px;border-radius:50%}
 .moon{color:#A9C8FF;line-height:0}
</style>
