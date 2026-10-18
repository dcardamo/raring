<script lang="ts">
 import Header from '../../lib/components/Header.svelte';
 import Icon from '../../lib/components/Icon.svelte';
 import Card from '../../lib/components/Card.svelte';
 import TrendSpark from '../components/TrendSpark.svelte';
 import HistoryChart from '../../lib/components/HistoryChart.svelte';
 import MetricDetail from '../../lib/components/MetricDetail.svelte';
 import StrengthHistory from '../../lib/components/StrengthHistory.svelte';
 import GearCard from '../../lib/components/GearCard.svelte';
 import ConsistencyChart from '../../lib/components/ConsistencyChart.svelte';
 import AlignmentCard from '../../lib/components/AlignmentCard.svelte';
 import BodyMap from '../../lib/components/BodyMap.svelte';
 import ReadinessDetail from '../../lib/screens/ReadinessDetail.svelte';
 import TrainingDetail from '../../lib/screens/TrainingDetail.svelte';
 import HealthDetail from '../../lib/screens/HealthDetail.svelte';
 import SleepScreen from './SleepScreen.svelte';
 import Adapt from './Adapt.svelte';
 import PlanDetail from './PlanDetail.svelte';
 import Settings from './Settings.svelte';
 import Misc from './Misc.svelte';
 import ActivityTile from '../components/ActivityTile.svelte';
 import SleepNights from '../components/SleepNights.svelte';
 import Glyph from '../components/Glyph.svelte';
 import {store, titleOf} from '../lib/store.svelte';
 import {METRICS} from '../lib/metrics';
 import {addDays, dayLabel, dur, range, weekRange, weekStart} from '../lib/dates';
 import {bodyBatteryDay, healthView, sleepView, strengthView, todayView, trainingView, acuteLoad, musclesBetween} from '../lib/views';
 import {EXERCISES, GEAR, weightFor, hrvFor, vo2maxFor, fitnessAgeFor} from '../lib/world';
 import {SPORTS, type Sport} from '../lib/sports';
 let {route}: {route: string} = $props();
 let ctx = $derived(store.ctx);
 const go = (r: string) => store.navigate(r);
 let [key, sub] = $derived(route.split('/'));
 let selectedFactor = $state('');
 // Activities list state lives in the history entry, so Back restores it.
 let filter = $derived((store.ui.filter as string) ?? 'all');
 let date = $derived((store.ui.date as string | null) ?? null);
 let limit = $derived((store.ui.limit as number) ?? 20);
 let acts = $derived(ctx.world.activities.filter(a => (filter === 'all' || a.sport === filter) && (!date || a.date === date)).slice().reverse());
 let sports = $derived(['all', ...new Set(ctx.world.activities.map(a => a.sport))]);
 let journalDayOpen = $state<string | null>(null);
 let nights14 = $derived(range(addDays(ctx.displayedDay, -13), ctx.displayedDay).map(d => { const n = ctx.nightOf(d); return {date: d, asleep: n?.asleep ?? null, need: n?.need ?? null, label: ['S', 'M', 'T', 'W', 'T', 'F', 'S'][new Date(d + 'T12:00:00Z').getUTCDay()]}; }));
 let marks = $derived(Object.keys(store.state.winddown).filter(d => store.state.winddown[d]));
 const metricValue = (id: string): [string, string] => {
  const d = ctx.displayedDay, n = ctx.night;
  const v: Record<string, () => [string, string]> = {
   readiness: () => [String(ctx.readiness?.value ?? '—'), ctx.readiness?.level ?? (ctx.world.readinessSupported ? 'No reading yet' : 'Not recorded by this watch')],
   sleepScore: () => [String(n?.score ?? '—'), n ? dayLabel(d) : 'No recorded night'], sleepTime: () => [n ? dur(n.asleep) : '—', n ? `Need ${dur(n.need)}` : 'No recorded night'],
   sleepHr: () => [n ? String(todayView(ctx).vitals.readings[0].value) : '—', 'Overnight average, separate from daily resting HR'],
   acuteLoad: () => [String(acuteLoad(ctx, ctx.calendarDate)), 'Garmin’s range for you: 300–560'],
   fitnessAge: () => ['41', 'Garmin estimate · 3 Nov · age 44'],
  };
  return v[id]?.() ?? ['—', ''];
 };
 let weekly = $derived(Array.from({length: 12}, (_, i) => { const from = addDays(weekStart(ctx.calendarDate), (i - 11) * 7); let v: number | null = null; for (const d of range(from, addDays(from, 6))) v = weightFor(d) ?? v; return v; }));
 let exerciseView = $derived.by(() => {
  const sv = strengthView(ctx);
  if (!sv || key !== 'strength' || !sub || sub === 'muscles') return sv;
  const e = EXERCISES.find(x => x.id === sub);
  const chosen = sv.exercises.find(x => x.name === e?.name);
  return chosen ? {...sv, exercises: [chosen, ...sv.exercises.filter(x => x !== chosen)]} : sv;
 });
</script>

{#snippet header(title: string, eyebrow = '')}<Header {title} {eyebrow} back={() => store.back()} backLabel={store.parentLabel}/>{/snippet}

{#if key === 'sleep' && sub === 'patterns'}
 {@render header('Sleep patterns', `14 nights to ${dayLabel(ctx.displayedDay)}`)}
 <Card title="Duration and need"><SleepNights nights={nights14}/><p class="small">{nights14.filter(n => n.asleep != null && n.need != null && n.need - n.asleep >= 60).length} of {nights14.filter(n => n.asleep != null).length} recorded nights were an hour or more below that night’s need.{#if nights14.some(n => n.asleep == null)} {nights14.filter(n => n.asleep == null).length} with no recorded night.{/if}</p></Card>
 {@const sv = sleepView(ctx)}
 {#if sv.consistency.length}<Card title="Timing"><ConsistencyChart nights={sv.consistency} date={sv.date} bedWindow={sv.bedWindow} wakeWindow={sv.wakeWindow}/><p class="small">Garmin · 7-night average: bed 23:14, wake 06:58</p></Card>{/if}
 {#if sv.alignment}<AlignmentCard alignment={sv.alignment}/>{/if}
 <Card title="Wind-down · self-reported">{#if marks.length}<div class="chiprow">{#each marks as d}<span class="selfchip"><Glyph sport="yoga" mark="self" size={20} label="Self-reported"/>{dayLabel(d)}</span>{/each}</div>{:else}<p class="small">Nothing marked. Optional.</p>{/if}</Card>
 <Card title="Time asleep"><HistoryChart values={[]} endDate={ctx.displayedDay} label="Time asleep" metric="sleepTime" initialRange="days30" nights/></Card>

{:else if key === 'sleep'}
 <SleepScreen/>

{:else if key === 'readiness'}
 {#snippet rh()}<Header title="Readiness" eyebrow={`${dayLabel(ctx.displayedDay)} · Garmin`} back={() => store.back()} backLabel={store.parentLabel}/>{/snippet}
 {#if !ctx.world.readinessSupported}{@render rh()}<Card><p>This watch reports that it can’t record Training Readiness.</p><p class="small">Sleep, Body Battery, stress and health readings still work.</p></Card>
 {:else if !ctx.readiness}{@render rh()}<Card><p>No readiness for {dayLabel(ctx.displayedDay)} yet.</p><p class="small">{ctx.phase === 'waiting' ? 'It arrives after last night syncs.' : 'Your watch hasn’t uploaded a reading.'}</p></Card>
 {:else}<ReadinessDetail todayData={todayView(ctx)} header={rh} bind:selectedFactor/>
  <Card title="Readiness history" onclick={() => go('metric/readiness')}><p class="small">Ranges and comparisons</p></Card>{/if}

{:else if key === 'training'}
 {#snippet th()}<Header title={sub === 'performance' ? 'Performance' : 'Training'} eyebrow="Last 7 days · Garmin" back={() => store.back()} backLabel={store.parentLabel}/>{/snippet}
 <TrainingDetail trainingData={trainingView(ctx)} header={th} navigate={go}/>
 <section class="card"><button class="rowlink" onclick={() => go('help/acute-load')}><Icon name="info" size={18}/><span><b>What acute load is</b><small>Context, not a budget</small></span><Icon name="chevron" size={16}/></button><button class="rowlink" onclick={() => go('metric/acuteLoad')}><Icon name="trend-up" size={18}/><span><b>Acute load trend</b><small>With Garmin’s range</small></span><Icon name="chevron" size={16}/></button><button class="rowlink" onclick={() => go('activities')}><Icon name="run" size={18}/><span><b>All activities</b><small>{ctx.world.activities.length} in available history</small></span><Icon name="chevron" size={16}/></button></section>

{:else if key === 'gear'}
 {@render header('Gear', 'Garmin gear list · fetched Wed 18 Nov 07:12')}
 <GearCard gear={GEAR as never}/>

{:else if key === 'strength' && sub === 'muscles'}
 {@render header('Muscles', 'Recorded sets')}
 <div class="mgrid">
  {#each [['This week', ctx.weekFrom, ctx.calendarDate, 12], ['Last 4 weeks', addDays(ctx.weekFrom, -21), ctx.calendarDate, 30], ['Last 12 weeks', addDays(ctx.weekFrom, -77), ctx.calendarDate, 80]] as [label, from, to, max]}
   <Card title={String(label)}><BodyMap figure="female" muscles={musclesBetween(ctx, String(from), String(to))} width={150} max={Number(max)}/></Card>
  {/each}
 </div>
 <p class="small">Recorded work only. <button class="link inline" onclick={() => go('help/strength-muscles')}>About the map</button></p>

{:else if key === 'strength'}
 {@render header(sub ? titleOf(route) : 'Strength', 'Last 12 weeks')}
 {#if exerciseView}<StrengthHistory view={exerciseView} navigate={go}/>
  <p class="small best-note">Bests are the best in available history, since {dayLabel(ctx.world.historyStart)}.</p>
 {:else}<Card title="No sets yet"><p class="small">Start a Strength activity on your watch and press lap after each set.</p><button class="link" onclick={() => go('help/strength-start')}>Lift with your watch</button></Card>{/if}

{:else if key === 'activities'}
 {@render header('Activities', `${ctx.world.activities.length} recorded · since ${dayLabel(ctx.world.historyStart)}`)}
 <div class="chiprow filt" role="group" aria-label="Filter by sport">
  {#each sports as s}<button class="chipb" aria-pressed={filter === s} onclick={() => store.setUi({filter: s, limit: 20})}>{#if s !== 'all'}<Glyph sport={s as Sport} size={20}/>{/if}{s === 'all' ? 'All' : SPORTS[s as Sport].name}</button>{/each}
  {#if date}<button class="chipb" aria-pressed="true" onclick={() => store.setUi({date: null})}>{dayLabel(date)} <Icon name="close" size={14}/><span class="sr">Clear date</span></button>{/if}
 </div>
 {#if ctx.world.coverageIncompleteFrom}<div class="notice-strip"><Icon name="info" size={16}/><p>Garmin’s activity list is incomplete since {dayLabel(ctx.world.coverageIncompleteFrom)}</p></div>{/if}
 {#each acts.slice(0, limit) as a, i (a.id)}
  {#if i === 0 || weekStart(a.date) !== weekStart(acts[i - 1].date)}<div class="sec-h"><h2>{weekRange(weekStart(a.date))}</h2></div>{/if}
  <ActivityTile {a} {ctx} navigate={go} showDate/>
 {:else}<p class="small">No recorded activities match.</p>{/each}
 {#if acts.length > limit}<button class="secondary" onclick={() => store.setUi({limit: limit + 40})}>Show more ({acts.length - limit})</button>{/if}

{:else if key === 'health' && sub === 'weight'}
 {@render header('Weight', 'Garmin weigh-ins · kg')}
 <Card title="Last weigh-in"><div class="big n">74.2<span class="unit">kg</span></div><p class="small">Sat 14 Nov · body fat 19.8%</p><div class="wspark"><TrendSpark values={weekly} w={320} h={80} sparse decimals={1}/></div><p class="small">12 weeks · weeks without a weigh-in stay blank</p></Card>
 <Card onclick={() => go('metric/weight')}><p class="small">Trend and comparisons</p></Card>

{:else if key === 'health'}
 <Header title="Health" eyebrow={`${dayLabel(ctx.displayedDay)} · Garmin`} back={() => store.back()} backLabel={store.parentLabel}/>
 {#if store.state.vitalsException}<div class="notice-strip"><Icon name="alert" size={18}/><p>Two overnight readings are outside your usual ranges</p></div>{/if}
 <HealthDetail healthData={healthView(ctx)} navigate={go} toggleJournal={on => on ? store.openSheet('consent') : (store.set(s => ({journal: {...s.journal, consent: false, open: false}}), 'Journal reads off'))} bind:journalOpen={() => store.state.journal.open, v => store.set(st => ({journal: {...st.journal, open: v}}))} bind:journalDayOpen/>
 {#if !store.state.journal.consent && store.state.journal.downloaded}<p class="small">Downloaded journal days stay on this phone and in exports you create until you erase local data.</p>{/if}
 <section class="card"><button class="rowlink" onclick={() => go('metric/bodyBattery')}><Icon name="bolt" size={18}/><span><b>Body Battery {ctx.bodyBattery ?? '—'}</b><small>At watch upload · {ctx.upload.detail}</small></span><Icon name="chevron" size={16}/></button><button class="rowlink" onclick={() => go('health/weight')}><Icon name="scale" size={18}/><span><b>Weight</b><small>Private to Health</small></span><Icon name="chevron" size={16}/></button><button class="rowlink" onclick={() => go('settings/journal')}><Icon name="info" size={18}/><span><b>Journal privacy</b><small>Settings</small></span><Icon name="chevron" size={16}/></button></section>

{:else if key === 'metric'}
 {@const m = METRICS[sub]}
 <Header title={m.name} eyebrow={m.source.toUpperCase()} back={() => store.back()} backLabel={store.parentLabel}/>
 {#if m.shipped}
  {#if sub === 'bodyBattery' && ctx.bodyBattery != null}{@const bb = bodyBatteryDay(ctx)}<Card title="Body Battery · through the day"><TrendSpark values={bb.points.map(p => p[1])} w={320} h={110} color="#3DDC97"/><p class="small">22:00 yesterday → {ctx.clock.time} · +{bb.charged} overnight · Garmin</p></Card>{/if}
  <MetricDetail metric={sub} data={['vo2max', 'endurance', 'hill', 'runningTolerance'].includes(sub) ? trainingView(ctx) : ['bodyBattery', 'stress'].includes(sub) ? todayView(ctx) : healthView(ctx)}/>
 {:else}
  {@const [v, note] = metricValue(sub)}
  <div class="metric-hero"><div class="metric-value n">{v}{#if m.unit && m.unit !== 'h' && v !== '—'}<span class="unit">{m.unit}</span>{/if}</div><p class="small">{note}</p></div>
  {#if m.trend}<Card title="History"><HistoryChart values={[]} endDate={ctx.calendarDate} label={m.name} unit={m.unit === 'h' ? 'min' : m.unit} metric={m.trend} initialRange={sub === 'fitnessAge' ? 'year' : 'days30'} nights={sub.startsWith('sleep')}/></Card>
  {:else if sub === 'weight'}<Card title="12 weeks"><TrendSpark values={weekly} w={320} h={90} sparse decimals={1}/></Card>{/if}
  {#if sub === 'fitnessAge'}<Card title="Garmin’s drivers"><dl class="kv"><dt>VO₂ max</dt><dd>47</dd><dt>Resting HR</dt><dd>54 bpm</dd><dt>Body fat</dt><dd>19.8%</dd><dt>Vigorous · 4-week average</dt><dd>62 min/wk</dd></dl><p class="small">Garmin’s attainable estimate: 39</p></Card>{/if}
  {#if sub === 'acuteLoad'}<Card onclick={() => go('help/acute-load')} title="What acute load is"><p class="small">Context, not a budget</p></Card>{/if}
 {/if}

{:else if key === 'trends'}
 {@render header('Trends', 'From the history on this phone')}
 <section class="card">{#each Object.entries(METRICS) as [id, m]}<button class="rowlink" onclick={() => go(`metric/${id}`)}><span class="tr-i"><Icon name={id === 'weight' ? 'scale' : 'trend-up'} size={22}/></span><span><b>{m.name}</b><small>{m.area} · {m.source}</small></span><Icon name="chevron" size={16}/></button>{/each}</section>

{:else if key === 'intention' || key === 'scheduled' || route === 'plan/commitments' || key === 'review' || key === 'reviews'}
 <PlanDetail {route}/>
{:else if key === 'adapt'}
 <Adapt {route}/>
{:else if key === 'settings'}
 <Settings {route}/>
{:else}
 <Misc {route}/>
{/if}

<style>
 .selfchip{display:inline-flex;align-items:center;gap:6px;padding:4px 10px 4px 4px;border-radius:999px;border:1px dashed rgba(228,232,240,.3);font-size:12px}
 .mgrid{display:grid;gap:0}
 .mgrid :global(.card){display:flex;flex-direction:column;align-items:center}
 .best-note{margin:12px 4px}
 .filt{margin-top:10px}
 .sr{position:absolute;width:1px;height:1px;overflow:hidden;clip:rect(0 0 0 0)}
 .wspark{margin-top:12px}
 .tr-i{width:56px}
</style>
