<script lang="ts">
 import Icon from './Icon.svelte';
 import type {PlanBoard} from '../generated/PlanBoard';
 import type {LocalAction} from '../generated/LocalAction';
 import type {Sport} from '../generated/Sport';
 import type {Draft} from './AddSessionSheet.svelte';
 import {GOALS, SPORT_ICONS, SPORT_NAMES, amount, backlogDraft} from '../goals';
 import {sheetOpened} from '../sheet';
 import {dateLabel} from '../format';
 // Choose another: today's Garmin schedule and your sessions, one taken from
 // later in the week or from what's left, a new session, or rest. Keeping one
 // only changes what Today leads with; nothing is sent to Garmin.
 let {board, kept, excluded = [], limited = false, close, act, add}: {board: PlanBoard, kept: string | null, excluded?: Sport[], limited?: boolean, close: () => void, act: (a: LocalAction) => void, add: (d: Draft) => void} = $props();
 $effect(() => sheetOpened(close));
 let day = $derived(board.today);
 let garmin = $derived(board.garmin.filter(g => g.date === day).map(g => ({id: `garmin:${day}:${g.name}`, name: g.name})));
 let open = $derived(board.items.filter(p => p.state === 'planned' && p.intention.kind === 'workout'));
 let todays = $derived(open.filter(p => p.intention.date === day));
 let later = $derived(open.filter(p => p.intention.date > day && !p.coach));
 const name = (p: PlanBoard['items'][number]) => p.intention.name ?? SPORT_NAMES[p.intention.sport];
 const keep = (id: string) => { act({action: 'keepToday', planId: id}); close(); };
 const bring = (id: string) => { act({action: 'moveIntention', id, date: day}); act({action: 'keepToday', planId: id}); close(); };
</script>
<svelte:window onkeydown={e => { if (e.key === 'Escape') close(); }}/>
<button class="scrim" aria-label="Close" onclick={close}></button>
<div class="sheet" role="dialog" aria-modal="true" aria-labelledby="choose-title">
 <span class="grab"></span>
 <div class="sheet-h"><h2 id="choose-title">Choose today’s session</h2><button class="close" aria-label="Close" onclick={close}><Icon name="close" size={20}/></button></div>
 {#if garmin.length}<h3>Garmin schedule</h3><div class="opts">{#each garmin as g}<button aria-pressed={kept === g.id} onclick={() => keep(g.id)}><span class="gm" aria-hidden="true">◇</span><span><b>{g.name}</b><small>{kept === g.id ? 'Kept for today' : `Garmin · read-only${limited ? ' · Review needed' : ''}`}</small></span></button>{/each}</div>{/if}
 {#if todays.length}<h3>Planned today</h3><div class="opts">{#each todays as p}<button aria-pressed={kept === p.intention.id} disabled={excluded.includes(p.intention.sport)} onclick={() => keep(p.intention.id)}><Icon name={SPORT_ICONS[p.intention.sport]} size={20}/><span><b>{name(p)}{p.intention.time ? ` · ${p.intention.time}` : ''}</b><small>{excluded.includes(p.intention.sport) ? 'Excluded by your limits' : kept === p.intention.id ? 'Kept for today' : `${p.coach ? 'Garmin Coach · ' : ''}${p.intention.durationMin ? `${p.intention.durationMin} min` : 'Any duration'}`}</small></span></button>{/each}</div>{/if}
 {#if later.length}<h3>Later this week</h3><div class="opts">{#each later as p}<button disabled={excluded.includes(p.intention.sport)} onclick={() => bring(p.intention.id)}><Icon name={SPORT_ICONS[p.intention.sport]} size={20}/><span><b>{name(p)}</b><small>{excluded.includes(p.intention.sport) ? 'Excluded by your limits' : `Planned ${dateLabel(p.intention.date)} · move to today`}</small></span></button>{/each}</div>{/if}
 {#if !board.backlogPaused && board.backlog.some(b => b.kind !== 'sleepTiming')}<h3>Left this week</h3><div class="opts">{#each board.backlog.filter(b => b.kind !== 'sleepTiming') as b}{@const d = backlogDraft(b, day)}<button disabled={excluded.includes(d.sport)} onclick={() => { close(); add(d); }}><Icon name={GOALS[b.kind].icon} size={20}/><span><b>{b.kind === 'strength' ? 'Strength session' : `${amount(b.kind, b.amount)} ${GOALS[b.kind].name}`}</b><small>{excluded.includes(d.sport) ? 'Excluded by your limits' : 'Do it today'}</small></span></button>{/each}</div>{/if}
 <h3>Something else</h3>
 <div class="opts">
  <button onclick={() => { close(); add({date: day}); }}><Icon name="pencil" size={20}/><span><b>Add a session</b><small>Pick a sport and time for today</small></span></button>
  <button onclick={() => { act({action: 'restToday'}); close(); }}><Icon name="moon" size={20}/><span><b>Rest today</b><small>No exercise prompts today or tonight</small></span></button>
 </div>
</div>
<style>
 .scrim{position:fixed;inset:0;border:0;background:rgba(5,8,14,.62);z-index:20;padding:0}
 .sheet{position:fixed;left:0;right:0;bottom:0;z-index:21;max-width:560px;margin:0 auto;background:#151B28;border:1px solid var(--line);border-bottom:0;border-radius:26px 26px 0 0;padding:12px 20px 28px;max-height:88vh;overflow:auto}
 .grab{display:block;width:40px;height:4px;border-radius:2px;background:rgba(255,255,255,.18);margin:0 auto 16px}
 .sheet-h{display:flex;justify-content:space-between;align-items:center;gap:12px}
 .close{width:44px;height:44px;border-radius:50%;border:1px solid var(--line);background:none;display:grid;place-items:center;color:var(--text2)}
 h3{font:600 11px Inter;letter-spacing:.1em;text-transform:uppercase;color:var(--text3);margin:16px 0 0}
 .opts{display:grid;gap:8px;margin-top:8px}.opts button{display:grid;grid-template-columns:24px 1fr;gap:12px;align-items:center;text-align:left;padding:12px 14px;border:1px solid var(--line);border-radius:16px;background:rgba(255,255,255,.04);color:var(--text);min-height:56px}
 .gm{display:grid;place-items:center;width:24px;font-size:18px;color:var(--text2)}
 .opts button[aria-pressed=true]{border-color:var(--ink)}
 .opts button:disabled{color:var(--text3);cursor:default}.opts b{display:block;font:600 15px Inter}.opts small{display:block;font-size:12px;color:var(--text2);margin-top:2px}
</style>
