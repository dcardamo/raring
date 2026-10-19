<script lang="ts">
 import Icon from '../components/Icon.svelte';
 import DayCells from '../components/DayCells.svelte';
 import HelpButton from '../components/HelpButton.svelte';
 import WeekSummary from '../components/WeekSummary.svelte';
 import type {PlanBoard} from '../generated/PlanBoard';
 import type {PlanItem} from '../generated/PlanItem';
 import type {LocalAction} from '../generated/LocalAction';
 import {GOALS, backlogDraft, SPORT_ICONS, SPORT_NAMES, STATE_WORD, amount} from '../goals';
 import {clock, duration} from '../format';
 // Plan: the week at a glance (rings and one line per goal), the days with
 // what was planned and recorded, and what's left. Garmin's schedule is shown
 // read-only in the same days.
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
 const dateName = (d: string) => new Date(d + 'T12:00:00Z').toLocaleDateString('en-GB', {weekday: 'short', day: 'numeric', month: 'short', year: 'numeric', timeZone: 'UTC'});
 const dayName = (d: string, long = false) => new Date(d + 'T12:00:00Z').toLocaleDateString('en-GB', {weekday: long ? 'long' : 'short', day: 'numeric', month: 'short', timeZone: 'UTC'});
 let range = $derived(`${dayName(board.start)} – ${dayName(board.end)}`);
 const recordingOf = (p: PlanItem) => board.recordings.find(r => r.id === p.recording);
 const title = (p: PlanItem) => p.intention.name ?? SPORT_NAMES[p.intention.sport];
 const when = (p: PlanItem) => [p.intention.time, p.intention.durationMin ? `${p.intention.durationMin} min` : null].filter(Boolean).join(' · ') || 'Any time';
 const goalsOf = (p: PlanItem) => (Object.entries(p.intention.allocation) as [string, number][]).filter(([, v]) => v > 0)
  .map(([k, v]) => ({strengthSessions: amount('strength', v), zone2Min: amount('zone2', v) + ' Zone 2', highIntensityMin: amount('highIntensity', v) + ' Zone 4–5', intensityMin: amount('intensity', v) + ' intensity', sleepNights: 'sleep timing'} as Record<string, string>)[k]).join(' · ');
 const garminNote = $derived(board.garminState === 'partial' ? 'Garmin’s schedule covers only part of this week' : board.garminState === 'stale' ? 'Garmin’s schedule may be out of date' : board.garminState === 'unknown' ? 'Garmin’s schedule isn’t available' : null);
</script>
{@render header()}
<section class="card wk">
 <div class="wk-h"><div><h2>{range}</h2>{#if board.pause}<small>{board.pause.reason === 'ill' ? 'Ill' : 'Paused'} {board.pause.wholeWeek ? 'all week' : `since ${dayName(board.pause.since)}`}{board.pause.until && !board.pause.wholeWeek ? ` to ${dayName(board.pause.until)}` : ''} · nothing to catch up</small>{/if}</div>
  <span class="wk-a"><HelpButton topic="plan"/><button class="small-edit" onclick={() => navigate('plan/commitments')}><Icon name="pencil" size={13}/>Goals</button></span></div>
 <WeekSummary {board} onGoals={() => navigate('plan/commitments')}/>
 {#if ev}
  <div class="evb" role="group" aria-label={`${ev.event.name} this week`}>
   <div class="ev-h"><span class="cd"><b class="n">{ev.event.daysUntil}</b><small>days</small></span><span><b>{ev.event.name}</b><small>{dateName(ev.event.date)} · {SPORT_NAMES[ev.sport]} this week: {Math.round(ev.recordedMin)} min recorded</small></span></div>
   {#each board.items.filter(p => ev!.items.includes(p.intention.id)) as p (p.intention.id)}
    <button class="ev-row" class:done={p.state === 'done'} onclick={() => openItem(p)}><Icon name={p.state === 'done' ? 'check' : SPORT_ICONS[p.intention.sport]} size={15}/><span>{dayName(p.intention.date)} · {title(p)}</span><small>{STATE_WORD[p.state]}</small></button>
   {/each}
   <p class="small">Each session also counts toward your weekly goals, once per goal.</p>
  </div>
 {/if}
 {#if board.coach && coachItems.length}
  <div class="evb coach" role="group" aria-label={`${board.coach.name} this week`}>
   <div class="ev-h"><span class="cd gm" aria-hidden="true">◇</span><span><b>{board.coach.name}</b><small>Garmin Coach · read-only{board.coach.state === 'stale' ? ' · may be out of date' : ''} · {coachItems.filter(p => p.state === 'done').length} of {coachItems.length} done</small></span></div>
   {#each coachItems as p (p.intention.id)}
    <button class="ev-row" class:done={p.state === 'done'} onclick={() => openItem(p)}><Icon name={p.state === 'done' ? 'check' : SPORT_ICONS[p.intention.sport]} size={15}/><span>{dayName(p.intention.date)} · {title(p)}{p.intention.durationMin ? ` · ${p.intention.durationMin} min` : ''}</span><small>{STATE_WORD[p.state]}</small></button>
   {/each}
   <p class="small">Strength sessions count as planned toward your strength goal; cardio sessions plan no zone or intensity minutes. Done means a matching recording, not Garmin’s status.</p>
  </div>
 {/if}
 {#each review as p}<p class="review"><b>Needs review</b> · {title(p)}, {dayName(p.intention.date)}: {p.candidates.length === 1 ? 'one recording may be this session' : `${p.candidates.length} recordings may be this session`}</p>{/each}
 {#if !board.coverageComplete && board.ledger.length}<p class="small">Garmin’s activity list is incomplete for this week, so some goals are awaiting data.</p>{/if}
 {#if board.readOnly}<p class="small">Some plan records were saved by a newer Raring and can’t be changed here.</p>{/if}
</section>
{#if board.explainObservation}
 <section class="card note"><Icon name="moon" size={18}/><div><b>Sleep timing is now an observation</b><p class="small">Your earlier asleep-on-time target is kept as a reference and compared with each night’s saved plan, without a verdict. Earlier weeks keep their results.</p><button class="secondary" onclick={() => navigate('sleep/tonight')}>Review timing</button><button class="secondary" onclick={() => act({action: 'observationExplained'})}>Got it</button></div></section>
{/if}

<div class="sec-h"><h2>Days</h2><HelpButton topic="rings"/></div>
<section class="card days">
 <DayCells days={board.days} {selected} select={d => picked = d}/>
 <div class="agenda-h"><h3>{day.today ? 'Today' : dayName(selected, true)}</h3>{#if day.mark}<span class="pill">{({rest: 'Rest day', ill: 'Ill', paused: 'Paused', unknown: 'Awaiting data'} as const)[day.mark]}</span>{/if}</div>
 {#each items as p (p.intention.id)}
  {@const r = recordingOf(p)}
  <button class="row-item" class:serves={ev?.items.includes(p.intention.id)} class:done={p.state === 'done'} class:muted={p.state === 'notRecorded' || p.state === 'missed' || p.state === 'awaiting'} onclick={() => openItem(p)}>
   <span class="ico" class:outline={p.state !== 'done'}><Icon name={p.state === 'done' ? 'check' : SPORT_ICONS[p.intention.sport]} size={18}/></span>
   <span><small>{p.coach ? 'Garmin Coach · ' : ''}{p.coach && limited && p.state === 'planned' ? 'Review needed · ' : ''}{STATE_WORD[p.state]}{p.matchedBy === 'automatic' ? ' · matched by Raring' : p.matchedBy === 'linked' ? ' · linked by you' : ''}</small><b>{title(p)}</b>{#if ev?.items.includes(p.intention.id)}<i class="for">For {ev.event.name}</i>{/if}<em>{r ? `${clock(r.startLocal)} · ${duration(r.durationMin)}` : when(p)}{goalsOf(p) ? ' · ' + goalsOf(p) : ''}</em></span>
   <Icon name="chevron" size={15}/>
  </button>
 {/each}
 {#each loose as r (r.id)}
  <button class="row-item done" onclick={() => navigate('workout/' + r.id)}><span class="ico"><Icon name={r.icon} size={18}/></span><span><small>Recorded · not in your plan</small><b>{r.name}</b><em>{clock(r.startLocal)} · {duration(r.durationMin)}</em></span><Icon name="chevron" size={15}/></button>
 {/each}
 {#each coachRest as r (r.id)}<div class="row-item garmin"><span class="ico gm" aria-hidden="true">◇</span><span><small>Garmin Coach · read-only</small><b>Rest day</b><em>{board.coach?.name}</em></span></div>{/each}
 {#each garmin as g}
  <button class="row-item garmin" onclick={() => openGarmin?.(g)}><span class="ico gm" aria-hidden="true">◇</span><span><small>Garmin schedule · read-only{limited ? ' · Review needed' : ''}</small><b>{g.name}</b><em>{limited ? 'Contents unknown to Raring' : 'Not counted toward your goals'}</em></span><Icon name="chevron" size={15}/></button>
 {/each}
 {#if !items.length && !loose.length && !garmin.length && !coachRest.length}<p class="empty-day">{day.mark === 'rest' ? 'A day of rest' : 'Nothing on this day'}</p>{/if}
 {#if selected >= board.today}<button class="secondary" onclick={() => openAdd({date: selected})}>Add a session</button>{/if}
 {#if garminNote}<p class="small">{garminNote}.</p>{/if}
</section>

{#if board.backlogPaused || board.backlog.length}
 <div class="sec-h"><h2>Left this week</h2></div>
 <section class="card">
  {#if board.backlogPaused}<p class="small">Paused · nothing to catch up.</p>
  {:else}<ul class="left">{#each board.backlog as b}<li style:--c={GOALS[b.kind].color}><i></i><span>{b.kind === 'strength' ? 'Strength session' : `${amount(b.kind, b.amount)} ${GOALS[b.kind].name}`}</span>{#if b.kind !== 'sleepTiming'}<button class="small-edit" onclick={() => take(b, board.today)}>Do it today</button>{/if}</li>{/each}</ul>{/if}
 </section>
{/if}

<div class="sec-h"><h2>Events</h2><button class="link" onclick={openEvent}>Add a race or event</button></div>
{#each board.events as e}
 <div class="card event"><span class="cd"><b class="n">{e.daysUntil}</b><small>days</small></span><span><b>{e.name}</b><small>{dateName(e.date)} · {e.source === 'garmin' ? `Garmin calendar${e.partial ? ' · later races may be missing' : ''}` : 'Saved on this phone'}</small></span>{#if e.id}<button class="link" aria-label={`Remove ${e.name}`} onclick={() => act({action: 'removeEvent', id: e.id!})}>Remove</button>{/if}</div>
{:else}<p class="small none">No race or event. A date to count down to is optional.</p>{/each}
<style>
 .wk-a{display:flex;gap:8px;align-items:center}
 :global(.sec-h:has(.qh)){display:flex;align-items:center;gap:8px}
 .evb.coach{--ev:#BDAAFF}.cd.gm{font-size:20px}
 .evb{--ev:#FFB547;margin:12px 0 4px;padding:10px 12px;border:1px solid color-mix(in srgb,var(--ev) 35%,transparent);border-radius:16px;background:color-mix(in srgb,var(--ev) 7%,transparent)}
 .ev-h{display:flex;gap:10px;align-items:center}.ev-h b{display:block;font:600 15px Inter}.ev-h small{display:block;font-size:12px;color:var(--text2)}
 .cd{display:grid;place-items:center;min-width:44px;color:var(--ev)}.cd b{font-size:20px}.cd small{font-size:10px;color:var(--text2)}
 .ev-row{display:grid;grid-template-columns:18px minmax(0,1fr) auto;gap:8px;align-items:center;width:100%;min-height:44px;margin-left:12px;padding:4px 0 4px 10px;border:0;border-left:2px solid var(--ev);background:none;color:var(--text);text-align:left;font-size:13px}
 .ev-row small{color:var(--text2);font-size:11.5px;padding-right:12px}.ev-row.done{color:var(--text)}.ev-row.done :global(svg){color:#3DDC97}
 .row-item.serves{box-shadow:inset 3px 0 0 #FFB547}.for{display:block;font:600 11px Inter;font-style:normal;color:#FFB547}
 .wk{padding:14px}.wk-h{display:flex;flex-wrap:wrap;justify-content:space-between;align-items:flex-start;gap:8px;margin-bottom:12px}.wk-h h2{font:600 20px Outfit}.wk-h small{display:block;font-size:11.5px;color:#A9C8FF;margin-top:2px}
 .small-edit{background:rgba(255,255,255,.06);border:1px solid var(--line);border-radius:11px;display:flex;gap:5px;align-items:center;padding:8px 10px;color:var(--text);font:600 12px Inter;min-height:36px}
 .review{margin-top:10px;padding:8px 10px;border:1px dashed rgba(255,181,71,.45);border-radius:12px;background:rgba(255,181,71,.06);color:var(--text2);font-size:12px}.review b{color:var(--fair);font-weight:600}
 .note{display:flex;gap:12px;align-items:flex-start}.note b{font:600 15px Outfit}.note :global(svg){color:#A9C8FF;flex:none;margin-top:2px}
 .sec-h{display:flex;justify-content:space-between;align-items:baseline;margin:22px 4px 0}.sec-h h2{font:600 19px Outfit}
 .days{padding:12px 10px}.agenda-h{display:flex;justify-content:space-between;align-items:center;border-top:1px solid var(--line);margin:14px 2px 4px;padding-top:10px}.agenda-h h3{margin:0;font:600 17px Outfit;color:var(--text);text-transform:none;letter-spacing:0}
 .row-item{display:grid;grid-template-columns:32px 1fr 15px;align-items:center;gap:12px;width:100%;margin-top:8px;padding:10px;border:1px solid var(--line);border-radius:16px;background:rgba(255,255,255,.03);color:var(--text);text-align:left}
 .row-item:disabled{cursor:default}.row-item small{display:block;font:600 11px Inter;color:var(--text3)}.row-item b{display:block;font:600 15px Inter}.row-item em{display:block;font-style:normal;font-size:12px;color:var(--text2);margin-top:2px}
 .row-item.done small{color:var(--good)}.row-item.muted{opacity:.72}.row-item.garmin{border-style:dashed;border-color:rgba(164,139,255,.35)}
 .ico{width:32px;height:32px;border-radius:11px;display:grid;place-items:center;background:rgba(255,255,255,.08)}.ico.outline{background:none;border:1.5px solid var(--text3)}.done .ico{background:rgba(61,220,151,.16);color:var(--good)}.gm{font-size:20px;color:#BDAAFF;background:none}
 .empty-day{color:var(--text3);font-size:13px;margin:12px 4px}
 .left{list-style:none;margin:0;padding:0;display:grid;gap:8px;font-size:14px}.left li{display:grid;grid-template-columns:10px 1fr auto;gap:8px;align-items:center}.left i{width:9px;height:9px;border-radius:50%;background:var(--c)}
 .link{background:none;border:0;color:var(--ink);font:600 13px Inter;min-height:40px}
 .event{display:grid;grid-template-columns:40px 1fr auto;gap:10px;align-items:center}.event b{display:block;font:600 15px Inter}.event small{display:block;font-size:12px;color:var(--text2)}
 .cd{display:grid;place-items:center;line-height:1}.cd b{font-size:20px;color:var(--fair)}.cd small{font-size:10px;color:var(--text2)}
 .none{margin:8px 4px}
</style>
