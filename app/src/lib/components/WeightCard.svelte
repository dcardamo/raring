<script lang="ts">
 import {getContext,untrack} from 'svelte';
 import type {WeightView} from '../generated/WeightView';
 import type {TrendRange} from '../generated/TrendRange';
 import * as api from '../api';
 import {dateLabel,num} from '../format';
 import {colors} from '../charts';
 import {masonry} from '../masonry';
 import RangePicker from './RangePicker.svelte';
 import MetricSummary from './MetricSummary.svelte';
 import WeightChart from './WeightChart.svelte';
 import Sparkline from './Sparkline.svelte';
 import HelpButton from './HelpButton.svelte';
 import Card from './Card.svelte';
 import Icon from './Icon.svelte';
 // Garmin scale weight and Raring's labelled trend, laid out like every other history card in the neutral blue.
 // All metric math and freshness come from Rust. On Health (`navigate` given) it is the compact card that opens
 // the detail; the detail adds every range, each change and body composition.
 // `endDate` anchors the card to an opened past day; otherwise it ends today.
 let {range=$bindable('days30'),initial,navigate,endDate}:{range?:TrendRange,initial?:WeightView,navigate?:(route:string)=>void,endDate?:string}=$props();
 let compact=$derived(!!navigate);
 // Health offers four of the detail's ranges; one chosen only on the detail (14D, Max) shows here as 30D.
 const COMPACT:TrendRange[]=['days7','days30','days90','year'];
 let wanted=$derived<TrendRange>(compact && !COMPACT.includes(range)?'days30':range);
 let view=$state<WeightView|null>(untrack(()=>initial??null)),busy=$state(false),error=$state('');
 const generation=getContext<()=>number>('trend-generation')??(()=>0);
 const color=colors.neutral;
 $effect(()=>{
  generation();const read=wanted;let active=true;busy=true;error='';
  api.weight_trend(read,endDate).then(v=>{if(active){view=v;busy=false}}).catch(e=>{if(active){error=String(e);busy=false}});
  return()=>{active=false};
 });
 // Rounded before signing: a change that rounds to nothing reads 0.0, never −0.0.
 const sign=(v:number|null)=>{if(v==null)return '—';const r=Math.round(v*10)/10;return `${r>0?'+':r<0?'−':''}${Math.abs(r).toFixed(1)}`};
 const WORD={losing:'Losing',gaining:'Gaining',steady:'Steady'} as const;
 const DAYS:Partial<Record<TrendRange,number>>={days7:7,days14:14,days30:30,days90:90};
 // The period's direction word, where the core has one for a change that long.
 let direction=$derived.by(()=>{const v=view;return v?v.changes.find(c=>c.days===DAYS[v.range]) ?? null:null});
 let ready=$derived(!!view?.unit && view.trend!=null);
 const ago=(days:number|null)=>days==null?'':days===0?'today':days===1?'1 day ago':`${days} days ago`;
</script>
{#snippet history()}
 {#if error}<p class="warning" role="status">{error}</p>{/if}
 {#if view}
  {#if view.unit && view.trend!=null}
   <div class="period-summary">
    <p class="period-source">Garmin weigh-ins · {dateLabel(view.from!)} {view.from!.slice(0,4)} – {dateLabel(view.to!)} {view.to!.slice(0,4)}</p>
    <div class="summary-pair">
     {#if compact}<MetricSummary label="Trend weight" value={num(view.trend,1)} unit={view.unit} context="Raring trend"/>{:else}<MetricSummary label="Average trend" value={num(view.average,1)} unit={view.unit} context="Raring trend"/>{/if}
     <MetricSummary label="Trend change" value={sign(view.change)} unit={view.unit} context={direction?.direction?WORD[direction.direction]+(direction.perWeek!=null?` · ${sign(direction.perWeek)} ${view.unit}/wk`:''):''}/>
    </div>
    <p class="fresh" class:stale={view.freshness.stale}>{#if view.freshness.stale}<Icon name="clock" size={14}/>{/if}Last weigh-in {dateLabel(view.freshness.recordedOn!)} · {ago(view.freshness.ageDays)}{view.freshness.stale?' · older than 7 days':''}</p>
   </div>
   <WeightChart points={view.points} unit={view.unit} {compact} {color}/>
  {:else}<p class="small">{view.note??'No weigh-ins yet'}</p>{/if}
 {:else}<p class="small">Loading stored weigh-ins…</p>{/if}
{/snippet}
{#if compact}
 <section class="card weight-card" aria-label="Weight" aria-busy={busy}>
  <h3><span>Weight</span><span class="card-actions"><HelpButton topic="trend-weight"/></span></h3>
  <div class="picker"><RangePicker value={wanted} onchange={r=>range=r} label="Weight range" only={COMPACT}/></div>
  {@render history()}
  <button class="link-row history-more" onclick={()=>navigate?.('health/weight')}>Changes and body composition<Icon name="chevron" size={15}/></button>
 </section>
{:else}
 {#if ready && view}
  <div class="metric-hero"><div class="metric-value n" style:color={color}>{num(view.trend,1)}<span class="unit">{view.unit}</span></div><p class="small">Raring trend weight · Garmin scale {num(view.scale,1)} {view.unit}</p></div>
 {/if}
 <div class="dashboard" {@attach masonry}>
  <section class="card weight-card" aria-label="Weight" aria-busy={busy}>
   <h3><span>Scale and trend</span><span class="card-actions"><HelpButton topic="trend-weight"/></span></h3>
   <div class="picker"><RangePicker value={range} onchange={r=>range=r} label="Weight range"/></div>
   {@render history()}
  </section>
  {#if ready && view?.changes.length}
   <Card title={`Trend change · ending ${dateLabel(view.to!)}`} extra="weight-changes">
    <!-- Under a week, a sparkline's own scale turns a steady 0.1 into a steep line. -->
    <ul class="changes">{#each view.changes as c}<li><span>{c.days} {c.days===1?'day':'days'}</span><span class="spark">{#if c.days>=7}<Sparkline values={c.spark} w={56} h={24} {color} fill={false}/>{/if}</span><b class="n">{sign(c.change)}{#if c.change!=null}<small>{view.unit}</small>{/if}</b><span class="direction">{c.direction?WORD[c.direction]:'Not enough history'}{#if c.perWeek!=null}<small>{sign(c.perWeek)} {view.unit}/wk</small>{/if}</span></li>{/each}</ul>
   </Card>
  {/if}
  {#if ready && view && (view.bodyFat!=null || view.bmi!=null)}
   <Card title="Body composition"><div class="body">{#if view.bodyFat!=null}<MetricSummary small label="Body fat · Garmin" value={num(view.bodyFat,1)} unit="%"/>{/if}{#if view.bmi!=null}<MetricSummary small label="BMI · Garmin" value={num(view.bmi,1)}/>{/if}</div></Card>
  {/if}
 </div>
{/if}
<style>
 .picker{display:flex;min-width:0}
 .period-summary{margin:16px 0}
 .period-source{font:400 12px/1.5 Inter;color:var(--text2);margin:0 0 10px;overflow-wrap:anywhere}
 .fresh{display:flex;align-items:center;gap:5px;margin:8px 0 0;font-size:12px;color:var(--text2)}.fresh.stale{color:var(--fair);font-weight:600}
 .summary-pair{display:grid;grid-template-columns:repeat(auto-fit,minmax(min(130px,100%),1fr));gap:12px}
 .changes{list-style:none;margin:0;padding:0}.changes li{display:grid;grid-template-columns:58px 56px minmax(0,1fr) auto;gap:8px;align-items:center;padding:9px 0;border-top:1px solid var(--line);font-size:12px;color:var(--text2)}.changes li:first-child{border-top:0}.changes b{font-size:15px;color:var(--text);text-align:right;white-space:nowrap}.changes b small{font:500 11px Inter;margin-left:3px;color:var(--text2)}.direction{text-align:right;color:var(--text);font-weight:600}.direction small{display:block;color:var(--text2);font-size:11px;font-weight:500}
 .body{display:grid;grid-template-columns:repeat(auto-fit,minmax(min(130px,100%),1fr));gap:12px}
 @container(max-width:330px){.changes li{grid-template-columns:58px 1fr auto}.spark{display:none}}
</style>
