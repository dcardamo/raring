<script lang="ts">
 import type {StageSegment} from '../generated/StageSegment';
 import {colors} from '../charts';
 import {clock,elapsed} from '../format';
 let {segments,startLocal=null}:{segments:StageSegment[],startLocal?:string|null}=$props();
 const names=['Awake','REM','Light','Deep'];
 const palette:Record<string,string>={awake:colors.awake,rem:colors.rem,light:colors.light,deep:colors.deep};
 let total=$derived(Math.max(1,...segments.map(s=>s.startMin+s.minutes)));
 const lane=(stage:string)=>names.indexOf(stage==='rem'?'REM':stage[0].toUpperCase()+stage.slice(1));
 function label(minutes:number){
  const time=clock(startLocal);
  if(time==='—')return elapsed(minutes*60);
  const [h,m]=time.split(':').map(Number),value=(h*60+m+Math.round(minutes))%1440;
  return `${String(Math.floor(value/60)).padStart(2,'0')}:${String(value%60).padStart(2,'0')}`;
 }
</script>
<svg viewBox="0 0 326 142" class="plot" aria-label="Sleep stages over the night">
 {#each names as name,i}<text x="0" y={i*30+19} fill="#8E98AC" font-size="11" font-weight="600">{name}</text><line x1="46" x2="326" y1={i*30+15} y2={i*30+15} stroke="rgba(255,255,255,.05)"/>{/each}
 {#each segments as s,i}{@const x=46+s.startMin/total*280}
  {#if i>0}<line x1={x} x2={x} y1={lane(segments[i-1].stage)*30+15} y2={lane(s.stage)*30+15} stroke="rgba(255,255,255,.22)"/>{/if}
  <rect x={x} y={lane(s.stage)*30+4} width={Math.max(1,s.minutes/total*280)} height="22" rx="5" fill={palette[s.stage]}/>
 {/each}
 {#each [0,1/3,2/3,1] as part,i}<text class="time-label" x={46+part*280} y="139" fill="#8E98AC" font-size="11" text-anchor={i===0?'start':i===3?'end':'middle'}>{label(total*part)}</text>{/each}
</svg>
<style>.plot{display:block;width:100%}</style>
