<script lang="ts">
 import type {Snippet} from 'svelte';
 import type {SleepView} from '../generated/SleepView';
 import {masonry} from '../masonry';
 import Card from '../components/Card.svelte';
 import Icon from '../components/Icon.svelte';
 import Gauge from '../components/Gauge.svelte';
 import Hypnogram from '../components/Hypnogram.svelte';
 import AmountRows from '../components/AmountRows.svelte';
 import Sparkline from '../components/Sparkline.svelte';
 import ConsistencyChart from '../components/ConsistencyChart.svelte';
 import HistoryChart from '../components/HistoryChart.svelte';
 import AlignmentCard from '../components/AlignmentCard.svelte';
 import SleepPlanCard from '../components/SleepPlanCard.svelte';
 import {colors,toneColor} from '../charts';
 import {duration,clock,dateLabel,signed,num} from '../format';
 // Last night and Tonight are two tabs of one page; Tonight's content comes from the caller.
 let {sleepData,header,tab='last',setTab,tonight,navigate=()=>{},historical=false}:{sleepData:SleepView,header:Snippet,tab?:'last'|'tonight',setTab?:(t:'last'|'tonight')=>void,tonight?:Snippet,navigate?:(route:string)=>void,historical?:boolean}=$props();
 const stageOrder=['awake','rem','light','deep'] as const;
 // Stage coverage can differ; do not present one stage's count as everybody's.
 let stageCounts=$derived([...new Set(sleepData.totals.filter(s=>s.comparisonMin!=null).map(s=>s.comparisonNights))].sort((a,b)=>a-b));
 let stageComparisonHeading=$derived(stageCounts.length?`Ticks: your 30-night average · ${stageCounts.join(' or ')} ${stageCounts.length===1&&stageCounts[0]===1?'night':'nights'}`:'Your 30-night average is unavailable');
 let stageRows=$derived(stageOrder.flatMap(stage=>{
  const s=sleepData.totals.find(t=>t.stage===stage);if(!s)return [];
  const name=stage==='rem'?'REM':stage[0].toUpperCase()+stage.slice(1);
  return [{label:name,value:s.minutes,text:duration(s.minutes),color:colors[stage],comparison:s.comparisonMin,
   comparisonText:s.comparisonMin!=null?`Avg ${duration(s.comparisonMin)}`:'Average unavailable',
   accessibleLabel:`${name}, ${duration(s.minutes)} h:mm. ${s.comparisonMin!=null?`Raring 30-night average ${duration(s.comparisonMin)} h:mm from ${s.comparisonNights} recorded ${s.comparisonNights===1?'night':'nights'}, through ${dateLabel(sleepData.date)}.`:'30-night average unavailable.'}`}];
 }));
 let needScale=$derived(Math.max(1,sleepData.asleepMin??0,sleepData.needMin??0)*1.1);
 const OVERNIGHT:Record<string,string>={heartRate:'Heart rate',hrv:'HRV',respiration:'Respiration',spo2:'SpO₂'};
 // Each overnight tile opens its metric's detail; a past night's only dated detail is heart rate.
 const tileRoute=(kind:string)=>historical?(kind==='heartRate'?`metric/heartRate/${sleepData.date}`:null):({heartRate:'metric/heartRate',hrv:'metric/hrv',respiration:'metric/respiration',spo2:'metric/spo2'} as Record<string,string>)[kind]??null;
 function napWhen(local:string|null,label:string){
  if(!local || !/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}/.test(local) || clock(local)==='—')return `${label} date/time unavailable`;
  const day=dateLabel(local.slice(0,10));
  return day==='Date unavailable'?`${label} date/time unavailable`:`${day} ${clock(local)}`;
 }
</script>
 <div class="page-hero">{@render header()}{#if setTab}<div class="sleep-tabs" role="tablist" aria-label="Sleep"><button role="tab" aria-selected={tab==='last'} onclick={()=>setTab('last')}>Last night</button><button role="tab" aria-selected={tab==='tonight'} onclick={()=>setTab('tonight')}>Tonight</button></div>{/if}{#if tab==='last'}<section class="sleep-hero">{#if sleepData.score}<Gauge size={150} stroke={12} value={sleepData.score.value} max={sleepData.score.max} caption="Score" label={sleepData.score.label} color={toneColor(sleepData.score.tone)}/>{/if}<div class="need"><div class="eyebrow">Asleep</div><div class="big n">{duration(sleepData.asleepMin)}</div><div class="needbar" role="img" aria-label={`Garmin time asleep ${duration(sleepData.asleepMin)}; last night's need ${duration(sleepData.needMin)}`}>{#if sleepData.asleepMin!=null}<i style:width={sleepData.asleepMin/needScale*100+'%'}></i>{/if}{#if sleepData.needMin!=null}<u style:left={sleepData.needMin/needScale*100+'%'}></u>{/if}</div><div class="need-sub">{#if sleepData.needMin!=null}Last night's need <b>{duration(sleepData.needMin)}</b>{#if sleepData.asleepMin!=null && Math.abs(sleepData.asleepMin-sleepData.needMin)>=1} <span class="nowrap">&nbsp;· <strong>{signed(sleepData.asleepMin-sleepData.needMin)} min</strong></span>{/if}{:else}Sleep need unavailable{/if}</div></div></section>{/if}</div>{#if tab==='last'}<div class="dashboard" {@attach masonry}>
 {#if historical && sleepData.detail.unavailable.includes('sleep')}<Card title="Not recorded"><p class="small">Garmin has no sleep record for this night.</p></Card>{/if}
 {#if sleepData.stages.length || sleepData.totals.length}<Card title="Stages" help="sleep-last">{#if sleepData.stages.length}<Hypnogram segments={sleepData.stages} startLocal={sleepData.startLocal}/>{/if}{#if sleepData.totals.length}<div class="stage-amounts" data-chart="stage-comparison"><h4 class="stage-comparison-heading">{stageComparisonHeading}</h4><AmountRows rows={stageRows} label="Garmin sleep stage durations" unit="h:mm" format={duration}/></div>{/if}</Card>{/if}
 {#if sleepData.overnight.length}<Card title="Overnight"><div class="tiles">{#each sleepData.overnight as tile}{@const route=tileRoute(tile.kind)}{@const read=tile.series.filter((v):v is number=>v!=null)}{@const d=tile.kind==='respiration'?1:0}<svelte:element this={route?'button':'div'} class="tile" class:open={!!route} role={route?undefined:'group'} aria-label={`${OVERNIGHT[tile.kind]} ${num(tile.value,d)} ${tile.unit}${read.length?`, ${num(Math.min(...read),d)} to ${num(Math.max(...read),d)} overnight`:''}`} onclick={route?()=>navigate(route):undefined}><span><Icon name={tile.kind==='heartRate'?'heart':tile.kind==='spo2'?'drop':tile.kind==='respiration'?'lungs':'hrv'} size={14}/><span class="muted">{OVERNIGHT[tile.kind]}</span>{#if route}<Icon name="chevron" size={13}/>{/if}</span><b class="n">{num(tile.value,d)}<small>{tile.unit}</small></b>{#if read.length>1}<span class="range">Low {num(Math.min(...read),d)} · high {num(Math.max(...read),d)}</span>{/if}<Sparkline values={tile.series} w={128} h={30} color={colors.ink} fill={false}/>{#if sleepData.startLocal && sleepData.endLocal}<span class="span-times"><span>{clock(sleepData.startLocal)}</span><span>{clock(sleepData.endLocal)}</span></span>{/if}</svelte:element>{/each}</div></Card>{/if}
 {#if sleepData.consistency.length}<Card title="Consistency" extra="consistency">{#snippet aside()}{#if sleepData.consistencySpreadMin!=null && sleepData.consistencySpreadMin>=1}<span class="pill">Bedtime spread ±{Math.round(sleepData.consistencySpreadMin)} min</span>{/if}{/snippet}{#if sleepData.garminAverages}{@const avg=sleepData.garminAverages}<div class="averages"><div><span>Avg bedtime</span><b class="n">{avg.bedtime}</b></div><div><span>Avg wake</span><b class="n">{avg.wakeTime}</b></div><small>Garmin · last {avg.nights} {avg.nights===1?'night':'nights'}</small></div>{/if}<ConsistencyChart nights={sleepData.consistency} date={sleepData.date} bedWindow={sleepData.bedWindow} wakeWindow={sleepData.wakeWindow}/></Card>{/if}
 <Card title="Recorded naps" extra="naps">{#if sleepData.napsState==='unknown'}<p class="small">Naps not checked yet.</p>{:else if sleepData.napsState==='unavailable'}<p class="small">Couldn’t read naps from Garmin.</p>{:else if sleepData.napsState==='empty'}<p class="small">No naps recorded for {dateLabel(sleepData.date)}.</p>{:else}{#if sleepData.napsState==='stale'}<p class="small">Last checked naps may be out of date. Refresh to check Garmin.</p>{/if}{#each sleepData.naps as nap}<div class="nap-row"><span>{napWhen(nap.startLocal,'Start')} – {napWhen(nap.endLocal,'End')}</span><b class="n">{nap.durationMin!=null?duration(nap.durationMin):'Duration unavailable'}</b>{#if nap.feedback}<small>Garmin feedback · {nap.feedback.replaceAll('_',' ').toLowerCase()}</small>{/if}</div>{/each}{/if}</Card>
 <Card title="Sleep history"><HistoryChart compact values={[]} endDate={sleepData.date} label="Sleep score" metric="sleepScore" overlayOnly="sleepTime" initialRange="days30" nights openDay={day=>navigate('sleep/'+day)} more={()=>navigate('health')} moreLabel="More in Health"/></Card>
 {#if sleepData.alignment}<AlignmentCard alignment={sleepData.alignment}/>{/if}
 {#if sleepData.sleepPlan && !setTab && !historical}<SleepPlanCard plan={sleepData.sleepPlan} evidence/>{/if}
</div>{:else}<div class="dashboard" {@attach masonry}>{@render tonight?.()}</div>{/if}
<style>
 .stage-amounts{margin-top:20px}
 .stage-comparison-heading{font:400 12px/1.5 Inter;color:var(--text2);margin:0 0 12px}
 .tile.open{display:block;width:100%;border:0;color:var(--text);text-align:left;font:inherit}
 .tile>span:first-child{display:flex;align-items:center;gap:5px}
 .tile>span:first-child :global(svg:last-child){margin-left:auto;color:var(--text2)}
 .range{display:block;font-size:12px;color:var(--text2);margin:2px 0 4px}
 .span-times{display:flex;justify-content:space-between;font:500 11px Inter;color:var(--text2);margin-top:2px}
</style>
