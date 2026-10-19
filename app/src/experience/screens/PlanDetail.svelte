<script lang="ts">
 import Header from '../../lib/components/Header.svelte';
 import Icon from '../../lib/components/Icon.svelte';
 import Card from '../../lib/components/Card.svelte';
 import Glyph from '../components/Glyph.svelte';
 import Ring from '../components/Ring.svelte';
 import ActivityTile from '../components/ActivityTile.svelte';
 import CommitmentBars from '../components/CommitmentBars.svelte';
 import {store} from '../lib/store.svelte';
 import {A} from '../lib/actions';
 import {addDays, dayLabel, weekRange, weekId, weekMonday} from '../lib/dates';
 import {SPORTS} from '../lib/sports';
 import {latestReviewWeek, versionFor} from '../lib/rules';
 import {PLAN_WEEKS, SCHEDULE, COACH, activityById} from '../lib/world';
 import {allocationFor, intentionState, isOpen} from '../lib/allocation';
 import MatchPanel from '../components/MatchPanel.svelte';
 import ItemRow from '../components/ItemRow.svelte';
 import {garminItem, itemMatch, localItem, STATE_WORD} from '../lib/match';
 import {commitmentWeeks, totals} from '../lib/progress';
 import {weekOutcome, weekRings, activeKeys} from '../lib/week';
 import {trainingView} from '../lib/views';
 import {goalsFor, GOAL_INFO} from '../lib/allocation';
 import {daysBetween} from '../lib/dates';
 import {rand} from '../lib/world';
 let {route}: {route: string} = $props();
 let ctx = $derived(store.ctx);
 const go = (r: string) => store.navigate(r);
 let [key, sub] = $derived(route.split('/'));
 // race / event goal
 let race = $derived(key === 'plan' && sub === 'race' ? [...ctx.world.ambitions, ...store.state.ambitionsAdded].find(a => a.id === route.split('/')[2]) : undefined);
 let raceLabel = $derived(race ? (/half/i.test(race.name) ? 'Half' : /marathon/i.test(race.name) ? 'Full' : /5k/i.test(race.name) ? '5K' : '10K') : '10K');
 let prediction = $derived(race ? trainingView(ctx).races.find(r => r.label === raceLabel) ?? null : null);
 // Prediction history is not stored today (see provenance); a synthetic weekly series stands in.
 let predSeries = $derived(prediction ? Array.from({length: 10}, (_, i) => ({from: addDays(ctx.weekFrom, -7 * (9 - i)), s: Math.round(prediction!.seconds + (9 - i) * 9 + (rand(raceLabel + i) - .5) * 30)})) : []);
 let raceWeeks = $derived(race ? [...Array.from({length: 4}, (_, i) => weekOutcome(ctx, addDays(ctx.weekFrom, -7 * (4 - i)))), null] : []);
 let g = $derived(goalsFor(ctx)), keys = $derived(activeKeys(g).filter(k => k !== 'sleep'));
 const mmss = (s: number) => `${Math.floor(s / 60)}:${String(Math.round(s % 60)).padStart(2, '0')}`;
 let planProfile = $derived(['mixed', 'sparse'].includes(store.state.profile));
 // intention
 let i = $derived(key === 'intention' ? store.state.intentions.find(x => x.id === sub) : undefined);
 let candidates = $derived(i ? ctx.world.activities.filter(a => a.date === i!.date && a.sport === i!.sport) : []);
 let linked = $derived(i?.link ? activityById(i.link) : undefined);
 // scheduled
 let s = $derived(key === 'scheduled' ? SCHEDULE.find(x => x.id === sub)! : undefined);
 // review
 let wkId = $derived(key === 'review' ? sub : '');
 let week = $derived(PLAN_WEEKS.find(w => w.id === wkId) ?? (wkId === weekId(ctx.calendarDate) || wkId === '2026-W47' ? {id: wkId, from: ctx.weekFrom, state: 'current', label: 'In progress', detail: ''} : null));
 let wkTotals = $derived(week ? totals(ctx, week.from, addDays(week.from, 6) > ctx.calendarDate ? ctx.calendarDate : addDays(week.from, 6)) : null);
 let wkActs = $derived(week ? ctx.world.activities.filter(a => a.date >= week!.from && a.date <= addDays(week!.from, 6)) : []);
 let versions = $derived(versionFor(ctx));
 const stateRing = (w: {state: string}, have: number, target: number) => w.state === 'paused' ? 'paused' : w.state === 'unknown' ? 'unknown' : have >= target ? 'reached' : 'open';
</script>

{#snippet header(title: string, eyebrow = '')}<Header {title} {eyebrow} back={() => store.back()} backLabel={store.parentLabel}/>{/snippet}

{#if key === 'intention'}
 {#if !i}{@render header('Your plan')}<Card><p>This intention was removed.</p></Card>
 {:else}
  {@render header(`${SPORTS[i.sport].name} · ${dayLabel(i.date)}`, `Your plan${i.time ? ` · ${i.time}` : ' · any time'}`)}
  {@const st = intentionState(ctx, i)}{@const alloc = allocationFor(i)}
  <section class="card ihero" style:--c={SPORTS[i.sport].color}>
   <Glyph sport={i.sport} mark={st === 'recorded' ? 'recorded' : 'plan'} size={52}/>
   <div><b>{i.name ?? SPORTS[i.sport].name}{i.time ? ` at ${i.time}` : ''}</b><small>Your plan{i.garminId ? ' · counts a Garmin session' : ''}</small>
    <span class="status s-{st}">{st === 'recorded' ? '✓ Done' : STATE_WORD[st]}</span></div>
  </section>
  {#if Object.values(alloc).some(Boolean)}<p class="alloc">Counts toward: {Object.entries(alloc).filter(([, n]) => n).map(([k, n]) => `${n} ${({strength: 'strength session', zone2: 'Zone 2 min', hi: 'Zone 4–5 min', im: 'intensity min', sleep: 'sleep-timing night'} as Record<string,string>)[k]}`).join(' · ')}</p>{/if}
  {#if st === 'review' || st === 'not-recorded' || st === 'awaiting'}<div class="btns"><button class="secondary" onclick={() => store.openSheet('reschedule', i!.id)}>Reschedule</button><button class="secondary" onclick={() => A.finishIntention(i!.id, 'released')}>Let it go</button>{#if st !== 'review'}<button class="secondary" onclick={() => A.finishIntention(i!.id, 'missed')}>Mark as missed</button>{/if}</div>{/if}
  {#if store.state.limits.exclude.includes(i.sport)}<div class="notice-strip"><Icon name="alert" size={16}/><p>You excluded {SPORTS[i.sport].name.toLowerCase()} in your limits</p></div>{/if}
  {#if store.state.limits.note}<div class="notice-strip"><Icon name="alert" size={16}/><p>Review needed · your movement note applies (“{store.state.limits.note}”)</p></div>{/if}
  <div class="btns"><button class="secondary" onclick={() => store.openSheet('add', i!.id)}>Edit</button><button class="secondary" onclick={() => A.removeIntention(i!.id)}>Remove</button></div>
  {#if !i.garminId}<button class="send" disabled aria-describedby="send-n"><span aria-hidden="true">◇</span>Send to Garmin</button><p class="small" id="send-n">Not available yet: Raring can’t write to your Garmin calendar or watch.</p>{/if}
  {#if i.kind !== 'sleep'}<MatchPanel p={localItem(i)}/>{/if}
  <div class="sec-h"><h2>History</h2></div>
  <section class="card"><ol class="hist">{#each i.history as [d, t]}<li><b>{d}</b><span>{t}</span></li>{/each}</ol></section>
 {/if}

{:else if key === 'scheduled' && s}
 {@const conflict = store.state.limits.exclude.includes(s.sport)}{@const review = !!store.state.limits.note}
 {@render header(s.name, `${dayLabel(s.date)} · Garmin schedule`)}
 {@const gi = garminItem(s)}{@const gm = itemMatch(ctx, gi)}
 <section class="card ihero garmin" style:--c={SPORTS[s.sport].color}><Glyph sport={s.sport} mark={gm.state === 'done' ? 'recorded' : 'garmin'} size={52}/><div><b>{s.name}</b><small>Garmin schedule · read-only · {s.min} min · {SPORTS[s.sport].name}</small><span class="status s-{gm.state === 'done' ? 'recorded' : gm.state}">{gm.state === 'done' ? '✓ Done' : STATE_WORD[gm.state]}</span></div></section>
 <section class="card"><dl class="kv"><dt>Steps</dt><dd>{s.steps}</dd><dt>Plans toward</dt><dd>{Object.entries(s.plan).filter(([, n]) => n).map(([k, n]) => `${n} ${({strength: 'strength session', zone2: 'Zone 2 min', hi: 'Zone 4–5 min', im: 'intensity min'} as Record<string, string>)[k]}`).join(' · ') || '—'}</dd><dt>Managed in</dt><dd>Garmin Connect</dd><dt>Garmin’s completion flag</dt><dd>Not read · unverified</dd><dt>Active Garmin Coach plan</dt><dd>{COACH.name}</dd><dt>Schedule checked</dt><dd>18 Nov 07:12</dd></dl></section>
 <MatchPanel p={gi}/>
 {#if conflict}<div class="notice-strip"><Icon name="alert" size={16}/><p>Conflicts with your excluded activity · stays on your Garmin calendar</p></div>{/if}
 {#if review}<div class="notice-strip"><Icon name="alert" size={16}/><p>Review needed · Raring can’t see this workout’s exercises{store.state.limits.routines[s.id] === 'chosen' ? ' · you chose to keep using it' : ''}</p></div>{/if}
 {#if ctx.paused}<p class="small">Training is paused for you.</p>{/if}
 {#if s.date === ctx.calendarDate && ctx.readiness}<section class="card rrow"><Ring value={ctx.readiness.value} target={100} size={52} stroke={5} color={ctx.readiness.value >= 75 ? '#3DDC97' : ctx.readiness.value >= 50 ? '#FFB547' : '#FF6B6B'} label={`Readiness ${ctx.readiness.value}`}/><div><b>Readiness {ctx.readiness.level}</b><small>{ctx.readiness.value < 50 ? 'An easier day may fit' : 'Today, Garmin'}</small></div></section>{/if}
 <div class="btns">
  {#if s.date === ctx.calendarDate && !ctx.restActive}<button class="secondary" onclick={A.rest}>Rest today</button>{/if}
  {#if s.date === ctx.calendarDate}<button class="secondary" onclick={() => A.dismiss(`sched:${s.id}:${ctx.calendarDate}`)}>Not today</button>{/if}
  {#if review && store.state.limits.routines[s.id] !== 'chosen'}<button class="secondary" onclick={() => A.chooseRoutine(s.id)}>I intend to use this</button>{/if}
 </div>

{:else if route === 'plan/garmin'}
 {@render header(COACH.name, 'Garmin Coach · read-only')}
 <section class="card ihero garmin" style:--c="#A48BFF"><span class="coach-d" aria-hidden="true">◇</span><div><b>{COACH.name}</b><small>Garmin Coach · since {dayLabel(COACH.since)}</small><span class="status">Completion unknown</span></div></section>
 <section class="card"><dl class="kv"><dt>Managed in</dt><dd>Garmin Connect</dd><dt>Schedule checked</dt><dd>{store.state.calendar === 'stale' ? 'Mon 16 Nov' : '18 Nov 07:12'}</dd><dt>Covers</dt><dd>{store.state.calendar === 'partial' ? '18 Nov onward · one page' : '18 Nov–1 Dec'}</dd></dl>
  {#if store.state.calendar === 'partial'}<p class="small">Garmin returned a full page; later sessions may be missing.</p>{:else if store.state.calendar === 'stale'}<p class="small">May be out of date until the next sync.</p>{/if}</section>
 <div class="sec-h"><h2>Sessions</h2></div>
 {#each ctx.world.schedule as g}<ItemRow p={garminItem(g)} showDay actions={false}/>{/each}

{:else if race}
 {@render header(race.name, `${race.date ? dayLabel(race.date) : 'No date'} · ${race.source}`)}
 <section class="card race-hero">{#if race.date}<div class="cdown"><b class="n">{daysBetween(ctx.calendarDate, race.date)}</b><small>days to go · {Math.ceil(daysBetween(ctx.calendarDate, race.date) / 7)} weeks</small></div>{/if}
  {#if prediction}<div class="pred"><small>Garmin’s {raceLabel} prediction</small><b class="n">{mmss(prediction.seconds)}</b></div>{/if}</section>
 {#if prediction}
  {@const lo = Math.min(...predSeries.map(p => p.s)) - 20}{@const hi = Math.max(...predSeries.map(p => p.s)) + 20}
  <section class="card"><h3>{raceLabel} prediction, last 10 weeks</h3>
   <svg class="pchart" viewBox="0 0 320 130" role="img" aria-label={`${raceLabel} prediction from ${mmss(predSeries[0].s)} to ${mmss(predSeries.at(-1)!.s)}`}>
    {#each [lo, (lo + hi) / 2, hi] as v}<line x1="44" x2="316" y1={10 + (v - lo) / (hi - lo) * 96} y2={10 + (v - lo) / (hi - lo) * 96} stroke="rgba(255,255,255,.08)"/><text x="40" y={14 + (v - lo) / (hi - lo) * 96} text-anchor="end">{mmss(v)}</text>{/each}
    {#each predSeries as p, i}<circle cx={52 + i * 29} cy={10 + (p.s - lo) / (hi - lo) * 96} r="3.5" fill="#FFB547"/>{/each}
    <text x="52" y="126">{dayLabel(predSeries[0].from).slice(4)}</text><text x="313" y="126" text-anchor="end">{dayLabel(predSeries.at(-1)!.from).slice(4)}</text>
   </svg><p class="small">Lower is faster. Weekly Garmin estimates; nothing drawn between them.</p></section>
 {/if}
 <section class="card"><h3>Your weeks toward it</h3>
  {#if keys.length}<table class="rweeks"><thead><tr><th>Week</th>{#each keys as k}<th style:color={GOAL_INFO[k].color}>{GOAL_INFO[k].name}</th>{/each}</tr></thead><tbody>
   {#each raceWeeks as w}{@const cur = w ? null : weekRings(ctx)}<tr><th>{w ? weekRange(w.from).replace(/ \w+$/, '') : 'This week'}</th>{#each keys as k}{@const o = w ? w.goals.find(x => x.key === k) : cur!.rings.find(x => x.key === k)}<td>{#if w && !w.counted}<span class="nc">{w.reasons[0] ?? '—'}</span>{:else if o}<b class="n">{o.recorded}</b>/{w ? (o as {target: number}).target : g[k]}{:else}—{/if}</td>{/each}</tr>{/each}
  </tbody></table>{:else}<p class="small">Choose weekly goals in Plan to see how each week feeds this race.</p>{/if}
 </section>
 {#if ctx.world.schedule.length}<section class="card"><h3>Garmin Coach · {COACH.name}</h3>{#each ctx.world.schedule.filter(s => s.date >= ctx.weekFrom && s.date <= ctx.weekTo) as s}<button class="rowlink" onclick={() => go(`scheduled/${s.id}`)}><Glyph sport={s.sport} mark="garmin" size={26}/><span><b>{s.name}</b><small>{dayLabel(s.date)} · completion unknown</small></span><Icon name="chevron" size={16}/></button>{/each}</section>{/if}

{:else if route === 'plan/commitments'}
 {@render header('Commitments', `Week of ${dayLabel(ctx.weekFrom)}`)}
 {#if !ctx.plan}
  <Card title="Add a commitment"><p class="small">A quantity and a period. Everything else is optional.</p>
   <div class="pick">{#each [1, 2, 3] as n}<button class="pk" onclick={() => A.ownCommit(n)}><Ring value={n} target={n} size={54} stroke={5} color={SPORTS.strength.color} state="open" label={`${n} strength sessions a week`}><b class="n">{n}</b></Ring><span>strength / week</span></button>{/each}</div>
  </Card>
 {:else}
  {@const p = ctx.plan}
  <Card title="This week">
   <div class="rings">{#each p.tracks as t}<div class="cr"><Ring value={t.have} target={t.target} size={70} stroke={7} state={t.state} color={t.key === 'strength' ? SPORTS.strength.color : '#36C9C6'} label={`${t.key}: ${t.label}`}/><b>{t.key === 'strength' ? 'Strength' : 'Intensity'}</b><small>{t.label}</small></div>{/each}</div>
   <div class="step"><span>Strength sessions</span><button class="iconbtn" aria-label="One fewer strength session from today" onclick={() => A.revise('strength', -1)}>−</button><b class="n">{p.target.strength}</b><button class="iconbtn" aria-label="One more strength session from today" onclick={() => A.revise('strength', 1)}>+</button></div>
   <div class="step"><span>Intensity minutes</span><button class="iconbtn" aria-label="30 fewer minutes from today" onclick={() => A.revise('im', -30)}>−</button><b class="n">{p.target.im}</b><button class="iconbtn" aria-label="30 more minutes from today" onclick={() => A.revise('im', 30)}>+</button></div>
   <p class="small">Changes apply from {dayLabel(ctx.calendarDate)}; earlier weeks keep their targets.</p>
   <button class="link" onclick={() => go('adapt/lighter')}>Busy or travelling?</button>
  </Card>
  {#if planProfile}
   <Card title="Over time"><CommitmentBars weeks={commitmentWeeks(ctx, 'strength')} name="Strength sessions" color={SPORTS.strength.color} onWeek={id => go(`review/${id}`)}/><CommitmentBars weeks={commitmentWeeks(ctx, 'im')} name="Intensity minutes" unit=" min" color="#36C9C6" onWeek={id => go(`review/${id}`)}/></Card>
  {/if}
  <Card title="Versions"><ol class="hist">
   {#each versions.versions.filter(v => v.from <= p.wk) as v}<li><b>From {dayLabel(weekMonday(v.from))}</b><span>{v.strength} strength · {v.im} min · {v.note}</span></li>{/each}
   {#each p.revisions as r}<li><b>{dayLabel(r.from)}</b><span>{r.strength} strength · {r.im} min · this week from then</span></li>{/each}
   {#if p.pause}<li><b>{dayLabel(p.pause.since)}{p.pause.until ? `–${dayLabel(p.pause.until)}` : ''}</b><span>Paused</span></li>{/if}
  </ol></Card>
  <Card title="Zone 2 and sleep timing"><p class="small">Zone 2 uses recorded heart-rate zones, apart from Garmin’s intensity minutes. Sleep timing is an observation.</p><button class="link" onclick={() => go('sleep/timing')}>Sleep timing</button></Card>
 {/if}

{:else if key === 'review'}
 {#if !week}{@render header('Week review')}<Card><p>No review exists for that week.</p></Card>
 {:else}
  {@const current = week.state === 'current'}
  {@render header(`Week ${wkId.slice(6)}`, weekRange(week.from))}
  <section class="card rv-hero">
   <div class="rvr">
    {#if current && ctx.plan}{#each ctx.plan.tracks as t}<Ring value={t.have} target={t.target} size={64} stroke={6} state={t.state} color={t.key === 'strength' ? SPORTS.strength.color : '#36C9C6'} label={`${t.key}: ${t.label}`}/>{/each}
    {:else if planProfile && week.strength && week.im}<Ring value={week.strength[0]} target={week.strength[1]} size={64} stroke={6} state={stateRing(week, week.strength[0], week.strength[1])} color={SPORTS.strength.color} label={`Strength ${week.strength[0]} of ${week.strength[1]}`}/><Ring value={week.im[0]} target={week.im[1]} size={64} stroke={6} state={stateRing(week, week.im[0], week.im[1])} color="#36C9C6" label={`Intensity ${week.im[0]} of ${week.im[1]}`}/>
    {:else}<Ring value={0} target={1} size={64} stroke={6} state="none" center={false} label="No plan that week"/>{/if}
   </div>
   <div class="rvs"><div><b class="n">{wkTotals?.activities ?? 0}</b><small>activities</small></div><div><b class="n">{wkTotals?.im ?? 0}</b><small>intensity min</small></div><div><b class="n">{wkTotals ? Math.round(wkTotals.minutes / 6) / 10 : 0}</b><small>hours</small></div></div>
   <p class="small">{current ? 'In progress' : planProfile ? week.label : 'No accepted commitments'}{week.version && planProfile ? ` · targets then: ${week.version}` : ''}</p>
  </section>
  <Card title="The win"><p>{current ? (wkActs.some(a => a.record) ? 'Monday’s strength session, with a new bench press best.' : wkActs.length ? `${wkActs.length} activities recorded.` : 'Your recorded data, with or without targets.') : week.state === 'reached' ? 'Every commitment met against that week’s targets.' : week.state === 'paused' ? `${wkActs.length} activities during a paused week.` : week.state === 'unknown' ? `${wkActs.length} known workouts.` : week.state === 'none' ? `${wkActs.length} recorded activities.` : week.detail}</p></Card>
  <Card title="An observation"><p>{current ? 'One short night (5h 10) on Tuesday; Garmin rated Wednesday’s readiness Low.' : week.state === 'unknown' ? 'Garmin’s activity list was incomplete; this week awaits evidence.' : 'Bed times varied by 40 min; Garmin rated 5 of 7 nights aligned.'}</p></Card>
  {#if wkActs.length}<div class="sec-h"><h2>Recorded</h2></div>{#each wkActs.slice(-3).reverse() as a}<ActivityTile {a} {ctx} navigate={go} showDate/>{/each}{/if}
  <Card title="A choice for next week"><div class="btns"><button class="secondary" onclick={A.keep}>Keep commitments</button><button class="secondary" onclick={() => go('plan/commitments')}>Change</button><button class="secondary" onclick={() => go('adapt/lighter')}>Lighter week</button></div></Card>
 {/if}

{:else if key === 'reviews'}
 {@render header('Weekly reviews', 'On this phone')}
 <section class="card">
  {#each [...(latestReviewWeek(ctx) === weekId(ctx.calendarDate) ? [{id: weekId(ctx.calendarDate), from: ctx.weekFrom, state: 'current', label: 'In progress'}] : []), ...[...PLAN_WEEKS].reverse()] as w}
   <button class="rowlink" onclick={() => go(`review/${w.id}`)}><Ring value={'strength' in w && w.strength ? w.strength[0] : 0} target={'strength' in w && w.strength ? w.strength[1] : 1} size={34} stroke={4} center={false} state={planProfile ? (w.state === 'partial' ? 'open' : w.state) : 'none'} label={w.label}/><span><b>Week {w.id.slice(6)} · {weekRange(w.from)}</b><small>{planProfile ? w.label : 'No accepted commitments'}</small></span><Icon name="chevron" size={16}/></button>
  {/each}
 </section>
{/if}

<style>
 .ihero{display:flex;align-items:center;gap:14px;background:linear-gradient(120deg,color-mix(in srgb,var(--c) 14%,transparent),var(--card))}
 .ihero b{display:block;font:600 19px Outfit}.ihero small{display:block;color:var(--text2);font-size:12px;margin-top:2px}
 .status{display:inline-block;margin-top:8px;padding:3px 9px;border-radius:999px;font:600 11px Inter;background:rgba(255,255,255,.07);color:var(--text2)}.s-recorded{color:#3DDC97}.s-review{color:#FFB547}.s-recorded,.s-done{color:#3DDC97}.s-not-recorded,.s-awaiting{border:1px dashed rgba(228,232,240,.3);background:none}.alloc{margin:8px 4px;font-size:13px;color:var(--text2)}.ihero.garmin{border-color:rgba(164,139,255,.3)}
 .coach-d{font-size:44px;color:#BDAAFF;line-height:1}.gsess{padding:10px 14px}
 .race-hero{display:flex;justify-content:space-between;align-items:flex-end;gap:12px;background:linear-gradient(120deg,rgba(255,181,71,.12),var(--card))}.cdown b{display:block;font-size:48px;font-weight:600;line-height:1;color:#FFB547}.cdown small,.pred small{font-size:12px;color:var(--text2)}.pred{text-align:right}.pred b{display:block;font-size:28px;font-weight:600}
 .pchart{width:100%;height:auto}.pchart text{font:500 10px Inter;fill:var(--text3)}
 .rweeks{width:100%;border-collapse:collapse;font-size:12.5px}.rweeks th,.rweeks td{padding:7px 4px;border-bottom:1px solid var(--line);text-align:right}.rweeks thead th{font:600 10.5px Inter}.rweeks tbody th{text-align:left;font-weight:500;color:var(--text2)}.rweeks .nc{font-size:10.5px;color:var(--text3)}
 .send{display:flex;gap:8px;align-items:center;justify-content:center;width:100%;margin-top:10px;min-height:42px;border-radius:13px;border:1px dashed rgba(164,139,255,.4);background:none;color:var(--text3);font:600 13px Inter}.send:disabled{cursor:not-allowed}
 .hist{list-style:none;padding:0;margin:0;display:grid;gap:8px}
 .hist li{display:grid;grid-template-columns:118px 1fr;gap:10px;font-size:13px}
 .hist span{color:var(--text2)}
 .rrow{display:flex;align-items:center;gap:12px}
 .rrow b{display:block}.rrow small{color:var(--text2);font-size:12px}
 .pick{display:grid;grid-template-columns:repeat(3,1fr);gap:10px;margin-top:12px}
 .pk{display:flex;flex-direction:column;align-items:center;gap:6px;padding:12px 4px;border-radius:16px;border:1px solid var(--line);background:rgba(255,255,255,.03);color:var(--text2);font-size:12px}
 .rings{display:grid;grid-template-columns:1fr 1fr;gap:10px;margin-bottom:14px}
 .cr{display:flex;flex-direction:column;align-items:center;gap:3px}
 .cr b{margin-top:6px;font-weight:600}.cr small{color:var(--text2);font-size:12px}
 .step{display:grid;grid-template-columns:1fr 40px 44px 40px;align-items:center;gap:6px;padding:8px 0;border-top:1px solid var(--line)}
 .step b{text-align:center;font-size:20px}
 .rv-hero{display:grid;gap:12px}
 .rvr{display:flex;gap:10px}
 .rvs{display:grid;grid-template-columns:repeat(3,1fr);gap:8px}
 .rvs b{display:block;font-size:24px;font-weight:600}.rvs small{color:var(--text3);font-size:11px}
</style>
