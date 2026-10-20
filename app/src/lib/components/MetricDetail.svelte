<script lang="ts">
 import {masonry} from '../masonry';
 import {addDays} from '../history';
 import * as api from '../api';
 import type {HeartRateDay} from '../generated/HeartRateDay';
 import type {TodayView} from '../generated/TodayView';
 import type {HealthView} from '../generated/HealthView';
 import type {TrainingView} from '../generated/TrainingView';
 import {metricDescriptions,metricSource} from '../metrics';
 import {num,dateLabel,duration} from '../format';
 import {vitalStatus,rangeName,vitalName,vitalTrend,isSleepHr,trendBand} from '../signals';
 import HistoryChart from './HistoryChart.svelte';
 import {toneColor,batteryColor,colors,stressColors,stressNames} from '../charts';
 import Card from './Card.svelte';
 import Gauge from './Gauge.svelte';
 import RangeBar from './RangeBar.svelte';
 import AmountRows from './AmountRows.svelte';
 import StressTimeline from './StressTimeline.svelte';
 import WhySheet from './WhySheet.svelte';
 let {metric,data,dayDate=null}:{metric:string,data:TodayView|HealthView|TrainingView,dayDate?:string|null}=$props();
 let source=$derived(metricSource(metric));
 let battery=$derived(metric==='bodyBattery'?(data as TodayView).bodyBattery:null);
 let stress=$derived(metric==='stress'?(data as TodayView|HealthView).stress:null);
 let isDay=$derived(dayDate!=null);
 let vital=$derived(source==='health'?(data as HealthView).vitals.readings.find(v=>v.kind===metric):null);
 let training=$derived(source==='training'?data as TrainingView:null);
 let trainingValue=$derived(training?({vo2max:training.vo2max,endurance:training.enduranceScore,hill:training.hillScore,runningTolerance:training.runningToleranceKm,acuteLoad:training.acute?.value??null} as Record<string,number|null>)[metric]:null);
 // Sleep HR is Garmin's overnight average, against its own usual range; never resting HR.
 let sleepHr=$derived(metric==='sleepHr'?(data as HealthView).nightVitals?.sleepHr??null:null);
 const usualWords:Record<string,string>={within:'Within usual',above:'Above usual',below:'Below usual',unset:'Usual range not set yet',calibrating:'Garmin is still building this baseline'};
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
  <div class="hr-days"><button disabled={hrBusy} onclick={()=>changeHrDate(addDays(hrDate||shown!.date,-1))}>← Previous day</button><span>{dateLabel(hrDate||shown.date)}</span><button disabled={hrBusy || (hrDate||shown.date)>=(data as HealthView).date} onclick={()=>changeHrDate(addDays(hrDate||shown!.date,1))}>Next day →</button></div>
  {#if hrError}<p class="small">{hrError}</p>{/if}
 {:else if stress}
  <div class="metric-value n" style:color={stress.level?stressColors[stress.level]:toneColor('neutral')}>{num(stress.average)}</div>
  <p class="small">{stress.level?stressNames[stress.level]+' · ':''}average {isDay?'for '+dateLabel((data as HealthView).date):'so far today'}{#if stress.max!=null}{` · peak ${num(stress.max)}`}{/if}</p>
  {#if stress.qualifier}<span class="pill" style:color={toneColor(stress.tone)}>{stress.qualifier}</span>{/if}
 {:else if battery}
  <Gauge size={210} stroke={15} value={battery.now} caption="Latest synced" color={battery.now!=null?batteryColor(battery.now):colors.secondary}/>
  {#if (data as TodayView).bodyBatteryWhy}<WhySheet why={(data as TodayView).bodyBatteryWhy!} label={battery.now!=null?`Why ${num(battery.now)}?`:'Why?'}/>{/if}
 {:else if metric==='sleepHr'}
  <div class="metric-value n" style:color={sleepHr?toneColor(sleepHr.tone):null}>{num(sleepHr?.value??null)}<span class="unit">bpm</span></div>
  <p class="small">{dateLabel((data as HealthView).date)} · Garmin’s overnight average</p>
  {#if sleepHr && sleepHr.value!=null}<span class="pill" style:color={toneColor(sleepHr.tone)}>{usualWords[sleepHr.state]}{sleepHr.usual?` · ${num(sleepHr.usual[0])}–${num(sleepHr.usual[1])} bpm`:''}</span>{/if}
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
  <Card title="Daily range"><div class="battery-totals"><div><b class="big n">{num(shown.minimum)}</b><span>Minimum · bpm</span></div><div><b class="big n">{num(shown.maximum)}</b><span>Maximum · bpm</span></div></div></Card>
  <Card title="Daily resting HR"><HistoryChart values={[]} endDate={(data as HealthView).date} label="Resting heart rate" unit="bpm" metric="restingHr" initialRange="days30" band={trendBand('restingHr',data as HealthView)} bandLabel="Raring usual range"/></Card>
 {:else if stress}
  {#if stress.timeline.length}<Card title="Through the day"><StressTimeline {stress}/></Card>{:else}<Card title="Through the day"><p class="small">Garmin has no stress readings for this day.</p></Card>{/if}
  {#if stress.time.length}<Card title="Time in each level"><AmountRows label="Garmin stress durations" unit="h:mm" format={duration} rows={[...stress.time.map(t=>({label:stressNames[t.level],value:t.seconds/60,text:duration(t.seconds/60),color:stressColors[t.level]})),...(stress.activeS!=null?[{label:'Active · not rated',value:stress.activeS/60,text:duration(stress.activeS/60),color:colors.secondary}]:[])]}/></Card>{/if}
  <Card title="Daily average">
   <HistoryChart values={stress.trend} endDate={(data as TodayView|HealthView).date} label="Average stress" color={colors.ink} metric="stress"/>
  </Card>
 {:else if battery}
  <Card title="Daily high"><HistoryChart values={[]} endDate={(data as TodayView).date} label="Highest Body Battery" metric="bodyBattery" initialRange="days30"/></Card>
  <Card title="Today"><div class="battery-totals"><div><b class="big n">{num(battery.charged)}</b><span>Charged</span></div><div><b class="big n">{num(battery.drained)}</b><span>Drained</span></div></div></Card>
 {:else if metric==='sleepHr'}
  <Card title="Nightly history"><HistoryChart values={[]} endDate={(data as HealthView).date} label="Sleep HR" unit="bpm" band={sleepHr?.usual??null} bandLabel={sleepHr?.rangeSource==='garmin'?'Garmin’s Health Status baseline':'Raring usual range'} nights metric="sleepHr" initialRange="days30"/></Card>
 {:else if vital}
  <Card title="Nightly history">
   {#key metric}
    <HistoryChart values={vital.trend} endDate={(data as HealthView).date} label={vitalName(vital)} unit={vital.unit} {decimals} band={vital.normal} bandLabel={'Current '+rangeName(vital)} nights metric={vitalTrend(vital)} initialRange="days30"/>
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
 <Card title="About this metric"><p class="metric-copy">{(vital && isSleepHr(vital)) || metric==='sleepHr' ? 'Garmin’s average heart rate during sleep, compared with its Health Status baseline when Garmin sends one, otherwise with the middle 80% of your last 30 recorded nights. Your daily resting heart rate is on the heart-rate page.' : metricDescriptions[metric]}</p></Card>
</div>
<style>
 .hr-days{display:flex;justify-content:center;align-items:center;gap:10px;flex-wrap:wrap;margin:8px 0 4px;color:var(--text2);font-size:12px}
 .hr-days button{background:rgba(255,255,255,.07);border:0;border-radius:10px;padding:8px 10px;color:var(--text);font-size:12px}
 .hr-days button:disabled{opacity:.4;cursor:default}
</style>
