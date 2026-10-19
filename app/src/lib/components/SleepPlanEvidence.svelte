<script lang="ts">
 import type {SleepPlan} from '../generated/SleepPlan';
 import {duration} from '../format';
 // Where tonight's plan comes from: Garmin's window or Raring's estimate, the need and the wind-down lead.
 let {plan}:{plan:SleepPlan}=$props();
 let garmin=$derived(plan.bedtimeSource==='garmin');
 let wake=$derived(plan.wakeNights===14?'Usual wake':'Wake reference');
 let needSource=$derived(plan.source==='tonight'?'Garmin\'s need for tonight':plan.source==='average'?'14-day average of Garmin\'s sleep need':'Garmin\'s latest sleep need');
 let wakes=$derived(`median of ${plan.wakeNights} recorded wake ${plan.wakeNights===1?'time':'times'}`);
</script>
<dl class="evidence">
 <dt>{garmin?'Bedtime window':'Asleep by'}</dt>
 <dd>{garmin?'Garmin\'s recommendation for tonight.':`Raring's estimate: ${wake.toLowerCase()} ${plan.wakeTime} (${wakes}) minus the need. Garmin sent no window for tonight.`}</dd>
 <dt>Sleep need</dt>
 <dd>{needSource}{plan.baselineMin!=null?`; your baseline is ${duration(plan.baselineMin)}`:''}.{#each plan.adjustments as a}<br>{a.factor}: {a.detail}{/each}</dd>
 <dt>Wind down</dt>
 <dd>{plan.windDownLeadMin} min before {garmin?'the window starts':'the asleep-by time'}. Change it in Settings.</dd>
 {#if plan.tip}<dt>Rhythm</dt><dd>{plan.tip}</dd>{/if}
</dl>
<p>A suggestion, not an alarm. Raring doesn't change your watch's sleep schedule.</p>
<style>
 .evidence{display:grid;grid-template-columns:auto 1fr;gap:8px 12px;margin:8px 0 0;font-size:12px;line-height:1.45}
 dt{color:var(--text3);font-weight:600}
 dd{margin:0;color:var(--text2)}
 p{font-size:13px;color:var(--text2)}
</style>
