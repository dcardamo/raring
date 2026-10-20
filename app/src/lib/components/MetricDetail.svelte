<script lang="ts">
 import {masonry} from '../masonry';
 import {addDays} from '../history';
 import * as api from '../api';
 import type {HeartRateDay} from '../generated/HeartRateDay';
 import type {TodayView} from '../generated/TodayView';
 import type {HealthView} from '../generated/HealthView';
 import type {TrainingView} from '../generated/TrainingView';
 import type {VitalReading} from '../generated/VitalReading';
 import type {TrendMetric} from '../generated/TrendMetric';
 import {metricDescriptions,metricSource} from '../metrics';
 import {num,dateLabel,duration} from '../format';
 import {vitalStatus,rangeName,vitalName} from '../signals';
 import HistoryChart from './HistoryChart.svelte';
 import {toneColor,batteryColor,colors,stressColors,stressNames} from '../charts';
 import Card from './Card.svelte';
 import Gauge from './Gauge.svelte';
 import RangeBar from './RangeBar.svelte';
 import StressTimeline from './StressTimeline.svelte';
 import WhySheet from './WhySheet.svelte';
 let {metric,data,dayDate=null}:{metric:string,data:TodayView|HealthView|TrainingView,dayDate?:string|null}=$props();
 let source=$derived(metricSource(metric));
 let battery=$derived(metric==='bodyBattery'?(data as TodayView).bodyBattery:null);
 let stress=$derived(metric==='stress'?(data as TodayView|HealthView).stress:null);
 let isDay=$derived(dayDate!=null);
 let stressTotal=$derived(stress?stress.time.reduce((a,b)=>a+b.seconds,stress.activeS??0):0);
 let vital=$derived(source==='health'?(data as HealthView).vitals.readings.find(v=>v.kind===metric):null);
 let training=$derived(source==='training'?data as TrainingView:null);
 let trainingValue=$derived(training?({vo2max:training.vo2max,endurance:training.enduranceScore,hill:training.hillScore,runningTolerance:training.runningToleranceKm,acuteLoad:training.acute?.value??null} as Record<string,number|null>)[metric]:null);
 // Health Status's HR is Garmin's sleep average, so its history is Sleep HR, not resting HR.
 const vitalTrend=(v:VitalReading):TrendMetric=>v.kind==='restingHr'?(v.status || v.rangeSource==='garmin'?'sleepHr':'restingHr'):v.kind;
 let decimals=$derived(metric==='skinTemp'||metric==='respiration'?1:0);
 let initial=$derived(metric==='heartRate'?(data as HealthView).heartRateDay:null);
 let heartRate=$state<HeartRateDay|null>(null);
 let hrDate=$state(''),hrBusy=$state(false),hrError=$state('');
 let shown=$derived(heartRate??initial);
 async function changeHrDate(date:string){
  hrBusy=true;hrError='';
  try { heartRate=await api.heart_rate_day(date);hrDate=date; }
  catch { hrError='Could not load that day. Try again later.'; }
  finally { hrBusy=false; }
 }
 // An opened dated heart-rate page reads that day once; navigation never fetches.
 let fetchedDay=$state('');
 $effect(()=>{
  if(metric!=='heartRate' || !dayDate || fetchedDay===dayDate)return;
  fetchedDay=dayDate;
  void changeHrDate(dayDate);
 });
</script>
<div class="metric-hero">
 {#if metric==='heartRate' && shown}
  <div class="metric-value n">{num(shown.resting)}<span class="unit">bpm</span></div>
  <p class="small">Garmin daily resting HR · {dateLabel(hrDate||shown.date)}</p>
  <div class="hr-days"><button disabled={hrBusy} onclick={()=>changeHrDate(addDays(hrDate||shown!.date,-1))}>← Previous</button><span>{dateLabel(hrDate||shown.date)}</span><button disabled={hrBusy || (hrDate||shown.date)>=(data as HealthView).date} onclick={()=>changeHrDate(addDays(hrDate||shown!.date,1))}>Next →</button></div>
  {#if hrError}<p class="small">{hrError}</p>{/if}
 {:else if stress}
  <div class="metric-value n" style:color={stress.level?stressColors[stress.level]:toneColor('neutral')}>{num(stress.average)}</div>
  <p class="small">{stress.level?stressNames[stress.level]+' · ':''}average {isDay?'for '+dateLabel((data as HealthView).date):'so far today'}{#if stress.max!=null}{` · peak ${num(stress.max)}`}{/if}</p>
  {#if stress.qualifier}<span class="pill" style:color={toneColor(stress.tone)}>{stress.qualifier}</span>{/if}
 {:else if battery}
  <Gauge size={210} stroke={15} value={battery.now} caption="Latest synced" color={battery.now!=null?batteryColor(battery.now):colors.secondary}/>
  {#if (data as TodayView).bodyBatteryWhy}<WhySheet why={(data as TodayView).bodyBatteryWhy!} label={battery.now!=null?`Why ${num(battery.now)}?`:'Why?'}/>{/if}
 {:else if vital}
  <div class="metric-value n" style:color={toneColor(vital.tone)}>{num(vital.value,decimals)}<span class="unit">{vital.unit}</span></div>
  <p class="small">{dateLabel((data as HealthView).date)} · latest synced</p>
  <span class="pill" style:color={toneColor(vitalStatus(vital).tone)}>{vitalStatus(vital).label}</span>
 {:else}
  <div class="metric-value n">{num(trainingValue)}{#if metric==='runningTolerance'}<span class="unit">km</span>{/if}</div>
  <p class="small">Latest synced estimate</p>
 {/if}
</div>
<div class="dashboard" {@attach masonry}>
 {#if metric==='heartRate' && shown}
  <Card title="Through the day">{#if shown.timeline.length}{#key shown.date}<StressTimeline heartRate={shown}/>{/key}{:else}<p class="small">No heart-rate samples for this day.</p>{/if}</Card>
  <Card title="Daily values"><div class="battery-totals"><div><b class="big n">{num(shown.resting)}</b><span>Garmin resting HR · bpm</span></div><div><b class="big n">{num(shown.minimum)}</b><span>Minimum · bpm</span></div><div><b class="big n">{num(shown.maximum)}</b><span>Maximum · bpm</span></div></div></Card>
 {:else if stress}
  {#if stress.timeline.length}<Card title="Through the day"><StressTimeline {stress}/></Card>{:else}<Card title="Through the day"><p class="small">Garmin has no stress readings for this day.</p></Card>{/if}
  {#if stress.time.length}<Card title="Time in each level">{#each stress.time as t}<div class="stress-row"><b>{stressNames[t.level]}</b><div class="bar"><i style:width={t.seconds/Math.max(stressTotal,1)*100+'%'} style:background={stressColors[t.level]}></i></div><span class="n">{duration(t.seconds/60)}</span><small>{Math.round(t.seconds/Math.max(stressTotal,1)*100)}%</small></div>{/each}{#if stress.activeS}<div class="stress-row"><b>Active</b><div class="bar"><i style:width={stress.activeS/Math.max(stressTotal,1)*100+'%'} style:background={colors.secondary}></i></div><span class="n">{duration(stress.activeS/60)}</span><small>{Math.round(stress.activeS/Math.max(stressTotal,1)*100)}%</small></div>{/if}</Card>{/if}
  <Card title="Daily average">
   <HistoryChart values={stress.trend} endDate={(data as TodayView|HealthView).date} label="Average stress" color={colors.ink} metric="stress"/>
  </Card>
 {:else if battery}
  <Card title="Daily high"><HistoryChart values={[]} endDate={(data as TodayView).date} label="Highest Body Battery" metric="bodyBattery" initialRange="days30"/></Card>
  <Card title="Today"><div class="battery-totals"><div><b class="big n">{num(battery.charged)}</b><span>Charged</span></div><div><b class="big n">{num(battery.drained)}</b><span>Drained</span></div></div></Card>
 {:else if vital}
  <Card title="Nightly history">
   {#key metric}
    <HistoryChart values={vital.trend} endDate={(data as HealthView).date} label={vitalName(vital)} unit={vital.unit} {decimals} band={vital.normal} bandLabel={'Current '+rangeName(vital)} nights metric={vitalTrend(vital)}/>
   {/key}
  </Card>
  <Card title={rangeName(vital)}>
   {#if vital.normal}<RangeBar value={vital.value} scale={vital.scale} range={vital.normal} color={toneColor(vital.tone)}/><p class="metric-copy range-caption">{num(vital.normal[0],decimals)}–{num(vital.normal[1],decimals)} {vital.unit}</p>{:else}<p class="metric-copy">{vital.status==='ONBOARDING'?"Garmin is still building this baseline.":metric==='hrv'?"Waiting for Garmin to report your HRV baseline.":"At least 14 recorded nights are needed for Raring's fallback range."}</p>{/if}
  </Card>
 {:else if training && metric==='acuteLoad'}
  <Card title="History"><HistoryChart values={[]} label="Acute load" metric="acuteLoad" initialRange="days30" band={training.acute?.band ?? null} bandLabel="Garmin's optimal range now"/></Card>
 {:else if training && metric==='vo2max' && training.vo2maxTrend.some(v=>v!=null)}
  <Card title="History"><HistoryChart values={[]} label="VO₂ max" metric="vo2max" initialRange="year"/></Card>
 {/if}
 <Card title="About this metric"><p class="metric-copy">{vital?.kind==='restingHr' && (vital.status || vital.rangeSource==='garmin') ? 'Garmin’s average heart rate during sleep, compared with its Health Status baseline. This is not your resting heart rate or a medical target.' : metricDescriptions[metric]}</p></Card>
</div>
<style>
 .hr-days{display:flex;justify-content:center;align-items:center;gap:10px;flex-wrap:wrap;margin:8px 0 4px;color:var(--text2);font-size:12px}
 .hr-days button{background:rgba(255,255,255,.07);border:0;border-radius:10px;padding:8px 10px;color:var(--text);font-size:12px}
 .hr-days button:disabled{opacity:.4;cursor:default}
</style>
