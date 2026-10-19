<script lang="ts">
 import Icon from '../components/Icon.svelte';
 import DayCells from '../components/DayCells.svelte';
 import WeekSummary from '../components/WeekSummary.svelte';
 import type {PlanBoard} from '../generated/PlanBoard';
 import type {PlanItem} from '../generated/PlanItem';
 import type {LocalAction} from '../generated/LocalAction';
 import {GOALS, SPORT_ICONS, SPORT_NAMES, STATE_WORD, amount} from '../goals';
 import {clock, duration} from '../format';
 // Plan: the week at a glance (rings and one line per goal), the days with
 // what was planned and recorded, and what's left. Garmin's schedule is shown
 // read-only in the same days.
 let {board, navigate, act, header}: {board: PlanBoard, navigate: (route: string) => void, act: (a: LocalAction) => void, header: import('svelte').Snippet} = $props();
 let picked = $state<string | null>(null);
 let selected = $derived(picked && board.days.some(d => d.date === picked) ? picked : board.days.find(d => d.today)?.date ?? board.start);
 let day = $derived(board.days.find(d => d.date === selected)!);
 let items = $derived(board.items.filter(p => p.intention.date === selected && p.state !== 'letGo'));
 let matched = $derived(new Set(board.items.map(p => p.recording).filter(Boolean)));
 let loose = $derived(board.recordings.filter(r => r.date === selected && !matched.has(r.id)));
 let garmin = $derived(board.garmin.filter(g => g.date === selected));
 let review = $derived(board.items.filter(p => p.state === 'review'));
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
  <button class="small-edit" onclick={() => navigate('plan/commitments')}><Icon name="pencil" size={13}/>Goals</button></div>
 <WeekSummary {board} onGoals={() => navigate('plan/commitments')}/>
 {#each review as p}<p class="review"><b>Needs review</b> · {title(p)}, {dayName(p.intention.date)}: {p.candidates.length === 1 ? 'one recording may be this session' : `${p.candidates.length} recordings may be this session`}</p>{/each}
 {#if !board.coverageComplete && board.ledger.length}<p class="small">Garmin’s activity list is incomplete for this week, so some goals are awaiting data.</p>{/if}
 {#if board.readOnly}<p class="small">Some plan records were saved by a newer Raring and can’t be changed here.</p>{/if}
</section>
{#if board.explainObservation}
 <section class="card note"><Icon name="moon" size={18}/><div><b>Sleep timing is now an observation</b><p class="small">Your earlier asleep-on-time target is kept as a reference and compared with each night’s saved plan, without a verdict. Earlier weeks keep their results.</p><button class="secondary" onclick={() => act({action: 'observationExplained'})}>Got it</button></div></section>
{/if}

<div class="sec-h"><h2>Days</h2></div>
<section class="card days">
 <DayCells days={board.days} {selected} select={d => picked = d}/>
 <div class="agenda-h"><h3>{day.today ? 'Today' : dayName(selected, true)}</h3>{#if day.mark}<span class="pill">{({rest: 'Rest day', ill: 'Ill', paused: 'Paused', unknown: 'Awaiting data'} as const)[day.mark]}</span>{/if}</div>
 {#each items as p (p.intention.id)}
  {@const r = recordingOf(p)}
  <button class="row-item" class:done={p.state === 'done'} class:muted={p.state === 'notRecorded' || p.state === 'missed' || p.state === 'awaiting'} disabled={!r} onclick={() => r && navigate('workout/' + r.id)}>
   <span class="ico" class:outline={p.state !== 'done'}><Icon name={p.state === 'done' ? 'check' : SPORT_ICONS[p.intention.sport]} size={18}/></span>
   <span><small>{STATE_WORD[p.state]}{p.matchedBy === 'automatic' ? ' · matched by Raring' : p.matchedBy === 'linked' ? ' · linked by you' : ''}</small><b>{title(p)}</b><em>{r ? `${clock(r.startLocal)} · ${duration(r.durationMin)}` : when(p)}{goalsOf(p) ? ' · ' + goalsOf(p) : ''}</em></span>
   {#if r}<Icon name="chevron" size={15}/>{/if}
  </button>
 {/each}
 {#each loose as r (r.id)}
  <button class="row-item done" onclick={() => navigate('workout/' + r.id)}><span class="ico"><Icon name={r.icon} size={18}/></span><span><small>Recorded · not in your plan</small><b>{r.name}</b><em>{clock(r.startLocal)} · {duration(r.durationMin)}</em></span><Icon name="chevron" size={15}/></button>
 {/each}
 {#each garmin as g}
  <div class="row-item garmin"><span class="ico gm" aria-hidden="true">◇</span><span><small>Garmin schedule · read-only</small><b>{g.name}</b><em>Not counted toward your goals</em></span></div>
 {/each}
 {#if !items.length && !loose.length && !garmin.length}<p class="empty-day">{day.mark === 'rest' ? 'A day of rest' : 'Nothing on this day'}</p>{/if}
 {#if garminNote}<p class="small">{garminNote}.</p>{/if}
</section>

{#if board.backlogPaused || board.backlog.length}
 <div class="sec-h"><h2>Left this week</h2></div>
 <section class="card">
  {#if board.backlogPaused}<p class="small">Paused · nothing to catch up.</p>
  {:else}<ul class="left">{#each board.backlog as b}<li style:--c={GOALS[b.kind].color}><i></i>{b.kind === 'strength' ? 'Strength session' : `${amount(b.kind, b.amount)} ${GOALS[b.kind].name}`}</li>{/each}</ul>{/if}
 </section>
{/if}
<style>
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
 .left{list-style:none;margin:0;padding:0;display:grid;gap:8px;font-size:14px}.left li{display:flex;gap:8px;align-items:center}.left i{width:9px;height:9px;border-radius:50%;background:var(--c)}
</style>
