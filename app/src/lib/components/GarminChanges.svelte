<script lang="ts">
 import Card from './Card.svelte';
 import type {GarminChange} from '../generated/GarminChange';
 import {RESULT} from '../garminWrite';
 import {clock} from '../format';
 // Settings' log of Raring's writes to Garmin (Rules: Garmin writes); read-only.
 let {changes,failed=false}:{changes:GarminChange[]|null,failed?:boolean}=$props();
 const when=(at:string)=>{const d=new Date(at);return Number.isNaN(d.getTime())?'Time unavailable':`${d.toLocaleDateString('en-GB',{weekday:'short',day:'numeric',month:'short'})} ${clock(at)}`};
 const value=(v:string|null)=>v??'none';
</script>
<Card title="Recent Garmin changes">
 {#if failed}<p class="small" role="status">Couldn’t load recent changes.</p>
 {:else if changes===null}<p class="small">Loading…</p>
 {:else if changes.length===0}<p class="small">Raring hasn’t changed anything in Garmin. Each change you make here will be listed with Garmin’s answer.</p>
 {:else}<ul class="changes" aria-label="Recent Garmin changes">{#each changes as change,i (change.at+i)}<li>
  <div class="head"><b>{change.resource}</b><small>{when(change.at)}</small></div>
  {#each change.fields as f (f.field)}<div class="small">{f.label}: {#if f.after===null}{value(f.before)}{:else}{value(f.before)} → {f.after}{/if}</div>{/each}
  <div class="result {change.outcome}">{RESULT[change.outcome]+(change.waitingForWatch?' · Waiting for watch sync':'')}</div>
  {#if change.message}<div class="small">{change.message}</div>{/if}
  {#each change.mismatches as m (m.field)}<div class="small">{#if m.before===null && m.after===null}{m.label} came back different{:else}{m.label}: sent {value(m.before)}, Garmin has {value(m.after)}{/if}</div>{/each}
 </li>{/each}</ul>{/if}
</Card>
<style>
 .changes{list-style:none;margin:0;padding:0;display:grid;gap:12px}
 .changes li{display:grid;gap:2px}
 .head{display:flex;justify-content:space-between;gap:8px;align-items:baseline}
 .head small{font:500 12px Inter;color:var(--text2);white-space:nowrap}
 .changes .small{margin:0}
 .result{font:600 13px Inter;color:var(--text)}
 .result.differs,.result.unconfirmed,.result.conflict,.result.duplicate{color:var(--fair)}
 .result.rejected,.result.failed,.result.needsSignIn{color:var(--poor)}
</style>
