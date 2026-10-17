<script lang="ts">
 import type {SleepPlan} from '../generated/SleepPlan';
 import {duration} from '../format';
 import Card from './Card.svelte';
 import Icon from './Icon.svelte';
 // `evidence` adds the "Why this plan" sheet; Today opens Sleep instead.
 let {plan,open,evidence=false}:{plan:SleepPlan;open?:()=>void;evidence?:boolean}=$props();
 let why=$state(false);
 let garmin=$derived(plan.bedtimeSource==='garmin');
 let wake=$derived(plan.wakeNights===14?'Usual wake':'Wake reference');
 let needSource=$derived(plan.source==='tonight'?'Garmin\'s need for tonight':plan.source==='average'?'14-day average of Garmin\'s sleep need':'Garmin\'s latest sleep need');
 let wakes=$derived(`median of ${plan.wakeNights} recorded wake ${plan.wakeNights===1?'time':'times'}`);
</script>

<Card title="Tonight's sleep" onclick={open} extra="sleep-plan">
 {#snippet aside()}<span class="pill"><Icon name="moon" size={13}/> {garmin?'Garmin plan':'Raring estimate'}</span>{/snippet}
 <div class="plan-row">
  <div><span>Wind down</span><b class="n">{plan.windDown}</b></div>
  {#if plan.bedtimeStart}
   <div class="window"><span>Bedtime</span><b class="n">{plan.bedtimeStart}–{plan.bedtime}</b></div>
  {:else}
   <div class="window"><span>Asleep by</span><b class="n">{plan.bedtime}</b></div>
  {/if}
  <div><span>Sleep need</span><b class="n">{duration(plan.needMin)}</b></div>
 </div>
 {#if plan.stale}
  <p>No sleep has synced today; this estimate uses older nights.</p>
 {:else if garmin}
  <p>{plan.reason ?? 'Garmin\'s bedtime window and sleep need for tonight.'}</p>
 {:else}
  <p>{wake} {plan.wakeTime} ({wakes}) minus {plan.source==='tonight'?'tonight\'s need':plan.source==='average'?'your 14-day average need':'your latest need'}.</p>
 {/if}
 {#if evidence}
  <button class="why" aria-expanded={why} onclick={()=>why=!why}>Why this plan <span class="chevron" class:open={why}><Icon name="arrow-right" size={13}/></span></button>
  {#if why}
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
  {/if}
 {/if}
</Card>

<style>
 .plan-row{display:grid;grid-template-columns:auto 1fr auto;align-items:end;gap:12px}
 .plan-row>div{display:grid;gap:6px}
 .plan-row>.window{text-align:center}
 .plan-row>div:last-child{text-align:right}
 .plan-row span{font-size:11px;color:var(--text2);white-space:nowrap}
 .plan-row b{font-size:26px;font-weight:500;line-height:1;white-space:nowrap}
 p{font-size:12px;line-height:1.5;color:var(--text2);margin-top:14px}
 .why{display:inline-flex;align-items:center;gap:4px;margin-top:10px;padding:6px 0;background:none;border:0;color:var(--text);font:600 12px Inter}
 .evidence{display:grid;grid-template-columns:auto 1fr;gap:8px 12px;margin:8px 0 0;font-size:12px;line-height:1.45}
 .chevron{display:inline-flex;transform:rotate(90deg);transition:transform .15s}
 .chevron.open{transform:rotate(-90deg)}
 dt{color:var(--text3);font-weight:600}
 dd{margin:0;color:var(--text2)}
 :global(.sleep-plan){container-type:inline-size}
 @container (max-width:310px){.plan-row{gap:8px}.plan-row b{font-size:21px}}
</style>
