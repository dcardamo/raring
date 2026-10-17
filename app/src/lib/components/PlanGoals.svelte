<script lang="ts">
 import type {PlanProgress} from '../generated/PlanProgress';
 import {num} from '../format';
 import {planColor,planIcons,planNames,planUnits} from '../plan';
 import {colors} from '../charts';
 import Icon from './Icon.svelte';
 let {items,compact=false}:{items:PlanProgress[];compact?:boolean}=$props();
</script>

<div class="goals" class:compact>
 {#each items as item (item.kind)}
  {@const color=item.unavailable?colors.secondary:planColor(item.state)}
  <div class="goal" data-kind={item.kind} data-state={item.unavailable?'unavailable':item.state}>
   <span class="goal-icon"><Icon name={planIcons[item.kind]} size={compact?16:18}/></span>
   <div class="goal-main">
    <div class="goal-title"><b>{planNames[item.kind]}</b>
     {#if item.unavailable}<span class="n">—</span>
     {:else}<span class="n"><strong style:color={item.state==='behind'?colors.fair:item.state==='done'&&compact?colors.good:undefined}>{num(item.value)}</strong>{compact?'/':' of '}{num(item.target)}{compact?'':` ${planUnits[item.kind]}`}</span>{/if}
    </div>
    <div class="goal-bar" role="presentation"><i style:width={(item.unavailable?0:item.completion*100)+'%'} style:background={color}></i></div>
    {#if !compact}<small>{#if item.unavailable}{item.unavailable}{:else}{#if item.state==='behind'}<i class="dot" aria-label="Behind pace"></i>{/if}{item.note}{/if}</small>
    {:else if item.unavailable}<small>{item.unavailable}</small>{/if}
   </div>
  </div>
 {/each}
</div>

<style>
 .goals{display:grid;gap:18px}
 .goals.compact{gap:12px}
 .goal{display:grid;grid-template-columns:auto 1fr;gap:14px;align-items:start}
 .goal-icon{width:34px;height:34px;border-radius:11px;display:grid;place-items:center;background:rgba(255,255,255,.06);color:var(--text2)}
 .compact .goal-icon{width:28px;height:28px;border-radius:9px}
 .goal-main{min-width:0}
 .goal-title{display:flex;justify-content:space-between;align-items:baseline;gap:8px}
 .goal-title b{font:600 16px Inter}
 .compact .goal-title b{font:500 14px Inter}
 .goal-title span{font:500 14px Inter;color:var(--text2);white-space:nowrap}
 .goal-title strong{font:600 22px Outfit;color:var(--text);margin-right:2px}
 .compact .goal-title strong{font-size:16px}
 .goal-bar{height:7px;border-radius:4px;background:rgba(255,255,255,.08);margin-top:8px;overflow:hidden}
 .compact .goal-bar{height:5px;margin-top:6px}
 .goal-bar i{display:block;height:100%;border-radius:4px}
 small{display:flex;align-items:center;gap:6px;margin-top:7px;font-size:13px;color:var(--text2)}
 .dot{width:8px;height:8px;border-radius:50%;background:var(--fair);flex:none}
</style>
