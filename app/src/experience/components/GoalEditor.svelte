<script lang="ts">
 import {untrack} from 'svelte';
 import Icon, {type IconName} from '../../lib/components/Icon.svelte';
 import {store} from '../lib/store.svelte';
 import {goalsFor, GOAL_INFO, type GoalKey} from '../lib/allocation';
 import {A} from '../lib/actions';
 import {addDays, dur, hm, toMin} from '../lib/dates';
 import type {GoalTargets} from '../lib/rules';
 // Weekly goals with steppers. Zone goals count only Garmin's recorded zone
 // time; Sleep Score is Garmin's reading shown against a target, never earned.
 let {done = () => store.closeSheet(), initial, save, label = 'Save weekly goals'}: {done?: () => void, initial?: GoalTargets, save?: (t: GoalTargets) => void, label?: string} = $props();
 let target = $state(untrack(() => ({...(initial ?? goalsFor(store.ctx))})));
 const STEP: Record<GoalKey | 'sleepScore', [number, number]> = {strength: [1, 7], zone2: [10, 600], hi: [5, 300], im: [10, 600], sleep: [1, 7], sleepScore: [5, 95]};
 const ROWS: {key: GoalKey | 'sleepScore', name: string, sub: string, icon: IconName, color: string}[] = [
  {key: 'strength', name: 'Strength', sub: 'sessions a week', icon: 'strength', color: GOAL_INFO.strength.color},
  {key: 'zone2', name: 'Zone 2', sub: 'min a week · recorded heart-rate zone time', icon: 'pulse', color: GOAL_INFO.zone2.color},
  {key: 'hi', name: 'Zone 4–5', sub: 'min a week · hard efforts, recorded zone time', icon: 'flame', color: GOAL_INFO.hi.color},
  {key: 'im', name: 'Intensity minutes', sub: 'min a week · Garmin’s count', icon: 'flame', color: GOAL_INFO.im.color},
  {key: 'sleep', name: 'Sleep timing', sub: 'nights a week · your action', icon: 'moon', color: GOAL_INFO.sleep.color},
  {key: 'sleepScore', name: 'Average Sleep Score', sub: 'a Garmin reading against your target', icon: 'moon', color: '#A9C8FF'},
 ];
 const step = (k: GoalKey | 'sleepScore', dir: 1 | -1) => {
  const [s, max] = STEP[k], v = target[k] ?? 0;
  target[k] = k === 'sleepScore' && v === 0 && dir > 0 ? 75 : Math.max(0, Math.min(max, v + dir * s));
  if (k === 'sleepScore' && target[k] < 50 && dir < 0) target[k] = 0;
 };
 let t = $derived(store.ctx.tonight);
 let recent = $derived(Array.from({length: 7}, (_, i) => store.ctx.nightOf(addDays(store.ctx.displayedDay, -i))).filter(n => n != null));
 let below = $derived(recent.filter(n => n!.asleep < n!.need).length);
 let suggested = $derived(hm((toMin(t.wake) - t.need + 1440) % 1440));
</script>
<div class="eg-list">
 {#each ROWS as r}
  <div class="eg" style:--c={r.color} class:off={!target[r.key]}>
   <span class="eg-i"><Icon name={r.icon} size={19}/></span>
   <span class="eg-t"><b>{r.name}</b><small>{r.sub}</small></span>
   <span class="stepper" role="group" aria-label={`${r.name} weekly target`}>
    <button aria-label={`Less ${r.name}`} disabled={!target[r.key]} onclick={() => step(r.key, -1)}>−</button>
    <output class="n" aria-live="polite">{target[r.key] || 'Off'}</output>
    <button aria-label={`More ${r.name}`} onclick={() => step(r.key, 1)}>+</button>
   </span>
  </div>
 {/each}
</div>
{#if target.sleep > 0}
 <div class="sleep-kind">
  <div class="seg2" role="group" aria-label="Sleep timing kind"><button aria-pressed={target.sleepKind === 'bedtime'} onclick={() => target.sleepKind = 'bedtime'}>Asleep by</button><button aria-pressed={target.sleepKind === 'winddown'} onclick={() => target.sleepKind = 'winddown'}>Wind down at</button></div>
  <label class="field"><span>Time</span><input type="time" aria-label="Sleep timing time" bind:value={target.sleepTime}/></label>
  {#if recent.length}<p class="small">Last {recent.length} nights: {below} below Garmin’s sleep need.</p>{/if}
  <button class="link" onclick={() => target.sleepTime = suggested}>Use {suggested}: Garmin’s need {dur(t.need)} before your usual {t.wake} wake</button>
 </div>
{/if}
<button class="primary" onclick={() => { const v = {...target}; if (v.sleep > 0 && !v.sleepTime) return; if (save) save(v); else A.saveGoals(v); done(); }}>{label}</button>
<style>
 .eg-list{display:grid;gap:7px;margin-top:6px}
 .eg{display:grid;grid-template-columns:34px 1fr auto;gap:10px;align-items:center;padding:9px 10px;border-radius:15px;background:var(--card);border:1px solid var(--line)}
 .eg.off{opacity:.72}.eg-i{display:grid;place-items:center;width:34px;height:34px;border-radius:11px;background:color-mix(in srgb,var(--c) 14%,transparent);color:var(--c)}
 .eg-t{min-width:0}.eg b{display:block;font:600 15px Outfit}.eg small{display:block;font-size:11px;color:var(--text2)}
 .stepper{display:flex;align-items:center;gap:4px}.stepper button{width:36px;height:36px;border-radius:11px;border:1px solid var(--line);background:rgba(255,255,255,.06);color:var(--text);font:600 19px Inter}.stepper button:disabled{opacity:.35}
 .stepper output{min-width:42px;text-align:center;font-size:18px;font-weight:600;color:var(--c)}.off .stepper output{font:500 12px Inter;color:var(--text3)}
 .sleep-kind{margin-top:10px;padding:10px 12px;border:1px solid var(--line);border-radius:15px}.seg2{display:flex;gap:6px}.seg2 button{flex:1;min-height:36px;border-radius:10px;border:1px solid var(--line);background:none;color:var(--text2);font:600 12px Inter}.seg2 button[aria-pressed=true]{background:var(--ink);color:var(--bg)}
 .sleep-kind .link{font-size:12px;text-align:left;margin-top:4px}
 @container(max-width:300px){.eg{grid-template-columns:1fr auto}.eg-i{display:none}}
</style>
