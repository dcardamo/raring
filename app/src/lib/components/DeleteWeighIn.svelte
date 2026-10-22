<script lang="ts">
 import * as api from '../api';
 import type {WeighIn} from '../generated/WeighIn';
 import type {WeightUnit} from '../generated/WeightUnit';
 import {dateLabel,num} from '../format';
 import {refusal,writeSummary} from '../garminWrite';
 // The weight detail's last weigh-in, deleted from Garmin only after an in-place confirmation
 // (Rules: Garmin writes). The core finds it again in a fresh read before deleting. A deleted
 // weigh-in is handed to the parent, which drops it; other answers stay here.
 let {weighIn,unit,onwrite,ondeleted}:{weighIn:WeighIn,unit:WeightUnit,onwrite?:()=>void,ondeleted?:(text:string)=>void}=$props();
 let confirming=$state(false),busy=$state(false),status=$state('');
 let label=$derived(`${num(weighIn.value,1)} ${unit}${weighIn.bodyFat!=null?` · ${num(weighIn.bodyFat,1)} % body fat`:''} · ${dateLabel(weighIn.date)}${weighIn.time?` · ${weighIn.time}`:''}`);
 async function remove(){
  busy=true;status='';
  try{
   const change=await api.garmin_write({kind:'deleteWeighIn',date:weighIn.date,timestampGmt:weighIn.timestampGmt,samplePk:weighIn.samplePk,confirm:true});
   confirming=false;
   if(change.outcome==='applied'){ondeleted?.(`Deleted ${label} from Garmin.`);return}
   status=writeSummary(change);
   if(change.outcome==='conflict'||change.outcome==='needsSignIn')onwrite?.();
  }catch(e){status=refusal(e)}finally{busy=false}
 }
</script>
<p class="reading"><b class="n">{label}</b><small>{weighIn.source==='MANUAL'?'Entered by hand':'From Garmin'}</small></p>
{#if !confirming}
 <button class="secondary danger" onclick={()=>{confirming=true;status=''}}>Delete this weigh-in</button>
{:else}
 <div class="confirm" role="group" aria-label="Delete this weigh-in">
  <p>Delete {label} from Garmin? It goes from Garmin Connect too, and Raring can’t bring it back.</p>
  <div class="acts"><button class="secondary danger" disabled={busy} onclick={remove}>{busy?'Deleting…':'Delete from Garmin'}</button><button class="secondary" disabled={busy} onclick={()=>confirming=false}>Cancel</button></div>
 </div>
{/if}
{#if status}<p class="small" role="status">{status}</p>{/if}
<style>
 .reading{display:flex;justify-content:space-between;align-items:baseline;gap:12px;flex-wrap:wrap}.reading b{font-size:16px}.reading small{font:500 12px Inter;color:var(--text2)}
 .confirm p{font-size:13px;line-height:1.45;margin-top:12px}
 .acts{display:flex;flex-wrap:wrap;gap:8px}.acts button{flex:1 1 120px;width:auto}
</style>
