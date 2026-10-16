<script lang="ts">
 import type {TodayView} from '../generated/TodayView';
 import {limitingFactor} from '../signals';
 import {toneColor} from '../charts';
 import Card from './Card.svelte';
 import Icon from './Icon.svelte';

 let {today,open}:{today:TodayView;open:()=>void}=$props();
 let factor=$derived(limitingFactor(today.factors));
</script>

<Card title="Today's focus" onclick={open} extra="daily-focus">
 {#snippet aside()}<span class="source">From Garmin readiness</span>{/snippet}
 {#if today.call}
  <span class="sr-only">Suggested effort: {today.call}.</span>
  <div class="segments" aria-hidden="true">
   {#each ['rest','easy','steady','push'] as effort}
    <span class:on={today.call===effort}>{effort}</span>
   {/each}
  </div>
 {/if}
 {#if factor}
  <div class="focus-reason">
   <span class="factor-icon" style:color={toneColor(factor.tone)}><Icon name={factor.key==='sleep'||factor.key==='sleepHistory'?'moon':factor.key==='hrv'?'hrv':factor.key==='recovery'?'clock':'gauge'} size={20}/></span>
   <div><span class="reason-label">Lowest readiness factor</span><strong>{factor.name}</strong></div>
   <b class="n" style:color={toneColor(factor.tone)}>{Math.round(factor.percent!)}<small>/100</small></b>
  </div>
  <p class="focus-detail">{factor.detail}</p>
 {:else if today.callNote}
  <p class="focus-detail">{today.callNote}</p>
 {/if}
 <div class="focus-footer"><span>See all readiness factors</span><Icon name="arrow-right" size={16}/></div>
</Card>

<style>
 .sr-only{position:absolute;width:1px;height:1px;overflow:hidden;clip-path:inset(50%);white-space:nowrap}
 .source{font-size:10px;font-weight:500;letter-spacing:0;text-transform:none;color:var(--text2)}
 .focus-reason{display:flex;align-items:center;gap:10px;margin-top:18px}
 .factor-icon{display:grid;place-items:center;width:38px;height:38px;background:var(--card);border-radius:12px}
 .focus-reason>div{flex:1}
 .reason-label{display:block;font-size:11px;color:var(--text2);margin-bottom:3px}
 strong{font:600 17px Outfit}
 .focus-reason>b{font-size:26px}
 small{font:500 11px Inter;color:var(--text2);margin-left:2px}
 .focus-detail{color:var(--text2);font-size:12px;line-height:1.5;margin-top:10px}
 .focus-footer{display:flex;justify-content:space-between;align-items:center;margin-top:14px;padding-top:12px;border-top:1px solid var(--line);font-size:12px;color:var(--mint)}
</style>
