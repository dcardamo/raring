<script lang="ts">
 import type {TodayView} from '../generated/TodayView';
 import type {HealthView} from '../generated/HealthView';
 import type {TrainingView} from '../generated/TrainingView';
 import {metricNames,metricDescriptions,metricSource} from '../metrics';
 import {num,dateLabel} from '../format';
 import {rangeStatus} from '../signals';
 import HistoryChart from './HistoryChart.svelte';
 import {toneColor,colors} from '../charts';
 import Card from './Card.svelte';
 import Gauge from './Gauge.svelte';
 import RangeBar from './RangeBar.svelte';
 import Sparkline from './Sparkline.svelte';
 let {metric,data}:{metric:string,data:TodayView|HealthView|TrainingView}=$props();
 let source=$derived(metricSource(metric));
 let battery=$derived(source==='today'?(data as TodayView).bodyBattery:null);
 let vital=$derived(source==='health'?(data as HealthView).vitals.readings.find(v=>v.kind===metric):null);
 let training=$derived(source==='training'?data as TrainingView:null);
 let trainingValue=$derived(training?({vo2max:training.vo2max,endurance:training.enduranceScore,hill:training.hillScore,runningTolerance:training.runningToleranceKm} as Record<string,number|null>)[metric]:null);
 let decimals=$derived(metric==='skinTemp'||metric==='respiration'?1:0);
</script>
<div class="metric-hero">
 {#if battery}
  <Gauge size={210} stroke={15} value={battery.now} caption="Latest synced" color={colors.mint}/>
 {:else if vital}
  <div class="metric-value n" style:color={toneColor(vital.tone)}>{num(vital.value,decimals)}<span class="unit">{vital.unit}</span></div>
  <p class="small">{dateLabel((data as HealthView).date)} · latest synced</p>
  <span class="pill" style:color={toneColor(rangeStatus(vital.value,vital.normal,vital.kind).tone)}>{rangeStatus(vital.value,vital.normal,vital.kind).label}</span>
 {:else}
  <div class="metric-value n">{num(trainingValue)}{#if metric==='runningTolerance'}<span class="unit">km</span>{/if}</div>
  <p class="small">Latest synced estimate</p>
 {/if}
</div>
<div class="dashboard">
 {#if battery}
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
