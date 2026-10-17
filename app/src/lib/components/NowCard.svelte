<script lang="ts">
 import type {TodayView} from '../generated/TodayView';
 import {batteryColor,stressColors,stressNames} from '../charts';
 import {clock,num} from '../format';
 import Card from './Card.svelte';
 import Icon from './Icon.svelte';
 // The latest synced readings, labeled with the watch's upload: never live data.
 let {today,lead=false,open}:{today:TodayView;lead?:boolean;open:(route:string)=>void}=$props();
 let done=$derived(today.activities.find(a=>a.startLocal.startsWith(today.date)));
 let asOf=$derived(today.watchLastUpload?`As of watch sync ${clock(today.watchLastUpload)}`:'Latest synced');
</script>

<Card title="Right now" extra="now-card{lead?' lead':''}">
 {#snippet aside()}<span class="source">{asOf}</span>{/snippet}
 <div class="now-grid">
  {#if today.bodyBattery.now!=null}
   <button class="now-tile" onclick={()=>open('metric/bodyBattery')}><span class="cell"><i style:height={Math.max(0,Math.min(100,today.bodyBattery.now))+'%'} style:background={batteryColor(today.bodyBattery.now)}></i></span><span><b class="n" style:color={batteryColor(today.bodyBattery.now)}>{num(today.bodyBattery.now)}</b><small>Body Battery</small></span></button>
  {/if}
  {#if today.stress.average!=null}
   <button class="now-tile" onclick={()=>open('metric/stress')}><span class="ring" style:color={today.stress.level?stressColors[today.stress.level]:undefined}><Icon name="pulse" size={18}/></span><span><b class="n">{num(today.stress.average)}</b><small>Stress{today.stress.level?` · ${stressNames[today.stress.level].toLowerCase()}`:''} avg</small></span></button>
  {/if}
 </div>
 {#if done}
  <button class="context" onclick={()=>open('workout/'+done.id)}><span class="icon"><Icon name={done.icon} size={20}/></span><span><small>Done today</small><b>{done.name}{#if done.load!=null} · <span class="n">{num(done.load)}</span> load{/if}</b></span></button>
 {:else if today.focus.recoveryHours!=null}
  <div class="context"><span class="icon"><Icon name="clock" size={20}/></span><span><small>Garmin's recovery time at readiness</small><b><span class="n">{num(today.focus.recoveryHours)}</span> h</b></span></div>
 {/if}
</Card>

<style>
 .source{font-size:10px;font-weight:500;letter-spacing:0;text-transform:none;color:var(--text2)}
 .now-grid{display:grid;grid-template-columns:repeat(auto-fit,minmax(130px,1fr));gap:10px}
 .now-tile{display:flex;align-items:center;gap:12px;padding:10px;border:0;border-radius:16px;background:rgba(255,255,255,.035);color:var(--text);text-align:left}
 .now-tile>span:last-child{display:grid;gap:4px}
 .now-tile b{font-size:28px;font-weight:600;line-height:1}
 small{font:500 11px Inter;color:var(--text2)}
 .cell{position:relative;display:flex;align-items:flex-end;width:22px;height:40px;padding:2px;border:2px solid rgba(255,255,255,.18);border-radius:6px}
 .cell i{display:block;width:100%;border-radius:3px}
 .ring{display:grid;place-items:center;width:36px;height:36px;border-radius:50%;box-shadow:inset 0 0 0 2px currentColor;color:var(--text2)}
 .context{display:flex;align-items:center;gap:10px;width:100%;margin-top:12px;padding:0;border:0;background:none;color:var(--text);text-align:left}
 .context>span:last-child{display:grid;gap:3px}
 .context b{font:600 16px Outfit}
 .icon{display:grid;place-items:center;width:38px;height:38px;flex:none;border-radius:12px;background:var(--card)}
</style>
