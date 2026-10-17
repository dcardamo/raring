<script lang="ts">
 import Card from './Card.svelte';
 import type {PlannedTraining} from '../generated/PlannedTraining';
 import {dateLabel} from '../format';
 let {planned, quiet=false, open}:{planned:PlannedTraining;quiet?:boolean;open?:()=>void}=$props();
</script>
{#if !quiet}
 <Card title="Next session" onclick={open} extra="planned-session">
  {#if planned.session}
   <div class="planned-detail"><b>{planned.session.name}</b><span>{dateLabel(planned.session.date)} · Garmin schedule</span>{#if planned.coach}<span>Active Coach plan · {planned.coach}</span>{/if}</div>
  {:else if planned.state==='empty'}
   <p class="small">No workouts in Garmin’s cached schedule{#if planned.coverageStart && planned.coverageEnd}, {dateLabel(planned.coverageStart)}–{dateLabel(planned.coverageEnd)}{/if}.{#if planned.coach} {planned.coach} is active.{/if}</p>
  {:else if planned.state==='stale'}
   <p class="small">Schedule needs a refresh.{#if planned.coverageStart && planned.coverageEnd} Cached coverage: {dateLabel(planned.coverageStart)}–{dateLabel(planned.coverageEnd)}.{/if}{#if planned.coach} {planned.coach} is active.{/if}</p>
  {:else}
   <p class="small">Schedule unavailable.{#if planned.coach} {planned.coach} is active.{/if}</p>
  {/if}
 </Card>
{/if}
<style>
 .planned-detail{display:flex;flex-direction:column;gap:5px}.planned-detail b{font:600 17px Outfit,Inter,sans-serif}.planned-detail span{color:var(--text2);font-size:13px}
</style>
