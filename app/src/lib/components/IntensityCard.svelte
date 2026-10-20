<script lang="ts">
 import type {IntensityCard} from '../generated/IntensityCard';
 import {colors,goalColors} from '../charts';
 import {num} from '../format';
 import {weekRange} from '../plan';
 import Card from './Card.svelte';
 import Gauge from './Gauge.svelte';
 let {intensity,open,id}:{intensity:IntensityCard;open?:()=>void;id?:string}=$props();
 let met=$derived(intensity.goal!=null && intensity.total>=intensity.goal);
</script>

<Card title="Intensity minutes" onclick={open} extra="intensity-card" {id}>
 {#snippet aside()}<span class="pill">{weekRange(intensity.start,intensity.end)}</span>{/snippet}
 <div class="ring">
  <Gauge size={112} stroke={11} value={intensity.total} max={intensity.goal??Math.max(intensity.total,1)} display={num(intensity.total)} caption={intensity.goal!=null?`of ${num(intensity.goal)}`:'min'} color={met?colors.good:goalColors.intensity}/>
  <div class="split">
   <div><b class="n">{num(intensity.moderate)}</b><span>moderate min</span></div>
   <div><b class="n">{num(intensity.vigorous)}</b><span>vigorous min, counted twice</span></div>
   {#if intensity.goal!=null}<p>{met?"Garmin's weekly goal reached":`${num(Math.ceil(intensity.goal-intensity.total))} min to Garmin's weekly goal`}</p>{/if}
  </div>
 </div>
</Card>

<style>
 .ring{display:flex;align-items:center;gap:20px}
 .split{display:grid;gap:10px;min-width:0}
 .split div{display:flex;align-items:baseline;gap:8px}
 .split b{font-size:22px}
 .split span,p{font-size:13px;color:var(--text2)}
</style>
