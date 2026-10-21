<script lang="ts">
 import {getContext} from 'svelte';
 import * as api from '../api';
 import type {StressMonth} from '../generated/StressMonth';
 import type {StressDay} from '../generated/StressDay';
 import {dateLabel,num} from '../format';
 import {toneColor} from '../charts';
 import Icon from './Icon.svelte';
 // A month of Garmin's word for each day, coloured without a key, with Garmin's rating of the night
 // before as a bar on top. `date` is the day the page shows; tapping another day opens it. Months come
 // from local data only.
 let {date,open}:{date:string,open:(date:string)=>void}=$props();
 const generation=getContext<()=>number>('trend-generation') ?? (()=>0);
 // Opening another day shows its month; the arrows move away from it.
 let month=$derived(date.slice(0,7)+'-01');
 let view=$state<StressMonth|null>(null),error=$state('');
 $effect(()=>{
  generation();
  const wanted=month;
  let cancelled=false;
  error='';
  api.stress_month(wanted).then(next=>{if(!cancelled)view=next}).catch(e=>{if(!cancelled)error=String(e)});
  return ()=>{cancelled=true};
 });
 const shift=(first:string,n:number)=>{const d=new Date(first+'T12:00:00Z');d.setUTCMonth(d.getUTCMonth()+n);return d.toISOString().slice(0,10)};
 const monthName=(first:string)=>new Date(first+'T12:00:00Z').toLocaleDateString('en-GB',{month:'long',year:'numeric',timeZone:'UTC'});
 let shown=$derived(view?.month===month?view:null);
 let lead=$derived(shown?(new Date(shown.month+'T12:00:00Z').getUTCDay()+6)%7:0);
 let current=$derived(view?view.today.slice(0,7)+'-01':month);
 let selected=$derived(shown?.days.find(d=>d.date===date)??null);
 const words=(d:StressDay)=>{
  const parts=[d.kind==='rated'?d.word??'':d.kind==='awakeOnly'?`awake only · Garmin's word for the time awake: ${d.word}`:d.kind==='lowWear'?'partly worn · Garmin couldn’t rate the day':d.kind==='unrated'?'no word from Garmin':d.kind==='none'?'no stress readings':'still to come'];
  if(d.average!=null)parts.push(`average ${num(d.average)}`);
  if(d.night)parts.push(`night before ${d.night.rating??'not rated'}${d.night.average!=null?`, average ${num(d.night.average)} asleep`:''}`);
  if(d.workouts)parts.push(d.workouts===1?'1 workout':`${d.workouts} workouts`);
  return parts.join(' · ');
 };
</script>
<div class="stress-calendar">
 <div class="cal-head">
  <button class="nav" aria-label="Previous month" disabled={!view || !view.first || month<=view.first} onclick={()=>month=shift(month,-1)}><Icon name="arrow-left" size={17}/></button>
  <h4>{monthName(month)}</h4>
  <button class="nav" aria-label="Next month" disabled={month>=current} onclick={()=>month=shift(month,1)}><Icon name="arrow-right" size={17}/></button>
 </div>
 {#if error}<p class="small" role="status">Couldn't read this month: {error}</p>
 {:else if shown}
  <div class="grid" data-chart="stress-calendar" role="group" aria-label={`Stress by day, ${monthName(month)}`}>
   {#each 'MTWTFSS'.split('') as letter}<span class="weekday" aria-hidden="true">{letter}</span>{/each}
   {#each Array(lead) as _}<span></span>{/each}
   {#each shown.days as d (d.date)}
    <button class="day {d.kind}" class:selected={d.date===date} style:--tint={d.kind==='rated'?toneColor(d.tone):null}
     disabled={d.kind==='future'} aria-current={d.date===date?'date':undefined} aria-label={`${dateLabel(d.date)}: ${words(d)}`} onclick={()=>{if(d.date!==date)open(d.date)}}>
     <i class="night" class:on={!!d.night} style:background={d.night?toneColor(d.night.tone):null} data-mark="night"></i><b class="n">{Number(d.date.slice(8))}</b><i class="workout" class:on={d.workouts>0} data-mark="workout"></i>
    </button>
   {/each}
  </div>
  <p class="day-readout">{#if selected}<b>{dateLabel(selected.date)}</b> · {words(selected)}{:else}Tap a day to open it.{/if}</p>
  <div class="legend"><span><i class="bar"></i>Night before</span><span><i class="worn"></i>Partly worn</span><span><i class="dot"></i>Workout</span></div>
 {:else}<p class="small">Reading this month…</p>{/if}
</div>
<style>
 .cal-head{display:flex;align-items:center;justify-content:space-between;gap:8px;margin-bottom:10px}
 .cal-head h4{margin:0;font:600 14px Inter;color:var(--text)}
 .nav{display:grid;place-items:center;width:40px;height:40px;border-radius:12px;border:1px solid var(--line);background:var(--card);color:var(--text)}
 .nav:disabled{opacity:.35}
 .grid{display:grid;grid-template-columns:repeat(7,minmax(0,1fr));gap:4px}
 .weekday{text-align:center;font:600 11px Inter;color:var(--text3);padding-bottom:2px}
 .day{display:flex;flex-direction:column;align-items:center;justify-content:center;gap:3px;min-height:42px;padding:4px 0;border:0;border-radius:10px;background:rgba(255,255,255,.03);color:var(--text);cursor:pointer}
 .day b{font:600 13px/1.1 var(--font-number)}
 .day.rated{background:color-mix(in srgb,var(--tint) 34%,transparent)}
 /* Garmin rated only part of the day: outlined, never coloured. */
 .day.lowWear,.day.awakeOnly{background:none;outline:1px dashed var(--text3);outline-offset:-1px}
 .day.none,.day.unrated{background:none;color:var(--text2)}
 .day.future{background:none;color:var(--text3);cursor:default}
 .day.selected{box-shadow:inset 0 0 0 2px var(--ink)}
 /* The night before sits on top, as it comes first in the day. */
 .night{width:62%;height:3px;border-radius:2px;visibility:hidden}
 .workout{width:5px;height:5px;border-radius:50%;background:var(--ink);visibility:hidden}
 .night.on,.workout.on{visibility:visible}
 .day-readout{margin:12px 0 0;font-size:13px;line-height:1.45;color:var(--text2)}
 .day-readout b{color:var(--text);font-weight:600}
 .legend{display:flex;flex-wrap:wrap;gap:4px 12px;margin-top:10px;font-size:12px;color:var(--text2)}
 .legend span{display:inline-flex;align-items:center;gap:6px}
 .legend .worn{width:12px;height:10px;border-radius:3px;outline:1px dashed var(--text3);outline-offset:-1px}
 .legend .dot{width:5px;height:5px;border-radius:50%;background:var(--ink)}
 .legend .bar{width:12px;height:3px;border-radius:2px;background:var(--text2)}
</style>
