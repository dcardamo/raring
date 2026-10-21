<script lang="ts">
 import Icon from '../components/Icon.svelte';
 import DayCells from '../components/DayCells.svelte';
 import HelpButton from '../components/HelpButton.svelte';
 import WeekSummary from '../components/WeekSummary.svelte';
 import type {PlanBoard} from '../generated/PlanBoard';
 import type {PlanItem} from '../generated/PlanItem';
 import type {LocalAction} from '../generated/LocalAction';
 import {GOALS, backlogDraft, itemIcon, SPORT_NAMES, STATE_WORD, amount} from '../goals';
 import {clock, dateLabel, dateLabelIn, hoursMinutes} from '../format';
 // Plan: the week at a glance (rings, one line per goal and one place for what
 // needs you), the days with what was planned and recorded, and what's left.
 // Garmin's schedule and Garmin Coach are shown read-only in the same days.
 let {board, navigate, act, header, openAdd, openItem, openEvent, openGarmin, limited = false}: {board: PlanBoard, navigate: (route: string) => void, act: (a: LocalAction) => void, header: import('svelte').Snippet,
  openAdd: (draft: {date?: string, sport?: import('../generated/Sport').Sport, allocation?: Record<string, number>, name?: string}) => void, openItem: (item: PlanItem) => void, openEvent: () => void, openGarmin?: (g: PlanBoard['garmin'][number]) => void, limited?: boolean} = $props();
 const take = (b: PlanBoard['backlog'][number], date: string) => openAdd(backlogDraft(b, date));
 let picked = $state<string | null>(null);
 let selected = $derived(picked && board.days.some(d => d.date === picked) ? picked : board.days.find(d => d.today)?.date ?? board.start);
 let day = $derived(board.days.find(d => d.date === selected)!);
 let items = $derived(board.items.filter(p => p.intention.date === selected && p.state !== 'letGo'));
 let matched = $derived(new Set(board.items.map(p => p.recording).filter(Boolean)));
 let loose = $derived(board.recordings.filter(r => r.date === selected && !matched.has(r.id)));
 let garmin = $derived(board.garmin.filter(g => g.date === selected));
 let ev = $derived(board.eventWeek);
 let coachRest = $derived(board.coach?.sessions.filter(s => s.restDay && s.date === selected) ?? []);
 let coachItems = $derived(board.items.filter(p => p.coach));
 let review = $derived(board.items.filter(p => p.state === 'review'));
 // Only your own sessions can be rescheduled or let go; Garmin Coach's can't.
 let missed = $derived(board.items.filter(p => p.state === 'notRecorded' && !p.coach));
 let showMissed = $state(false);
 const dayName = (d: string, long = false) => new Date(d + 'T12:00:00Z').toLocaleDateString('en-GB', {weekday: long ? 'long' : 'short', day: 'numeric', month: 'short', timeZone: 'UTC'});
 const weekday = (d: string) => dateLabel(d).split(' ')[0];
 let range = $derived(`${dayName(board.start)} – ${dayName(board.end)}`);
 const recordingOf = (p: PlanItem) => board.recordings.find(r => r.id === p.recording);
 // Unnamed sessions are told apart by sport and time.
 const title = (p: PlanItem) => p.intention.name ?? (p.intention.time ? `${SPORT_NAMES[p.intention.sport]} ${p.intention.time}` : SPORT_NAMES[p.intention.sport]);
 const recName = (id: string) => { const r = board.recordings.find(x => x.id === id); return r ? `${r.name} ${clock(r.startLocal)}` : 'A recording'; };
 const reviewText = (p: PlanItem) => `${title(p)}${p.intention.durationMin ? ` (${p.intention.durationMin} min)` : ''}, ${dayName(p.intention.date)}: ${p.candidates.map(recName).join(' or ')}?`;
 // Sessions a loose recording may be, for its label and link choice.
 const contenders = (id: string) => review.filter(p => p.candidates.includes(id));
 const named = (p: PlanItem) => p.coach ? `${title(p)} (Garmin Coach)` : title(p);
 const when = (p: PlanItem) => [p.intention.time, p.intention.durationMin ? `${p.intention.durationMin} min` : null].filter(Boolean).join(' · ') || 'Any time';
 const goalsOf = (p: PlanItem) => (Object.entries(p.intention.allocation) as [string, number][]).filter(([, v]) => v > 0)
  .map(([k, v]) => ({strengthSessions: amount('strength', v), zone2Min: amount('zone2', v) + ' Zone 2', highIntensityMin: amount('highIntensity', v) + ' Zone 4–5', intensityMin: amount('intensity', v) + ' intensity', sleepNights: 'sleep timing'} as Record<string, string>)[k]).join(' · ');
 const garminNote = $derived(board.garminState === 'partial' ? 'Garmin’s schedule covers only part of this week' : board.garminState === 'stale' ? 'Garmin’s schedule may be out of date' : null);
 function remove(e: PlanBoard['events'][number]) { if (e.id && confirm(`Remove ${e.name}?`)) act({action: 'removeEvent', id: e.id}); }
</script>
{@render header()}
<section class="card wk">
 <div class="wk-h"><div><h2>{range}</h2>{#if board.pause}<small>{board.pause.reason === 'ill' ? 'Ill' : 'Paused'} {board.pause.wholeWeek ? 'all week' : `since ${dayName(board.pause.since)}`}{board.pause.until && !board.pause.wholeWeek ? ` to ${dayName(board.pause.until)}` : ''}{board.backlogPaused ? '' : ' · nothing to catch up'}</small>{/if}</div>
  <span class="wk-a"><HelpButton topic="plan"/><button class="small-edit" onclick={() => navigate('plan/commitments')}><Icon name="pencil" size={13}/>Goals</button></span></div>
 <WeekSummary {board} onGoals={() => navigate('plan/commitments')}/>
 {#if review.length || missed.length}
  <div class="needs" role="group" aria-label="Needs you">
   {#each review as p (p.intention.id)}<button class="need" onclick={() => openItem(p)}><span><b>Needs review</b> {reviewText(p)}</span><Icon name="chevron" size={15}/></button>{/each}
   {#if missed.length}<button class="need" aria-expanded={showMissed} onclick={() => showMissed = !showMissed}><span><b>{missed.length} not recorded</b> · {[...new Set(missed.map(p => weekday(p.intention.date)))].join(', ')}</span><Icon name="chevron-down" size={15}/></button>
    {#if showMissed}{#each missed as p (p.intention.id)}<div class="nr-row"><span>{weekday(p.intention.date)} · {title(p)}</span><button class="small-edit" onclick={() => openItem(p)}>Reschedule</button><button class="small-edit" onclick={() => act({action: 'setIntentionStatus', id: p.intention.id, status: 'letGo'})}>Let it go</button></div>{/each}{/if}
   {/if}
  </div>
 {/if}
 {#if board.coach && coachItems.length}<p class="coach-line"><span aria-hidden="true">◇</span> <b>{board.coach.name}</b> · Garmin Coach · read-only{board.coach.state === 'stale' ? ' · may be out of date' : ''} · {coachItems.filter(p => p.state === 'done').length} of {coachItems.length} done</p>{/if}
 {#if !board.coverageComplete && board.ledger.length}<p class="small">Garmin’s activity list is incomplete for this week, so some goals are awaiting data.</p>{/if}
 {#if board.readOnly}<p class="small">Some plan records were saved by a newer Raring and can’t be changed here.</p>{/if}
</section>
{#if board.explainObservation}
 <section class="card note"><Icon name="moon" size={18}/><div><b>Sleep timing is now an observation</b><p class="small">Your earlier asleep-on-time target is kept as a reference and compared with each night’s saved plan, without a verdict. Earlier weeks keep their results.</p><button class="secondary" onclick={() => navigate('sleep/tonight')}>Review timing</button><button class="secondary" onclick={() => act({action: 'observationExplained'})}>Got it</button></div></section>
{/if}

<div class="dashboard plan-layout">
 <div class="plan-days">
<div class="sec-h"><h2>Days</h2><HelpButton topic="rings"/></div>
<section class="card days">
 <DayCells days={board.days} {selected} select={d => picked = d} coachRest={board.coach?.sessions.filter(c => c.restDay).map(c => c.date)}/>
 <div class="agenda-h"><h3>{day.today ? 'Today' : dayName(selected, true)}</h3>{#if day.mark}<span class="pill">{({rest: 'Rest day', ill: 'Ill', paused: 'Paused', unknown: 'Awaiting data'} as const)[day.mark]}</span>{/if}</div>
 {#each items as p (p.intention.id)}
  {@const r = recordingOf(p)}
  <button class="row-item" class:serves={ev?.items.includes(p.intention.id)} class:done={p.state === 'done'} class:muted={p.state === 'notRecorded' || p.state === 'missed' || p.state === 'awaiting'} onclick={() => openItem(p)}>
   <span class="ico" class:outline={p.state !== 'done'}><Icon name={p.state === 'done' ? 'check' : itemIcon(p)} size={18}/></span>
   <span><small>{p.coach ? 'Garmin Coach · ' : ''}{p.coach && limited && p.state === 'planned' ? 'Review needed · ' : ''}{STATE_WORD[p.state]}{p.matchedBy === 'automatic' ? ' · matched by Raring' : p.matchedBy === 'linked' ? ' · linked by you' : ''}</small><b>{title(p)}</b>{#if ev?.items.includes(p.intention.id)}<i class="for">For {ev.event.name}</i>{/if}<em>{r ? `${clock(r.startLocal)} · ${hoursMinutes(r.durationMin)}` : when(p)}{goalsOf(p) ? ' · ' + goalsOf(p) : ''}</em>{#if coachRest.length && !p.coach}<i class="clash">Garmin Coach rests today</i>{/if}</span>
   <Icon name="chevron" size={15}/>
  </button>
 {/each}
 {#each loose as r (r.id)}{@const may = contenders(r.id)}
  <button class="row-item" class:done={!may.length} onclick={() => may.length ? openItem(may[0]) : navigate('workout/' + r.id)}><span class="ico"><Icon name={r.icon} size={18}/></span><span><small class:ask={may.length}>{may.length ? `Needs review · may be ${may.map(named).join(' or ')}` : 'Recorded · not in your plan'}</small><b>{r.name}</b><em>{clock(r.startLocal)} · {hoursMinutes(r.durationMin)}</em></span><Icon name="chevron" size={15}/></button>
 {/each}
 {#each coachRest as r (r.id)}<div class="row-item garmin"><span class="ico gm" aria-hidden="true">◇</span><span><small>Garmin Coach · read-only</small><b>{items.some(p => !p.coach) ? 'Garmin Coach rests today' : 'Rest day'}</b><em>{board.coach?.name}</em></span></div>{/each}
 {#each garmin as g}
  <button class="row-item garmin" onclick={() => openGarmin?.(g)}><span class="ico gm" aria-hidden="true">◇</span><span><small>Garmin schedule · read-only{limited ? ' · Review needed' : ''}</small><b>{g.name}</b><em>{limited ? 'Contents unknown to Raring' : 'Not counted toward your goals'}</em></span><Icon name="chevron" size={15}/></button>
 {/each}
 {#if !items.length && !loose.length && !garmin.length && !coachRest.length}<p class="empty-day">{day.mark === 'rest' ? 'A day of rest' : 'Nothing on this day'}</p>{/if}
 {#if selected >= board.today}<button class="secondary" onclick={() => openAdd({date: selected})}>Add a session</button>{/if}
 {#if garminNote}<p class="small">{garminNote}.</p>{/if}
</section>
 </div>
 <div class="plan-support">
{#if board.backlogPaused || board.backlog.length}
 <div class="sec-h"><h2>Left this week</h2></div>
 <section class="card">
  {#if board.backlogPaused}<p class="small">Paused · nothing to catch up.</p>
  {:else}<ul class="left">{#each board.backlog as b}<li style:--c={GOALS[b.kind].color}><i></i><span>{b.kind === 'strength' ? 'Strength session' : `${amount(b.kind, b.amount)} ${GOALS[b.kind].name}`}</span>{#if b.kind !== 'sleepTiming'}<button class="small-edit" onclick={() => take(b, board.today)}>Do it today</button>{/if}</li>{/each}</ul>{/if}
 </section>
{/if}

<section class="events-section">
<div class="sec-h"><h2>Events</h2><button class="link" onclick={openEvent}>Add a race or event</button></div>
{#each board.events as e}
 <div class="card event"><span class="cd"><b class="n">{e.daysUntil}</b><small>days</small></span><span><b>{e.name}</b><small>{dateLabelIn(e.date, board.today)} · {e.source === 'garmin' ? `Garmin calendar${e.partial ? ' · later races may be missing' : ''}` : 'Saved on this phone'}</small>{#if ev && ev.event.name === e.name && ev.event.date === e.date}<small class="ev-week">{SPORT_NAMES[ev.sport]} this week: {Math.round(ev.recordedMin)} min recorded</small>{/if}</span>{#if e.id}<button class="link" aria-label={`Remove ${e.name}`} onclick={() => remove(e)}>Remove</button>{/if}</div>
{:else}<p class="small none">No race or event yet.</p>{/each}
</section>
 </div>
</div>
<style>
 .plan-layout{margin-top:16px}.plan-layout>div{min-width:0}.plan-days>.sec-h:first-child,.plan-support>.sec-h:first-child{margin-top:0}.plan-layout .days{margin-top:16px}.events-section{min-width:0}
 .wk-a{display:flex;gap:8px;align-items:center}
 :global(.sec-h:has(.qh)){display:flex;align-items:center;gap:8px}
 .row-item.serves{box-shadow:inset 3px 0 0 var(--fair)}.for,.clash{display:block;font:600 12px Inter;font-style:normal;color:var(--fair)}.clash{color:#BDAAFF}
 .needs{display:grid;gap:6px;margin-top:12px}
 .need{display:grid;grid-template-columns:minmax(0,1fr) 15px;gap:8px;align-items:center;width:100%;min-height:44px;padding:8px 10px;border:1px dashed rgba(255,181,71,.45);border-radius:12px;background:rgba(255,181,71,.06);color:var(--text2);font-size:13px;text-align:left}.need b{color:var(--fair);font-weight:600}
 .nr-row{display:flex;flex-wrap:wrap;gap:6px;align-items:center;padding:2px 2px 2px 10px;font-size:13px;color:var(--text2)}.nr-row span{min-width:0;flex:1 1 100%}.nr-row .small-edit{white-space:normal;text-align:center}
 .coach-line{margin:10px 0 0;font-size:13px;color:var(--text2)}.coach-line span{color:#BDAAFF}.coach-line b{color:var(--text);font-weight:600}
 .row-item small.ask{color:var(--fair)}.ev-week{color:var(--fair)}
 .wk{padding:14px}.wk-h{display:flex;flex-wrap:wrap;justify-content:space-between;align-items:flex-start;gap:8px;margin-bottom:12px}.wk-h h2{font:600 20px Outfit}.wk-h small{display:block;font-size:11.5px;color:var(--goal-sleep);margin-top:2px}
 .small-edit{background:rgba(255,255,255,.06);border:1px solid var(--line);border-radius:11px;display:flex;gap:5px;align-items:center;padding:8px 10px;color:var(--text);font:600 12px Inter;min-height:36px}
 .note{display:flex;gap:12px;align-items:flex-start}.note b{font:600 15px Outfit}.note :global(svg){color:var(--goal-sleep);flex:none;margin-top:2px}
 .sec-h{display:flex;justify-content:space-between;align-items:baseline;margin:22px 4px 0}.sec-h h2{font:600 19px Outfit}
 .days{padding:12px 10px}.agenda-h{display:flex;justify-content:space-between;align-items:center;border-top:1px solid var(--line);margin:14px 2px 4px;padding-top:10px}.agenda-h h3{margin:0;font:600 17px Outfit;color:var(--text);text-transform:none;letter-spacing:0}
 .row-item{display:grid;grid-template-columns:32px minmax(0,1fr) 15px;align-items:center;gap:12px;width:100%;margin-top:8px;padding:10px;border:1px solid var(--line);border-radius:16px;background:rgba(255,255,255,.03);color:var(--text);text-align:left;overflow-wrap:anywhere}
 .row-item:disabled{cursor:default}.row-item small{display:block;font:600 11px Inter;color:var(--text3)}.row-item b{display:block;font:600 15px Inter}.row-item em{display:block;font-style:normal;font-size:12px;color:var(--text2);margin-top:2px}
 .row-item.done small{color:var(--good)}.row-item.muted{opacity:.72}.row-item.garmin{border-style:dashed;border-color:rgba(164,139,255,.35)}
 .ico{width:32px;height:32px;border-radius:11px;display:grid;place-items:center;background:rgba(255,255,255,.08)}.ico.outline{background:none;border:1.5px solid var(--text3)}.done .ico{background:rgba(61,220,151,.16);color:var(--good)}.gm{font-size:20px;color:#BDAAFF;background:none}
 .empty-day{color:var(--text3);font-size:13px;margin:12px 4px}
 .left{list-style:none;margin:0;padding:0;display:grid;gap:8px;font-size:14px}.left li{display:flex;flex-wrap:wrap;gap:8px;align-items:center}.left li span{flex:1 1 10em;min-width:0;overflow-wrap:anywhere}.left i{flex:none;width:9px;height:9px;border-radius:50%;background:var(--c)}.left .small-edit{white-space:normal;text-align:center}
 .link{background:none;border:0;color:var(--ink);font:600 13px Inter;min-height:40px}
 .event{display:flex;flex-wrap:wrap;gap:10px;align-items:center}.event>span:nth-child(2){flex:1 1 10em;min-width:0;overflow-wrap:anywhere}.event .link{margin-left:auto;white-space:normal;text-align:right}.event b{display:block;font:600 15px Inter}.event small{display:block;font-size:12px;color:var(--text2)}
 .cd{display:grid;place-items:center;line-height:1}.cd b{font-size:20px;color:var(--fair)}.cd small{font-size:12px;color:var(--text2)}
 .none{margin:8px 4px}
</style>
