<script lang="ts">
 import {historyPoints,historyScale,readinessTone} from '../history';
 import {dateLabel,num} from '../format';
 import {colors,toneColor} from '../charts';
 import Icon from './Icon.svelte';

 let {values,endDate,label,unit='',decimals=0,band=null,bandLabel='Your usual range',readiness=false,nights=false,color=colors.ink}:{
  values:(number|null)[];endDate:string;label:string;unit?:string;decimals?:number;band?:[number,number]|null;
  bandLabel?:string;readiness?:boolean;nights?:boolean;color?:string;
 }=$props();
 const id=$props.id();
 let days=$state(7),offset=$state(0);
 let points=$derived(historyPoints(values,endDate,days));
 let index=$derived(days-1-Math.min(offset,days-1));
 let selected=$derived(points[index]);
 let scale=$derived(readiness?[0,100] as [number,number]:historyScale(points.map(p=>p.value),band));
 const x=(i:number)=>38+(i+.5)*274/days;
 const y=(v:number)=>12+(scale[1]-v)/(scale[1]-scale[0])*126;
 let path=$derived(points.map((p,i)=>p.value==null?'':`${i && points[i-1].value!=null?'L':'M'}${x(i)},${y(p.value)}`).join(''));
 let present=$derived(points.filter(p=>p.value!=null).length);
 let selectedText=$derived(`${dateLabel(selected.date)}: ${selected.value==null?'Not recorded':num(selected.value,decimals)+' '+unit}`);
 function selectFromChart(e:MouseEvent) {
  if(e.detail===0)return;
  const rect=e.currentTarget instanceof HTMLElement?e.currentTarget.getBoundingClientRect():null;
  if(rect)offset=days-1-Math.max(0,Math.min(days-1,Math.floor(((e.clientX-rect.left)/rect.width*320-38)/274*days)));
 }
</script>

<div class="history">
 <div class="history-top">
  <span class="history-label">{label}</span>
  <div class="periods" role="group" aria-label="History period">
   {#each [7,30] as period}<button class:active={days===period} aria-pressed={days===period} onclick={()=>{days=period;offset=0}}>{period} {nights?'nights':'days'}</button>{/each}
  </div>
 </div>
 <div class="history-reading" aria-live="polite" aria-atomic="true">
  <div><b class="n" style:color={selected.value==null?toneColor('neutral'):readiness?toneColor(readinessTone(selected.value)):color}>{num(selected.value,decimals)}</b><span>{selected.value==null?'Not recorded':unit}</span></div>
  <span>{dateLabel(selected.date)}{#if offset===0}<small>Latest {nights?'night':'day'}</small>{/if}</span>
 </div>
 <button class="chart-surface" tabindex="-1" aria-hidden="true" onclick={selectFromChart}>
  <svg viewBox="0 0 320 154" role="img" aria-label={`${label}, ${days} ${nights?'nights':'days'}. ${present} recorded. Missing readings are gaps.`}>
   {#if band}
    <rect x="38" y={y(band[1])} width="274" height={Math.max(1,y(band[0])-y(band[1]))} fill="#fff" opacity=".13"/>
    <line x1="38" x2="312" y1={y(band[0])} y2={y(band[0])} stroke="#fff" opacity=".2" stroke-dasharray="3 4"/>
    <line x1="38" x2="312" y1={y(band[1])} y2={y(band[1])} stroke="#fff" opacity=".2" stroke-dasharray="3 4"/>
   {/if}
   {#each [scale[1],(scale[0]+scale[1])/2,scale[0]] as tick}
    <text x="0" y={y(tick)+3} fill="#8E98AC" font-size="10">{num(tick,decimals)}</text>
    <line x1="38" x2="312" y1={y(tick)} y2={y(tick)} stroke="currentColor" opacity=".08"/>
   {/each}
   <line x1={x(index)} x2={x(index)} y1="8" y2="142" stroke="currentColor" opacity=".35" stroke-dasharray="2 3"/>
   {#if !readiness}<path d={path} fill="none" stroke={color} stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>{/if}
   {#each points as p,i}
    {#if p.value!=null}
     {#if readiness}
      <rect class="history-bar" x={x(i)-Math.min(11,95/days)} y={Math.min(136,y(p.value))} width={Math.min(22,190/days)} height={Math.max(2,138-y(p.value))} rx="2" fill={toneColor(readinessTone(p.value))} opacity={index===i?1:.55}/>
     {:else}
      <circle cx={x(i)} cy={y(p.value)} r={index===i?4:2.5} fill={color} stroke="var(--bg)" stroke-width={index===i?2:0}/>
     {/if}
    {:else}<circle class="history-gap" cx={x(i)} cy="145" r="1.5" fill="currentColor" opacity=".3"/>{/if}
   {/each}
  </svg>
 </button>
 <div class="history-axis"><span>{dateLabel(points[0].date)}</span><span>{dateLabel(points[days-1].date)}</span></div>
 <div class="history-controls">
  <button aria-label="Previous reading" disabled={index===0} onclick={()=>offset++}><Icon name="arrow-left" size={17}/></button>
  <input aria-label={'Selected '+label.toLowerCase()+' date'} aria-describedby={id} aria-valuetext={selectedText} type="range" min="0" max={days-1} step="1" value={index} oninput={(e)=>offset=days-1-Number(e.currentTarget.value)}/>
  <button aria-label="Next reading" disabled={index===days-1} onclick={()=>offset--}><Icon name="arrow-right" size={17}/></button>
 </div>
 <span id={id} class="sr-only">{label}, {days} {nights?'nights':'days'}. {present} recorded. Missing readings are gaps. Use the slider to select a date.</span>
 <div class="history-legend">
  {#if band}<span><i style:background="rgba(255,255,255,.13)"></i>{bandLabel}</span>{/if}
  {#if readiness}<span><i style:background={colors.poor}></i>Low / Poor</span><span><i style:background={colors.fair}></i>Moderate</span><span><i style:background={colors.good}></i>High / Prime</span>{/if}
  <span class="record-count">{present} / {days} recorded</span>
 </div>
 {#if !present}<p class="history-empty">No readings in this period. Missing data is never shown as zero.</p>{/if}
</div>

<style>
 .sr-only{position:absolute;width:1px;height:1px;overflow:hidden;clip-path:inset(50%);white-space:nowrap}
 .history-top{display:flex;justify-content:space-between;align-items:center;gap:12px}
 .history-label{color:var(--text2);font-size:12px}
 .periods{display:flex;gap:3px;border:1px solid var(--line);border-radius:13px;padding:3px;flex:none}
 .periods button{border:0;border-radius:10px;background:none;color:var(--text2);min-height:38px;padding:0 10px;font-size:12px}
 .periods .active{background:rgba(255,255,255,.1);color:var(--text)}
 .history-reading{display:flex;justify-content:space-between;align-items:center;margin:20px 0 14px;gap:12px}
 .history-reading b{font-size:38px;font-weight:500}
 .history-reading div>span{color:var(--text2);font-size:12px;margin-left:5px}
 .history-reading>span{font-size:12px;text-align:right}
 .history-reading small{display:block;color:var(--text2);font-size:10px;margin-top:4px}
 .chart-surface{display:block;width:100%;padding:0;border:0;background:none;touch-action:pan-y}
 svg{display:block;width:100%;height:auto;overflow:visible;font-family:Inter,sans-serif}
 .history-axis{display:flex;justify-content:space-between;padding-left:12%;font-size:10px;color:var(--text2)}
 .history-controls{display:flex;align-items:center;gap:12px;margin-top:6px}
 .history-controls button{display:grid;place-items:center;flex:none;background:var(--card);border:1px solid var(--line);border-radius:12px;width:44px;height:44px}
 input{width:100%;min-width:0;height:44px;margin:0;accent-color:var(--ink);cursor:ew-resize}
 .history-legend{display:flex;gap:10px;flex-wrap:wrap;font-size:10px;color:var(--text2);margin-top:12px;align-items:center}
 .history-legend span{display:inline-flex;align-items:center;gap:5px}
 .history-legend i{width:6px;height:6px;border-radius:2px;opacity:.7}
 .record-count{margin-left:auto}
 .history-empty{font-size:12px;line-height:1.5;color:var(--text2);margin-top:16px}
</style>
