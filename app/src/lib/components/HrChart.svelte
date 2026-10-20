<script lang="ts">
 import type {HrSample} from '../generated/HrSample';
 import {colors} from '../charts';
 import {declutter} from '../declutter';
 // Heart rate over the workout: a zone-coloured trace on a bpm axis, each
 // zone's floor labelled, the average marked, and round minute ticks. Drawn
 // at its real width so text keeps its size.
 let {samples,floors,durationS=0,avg=null,axis=true,name=''}:{samples:HrSample[],floors:number[],durationS?:number,avg?:number|null,axis?:boolean,name?:string}=$props();
 let w=$state(320);
 const H=160,L=34,R=8,T=20,B=20;
 let maxT=$derived(Math.max(1,durationS,...samples.map(s=>s.tS)));
 let lo=$derived(Math.floor((Math.min(...samples.map(s=>s.bpm))-4)/10)*10),hi=$derived(Math.ceil((Math.max(...samples.map(s=>s.bpm))+4)/10)*10);
 const y=(v:number)=>T+(hi-v)/(hi-lo||1)*(H-T-B);
 const x=(t:number)=>L+t/maxT*(w-L-R);
 let d=$derived(samples.map((s,i)=>`${i?'L':'M'}${x(s.tS).toFixed(1)},${y(s.bpm).toFixed(1)}`).join(''));
 // Incomplete boundaries cannot identify all five zones; keep the trace visible and neutral.
 let zoned=$derived(floors.length===4 && floors.every((f,i)=>Number.isFinite(f) && (!i || f>floors[i-1])));
 let stops=$derived(zoned?[...floors].reverse():[]);
 let ticks=$derived.by(()=>{const step=[10,20,30,40,50].find(s=>(hi-lo)/s<=4)??50,out:number[]=[];for(let t=Math.ceil(lo/step)*step;t<=hi;t+=step)out.push(t);return out});
 let minutes=$derived.by(()=>{const m=maxT/60,step=[1,2,5,10,15,20,30,60].find(s=>m/s<=6)??120,out:number[]=[];for(let t=0;t<=m;t+=step)out.push(t);return out});
 let shownFloors=$derived(zoned?floors.map((f,i)=>({f,zone:i+2})).filter(z=>z.f>lo && z.f<hi):[]);
</script>
<div class="hr" bind:clientWidth={w} data-chart="hr" {@attach declutter}>
<svg width={w} height={H} role="img" aria-label={`Heart rate over ${Math.round(maxT/60)} minutes, ${lo}–${hi} bpm${avg!=null?`, average ${Math.round(avg)}`:''}`}>
 <defs>
  <linearGradient id="hr-zones" x1="0" y1="0" x2="0" y2={H} gradientUnits="userSpaceOnUse">
   {#each stops as floor,i}{@const offset=Math.max(0,Math.min(1,y(floor)/H))}<stop {offset} stop-color={colors.z[4-i]}/><stop {offset} stop-color={colors.z[3-i]}/>{/each}
  </linearGradient>
 </defs>
 {#each ticks as t}<line x1={L} x2={w-R} y1={y(t)} y2={y(t)} stroke="var(--chart-grid)" stroke-width="var(--stroke-grid)"/><text class="t" data-thin="2" x={L-5} y={y(t)+4} text-anchor="end">{t}</text>{/each}
 <text class="u" x="2" y="12">{name?`${name} · `:''}bpm</text>
 {#each shownFloors as z}<line x1={L} x2={w-R} y1={y(z.f)} y2={y(z.f)} stroke={colors.z[z.zone-1]} stroke-opacity=".55" stroke-dasharray="3 4"/><text class="z" data-thin="1" x={w-R-2} y={y(z.f)-3} text-anchor="end" fill={colors.z[z.zone-1]}>Z{z.zone} {z.f}</text>{/each}
 {#if avg!=null && avg>lo && avg<hi}<line x1={L} x2={w-R} y1={y(avg)} y2={y(avg)} stroke={colors.ink} stroke-opacity=".7" stroke-dasharray="6 3"/><text class="z" data-thin="3" x={L+4} y={y(avg)-3} fill={colors.ink}>avg {Math.round(avg)}</text>{/if}
 {#if d}<path class="hr-trace" {d} fill="none" stroke={zoned?'url(#hr-zones)':colors.ink} stroke-width="var(--stroke-chart)" stroke-linejoin="round"/>{/if}
 {#if axis}{#each minutes as m,i}<text class="t" data-thin={i===0?undefined:'1'} x={x(m*60)} y={H-4} text-anchor={i===0?'start':'middle'}>{m}{i===minutes.length-1?' min':''}</text>{/each}{/if}
</svg>
</div>
<style>.hr{width:100%}svg{display:block}.t{font:500 11px Inter;fill:var(--text3)}.u{font:500 12px Inter;fill:var(--text3)}.z{font:600 12px Inter;paint-order:stroke;stroke:var(--bg);stroke-width:3px}</style>
