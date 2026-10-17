<script lang="ts">
 import type {TodayView} from '../generated/TodayView';
 import {limitingFactor} from '../signals';
 import {toneColor} from '../charts';
 import {duration} from '../format';
 import {watchLabel} from '../watch';
 import Card from './Card.svelte';
 import Icon from './Icon.svelte';

 // Mornings lead with last night, then the limiting factor, then the effort.
 let {today,now=new Date(),lead=false,open}:{today:TodayView;now?:Date;lead?:boolean;open:()=>void}=$props();
 let focus=$derived(today.focus);
 let morning=$derived(focus.phase==='wake');
 // Before last night syncs, yesterday's factors and effort aren't about this morning.
 let waiting=$derived(focus.sleepMissing && !focus.effort);
 let factor=$derived(waiting?null:limitingFactor(today.factors));
 let missingReason=$derived(today.missingMetricReasons.sleep ?? (today.watchSleepUploadStale?'Last night\'s sleep hasn\'t uploaded yet.':'Your sleep appears after the watch uploads to Garmin Connect.'));
</script>

{#snippet effort()}
 {#if today.call && focus.effort}
  <span class="sr-only">Suggested effort: {today.call}.</span>
  <div class="segments" aria-hidden="true">
   {#each ['rest','easy','steady','push'] as effort}
    <span class:on={today.call===effort}>{effort}</span>
   {/each}
  </div>
 {/if}
{/snippet}

<Card title={morning?'This morning':'Today\'s focus'} onclick={open} extra="daily-focus{lead?' lead':''}">
 {#snippet aside()}<span class="source">From Garmin readiness</span>{/snippet}
 {#if morning}
  {#if focus.lastNight}
   {@const night=focus.lastNight}
   <div class="night">
    <span class="factor-icon" style:color={today.sleep?toneColor(today.sleep.tone):undefined}><Icon name="moon" size={20}/></span>
    <div><span class="reason-label">Last night{night.woke?` · woke ${night.woke}`:''}</span><strong><span class="n">{duration(night.asleepMin)}</span> asleep</strong></div>
    {#if night.needMin!=null}<small>need <b class="n">{duration(night.needMin)}</b></small>{/if}
   </div>
  {:else}
   <div class="night">
    <span class="factor-icon muted-icon"><Icon name="moon" size={20}/></span>
    <div><span class="reason-label">Last night</span><strong>Not synced yet</strong></div>
    {#if today.watchLastUpload}<small>{watchLabel(today.watchLastUpload,now)}</small>{/if}
   </div>
   <p class="focus-detail">{missingReason}</p>
  {/if}
 {:else}
  {@render effort()}
 {/if}
 {#if factor}
  <div class="focus-reason">
   <span class="factor-icon" style:color={toneColor(factor.tone)}><Icon name={factor.key==='sleep'||factor.key==='sleepHistory'?'moon':factor.key==='hrv'?'hrv':factor.key==='recovery'?'clock':'gauge'} size={20}/></span>
   <div><span class="reason-label">Lowest readiness factor</span><strong>{factor.name}</strong></div>
   {#if factor.percent!=null}<b class="n" style:color={toneColor(factor.tone)}>{Math.round(factor.percent)}<small>/100</small></b>{/if}
  </div>
  <p class="focus-detail">{factor.detail}</p>
 {:else if today.callNote && !waiting}
  <p class="focus-detail">{today.callNote}</p>
 {/if}
 {#if morning}<div class="after">{@render effort()}</div>{/if}
 <div class="focus-footer"><span>See all readiness factors</span><Icon name="arrow-right" size={16}/></div>
</Card>

<style>
 .sr-only{position:absolute;width:1px;height:1px;overflow:hidden;clip-path:inset(50%);white-space:nowrap}
 .source{font-size:10px;font-weight:500;letter-spacing:0;text-transform:none;color:var(--text2)}
 .focus-reason,.night{display:flex;align-items:center;gap:10px;margin-top:18px}
 .night{margin-top:0}
 .night>div{flex:1;min-width:0}
 .night small b{font:600 14px Outfit;color:var(--text)}
 .muted-icon{color:var(--text3)}
 .after:not(:empty){margin-top:16px}
 .factor-icon{display:grid;place-items:center;width:38px;height:38px;background:var(--card);border-radius:12px}
 .focus-reason>div{flex:1}
 .reason-label{display:block;font-size:11px;color:var(--text2);margin-bottom:3px}
 strong{font:600 17px Outfit}
 .focus-reason>b{font-size:26px}
 small{font:500 11px Inter;color:var(--text2);margin-left:2px}
 .focus-detail{color:var(--text2);font-size:12px;line-height:1.5;margin-top:10px}
 .focus-footer{display:flex;justify-content:space-between;align-items:center;margin-top:14px;padding-top:12px;border-top:1px solid var(--line);font-size:12px;color:var(--ink)}
</style>
