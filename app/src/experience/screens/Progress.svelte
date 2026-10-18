<script lang="ts">
 import Icon from '../../lib/components/Icon.svelte';
 import BodyMap from '../../lib/components/BodyMap.svelte';
 import ConsistencyChart from '../../lib/components/ConsistencyChart.svelte';
 import RootHeader from '../components/RootHeader.svelte';
 import PeriodBars from '../components/PeriodBars.svelte';
 import ActivityCalendar from '../components/ActivityCalendar.svelte';
 import CommitmentBars from '../components/CommitmentBars.svelte';
 import TrendTile from '../components/TrendTile.svelte';
 import LiftChart from '../components/LiftChart.svelte';
 import SleepNights from '../components/SleepNights.svelte';
 import Ring from '../components/Ring.svelte';
 import Glyph from '../components/Glyph.svelte';
 import {store} from '../lib/store.svelte';
 import {PERIODS, buckets, calendarMonth, commitmentWeeks, liftProgress, records, shift, sportMix, strengthWeeks, summary, trendTile, TREND_TILES, type Period} from '../lib/progress';
 import {addDays, dayLabel, dayMonth, dur, monthName, monthStart, monthYear, range, weekId, weekRange, weekStart} from '../lib/dates';
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
</script>

<RootHeader title="Progress"/>

<div class="pctl">
 <div class="seg" role="tablist" aria-label="Period">{#each PERIODS as p}<button role="tab" aria-selected={period === p.key} onclick={() => setP(p.key)}>{p.label}</button>{/each}</div>
 <div class="pnav"><button class="iconbtn" aria-label="Previous period" disabled={!canPrev} onclick={() => move(-1)}><Icon name="arrow-left" size={16}/></button><b>{s.label}</b><button class="iconbtn" aria-label="Next period" disabled={!canNext} onclick={() => move(1)}><Icon name="arrow-right" size={16}/></button></div>
</div>

<section class="report" aria-label="Period report">
 {#if s.partial}<p class="small">Known activities · partial history ({s.now.coveredDays}/{s.now.days} days covered)</p>{:else if !s.prev}<p class="small">Comparison unavailable · earlier coverage or period length</p>{/if}
 {#if highlight}<button class="hl" onclick={() => go(highlight.route)}><span class="hl-i"><Icon name="star" size={16}/></span><span><small>{highlight.kind === 'best' ? 'New best' : highlight.kind === 'estimate' ? 'Garmin estimate' : highlight.kind === 'plan' ? 'Milestone' : 'Record'} · {dayMonth(highlight.date)}</small><b>{highlight.title} {highlight.value}</b></span><Icon name="chevron" size={16}/></button>{/if}
 {#if s.prev}<p class="vs">vs {prevRange}</p>{/if}
 <div class="nums">
  <div class="num lead"><b class="n">{s.now.activities}</b><span>Activities</span><em>{s.prev ? delta(s.now.activities - s.prev.activities) : ''}</em></div>
  <div class="num"><b class="n">{hours(s.now.minutes)}<small>h</small></b><span>Active time</span><em>{s.prev ? delta((s.now.minutes - s.prev.minutes) / 60, ' h') : ''}</em></div>
  <div class="num"><b class="n">{Math.round(s.now.km)}<small>km</small></b><span>Distance</span><em>{s.prev ? delta(s.now.km - s.prev.km, ' km') : ''}</em></div>
  <div class="num"><b class="n">{s.now.im}</b><span>Intensity min</span><em>{s.prev ? delta(pct(s.now.im, s.prev.im)) : ''}</em></div>
 </div>
 <PeriodBars buckets={bars} h={period === 'year' ? 104 : 96}/>
 {#if mix.length}<div class="mix" role="img" aria-label={`Time by activity: ${mix.map(m => `${m.name} ${hours(m.min)} h`).join(', ')}`}>{#each mix as m}<i style:flex={m.min} style:background={m.color}></i>{/each}</div>
  <div class="mix-key">{#each mix.slice(0, 5) as m}<span><Glyph sport={m.sport} size={18}/>{m.name}<b class="n">{hours(m.min)}h</b></span>{/each}</div>
 {:else}<p class="small">No recorded activities in this period.</p>{/if}
</section>

<div class="sec-h"><h2>{monthYear(cal.from)}</h2><span class="small">Active on {calActive} of {calKnown} covered days</span></div>
<section class="card">
 <ActivityCalendar month={cal} onDay={d => { go('activities'); setTimeout(() => store.setUi({date: d}), 50); }}/>
 <p class="cal-key" aria-hidden="true"><span><i class="k a"></i>Recorded</span><span><i class="k r"></i>Rest</span><span><i class="k p"></i>Paused</span><span><i class="k u"></i>Awaiting evidence</span></p>
</section>

{#if strengthW.length || imW.length}
 <div class="sec-h"><h2>Commitments over time</h2><button class="link" onclick={() => go('plan/commitments')}>Edit</button></div>
 <section class="card">
  {#if strengthW.length}<CommitmentBars weeks={strengthW} name="Strength sessions" color={SPORTS.strength.color} onWeek={id => go(id === ctx.plan?.wk ? 'plan' : `review/${id}`)}/>{/if}
  {#if imW.length}<CommitmentBars weeks={imW} name="Intensity minutes" unit=" min" color="#36C9C6" onWeek={id => go(id === ctx.plan?.wk ? 'plan' : `review/${id}`)}/>{/if}
 </section>
{:else if ctx.world.activities.length}
 <button class="add-c" onclick={() => go('plan/commitments')}><Ring value={0} target={2} size={34} stroke={4} label="No commitment yet" center={false}/><span><b>Add a commitment</b><small>Optional · e.g. 2 strength sessions a week</small></span><Icon name="chevron" size={16}/></button>
{/if}

<div class="sec-h"><h2>Fitness and health</h2><button class="link" onclick={() => go('trends')}>All trends</button></div>
<div class="tgrid">
 {#each tiles as {t, v}}<TrendTile name={t.name} value={v.value} unit={t.unit} change={v.change} points={v.points} sparse={t.sparse} noNewEstimate={!!t.sparse && v.count === 0} decimals={t.decimals ?? 0} note={t.sparse && v.latestDate ? dayMonth(v.latestDate) : undefined} color={t.id === 'sleepTime' ? '#5C9BFF' : t.id === 'stress' ? '#FFB547' : t.id === 'bodyBattery' ? '#3DDC97' : t.id === 'hrv' || t.id === 'restingHr' ? '#FF8A8A' : '#E4E8F0'} onclick={() => go(`metric/${t.id}`)}/>{/each}
</div>

{#if hasStrength}
 <div class="sec-h"><h2>Strength</h2><button class="link" onclick={() => go('strength')}>Strength</button></div>
 <section class="card strength">
  <div class="st-top">
   <div class="st-map"><BodyMap figure="female" {muscles} width={104} max={Math.max(8, period === 'year' ? 60 : period === 'quarter' ? 30 : 12)} legend={false}/><small>Sets by muscle · this period</small></div>
   <div class="st-weeks" role="img" aria-label={`Strength sets per week: ${sWeeks.map(w => w.sets).join(', ')}`}>
    <small>Sets per week</small>
    <div class="swk">{#each sWeeks as w}<span class:unknown={!w.known} class:timer={w.timerOnly && !w.sets} style:height={Math.max(3, w.sets / Math.max(1, ...sWeeks.map(x => x.sets)) * 70) + 'px'}></span>{/each}</div>
    <b class="n">{sWeeks.reduce((t, w) => t + w.sessions, 0)}<small> sessions this period</small></b>
   </div>
  </div>
  {#each lifts as l}
   <button class="lift-row" onclick={() => go(`strength/${l.exercise.id}`)}>
    <span class="lr-h"><b>{l.exercise.name}</b><span class="n">{l.start ?? '—'} → {l.end} kg</span></span>
    <LiftChart points={l.points} name={l.exercise.name} best={l.best}/>
   </button>
  {:else}<p class="small">Fewer than two sessions with sets for any exercise this period.</p>{/each}
 </section>
{/if}

{#if miles.length}
 <div class="sec-h"><h2>Records and milestones</h2></div>
 <div class="miles">
  {#each miles as m}<button class="mile" class:inperiod={m.date >= s.from && m.date <= s.end} onclick={() => go(m.route)}>
   <span class="mi k-{m.kind}">{#if 'sport' in m && m.sport}<Glyph sport={m.sport} size={30}/>{:else if m.kind === 'best'}<Icon name="star" size={18}/>{:else if m.kind === 'estimate'}<Icon name="trend-up" size={18}/>{:else}<Icon name="check" size={18}/>{/if}</span>
   <b class="n">{m.value}</b><span class="mt">{m.title}</span><small>{dayLabel(m.date)}</small><em>{m.note}</em>
  </button>{/each}
 </div>
{/if}

<div class="sec-h"><h2>Sleep</h2><button class="link" onclick={() => go('sleep/patterns')}>Patterns</button></div>
<section class="card">
 <div class="sl-top"><div><b class="n big2">{dur(avgSleep)}</b><span class="small">average asleep · {recorded.length} nights{missing ? ` · ${missing} without a record` : ''}</span></div><div class="sl-short"><b class="n">{shortNights}</b><span class="small">an hour or more below need</span></div></div>
 <SleepNights {nights}/>
 {#if period === 'quarter' || period === 'year'}<p class="small">{period === 'year' ? 'Fortnight' : 'Weekly'} averages · recorded nights</p>{/if}
 <p class="cal-key" aria-hidden="true"><span><i class="k sl"></i>Asleep</span><span><i class="k nd"></i>That night’s need</span><span><i class="k u"></i>No recorded night</span></p>
 {#if period === 'week' || period === 'month'}{@const sv = sleepView(ctx)}{#if sv.consistency.length}<h3 class="sub">Timing · last 14 nights</h3><ConsistencyChart nights={sv.consistency} date={sv.date} bedWindow={sv.bedWindow} wakeWindow={sv.wakeWindow}/>{/if}{/if}
</section>

{#if reviewWeeks.length}
 <div class="sec-h"><h2>Weekly reviews</h2><button class="link" onclick={() => go('reviews')}>All reviews</button></div>
 <div class="reviews">
  <button class="rv" onclick={() => go(`review/${latestReviewWeek(ctx)}`)}><span class="rv-rings">{#if latestReviewWeek(ctx) === weekId(ctx.calendarDate)}{#each ctx.plan?.tracks ?? [] as t}<Ring value={t.have} target={t.target} size={30} stroke={3.5} state={t.state} center={false} label={`${t.key}: ${t.label}`}/>{/each}{:else}{@const previous = PLAN_WEEKS.find(w => w.id === latestReviewWeek(ctx))}{#each [previous?.strength, previous?.im].filter(p => p != null) as p}<Ring value={p![0]} target={p![1]} size={30} stroke={3.5} state={previous?.state === 'paused' || previous?.state === 'unknown' ? previous.state : p![0] >= p![1] ? 'reached' : 'open'} center={false} label={`${p![0]} of ${p![1]}`}/>{/each}{/if}</span><b>{latestReviewWeek(ctx) === weekId(ctx.calendarDate) ? 'This week' : 'Last week'}</b><small>{weekRange(latestReviewWeek(ctx) === weekId(ctx.calendarDate) ? ctx.weekFrom : addDays(ctx.weekFrom, -7))}</small></button>
  {#each reviewWeeks.filter(w => w.id !== latestReviewWeek(ctx)) as w}<button class="rv" onclick={() => go(`review/${w.id}`)}><span class="rv-rings">{#if w.strength && w.im}<Ring value={w.strength[0]} target={w.strength[1]} size={30} stroke={3.5} state={w.state === 'paused' ? 'paused' : w.state === 'unknown' ? 'unknown' : w.strength[0] >= w.strength[1] ? 'reached' : 'open'} center={false} label={`Strength ${w.strength[0]} of ${w.strength[1]}`}/><Ring value={w.im[0]} target={w.im[1]} size={30} stroke={3.5} state={w.state === 'paused' ? 'paused' : w.state === 'unknown' ? 'unknown' : w.im[0] >= w.im[1] ? 'reached' : 'open'} center={false} label={`Intensity ${w.im[0]} of ${w.im[1]}`}/>{:else}<Ring value={0} target={1} size={30} stroke={3.5} state="none" center={false} label="No plan yet"/>{/if}</span><b>{w.label}</b><small>{weekRange(w.from)}</small></button>{/each}
 </div>
{/if}
<p class="end-note">Looking for something else? <button class="link inline" onclick={() => go('explore')}>Explore</button></p>

<style>
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
 .hl small{display:block;font-size:11px;color:var(--text2)}
 .hl b{display:block;font-size:14px}
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
 .cal-key span{display:inline-flex;align-items:center;gap:6px}
 .k{width:11px;height:11px;border-radius:3px;display:inline-block}
 .k.a{background:rgba(255,255,255,.14);box-shadow:inset 0 -3px 0 #36C9C6}.k.r{background:rgba(169,200,255,.2)}.k.p{background:repeating-linear-gradient(45deg,rgba(169,200,255,.5) 0 2px,transparent 2px 4px)}.k.u{border:1.5px dotted #8E98AC}
 .k.sl{background:#5C9BFF}.k.nd{height:0;border-top:2px solid #EEF1F7;border-radius:0}
 .add-c{display:grid;grid-template-columns:auto 1fr auto;gap:12px;align-items:center;width:100%;margin-top:14px;padding:12px 14px;border-radius:18px;border:1px dashed rgba(228,232,240,.25);background:none;color:var(--text);text-align:left}
 .add-c b{display:block}.add-c small{display:block;font-size:12px;color:var(--text2)}
 .tgrid{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:10px;margin-top:10px}
 .st-top{display:flex;gap:14px;align-items:center}
 .st-map{display:flex;flex-direction:column;align-items:center;gap:4px}
 .st-map small,.st-weeks small{font-size:11px;color:var(--text3)}
 .st-weeks{flex:1;display:flex;flex-direction:column;gap:6px;min-width:0}
 .swk{display:flex;align-items:flex-end;gap:3px;height:72px}
 .swk span{flex:1;background:#FF7DB8;border-radius:3px 3px 2px 2px;opacity:.85}
 .swk span.timer{background:none;border:1.5px dashed #FF7DB8}
 .swk span.unknown{border:1.5px dotted #8E98AC;background:rgba(255,125,184,.18)}
 .st-weeks b{font-size:22px}.st-weeks b small{font:500 12px Inter;color:var(--text3)}
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
 .sl-short{text-align:right}.sl-short b{font-size:22px;display:block}
 .sub{font:600 11px Inter;letter-spacing:.12em;text-transform:uppercase;color:var(--text3);margin:18px 0 8px}
 .reviews{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:10px;margin-top:10px}
 .rv{display:flex;flex-direction:column;align-items:flex-start;gap:3px;padding:12px;border-radius:18px;background:var(--card);border:1px solid var(--line);color:var(--text);text-align:left}
 .rv-rings{display:flex;gap:4px;margin-bottom:6px}
 .rv b{font-size:13px;font-weight:600}.rv small{font-size:11px;color:var(--text3)}
 @media(max-width:370px){.nums{grid-template-columns:repeat(2,1fr)}.num.lead b{font-size:30px}}
</style>
