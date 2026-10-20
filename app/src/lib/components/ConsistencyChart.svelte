<script lang="ts">
 import type {NightWindow} from '../generated/NightWindow';
 import {colors} from '../charts';
 import {historyPoints} from '../history';
 import {declutter} from '../declutter';
 // Drawn at the measured width so the clock labels keep their size; the usual
 // windows' times are in the key, not on the plot.
 let {nights,date,bedWindow=null,wakeWindow=null,days=14}:{nights:NightWindow[],date:string,bedWindow?:[number,number]|null,wakeWindow?:[number,number]|null,days?:number}=$props();
 let width=$state(326);
 let W=$derived(Math.max(240,Math.round(width)));
 let points=$derived(historyPoints([],date,days));
 let slots=$derived(points.map(day=>nights.find(n=>n.date===day.date)));
 const weekday=(d:string)=>new Date(`${d}T12:00:00`).toLocaleDateString('en-GB',{weekday:'short'});
 const dayMonth=(d:string)=>new Date(`${d}T12:00:00`).toLocaleDateString('en-GB',{day:'numeric',month:'short'});
 const hour=(h:number)=>h<12?h+24:h;
 const clock=(h:number)=>{const m=Math.round(((h%24)+24)%24*60)%1440;return `${String(Math.floor(m/60)).padStart(2,'0')}:${String(m%60).padStart(2,'0')}`};
 const L=42,H=140,T=8,B=24;
 let low=$derived(Math.min(21.5,...nights.map(n=>hour(n.bedHour)-.5))),high=$derived(Math.max(31.5,...nights.map(n=>hour(n.wakeHour)+.5)));
 const y=(h:number)=>T+(hour(h)-low)/(high-low)*(H-T-B);
 let col=$derived((W-L)/days);
 const x=(i:number)=>L+i*col+col/2;
 let bar=$derived(Math.min(14,col*.6));
 // Each week's Monday, and last night at the end.
 let marks=$derived(points.map((p,i)=>({i,date:p.date})).filter(p=>new Date(`${p.date}T12:00:00`).getDay()===1));
</script>
<div class="consistency" bind:clientWidth={width} data-chart="consistency" {@attach declutter}>
 <svg width={W} height={H} class="plot" role="img" aria-label={`${days===14?'Bed and wake times over two weeks':'Bed and wake times this week'}${bedWindow?`; usual bedtime ${clock(bedWindow[0])}–${clock(bedWindow[1])}`:''}${wakeWindow?`; usual wake ${clock(wakeWindow[0])}–${clock(wakeWindow[1])}`:''}`}>
  {#if bedWindow}<rect class="usual" x={L} y={y(bedWindow[0])} width={W-L} height={Math.max(2,y(bedWindow[1])-y(bedWindow[0]))} rx="3" fill="var(--goal-sleep)" opacity=".24"/>{/if}
  {#if wakeWindow}<rect class="usual" x={L} y={y(wakeWindow[0])} width={W-L} height={Math.max(2,y(wakeWindow[1])-y(wakeWindow[0]))} rx="3" fill="#FFD666" opacity=".22"/>{/if}
  {#each [22,0,2,4,6,8].filter(h=>hour(h)>=low && hour(h)<=high) as h}<text data-thin={h%4===2?2:''} x="0" y={y(h)+4} fill="#A3ACBE" font-size="11">{clock(h)}</text><line x1={L} x2={W} y1={y(h)} y2={y(h)} stroke="rgba(255,255,255,.06)"/>{/each}
  {#each marks as m}<line x1={x(m.i)-col/2} x2={x(m.i)-col/2} y1={T} y2={H-B} stroke="rgba(255,255,255,.1)" stroke-dasharray="2 3"/>{/each}
  {#each slots as n,i}{#if n}<rect class="night" data-date={n.date} x={x(i)-bar/2} y={y(n.bedHour)} width={bar} height={Math.max(1,y(n.wakeHour)-y(n.bedHour))} rx={bar/2} fill={i===days-1?colors.ink:'rgba(228,232,240,.45)'}/>{/if}{/each}
  {#each marks as m}<text data-thin="1" x={x(m.i)-col/2+2} y={H-6} fill="#A3ACBE" font-size="11">{days===14?dayMonth(m.date):weekday(m.date)}</text>{/each}
  <text x={W} y={H-6} text-anchor="end" fill="#A3ACBE" font-size="12">{days===14?'Last night':weekday(points[days-1].date)}</text>
 </svg>
 {#if bedWindow || wakeWindow}
  <p class="key">{#if bedWindow}<span><i class="bed"></i>Usual bedtime <b class="n">{clock(bedWindow[0])}–{clock(bedWindow[1])}</b></span>{/if}{#if wakeWindow}<span><i class="wake"></i>Usual wake <b class="n">{clock(wakeWindow[0])}–{clock(wakeWindow[1])}</b></span>{/if}</p>
 {/if}
</div>
<style>
 .consistency{width:100%;min-width:0}
 .plot{display:block;font-family:Inter,sans-serif}
 .key{display:flex;flex-wrap:wrap;gap:4px 14px;margin:8px 0 0;font-size:12px;color:var(--text2)}
 .key span{display:inline-flex;flex-wrap:wrap;align-items:center;gap:5px}
 .key b{white-space:nowrap}
 .key b{color:var(--text);font-weight:500}
 .key i{width:10px;height:10px;border-radius:3px}
 .key .bed{background:rgba(169,200,255,.5)}.key .wake{background:rgba(255,214,102,.45)}
</style>
