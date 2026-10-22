<script lang="ts">
 import Icon from './Icon.svelte';
 import type {PlanBoard} from '../generated/PlanBoard';
 import type {PlanItem} from '../generated/PlanItem';
 import type {LocalAction} from '../generated/LocalAction';
 import type {Allocation} from '../generated/Allocation';
 import {GOALS, SPORT_NAMES, STATE_WORD} from '../goals';
 import {sheetOpened} from '../sheet';
 import {clock, dateLabel, hoursMinutes} from '../format';
 // One session's choices. Moving adds no catch-up; only its person marks a
 // passed session missed; a match Raring made can be undone.
 let {board, item, close, act, navigate, limits = null}: {board: PlanBoard, item: PlanItem, close: () => void, act: (a: LocalAction) => void, navigate: (route: string) => void, limits?: import('../generated/Limits').Limits | null} = $props();
 let conflict = $derived(!!item.coach && !!limits && limits.excluded.includes(item.intention.sport) && item.coach.kind === 'strength');
 let unknownContents = $derived(!!item.coach && !!limits && (!!limits.note || (item.coach.kind !== 'strength' && limits.excluded.length > 0)));
 let moving = $state(false), editing = $state(false);
 let time = $state(''), name = $state(''), minutes = $state<number | null>(null), alloc = $state<Allocation>({strengthSessions: 0, zone2Min: 0, highIntensityMin: 0, intensityMin: 0, sleepNights: 0});
 // Shares for the goals on this week; zero leaves a goal out.
 let shares = $derived(board.ledger.filter(l => !l.observation && l.kind !== 'sleepScore').map(l => [l.kind, ({strength: 'strengthSessions', zone2: 'zone2Min', highIntensity: 'highIntensityMin', intensity: 'intensityMin', sleepTiming: 'sleepNights'} as const)[l.kind as 'strength']] as const).filter(([, f]) => !!f));
 const STEP: Record<string, number> = {strengthSessions: 1, sleepNights: 1, zone2Min: 5, highIntensityMin: 5, intensityMin: 5};
 function startEdit() { time = i.time ?? ''; name = i.name ?? ''; minutes = i.durationMin; alloc = {...i.allocation}; editing = true; }
 let i = $derived(item.intention);
 let passed = $derived(i.date < board.today);
 let open = $derived(i.status === 'open');
 let days = $derived(Array.from({length: 7}, (_, n) => { const d = new Date(board.today + 'T12:00:00Z'); d.setUTCDate(d.getUTCDate() + n); return d.toISOString().slice(0, 10); }).filter(d => d !== i.date && d <= board.end));
 let candidates = $derived(item.candidates.map(id => board.recordings.find(r => r.id === id)).filter(r => !!r));
 let recording = $derived(board.recordings.find(r => r.id === item.recording));
 $effect(() => sheetOpened(close));
 const run = (a: LocalAction) => { act(a); close(); };
 const history = (h: PlanItem['intention']['history'][number]) => h.change === 'moved' ? `Moved from ${dateLabel(h.from ?? '')} to ${dateLabel(h.to ?? '')}` : h.change === 'added' ? `Added for ${dateLabel(h.to ?? i.date)}` : h.change === 'linkCleared' ? 'Link removed' : h.change === 'letGo' ? 'Let go' : h.change === 'missed' ? 'Marked missed' : h.change === 'reopened' ? 'Reopened' : 'Edited';
</script>
<svelte:window onkeydown={e => { if (e.key === 'Escape') close(); }}/>
<button class="scrim" aria-label="Close" onclick={close}></button>
<div class="sheet" role="dialog" aria-modal="true" aria-labelledby="item-title">
 <span class="grab"></span>
 <div class="sheet-h"><h2 id="item-title">{i.name ?? SPORT_NAMES[i.sport]}</h2><button class="close" aria-label="Close" onclick={close}><Icon name="close" size={20}/></button></div>
 {#if item.coach}<p class="small coach">◇ Garmin Coach · read-only · {item.coach.kind === 'strength' ? 'counts as a planned strength session' : item.coach.kind === 'cardio' ? 'any cardio recording that day can match it' : 'Garmin’s sport isn’t known to Raring'}{item.coach.durationMin ? ` · about ${item.coach.durationMin} min` : ''}{item.coach.effect ? ` · ${item.coach.effect}` : ''}</p>{#if conflict}<p class="warn" role="note"><b>Conflicts with your excluded activity.</b> Garmin Coach planned strength; Raring doesn’t change Garmin’s plan.</p>{:else if unknownContents}<p class="warn" role="note"><b>Review needed.</b> Contents unknown to Raring: Garmin doesn’t send this session’s movements{item.coach.kind === 'cardio' ? ' or sport' : ''}, so it can’t be checked against your limits.</p>{/if}<p class="small">Garmin: {item.coach.garminStatus ?? 'status unknown'} · Raring marks it done only from a matching recording.</p>{/if}
 <p class="small">{dateLabel(i.date)}{i.time ? ` · ${i.time}` : ''} · {STATE_WORD[item.state]}{item.matchedBy === 'automatic' ? ' · matched by Raring (same day and sport)' : item.matchedBy === 'linked' ? ' · linked by you' : ''}</p>
 {#if recording}<button class="opt" onclick={() => { close(); navigate('workout/' + recording.id); }}><Icon name={recording.icon} size={18}/><span><b>{recording.name}</b><small>{clock(recording.startLocal)} · {hoursMinutes(recording.durationMin)}</small></span></button>
  <button class="opt" onclick={() => run({action: 'unlink', planId: i.id, activityId: recording.id})}><Icon name="close" size={18}/><span><b>Not this session</b><small>Keep them apart; the recording still counts</small></span></button>{/if}
 {#if item.state === 'review'}{#each candidates as r}<button class="opt" onclick={() => run({action: 'link', planId: i.id, activityId: r!.id})}><Icon name="check" size={18}/><span><b>This was my session</b><small>{r!.name} · {clock(r!.startLocal)}</small></span></button>{/each}{/if}
 {#if editing && !item.coach}
  <div class="form" role="group" aria-label="Edit session">
   <label><span>Name</span><input maxlength="60" bind:value={name} placeholder={SPORT_NAMES[i.sport]}/></label>
   <label><span>Time</span><input type="time" bind:value={time}/></label>
   <label><span>Minutes</span><input type="number" min="1" max="600" inputmode="numeric" value={minutes ?? ''} oninput={e => { const v = Number(e.currentTarget.value); minutes = e.currentTarget.value && v > 0 ? Math.round(v) : null; }}/></label>
   {#each shares as [kind, field] (kind)}<div class="step"><span>{GOALS[kind].name}</span><button aria-label={`Less ${GOALS[kind].name}`} disabled={alloc[field] <= 0} onclick={() => alloc[field] = Math.max(0, alloc[field] - STEP[field])}>−</button><b class="n">{alloc[field]}</b><button aria-label={`More ${GOALS[kind].name}`} onclick={() => alloc[field] += STEP[field]}>+</button></div>{/each}
   <div class="row2"><button class="secondary" onclick={() => editing = false}>Cancel</button><button class="primary" onclick={() => run({action: 'editIntention', id: i.id, time: time || null, name: name.trim() || null, durationMin: minutes, allocation: {...alloc}})}>Save</button></div>
  </div>
 {:else if open && item.state !== 'done' && !item.coach}
  <button class="opt" onclick={startEdit}><Icon name="pencil" size={18}/><span><b>Edit</b><small>Time, name, minutes and goal shares</small></span></button>
  {#if moving}<div class="chips2" role="group" aria-label="Move to">{#each days as d}<button onclick={() => run({action: 'moveIntention', id: i.id, date: d})}>{d === board.today ? 'Today' : dateLabel(d)}</button>{/each}</div>
  {:else if days.length}<button class="opt" onclick={() => moving = true}><Icon name="arrow-right" size={18}/><span><b>{passed ? 'Reschedule' : 'Move'}</b><small>To today or a later day this week · nothing to catch up</small></span></button>{/if}
  <button class="opt" onclick={() => run({action: 'setIntentionStatus', id: i.id, status: 'letGo'})}><Icon name="close" size={18}/><span><b>Let it go</b><small>Not planned any more</small></span></button>
  {#if passed}<button class="opt" onclick={() => run({action: 'setIntentionStatus', id: i.id, status: 'missed'})}><Icon name="alert" size={18}/><span><b>Mark as missed</b><small>Record that it didn’t happen</small></span></button>{/if}
  {#if !passed}<button class="opt" onclick={() => run({action: 'removeIntention', id: i.id})}><Icon name="close" size={18}/><span><b>Remove</b><small>Delete it from your plan</small></span></button>{/if}
 {:else if !open && !item.coach}<button class="opt" onclick={() => run({action: 'setIntentionStatus', id: i.id, status: 'open'})}><Icon name="arrow-left" size={18}/><span><b>Reopen</b><small>Back in your plan</small></span></button>{/if}
 {#if !item.coach}<button class="opt" disabled aria-describedby="garmin-na"><Icon name="arrow-right" size={18}/><span><b>Send to Garmin</b><small id="garmin-na">Not yet available: Raring sends only weigh-ins to Garmin so far</small></span></button>{/if}
 {#if i.history.length}<ol class="hist">{#each i.history as h}<li>{history(h)}</li>{/each}</ol>{/if}
</div>
<style>
 .scrim{position:fixed;inset:0;border:0;background:rgba(5,8,14,.62);z-index:20;padding:0}
 .sheet{position:fixed;left:0;right:0;bottom:0;z-index:21;max-width:560px;margin:0 auto;background:var(--card);border:1px solid var(--line);border-bottom:0;border-radius:26px 26px 0 0;padding:12px 20px calc(28px + env(safe-area-inset-bottom));max-height:90dvh;overflow:auto}
 .grab{display:block;width:40px;height:4px;border-radius:2px;background:rgba(255,255,255,.18);margin:0 auto 16px}
 .sheet-h{display:flex;justify-content:space-between;align-items:center;gap:12px}.sheet-h h2{min-width:0;overflow-wrap:anywhere}
 .close{flex:none;width:44px;height:44px;border-radius:50%;border:1px solid var(--line);background:none;display:grid;place-items:center;color:var(--text2)}
 .opt{display:grid;grid-template-columns:24px minmax(0,1fr);gap:12px;align-items:center;width:100%;text-align:left;margin-top:8px;padding:12px 14px;border:1px solid var(--line);border-radius:16px;background:rgba(255,255,255,.04);color:var(--text);min-height:52px}
 .opt:disabled{color:var(--text2);border-style:dashed;cursor:default}.opt span{min-width:0;overflow-wrap:anywhere}.opt b{display:block;font:600 15px Inter}.opt small{display:block;font-size:12px;color:var(--text2);margin-top:2px}
 .chips2{display:flex;flex-wrap:wrap;gap:8px;margin:12px 0 0}.chips2 button{min-height:40px;padding:8px 12px;border-radius:999px;border:1px solid var(--line);background:none;color:var(--text);font:600 13px Inter}
 .form{display:grid;gap:10px;margin-top:12px}.form label{display:grid;min-width:0;gap:4px;font-size:13px;color:var(--text2)}.form input{width:100%;min-width:0;box-sizing:border-box;background:rgba(255,255,255,.06);border:1px solid var(--line);border-radius:12px;color:var(--text);padding:10px;font:400 15px Inter;min-height:44px}
 .step{display:grid;grid-template-columns:minmax(0,1fr) 44px 48px 44px;gap:6px;align-items:center;font-size:14px}.step button{min-height:44px;border-radius:12px;border:1px solid var(--line);background:none;color:var(--text);font:600 18px Inter}.step button:disabled{color:var(--text3)}.step b{text-align:center}
 .row2{display:grid;grid-template-columns:repeat(auto-fit,minmax(min(100%,9rem),1fr));gap:8px}.row2 button{min-width:0;margin:0}
 .coach{color:#BDAAFF}
 .warn{font-size:14px;line-height:1.45;margin:10px 0 0;padding:10px 12px;border-radius:12px;background:rgba(255,181,71,.08);border:1px solid rgba(255,181,71,.3)}
 .hist{margin:14px 4px 0;padding-left:18px;font-size:12px;color:var(--text2);display:grid;gap:3px}
</style>
