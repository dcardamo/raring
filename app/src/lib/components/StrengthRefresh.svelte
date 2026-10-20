<script lang="ts">
 import {dateLabel,clock} from '../format';
 import type {StrengthSession} from '../generated/StrengthSession';
 // One explicit Garmin read, only when someone asks. Opening a workout never refreshes.
 let {checkedAt,refresh}:{checkedAt:string|null,refresh:()=>Promise<StrengthSession|null>}=$props();
 let busy=$state(false),note=$state('');
 const label=$derived(checkedAt?`Sets checked ${dateLabel(checkedAt)} · ${clock(checkedAt)}`:'Sets from Garmin not checked yet');
 async function run(){
  if(busy)return;
  busy=true;note='';
  try{
   const session=await refresh();
   if(!session)note='Garmin has no sets for this workout.';
  }catch(e){note=String(e).replace(/^Error:\s*/,'')}
  finally{busy=false}
 }
</script>
<div class="sets-refresh">
 <span class="small">{label}</span>
 <button class="secondary" disabled={busy} onclick={run}>{busy?'Checking Garmin…':'Refresh from Garmin'}</button>
</div>
{#if note}<p class="small" role="status">{note}</p>{/if}
<style>
 .sets-refresh{display:flex;flex-direction:column;gap:8px;align-items:flex-start}
 .sets-refresh .small{margin:0}
 .sets-refresh button{min-height:40px}
</style>
