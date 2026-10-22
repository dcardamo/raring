<script lang="ts">
 import Icon from './Icon.svelte';
 import * as api from '../api';
 import type {WeightUnit} from '../generated/WeightUnit';
 import type {GarminChange} from '../generated/GarminChange';
 import {BODY_FAT_BOUNDS,WEIGHT_BOUNDS,localInput,refusal,withOffset,writeSummary} from '../garminWrite';
 // Profile's Add a weight (Rules: Garmin writes): the weight in kg or lb, the scale's body fat if
 // it gives one, at a time that defaults to now. It opens in the unit last submitted (the core keeps
 // it); either can be typed regardless of Garmin's setting. The time is fixed when the form opens,
 // so trying again after an unconfirmed save repeats the same weigh-in and the core finds the first
 // attempt instead of adding a second.
 let {unit,onwrite}:{unit:WeightUnit,onwrite:()=>void}=$props();
 let open=$state(false),entry=$state<WeightUnit>('kg'),value=$state<number|null>(null),fat=$state<number|null>(null),at=$state(''),busy=$state(false),result=$state<GarminChange|null>(null),error=$state('');
 let saved=$state('');
 function start(){entry=unit;at=localInput(new Date());value=null;fat=null;result=null;error='';saved='';open=true}
 // What Garmin now has, named: "Saved in Garmin: 162.7 lb, 20.4 % body fat at 07:58 on 21 Oct."
 const savedLine=(change:GarminChange)=>{const what=change.fields[0]?.after;const line=writeSummary(change);return what&&change.outcome==='applied'?line.replace(/^Saved in Garmin\./,`Saved in Garmin: ${what}.`):line};
 async function save(){
  const when=withOffset(at);
  if(value==null||!Number.isFinite(value)||!when){error='Enter a weight and a time';return}
  const bodyFat=fat==null||Number.isNaN(fat)?null:fat;
  busy=true;error='';result=null;
  try{
   const change=await api.garmin_write({kind:'addWeight',value,unit:entry,at:when,bodyFat});
   if(change.outcome==='applied'||change.outcome==='differs'){open=false;saved=savedLine(change);onwrite()}
   else{result=change;if(change.outcome==='needsSignIn')onwrite()}
  }catch(e){error=refusal(e)}finally{busy=false}
 }
</script>
{#if !open}
 <button class="secondary" onclick={start}><Icon name="scale" size={16}/>Add a weight</button>
 {#if saved}<p class="small" role="status">{saved}</p>{/if}
{:else}
 <form class="add" aria-label="Add a weight" onsubmit={e=>{e.preventDefault();void save()}}>
  <div class="row"><span id="weight-label">Weight</span><span class="field"><input type="number" inputmode="decimal" step="0.1" min={WEIGHT_BOUNDS[entry][0]} max={WEIGHT_BOUNDS[entry][1]} required bind:value disabled={busy} aria-labelledby="weight-label"/>
   <span class="units" role="group" aria-label="Weight unit">{#each ['kg','lb'] as const as u}<button type="button" aria-pressed={entry===u} disabled={busy} onclick={()=>entry=u}>{u}</button>{/each}</span></span></div>
  <label><span>Body fat<small>optional</small></span><span class="field"><input type="number" inputmode="decimal" step="0.1" min={BODY_FAT_BOUNDS[0]} max={BODY_FAT_BOUNDS[1]} bind:value={fat} disabled={busy}/><span class="suffix">%</span></span></label>
  <label>Time<input type="datetime-local" required bind:value={at} max={localInput(new Date())} disabled={busy}/></label>
  <p class="small">Saved in Garmin as a weigh-in you entered, with body fat if you give it. Raring reads that day’s weigh-ins first and won’t add the same reading twice.</p>
  <div class="acts"><button class="primary" type="submit" disabled={busy}>{busy?'Saving to Garmin…':'Save to Garmin'}</button><button class="secondary" type="button" disabled={busy} onclick={()=>open=false}>Cancel</button></div>
  {#if error||result}<p class="small" role="status">{error||writeSummary(result!)}</p>{/if}
 </form>
{/if}
<style>
 .add{display:grid;gap:12px;margin-top:14px;padding-top:14px;border-top:1px solid var(--line)}
 .add label,.row{display:flex;justify-content:space-between;align-items:center;gap:12px;font-size:14px;color:var(--text2)}
 .add label small{font:500 11px Inter;color:var(--text3);margin-left:4px}
 .add input{font:600 16px Inter;color:var(--text);background:var(--surface2);border:1px solid var(--line);border-radius:10px;padding:8px 10px;min-height:44px;min-width:0;color-scheme:dark}
 .field{display:flex;align-items:center;gap:6px;color:var(--text)}.field input{width:6.5em;text-align:right}
 .suffix{min-width:2.2em}
 .units{display:flex;border:1px solid var(--line);border-radius:10px;overflow:hidden}
 .units button{min-width:44px;min-height:44px;border:0;background:none;color:var(--text2);font:600 14px Inter}
 .units button[aria-pressed=true]{background:var(--ink);color:var(--bg)}
 .acts{display:flex;flex-wrap:wrap;gap:8px}.acts button{flex:1 1 120px;width:auto;margin-top:0}
 .add .small{margin:0}
</style>
