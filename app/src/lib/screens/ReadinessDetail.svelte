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
 {#if todayData.factors.length}<Card title="What's driving it"><div class="factor-list">{#each [...todayData.factors].sort((a,b)=>(a.percent??101)-(b.percent??101)) as factor}{@const input=todayData.readinessWhy?.inputs.find(i=>i.key===factor.key)}<button class="factor-row" aria-expanded={selectedFactor===factor.key} onclick={()=>selectedFactor=selectedFactor===factor.key?'':factor.key}><span>{factor.name}</span><b><span class="n">{num(factor.percent)}</span>{#if input?.rating}<small style:color={toneColor(factor.tone)}>{input.rating}</small>{/if}</b><div class="bar"><i style:width={(factor.percent??0)+'%'} style:background={toneColor(factor.tone)}></i></div>{#if selectedFactor===factor.key}<p>{input?`${input.value?`${input.value} · `:''}${input.detail}`:factor.detail}</p>{/if}</button>{/each}</div></Card>{/if}
 {#if todayData.readinessTrend.length}<Card title="Readiness history"><ReadinessTrend values={todayData.readinessTrend} date={todayData.date}/></Card>{/if}
 {#if todayData.call && todayData.focus.effort}<Card title="Today's call"><EffortBar effort={todayData.call}/>{#if todayData.readinessWhy}<p class="reason">{todayData.readinessWhy.summary}</p>{/if}</Card>{/if}
</div>
<style>
 .reason{margin:2px 4px 0;font-size:13px;color:var(--text2)}
 .factor-row>b{display:flex;gap:8px;align-items:baseline;font:600 12px Outfit;color:var(--text)}
 .factor-row>b small{font:600 12px Inter}
</style>
