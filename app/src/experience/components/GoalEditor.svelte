<script lang="ts">
 import {untrack} from 'svelte';
 import Icon from '../../lib/components/Icon.svelte';
 import {store} from '../lib/store.svelte';
 import {goalsFor, GOAL_INFO, type GoalKey} from '../lib/allocation';
 import {A} from '../lib/actions';
 let {done=()=>store.closeSheet()}:{done?:()=>void}=$props();
 let target=$state(untrack(()=>({...goalsFor(store.ctx)})));
 const keys:GoalKey[]=['strength','zone2','im','sleep'];
</script>
<p class="small">Choose what matters this week. Zero turns a goal off.</p>
<div class="edit-goals">
 {#each keys as key}<label class="eg" style:--c={GOAL_INFO[key].color}>
  <span class="eg-i"><Icon name={key==='strength'?'strength':key==='zone2'?'pulse':key==='im'?'flame':'moon'} size={21}/></span>
  <span><b>{GOAL_INFO[key].name}</b><small>{GOAL_INFO[key].unit} per week</small></span>
  <input aria-label={`${GOAL_INFO[key].name} weekly target`} type="number" min="0" max={key==='strength'||key==='sleep'?7:1200} step={key==='strength'||key==='sleep'?1:5} bind:value={target[key]}/>
 </label>{/each}
</div>
{#if target.sleep>0}<label class="field"><span>Sleep timing</span><select aria-label="Sleep timing kind" bind:value={target.sleepKind}><option value="winddown">Wind-down</option><option value="bedtime">Bedtime</option></select></label><label class="field"><span>Chosen time</span><input type="time" bind:value={target.sleepTime}/></label>{/if}
<button class="primary" onclick={()=>{if(keys.some(k=>!Number.isFinite(target[k])||target[k]<0)||(target.sleep>0&&!target.sleepTime))return; A.saveGoals({...target});done();}}>Save weekly goals</button>
<p class="small">From today</p>
<style>.edit-goals{margin-top:12px;display:grid;gap:8px}.eg{display:grid;grid-template-columns:38px 1fr 84px;gap:10px;align-items:center;padding:12px;border-radius:16px;background:var(--card);border:1px solid var(--line)}.eg-i{display:grid;place-items:center;width:38px;height:38px;border-radius:12px;background:color-mix(in srgb,var(--c) 14%,transparent);color:var(--c)}.eg b{display:block;font:600 16px Outfit}.eg small{display:block;font-size:12px;color:var(--text2)}.eg input{width:84px;border:1px solid var(--line);background:var(--bg);color:var(--text);font:600 22px Outfit;border-radius:10px;padding:8px;text-align:center}</style>
