<script lang="ts">
 import {masonry} from '../masonry';
 import type {TodayView} from '../generated/TodayView';
 import type {HealthView} from '../generated/HealthView';
 import type {TrainingView} from '../generated/TrainingView';
 import {metricNames,metricDescriptions,metricSource} from '../metrics';
 import {num,dateLabel,duration} from '../format';
 import {rangeStatus} from '../signals';
 import HistoryChart from './HistoryChart.svelte';
 import {toneColor,batteryColor,colors,stressColors,stressNames} from '../charts';
 import Card from './Card.svelte';
 import Gauge from './Gauge.svelte';
 import RangeBar from './RangeBar.svelte';
 import Sparkline from './Sparkline.svelte';
 import StressTimeline from './StressTimeline.svelte';
 import WhySheet from './WhySheet.svelte';
 let {metric,data}:{metric:string,data:TodayView|HealthView|TrainingView}=$props();
 let source=$derived(metricSource(metric));
 let battery=$derived(metric==='bodyBattery'?(data as TodayView).bodyBattery:null);
 let stress=$derived(metric==='stress'?(data as TodayView).stress:null);
 let stressTotal=$derived(stress?stress.time.reduce((a,b)=>a+b.seconds,stress.activeS??0):0);
 let vital=$derived(source==='health'?(data as HealthView).vitals.readings.find(v=>v.kind===metric):null);
 let training=$derived(source==='training'?data as TrainingView:null);
 let trainingValue=$derived(training?({vo2max:training.vo2max,endurance:training.enduranceScore,hill:training.hillScore,runningTolerance:training.runningToleranceKm} as Record<string,number|null>)[metric]:null);
 let decimals=$derived(metric==='skinTemp'||metric==='respiration'?1:0);
</script>
<div class="metric-hero">
 {#if stress}
  <div class="metric-value n" style:color={stress.level?stressColors[stress.level]:toneColor('neutral')}>{num(stress.average)}</div>
  <p class="small">{stress.level?stressNames[stress.level]+' · ':''}average so far today{#if stress.max!=null}{` · peak ${num(stress.max)}`}{/if}</p>
  {#if stress.qualifier}<span class="pill" style:color={toneColor(stress.tone)}>{stress.qualifier}</span>{/if}
 {:else if battery}
  <Gauge size={210} stroke={15} value={battery.now} caption="Latest synced" color={battery.now!=null?batteryColor(battery.now):colors.secondary}/>
  {#if (data as TodayView).bodyBatteryWhy}<WhySheet why={(data as TodayView).bodyBatteryWhy!} label={battery.now!=null?`Why ${num(battery.now)}?`:'Why?'}/>{/if}
 {:else if vital}
  <div class="metric-value n" style:color={toneColor(vital.tone)}>{num(vital.value,decimals)}<span class="unit">{vital.unit}</span></div>
  <p class="small">{dateLabel((data as HealthView).date)} · latest synced</p>
  <span class="pill" style:color={toneColor(rangeStatus(vital.value,vital.normal,vital.kind).tone)}>{rangeStatus(vital.value,vital.normal,vital.kind).label}</span>
 {:else}
  <div class="metric-value n">{num(trainingValue)}{#if metric==='runningTolerance'}<span class="unit">km</span>{/if}</div>
  <p class="small">Latest synced estimate</p>
 {/if}
</div>
<div class="dashboard" {@attach masonry}>
 {#if stress}
  {#if stress.timeline.length}<Card title="Through the day"><StressTimeline {stress}/></Card>{/if}
  {#if stress.time.length}<Card title="Time in each level">{#each stress.time as t}<div class="stress-row"><b>{stressNames[t.level]}</b><div class="bar"><i style:width={t.seconds/Math.max(stressTotal,1)*100+'%'} style:background={stressColors[t.level]}></i></div><span class="n">{duration(t.seconds/60)}</span><small>{Math.round(t.seconds/Math.max(stressTotal,1)*100)}%</small></div>{/each}{#if stress.activeS}<div class="stress-row"><b>Active</b><div class="bar"><i style:width={stress.activeS/Math.max(stressTotal,1)*100+'%'} style:background={colors.secondary}></i></div><span class="n">{duration(stress.activeS/60)}</span><small>{Math.round(stress.activeS/Math.max(stressTotal,1)*100)}%</small></div>{/if}</Card>{/if}
  <Card title="Daily average">
   <HistoryChart values={stress.trend} endDate={(data as TodayView).date} label="Average stress" color={colors.ink}/>
  </Card>
 {:else if battery}
  <Card title="Today"><div class="battery-totals"><div><b class="big n">{num(battery.charged)}</b><span>Charged</span></div><div><b class="big n">{num(battery.drained)}</b><span>Drained</span></div></div></Card>
 {:else if vital}
  <Card title="Nightly history">
   {#key metric}
    <HistoryChart values={vital.trend} endDate={(data as HealthView).date} label={metricNames[metric]} unit={vital.unit} {decimals} band={vital.normal} bandLabel={metric==='hrv'?'Current Garmin baseline':'Current usual range'} nights/>
   {/key}
  </Card>
  <Card title={metric==='hrv'?'Garmin baseline':'Your usual range'}>
   {#if vital.normal}<RangeBar value={vital.value} scale={vital.scale} range={vital.normal} color={toneColor(vital.tone)}/><p class="metric-copy range-caption">{num(vital.normal[0],decimals)}–{num(vital.normal[1],decimals)} {vital.unit}</p>{:else}<p class="metric-copy">{metric==='hrv'?"Waiting for Garmin to report your HRV baseline.":"At least 14 recorded nights are needed. Missing nights do not count toward your range."}</p>{/if}
  </Card>
 {:else if training && metric==='vo2max' && training.vo2maxTrend.some(v=>v!=null)}
  <Card title="Weekly trend"><div class="metric-chart"><Sparkline values={training.vo2maxTrend} w={500} h={150}/></div><div class="chart-ends"><span>Earlier</span><span>Latest</span></div></Card>
 {/if}
 <Card title="About this metric"><p class="metric-copy">{metricDescriptions[metric]}</p></Card>
</div>
