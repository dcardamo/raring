<script lang="ts">
 import type {Split} from '../generated/Split';
 import Card from './Card.svelte';
 import {colors} from '../charts';
 import {elapsed,num} from '../format';
 let {splits}:{splits:Split[]}=$props();
 let paced=$derived(splits.every(s=>s.distanceKm>0 && s.seconds>0));
 let values=$derived(splits.map(s=>paced?s.seconds/s.distanceKm:s.seconds));
 let slow=$derived(Math.max(...values)),fast=$derived(Math.min(...values));
 let fastest=$derived(splits[values.indexOf(fast)]?.index);
</script>
<Card title="Splits">
 {#snippet aside()}{#if paced && splits.length>1}<span class="pill ok">Fastest split {fastest}</span>{/if}{/snippet}
 <p class="small">{paced?'Pace /km · lap distance':'Lap duration'}</p>
 {#each splits as split,i}
  <div class="split-row"><span>{split.index}</span><div class="bar"><i style:width={(slow-values[i]+15)/(slow-fast+15)*92+'%'} style:background={paced && values[i]===fast?colors.mint:'rgba(255,138,61,.6)'}></i></div><div class="split-value"><b class="n">{elapsed(values[i])}</b>{#if split.distanceKm>0}<small>{num(split.distanceKm,2)} km</small>{/if}</div></div>
 {/each}
</Card>
<style>.split-row{grid-template-columns:22px 1fr 66px}.split-value{text-align:right}.split-value small{display:block;color:var(--text3);font-size:11px}</style>
