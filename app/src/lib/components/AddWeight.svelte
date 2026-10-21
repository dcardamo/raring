<script lang="ts">
 import Icon from './Icon.svelte';
 import * as api from '../api';
 import type {WeightUnit} from '../generated/WeightUnit';
 import type {GarminChange} from '../generated/GarminChange';
 import {WEIGHT_BOUNDS,localInput,refusal,withOffset,writeSummary} from '../garminWrite';
 // Profile's Add a weight (Rules: Garmin writes): the weight only, in Garmin's unit, at a time that
 // defaults to now. The time is fixed when the form opens, so trying again after an unconfirmed save
 // repeats the same weigh-in and the core finds the first attempt instead of adding a second.
 let {unit,onwrite}:{unit:WeightUnit|null,onwrite:()=>void}=$props();
 let open=$state(false),value=$state<number|null>(null),at=$state(''),busy=$state(false),result=$state<GarminChange|null>(null),error=$state('');
 let saved=$state('');
 function start(){at=localInput(new Date());value=null;result=null;error='';saved='';open=true}
 async function save(){
  const when=withOffset(at);
  if(!unit||value==null||!Number.isFinite(value)||!when){error='Enter a weight and a time';return}
  busy=true;error='';result=null;
  try{
   const change=await api.garmin_write({kind:'addWeight',value,unit,at:when});
   if(change.outcome==='applied'||change.outcome==='differs'){open=false;saved=writeSummary(change);onwrite()}
   else{result=change;if(change.outcome==='needsSignIn')onwrite()}
  }catch(e){error=refusal(e)}finally{busy=false}
 }
</script>
{#if !unit}
 <button class="na" disabled aria-describedby="weight-na"><Icon name="scale" size={16}/>Add a weight</button>
 <p class="small" id="weight-na">Waiting for Garmin’s measurement units.</p>
{:else if !open}
 <button class="secondary" onclick={start}><Icon name="scale" size={16}/>Add a weight</button>
 {#if saved}<p class="small" role="status">{saved}</p>{/if}
{:else}
 <form class="add" aria-label="Add a weight" onsubmit={e=>{e.preventDefault();void save()}}>
  <label>Weight<span class="field"><input type="number" inputmode="decimal" step="0.1" min={WEIGHT_BOUNDS[unit][0]} max={WEIGHT_BOUNDS[unit][1]} required bind:value disabled={busy}/><span>{unit}</span></span></label>
  <label>Time<input type="datetime-local" required bind:value={at} max={localInput(new Date())} disabled={busy}/></label>
  <p class="small">Saved in Garmin as a weigh-in you entered. Raring reads that day’s weigh-ins first and won’t add the same weight twice.</p>
  <div class="acts"><button class="primary" type="submit" disabled={busy}>{busy?'Saving to Garmin…':'Save to Garmin'}</button><button class="secondary" type="button" disabled={busy} onclick={()=>open=false}>Cancel</button></div>
  {#if error||result}<p class="small" role="status">{error||writeSummary(result!)}</p>{/if}
 </form>
{/if}
<style>
 .na{display:flex;gap:8px;align-items:center;justify-content:center;width:100%;margin-top:14px;min-height:44px;border-radius:12px;border:1px dashed var(--line);background:none;color:var(--text2);font:600 13px Inter;cursor:default}
 .add{display:grid;gap:12px;margin-top:14px;padding-top:14px;border-top:1px solid var(--line)}
 .add label{display:flex;justify-content:space-between;align-items:center;gap:12px;font-size:14px;color:var(--text2)}
 .add input{font:600 16px Inter;color:var(--text);background:var(--surface2);border:1px solid var(--line);border-radius:10px;padding:8px 10px;min-height:44px;min-width:0;color-scheme:dark}
 .field{display:flex;align-items:center;gap:6px;color:var(--text)}.field input{width:6.5em;text-align:right}
 .acts{display:flex;flex-wrap:wrap;gap:8px}.acts button{flex:1 1 120px;width:auto;margin-top:0}
 .add .small{margin:0}
</style>
