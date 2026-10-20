<script lang="ts">
 import type {StressCard} from '../generated/StressCard';
 import type {HeartRateDay} from '../generated/HeartRateDay';
 import {colors,stressColors,stressLevel,stressNames} from '../charts';
 import {num} from '../format';
 let {stress,heartRate,compact=false}:{stress?:StressCard,heartRate?:HeartRateDay,compact?:boolean}=$props();
 const W=320;
 let timeline=$derived(heartRate?.timeline??stress?.timeline??[]);
 let ranges=$derived(heartRate?.activities??stress?.active??[]);
 let sleep=$derived(heartRate?.sleep??(stress?.sleep?[stress.sleep]:[]));
 let step=$derived(heartRate?2:3);
 let dayMinutes=$derived(heartRate?.durationMin??1440);
 let left=$derived(compact?0:26),width=$derived(W-left),top=$derived(compact?2:20),h=$derived(compact?44:120);
 const x=(minute:number)=>left+minute/dayMinutes*width;
 let low=$derived(heartRate?40:0),high=$derived(heartRate?180:100);
 const y=(value:number)=>top+h-Math.max(0,Math.min(1,(value-low)/(high-low)))*h;
 const time=(minute:number)=>`${String(Math.floor(minute/60)%24).padStart(2,'0')}:${String(Math.floor(minute%60)).padStart(2,'0')}`;
 // Heart rate runs on elapsed time from the day's start, so it never wears clock-style labels.
 const elapsed=(minute:number)=>{const m=Math.floor(minute);return `${Math.floor(m/60)} h ${String(m%60).padStart(2,'0')} m`};
 // A compact chart averages only present samples; missing slots remain gaps.
 let bucket=$derived(compact?(heartRate?8:5):1);
 let paths=$derived.by(()=>{
  const out:Record<string,string>=heartRate?{heart:''}:{rest:'',low:'',medium:'',high:''};
  for(let i=0;i<timeline.length;i+=bucket){
   const values=timeline.slice(i,i+bucket).filter((v):v is number=>v!=null);
   if(!values.length)continue;
   const value=values.reduce((a,b)=>a+b,0)/values.length,x0=x(i*step),bar=width*bucket*step/dayMinutes;
   out[heartRate?'heart':stressLevel(value)]+=`M${x0.toFixed(2)},${top+h}V${y(heartRate?value:Math.max(value,2)).toFixed(2)}h${Math.max(0.4,bar-(compact?.8:0)).toFixed(2)}V${top+h}Z`;
  }
  return out;
 });
 let latest=$derived(timeline.findLastIndex(v=>v!=null));
 let selected=$state<number|null>(null);
 let slot=$derived(Math.min(selected??latest,timeline.length-1));
 let reading=$derived(slot<0?null:timeline[slot]??null);
 let moving=$derived(slot>=0 && ranges.some(([a,b])=>slot*step>=a && slot*step<b));
 let label=$derived(slot<0?'No readings yet':`${heartRate?elapsed(slot*step)+' into the day':time(slot*step)}: ${reading!=null?heartRate?`${num(reading)} bpm`:`${num(reading)}, ${stressNames[stressLevel(reading)]}`:moving?heartRate?'activity, no reading':'active, not rated':'no reading'}`);
 function key(e:KeyboardEvent){
  const last=timeline.length-1,jump=e.shiftKey?20:1;
  const next=e.key==='ArrowLeft'?slot-jump:e.key==='ArrowRight'?slot+jump:e.key==='Home'?0:e.key==='End'?last:null;
  if(next==null || last<0)return;
  e.preventDefault();selected=Math.max(0,Math.min(last,next));
 }
 function select(e:MouseEvent){
  const rect=(e.currentTarget as SVGElement).getBoundingClientRect();
  const minute=((e.clientX-rect.left)/rect.width*W-left)/width*dayMinutes;
  if(minute<0)return;
  selected=Math.max(0,Math.min(timeline.length-1,Math.floor(minute/step)));
 }
</script>
{#if !compact}
 <div class="readout" aria-live="polite">
  {#if slot>=0}<span>{heartRate?`${elapsed(slot*step)} into the day`:time(slot*step)}</span>{/if}
  {#if reading!=null}<b class="n" style:color={heartRate?colors.ink:stressColors[stressLevel(reading)]}>{num(reading)}</b><em>{heartRate?'bpm':stressNames[stressLevel(reading)]}</em>
  {:else if moving}<em>{heartRate?'Activity · no reading':'Active · not rated'}</em>{:else if slot>=0}<em>No reading</em>{/if}
 </div>
{/if}
{#if compact}
<svg viewBox="0 0 {W} {top+h+2}" class="compact" role="img" aria-label={heartRate?'Heart rate through the day':'Stress through the day'}>{@render plot()}</svg>
{:else}
<svg viewBox="0 0 {W} {top+h+22}" role="slider" tabindex="0" aria-label={heartRate?'Heart rate through the day. Gaps are missing or off-wrist readings.':'Stress through the day. Gaps are times without a reading.'} aria-valuemin={0} aria-valuemax={Math.max(0,timeline.length-1)} aria-valuenow={Math.max(0,slot)} aria-valuetext={label} onclick={select} onkeydown={key}>{@render plot()}</svg>
{/if}
{#snippet plot()}
 {#each sleep as [a,b]}<rect x={x(a)} y={top-(compact?2:8)} width={Math.max(0,x(b)-x(a))} height={h+(compact?4:8)} rx="6" fill={colors.ink} opacity=".09"/>{#if !compact}<text x={x(a)>W-60?x(a)-46:x(a)+6} y={top+4} fill={colors.ink} font-size="10">Asleep</text>{/if}{/each}
 {#if !compact}{#each heartRate?[60,100,140]:[25,50,75] as level}<line x1={left} x2={W} y1={y(level)} y2={y(level)} stroke="currentColor" opacity=".07"/><text x="0" y={y(level)+3} fill="#8E98AC" font-size="9">{level}</text>{/each}{/if}
 {#each ranges as [a,b]}<rect x={x(a)} y={top+h-(compact?4:6)} width={Math.max(1.5,x(b)-x(a))} height={compact?4:6} rx="2" fill={colors.secondary} opacity=".85"/>{/each}
 {#if compact}<line x1="0" x2={W} y1={top+h+.5} y2={top+h+.5} stroke="currentColor" opacity=".12"/>{/if}
 {#each Object.entries(paths) as [level,d]}{#if d}<path {d} fill={heartRate?colors.ink:stressColors[level]} opacity=".9"/>{/if}{/each}
 {#if !compact && slot>=0}<line x1={x(slot*step+step/2)} x2={x(slot*step+step/2)} y1={top-8} y2={top+h} stroke="currentColor" opacity=".4" stroke-dasharray="2 3"/>{/if}
 {#if !compact}{#each [0,1,2,3,4] as i}{@const m=heartRate?dayMinutes*i/4:i*360}<text x={x(m)} y={top+h+16} fill="#8E98AC" font-size="10" text-anchor={i===0?'start':i===4?'end':'middle'}>{heartRate?`${Math.round(m/60)} h`:i===4?'24:00':time(m)}</text>{/each}{/if}
{/snippet}
{#if !compact}
 {#if heartRate && dayMinutes!==1440}<p class="axis-note">A {num(dayMinutes/60,1)}-hour day: hours count from its start</p>{/if}
 <div class="legend">{#if heartRate}<span><i style:background={colors.ink}></i>Heart rate</span>{:else}{#each ['rest','low','medium','high'] as level}<span><i style:background={stressColors[level]}></i>{stressNames[level]}</span>{/each}{/if}<span><i style:background={colors.secondary}></i>{heartRate?'Activity':'Active'}</span>{#if sleep.length}<span><i style:background={colors.ink} style:opacity=".5"></i>Asleep</span>{/if}</div>
{/if}
<style>
 svg{display:block;width:100%;height:auto;overflow:visible;font-family:Inter,sans-serif;color:var(--text)}
 svg:not(.compact){cursor:crosshair;touch-action:pan-y;border-radius:8px}
 .readout{display:flex;align-items:baseline;gap:8px;min-height:30px;margin-bottom:6px;font-size:13px;color:var(--text2)}
 .readout b{font-size:26px;font-weight:600}
 .readout em{font-style:normal}
 .axis-note{margin:4px 0 0;font-size:10px;color:var(--text2)}
 .legend{display:flex;flex-wrap:wrap;gap:10px;margin-top:12px;font-size:10px;color:var(--text2)}
 .legend span{display:inline-flex;align-items:center;gap:5px}
 .legend i{width:7px;height:7px;border-radius:2px}
</style>
