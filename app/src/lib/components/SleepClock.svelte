<script lang="ts">
 import type {SleepPlan} from '../generated/SleepPlan';
 import {sleepClock,arcPath} from '../clock';
 import {duration} from '../format';
 // Tonight's plan on a 24-hour ring, midnight at the top: the wind-down, the
 // minutes to fall asleep (lighter), the planned sleep to the expected wake,
 // Garmin's window just outside the ring, and now. The times in the centre
 // are the only text inside; the facts beside it carry the rest. Compact on
 // Today's evening card, with the duration only.
 let {plan,now,compact=false}:{plan:SleepPlan,now:Date,compact?:boolean}=$props();
 let c=$derived(sleepClock(plan,now));
 const hm=(iso:string)=>iso.slice(11,16);
 let up=$derived(plan.wakeAt?hm(plan.wakeAt):null);
 let inBed=$derived(plan.inBed||plan.bedtime);
 let clockNow=$derived(`${String(now.getHours()).padStart(2,'0')}:${String(now.getMinutes()).padStart(2,'0')}`);
 let label=$derived(`Tonight on a 24-hour clock: in bed ${inBed}, asleep by ${plan.bedtime}${up?`, up ${up}`:''}; wind down from ${plan.windDown}${plan.bedtimeStart?`; Garmin's window ${plan.bedtimeStart}–${plan.bedtime}`:''}; now ${clockNow}`);
 const R=34;
 let nowRad=$derived(c?(c.now-90)*Math.PI/180:0);
</script>
{#if c}
 <div class="clock" class:compact role="img" aria-label={label} data-chart="sleep clock">
  <svg viewBox="0 0 100 100" aria-hidden="true">
   <circle cx="50" cy="50" r={R} fill="none" stroke="rgba(255,255,255,.08)" stroke-width="8"/>
   {#each [0,90,180,270] as a}<line x1="50" y1={50-R-5} x2="50" y2={50-R-2} stroke="rgba(255,255,255,.35)" stroke-width="1" transform={`rotate(${a} 50 50)`}/>{/each}
   {#if c.windDown}<path d={arcPath(50,R,c.windDown)} fill="none" stroke="#FFB547" stroke-opacity=".85" stroke-width="5"/>{/if}
   {#if c.fallAsleep}<path d={arcPath(50,R,c.fallAsleep)} fill="none" stroke="#A9C8FF" stroke-opacity=".55" stroke-width="8"/>{/if}
   <path d={arcPath(50,R,c.sleep)} fill="none" stroke="#5C9BFF" stroke-width="8" stroke-linecap="round"/>
   {#if c.window}<path d={arcPath(50,R+6.5,c.window)} fill="none" stroke="#EEF1F7" stroke-width="2.5" stroke-linecap="round"/>{/if}
   <circle data-mark="now" cx={50+R*Math.cos(nowRad)} cy={50+R*Math.sin(nowRad)} r="3.6" fill="#fff" stroke="var(--bg)" stroke-width="1.5"/>
  </svg>
  <span class="tick t0">00</span><span class="tick t6">06</span><span class="tick t12">12</span><span class="tick t18">18</span>
  <div class="centre">
   <p class="dur"><b class="n">{duration(plan.needMin)}</b>{#if !compact}<small>{' asleep'}</small>{/if}</p>
   {#if !compact}<p class="when">In bed <b class="n">{inBed}</b></p>{#if up}<p class="when">Up <b class="n">{up}</b></p>{/if}{/if}
  </div>
 </div>
{/if}
<style>
 .clock{position:relative;width:min(100%,20em);aspect-ratio:1;margin:0 auto;container-type:inline-size}
 .clock.compact{width:min(100%,9.5em);margin:0}.compact svg{inset:7%;width:86%;height:86%}
 svg{position:absolute;inset:0;width:100%;height:100%;overflow:visible}
 .tick{position:absolute;font:600 12px Inter;color:var(--text2);line-height:1}
 .t0{top:0;left:50%;transform:translateX(-50%)}.t12{bottom:0;left:50%;transform:translateX(-50%)}
 .t6{right:0;top:50%;transform:translateY(-50%)}.t18{left:0;top:50%;transform:translateY(-50%)}
 .centre{position:absolute;inset:0;display:flex;flex-direction:column;align-items:center;justify-content:center;gap:.1em;text-align:center}
 .centre p{margin:0;white-space:nowrap}
 .dur b{font-size:min(2em,12cqi);font-weight:600;line-height:1}.dur small{font:500 min(1em,5.5cqi) Inter;color:var(--text2)}
 .compact .dur b{font-size:min(1.5em,15cqi)}
 .when{font:500 min(1em,6cqi) Inter;color:var(--text2)}.when b{color:var(--text);font-weight:600}
</style>
