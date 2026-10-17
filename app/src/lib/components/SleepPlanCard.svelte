<script lang="ts">
 import type {SleepPlan} from '../generated/SleepPlan';
 import {duration} from '../format';
 import Card from './Card.svelte';
 import Icon from './Icon.svelte';
 let {plan,open}:{plan:SleepPlan;open?:()=>void}=$props();
</script>

<Card title="Tonight's sleep" onclick={open} extra="sleep-plan">
 {#snippet aside()}<span class="pill violet"><Icon name="moon" size={13}/> Plan ahead</span>{/snippet}
 {#if plan.bedtimeStart}
  <div class="sleep-window">
   <div><span>Garmin bedtime</span><b class="n">{plan.bedtimeStart}–{plan.bedtime}</b></div>
   <div><span>Sleep need</span><b class="n need">{duration(plan.needMin)}</b></div>
  </div>
  <p>Garmin's bedtime window and need for tonight. A suggestion, not an alarm.</p>
 {:else}
  <div class="sleep-window">
   <div><span>Asleep by</span><b class="n">{plan.bedtime}</b></div>
   <div class="sleep-bridge"><span class="n">{duration(plan.needMin)}</span><i></i><small>sleep need</small></div>
   <div><span>{plan.wakeNights===14?'Usual wake':'Wake reference'}</span><b class="n">{plan.wakeTime}</b></div>
  </div>
  <p>{plan.source==='tonight'?"Garmin's need for tonight":plan.source==='average'?'14-day average Garmin sleep need':'Latest Garmin sleep need'} · median of {plan.wakeNights} recorded wake {plan.wakeNights===1?'time':'times'}. A suggestion, not an alarm.</p>
 {/if}
</Card>

<style>
 .sleep-window{display:flex;align-items:center;justify-content:space-between;gap:12px}
 .sleep-window>div{display:grid;gap:6px}
 .sleep-window>div:last-child{text-align:right}
 .sleep-window span{font-size:11px;color:var(--text2)}
 .sleep-window b{font-size:30px;font-weight:500;line-height:1}
 .sleep-window b.need{color:var(--violet)}
 .sleep-bridge{flex:1;text-align:center;max-width:120px}
 .sleep-bridge span{font-size:20px;color:var(--violet)}
 .sleep-bridge i{display:block;height:2px;background:linear-gradient(90deg,transparent,var(--violet),transparent)}
 small{font-size:10px;color:var(--text2)}
 p{font-size:11px;line-height:1.5;color:var(--text2);margin-top:16px}
</style>
