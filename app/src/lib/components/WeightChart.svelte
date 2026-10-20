<script lang="ts">
 import {smoothPath} from '../curve';
 import type {WeightPoint} from '../generated/WeightPoint';
 import {dateLabel,num} from '../format';
 import {historyAxis} from '../history';
 import {colors} from '../charts';
 import Icon from './Icon.svelte';
 // The shared history chart's layout for weight: Garmin weigh-ins as dots on a faint line, Raring's trend solid,
 // both in the neutral blue. Tap a point or use the arrow keys (compact), or the slider and buttons, to read one.
 let {points,unit,compact=false,color=colors.neutral}:{points:WeightPoint[],unit:string,compact?:boolean,color?:string}=$props();
 const id=$props.id();
 let width=$state(320),offset=$state(0);
 $effect(()=>{points;offset=0});
 let count=$derived(points.length);
 let index=$derived(Math.max(0,count-1-Math.min(offset,count-1)));
 let selected=$derived(points[index] as WeightPoint|undefined);
 let W=$derived(Math.max(280,Math.round(width)));
 let H=$derived(W>=560?200:154);
 const L=38;
 let R=$derived(W-8);
 let axis=$derived(historyAxis(points.flatMap(p=>[p.scale,p.trend]),null));
 let scale=$derived(axis.scale);
 let tickDigits=$derived(axis.ticks.some(t=>!Number.isInteger(t))?1:0);
 const stamp=(d:string)=>Date.parse(d+'T12:00:00Z');
 let first=$derived(points.length?stamp(points[0].date):0),span=$derived(points.length>1?Math.max(1,stamp(points.at(-1)!.date)-first):0);
 const x=(d:string)=>span?L+(R-L)*(stamp(d)-first)/span:(L+R)/2;
 const y=(v:number)=>12+(scale[1]-v)/(scale[1]-scale[0])*(H-28);
 let scalePath=$derived.by(()=>{
  const pts:[number,number|null][]=[];
  for(const p of points){if(p.scale==null)continue;if(p.breakScale)pts.push([0,null]);pts.push([x(p.date),y(p.scale)]);}
  return smoothPath(pts);
 });
 let trendPath=$derived(smoothPath(points.map(p=>[x(p.date),p.trend==null?null:y(p.trend)])));
 let weighIns=$derived(points.filter(p=>p.scale!=null).length);
 // Over about four months the ends read as months, like the shared chart's weekly ranges.
 let long=$derived(span>120*86_400_000);
 const axisDate=(d:string)=>long?new Date(d+'T12:00:00Z').toLocaleDateString('en-GB',{timeZone:'UTC',month:'short',year:'numeric'}):dateLabel(d);
 const label=(p:WeightPoint)=>`${dateLabel(p.date)} ${p.date.slice(0,4)}: scale ${p.scale==null?'not recorded':p.scale.toFixed(1)+' '+unit}, trend ${p.trend==null?'unavailable':p.trend.toFixed(1)+' '+unit}`;
 function measure(node:HTMLElement){
  let frame=0;
  const observer=new ResizeObserver(([entry])=>{cancelAnimationFrame(frame);frame=requestAnimationFrame(()=>width=entry.contentRect.width)});
  observer.observe(node);
  return ()=>{cancelAnimationFrame(frame);observer.disconnect()};
 }
 // The nearest point in time to the tap.
 function selectFromChart(e:MouseEvent){
  if(e.detail===0 || !count || !(e.currentTarget instanceof HTMLElement))return;
  const rect=e.currentTarget.getBoundingClientRect(),at=(e.clientX-rect.left)/rect.width*W;
  let best=0;points.forEach((p,i)=>{if(Math.abs(x(p.date)-at)<Math.abs(x(points[best].date)-at))best=i});
  offset=count-1-best;
 }
</script>
<div class="weight-chart" {@attach measure} data-chart="weight">
 <div class="plot-units"><span>Weight · {unit}</span></div>
 <button class="chart-surface" tabindex={compact?0:-1} aria-hidden={compact?undefined:true} aria-label={compact&&selected?`Weight chart. ${label(selected)}. Left and right arrows choose a reading.`:undefined} onclick={selectFromChart} onkeydown={compact?(e)=>{if(e.key==='ArrowLeft'&&index>0){e.preventDefault();offset++}else if(e.key==='ArrowRight'&&index<count-1){e.preventDefault();offset--}}:undefined}>
  <svg viewBox={`0 0 ${W} ${H}`} role="img" aria-label={`Scale weight and Raring trend, ${unit}, ${weighIns} weigh-ins. Scale gaps are not weigh-ins.`}>
   {#each axis.ticks as tick}
    <text x="0" y={y(tick)+3} fill="var(--text3)" font-size="11">{num(tick,tickDigits)}</text>
    <line x1={L} x2={R} y1={y(tick)} y2={y(tick)} stroke="var(--chart-grid)" stroke-width="var(--stroke-grid)"/>
   {/each}
   {#if selected}<line x1={x(selected.date)} x2={x(selected.date)} y1="8" y2={H-12} stroke="currentColor" opacity=".35" stroke-dasharray="2 3"/>{/if}
   <path class="scale-line" d={scalePath} fill="none" stroke={color} stroke-opacity=".35" stroke-width="1.5"/>
   <path class="trend-line" d={trendPath} fill="none" stroke={color} stroke-width="var(--stroke-chart)" stroke-linecap="round"/>
   {#each points as p}{#if p.scale!=null}<circle class="scale-mark" cx={x(p.date)} cy={y(p.scale)} r="2.5" fill={color} opacity=".7"/>{/if}{/each}
   {#if selected?.trend!=null}<circle class="trend-mark" cx={x(selected.date)} cy={y(selected.trend)} r="4" fill="var(--card)" stroke={color} stroke-width="2"/>{/if}
   {#if selected?.scale!=null}<circle cx={x(selected.date)} cy={y(selected.scale)} r="4" fill={color} stroke="var(--card)" stroke-width="2"/>{/if}
  </svg>
 </button>
 {#if count}<div class="history-axis"><span>{axisDate(points[0].date)}</span><span>{axisDate(points[count-1].date)}</span></div>{/if}
 {#if selected}
  <div class="history-reading" aria-live="polite" aria-atomic="true">
   <div class="reading-values">
    <div><b class="n" style:color={selected.scale==null?undefined:color}>{selected.scale==null?'—':num(selected.scale,1)}</b><span>{selected.scale==null?'Not weighed':unit+' · scale'}</span></div>
    {#if selected.trend!=null}<div class="overlay-reading"><b class="n">{num(selected.trend,1)}</b><span>trend</span></div>{/if}
   </div>
   <span>{dateLabel(selected.date)}{long?' '+selected.date.slice(0,4):''}{#if offset===0}<small>Latest weigh-in</small>{/if}</span>
  </div>
  {#if !compact}<div class="history-controls">
   <button aria-label="Previous reading" disabled={index===0} onclick={()=>offset++}><Icon name="arrow-left" size={17}/></button>
   <input aria-label="Weight readings" aria-describedby={id} aria-valuetext={label(selected)} type="range" min="0" max={Math.max(0,count-1)} step="1" value={index} oninput={(e)=>offset=count-1-Number(e.currentTarget.value)}/>
   <button aria-label="Next reading" disabled={index>=count-1} onclick={()=>offset--}><Icon name="arrow-right" size={17}/></button>
  </div>{/if}
  <span id={id} class="sr-only">{label(selected)}</span>
 {/if}
 <div class="history-legend">
  <span><i class="line" style:border-color={color}></i>Raring trend</span>
  <span><i class="dot" style:background={color}></i>Garmin weigh-in</span>
  <span class="record-count">{weighIns} {weighIns===1?'weigh-in':'weigh-ins'}</span>
 </div>
</div>
<style>
 .weight-chart{width:100%;min-width:0}
 .plot-units{font:400 12px/1.5 Inter;color:var(--text2);margin-bottom:8px}
 .chart-surface{display:block;width:100%;padding:0;border:0;background:none;touch-action:pan-y}
 svg{display:block;width:100%;height:auto;overflow:visible;font-family:Inter,sans-serif}
 .history-axis{display:flex;justify-content:space-between;gap:8px;padding-left:38px;font-size:11px;color:var(--text2)}
 .history-reading{display:flex;justify-content:space-between;align-items:center;gap:12px;flex-wrap:wrap;padding-top:12px;border-top:1px solid var(--line);margin:12px 0 4px}
 .reading-values{display:flex;align-items:baseline;gap:16px;flex-wrap:wrap}
 .reading-values>div{display:flex;flex-wrap:wrap;align-items:baseline;min-width:0}
 .history-reading b{font-size:23px;font-weight:500;letter-spacing:-.03em}
 .history-reading .overlay-reading b{color:var(--text2)}
 .reading-values span{color:var(--text2);font-size:12px;margin-left:5px}
 .history-reading>span{font-size:12px;text-align:right;flex:1}
 .history-reading small{display:block;color:var(--text2);font-size:12px;margin-top:4px}
 .history-controls{display:flex;align-items:center;gap:12px;margin-top:6px}
 .history-controls button{display:grid;place-items:center;flex:none;background:var(--card);border:1px solid var(--line);border-radius:12px;width:44px;height:44px}
 input{width:100%;min-width:0;height:44px;margin:0;accent-color:var(--ink);cursor:ew-resize}
 .history-legend{display:flex;gap:4px 10px;flex-wrap:wrap;font-size:12px;color:var(--text2);margin-top:12px;align-items:center}
 .history-legend span{display:inline-flex;align-items:center;gap:5px}
 .history-legend i.line{width:14px;height:0;border-top:2px solid}
 .history-legend i.dot{width:8px;height:8px;border-radius:50%}
 .record-count{margin-left:auto}
 .sr-only{position:absolute;width:1px;height:1px;overflow:hidden;clip-path:inset(50%);white-space:nowrap}
</style>
