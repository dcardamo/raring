<script lang="ts">
 import type {Snippet} from 'svelte';
 import type {TodayView} from '../generated/TodayView';
 import {masonry} from '../masonry';
 import Card from '../components/Card.svelte';
 import Icon from '../components/Icon.svelte';
 import Gauge from '../components/Gauge.svelte';
 import WhySheet from '../components/WhySheet.svelte';
 import ReadinessTrend from '../components/ReadinessTrend.svelte';
 import EffortBar from '../components/EffortBar.svelte';
 import {toneColor} from '../charts';
 import {num} from '../format';
 let {todayData,header,selectedFactor=$bindable('')}:{todayData:TodayView,header:Snippet,selectedFactor?:string}=$props();
</script>
 <div class="page-hero">{@render header()}<div class="detail-gauge">{#if todayData.readiness}<Gauge size={240} stroke={17} value={todayData.readiness.value} max={todayData.readiness.max} label={todayData.readiness.label} caption="Readiness" color={toneColor(todayData.readiness.tone)}/>{:else}<p>{todayData.missingMetricReasons.readiness ?? 'Waiting for readiness'}</p>{/if}{#if todayData.readiness}<h2>{todayData.headline}</h2>{/if}{#if todayData.readinessWhy}<div class="why-line"><WhySheet why={todayData.readinessWhy} label={todayData.readiness?.label?`Why ${todayData.readiness.label}?`:'Why?'}/></div>{/if}</div></div><div class="dashboard" {@attach masonry}>
 {#if todayData.factors.length}<Card title="What's driving it"><p class="factor-key">Garmin factor levels · 0–100</p><div class="factor-list">{#each [...todayData.factors].sort((a,b)=>(a.percent??101)-(b.percent??101)) as factor}{@const input=todayData.readinessWhy?.inputs.find(i=>i.key===factor.key)}<button class="factor-row" aria-expanded={selectedFactor===factor.key} onclick={()=>selectedFactor=selectedFactor===factor.key?'':factor.key}><div class="factor-heading"><span>{factor.name}</span>{#if input?.value}<b class="n">{input.value}</b>{/if}<span class="rating" style:color={toneColor(factor.tone)}>{input?.rating??'Not rated'}</span></div><div class="factor-scale"><div class="bar">{#if factor.percent!=null}<i style:width={factor.percent+'%'} style:background={toneColor(factor.tone)}></i>{/if}</div><small class="factor-level">{num(factor.percent)}</small></div>{#if selectedFactor===factor.key}<p>{input?`${input.value?`${input.value} · `:''}${input.detail}`:factor.detail}</p>{/if}</button>{/each}</div></Card>{/if}
 {#if todayData.readinessTrend.length}<Card title="Readiness history"><ReadinessTrend values={todayData.readinessTrend} date={todayData.date}/></Card>{/if}
 {#if todayData.call && todayData.focus.effort}<Card title="Today's call"><EffortBar effort={todayData.call}/>{#if todayData.readinessWhy}<p class="reason">{todayData.readinessWhy.summary}</p>{/if}</Card>{/if}
</div>
<style>
 .reason{margin:2px 4px 0;font-size:13px;color:var(--text2)}
 .factor-row{display:block;min-height:44px}
 .factor-heading{display:flex;gap:4px 10px;flex-wrap:wrap;align-items:baseline}
 .factor-heading>span:first-child{flex:1}
 .factor-heading b{font-size:20px;font-weight:500;color:var(--text)}
 .rating{font:600 12px/1.5 Inter}
 .factor-key{font:400 12px/1.5 Inter;color:var(--text2);margin:0 0 12px}
 .factor-scale{display:flex;gap:8px;align-items:center;margin-top:6px}
 .factor-scale .bar{margin:0;flex:1}
 .factor-level{font:400 12px/1.5 Inter;color:var(--text2)}
</style>
