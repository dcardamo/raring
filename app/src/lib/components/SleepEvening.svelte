<script lang="ts">
 import type {SleepPlan} from '../generated/SleepPlan';
 import {sleepEvening} from '../clock';
 import {declutter} from '../declutter';
 // Tonight's evening on one left-to-right axis: the wind-down, the minutes to
 // fall asleep, Garmin's window bracketed under the bed edge, and now against
 // them. Distance is time to bed, never a countdown, and the night's length is
 // the line above rather than the scale. Every time is written once, in text
 // that wraps; the strip carries only the hours and now.
 let {plan,now,live=false}:{plan:SleepPlan,now:Date,live?:boolean}=$props();
 let e=$derived(sleepEvening(plan,now));
 const hm=(iso:string|null|undefined)=>iso?iso.slice(11,16):null;
 let bed=$derived(plan.inBed||plan.bedtime);
 let wake=$derived(hm(plan.wakeAt));
 // Wind-down asks for the window; the evening for the time to be in bed.
 let kicker=$derived(live?'Bedtime window':plan.bedtimeStart?'In bed by':'Asleep by');
 let big=$derived(live?plan.bedtimeStart?`${plan.bedtimeStart}–${plan.bedtime}`:plan.bedtime:bed);
 // Each time is written once: what the strip draws, and what the headline leaves out.
 let line=$derived([
  `Wind down ${plan.windDown}`,
  live&&plan.bedtimeStart?`in bed ${bed}`:plan.bedtimeStart?`Garmin window ${plan.bedtimeStart}–${plan.bedtime}`:`asleep by ${plan.bedtime}`,
  wake?`up ${wake}`:null,
 ].filter(Boolean).join(' · '));
 let clock=$derived(`${String(now.getHours()).padStart(2,'0')}:${String(now.getMinutes()).padStart(2,'0')}`);
 // A label at either edge reads inward, so larger text cannot leave the card.
 let anchor=$derived.by(()=>{const x=e?.now??0;return x>82?'right':x<8?'left':'mid'});
 let label=$derived(`Tonight’s evening: ${line.replaceAll(' · ',', ')}; now ${clock}`);
</script>
{#if e}
 <div class="eve">
  <p class="kick">{kicker}</p>
  <p class="big n">{big}</p>
  <p class="line">{line}</p>
  <div class="strip" data-chart="sleep evening" {@attach declutter} role="img" aria-label={label}>
   <div class="track">
    {#if e.windDown}<i class="wd" style:left={e.windDown.left+'%'} style:width={e.windDown.width+'%'}></i>{/if}
    {#if e.fallAsleep}<i class="fa" style:left={e.fallAsleep.left+'%'} style:width={e.fallAsleep.width+'%'}></i>{/if}
    <u data-mark="now" style:left={e.now+'%'}></u>
   </div>
   <div class="ticks">{#each e.ticks as t}<span data-thin="1" style:left={t.at+'%'}>{String(t.h).padStart(2,'0')}</span>{/each}</div>
   {#if e.window}<div class="win"><i style:left={e.window.left+'%'} style:width={e.window.width+'%'}></i></div>{/if}
   <span class="now" class:left={anchor==='left'} class:right={anchor==='right'} style:left={e.now+'%'} data-thin="3">Now {clock}</span>
  </div>
 </div>
{/if}
<style>
 .eve{display:grid;margin:2px 0 6px}
 .kick{font-size:11px;color:var(--text2);margin:0}
 .big{font:500 34px/1.05 Outfit;font-variant-numeric:tabular-nums;margin:4px 0 0;letter-spacing:-.01em}
 .line{font-size:12px;line-height:1.5;color:var(--text2);margin:6px 0 0}
 .strip{position:relative;margin-top:30px;container-type:inline-size}
 .track{position:relative;height:12px;border-radius:6px;background:var(--chart-track)}
 .track i{position:absolute;top:0;bottom:0}
 .track .wd{background:var(--fair);opacity:.5;border-radius:6px 0 0 6px}
 .track .fa{background:var(--goal-sleep);opacity:.6}
 .track u{position:absolute;top:-6px;bottom:-6px;width:2px;transform:translateX(-50%);border-radius:1px;background:var(--text);box-shadow:0 0 0 2px var(--bg)}
 .ticks{position:relative;height:15px;margin-top:6px}
 .ticks span{position:absolute;transform:translateX(-50%);font:500 11px Inter;color:var(--text2)}
 .ticks span:first-child{transform:none}.ticks span:last-child{transform:translateX(-100%)}
 .win{position:relative;height:14px;margin-top:2px}
 .win i{position:absolute;top:4px;height:2px;border-radius:1px;background:var(--text3)}
 .win i::before,.win i::after{content:"";position:absolute;top:-2px;width:2px;height:6px;border-radius:1px;background:var(--text3)}
 .win i::before{left:0}.win i::after{right:0}
 .now{position:absolute;bottom:calc(100% + 5px);transform:translateX(-50%);white-space:nowrap;font:600 12px/1.2 Inter;color:var(--text);background:var(--card);padding:1px 4px;border-radius:4px}
 .now.left{transform:none}.now.right{transform:translateX(-100%)}
</style>
