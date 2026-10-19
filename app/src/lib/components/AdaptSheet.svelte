<script lang="ts">
 import Icon from './Icon.svelte';
 import type {AdaptState} from '../generated/AdaptState';
 import type {LocalAction} from '../generated/LocalAction';
 import type {Sport} from '../generated/Sport';
 import {SPORT_NAMES} from '../goals';
 import {sheetOpened} from '../sheet';
 import {dateLabel} from '../format';
 // Rest, illness, a pause and limits: what to change, never why. Each holds until
 // the person changes it; nothing resumes on a date or a good reading.
 let {current, close, act, busy = false}: {current: AdaptState | null, close: () => void, act: (a: LocalAction) => void, busy?: boolean} = $props();
 let limits = $state(false);
 let excluded = $state<Sport[]>([]);
 let note = $state('');
 const SPORTS: Sport[] = ['running', 'cycling', 'walking', 'hiking', 'swimming', 'strength'];
 $effect(() => sheetOpened(close));
 function editLimits() { excluded = [...(current?.limits.excluded ?? [])]; note = current?.limits.note ?? ''; limits = true; }
 const toggle = (s: Sport) => excluded = excluded.includes(s) ? excluded.filter(x => x !== s) : [...excluded, s];
 const run = (a: LocalAction) => { act(a); close(); };
</script>
<svelte:window onkeydown={e => { if (e.key === 'Escape') close(); }}/>
<button class="scrim" aria-label="Close" onclick={close}></button>
<div class="sheet" role="dialog" aria-modal="true" aria-labelledby="adapt-title">
 <span class="grab"></span>
 <div class="sheet-h"><h2 id="adapt-title">{limits ? 'Your limits' : 'Adapt today'}</h2><button class="close" aria-label="Close" onclick={close}><Icon name="close" size={20}/></button></div>
 {#if limits}
  <p class="small">Activities to avoid are left out of suggestions. A movement note is shown wherever a session’s contents are unknown; Raring doesn’t check workouts against it.</p>
  <div class="chips2" role="group" aria-label="Activities to avoid">{#each SPORTS as s}<button aria-pressed={excluded.includes(s)} onclick={() => toggle(s)}>{SPORT_NAMES[s]}</button>{/each}</div>
  <label class="note"><span>Movement note</span><textarea rows="3" maxlength="500" bind:value={note} placeholder="For example: no overhead pressing"></textarea></label>
  <button class="primary" disabled={busy} onclick={() => run({action: 'setLimits', limits: {excluded, note}})}>Save limits</button>
 {:else}
  <div class="opts">
   {#if current?.rest}<button onclick={() => run({action: 'cancelRest'})}><Icon name="moon" size={20}/><span><b>Rest day is on</b><small>Turn it off · prompts return today</small></span></button>
   {:else}<button disabled={busy || !!current?.pause} onclick={() => run({action: 'restToday'})}><Icon name="moon" size={20}/><span><b>Rest today</b><small>No exercise prompts today or tonight</small></span></button>{/if}
   {#if current?.pause}<button onclick={() => run({action: 'resume'})}><Icon name="pause" size={20}/><span><b>Resume training</b><small>{current.pause.reason === 'ill' ? 'Ill' : 'Paused'} since {dateLabel(current.pause.since)} · your goals stay as they are</small></span></button>
   {:else}
    <button disabled={busy} onclick={() => run({action: 'pause', reason: 'ill', review: null})}><Icon name="pause" size={20}/><span><b>I’m ill</b><small>Pause goals until you resume · nothing to catch up</small></span></button>
    <button disabled={busy} onclick={() => run({action: 'pause', reason: 'pause', review: null})}><Icon name="pause" size={20}/><span><b>Pause training</b><small>Travel, a busy spell · until you resume</small></span></button>
   {/if}
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
 .note{display:grid;gap:6px;font-size:13px;color:var(--text2)}.note textarea{background:rgba(255,255,255,.06);border:1px solid var(--line);border-radius:12px;color:var(--text);padding:10px;font:400 14px Inter;resize:vertical}
</style>
