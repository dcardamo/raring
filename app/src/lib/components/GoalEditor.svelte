<script lang="ts">
 import {untrack} from 'svelte';
 import Icon from './Icon.svelte';
 import type {GoalTargets} from '../generated/GoalTargets';
 import {GOALS, noGoals} from '../goals';
 // Weekly goals with steppers. Zone goals count only Garmin's recorded zone
 // time; Sleep Score is Garmin's reading shown against a target, never earned.
 let {initial, garminGoal, save, busy = false}: {initial: GoalTargets | null, garminGoal: number | null, save: (t: GoalTargets) => void, busy?: boolean} = $props();
 let t = $state<GoalTargets>(untrack(() => structuredClone($state.snapshot(initial) ?? noGoals())));
 type Key = 'strengthSessions' | 'zone2Min' | 'highIntensityMin' | 'intensityMin' | 'sleepNights' | 'sleepScore';
 const ROWS: {key: Key, kind: keyof typeof GOALS, sub: string, step: number, max: number}[] = [
  {key: 'strengthSessions', kind: 'strength', sub: 'sessions a week', step: 1, max: 14},
  {key: 'zone2Min', kind: 'zone2', sub: 'min a week · recorded heart-rate zone time', step: 10, max: 600},
  {key: 'highIntensityMin', kind: 'highIntensity', sub: 'min a week · recorded zones 4 and 5', step: 5, max: 300},
  {key: 'intensityMin', kind: 'intensity', sub: 'min a week · Garmin’s intensity minutes', step: 10, max: 600},
  {key: 'sleepNights', kind: 'sleepTiming', sub: 'nights a week', step: 1, max: 7},
  {key: 'sleepScore', kind: 'sleepScore', sub: 'average · Garmin’s reading against your target', step: 5, max: 95},
 ];
 let observed = $derived(t.sleepKind === 'savedPlan');
 const value = (k: Key) => k === 'intensityMin' && t.intensityGarminGoal ? (garminGoal ?? 0) : t[k];
 const on = (k: Key) => k === 'intensityMin' ? t.intensityGarminGoal || t.intensityMin > 0 : t[k] > 0;
 function step(r: typeof ROWS[number], dir: 1 | -1) {
  if (r.key === 'intensityMin' && t.intensityGarminGoal) { t.intensityGarminGoal = false; t.intensityMin = garminGoal ?? 150; }
  let v = Math.max(0, Math.min(r.max, t[r.key] + dir * r.step));
  if (r.key === 'sleepScore') v = dir > 0 && t.sleepScore === 0 ? 75 : v < 50 ? 0 : v;
  t[r.key] = v;
  if (r.key === 'sleepNights' && v > 0 && observed) t.sleepKind = 'savedPlan';
  if (r.key === 'sleepNights' && v > 0 && !observed && !t.sleepTime) t.sleepTime = '22:30';
 }
 // Turning a migrated observation off is final; a new sleep goal is chosen afresh.
 function endObservation() { t.sleepNights = 0; t.sleepKind = 'asleepBy'; }
</script>
<div class="eg-list">
 {#each ROWS as r}
  {@const g = GOALS[r.kind]}
  <div class="eg" style:--c={g.color} class:off={!on(r.key)}>
   <span class="eg-i"><Icon name={g.icon} size={19}/></span>
   <span class="eg-t"><b>{r.kind === 'intensity' ? 'Intensity minutes' : r.kind === 'sleepScore' ? 'Average Sleep Score' : g.name}</b><small>{r.key === 'intensityMin' && t.intensityGarminGoal ? 'Garmin’s weekly goal' : r.sub}</small></span>
   {#if r.key === 'sleepNights' && observed}
    <span class="obs"><output class="n">{t.sleepNights}</output><button class="link" onclick={endObservation}>Turn off</button></span>
   {:else}
    <span class="stepper" role="group" aria-label={`${g.name} weekly target`}>
     <button aria-label={`Less ${g.name}`} disabled={!on(r.key)} onclick={() => step(r, -1)}>−</button>
     <output class="n" aria-live="polite">{on(r.key) ? value(r.key) : 'Off'}</output>
     <button aria-label={`More ${g.name}`} onclick={() => step(r, 1)}>+</button>
    </span>
   {/if}
  </div>
 {/each}
</div>
{#if observed && t.sleepNights > 0}
 <p class="small note">Sleep timing from your earlier plan is observed against each night's saved plan. Earlier weeks keep their results.</p>
{:else if t.sleepNights > 0}
 <div class="sleep-kind">
  <div class="seg2" role="group" aria-label="Sleep timing kind"><button aria-pressed={t.sleepKind === 'asleepBy'} onclick={() => t.sleepKind = 'asleepBy'}>Asleep by</button><button aria-pressed={t.sleepKind === 'windDown'} onclick={() => t.sleepKind = 'windDown'}>Wind down at</button></div>
  <label class="field"><span>Time</span><input type="time" aria-label="Sleep timing time" bind:value={t.sleepTime}/></label>
  <p class="small">{t.sleepKind === 'asleepBy' ? 'Counts nights Garmin records you asleep by this time.' : 'Counts evenings you report winding down.'}</p>
 </div>
{/if}
<button class="primary" disabled={busy || t.sleepNights > 0 && !observed && !t.sleepTime} onclick={() => save($state.snapshot(t))}>Save weekly goals</button>
<p class="small">Changes apply from today; this week keeps its starting targets for reference.</p>
<style>
 .eg-list{display:grid;gap:7px;margin-top:6px}
 .eg{display:grid;grid-template-columns:34px 1fr auto;gap:10px;align-items:center;padding:9px 10px;border-radius:15px;background:var(--card);border:1px solid var(--line)}
 .eg.off .eg-i{opacity:.6}.eg-i{display:grid;place-items:center;width:34px;height:34px;border-radius:11px;background:color-mix(in srgb,var(--c) 14%,transparent);color:var(--c)}
 .eg-t{min-width:0}.eg b{display:block;font:600 15px Outfit}.eg small{display:block;font-size:11px;color:var(--text2)}
 .stepper,.obs{display:flex;align-items:center;gap:4px}.stepper button{width:36px;height:36px;border-radius:11px;border:1px solid var(--line);background:rgba(255,255,255,.06);color:var(--text);font:600 19px Inter}.stepper button:disabled{color:var(--text3);background:none}
 .stepper output,.obs output{min-width:42px;text-align:center;font-size:18px;font-weight:600;color:var(--c)}.off .stepper output{font:500 12px Inter;color:var(--text2)}
 .link{background:none;border:0;color:var(--ink);font:600 12px Inter;min-height:36px}
 .sleep-kind{margin-top:10px;padding:10px 12px;border:1px solid var(--line);border-radius:15px}.seg2{display:flex;gap:6px}.seg2 button{flex:1;min-height:36px;border-radius:10px;border:1px solid var(--line);background:none;color:var(--text2);font:600 12px Inter}.seg2 button[aria-pressed=true]{background:var(--ink);color:var(--bg)}
 .field{display:flex;justify-content:space-between;align-items:center;margin-top:10px;font-size:13px;color:var(--text2)}.field input{background:rgba(255,255,255,.06);border:1px solid var(--line);border-radius:10px;color:var(--text);padding:8px 10px;font:500 14px Inter}
 .note{margin:10px 2px 0}
 @container(max-width:300px){.eg{grid-template-columns:1fr auto}.eg-i{display:none}}
</style>
