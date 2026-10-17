<script lang="ts">
 import type {StressCard} from '../generated/StressCard';
 import {colors,stressColors,stressLevel,stressNames} from '../charts';
 import {num} from '../format';
 let {stress,compact=false}:{stress:StressCard,compact?:boolean}=$props();
 const W=320;
 let left=$derived(compact?0:26),width=$derived(W-left),top=$derived(compact?2:20),h=$derived(compact?44:120);
 const x=(minute:number)=>left+minute/1440*width;
 const y=(value:number)=>top+h-value/100*h;
 const time=(minute:number)=>`${String(Math.floor(minute/60)%24).padStart(2,'0')}:${String(Math.floor(minute%60)).padStart(2,'0')}`;
 // The card shows 15-minute means so bars stay legible; the detail keeps every reading.
 let bucket=$derived(compact?5:1);
 let paths=$derived.by(()=>{
  const out:Record<string,string>={rest:'',low:'',medium:'',high:''};
  for(let i=0;i<stress.timeline.length;i+=bucket){
   const values=stress.timeline.slice(i,i+bucket).filter((v):v is number=>v!=null);
   if(!values.length)continue;
   const value=values.reduce((a,b)=>a+b,0)/values.length,x0=x(i*3),bar=width*bucket/480;
   out[stressLevel(value)]+=`M${x0.toFixed(2)},${top+h}V${y(Math.max(value,2)).toFixed(2)}h${(compact?bar-.8:bar+.4).toFixed(2)}V${top+h}Z`;
  }
  return out;
 });
 let latest=$derived(stress.timeline.findLastIndex(v=>v!=null));
 let selected=$state<number|null>(null);
 let slot=$derived(selected??latest);
 let reading=$derived(slot<0?null:stress.timeline[slot]??null);
 let moving=$derived(slot>=0 && stress.active.some(([a,b])=>slot*3>=a && slot*3<b));
 let label=$derived(slot<0?'No readings yet':`${time(slot*3)}: ${reading!=null?`${num(reading)}, ${stressNames[stressLevel(reading)]}`:moving?'active, not rated':'no reading'}`);
 function key(e:KeyboardEvent){
  const last=stress.timeline.length-1,step=e.shiftKey?20:1;
  const next=e.key==='ArrowLeft'?slot-step:e.key==='ArrowRight'?slot+step:e.key==='Home'?0:e.key==='End'?last:null;
  if(next==null || last<0)return;
  e.preventDefault();selected=Math.max(0,Math.min(last,next));
 }
 function select(e:MouseEvent){
  const rect=(e.currentTarget as SVGElement).getBoundingClientRect();
  const minute=((e.clientX-rect.left)/rect.width*W-left)/width*1440;
  if(minute<0)return;
  selected=Math.max(0,Math.min(stress.timeline.length-1,Math.floor(minute/3)));
 }
</script>
{#if !compact}
 <div class="readout" aria-live="polite">
  {#if slot>=0}<span>{time(slot*3)}</span>{/if}
  {#if reading!=null}<b class="n" style:color={stressColors[stressLevel(reading)]}>{num(reading)}</b><em>{stressNames[stressLevel(reading)]}</em>
  {:else if moving}<em>Active · not rated</em>{:else if slot>=0}<em>No reading</em>{/if}
 </div>
{/if}
{#if compact}
<svg viewBox="0 0 {W} {top+h+2}" class="compact" role="img" aria-label="Stress through the day">{@render plot()}</svg>
{:else}
<svg viewBox="0 0 {W} {top+h+22}" role="slider" tabindex="0" aria-label="Stress through the day. Gaps are times without a reading." aria-valuemin={0} aria-valuemax={Math.max(0,stress.timeline.length-1)} aria-valuenow={Math.max(0,slot)} aria-valuetext={label} onclick={select} onkeydown={key}>{@render plot()}</svg>
{/if}
{#snippet plot()}
 {#if stress.sleep}<rect x={x(stress.sleep[0])} y={top-(compact?2:8)} width={x(stress.sleep[1])-x(stress.sleep[0])} height={h+(compact?4:8)} rx="6" fill={colors.ink} opacity=".09"/>{#if !compact}<text x={x(stress.sleep[0])+6} y={top+4} fill={colors.ink} font-size="10" opacity="1">Asleep</text>{/if}{/if}
 {#if !compact}{#each [25,50,75] as level}<line x1={left} x2={W} y1={y(level)} y2={y(level)} stroke="currentColor" opacity=".07"/><text x="0" y={y(level)+3} fill="#8E98AC" font-size="9">{level}</text>{/each}{/if}
 {#each stress.active as [a,b]}<rect x={x(a)} y={top+h-(compact?4:6)} width={Math.max(1.5,x(b)-x(a))} height={compact?4:6} rx="2" fill={colors.secondary} opacity=".85"/>{/each}
 {#if compact}<line x1="0" x2={W} y1={top+h+.5} y2={top+h+.5} stroke="currentColor" opacity=".12"/>{/if}
 {#each Object.entries(paths) as [level,d]}{#if d}<path {d} fill={stressColors[level]} opacity=".9"/>{/if}{/each}
 {#if !compact && slot>=0}<line x1={x(slot*3+1.5)} x2={x(slot*3+1.5)} y1={top-8} y2={top+h} stroke="currentColor" opacity=".4" stroke-dasharray="2 3"/>{/if}
 {#if !compact}{#each [0,360,720,1080,1440] as m,i}<text x={x(m)} y={top+h+16} fill="#8E98AC" font-size="10" text-anchor={i===0?'start':i===4?'end':'middle'}>{i===4?'24:00':time(m)}</text>{/each}{/if}
{/snippet}
{#if !compact}
 <div class="legend">{#each ['rest','low','medium','high'] as level}<span><i style:background={stressColors[level]}></i>{stressNames[level]}</span>{/each}<span><i style:background={colors.secondary}></i>Active</span>{#if stress.sleep}<span><i style:background={colors.ink} style:opacity=".5"></i>Asleep</span>{/if}</div>
{/if}
<style>
 svg{display:block;width:100%;height:auto;overflow:visible;font-family:Inter,sans-serif;color:var(--text)}
 svg:not(.compact){cursor:crosshair;touch-action:pan-y;border-radius:8px}
 .readout{display:flex;align-items:baseline;gap:8px;min-height:30px;margin-bottom:6px;font-size:13px;color:var(--text2)}
 .readout b{font-size:26px;font-weight:600}
 .readout em{font-style:normal}
 .legend{display:flex;flex-wrap:wrap;gap:10px;margin-top:12px;font-size:10px;color:var(--text2)}
 .legend span{display:inline-flex;align-items:center;gap:5px}
 .legend i{width:7px;height:7px;border-radius:2px}
</style>
