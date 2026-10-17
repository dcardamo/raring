<script lang="ts">
 import type {NightWindow} from '../generated/NightWindow';
 import {colors} from '../charts';
 import {historyPoints} from '../history';
 let {nights,date,bedWindow=null,wakeWindow=null,days=14}:{nights:NightWindow[],date:string,bedWindow?:[number,number]|null,wakeWindow?:[number,number]|null,days?:number}=$props();
 let points=$derived(historyPoints([],date,days));
 let slots=$derived(points.map(day=>nights.find(n=>n.date===day.date)));
 const weekday=(d:string)=>new Date(`${d}T12:00:00`).toLocaleDateString('en-GB',{weekday:'short'});
 const hour=(h:number)=>h<12?h+24:h;
 let low=$derived(Math.min(21.5,...nights.map(n=>hour(n.bedHour)-.5))),high=$derived(Math.max(31.5,...nights.map(n=>hour(n.wakeHour)+.5)));
 const y=(h:number)=>8+(hour(h)-low)/(high-low)*102;
</script>
<svg viewBox="0 0 326 128" class="plot" aria-label={days===14?"Bed and wake times over two weeks":"Bed and wake times this week"}>
 {#if bedWindow}<rect x="40" y={y(bedWindow[0])} width="286" height={Math.max(0,y(bedWindow[1])-y(bedWindow[0]))} rx="3" fill={colors.ink} opacity=".12"/>{/if}
 {#if wakeWindow}<rect x="40" y={y(wakeWindow[0])} width="286" height={Math.max(0,y(wakeWindow[1])-y(wakeWindow[0]))} rx="3" fill={colors.ink} opacity=".12"/>{/if}
 {#each [['22:00',22],['02:00',2],['06:00',6]] as tick}<text x="0" y={y(Number(tick[1]))+4} fill="#8E98AC" font-size="11">{tick[0]}</text><line x1="40" x2="326" y1={y(Number(tick[1]))} y2={y(Number(tick[1]))} stroke="rgba(255,255,255,.05)"/>{/each}
 {#each slots as n,i}{#if n}<rect class="night" data-date={n.date} x={43+i*286/days+(days===14?0:(286/days-14)/2)} y={y(n.bedHour)} width="14" height={Math.max(1,y(n.wakeHour)-y(n.bedHour))} rx="7" fill={i===days-1?colors.ink:'rgba(228,232,240,.38)'}/>{/if}{/each}
 <text x="40" y="126" fill="#8E98AC" font-size="11">{days===14?'2 weeks ago':weekday(points[0].date)}</text><text x="326" y="126" text-anchor="end" fill="#8E98AC" font-size="11">{days===14?'Last night':weekday(points[days-1].date)}</text>
</svg>
<style>.plot{display:block;width:100%}</style>
