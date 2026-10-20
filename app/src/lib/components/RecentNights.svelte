<script lang="ts">
 import type {SleepPlan} from '../generated/SleepPlan';
 import type {NightWindow} from '../generated/NightWindow';
 import {recentNights,clockLabel} from '../clock';
 import {dateLabel} from '../format';
 import {declutter} from '../declutter';
 // The last 14 nights from recorded sleep start to end on a clock axis, evening
 // at the top, against tonight's planned bedtime band and wake: whether you
 // usually make the window. A missing night is marked, never drawn as sleep.
 let {date,windows,plan,navigate}:{date:string,windows:NightWindow[],plan:SleepPlan|null,navigate:(route:string)=>void}=$props();
 let r=$derived(recentNights(date,windows,plan));
 let span=$derived(r.bottom-r.top);
 const y=(h:number)=>(h-r.top)/span*100;
 const hm=(h:number)=>`${clockLabel(Math.floor(h))}:${String(Math.round(h%1*60)%60).padStart(2,'0')}`;
 const weekday=(d:string)=>new Date(d+'T12:00:00Z').toLocaleDateString('en-GB',{weekday:'narrow',timeZone:'UTC'});
 let missing=$derived(r.nights.some(n=>n.bed==null));
</script>
<div class="recent" data-chart="recent nights" {@attach declutter} role="group" aria-label="Recorded sleep, last 14 nights, against tonight’s window">
 <span class="yl" aria-hidden="true">{#each r.ticks as t}<i data-thin={t===r.top||t===r.bottom?3:2} style:top={y(t)+'%'}>{clockLabel(t)}</i>{/each}</span>
 <div class="plot">
  {#each r.ticks as t}<span class="grid" style:top={y(t)+'%'}></span>{/each}
  {#if r.band}<span class="band" style:top={y(r.band[0])+'%'} style:height={Math.max(.8,y(r.band[1])-y(r.band[0]))+'%'}></span>{/if}
  {#if r.wake!=null}<span class="wake" style:top={y(r.wake)+'%'}></span>{/if}
  <div class="cols">
   {#each r.nights as n}
    <button class="col" aria-label={`Open ${dateLabel(n.date)}: ${n.bed!=null&&n.wake!=null?`asleep ${hm(n.bed)} to ${hm(n.wake)}`:'not recorded'}`} onclick={()=>navigate('sleep/'+n.date)}>
     {#if n.bed!=null && n.wake!=null && n.wake>n.bed}<b style:top={y(n.bed)+'%'} style:height={y(n.wake)-y(n.bed)+'%'}></b>{:else}<span class="gap" style:top={r.band?y(r.band[0])+'%':'40%'}></span>{/if}
    </button>
   {/each}
  </div>
 </div>
 <span></span>
 <div class="days" aria-hidden="true">{#each r.nights as n,i}<span data-thin={i===r.nights.length-1?3:1}>{weekday(n.date)}</span>{/each}</div>
</div>
<p class="key"><span><i class="k-bar"></i>Asleep</span>{#if r.band}<span><i class="k-band"></i>Tonight’s bedtime</span>{/if}{#if r.wake!=null}<span><i class="k-wake"></i>Tonight’s wake</span>{/if}{#if missing}<span><i class="k-gap"></i>Not recorded</span>{/if}</p>
<style>
 .recent{display:grid;grid-template-columns:auto 1fr;grid-template-rows:200px auto;column-gap:6px;row-gap:4px;margin-top:4px}
 .yl{position:relative;min-width:1.6em;font:500 11px Inter;color:var(--text2)}.yl i{position:absolute;right:0;transform:translateY(-50%);font-style:normal;white-space:nowrap}
 .plot{position:relative}
 .grid{position:absolute;left:0;right:0;border-top:1px solid var(--chart-grid)}
 .band{position:absolute;left:0;right:0;background:var(--chart-band);border-top:1px solid var(--line-strong);border-bottom:1px solid var(--line-strong)}
 .wake{position:absolute;left:0;right:0;border-top:2px dashed var(--ink);opacity:.7}
 .cols{position:absolute;inset:0;display:grid;grid-template-columns:repeat(14,minmax(0,1fr))}
 .col{position:relative;height:100%;border:0;padding:0;background:none;cursor:pointer}
 .col b{position:absolute;left:22%;right:22%;border-radius:4px;background:var(--light)}
 .col .gap{position:absolute;left:22%;right:22%;height:18px;border:1.5px dotted var(--text3);border-radius:4px}
 .days{display:grid;grid-template-columns:repeat(14,minmax(0,1fr));font:500 11px Inter;color:var(--text2);text-align:center}
 .key{display:flex;flex-wrap:wrap;gap:4px 14px;font-size:12px;color:var(--text2);margin:8px 0 0}.key span{display:inline-flex;gap:5px;align-items:center}
 .key i{display:inline-block;width:10px;height:8px;border-radius:2px}
 .k-bar{background:var(--light)}.k-band{background:var(--chart-band);border:1px solid var(--line-strong)}.k-wake{width:12px!important;height:0!important;border-top:2px dashed var(--ink)}.k-gap{border:1.5px dotted var(--text3)}
</style>
