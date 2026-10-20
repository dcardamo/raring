<script lang="ts">
 import type {VitalsCard} from '../generated/VitalsCard';
 import {vitalStatus,vitalSummary,vitalDecimals,vitalIcons,vitalName,rangeName,isSleepHr} from '../signals';
 import {toneColor} from '../charts';
 import {num,dateLabel} from '../format';
 import Card from './Card.svelte';
 import Icon from './Icon.svelte';
 import RangeBar from './RangeBar.svelte';
 let {vitals,compact=false,open,date}:{vitals:VitalsCard;compact?:boolean;open:(kind:string)=>void;date?:string}=$props();
 const id=$props.id();
 let summary=$derived(vitalSummary(vitals));
</script>

<Card title="Vitals" extra={compact?'vitals-monitor compact-monitor':'vitals-monitor'}>
 {#snippet aside()}<span class="source">{date?`Night ending ${dateLabel(date)}`:'Latest synced readings'}</span>{/snippet}
 <div class="monitor-heading" style:--signal={toneColor(summary.tone)}>
  <span class="monitor-icon"><Icon name={summary.tone==='good'?'check':'pulse'} size={20}/></span>
  <div><strong>{summary.headline}</strong><p>{summary.detail}</p></div>
 </div>
 <div class="signal-grid" class:compact>
  {#each vitals.readings as v}
   {@const state=vitalStatus(v)}
   {@const decimals=vitalDecimals(v.kind)}
   <button class="signal-tile" aria-label={'View '+vitalName(v)} aria-describedby={id+'-'+v.kind} onclick={()=>open(v.kind)} style:--signal={toneColor(state.tone)}>
    <span id={id+'-'+v.kind} class="sr-only">{num(v.value,decimals)} {v.unit}. {state.label}.</span>
    <span class="signal-name"><Icon name={vitalIcons[v.kind]} size={16}/><span>{compact?(isSleepHr(v)?'Sleep HR':({restingHr:'RHR',hrv:'HRV',respiration:'Resp',spo2:'SpO₂',skinTemp:'Skin'} as Record<string,string>)[v.kind]):vitalName(v)}</span>{#if !compact}<Icon name="arrow-right" size={12}/>{/if}</span>
    <span class="signal-value n">{v.kind==='skinTemp' && (v.value??0)>0?'+':''}{num(v.value,decimals)}{#if v.value!=null}<small>{v.unit}</small>{/if}</span>
    {#if compact}<span class="signal-dot" title={state.label}></span>
    {:else}
     <span class="signal-state">{state.label}</span>
     <RangeBar value={v.value} scale={v.scale} range={v.normal} color={toneColor(state.tone)}/>
     <span class="signal-range">{#if v.normal}{rangeName(v)} <span class="nowrap">{num(v.normal[0],decimals)}–{num(v.normal[1],decimals)} {v.unit}</span>{:else if v.status==='ONBOARDING' && v.value!=null}{:else if v.status==='ONBOARDING'}Garmin baseline building{:else if v.status}Garmin range unavailable{:else if v.kind==='hrv'}Awaiting Garmin baseline{:else}Needs 14 recorded nights{/if}</span>
    {/if}
   </button>
  {/each}
 </div>
 {#if summary.outside>=2}<p class="signal-warning">Several readings are outside your personal ranges. Take it easy and check how you feel.</p>{/if}
 {#if !compact}
  <details><summary>How your ranges work</summary><p>Garmin's Health Status baseline and status come first. Sleep HR is an overnight average, not resting HR. Where Garmin reports no Health Status metric, Raring uses the middle 80% of the last 30 recorded nights after at least 14 (HRV retains Garmin's own baseline). Garmin's seven-day HRV status can differ from its nightly Health Status. Missing or building readings are never in range. These are personal comparisons, not medical targets or a diagnosis.</p></details>
 {/if}
</Card>

<style>
 .sr-only{position:absolute;width:1px;height:1px;overflow:hidden;clip-path:inset(50%);white-space:nowrap}
 .nowrap{white-space:nowrap}
 .source{font-size:12px;font-weight:500;letter-spacing:0;text-transform:none;color:var(--text2)}
 .monitor-heading{display:flex;align-items:center;gap:10px;margin-bottom:18px}
 .monitor-icon{display:grid;place-items:center;flex:none;width:36px;height:36px;border:1px solid color-mix(in srgb,var(--signal) 35%,transparent);border-radius:12px;color:var(--signal);background:color-mix(in srgb,var(--signal) 8%,transparent)}
 strong{font:500 18px/1.25 Outfit;color:var(--signal)}
 .monitor-heading p{font-size:11px;color:var(--text2);margin-top:4px}
 .signal-grid{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:10px}
 .signal-tile{background:var(--surface2);border:1px solid transparent;border-radius:var(--radius-control);padding:13px;min-width:0;text-align:left;display:flex;flex-direction:column}
 .signal-tile:active{background:rgba(255,255,255,.08)}
 .signal-name{display:flex;align-items:center;gap:6px;font-size:11px;color:var(--text2);min-height:16px}
 .signal-name>span{flex:1}
 .signal-value{display:block;font-size:32px;font-weight:500;margin:8px 0 4px;white-space:nowrap}
 .signal-value small{font:400 11px Inter;color:var(--text2);margin-left:3px}
 .signal-state{color:var(--signal);font-size:11px;margin-bottom:14px}
 .signal-range{display:block;font-size:12px;color:var(--text2);margin-top:10px;line-height:1.5}
 .signal-tile:last-child:nth-child(odd){grid-column:1/-1}
 .signal-tile:last-child:nth-child(odd) .signal-value{margin-top:4px}
 .signal-grid.compact{grid-template-columns:repeat(5,minmax(0,1fr));gap:4px}
 .compact .signal-tile{padding:6px 0;align-items:center;border:0;background:none}
 .compact .signal-tile:last-child{grid-column:auto}
 .compact .signal-name{flex-direction:column;gap:6px}
 .compact .signal-value{font-size:21px;text-align:center;margin:9px 0 6px}
 .compact .signal-value small{display:block;font-size:9px;margin:2px 0 0;min-height:12px}
 .signal-dot{width:5px;height:5px;border-radius:50%;background:var(--signal)}
 .signal-warning{font-size:12px;line-height:1.5;color:var(--fair);padding:12px 0 0}
 details{border-top:1px solid var(--line);margin-top:16px;color:var(--text2);font-size:11px;line-height:1.6}
 summary{padding:14px 0 2px;min-height:44px;cursor:pointer}
 details p{padding-top:6px}
 summary:focus-visible{outline:2px solid var(--ink);outline-offset:2px}
</style>
