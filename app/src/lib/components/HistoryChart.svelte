<script lang="ts">
 import {untrack,getContext,onDestroy} from 'svelte';
 import type {ChartStateContext} from '../chart-state';
 import {addDays,historyAxis,readinessTone} from '../history';
 import {dateLabel} from '../format';
 import {colors,toneColor} from '../charts';
 import {ranges,localPoints,pointLabel,axisLabel,bucketDays,trendValue,trendNames,trendDecimals,defaultOverlay,changeValue,firstDay} from '../trends';
 import * as api from '../api';
 import type {TrendMetric} from '../generated/TrendMetric';
 import type {TrendRange} from '../generated/TrendRange';
 import type {TrendView} from '../generated/TrendView';
 import type {TrendPoint} from '../generated/TrendPoint';
 import Icon from './Icon.svelte';
 import RangePicker from './RangePicker.svelte';

 // Without `metric` the chart shows the screen's own 7 or 30 days, ending on `endDate`. With it, longer ranges and comparisons
 // come from the core, which averages long ranges into weeks or months.
 // `compact` is a summary card's chart: its own metric, four ranges, tap to read a day; `more` opens the full tool.
 let {values,endDate='',label,unit='',decimals=0,band=null,bandLabel='Your usual range',readiness=false,nights=false,color=colors.ink,metric,initialRange='days7',compact=false,ranges:compactRanges=['days7','days30','days90','year'],more,moreLabel='Compare and all history',openDay=null,overlayOnly}:{
  values:(number|null)[];endDate?:string;label:string;unit?:string;decimals?:number;band?:[number,number]|null;
  bandLabel?:string;readiness?:boolean;nights?:boolean;color?:string;metric?:TrendMetric;initialRange?:TrendRange;compact?:boolean;ranges?:TrendRange[];more?:()=>void;moreLabel?:string;openDay?:((date:string)=>void)|null;
  // One fixed metric with a single overlay toggle instead of the explorer's metric choice and comparisons.
  overlayOnly?:TrendMetric;
 }=$props();
 const id=$props.id();
 // A caller can retain controls when its canonical detail is reopened by Back.
 const saved=untrack(()=>compact?undefined:getContext<ChartStateContext|undefined>('chart-state')?.(metric ?? label));
 let range=$state<TrendRange>(saved?.initial?.range ?? untrack(()=>initialRange)),offset=$state(saved?.initial?.offset ?? 0),width=$state(320);
 const generation=getContext<()=>number>('trend-generation') ?? (()=>0);
 let primary=$state<TrendMetric|null>(saved?.initial?.primary ?? null);
 let activeMetric=$derived(primary ?? metric);
 let customStart=$state(saved?.initial?.customStart ?? ''),customEnd=$state(saved?.initial?.customEnd ?? ''),comparisonStart=$state(saved?.initial?.comparisonStart ?? '');
 let compare=$state<'off'|'previous'|'metric'>(saved?.initial?.compare ?? 'off'),overlayMetric=$state<TrendMetric|null>(saved?.initial?.overlayMetric ?? null);
 onDestroy(()=>saved?.save({range,offset,primary,customStart,customEnd,comparisonStart,compare,overlayMetric}));
 let view=$state<TrendView|null>(null),loadError=$state('');
 let days=$derived(ranges.find(r=>r.range===range)?.days ?? 0);
 let local=$derived(!activeMetric);
 let busy=$state(false);
 let shown=$derived(local?null:view);
 // Pickers stop at the core's bounds; the core still rejects typed dates outside them.
 let latestDay=$derived(endDate || shown?.to || new Date().toLocaleDateString('en-CA'));
 let bucket=$derived(shown?.bucket ?? 'day');
 let points=$derived<TrendPoint[]>(local?localPoints(values,endDate,days):shown?.series.points ?? []);
 let overlay=$derived(compare==='metric'?shown?.overlay ?? null:null);
 let previous=$derived(compare==='previous'?shown?.previous ?? null:null);
 let count=$derived(points.length);
 let index=$derived(Math.max(0,count-1-Math.min(offset,count-1)));
 let selected=$derived(points[index] as TrendPoint|undefined);
 let period=$derived(nights?'nights':'days');
 let activeLabel=$derived(primary?trendNames[primary]:label);
 let activeUnit=$derived(primary?shown?.series.unit??'':unit);
 let activeDecimals=$derived(primary?trendDecimals(primary):decimals);
 let activeReadiness=$derived(activeMetric==='readiness' || !activeMetric && readiness);
 // Garmin's estimates arrive every few days or weeks: open on the latest one, not an empty latest day.
 let sparse=$derived(activeMetric==='vo2max'||activeMetric==='fitnessAge');
 const latestOffset=(list:TrendPoint[])=>{const last=list.findLastIndex(p=>p.value!=null);return last<0?0:list.length-1-last};
 $effect(()=>{
  generation();
  if(!activeMetric)return;
  const query={metric:activeMetric,range,end:range==='custom'?customEnd:endDate||null,start:range==='custom'?customStart:null,comparisonStart:compare==='previous'?comparisonStart||null:null,overlay:compare==='metric'?overlayMetric:null,previous:compare==='previous'};
  let cancelled=false;
  view=null;busy=true;loadError='';
  api.trend(query)
   .then(next=>{if(!cancelled){view=next;busy=false;if(sparse)offset=latestOffset(next.series.points)}})
   .catch(e=>{if(!cancelled){loadError=String(e);busy=false}});
  return ()=>{cancelled=true};
 });
 // Garmin rates single days; averaged weeks and months and other metrics are ink.
 let levels=$derived(activeReadiness && bucket==='day');
 let shownBand=$derived(!primary && bucket==='day'?band:null);
 // Readiness keeps Garmin's 0–100 with its Moderate boundary; other metrics tick at round values.
 let axis=$derived(activeReadiness?{scale:[0,100] as [number,number],ticks:[0,50,100]}:historyAxis([...points,...(previous?.points ?? [])].map(p=>p.value),shownBand,activeUnit));
 let scale=$derived(axis.scale);
 let axis2=$derived(overlay?historyAxis(overlay.points.map(p=>p.value),null,overlay.unit):{scale:[0,1] as [number,number],ticks:[]});
 let scale2=$derived(axis2.scale);
 let W=$derived(Math.max(280,Math.round(width)));
 let H=$derived(W>=560?200:154);
 const L=38;
 let R=$derived(overlay?W-40:W-8);
 const x=(i:number)=>L+(i+.5)*(R-L)/Math.max(count,1);
 const y=(v:number,s:[number,number]=scale)=>12+(s[1]-v)/(s[1]-s[0])*(H-28);
 const path=(list:TrendPoint[],s:[number,number])=>list.map((p,i)=>p.value==null?'':`${i && list[i-1].value!=null?'L':'M'}${x(i).toFixed(1)},${y(p.value,s).toFixed(1)}`).join('');
 let present=$derived(points.filter(p=>p.value!=null).length);
 let bucketName=$derived(bucket==='week'?'weeks':bucket==='month'?'months':period);
 let overlayDecimals=$derived(overlay?trendDecimals(overlay.metric):0);
 const show=(v:number|null|undefined)=>trendValue(v,activeUnit,activeDecimals);
 // "/ 100" describes one reading, not an average or a change.
 let tileUnit=$derived(activeUnit.startsWith('/') || activeUnit==='min'?'':activeUnit);
 let canOpenDay=$derived(!!openDay && !!selected && bucket==='day' && selected.value!=null && selected.start<latestDay);
 let selectedDate=$derived(selected?bucket==='day'?dateLabel(selected.start)+(range==='custom'||range==='all'?' '+selected.start.slice(0,4):''):pointLabel(selected,bucket):'');
 let selectedText=$derived(selected?`${selectedDate}: ${selected.value==null?'Not recorded':show(selected.value)+' '+(activeUnit==='min'?'h:mm':activeUnit)}`:'');
 let barWidth=$derived(Math.max(1.5,Math.min(22,(R-L)/Math.max(count,1)*.62)));
 const pairs=Object.keys(trendNames) as TrendMetric[];
 let choices=$derived(pairs.filter(m=>m!==activeMetric));
 function setRange(next:TrendRange){
  if(next==='custom' && !customStart){customEnd=endDate||shown?.to||new Date().toISOString().slice(0,10);customStart=addDays(customEnd,-29)}
  range=next;offset=0;if(next==='all' && compare==='previous')compare='off';
 }
 function setCompare(next:'off'|'previous'|'metric'){
  compare=next;
  if(next==='metric' && activeMetric && (!overlayMetric || overlayMetric===activeMetric))overlayMetric=defaultOverlay(activeMetric);
 }
 // The chart draws at its card's width; a frame's delay keeps resizing out of the masonry layout pass.
 function measure(node:HTMLElement){
  let frame=0;
  const observer=new ResizeObserver(([entry])=>{cancelAnimationFrame(frame);frame=requestAnimationFrame(()=>width=entry.contentRect.width)});
  observer.observe(node);
  return ()=>{cancelAnimationFrame(frame);observer.disconnect()};
 }
 function selectFromChart(e:MouseEvent) {
  if(e.detail===0 || !count)return;
  const rect=e.currentTarget instanceof HTMLElement?e.currentTarget.getBoundingClientRect():null;
  if(rect)offset=count-1-Math.max(0,Math.min(count-1,Math.floor(((e.clientX-rect.left)/rect.width*W-L)/(R-L)*count)));
 }
</script>

<div class="history" {@attach measure} aria-busy={busy}>
 <div class="history-top">
  {#if metric && !compact && !overlayOnly}<label class="metric-choice"><span class="sr-only">Primary metric</span><select aria-label="Primary metric" value={activeMetric} onchange={(e)=>{primary=e.currentTarget.value as TrendMetric;offset=0;if(overlayMetric===primary)overlayMetric=defaultOverlay(primary)}}>{#each pairs as m}<option value={m}>{trendNames[m]}</option>{/each}</select></label>{:else if !compact}<span class="history-label">{label}</span>{/if}
  <RangePicker value={range} onchange={setRange} label="History period" custom={!!metric} local={!metric} {nights} only={compact?compactRanges:undefined}/>
 </div>
 {#if view?.freshness?.stale && view.freshness.recordedOn}<p class="stale-note" role="note">Last reading {dateLabel(view.freshness.recordedOn)} · {view.freshness.ageDays} days ago</p>{/if}
 {#if range==='custom'}<div class="dates"><label>Block start<input aria-label="Block start" type="date" min={firstDay} max={customEnd || latestDay} bind:value={customStart}/></label><label>Block end<input aria-label="Block end" type="date" min={customStart || firstDay} max={latestDay} bind:value={customEnd}/></label></div>{/if}
 <div class="history-reading" aria-live="polite" aria-atomic="true">
  <div class="reading-values">
   <div><b class="n" style:color={selected?.value==null?toneColor('neutral'):levels?toneColor(readinessTone(selected.value)):color}>{show(selected?.value)}</b><span>{selected?.value==null?'Not recorded':bucket==='week'?'weekly average':bucket==='month'?'monthly average':activeUnit==='min'?'h:mm':activeUnit}</span></div>
   {#if overlay && selected}<div class="overlay-reading"><b class="n">{trendValue(overlay.points[index]?.value,overlay.unit,overlayDecimals)}</b><span>{overlay.name}</span></div>{/if}
   {#if previous && selected}<div class="overlay-reading"><b class="n">{show(previous.points[index]?.value)}</b><span>{comparisonStart?'comparison block':'period before'}</span></div>{/if}
  </div>
  <span>{selectedDate}{#if bucket!=='day' && selected}<small>{selected.days} of {bucketDays(selected)} days recorded</small>{:else if offset===0 && selected}<small>{range==='custom'?'Block end':nights?'Latest night':'Latest day'}</small>{/if}</span>
 </div>
 {#if canOpenDay && selected && openDay}<button class="open-day" onclick={()=>openDay(selected.start)}>Open {dateLabel(selected.start)}<Icon name="chevron" size={15}/></button>{/if}
 <div data-chart="history">
 <button class="chart-surface" tabindex={compact?0:-1} aria-hidden={compact?undefined:true} aria-label={compact?`${activeLabel} chart. ${selectedText}. Left and right arrows choose a day.`:undefined} onclick={selectFromChart} onkeydown={compact?(e)=>{if(e.key==='ArrowLeft'&&index>0){e.preventDefault();offset++}else if(e.key==='ArrowRight'&&index<count-1){e.preventDefault();offset--}}:undefined}>
  <svg viewBox={`0 0 ${W} ${H}`} role="img" aria-label={`${activeLabel}, ${count} ${bucketName}. ${present} recorded. Missing readings are gaps.`}>
   {#if shownBand}
    <rect x={L} y={y(shownBand[1])} width={R-L} height={Math.max(1,y(shownBand[0])-y(shownBand[1]))} fill="var(--chart-band)"/>
    <line x1={L} x2={R} y1={y(shownBand[0])} y2={y(shownBand[0])} stroke="#fff" opacity=".2" stroke-dasharray="3 4"/>
    <line x1={L} x2={R} y1={y(shownBand[1])} y2={y(shownBand[1])} stroke="#fff" opacity=".2" stroke-dasharray="3 4"/>
   {/if}
   {#each axis.ticks as tick}
    <text x="0" y={y(tick)+3} fill="var(--text3)" font-size="11">{show(tick)}</text>
    <line x1={L} x2={R} y1={y(tick)} y2={y(tick)} stroke="var(--chart-grid)" stroke-width="var(--stroke-grid)"/>
   {/each}
   {#if overlay}{#each axis2.ticks as tick}<text x={W} y={y(tick,scale2)+3} fill="var(--text3)" font-size="11" text-anchor="end">{trendValue(tick,overlay.unit,overlayDecimals)}</text>{/each}{/if}
   {#if count}<line x1={x(index)} x2={x(index)} y1="8" y2={H-12} stroke="currentColor" opacity=".35" stroke-dasharray="2 3"/>{/if}
   {#if previous}<path class="history-previous" d={path(previous.points,scale)} fill="none" stroke={colors.ink} stroke-opacity=".45" stroke-width="1.6" stroke-dasharray="4 4" stroke-linecap="round"/>{/if}
   {#if !activeReadiness}<path d={path(points,scale)} fill="none" stroke={color} stroke-width="var(--stroke-chart)" stroke-linecap="round" stroke-linejoin="round"/>{/if}
   {#each points as p,i}
    {#if p.value!=null}
     {#if activeReadiness}
      <rect class="history-bar" x={x(i)-barWidth/2} y={Math.min(H-18,y(p.value))} width={barWidth} height={Math.max(2,H-16-y(p.value))} rx={Math.min(2,barWidth/2)} fill={levels?toneColor(readinessTone(p.value)):colors.ink} opacity={index===i?1:overlay?.4:levels?.55:.38}/>
     {:else if index===i || count<=40 || (points[i-1]?.value==null && points[i+1]?.value==null)}
      <circle cx={x(i)} cy={y(p.value)} r={index===i?4:2.5} fill={color} stroke="var(--bg)" stroke-width={index===i?2:0}/>
     {/if}
    {:else}<circle class="history-gap" cx={x(i)} cy={H-9} r="1.5" fill="currentColor" opacity=".3"/>{/if}
   {/each}
   {#each [previous,overlay] as other}
    {#if other}{#each other.points as p,i}{#if p.value!=null && other.points[i-1]?.value==null && other.points[i+1]?.value==null}<circle class="isolated-point" cx={x(i)} cy={y(p.value,other===overlay?scale2:scale)} r="2.5" fill={colors.secondary}/>{/if}{/each}{/if}
   {/each}
   {#if overlay}
    <path class="history-overlay" d={path(overlay.points,scale2)} fill="none" stroke={colors.secondary} stroke-width="1.6" stroke-dasharray="6 3" stroke-linecap="round" stroke-linejoin="round"/>
    {#if overlay.points[index]?.value!=null}<circle cx={x(index)} cy={y(overlay.points[index].value!,scale2)} r="4" fill="var(--bg)" stroke={colors.secondary} stroke-width="2"/>{/if}
   {/if}
  </svg>
 </button>
 {#if count}<div class="history-axis"><span>{bucket==='day'?dateLabel(points[0].start):axisLabel(points[0],bucket)}</span><span>{bucket==='day'?dateLabel(points[count-1].start):axisLabel(points[count-1],bucket)}</span></div>{/if}
 {#each [previous,overlay] as other}{#if other}<div class="sr-only">{other.name}: {other.days} recorded days</div>{/if}{/each}
 {#if !compact}<div class="history-controls">
  <button aria-label="Previous reading" disabled={index===0} onclick={()=>offset++}><Icon name="arrow-left" size={17}/></button>
  <input aria-label={'Selected '+activeLabel.toLowerCase()+' date'} aria-describedby={id} aria-valuetext={selectedText} type="range" min="0" max={Math.max(0,count-1)} step="1" value={index} oninput={(e)=>offset=count-1-Number(e.currentTarget.value)}/>
  <button aria-label="Next reading" disabled={index>=count-1} onclick={()=>offset--}><Icon name="arrow-right" size={17}/></button>
 </div>{/if}
 <span id={id} class="sr-only">{activeLabel}, {count} {bucketName}. {present} recorded. Missing readings are gaps. Use the slider to select a date.</span>
 <div class="history-legend">
  {#if shownBand}<span><i style:background="rgba(255,255,255,.13)"></i>{bandLabel}</span>{/if}
  {#if levels}<span><i style:background={colors.poor}></i>Low / Poor</span><span><i style:background={colors.fair}></i>Moderate</span><span><i style:background={colors.good}></i>High / Prime</span>{/if}
  {#if bucket!=='day'}<span><i style:background={colors.ink}></i>{bucket==='week'?'Weekly':'Monthly'} average of recorded days</span>{/if}
  {#if overlay}<span><i class="line" style:border-color={colors.secondary} style:border-top-style="dashed"></i>{overlay.name}, right axis</span>{/if}
  {#if previous}<span><i class="line dashed"></i>{comparisonStart?'Comparison block':range==='year'?'Year before':range==='custom'?'Previous block':days+' '+period+' before'}</span>{/if}
  <span class="record-count">{present} / {count} {bucket==='day'?'recorded':bucketName}</span>
 </div>
 </div>
 {#if overlayOnly}<div class="compare"><button class="overlay-toggle" aria-pressed={compare==='metric'} onclick={()=>{overlayMetric=overlayOnly;compare=compare==='metric'?'off':'metric'}}>Overlay {trendNames[overlayOnly].toLowerCase()}</button></div>{/if}
 {#if compact && more}<button class="link-row history-more" onclick={more}>{moreLabel}<Icon name="chevron" size={15}/></button>
 {:else if metric && !compact && !overlayOnly}
  <div class="compare">
   <span class="compare-label">Compare</span>
   <div class="periods compare-options" role="group" aria-label="Compare">
    <button class:active={compare==='off'} aria-pressed={compare==='off'} onclick={()=>setCompare('off')}>Off</button>
    <button class:active={compare==='previous'} aria-pressed={compare==='previous'} disabled={range==='all'} onclick={()=>setCompare('previous')}>Previous</button>
    <button class:active={compare==='metric'} aria-pressed={compare==='metric'} onclick={()=>setCompare('metric')}>Metric</button>
   </div>
   {#if compare==='metric'}
    <select aria-label="Metric to overlay" value={overlayMetric} onchange={(e)=>overlayMetric=e.currentTarget.value as TrendMetric}>{#each choices as m}<option value={m}>{trendNames[m]}</option>{/each}</select>
   {/if}
  </div>
  {#if compare==='previous'}<div class="dates"><label>Comparison block start<input aria-label="Comparison block start" type="date" min={firstDay} max={latestDay} bind:value={comparisonStart}/></label><button class="reset-date" onclick={()=>comparisonStart=''}>Use preceding period</button></div>{/if}
  {#if previous && shown}
   <p class="history-note">{shown.from} – {shown.to}<br/>Against {previous.points[0]?.start} – {previous.points.at(-1)?.end} · aligned by elapsed day</p>
   <div class="period-averages">
    <div><span>Average</span><b class="n">{show(shown.series.average)}<small>{tileUnit}</small></b><em>{shown.series.days} {period}</em></div>
    <div><span>{comparisonStart?'Comparison block':'Period before'}</span><b class="n muted">{show(previous.average)}<small>{tileUnit}</small></b><em>{previous.days} {period}</em></div>
    <div><span>Change</span><b class="n">{changeValue(shown.change,activeUnit,activeDecimals)}<small>{tileUnit}</small></b></div>
   </div>
  {/if}
  {#if compare==='metric'}<p class="history-note">Each line keeps its own scale. Moving together doesn't mean one causes the other.</p>{/if}
  <details class="history-note"><summary>About this history</summary><p>Each day counts once. Weeks and months average recorded days only; gaps never count as zero. Period averages use all recorded days, not averages of averages.</p>{#if activeMetric==='readiness'}<p>Morning readiness readings, including today. The main gauge can show a later update.</p>{:else if activeMetric==='bodyBattery'}<p>The highest Body Battery recorded each day, not the latest level.</p>{:else if activeMetric==='acuteLoad'}<p>Garmin's acute load from the day's latest readiness snapshot.</p>{:else if ['hrv','sleepHr','respiration','spo2','skinTemp'].includes(activeMetric??'')}<p>Health Status where available, otherwise the nightly record. Garmin's respiration values can differ slightly between these sources. Current baselines aren't historical targets.</p>{/if}</details>
 {/if}
 {#if loadError}<p class="history-empty" role="status">Couldn't read this range: {loadError}</p>
 {:else if !local && !shown}<p class="history-empty">Reading your history…</p>
 {:else if !present}<p class="history-empty">No readings in this period. Missing data is never shown as zero.</p>{/if}
</div>

<style>
 .stale-note{margin:6px 0 0;font:600 12px Inter;color:var(--fair)}
 .sr-only{position:absolute;width:1px;height:1px;overflow:hidden;clip-path:inset(50%);white-space:nowrap}
 .history-top{display:flex;justify-content:space-between;align-items:center;gap:12px;flex-wrap:wrap}
 .history-label{color:var(--text2);font-size:12px}
 .history-reading{display:flex;justify-content:space-between;align-items:center;margin:20px 0 14px;gap:12px}
 .reading-values{display:flex;align-items:baseline;gap:16px;flex-wrap:wrap}
 .history-reading b{font-size:var(--type-reading);font-weight:500;letter-spacing:-.03em}
 .history-reading .overlay-reading b{font-size:24px;color:var(--text2)}
 .history-reading .reading-values span{color:var(--text2);font-size:12px;margin-left:5px}
 .history-reading>span{font-size:12px;text-align:right;flex:none;max-width:130px}
 .history-reading small{display:block;color:var(--text2);font-size:12px;margin-top:4px}
 .chart-surface{display:block;width:100%;padding:0;border:0;background:none;touch-action:pan-y}
 svg{display:block;width:100%;height:auto;overflow:visible;font-family:Inter,sans-serif}
 .history-axis{display:flex;justify-content:space-between;gap:8px;padding-left:38px;font-size:11px;color:var(--text2)}
 .open-day{display:inline-flex;align-items:center;gap:4px;margin:0 0 8px;padding:0;border:0;background:none;color:var(--ink);font:600 13px Inter;min-height:40px}
 .history-controls{display:flex;align-items:center;gap:12px;margin-top:6px}
 .history-controls button{display:grid;place-items:center;flex:none;background:var(--card);border:1px solid var(--line);border-radius:12px;width:44px;height:44px}
 input{width:100%;min-width:0;height:44px;margin:0;accent-color:var(--ink);cursor:ew-resize}
 .history-legend{display:flex;gap:4px 10px;flex-wrap:wrap;font-size:12px;color:var(--text2);margin-top:12px;align-items:center}
 .history-legend span{display:inline-flex;align-items:center;gap:5px}
 .history-legend i{width:6px;height:6px;border-radius:2px;opacity:.7}
 .history-legend i.line{width:14px;height:0;border-radius:0;border-top:2px solid;opacity:1}
 .history-legend i.dashed{border-top:2px dashed rgba(228,232,240,.55)}
 .record-count{margin-left:auto}
 .compare{display:flex;align-items:center;gap:10px;flex-wrap:wrap;margin-top:14px;padding-top:14px;border-top:1px solid var(--line)}
 .overlay-toggle{min-height:40px;padding:8px 14px;border-radius:12px;border:1px solid var(--line);background:rgba(255,255,255,.04);color:var(--text);font:600 13px Inter}.overlay-toggle[aria-pressed=true]{border-color:var(--ink)}
 .compare-label{font-size:11px;font-weight:600;letter-spacing:.14em;text-transform:uppercase;color:var(--text2)}
 .periods{display:flex;gap:3px;border:1px solid var(--line);border-radius:13px;padding:3px;flex:none;flex-wrap:wrap}
 .periods button{border:0;border-radius:10px;background:none;color:var(--text2);min-height:38px;padding:0 10px;font-size:12px}
 .periods .active{background:var(--surface3);color:var(--text)}
 .periods button:disabled{opacity:.4}
 .compare-options{flex:1}
 .compare-options button{flex:1}
 select{width:100%;min-height:44px;border-radius:12px;border:1px solid var(--line);background:var(--card);color:var(--text);padding:0 12px;font:600 14px Inter,sans-serif}
 .period-averages{display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:8px;margin-top:14px}
 .period-averages div{background:rgba(255,255,255,.035);border-radius:var(--radius-control);padding:12px;min-width:0}
 .period-averages span,.period-averages em{display:block;font-size:12px;color:var(--text2);font-style:normal}
 .period-averages b{display:block;font-size:22px;font-weight:600;margin:4px 0 2px}
 .period-averages b.muted{color:var(--text2)}
 .period-averages small{font-size:12px;color:var(--text2);margin-left:2px;font-family:Inter,sans-serif;font-weight:500}
 details p{margin-top:8px}summary{cursor:pointer}
 .history-note,.history-empty{font-size:12px;line-height:1.5;color:var(--text2);margin-top:12px}
 .dates{display:flex;gap:12px;flex-wrap:wrap;margin-top:12px}.dates label{flex:1;min-width:120px;font-size:12px;color:var(--text2)}.dates input{display:block;font:inherit;color:var(--text);background:var(--card);border:1px solid var(--line);border-radius:8px;padding:6px;cursor:auto;box-sizing:border-box}.reset-date{background:var(--card);border:1px solid var(--line);border-radius:8px;padding:8px;color:var(--text2)}.metric-choice select{width:auto;max-width:100%;font-size:12px}.history-top{min-width:0}
 @media(min-width:660px){.compare-options{flex:none;min-width:260px}select{width:auto;min-width:200px}}
</style>
