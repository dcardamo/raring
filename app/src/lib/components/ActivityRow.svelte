<script lang="ts">
 import type {ActivityCard} from '../generated/ActivityCard';
 import ZoneStrip from './ZoneStrip.svelte';
 import Icon from './Icon.svelte';
 import {clock,dateLabel,elapsed,num} from '../format';
 let {activity,open}:{activity:ActivityCard,open:()=>void}=$props();
</script>
<button class="activity" onclick={open}>
 <span class="act-ico"><Icon name={activity.icon} size={24}/></span>
 <span class="act-main">
  <b>{activity.name}</b>
  <small class="activity-date">{dateLabel(activity.startLocal.slice(0,10))} · {clock(activity.startLocal)}</small>
  <small>{activity.distanceKm!=null?num(activity.distanceKm,2)+' km · ':''}{elapsed(activity.durationS)}{activity.avgHr!=null?` · avg HR ${num(activity.avgHr)}`:''}</small>
 </span>
 {#if activity.load!=null}<span class="act-load"><b class="n">{Math.round(activity.load)}</b><small>Load</small></span>{/if}
</button>
{#if activity.zoneSeconds?.some(seconds=>seconds>0)}<ZoneStrip seconds={activity.zoneSeconds}/>{/if}
