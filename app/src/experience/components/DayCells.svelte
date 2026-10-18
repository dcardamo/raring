<script lang="ts">
 import {store} from '../lib/store.svelte';
 import {dayEvidence} from '../lib/allocation';
 import {addDays,dayLabel,narrowDay} from '../lib/dates';
 import {toneColor} from '../../lib/charts';
 let {selected,select,compact=false}:{selected?:string,select?:(date:string)=>void,compact?:boolean}=$props();
 let days=$derived(Array.from({length:7},(_,i)=>dayEvidence(store.ctx,addDays(store.ctx.weekFrom,i))));
 const ring=(value:number,target:number)=>Math.min(100,value/Math.max(1,target)*100);
 const click=(date:string)=>select?select(date):store.openSheet('day',date);
</script>
<div class="day-cells" class:compact aria-label="Your week by day">
 {#each days as d}
  <button class="dc" class:current={d.date===store.ctx.calendarDate} class:selected={selected===d.date} class:future={d.future} class:paused={d.paused} class:rest={d.rest} class:unknown={d.unknown} onclick={()=>click(d.date)} aria-label={`${dayLabel(d.date)}: ${d.rest?'rest day, ':''}${d.paused?'paused, ':''}${d.unknown?'awaiting data, ':''}Sleep score ${d.night?.score??'unavailable'}; ${d.cardioLabel} ${d.future?'future':d.cardio}; strength ${d.future?'future':d.strength}; vitals ${d.vitals}`}>
   <span class="dow">{narrowDay(d.date)}</span><b class="date n">{Number(d.date.slice(8))}</b>
   <span class="circles">
    <span class="circle sleep" style:--c={d.night?toneColor(d.night.score>=80?'good':d.night.score>=60?'fair':'poor'):'#8E98AC'} style:--p={(d.night?.score??0)+'%'} class:blank={!d.night}><i>{d.night?.score??'–'}</i></span>
    <span class="circle cardio" style:--c=" #5CA8FF" style:--p={(d.future||d.unknown?0:ring(d.cardio,d.cardioTarget))+'%'} class:plain={!d.cardioTarget} class:blank={d.future||d.unknown} class:none={!d.cardioTarget&&!d.cardio}><i>{d.future?'·':d.unknown?'?':d.cardio||''}</i></span>
    <span class="circle strength" style:--c="#FF7DB8" style:--p={(d.future||d.unknown?0:ring(d.strength,d.strengthTarget))+'%'} class:plain={!d.strengthTarget} class:blank={d.future||d.unknown} class:none={!d.strengthTarget&&!d.strength}><i>{d.future?'·':d.unknown?'?':d.strength||''}</i></span>
    <span class="health" class:unusual={d.vitals==='unusual'} class:blank={d.vitals==='unknown'}>{d.vitals==='unknown'?'–':d.vitals==='unusual'?'!':'✓'}</span>
   </span>
   {#if d.rest}<span class="state">☾</span>{:else if d.paused}<span class="state">Ⅱ</span>{:else if d.unknown}<span class="state">?</span>{/if}
  </button>
 {/each}
</div>
<div class="cell-key"><span><i class="s"></i>Sleep score</span><span><i class="c"></i>Cardio</span><span><i class="st"></i>Strength</span><span><i class="h"></i>Vitals</span></div>
<style>
 .day-cells{display:grid;grid-template-columns:repeat(7,minmax(0,1fr));gap:4px}.dc{position:relative;min-width:0;padding:8px 2px;border:1px solid transparent;border-radius:13px;background:rgba(255,255,255,.025);color:var(--text);display:flex;flex-direction:column;align-items:center;gap:3px;cursor:pointer}.dc.current{background:rgba(255,255,255,.06);border-color:rgba(255,255,255,.22)}.dc.selected{border-color:var(--ink)}.dow{font:600 10px Inter;color:var(--text3)}.date{font-size:17px;font-weight:600}.circles{display:grid;grid-template-columns:20px 20px;gap:4px 2px;margin-top:4px}
 .circle{height:20px;width:20px;display:grid;place-items:center;border-radius:50%;background:conic-gradient(var(--c) var(--p),rgba(255,255,255,.09) var(--p));padding:2px}.circle i{display:grid;place-items:center;width:16px;height:16px;border-radius:50%;background:#121827;font:600 9px Outfit;color:var(--text);font-style:normal}.circle.plain{background:none;border:1px solid color-mix(in srgb,var(--c) 45%,transparent);padding:0}.circle.blank{background:none;border:1px dotted #596478}.circle.none{border-color:rgba(255,255,255,.1)}.circle.blank i{color:var(--text3)}.health{height:20px;width:20px;display:grid;place-items:center;border-radius:50%;background:rgba(61,220,151,.13);color:#3DDC97;font:600 10px Inter}.health.unusual{background:rgba(255,181,71,.14);color:#FFB547}.health.blank{background:none;border:1px dotted #596478;color:var(--text3)}
 .dc.future{background:none}.dc.future .date{color:var(--text3)}.dc.paused{background-image:repeating-linear-gradient(45deg,rgba(169,200,255,.09) 0 3px,transparent 3px 7px)}.dc.rest{background-color:rgba(169,200,255,.09)}.dc.unknown{border-style:dotted;border-color:#596478}.state{font-size:10px;color:#A9C8FF;height:10px}.cell-key{display:flex;justify-content:space-between;gap:4px;margin-top:10px;font:500 10px Inter;color:var(--text2);flex-wrap:wrap}.cell-key span{display:flex;align-items:center;gap:4px}.cell-key i{height:6px;width:6px;border-radius:50%}.s{background:#A9C8FF}.c{background:#5CA8FF}.st{background:#FF7DB8}.h{background:#3DDC97}
 @container(max-width:280px){.day-cells{grid-template-columns:repeat(7,44px);overflow-x:auto}}
</style>
