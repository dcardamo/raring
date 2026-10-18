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
 import {commitmentWeeks, totals} from '../lib/progress';
 let {route}: {route: string} = $props();
 let ctx = $derived(store.ctx);
 const go = (r: string) => store.navigate(r);
 let [key, sub] = $derived(route.split('/'));
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
   <Glyph sport={i.sport} mark={i.link ? 'recorded' : 'plan'} size={52}/>
   <div><b>{i.name ?? SPORTS[i.sport].name}{i.time ? ` at ${i.time}` : ''}</b><small>Your plan{i.garminId ? ' · counts a Garmin session' : ''}</small>
    <span class="status s-{st}">{({recorded:'Recorded · linked by you', released:'Let go', missed:'Marked missed by you', planned:'Planned', review:'Needs review', 'not-recorded':'Not recorded', awaiting:'Awaiting data', 'sleep-observation':'Timing observed in Sleep'} as Record<string,string>)[st]}</span></div>
  </section>
  {#if Object.values(alloc).some(Boolean)}<p class="alloc">Counts toward: {Object.entries(alloc).filter(([, n]) => n).map(([k, n]) => `${n} ${({strength: 'strength session', zone2: 'Zone 2 min', im: 'intensity min', sleep: 'sleep-timing night'} as Record<string,string>)[k]}`).join(' · ')}</p>{/if}
  {#if st === 'review' || st === 'not-recorded' || st === 'awaiting'}<div class="btns"><button class="secondary" onclick={() => store.openSheet('reschedule', i!.id)}>Reschedule</button><button class="secondary" onclick={() => A.finishIntention(i!.id, 'released')}>Let it go</button>{#if st !== 'review'}<button class="secondary" onclick={() => A.finishIntention(i!.id, 'missed')}>Mark as missed</button>{/if}</div>{/if}
  {#if store.state.limits.exclude.includes(i.sport)}<div class="notice-strip"><Icon name="alert" size={16}/><p>You excluded {SPORTS[i.sport].name.toLowerCase()} in your limits</p></div>{/if}
  {#if store.state.limits.note}<div class="notice-strip"><Icon name="alert" size={16}/><p>Review needed · your movement note applies (“{store.state.limits.note}”)</p></div>{/if}
  <div class="btns"><button class="secondary" onclick={() => store.openSheet('add', i!.id)}>Edit</button><button class="secondary" onclick={() => A.removeIntention(i!.id)}>Remove</button></div>
  <div class="sec-h"><h2>Recorded that day</h2></div>
  {#if linked}
   <ActivityTile a={linked} {ctx} navigate={go}/>
   <p class="selfrep"><Icon name="check" size={15}/>Linked by you</p><button class="link" onclick={() => A.unlink(i!.id)}>Unlink</button>
  {:else if candidates.length}
   {#each candidates as a}<ActivityTile {a} {ctx} navigate={go}/><div class="cand"><span class="small">Already counts</span><button class="secondary" onclick={() => A.link(i!.id, a.id)}>This was my session</button></div>{/each}
  {:else}<p class="small">{i.date < ctx.calendarDate ? 'No matching recording.' : 'Nothing recorded yet. Matching recordings appear here.'}</p>{/if}
  <div class="sec-h"><h2>History</h2></div>
  <section class="card"><ol class="hist">{#each i.history as [d, t]}<li><b>{d}</b><span>{t}</span></li>{/each}</ol></section>
 {/if}

{:else if key === 'scheduled' && s}
 {@const conflict = store.state.limits.exclude.includes(s.sport)}{@const review = !!store.state.limits.note}
 {@render header(s.name, `${dayLabel(s.date)} · Garmin schedule`)}
 {@const overlay = store.state.intentions.find(x => x.garminId === s.id && isOpen(x))}
 <section class="card ihero garmin" style:--c={SPORTS[s.sport].color}><Glyph sport={s.sport} mark="garmin" size={52}/><div><b>{s.name}</b><small>Garmin schedule · read-only · {s.min} min · {SPORTS[s.sport].name}</small><span class="status">Completion unknown</span></div></section>
 <section class="card"><dl class="kv"><dt>Managed in</dt><dd>Garmin Connect</dd><dt>Active Garmin Coach plan</dt><dd>{COACH.name}</dd><dt>Schedule checked</dt><dd>18 Nov 07:12</dd></dl></section>
 {#if overlay}<button class="rowlink" onclick={() => go(`intention/${overlay.id}`)}><Glyph sport={overlay.sport} mark="plan" size={30}/><span><b>Counts toward your goals</b><small>Your plan</small></span><Icon name="chevron" size={16}/></button>
 {:else}<button class="secondary" onclick={() => store.openSheet('allocate', 'garmin:' + s.id)}>Count toward my goals</button>{/if}
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
 {#each ctx.world.schedule as g}
  {@const counted = store.state.intentions.find(x => x.garminId === g.id && isOpen(x))}{@const same = ctx.world.activities.filter(a => a.date === g.date && a.sport === g.sport)}
  <section class="card gsess">
   <button class="rowlink" onclick={() => go(`scheduled/${g.id}`)}><Glyph sport={g.sport} mark="garmin" size={30}/><span><b>{g.name}</b><small>{dayLabel(g.date)} · {g.min} min{same.length ? ` · ${SPORTS[g.sport].name.toLowerCase()} recorded that day` : ''}</small></span><Icon name="chevron" size={16}/></button>
   {#if counted}<button class="link" onclick={() => go(`intention/${counted.id}`)}>Counts toward your goals</button>{:else}<button class="link" onclick={() => store.openSheet('allocate', 'garmin:' + g.id)}>Count toward my goals</button>{/if}
  </section>
 {/each}

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
 .status{display:inline-block;margin-top:8px;padding:3px 9px;border-radius:999px;font:600 11px Inter;background:rgba(255,255,255,.07);color:var(--text2)}.s-recorded{color:#3DDC97}.s-review{color:#FFB547}.s-not-recorded,.s-awaiting{border:1px dashed rgba(228,232,240,.3);background:none}.alloc{margin:8px 4px;font-size:13px;color:var(--text2)}.ihero.garmin{border-color:rgba(164,139,255,.3)}
 .coach-d{font-size:44px;color:#BDAAFF;line-height:1}.gsess{padding:10px 14px}.gsess .link{margin-left:42px;font-size:12px}
 .cand{display:flex;align-items:center;justify-content:space-between;gap:10px;margin-top:8px}
 .cand .secondary{width:auto;margin:0;padding:10px 14px}
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
