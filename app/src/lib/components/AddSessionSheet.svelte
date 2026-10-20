<script lang="ts">
 import {untrack} from 'svelte';
 import Icon from './Icon.svelte';
 import type {PlanBoard} from '../generated/PlanBoard';
 import type {LocalAction} from '../generated/LocalAction';
 import type {Sport} from '../generated/Sport';
 import type {Allocation} from '../generated/Allocation';
 import {GOALS, SPORT_NAMES} from '../goals';
 import {sheetOpened} from '../sheet';
 import {dateLabel} from '../format';
 // One short sheet: a sport and a day; time, length and what it counts toward
 // are optional. Zone minutes are what the person intends, never inferred.
 export type Draft = {date?: string, sport?: Sport, allocation?: Partial<Allocation>, name?: string};
 let {board, draft = {}, close, act, busy = false}: {board: PlanBoard, draft?: Draft, close: () => void, act: (a: LocalAction) => void, busy?: boolean} = $props();
 const SPORTS: Sport[] = ['running', 'cycling', 'walking', 'hiking', 'swimming', 'strength'];
 let sport = $state<Sport>(untrack(() => draft.sport ?? 'strength'));
 let date = $state(untrack(() => draft.date ?? board.today));
 let time = $state('');
 let minutes = $state<number | null>(null);
 let alloc = $state<Allocation>(untrack(() => ({strengthSessions: 0, zone2Min: 0, highIntensityMin: 0, intensityMin: 0, sleepNights: 0, ...draft.allocation})));
 // Only this plan week's remaining days: Plan shows one week, so a session
 // placed later would vanish from it until that week starts.
 let days = $derived(Array.from({length: 7}, (_, i) => { const d = new Date(board.today + 'T12:00:00Z'); d.setUTCDate(d.getUTCDate() + i); return d.toISOString().slice(0, 10); }).filter(d => d <= board.end));
 let name = $state(untrack(() => draft.name ?? ''));
 type Planned = 'strength' | 'zone2' | 'highIntensity' | 'intensity';
 let goalKinds = $derived(board.ledger.map(l => l.kind).filter((k): k is Planned => ['strength', 'zone2', 'highIntensity', 'intensity'].includes(k)));
 $effect(() => sheetOpened(close));
 $effect(() => { if (sport === 'strength' && goalKinds.includes('strength') && !alloc.strengthSessions) alloc.strengthSessions = 1; if (sport !== 'strength') alloc.strengthSessions = 0; });
 const FIELD = {strength: 'strengthSessions', zone2: 'zone2Min', highIntensity: 'highIntensityMin', intensity: 'intensityMin'} as const;
 function save() {
  act({action: 'addIntention', date, time: time || null, sport, typeKey: null, name: name.trim() || null, durationMin: minutes, allocation: {...alloc}, kind: 'workout', garminId: null});
  close();
 }
</script>
<svelte:window onkeydown={e => { if (e.key === 'Escape') close(); }}/>
<button class="scrim" aria-label="Close" onclick={close}></button>
<div class="sheet" role="dialog" aria-modal="true" aria-labelledby="add-title">
 <span class="grab"></span>
 <div class="sheet-h"><h2 id="add-title">Add to my plan</h2><button class="close" aria-label="Close" onclick={close}><Icon name="close" size={20}/></button></div>
 <div class="chips2" role="group" aria-label="Sport">{#each SPORTS as s}<button aria-pressed={sport === s} onclick={() => sport = s}>{SPORT_NAMES[s]}</button>{/each}</div>
 <div class="chips2" role="group" aria-label="Day">{#each days as d}<button aria-pressed={date === d} onclick={() => date = d}>{d === board.today ? 'Today' : dateLabel(d)}</button>{/each}</div>
 <div class="row2"><label class="field"><span>Time <small>optional</small></span><input type="time" bind:value={time} aria-label="Time"/></label>
  <label class="field"><span>Minutes <small>optional</small></span><input type="number" min="1" max="600" bind:value={minutes} aria-label="Planned minutes"/></label></div>
 <label class="field name"><span>Name <small>optional</small></span><input maxlength="60" bind:value={name} placeholder={SPORT_NAMES[sport]} aria-label="Session name"/></label>
 {#if goalKinds.length}
  <p class="small">Counts toward</p>
  {#each goalKinds as k}
   {#if k === 'strength'}{#if sport === 'strength'}<p class="alloc"><i style:--c={GOALS.strength.color}></i>One strength session</p>{/if}
   {:else}<label class="alloc"><i style:--c={GOALS[k].color}></i>{GOALS[k].name} <input type="number" min="0" max="600" step="5" bind:value={alloc[FIELD[k]]} aria-label={`${GOALS[k].name} minutes you intend`}/> min</label>{/if}
  {/each}
 {/if}
 <button class="primary" disabled={busy} onclick={save}>Place {date === board.today ? 'today' : `on ${dateLabel(date)}`}</button>
</div>
<style>
 .scrim{position:fixed;inset:0;border:0;background:rgba(5,8,14,.62);z-index:20;padding:0}
 .sheet{position:fixed;left:0;right:0;bottom:0;z-index:21;max-width:560px;margin:0 auto;background:var(--card);border:1px solid var(--line);border-bottom:0;border-radius:26px 26px 0 0;padding:12px 20px 28px;max-height:90vh;overflow:auto}
 .grab{display:block;width:40px;height:4px;border-radius:2px;background:rgba(255,255,255,.18);margin:0 auto 16px}
 .sheet-h{display:flex;justify-content:space-between;align-items:center;gap:12px}
 .close{width:44px;height:44px;border-radius:50%;border:1px solid var(--line);background:none;display:grid;place-items:center;color:var(--text2)}
 .chips2{display:flex;flex-wrap:wrap;gap:8px;margin:12px 0 0}.chips2 button{min-height:40px;padding:8px 12px;border-radius:999px;border:1px solid var(--line);background:none;color:var(--text2);font:600 13px Inter}.chips2 button[aria-pressed=true]{background:var(--ink);color:var(--bg)}
 .row2{display:grid;grid-template-columns:minmax(0,1fr) minmax(0,1fr);gap:10px;margin-top:12px}.name{margin-top:12px}
 .field{display:grid;gap:6px;font-size:13px;color:var(--text2)}.field small{color:var(--text3);font-size:12px}.field input{width:100%;box-sizing:border-box}.field input,.alloc input{background:rgba(255,255,255,.06);border:1px solid var(--line);border-radius:10px;color:var(--text);padding:9px 10px;font:500 14px Inter;min-width:0}
 .alloc{display:flex;align-items:center;gap:8px;margin:8px 0 0;font-size:14px}.alloc input{width:80px}.alloc i{width:9px;height:9px;border-radius:50%;background:var(--c)}
</style>
