<script lang="ts">
 import {untrack} from 'svelte';
 import Icon from './Icon.svelte';
 import type {AdaptState} from '../generated/AdaptState';
 import type {LocalAction} from '../generated/LocalAction';
 import type {Sport} from '../generated/Sport';
 import {SPORT_NAMES} from '../goals';
 import {sheetOpened} from '../sheet';
 import {dateLabel} from '../format';
 // Rest, illness, a pause and limits: what to change, never why. Each holds until
 // the person changes it; nothing resumes on a date or a good reading.
 // Resuming shows the goals as they are first: nothing is raised, lowered or caught up.
 let {current, close, act, busy = false, startLimits = false, goals = [], changeGoals, garminWorkouts = []}: {garminWorkouts?: {name: string, source: string}[], current: AdaptState | null, close: () => void, act: (a: LocalAction) => void, busy?: boolean, startLimits?: boolean, goals?: string[], changeGoals?: () => void} = $props();
 let resuming = $state(false);
 let limits = $state(false);
 let routines = $state(false);
 let routineName = $state(''), routineHow = $state('');
 let excluded = $state<Sport[]>([]);
 let note = $state('');
 const SPORTS: Sport[] = ['running', 'cycling', 'walking', 'hiking', 'swimming', 'strength'];
 $effect(() => sheetOpened(close));
 function editLimits() { excluded = [...(current?.limits.excluded ?? [])]; note = current?.limits.note ?? ''; limits = true; }
 // Profile opens straight into limits; only the value at opening matters.
 if (untrack(() => startLimits)) editLimits();
 const toggle = (s: Sport) => excluded = excluded.includes(s) ? excluded.filter(x => x !== s) : [...excluded, s];
 const run = (a: LocalAction) => { act(a); close(); };
</script>
<svelte:window onkeydown={e => { if (e.key === 'Escape') close(); }}/>
<button class="scrim" aria-label="Close" onclick={close}></button>
<div class="sheet" role="dialog" aria-modal="true" aria-labelledby="adapt-title">
 <span class="grab"></span>
 <div class="sheet-h"><h2 id="adapt-title">{limits ? 'Your limits' : routines ? 'My routines' : resuming ? 'Resume training' : 'Adapt today'}</h2><button class="close" aria-label="Close" onclick={close}><Icon name="close" size={20}/></button></div>
 {#if limits}
  <p class="small">Activities to avoid are left out of suggestions. A movement note is shown wherever a session’s contents are unknown; Raring doesn’t check workouts against it.</p>
  <div class="chips2" role="group" aria-label="Activities to avoid">{#each SPORTS as s}<button aria-pressed={excluded.includes(s)} onclick={() => toggle(s)}>{SPORT_NAMES[s]}</button>{/each}</div>
  <label class="note"><span>Movement note</span><textarea rows="3" maxlength="500" bind:value={note} placeholder="For example: no overhead pressing"></textarea></label>
  <button class="primary" disabled={busy} onclick={() => run({action: 'setLimits', limits: {excluded, note}})}>Save limits</button>
 {:else if resuming}
  <p class="small">Your goals stay as they were. Nothing is added to catch up.</p>
  {#if goals.length}<ul class="goals">{#each goals as g}<li>{g}</li>{/each}</ul>{:else}<p class="small">No weekly goals set.</p>{/if}
  <div class="opts">
   <button disabled={busy} onclick={() => run({action: 'resume'})}><Icon name="check" size={20}/><span><b>Resume as they are</b><small>Prompts and suggestions return</small></span></button>
   {#if changeGoals}<button disabled={busy} onclick={() => { act({action: 'resume'}); close(); changeGoals(); }}><Icon name="pencil" size={20}/><span><b>Resume and change goals</b><small>From this week on; earlier weeks keep theirs</small></span></button>{/if}
  </div>
 {:else if routines}
  {#if garminWorkouts.length}<h3 class="gh">Garmin workouts this week</h3>{#each garminWorkouts as g}<div class="routine"><span><b>◇ {g.name}</b><small>{g.source} · Contents unknown to Raring</small></span><span class="st">{current?.limits.note || current?.limits.excluded.length ? 'Review needed' : 'No limits set'}</span></div>{/each}{/if}
  <h3 class="gh">Your checklist</h3>
  <p class="small">A checklist you or your clinician supply. Raring adds no exercises, load or progression.</p>
  {#each current?.routines ?? [] as r (r.item.id)}
   <div class="routine"><span><b>{r.item.name}</b>{#if r.item.how}<small>{r.item.how}</small>{/if}</span>
    <button class="secondary" aria-pressed={r.done} disabled={busy} onclick={() => act({action: 'reportRoutine', id: r.item.id, done: !r.done})}>{#if r.done}<Icon name="check" size={14}/>Done today · self-reported{:else}Mark done today{/if}</button>
    <button class="remove" aria-label={`Remove ${r.item.name}`} disabled={busy} onclick={() => act({action: 'removeRoutine', id: r.item.id})}><Icon name="close" size={16}/></button></div>
  {:else}<p class="small">Nothing on your checklist yet.</p>{/each}
  <label class="note"><span>Exercise</span><input maxlength="80" bind:value={routineName} placeholder="For example: wall sit, 3 × 30 s"/></label>
  <label class="note"><span>How (optional)</span><input maxlength="300" bind:value={routineHow} placeholder="For example: from my physio"/></label>
  <button class="primary" disabled={busy || !routineName.trim()} onclick={() => { act({action: 'addRoutine', name: routineName, how: routineHow.trim() || null}); routineName = ''; routineHow = ''; }}>Add to checklist</button>
  <p class="small">Self-reported: no Garmin activity, intensity minutes or records are created, and nothing is sent to Garmin.</p>
 {:else}
  <div class="opts">
   {#if current?.rest}<button onclick={() => run({action: 'cancelRest'})}><Icon name="moon" size={20}/><span><b>Rest day is on</b><small>Turn it off · prompts return today</small></span></button>
   {:else}<button disabled={busy || !!current?.pause} onclick={() => run({action: 'restToday'})}><Icon name="moon" size={20}/><span><b>Rest today</b><small>No exercise prompts today or tonight</small></span></button>{/if}
   {#if current?.pause}<button onclick={() => resuming = true}><Icon name="pause" size={20}/><span><b>Resume training</b><small>{current.pause.reason === 'ill' ? 'Ill' : 'Paused'} since {dateLabel(current.pause.since)} · your goals stay as they are</small></span></button>
   {:else}
    <button disabled={busy} onclick={() => run({action: 'pause', reason: 'ill', review: null})}><Icon name="pause" size={20}/><span><b>I’m ill</b><small>Pause goals until you resume · nothing to catch up</small></span></button>
    <button disabled={busy} onclick={() => run({action: 'pause', reason: 'pause', review: null})}><Icon name="pause" size={20}/><span><b>Pause training</b><small>Travel, a busy spell · until you resume</small></span></button>
   {/if}
   <button onclick={() => routines = true}><Icon name="check" size={20}/><span><b>My routines</b><small>{current?.routines.length ? `${current.routines.filter(r => r.done).length} of ${current.routines.length} done today · self-reported` : 'A checklist you or your clinician supply'}</small></span></button>
   {#if goals.length && !current?.pause}<button disabled={busy} onclick={() => run({action: 'lighterWeek'})}><Icon name="clock" size={20}/><span><b>Lighter week</b><small>Busy or travelling: half this week’s training goals from today. Original targets stay on record; next week returns.</small></span></button>{/if}
   <button onclick={editLimits}><Icon name="info" size={20}/><span><b>Limits</b><small>{current && (current.limits.excluded.length || current.limits.note) ? [current.limits.excluded.map(s => SPORT_NAMES[s]).join(', ') && `Avoiding ${current.limits.excluded.map(s => SPORT_NAMES[s]).join(', ')}`, current.limits.note && 'movement note'].filter(Boolean).join(' · ') : 'Activities to avoid, a movement note'}</small></span></button>
  </div>
 {/if}
</div>
<style>
 .scrim{position:fixed;inset:0;border:0;background:rgba(5,8,14,.62);z-index:20;padding:0}
 .sheet{position:fixed;left:0;right:0;bottom:0;z-index:21;max-width:560px;margin:0 auto;background:#151B28;border:1px solid var(--line);border-bottom:0;border-radius:26px 26px 0 0;padding:12px 20px 28px;max-height:88vh;overflow:auto}
 .grab{display:block;width:40px;height:4px;border-radius:2px;background:rgba(255,255,255,.18);margin:0 auto 16px}
 .sheet-h{display:flex;justify-content:space-between;align-items:center;gap:12px}
 .close{width:44px;height:44px;border-radius:50%;border:1px solid var(--line);background:none;display:grid;place-items:center;color:var(--text2)}
 .opts{display:grid;gap:8px;margin-top:12px}.opts button{display:grid;grid-template-columns:24px 1fr;gap:12px;align-items:center;text-align:left;padding:12px 14px;border:1px solid var(--line);border-radius:16px;background:rgba(255,255,255,.04);color:var(--text);min-height:56px}
 .opts button:disabled{color:var(--text3);cursor:default}.opts b{display:block;font:600 15px Inter}.opts small{display:block;font-size:12px;color:var(--text2);margin-top:2px}
 .chips2{display:flex;flex-wrap:wrap;gap:8px;margin:12px 0}.chips2 button{min-height:40px;padding:8px 14px;border-radius:999px;border:1px solid var(--line);background:none;color:var(--text2);font:600 13px Inter}.chips2 button[aria-pressed=true]{background:var(--ink);color:var(--bg)}
 .routine{display:grid;grid-template-columns:minmax(0,1fr) auto 44px;gap:8px;align-items:center;padding:10px 0;border-bottom:1px solid var(--line)}.routine b{display:block;font:600 15px Inter}.routine small{display:block;font-size:12px;color:var(--text2)}.routine .secondary{width:auto;margin:0;min-height:40px;padding:6px 10px;font-size:12px}
 .remove{width:44px;height:44px;border-radius:50%;border:1px solid var(--line);background:none;color:var(--text2);display:grid;place-items:center}
 .gh{font:600 11px Inter;letter-spacing:.1em;text-transform:uppercase;color:var(--text2);margin:14px 0 0}.st{font:600 12px Inter;color:var(--fair);white-space:nowrap}
 .goals{margin:10px 0 0;padding-left:18px;font-size:14px;display:grid;gap:4px}
 .note input{background:rgba(255,255,255,.06);border:1px solid var(--line);border-radius:12px;color:var(--text);padding:10px;font:400 14px Inter}
 .note{display:grid;gap:6px;font-size:13px;color:var(--text2);margin-top:10px}.note textarea{background:rgba(255,255,255,.06);border:1px solid var(--line);border-radius:12px;color:var(--text);padding:10px;font:400 14px Inter;resize:vertical}
</style>
