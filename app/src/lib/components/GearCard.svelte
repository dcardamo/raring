<script lang="ts">
 import Card from './Card.svelte';
 import type {GearView} from '../generated/GearView';
 import {num} from '../format';
 let {gear}:{gear:GearView}=$props();
 let expanded=$state(false);
 let activeCount=$derived(gear.items.every(g=>g.status==='Active'||g.status==='Retired')
  ?gear.items.filter(g=>g.status==='Active').length:null);
 const distance=(value:number|null)=>value==null?'Distance unavailable':`${num(value,gear.unit==='m'?0:1)} ${gear.unit}`;
</script>
<Card title="Gear" extra="gear-card">
 <div class="gear-summary">
  <span>{activeCount==null?'Active count unavailable':`${activeCount} active`} · {gear.items.length} total</span>
  <button aria-expanded={expanded} onclick={()=>expanded=!expanded}>{expanded?'Less detail':'View gear'}</button>
 </div>
 {#each (expanded?gear.items:gear.items.slice(0,2)) as item (item.id)}
  <div class="gear-item">
   <div class="gear-heading"><b>{item.name}</b><span>{item.kind} · {item.status}</span></div>
   <div class="gear-distance"><b class="n">{distance(item.distance)}</b>{#if item.threshold!=null}<span>of {distance(item.threshold)} Garmin threshold</span>{:else}<span>No Garmin distance threshold</span>{/if}</div>
   {#if item.distance!=null && item.threshold!=null}<div class="gear-track" role="img" aria-label={`${distance(item.distance)} of ${distance(item.threshold)} Garmin threshold`}><i style:width={`${Math.min(100,item.distance/item.threshold*100)}%`}></i></div>{/if}
  </div>
 {/each}
 <p class="small gear-fetched">Distance from Garmin · fetched {new Date(gear.fetchedAt).toLocaleString()}</p>
</Card>
<style>
 .gear-summary,.gear-heading,.gear-distance{display:flex;justify-content:space-between;gap:10px;align-items:baseline}
 .gear-summary{font-size:13px;color:var(--muted,#a9b3c5);margin-bottom:8px}
 .gear-summary button{background:none;border:0;color:inherit;text-decoration:underline;cursor:pointer;font:inherit;padding:3px}
 .gear-item{padding:12px 0;border-top:1px solid rgba(255,255,255,.09)}
 .gear-heading{font-size:14px;min-width:0}.gear-heading b{overflow-wrap:anywhere}.gear-heading span{font-size:12px;white-space:nowrap;color:#aeb8ca}
 .gear-distance{font-size:12px;margin-top:6px;color:#aeb8ca}.gear-distance b{font-size:17px;color:#e4e8f0}.gear-distance span{text-align:right}
 .gear-track{height:5px;background:rgba(255,255,255,.12);border-radius:5px;margin-top:8px;overflow:hidden}.gear-track i{display:block;height:100%;background:#e4e8f0;border-radius:5px}
 .gear-fetched{margin:9px 0 0}
</style>
