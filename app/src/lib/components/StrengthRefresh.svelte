<script lang="ts">
 import {localDateTime} from '../format';
 import type {StrengthSession} from '../generated/StrengthSession';
 // One explicit Garmin read, only when someone asks. Opening a workout never refreshes.
 let {checkedAt,refresh}:{checkedAt:string|null,refresh:()=>Promise<StrengthSession|null>}=$props();
 let busy=$state(false),note=$state('');
 const label=$derived(checkedAt?`Sets checked ${localDateTime(checkedAt)}`:'Sets from Garmin not checked yet');
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
 <button class="link" disabled={busy} onclick={run}>{busy?'Checking Garmin…':'Refresh from Garmin'}</button>
</div>
{#if note}<p class="small" role="status">{note}</p>{/if}
<style>
 .sets-refresh{display:flex;flex-wrap:wrap;gap:0 12px;align-items:center}
 .sets-refresh .small{margin:0}
 .sets-refresh button{min-height:44px;padding:0;border:0;background:none;color:var(--ink);font:600 13px Inter;text-decoration:underline;text-underline-offset:3px}
</style>
