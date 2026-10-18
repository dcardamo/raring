<script lang="ts">
 import Icon from '../../lib/components/Icon.svelte';
 import BodyMap from '../../lib/components/BodyMap.svelte';
 import RootHeader from '../components/RootHeader.svelte';
 import AxisChart from '../components/AxisChart.svelte';
 import NestedRings from '../components/NestedRings.svelte';
 import HelpButton from '../components/HelpButton.svelte';
 import {weekOutcome, activeKeys, RING_ORDER, type Ring as WRing} from '../lib/week';
 import {goalsFor, recordedFor, GOAL_INFO} from '../lib/allocation';
 import {trainingView, strengthView} from '../lib/views';
 import {restingHrFor, hrvFor, vo2maxFor, fitnessAgeFor, nightFor} from '../lib/world';
 import DayCells from '../components/DayCells.svelte';
 import LiftChart from '../components/LiftChart.svelte';
 import Ring from '../components/Ring.svelte';
 import Glyph from '../components/Glyph.svelte';
 import {store} from '../lib/store.svelte';
 import {PERIODS, buckets, calendarMonth, commitmentWeeks, liftProgress, records, shift, sportMix, strengthWeeks, summary, trendTile, TREND_TILES, type Period} from '../lib/progress';
 import {addDays, addMonths, between, dayLabel, dayMonth, dur, monthName, monthStart, monthYear, range, weekId, weekRange, weekStart} from '../lib/dates';
 import {MILESTONES, PLAN_WEEKS} from '../lib/world';
 import {musclesBetween, sleepView} from '../lib/views';
 import {latestReviewWeek} from '../lib/rules';
 import {SPORTS} from '../lib/sports';
 let ctx = $derived(store.ctx);
 const go = (r: string) => store.navigate(r);
 let period = $derived((store.ui.period as Period) ?? 'month');
 let anchor = $derived((store.ui.anchor as string) ?? ctx.calendarDate);
 let s = $derived(summary(ctx, period, anchor));
 let bars = $derived(buckets(ctx, period, s.from, s.to, s.end));
 let mix = $derived(sportMix(ctx, s.from, s.end));
 let calAnchor = $derived(period === 'week' || period === 'month' ? anchor : s.end);
 let cal = $derived(calendarMonth(ctx, calAnchor));
 let calActive = $derived(cal.days.filter(d => d.covered && d.acts.length).length);
 let calKnown = $derived(cal.days.filter(d => d.covered).length);
 let canPrev = $derived(shift(period, s.from, -1) >= monthStart(ctx.world.historyStart) || shift(period, s.to, -1) >= ctx.world.historyStart);
 let canNext = $derived(!s.current);
 let strengthW = $derived(commitmentWeeks(ctx, 'strength'));
 let imW = $derived(commitmentWeeks(ctx, 'im'));
 let tiles = $derived(TREND_TILES.filter(t => !t.needs || t.needs(ctx)).map(t => ({t, v: trendTile(ctx, t, s.from, s.end, s.prevFrom, s.prevEnd)})).filter(x => x.v.count || x.v.value != null));
 let lifts = $derived(liftProgress(ctx, s.from, s.end));
 let sWeeks = $derived(strengthWeeks(ctx, s.from, s.end));
 let muscles = $derived(musclesBetween(ctx, s.from, s.end));
 let hasStrength = $derived(ctx.world.activities.some(a => a.sport === 'strength'));
 let miles = $derived([...(['mixed', 'sparse'].includes(ctx.state.profile) ? MILESTONES.filter(m => m.date <= ctx.calendarDate && (m.kind !== 'plan' || ctx.state.profile === 'mixed')) : []), ...records(ctx).map(r => ({id: r.id, date: r.date, kind: 'distance', title: r.title, value: r.value, note: 'In available history', route: r.route, sport: r.sport}))].filter((m, i, l) => l.findIndex(x => x.title === m.title) === i).sort((a, b) => b.date.localeCompare(a.date)));
 let highlight = $derived(miles.find(m => m.date >= s.from && m.date <= s.end));
 let nights = $derived.by(() => {
  if (period === 'week' || period === 'month') return range(s.from, s.to).map(d => { const n = d <= ctx.calendarDate && d >= ctx.world.historyStart ? ctx.nightOf(d) : null; return {date: d, future: d > ctx.displayedDay || d < ctx.world.historyStart, asleep: n?.asleep ?? null, need: n?.need ?? null, label: period === 'week' ? 'MTWTFSS'[(new Date(d + 'T12:00:00Z').getUTCDay() + 6) % 7] : String(Number(d.slice(8)))}; });
  const out = [];
  for (let w = weekStart(s.from); w <= s.end; w = addDays(w, period === 'year' ? 14 : 7)) {
   const ns = range(w, addDays(w, period === 'year' ? 13 : 6)).filter(d => d >= s.from && d <= s.end && d <= ctx.displayedDay && d >= ctx.world.historyStart).map(d => ctx.nightOf(d)).filter(n => n != null);
   const avg = (k: 'asleep' | 'need') => ns.length ? Math.round(ns.reduce((t, n) => t + n![k], 0) / ns.length) : null;
   out.push({date: w, future: w > ctx.calendarDate, asleep: avg('asleep'), need: avg('need'), label: period === 'year' ? monthName(w)[0] : dayMonth(w).split(' ')[0]});
  }
  return out;
 });
 let rawNights = $derived(range(s.from, s.end).filter(d => d >= ctx.world.historyStart && d <= ctx.displayedDay && d <= ctx.upload.date).map(d => ctx.nightOf(d)));
 let recorded = $derived(rawNights.filter(n => n != null));
 let missing = $derived(rawNights.length - recorded.length);
 let shortNights = $derived(recorded.filter(n => n.need - n.asleep >= 60).length);
 let avgSleep = $derived(recorded.length ? Math.round(recorded.reduce((t, n) => t + n.asleep, 0) / recorded.length) : null);
 let reviewWeeks = $derived(['mixed', 'sparse'].includes(ctx.state.profile) ? [...PLAN_WEEKS].reverse().slice(0, 4) : []);
 const pct = (a: number, b: number | null | undefined) => b == null ? null : a - b;
 const setP = (p: Period) => store.setUi({period: p, anchor: ctx.calendarDate});
 const move = (n: number) => store.setUi({period, anchor: shift(period, anchor, n)});
 const hours = (m: number) => m >= 600 ? `${Math.round(m / 60)}` : `${Math.floor(m / 60)}:${String(m % 60).padStart(2, '0')}`;
 const delta = (v: number | null, unit = '') => v == null ? '' : Math.abs(v) < 1 ? 'Same' : `${v > 0 ? '+' : '−'}${Math.abs(Math.round(v * 10) / 10)}${unit}`;
 let prevRange = $derived(s.prevFrom.slice(0, 7) === s.prevEnd.slice(0, 7) ? `${Number(s.prevFrom.slice(8))}–${dayMonth(s.prevEnd)}` : `${dayMonth(s.prevFrom)} – ${dayMonth(s.prevEnd)}${s.prevEnd.slice(0, 4) !== s.end.slice(0, 4) ? ' ' + s.prevEnd.slice(0, 4) : ''}`);

 // ---------- P3b: Overall / Cardio / Strength / Long-term ----------
 let tab = $derived((store.ui.section as string) ?? 'overall');
 let weeks12 = $derived(Array.from({length: 12}, (_, i) => addDays(ctx.weekFrom, -7 * (11 - i))).filter(w => w >= weekStart(ctx.world.historyStart)));
 let outcomes = $derived(weeks12.map(w => w === ctx.weekFrom ? null : weekOutcome(ctx, w)));
 const ringsOf = (o: ReturnType<typeof weekOutcome>): WRing[] => o.goals.map(g => ({key: g.key, name: g.name, unit: g.unit, color: GOAL_INFO[g.key].color, recorded: g.recorded, planned: 0, filled: Math.min(1, g.recorded / g.target), outline: 1}));
 const markOf = (o: ReturnType<typeof weekOutcome>) => o.ill >= 2 ? 'ill' as const : o.paused >= 2 ? 'paused' as const : !o.counted ? 'unknown' as const : null;
 let metCount = $derived(outcomes.filter(o => o && o.counted && o.goals.length && o.goals.every(g => g.state === 'met')).length);
 let countedCount = $derived(outcomes.filter(o => o && o.counted && o.goals.length).length);
 let ambitions = $derived([...ctx.world.ambitions, ...store.state.ambitionsAdded].filter(a => a.date));
 let tvw = $derived(trainingView(ctx));
 const weekMark = (w: string, i: number): 'ill' | 'paused' | 'unknown' | undefined => { const o = outcomes[i]; return o ? (markOf(o) ?? undefined) : undefined; };
 const cardioWeek = (key: 'zone2' | 'hi' | 'im') => weeks12.map((w, i) => ({x: dayMonth(w), v: w > ctx.calendarDate ? null : recordedFor(ctx.world.activities.filter(a => between(a.date, w, addDays(w, 6))), key), mark: weekMark(w, i)}));
 let cg = $derived(goalsFor(ctx));
 let cardioKeys = $derived((['zone2', 'hi', 'im'] as const).filter(k => cg[k] > 0 || k === 'zone2'));
 let vo2w = $derived(weeks12.map(w => { let v: number | null = null; for (let d = addDays(w, 6); d >= w && v == null; d = addDays(d, -1)) v = d <= ctx.calendarDate ? vo2maxFor(d) : null; return {x: dayMonth(w), v}; }));
 let strW = $derived(weeks12.map((w, i) => ({x: dayMonth(w), v: w > ctx.calendarDate ? null : recordedFor(ctx.world.activities.filter(a => between(a.date, w, addDays(w, 6))), 'strength'), mark: weekMark(w, i)})));
 let svw = $derived(strengthView(ctx));
 let bests = $derived(svw ? svw.exercises.map(e => ({name: e.name, best: e.bests.filter(b => b.weight != null).sort((a, b) => (b.weight ?? 0) - (a.weight ?? 0))[0]})).filter(x => x.best).slice(0, 6) : []);
 let months = $derived(Array.from({length: 12}, (_, i) => addMonths(monthStart(ctx.calendarDate), i - 11)));
 const monthly = (f: (d: string) => number | null, from: string) => { const vals = range(from, addDays(monthStart(addDays(from, 32)), -1)).filter(d => d <= ctx.displayedDay && d >= ctx.world.historyStart).map(f).filter((v): v is number => v != null); return vals.length ? vals.reduce((a, b) => a + b, 0) / vals.length : null; };
 let hoursM = $derived(months.map(m => ({x: monthName(m).slice(0, 1), v: m < monthStart(ctx.world.historyStart) ? null : Math.round(ctx.world.activities.filter(a => a.date.slice(0, 7) === m.slice(0, 7)).reduce((t, a) => t + a.min, 0) / 6) / 10, mark: ctx.state.pauses.some(p => p.since.slice(0, 7) === m.slice(0, 7)) ? 'paused' as const : ctx.world.coverageGaps.some(g => weekStart(m).slice(0, 7) === m.slice(0, 7) && g === weekId(addDays(m, 14))) ? 'unknown' as const : undefined})));
 let rhrM = $derived(months.map(m => ({x: monthName(m).slice(0, 1), v: monthly(d => nightFor(d) ? restingHrFor(d) : null, m)})));
 let hrvM = $derived(months.map(m => ({x: monthName(m).slice(0, 1), v: monthly(d => nightFor(d) ? hrvFor(d) : null, m)})));
 let vo2M = $derived(months.map(m => ({x: monthName(m).slice(0, 1), v: monthly(vo2maxFor, m)})));
 let faM = $derived(months.map(m => ({x: monthName(m).slice(0, 1), v: monthly(fitnessAgeFor, m)})));
</script>
<RootHeader title="Progress" help="progress-overall"/>
<div class="seg tabs4" role="tablist" aria-label="Progress section">{#each [['overall', 'Overall'], ['cardio', 'Cardio'], ['strength', 'Strength'], ['long', 'Long-term']] as [k, l]}<button role="tab" aria-selected={tab === k} onclick={() => store.setUi({section: k})}>{l}</button>{/each}</div>

{#if tab === 'overall'}
 <section class="card"><div class="q-h"><h2>Your weeks</h2><span class="small">{countedCount ? `goals met in ${metCount} of ${countedCount} complete weeks` : 'No weekly goals yet'}</span></div>
  <div class="wstrip" role="group" aria-label="Last 12 weeks">
   {#each weeks12 as w, i}{@const o = outcomes[i]}
    <button class="wk" class:cur={!o} onclick={() => o ? go(`review/${weekId(w)}`) : go('plan')} aria-label={`Week of ${dayMonth(w)}: ${o ? (o.goals.length ? o.goals.map(g => `${g.name} ${g.recorded} of ${g.target}`).join(', ') : 'no goals') + (o.reasons.length ? `; ${o.reasons.join(', ')}` : '') : 'this week'}`}>
     <NestedRings rings={o ? ringsOf(o) : []} size={26} stroke={2.6} gap={1} mark={o ? markOf(o) : null}/>
     <small>{dayMonth(w).split(' ')[0]}</small>
    </button>{/each}
  </div>
  <p class="why-l">{#each outcomes.filter(o => o && o.reasons.some(r => !r.includes('rest'))) as o}<span>{dayMonth(o!.from)}: {o!.reasons.filter(r => !r.includes('rest')).join(', ')}</span>{/each}</p>
 </section>
 {#each ambitions as a}<button class="race-l" onclick={() => go(`plan/race/${a.id}`)} aria-label={`${a.name}, ${dayLabel(a.date!)}`}><span class="cd"><b class="n">{Math.max(0, Math.round((Date.parse(a.date!) - Date.parse(ctx.calendarDate)) / 864e5))}</b><small>days</small></span><span><b>{a.name}</b><small>{dayLabel(a.date!)}{tvw.races.find(r => r.label === '10K') && /10k/i.test(a.name) ? ` · Garmin’s 10K prediction ${Math.floor(tvw.races.find(r => r.label === '10K')!.seconds / 60)}:${String(tvw.races.find(r => r.label === '10K')!.seconds % 60).padStart(2, '0')}` : ''}</small></span><Icon name="chevron" size={14}/></button>{/each}

 <div class="pctl">
  <div class="seg" role="tablist" aria-label="Period">{#each PERIODS as p}<button role="tab" aria-selected={period === p.key} onclick={() => setP(p.key)}>{p.label}</button>{/each}</div>
  <div class="pnav"><button class="iconbtn" aria-label="Previous period" disabled={!canPrev} onclick={() => move(-1)}><Icon name="arrow-left" size={16}/></button><b>{s.label}</b><button class="iconbtn" aria-label="Next period" disabled={!canNext} onclick={() => move(1)}><Icon name="arrow-right" size={16}/></button></div>
 </div>
 <section class="report" aria-label="Period report">
  {#if s.partial}<p class="small">Known activities · partial history ({s.now.coveredDays}/{s.now.days} days covered)</p>{:else if !s.prev}<p class="small">Comparison unavailable · earlier coverage or period length</p>{/if}
  {#if s.prev}<p class="vs">vs {prevRange}</p>{/if}
  <div class="nums">
   <div class="num lead"><b class="n">{s.now.activities}</b><span>Activities</span><em>{s.prev ? delta(s.now.activities - s.prev.activities) : ''}</em></div>
   <div class="num"><b class="n">{hours(s.now.minutes)}<small>h</small></b><span>Active time</span><em>{s.prev ? delta((s.now.minutes - s.prev.minutes) / 60, ' h') : ''}</em></div>
   <div class="num"><b class="n">{Math.round(s.now.km)}<small>km</small></b><span>Distance</span><em>{s.prev ? delta(s.now.km - s.prev.km, ' km') : ''}</em></div>
   <div class="num"><b class="n">{s.now.im}</b><span>Intensity min</span><em>{s.prev ? delta(pct(s.now.im, s.prev.im)) : ''}</em></div>
  </div>
  <AxisChart points={bars.map(b => ({x: b.short, v: b.future ? null : b.total, parts: b.sports.map(([sp, m]) => ({v: m, color: SPORTS[sp].color})), mark: b.known || b.future ? undefined : 'unknown'}))} unit="min" values={period === 'week'} height={140} label={`Minutes per ${period === 'week' || period === 'month' ? 'day' : period === 'quarter' ? 'week' : 'month'}: ${bars.map(b => `${b.label} ${b.future ? 'future' : b.total}`).join(', ')}`}/>
  {#if mix.length}<div class="mix-key">{#each mix.slice(0, 5) as m}<span><Glyph sport={m.sport} size={18}/>{m.name}<b class="n">{hours(m.min)}h</b></span>{/each}</div>{/if}
 </section>
 <div class="sec-h"><h2>{monthYear(cal.from)}</h2><span class="small">Active on {calActive} of {calKnown} covered days</span></div>
 <section class="card">
  <DayCells from={cal.from} count={cal.days.length} lead={(new Date(cal.from + 'T00:00:00Z').getUTCDay() + 6) % 7} size={30}/>
  <p class="cal-key">Rings show each goal per day; tap a day for details. <button class="link inline" onclick={() => store.openSheet('legend')}>Key</button></p>
 </section>
 {#if miles.length}
  <div class="sec-h"><h2>Records and milestones</h2></div>
  <div class="miles">
   {#each miles as m}<button class="mile" class:inperiod={m.date >= s.from && m.date <= s.end} onclick={() => go(m.route)}>
    <span class="mi k-{m.kind}">{#if 'sport' in m && m.sport}<Glyph sport={m.sport} size={30}/>{:else if m.kind === 'best'}<Icon name="star" size={18}/>{:else if m.kind === 'estimate'}<Icon name="trend-up" size={18}/>{:else}<Icon name="check" size={18}/>{/if}</span>
    <b class="n">{m.value}</b><span class="mt">{m.title}</span><small>{dayLabel(m.date)}</small><em>{m.note}</em>
   </button>{/each}
  </div>
 {/if}

{:else if tab === 'cardio'}
 {#each cardioKeys as k}
  <section class="card"><div class="q-h"><h2>{GOAL_INFO[k].name} {k === 'im' ? '' : 'minutes '}per week</h2>{#if cg[k]}<span class="small">goal {cg[k]}</span>{/if}</div>
   <AxisChart points={cardioWeek(k)} unit="min" goal={cg[k] || null} color={GOAL_INFO[k].color} height={140} every={3} label={`${GOAL_INFO[k].name} minutes per week, last 12 weeks: ${cardioWeek(k).map(p => p.v ?? 'none').join(', ')}${cg[k] ? `; goal ${cg[k]}` : ''}`}/>
   <p class="small">{k === 'im' ? 'Garmin’s intensity minutes' : 'Recorded heart-rate zone time only'} · hatched weeks: ill or paused · dotted: missing data</p>
  </section>
 {/each}
 <section class="card"><div class="q-h"><h2>VO₂ max, same weeks</h2><HelpButton topic="vo2max"/></div>
  <AxisChart points={vo2w} kind="dots" unit="ml/kg/min" color="#E4E8F0" height={120} every={3} label={`VO₂ max estimates by week: ${vo2w.map(p => p.v ?? 'none').join(', ')}`}/>
  <p class="small">Read it against the weeks above: fitness follows consistent weeks, with a lag.</p>
 </section>

{:else if tab === 'strength'}
 <section class="card"><div class="q-h"><h2>Strength sessions per week</h2>{#if cg.strength}<span class="small">goal {cg.strength}</span>{/if}</div>
  <AxisChart points={strW} unit="sessions" goal={cg.strength || null} color={SPORTS.strength.color} height={130} every={3} label={`Strength sessions per week, last 12 weeks: ${strW.map(p => p.v ?? 'none').join(', ')}`}/>
 </section>
 {#if bests.length}<section class="card"><div class="q-h"><h2>Bests in available history</h2><button class="link" onclick={() => go('strength')}>All lifts</button></div>
  <dl class="kv">{#each bests as b}<dt>{b.name}</dt><dd>{b.best!.weight} kg × {b.best!.reps}+ <small>{b.best!.date ? dayMonth(b.best!.date) : ''}</small></dd>{/each}</dl></section>{/if}
 {#each lifts.slice(0, 2) as l}<section class="card"><button class="lift-row" onclick={() => go(`strength/${l.exercise.id}`)}><span class="lr-h"><b>{l.exercise.name}</b><span class="n">{l.start ?? '—'} → {l.end} kg</span></span></button>
  <AxisChart points={l.points.map(p => ({x: dayMonth(p.date), v: p.weight}))} kind="line" unit="kg" color={SPORTS.strength.color} height={110} label={`${l.exercise.name}: heaviest set from ${l.start ?? 'none'} to ${l.end} kg`}/></section>{/each}

{:else}
 <section class="card"><div class="q-h"><h2>Training per month</h2><HelpButton topic="progress-overall"/></div>
  <AxisChart points={hoursM} unit="h" color="#36C9C6" height={120} fmt={v => String(Math.round(v * 10) / 10)} label={`Training hours per month: ${hoursM.map(p => p.v ?? 'none').join(', ')}`}/>
  <p class="small">What happened. The readings below are what it did, month by month.</p>
 </section>
 <section class="card"><div class="q-h"><h2>Resting heart rate</h2><span class="small">monthly average</span></div><AxisChart points={rhrM} kind="line" unit="bpm" color="#FF8A8A" height={120} label={`Resting heart rate monthly averages: ${rhrM.map(p => p.v == null ? 'none' : Math.round(p.v)).join(', ')}`}/></section>
 <section class="card"><div class="q-h"><h2>Overnight HRV</h2><span class="small">monthly average</span></div><AxisChart points={hrvM} kind="line" unit="ms" color="#A48BFF" height={120} label={`HRV monthly averages: ${hrvM.map(p => p.v == null ? 'none' : Math.round(p.v)).join(', ')}`}/></section>
 <section class="card"><div class="q-h"><h2>VO₂ max and Fitness Age</h2><HelpButton topic="fitness-age"/></div>
  <AxisChart points={vo2M} kind="dots" unit="VO₂ max" color="#E4E8F0" height={110} label={`VO₂ max by month: ${vo2M.map(p => p.v == null ? 'none' : Math.round(p.v)).join(', ')}`}/>
  <AxisChart points={faM} kind="dots" unit="years" color="#3DDC97" height={110} label={`Fitness Age by month: ${faM.map(p => p.v == null ? 'none' : Math.round(p.v)).join(', ')}`}/>
  <p class="small">Garmin estimates; months without one stay empty.</p>
 </section>
{/if}

<style>
 .tabs4{display:grid;grid-template-columns:repeat(4,1fr);gap:4px;padding:4px;border-radius:14px;background:rgba(255,255,255,.05);margin:12px 0 6px}.tabs4 button{height:38px;border-radius:10px;border:0;background:none;color:var(--text2);font:600 12.5px Inter}.tabs4 button[aria-selected="true"]{background:var(--ink);color:var(--bg)}
 .q-h{display:flex;justify-content:space-between;align-items:center;gap:8px;margin-bottom:8px}.q-h h2{font:600 17px Outfit}
 .wstrip{display:grid;grid-template-columns:repeat(12,minmax(0,1fr));gap:2px}.wk{display:grid;justify-items:center;gap:3px;background:none;border:0;padding:4px 0;color:var(--text3);min-height:44px}.wk small{font-size:9px}.wk.cur{opacity:.5}
 .why-l{display:flex;flex-direction:column;gap:2px;margin:8px 0 0;font-size:11.5px;color:#A9C8FF}
 .race-l{display:grid;grid-template-columns:40px 1fr 14px;gap:10px;align-items:center;width:100%;margin-top:10px;padding:10px 12px;border-radius:14px;border:1px solid rgba(255,181,71,.3);background:rgba(255,181,71,.06);color:var(--text);text-align:left}.race-l b{display:block;font:600 14px Inter}.race-l small{font-size:11px;color:var(--text2)}.cd{display:grid;place-items:center;line-height:1}.cd b{font-size:20px;color:#FFB547}.cd small{font-size:9px;color:var(--text3)}

 .pctl{margin-top:12px}
 .seg{display:grid;grid-template-columns:repeat(4,1fr);gap:4px;padding:4px;border-radius:14px;background:rgba(255,255,255,.05)}
 .seg button{height:36px;border-radius:10px;border:0;background:none;color:var(--text2);font:600 13px Inter}
 .seg button[aria-selected="true"]{background:var(--ink);color:var(--bg)}
 .pnav{display:flex;align-items:center;justify-content:space-between;margin-top:10px}
 .pnav b{font:600 18px Outfit}
 .pnav .iconbtn:disabled{opacity:.3}
 .report{margin-top:12px;padding:16px;border-radius:24px;background:linear-gradient(180deg,rgba(164,139,255,.12),rgba(255,255,255,.04) 60%);border:1px solid rgba(164,139,255,.25)}
 .hl{display:grid;grid-template-columns:auto 1fr auto;gap:10px;align-items:center;width:100%;margin-bottom:14px;padding:8px 12px 8px 8px;border-radius:16px;border:1px solid rgba(255,214,102,.3);background:rgba(255,214,102,.08);color:var(--text);text-align:left}
 .hl-i{width:32px;height:32px;border-radius:50%;display:grid;place-items:center;background:rgba(255,214,102,.18);color:#FFD666}
 
 
 .nums{display:grid;grid-template-columns:repeat(4,minmax(0,1fr));gap:8px;margin-bottom:16px}
 .num b{display:block;font-size:26px;font-weight:600;line-height:1.05}
 .num.lead b{font-size:34px}
 .num b small{font:500 12px Inter;color:var(--text3);margin-left:2px}
 .num span{display:block;font-size:11.5px;color:var(--text2);margin-top:2px}
 .num em{display:block;font-style:normal;font-size:11px;color:var(--text3);margin-top:3px}
 .mix{display:flex;height:8px;border-radius:4px;overflow:hidden;gap:2px;margin-top:6px}
 .mix-key{display:flex;flex-wrap:wrap;gap:8px 14px;margin-top:10px;font-size:12px;color:var(--text2)}
 .mix-key span{display:inline-flex;align-items:center;gap:6px}
 .mix-key b{color:var(--text);font-weight:600}
 .vs{margin:-4px 0 8px;text-align:right;font:600 11px Inter;color:var(--text3);letter-spacing:.04em}
 .cal-key{display:flex;flex-wrap:wrap;gap:6px 14px;margin-top:10px;font-size:11px;color:var(--text3)}
 
 .k{width:11px;height:11px;border-radius:3px;display:inline-block}
 .k.a{background:rgba(255,255,255,.14);box-shadow:inset 0 -3px 0 #36C9C6}.k.r{background:rgba(169,200,255,.2)}.k.p{background:repeating-linear-gradient(45deg,rgba(169,200,255,.5) 0 2px,transparent 2px 4px)}.k.u{border:1.5px dotted #8E98AC}
 .k.sl{background:#5C9BFF}.k.nd{height:0;border-top:2px solid #EEF1F7;border-radius:0}
 .add-c{display:grid;grid-template-columns:auto 1fr auto;gap:12px;align-items:center;width:100%;margin-top:14px;padding:12px 14px;border-radius:18px;border:1px dashed rgba(228,232,240,.25);background:none;color:var(--text);text-align:left}
 
 .tgrid{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:10px;margin-top:10px}
 .st-top{display:flex;gap:14px;align-items:center}
 .st-map{display:flex;flex-direction:column;align-items:center;gap:4px}
 
 .st-weeks{flex:1;display:flex;flex-direction:column;gap:6px;min-width:0}
 .swk{display:flex;align-items:flex-end;gap:3px;height:72px}
 
 
 
 
 .lift-row{display:block;width:100%;margin-top:14px;padding-top:12px;border:0;border-top:1px solid var(--line);background:none;color:var(--text);text-align:left}
 .lr-h{display:flex;justify-content:space-between;font-size:13px}
 .lr-h b{font-weight:600}.lr-h span{color:var(--text2)}
 .miles{display:flex;gap:10px;overflow-x:auto;padding:10px 2px 6px;scroll-snap-type:x mandatory;scrollbar-width:none}
 .mile{flex:none;width:148px;scroll-snap-align:start;display:flex;flex-direction:column;align-items:flex-start;gap:2px;padding:12px;border-radius:18px;background:var(--card);border:1px solid var(--line);color:var(--text);text-align:left}
 .mile.inperiod{border-color:rgba(255,214,102,.35);background:linear-gradient(180deg,rgba(255,214,102,.08),var(--card))}
 .mi{width:34px;height:34px;border-radius:50%;display:grid;place-items:center;background:rgba(255,255,255,.07);color:var(--text);margin-bottom:6px}
 .mi.k-best{background:rgba(255,214,102,.16);color:#FFD666}.mi.k-estimate{background:rgba(61,220,151,.14);color:#3DDC97}.mi.k-plan{background:rgba(164,139,255,.16);color:#C9BEFF}
 .mile b{font-size:19px;font-weight:600}
 .mt{font-size:13px;font-weight:600}
 .mile small{font-size:11px;color:var(--text3)}
 .mile em{font-style:normal;font-size:11px;color:var(--text2);margin-top:2px}
 .sl-top{display:flex;justify-content:space-between;align-items:flex-end;margin-bottom:12px;gap:12px}
 .big2{font-size:28px;font-weight:600;display:block}
 .sl-short{text-align:right}
 .sub{font:600 11px Inter;letter-spacing:.12em;text-transform:uppercase;color:var(--text3);margin:18px 0 8px}
 .reviews{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:10px;margin-top:10px}
 .rv{display:flex;flex-direction:column;align-items:flex-start;gap:3px;padding:12px;border-radius:18px;background:var(--card);border:1px solid var(--line);color:var(--text);text-align:left}
 .rv-rings{display:flex;gap:4px;margin-bottom:6px}
 
 @media(max-width:370px){.nums{grid-template-columns:repeat(2,1fr)}.num.lead b{font-size:30px}}
</style>
