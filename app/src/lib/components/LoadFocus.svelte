<script lang="ts">
 import type {LoadFocusBar} from '../generated/LoadFocusBar';
 import {colors,clamp} from '../charts';
 // Garmin's target range is a solid neutral band with its numbers; the value is a marker on top.
 let {bars}:{bars:LoadFocusBar[]}=$props();
 let max=$derived(Math.ceil(Math.max(900,...bars.map(b=>Math.max(b.target?.[1]??0,b.value)*1.1))/100)*100);
</script>
{#each bars as b}<div class="lf" role="img" aria-label={`${b.name} ${Math.round(b.value)}${b.target?`; Garmin's target ${b.target[0]}–${b.target[1]}`:''}${b.shortage?'; shortage':''}`}>
 <div class="top"><span>{b.name} {#if b.shortage}<em>Shortage</em>{/if}</span><b>{Math.round(b.value)}</b></div>
 <div class="bar">{#if b.target}<u style:left={clamp(b.target[0]/max)*100+'%'} style:width={(clamp(b.target[1]/max)-clamp(b.target[0]/max))*100+'%'}></u>{/if}<i style:width={clamp(b.value/max)*100+'%'} style:background={b.shortage?colors.fair:'rgba(228,232,240,.75)'}></i></div>
 {#if b.target}<small>Target {b.target[0]}–{b.target[1]}</small>{/if}
</div>{/each}
<style>
 .lf{margin-top:14px}.lf:first-child{margin-top:0}
 .top{display:flex;justify-content:space-between;align-items:center;font-size:13px;color:var(--text2);margin-bottom:8px}.top b{font:600 15px Outfit;color:var(--text)}
 em{font:600 11px Inter;color:var(--fair);background:rgba(255,181,71,.15);padding:4px 8px;border-radius:20px;font-style:normal}
 .bar{position:relative;height:10px;border-radius:5px;background:rgba(255,255,255,.06)}
 u{position:absolute;top:-3px;bottom:-3px;background:rgba(228,232,240,.22);border-radius:6px}
 i{position:absolute;left:0;top:2px;bottom:2px;border-radius:4px}
 small{display:block;margin-top:5px;font-size:11.5px;color:var(--text2)}
</style>
