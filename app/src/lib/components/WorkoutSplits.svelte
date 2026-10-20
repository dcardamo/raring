<script lang="ts">
 import type {Split} from '../generated/Split';
 import type {DistanceUnit} from '../generated/DistanceUnit';
 import Card from './Card.svelte';
 import {colors} from '../charts';
 import {elapsed,num} from '../format';
 import {distance,pace,paceUnit,speed,speedUnit} from '../units';
 // Rides split by speed, everything else by pace in the activity's unit
 // (per 100 m or yd for swims); laps without distance by time.
 let {splits,ride=false,unit}:{splits:Split[],ride?:boolean,unit:DistanceUnit}=$props();
 let measured=$derived(splits.every(s=>s.distanceKm>0 && s.seconds>0));
 // Larger is faster for every measure, so bars and "fastest" read the same way.
 let values=$derived(splits.map(s=>!measured?-s.seconds:ride?s.distanceKm/(s.seconds/3600):-s.seconds/s.distanceKm));
 let fast=$derived(Math.max(...values)),slow=$derived(Math.min(...values));
 let fastest=$derived(splits[values.indexOf(fast)]?.index);
 let equal=$derived(splits.slice(0,-1).every(s=>Math.abs(s.distanceKm-splits[0].distanceKm)<.005));
 const show=(v:number)=>!measured?elapsed(-v):ride?num(speed(v,unit),1):pace(-v,unit);
 let caption=$derived(!measured?'Lap time':ride?`Speed, ${speedUnit(unit)}`:`Pace ${paceUnit(unit)}`);
</script>
<Card title="Splits">
 {#snippet aside()}{#if measured && splits.length>1}<span class="pill ok">Fastest split {fastest}</span>{/if}{/snippet}
 <p class="small">{caption}{measured && equal?` · each ${distance(splits[0].distanceKm,unit)}`:''}{splits.length>1?` · slowest ${show(slow)}, fastest ${show(fast)}`:''}</p>
 {#each splits as split,i}
  <div class="split-row"><span>{split.index}</span><div class="bar"><i style:width={(fast===slow?1:.25+.75*(values[i]-slow)/(fast-slow))*100+'%'} style:background={measured && values[i]===fast?colors.good:'rgba(228,232,240,.45)'}></i></div><div class="split-value"><b class="n">{show(values[i])}</b>{#if split.distanceKm>0 && (!equal || Math.abs(split.distanceKm-splits[0].distanceKm)>=.005)}<small>{distance(split.distanceKm,unit)}</small>{/if}</div></div>
 {/each}
</Card>
<style>.split-row{grid-template-columns:22px 1fr 66px}.split-value{text-align:right}.split-value small{display:block;color:var(--text3);font-size:11px}</style>
