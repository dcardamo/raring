<script lang="ts">
 import type {StageSegment} from '../generated/StageSegment';
 import {colors} from '../charts';
 import {clock,elapsed} from '../format';
 import {declutter} from '../declutter';
 let {segments,startLocal=null}:{segments:StageSegment[],startLocal?:string|null}=$props();
 const names=['Awake','REM','Light','Deep'];
 const palette:Record<string,string>={awake:colors.awake,rem:colors.rem,light:colors.light,deep:colors.deep};
 let total=$derived(Math.max(1,...segments.map(s=>s.startMin+s.minutes)));
 const lane=(stage:string)=>names.indexOf(stage==='rem'?'REM':stage[0].toUpperCase()+stage.slice(1));
 const pct=(m:number)=>m/total*100;
 function label(minutes:number){
  const time=clock(startLocal);
  if(time==='—')return elapsed(minutes*60);
  const [h,m]=time.split(':').map(Number),value=(h*60+m+Math.round(minutes))%1440;
  return `${String(Math.floor(value/60)).padStart(2,'0')}:${String(value%60).padStart(2,'0')}`;
 }
</script>
<!-- HTML lanes, so the stage names keep their own column at any text size; only the transitions are drawn in SVG. -->
<div class="hyp" role="img" aria-label="Sleep stages over the night" data-chart="stages" {@attach declutter}>
 {#each names as name,i}<span class="name" style:grid-row={i+1}>{name}</span>{/each}
 <div class="plot" style:grid-row={`1 / ${names.length+1}`}>
  <svg viewBox="0 0 100 {names.length}" preserveAspectRatio="none" aria-hidden="true">{#each segments as s,i}{#if i>0}<line x1={pct(s.startMin)} x2={pct(s.startMin)} y1={lane(segments[i-1].stage)+.5} y2={lane(s.stage)+.5} stroke="rgba(255,255,255,.22)" vector-effect="non-scaling-stroke"/>{/if}{/each}</svg>
  {#each names as _,i}<b class="lane" style:top={i/names.length*100+'%'} style:height={100/names.length+'%'}></b>{/each}
  {#each segments as s}<i style:left={pct(s.startMin)+'%'} style:width={`max(1px, ${s.minutes/total*100}%)`} style:top={`calc(${lane(s.stage)/names.length*100}% + 4px)`} style:height={`calc(${100/names.length}% - 8px)`} style:background={palette[s.stage]}></i>{/each}
 </div>
 <span class="times">{#each [0,1/3,2/3,1] as part,i}<span class="time-label" data-thin={i===0||i===3?1:''} class:first={i===0} class:last={i===3} style:left={part*100+'%'}>{label(total*part)}</span>{/each}</span>
</div>
<style>
 .hyp{display:grid;grid-template-columns:max-content minmax(0,1fr);grid-auto-rows:30px;column-gap:8px;font:600 12.5px Inter;color:#A3ACBE}
 .name{align-self:center}
 .plot{grid-column:2;position:relative}
 .plot svg{position:absolute;inset:0;width:100%;height:100%}
 .lane{position:absolute;left:0;right:0;border-top:1px solid rgba(255,255,255,.05);transform:translateY(15px);height:0!important}
 .plot i{position:absolute;border-radius:5px}
 .times{grid-column:2;position:relative;height:auto;min-height:1.6em;font:500 11.5px Inter}
 .time-label{position:absolute;top:4px;transform:translateX(-50%);white-space:nowrap}
 .time-label.first{transform:none}.time-label.last{transform:translateX(-100%)}
</style>
