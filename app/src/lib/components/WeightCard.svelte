<script lang="ts">
 import {getContext,untrack} from 'svelte';
 import type {WeightView} from '../generated/WeightView';
 import type {TrendRange} from '../generated/TrendRange';
 import * as api from '../api';
 import {dateLabel,num} from '../format';
 import RangePicker from './RangePicker.svelte';
 import WeightChart from './WeightChart.svelte';
 import Sparkline from './Sparkline.svelte';
 import Icon from './Icon.svelte';
 // The approved two-line card: Garmin scale weight and Raring's labelled
 // trend, with neutral changes. All metric math and freshness come from Rust.
 let {range=$bindable('days30'),initial,navigate}:{range?:TrendRange,initial?:WeightView,navigate?:(route:string)=>void}=$props();
 let view=$state<WeightView|null>(untrack(()=>initial??null)),busy=$state(false),error=$state(''),help=$state(false);
 const generation=getContext<()=>number>('trend-generation')??(()=>0);
 const color='#8FA8FF';
 $effect(()=>{
  generation();const wanted=range;let active=true;busy=true;error='';
  api.weight_trend(wanted).then(v=>{if(active){view=v;busy=false}}).catch(e=>{if(active){error=String(e);busy=false}});
  return()=>{active=false};
 });
 const sign=(v:number|null)=>v==null?'—':`${v>0?'+':v<0?'−':'±'}${Math.abs(v).toFixed(1)}`;
 const WORD={losing:'Losing',gaining:'Gaining',steady:'Steady'} as const;
 const id=$props.id();
</script>
<section class="card weight-card" aria-label="Weight" aria-busy={busy} style:--weight={color}>
 <div class="heading"><h3>Weight</h3><button class="help" aria-label="About trend weight" aria-expanded={help} onclick={()=>help=!help}><Icon name="help" size={19}/></button></div>
 {#if help}<p class="explain">Scale values are Garmin weigh-ins. Raring’s trend smooths them: each daily weigh-in moves the trend a tenth of the way toward it; after a gap the adjustment uses the days between weigh-ins. The trend holds between measurements and stops at the last one. Small changes are labelled Steady; no weight goal or judgement is applied.</p>{/if}
 {#if error}<p class="warning" role="status">{error}</p>{/if}
 {#if view}
  {#if view.unit && view.trend!=null}
   <div class="weight-head"><small><i class="trend-key"></i>Trend weight · Raring</small><b class="n">{num(view.trend,1)}<span>{view.unit}</span></b><p><i class="scale-key"></i>Scale {num(view.scale,1)} {view.unit}</p></div>
   <p class="fresh" class:stale={view.freshness.stale}>{#if view.freshness.stale}<Icon name="clock" size={14}/>{/if}Last measured {dateLabel(view.freshness.recordedOn!)} · {view.freshness.ageDays===0?'today':`${view.freshness.ageDays} days ago`}{view.freshness.stale?' · older than 7 days':''}</p>
   <div class="summary"><span><small>Average trend</small><b class="n">{num(view.average,1)}<em>{view.unit}</em></b></span><span><small>Trend change</small><b class="n">{sign(view.change)}<em>{view.unit}</em></b></span></div>
   <p class="small dates">{dateLabel(view.from!)} {view.from!.slice(0,4)} – {dateLabel(view.to!)} {view.to!.slice(0,4)}</p>
   <WeightChart points={view.points} unit={view.unit} {color}/>
   <p class="key"><span><i class="scale-key"></i>Scale weight</span><span><i class="trend-key"></i>Trend weight</span></p>
  {:else}<p class="small">{view.note??'No weigh-ins yet'}</p>{/if}
 {:else}<p class="small">Loading stored weigh-ins…</p>{/if}
 <RangePicker value={range} onchange={r=>range=r} label="Weight range"/>
 {#if view?.unit && view.changes.length}
  <h4>Trend change · ending {dateLabel(view.to!)}</h4>
  <ul class="changes">{#each view.changes as c}<li><span>{c.days} {c.days===1?'day':'days'}</span><span class="spark"><Sparkline values={c.spark} w={56} h={24} {color} fill={false}/></span><b class="n">{sign(c.change)}{#if c.change!=null}<small>{view.unit}</small>{/if}</b><span class="direction">{c.direction?WORD[c.direction]:'Not enough history'}{#if c.perWeek!=null}<small>{sign(c.perWeek)} {view.unit}/wk</small>{/if}</span></li>{/each}</ul>
  <p class="small">Garmin recorded weigh-ins only · Raring trend.{#if view.bodyFat!=null} Body fat {num(view.bodyFat,1)}%.{/if}{#if view.bmi!=null} BMI {num(view.bmi,1)}.{/if}</p>
 {/if}
 <button class="unavailable" disabled aria-describedby={id+'-write'}><Icon name="scale" size={15}/>Add weight</button><p class="small" id={id+'-write'}>Not yet available: adding weight writes to Garmin.</p>
 {#if navigate}<button class="more" onclick={()=>navigate('health/weight')}>Weight details<Icon name="chevron" size={14}/></button>{/if}
</section>
<style>
 .weight-card{background:linear-gradient(160deg,rgba(143,168,255,.1),var(--card) 55%)}.heading{display:flex;justify-content:space-between;align-items:center}.heading h3{margin:0}.help{display:grid;place-items:center;width:40px;height:40px;background:none;border:0;border-radius:50%;color:var(--text2)}
 .weight-head small{display:flex;gap:6px;align-items:center;font-size:12px;color:var(--text2)}.weight-head b{display:block;font-size:30px;font-weight:600;color:var(--weight)}.weight-head b span{font:500 13px Inter;color:var(--text2);margin-left:4px}.weight-head p{display:flex;gap:6px;align-items:center;font-size:13px;color:var(--text2)}
 .trend-key,.scale-key{display:inline-block;width:14px;height:3px;border-radius:2px;background:var(--weight);flex:none}.scale-key{height:2px;opacity:.45}
 .fresh{display:flex;align-items:center;gap:5px;margin:6px 0;font-size:12px;color:var(--text2)}.fresh.stale{color:var(--fair)}
 .summary{display:flex;gap:24px;margin:8px 0 0}.summary small{display:block;font-size:12px;color:var(--text2)}.summary b{font-size:22px;font-weight:600}.summary em{font:500 12px Inter;color:var(--text2);margin-left:3px;font-style:normal}
 .key{display:flex;gap:14px;flex-wrap:wrap;font-size:12px;color:var(--text2);margin-bottom:8px}.key span{display:flex;align-items:center;gap:6px}
 h4{font:600 12px Inter;color:var(--text2);margin:16px 0 4px}.changes{list-style:none;margin:0;padding:0}.changes li{display:grid;grid-template-columns:44px 56px minmax(0,1fr) auto;gap:8px;align-items:center;padding:9px 0;border-top:1px solid var(--line);font-size:12px;color:var(--text2)}.changes b{font-size:15px;color:var(--text);text-align:right;white-space:nowrap}.changes b small{font:500 11px Inter;margin-left:3px;color:var(--text2)}.direction{text-align:right;color:var(--weight);font-weight:600}.direction small{display:block;color:var(--text2);font-size:11px;font-weight:500}
 .unavailable{display:flex;gap:8px;align-items:center;justify-content:center;width:100%;min-height:40px;padding:8px;margin-top:12px;border:1px dashed var(--line);border-radius:12px;background:none;color:var(--text2);font-weight:600;cursor:default}.more{display:flex;gap:6px;align-items:center;min-height:40px;margin-top:8px;background:none;border:0;color:var(--text);font-weight:600}.explain{font-size:13px;color:var(--text2);margin:6px 0 12px}
 @container(max-width:330px){.changes li{grid-template-columns:44px 1fr auto}.spark{display:none}}
</style>
