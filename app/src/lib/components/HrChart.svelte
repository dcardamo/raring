<script lang="ts">
 import type {HrSample} from '../generated/HrSample';
 import {colors} from '../charts';
 import {elapsed} from '../format';
 let {samples,floors,durationS=0}:{samples:HrSample[],floors:number[],durationS?:number}=$props();
 let maxT=$derived(Math.max(1,durationS,...samples.map(s=>s.tS)));
 let lo=$derived(Math.min(...samples.map(s=>s.bpm))-5),hi=$derived(Math.max(...samples.map(s=>s.bpm))+5);
 const y=(v:number)=>112-(v-lo)/(hi-lo||1)*104;
 let d=$derived(samples.map((s,i)=>`${i?'L':'M'}${(s.tS/maxT*326).toFixed(1)},${y(s.bpm).toFixed(1)}`).join(''));
 // Incomplete boundaries cannot identify all five zones; keep the trace visible and neutral.
 let zoned=$derived(floors.length===4 && floors.every((f,i)=>Number.isFinite(f) && (!i || f>floors[i-1])));
 let stops=$derived(zoned?[...floors].reverse():[]);
</script>
<svg class="chart" viewBox="0 0 326 136" aria-label="Heart rate during workout">
 <defs>
  <linearGradient id="hr-zones" x1="0" y1="0" x2="0" y2="120" gradientUnits="userSpaceOnUse">
   {#each stops as floor,i}{@const offset=Math.max(0,Math.min(1,y(floor)/120))}<stop {offset} stop-color={colors.z[4-i]}/><stop {offset} stop-color={colors.z[3-i]}/>{/each}
  </linearGradient>
  <linearGradient id="hr-area" x1="0" y1="0" x2="0" y2="1"><stop stop-color={colors.amber} stop-opacity=".28"/><stop offset="1" stop-color={colors.amber} stop-opacity="0"/></linearGradient>
 </defs>
 {#each floors.filter(f=>f>=lo && f<=hi) as f}<line x1="0" x2="326" y1={y(f)} y2={y(f)} stroke="rgba(255,255,255,.06)" stroke-dasharray="3 4"/>{/each}
 {#if d}<path d={`${d}L326,116L0,116Z`} fill="url(#hr-area)"/><path class="hr-trace" {d} fill="none" stroke={zoned?'url(#hr-zones)':colors.coral} stroke-width="2.2" stroke-linejoin="round"/>{/if}
 {#each [0,1/3,2/3,1] as part,i}<text x={part*326} y="132" fill="#69728A" font-size="11" text-anchor={i===0?'start':i===3?'end':'middle'}>{elapsed(maxT*part)}</text>{/each}
</svg>
<style>.chart{width:100%;display:block}</style>
