<script lang="ts">
 import type {ActivityCard} from '../generated/ActivityCard';
 import ZoneStrip from './ZoneStrip.svelte';
 import Icon from './Icon.svelte';
 import {clock,dateLabel,elapsed,num} from '../format';
 import {distance} from '../units';
 let {activity,open,compact=false}:{activity:ActivityCard,open:()=>void,compact?:boolean}=$props();
</script>
<button class="activity" class:compact onclick={open}>
 <span class="act-ico"><Icon name={activity.icon} size={24}/></span>
 <span class="act-main">
  <b>{activity.name}</b>
  <small class="activity-date">{compact ? clock(activity.startLocal) : `${dateLabel(activity.startLocal.slice(0,10))} · ${clock(activity.startLocal)}`}</small>
  <small>{activity.distanceKm!=null?distance(activity.distanceKm,activity.distanceUnit)+' · ':''}{elapsed(activity.durationS)}{activity.avgHr!=null?` · avg HR ${num(activity.avgHr)}`:''}</small>
 </span>
 {#if activity.load!=null}<span class="act-load"><b class="n">{Math.round(activity.load)}</b><small>Load</small></span>{/if}
</button>
{#if !compact && activity.zoneSeconds?.some(seconds=>seconds==null||seconds>0)}<ZoneStrip seconds={activity.zoneSeconds}/>{/if}
<style>
 .compact{display:grid;grid-template-columns:auto minmax(0,1fr) auto;gap:10px}
 .compact .act-main b,.compact .act-main small{overflow-wrap:anywhere}
 .compact .act-load{align-self:start}
 @container(max-width:25em){.compact{grid-template-columns:auto minmax(0,1fr)}.compact .act-load{grid-column:2;display:flex;align-items:baseline;gap:6px;text-align:left}.compact .act-load b{font-size:18px}}
</style>
