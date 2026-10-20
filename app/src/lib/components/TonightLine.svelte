<script lang="ts">
 import type {SleepPlan} from '../generated/SleepPlan';
 import {tonightLine} from '../tonight';
 import {duration} from '../format';
 import {declutter} from '../declutter';
 // Tonight on a line from the evening to the usual wake, in the same language as
 // Today's wind-down strip: the times live in the facts beside it, so the graphic
 // carries only positions, the hour axis and a key.
 let {plan,now,garmin}:{plan:SleepPlan,now:Date,garmin:boolean}=$props();
 let line=$derived(tonightLine(plan,now));
 let hhmm=$derived(`${String(now.getHours()).padStart(2,'0')}:${String(now.getMinutes()).padStart(2,'0')}`);
 let label=$derived(`Tonight: now ${hhmm}; wind down from ${plan.windDown}; ${garmin?`Garmin's window ${plan.bedtimeStart??plan.bedtime} to ${plan.bedtime}`:`asleep by ${plan.bedtime}`}${plan.wakeTime?`; expected wake ${plan.wakeTime}`:''}; need ${duration(plan.needMin)}`);
</script>
{#if line}
 <div class="line" role="img" aria-label={label} data-chart="tonight" {@attach declutter}>
  <div class="lanes">
   <div class="track">
    <i class="wd" style:left={line.windDown+'%'} style:width={line.start-line.windDown+'%'}></i>
    <i class="bw" style:left={line.start+'%'} style:width={Math.max(1,line.end-line.start)+'%'}></i>
   </div>
   {#if line.needFrom!=null && line.wake!=null}
    <div class="track lane2"><i style:left={line.needFrom+'%'} style:width={line.wake-line.needFrom+'%'}></i><b data-mark="wake" style:left={line.wake+'%'}></b></div>
   {/if}
   {#if line.now!=null}<u data-mark="now" style:left={line.now+'%'}></u>{/if}
  </div>
  <div class="ticks">{#each line.ticks as tick}<span data-thin class:first={tick.left<3} class:last={tick.left>97} style:left={tick.left+'%'}>{tick.label}</span>{/each}</div>
  <div class="key">
   <span><i class="wd"></i>Wind down</span>
   <span><i class="bw"></i>{garmin?'Bedtime window':'Asleep by'}</span>
   {#if line.wake!=null}<span><i class="sl"></i>Need, to usual wake</span>{/if}
   {#if line.now!=null}<span><i class="now"></i>Now</span>{/if}
  </div>
 </div>
{/if}
<style>
 .line{padding-top:6px}
 .lanes{position:relative}
 .track{position:relative;height:12px;border-radius:6px;background:rgba(255,255,255,.06)}
 .track i{position:absolute;top:0;bottom:0;border-radius:3px}
 .track.lane2{height:6px;margin-top:5px;background:none}
 .track.lane2 b{position:absolute;top:-3px;width:3px;height:12px;margin-left:-3px;border-radius:2px;background:var(--text)}
 .wd{background:var(--fair)}
 .bw{background:var(--goal-sleep)}
 .sl,.track.lane2 i{background:var(--deep)}
 u{position:absolute;top:-6px;bottom:-6px;width:3px;margin-left:-1.5px;border-radius:2px;background:var(--text);box-shadow:0 0 0 2px var(--card)}
 .ticks{position:relative;height:1.4em;margin-top:8px;font:500 11px Inter;color:var(--text2)}
 .ticks span{position:absolute;transform:translateX(-50%);white-space:nowrap}
 .ticks .first{transform:none}.ticks .last{transform:translateX(-100%)}
 .key{display:flex;flex-wrap:wrap;gap:4px 14px;margin-top:6px;font-size:12px;color:var(--text2)}
 .key span{display:inline-flex;align-items:center;gap:5px}
 .key i{width:10px;height:10px;border-radius:3px}
 .key i.now{width:3px;height:12px;border-radius:2px;background:var(--text)}
</style>
