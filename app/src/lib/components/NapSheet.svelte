<script lang="ts">
 import Icon from './Icon.svelte';
 import type {DayEvent} from '../generated/DayEvent';
 import {sheetOpened} from '../sheet';
 import {dateLabel,duration,minuteClock} from '../format';
 // A nap is its own recorded event, not part of last night's sleep. Show its
 // times and Body Battery change without opening the sleep page.
 let {nap, date, close}:{nap:DayEvent,date:string,close:()=>void}=$props();
 $effect(() => sheetOpened(close));
 const when = (m: number | null) => m == null ? 'Not recorded' : minuteClock(m);
 const change = $derived(nap.bodyBatteryChange == null ? 'Not measured'
  : `${nap.bodyBatteryChange > 0 ? '+' : ''}${nap.bodyBatteryChange}`);
</script>
<svelte:window onkeydown={e => { if (e.key === 'Escape') close(); }}/>
<button class="scrim" aria-label="Close" onclick={close}></button>
<div class="sheet" role="dialog" aria-modal="true" aria-labelledby="nap-title">
 <span class="grab"></span>
 <div class="sheet-h"><h2 id="nap-title">Nap</h2><button class="close" aria-label="Close" onclick={close}><Icon name="close" size={20}/></button></div>
 <p class="small">Garmin · {dateLabel(date)}</p>
 <div class="facts">
  <div><span>Start</span><b class="n">{when(nap.startMin)}</b></div>
  <div><span>End</span><b class="n">{when(nap.endMin)}</b></div>
  <div><span>Duration</span><b class="n">{nap.durationMin == null ? 'Unavailable' : duration(nap.durationMin)}</b></div>
  <div><span>Body Battery</span><b class="n">{change}</b></div>
 </div>
 {#if nap.feedback}<p class="small">Garmin feedback · {nap.feedback.replaceAll('_', ' ').toLowerCase()}</p>{/if}
 <p class="small">Garmin records naps separately from last night’s sleep: this doesn’t change its score, time asleep or tonight’s need.</p>
</div>
<style>
 .scrim{position:fixed;inset:0;border:0;background:rgba(5,8,14,.62);z-index:20;padding:0}
 .sheet{position:fixed;left:0;right:0;bottom:0;z-index:21;max-width:560px;margin:0 auto;background:var(--card);border:1px solid var(--line);border-bottom:0;border-radius:26px 26px 0 0;padding:12px 20px calc(28px + env(safe-area-inset-bottom));max-height:90dvh;overflow:auto}
 .grab{display:block;width:40px;height:4px;border-radius:2px;background:rgba(255,255,255,.18);margin:0 auto 16px}
 .sheet-h{display:flex;justify-content:space-between;align-items:center;gap:12px}.sheet-h h2{min-width:0;overflow-wrap:anywhere}
 .close{flex:none;width:44px;height:44px;border-radius:50%;border:1px solid var(--line);background:none;display:grid;place-items:center;color:var(--text2)}
 .facts{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:10px;margin:16px 0}
 .facts div{padding:12px 14px;border:1px solid var(--line);border-radius:16px;background:rgba(255,255,255,.04);min-width:0}
 .facts span{display:block;font:500 12px Inter;color:var(--text2)}
 .facts b{display:block;margin-top:2px;font-size:19px}
</style>
