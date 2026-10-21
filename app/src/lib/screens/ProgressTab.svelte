<script lang="ts">
 import {onDestroy, untrack, type Snippet} from 'svelte';
 import Icon from '../components/Icon.svelte';
 import NestedRings from '../components/NestedRings.svelte';
 import DayCells from '../components/DayCells.svelte';
 import AxisChart from '../components/AxisChart.svelte';
 import HelpButton from '../components/HelpButton.svelte';
 import RingLegend from '../components/RingLegend.svelte';
 import type {ProgressDomains} from '../generated/ProgressDomains';
 import type {DomainQuery} from '../generated/DomainQuery';
 import type {DomainWeek} from '../generated/DomainWeek';
 import type {DomainMonth} from '../generated/DomainMonth';
 import type {ProgressView} from '../generated/ProgressView';
 import type {Period} from '../generated/Period';
 import type {PeriodTotals} from '../generated/PeriodTotals';
 import {GOALS, SPORT_COLORS, SPORT_NAMES} from '../goals';
 import {dateLabel, dateLabelIn, dayMonthYear, hoursMinutes, monthLabel, num, shortMonth, signed, signedTime} from '../format';
 import {addDays} from '../history';
 import * as api from '../api';
 // Progress: a report for a chosen period, the calendar, goals over time, trends
 // and milestones. Follow-through and measured change stay apart; nothing is a grade.
 let {initial, navigate, header, onChange, section = $bindable('overall'), domainQuery = $bindable({weeks: 12, weeksEnd: null, months: 12, monthsEnd: null})}: {initial: ProgressView, navigate: (route: string) => void, header: Snippet, onChange:(view:ProgressView)=>void, section?: 'overall' | 'cardio' | 'strength' | 'long', domainQuery?: DomainQuery} = $props();
 const SECTIONS = [['overall', 'Overall'], ['cardio', 'Cardio'], ['strength', 'Strength'], ['long', 'Long-term']] as const;
 // Cardio and Strength choose plan weeks, Long-term calendar months; each window ends with the week or month of its end date.
 const WEEK_SPANS = [[12, '12 weeks'], [26, '6 months'], [52, 'Year']] as const;
 const MONTH_SPANS = [[12, 'Year'], [24, '2 years'], [60, '5 years']] as const;
 // Cardio, Strength and Long-term read one local view, once per visit and chosen window.
 let domains = $state<ProgressDomains | null>(null), domainsError = $state(''), domainsBusy = $state(false);
 let domainsRead = '', domainsRequest = 0;
 $effect(() => {
  const key = JSON.stringify(domainQuery);
  if (section === 'overall' || key === domainsRead) return;
  const id = ++domainsRequest, query = $state.snapshot(domainQuery);
  domainsRead = key; domainsBusy = true; domainsError = '';
  api.progress_domains(query).then(d => { if (id === domainsRequest) domains = d; })
   .catch(e => { if (id === domainsRequest) { domainsError = String(e); domainsRead = ''; } })
   .finally(() => { if (id === domainsRequest) domainsBusy = false; });
 });
 let weekly = $derived(section !== 'long');
 // Back and forward move a whole window; one reaching this week or month follows today again.
 const firstOfMonth = (date: string, months: number) => { const d = new Date(date.slice(0, 7) + '-01T12:00:00Z'); d.setUTCMonth(d.getUTCMonth() + months); return d.toISOString().slice(0, 10); };
 function shiftDomain(n: number) {
  const today = view.today;
  if (weekly) { const end = addDays(domainQuery.weeksEnd ?? today, 7 * domainQuery.weeks * n); domainQuery = {...domainQuery, weeksEnd: end >= today ? null : end}; }
  else { const end = firstOfMonth(domainQuery.monthsEnd ?? today, domainQuery.months * n); domainQuery = {...domainQuery, monthsEnd: end.slice(0, 7) >= today.slice(0, 7) ? null : end}; }
 }
 const dayMonth = (date: string) => `${Number(date.slice(8))} ${shortMonth(date)}`;
 let domainLabel = $derived(!domains ? '' : weekly ? `${domains.weeksFrom.slice(0, 4) === domains.weeksTo.slice(0, 4) ? dayMonth(domains.weeksFrom) : dayMonthYear(domains.weeksFrom)} – ${dayMonthYear(domains.weeksTo)}` : `${monthLabel(domains.monthsFrom)} – ${monthLabel(domains.monthsTo)}`);
 // About four date ticks whatever the window; values only while bars have room for them.
 let weekEvery = $derived(Math.max(1, Math.ceil((domains?.weeks.length ?? 0) / 4)));
 let longMonths = $derived((domains?.months.length ?? 0) > 12);
 let monthEvery = $derived(longMonths ? ((domains?.months.length ?? 0) > 24 ? 12 : 6) : 0);
 // The current week and month are unfinished: outlined, and labelled so.
 const wkLabel = (w: DomainWeek) => w.current ? 'This week' : `${Number(w.start.slice(8))} ${shortMonth(w.start)}`;
 // Ticks stay dates (the outline marks the current one); text labels name it.
 const weekPoints = (pick: (w: DomainWeek) => number | null) => (domains?.weeks ?? []).map(w => ({x: `${Number(w.start.slice(8))} ${shortMonth(w.start)}`, v: pick(w), current: w.current, mark: w.mark === 'rest' || w.mark == null ? undefined : w.mark}));
 const monthPoints = (pick: (m: DomainMonth) => number | null) => (domains?.months ?? []).map((m) => ({x: longMonths ? monthLabel(m.start) : shortMonth(m.start), v: pick(m), current: m.start.slice(0, 7) === domains?.today.slice(0, 7)}));
 // Each week against its own target, a stepped line; none before goals began.
 type Cardio = {kind: 'zone2' | 'highIntensity' | 'intensity', title: string, pick: (w: DomainWeek) => number | null, goal: (w: DomainWeek) => number | null};
 let cardioKinds = $derived.by(() => { const weeks = domains?.weeks ?? []; const out: Cardio[] = [{kind: 'zone2', title: 'Zone 2 minutes per week', pick: w => w.zone2Min, goal: w => w.zone2Goal}];
  if (weeks.some(w => w.highIntensityGoal)) out.push({kind: 'highIntensity', title: 'Zone 4–5 minutes per week', pick: w => w.highIntensityMin, goal: w => w.highIntensityGoal});
  out.push({kind: 'intensity', title: 'Intensity minutes per week', pick: w => w.intensityMin, goal: w => w.intensityGoal}); return out; });
 const latestGoal = (goal: (w: DomainWeek) => number | null) => (domains?.weeks ?? []).map(goal).findLast(g => g != null) ?? null;
 let view = $state<ProgressView>(untrack(() => initial));
 let busy = $state(false), error = $state('');
 let request = 0;
 $effect(() => { view = initial; request++; busy = false; });
 onDestroy(()=>{request++});
 const PERIODS: [Period, string][] = [['week', 'Week'], ['month', 'Month'], ['quarter', '3 months'], ['year', 'Year']];
 async function show(period: Period, date: string) {
  const id = ++request;
  busy = true; error = '';
  try { const next = await api.progress(period, date); if(id === request){view = next;onChange(next);} }
  catch(e){ if(id === request)error = String(e); }
  finally { if(id === request)busy = false; }
 }
 // A day opens its own Health page; a future day still opens the plan.
 const openCalendarDay = (date: string) => navigate(date < view.today ? 'health/' + date : date > view.today ? 'plan' : 'health');
 const shift = (n: number) => {
  const d = new Date(view.to + 'T12:00:00Z');
  if(view.period==='week')d.setUTCDate(d.getUTCDate()+7*n);
  else { d.setUTCDate(1); d.setUTCMonth(d.getUTCMonth()+n*({month:1,quarter:3,year:12}[view.period])); }
  const date = d.toISOString().slice(0,10);
  void show(view.period,date>view.today?view.today:date);
 };
 let label = $derived(view.period === 'week' ? `${dateLabel(view.from)} – ${dateLabel(view.to)}` : view.period === 'month' ? new Date(view.from + 'T12:00:00Z').toLocaleDateString('en-GB', {month: 'long', year: 'numeric', timeZone: 'UTC'}) : `${new Date(view.from + 'T12:00:00Z').toLocaleDateString('en-GB', {month: 'short', year: 'numeric', timeZone: 'UTC'})} – ${new Date(view.to + 'T12:00:00Z').toLocaleDateString('en-GB', {month: 'short', year: 'numeric', timeZone: 'UTC'})}`);
 const rows = (t: PeriodTotals) => [['Activities', t.activities, ''], ['Active time', t.activeMin, 'h'], ['Distance', t.distanceKm, 'km'], ['Intensity', t.intensityMin, 'min']] as const;
 const fmt = (v: number | null, unit: string) => v == null ? '—' : unit === 'h' ? hoursMinutes(v) : unit === 'km' ? num(v, 1) : num(v);
 const change = (now: number | null, before: number | null, unit: string) => {
  if (now == null || before == null) return null;
  if (unit === 'h') return signedTime(now - before) || null;
  const d = signed(Math.round((now - before) * 10) / 10);
  return d ? `${d}${unit ? ' ' + unit : ''}` : null;
 };
 // Minutes by sport per bucket; hours once buckets are weeks or months. Days
 // to come have no bar and no track; an unlisted day is outlined as unknown.
 let hours = $derived(view.period === 'quarter' || view.period === 'year');
 const bucketLabel = (from: string) => view.period === 'year' ? shortMonth(from) : view.period === 'quarter' ? `${Number(from.slice(8))} ${shortMonth(from)}` : String(Number(from.slice(8)));
 let activity = $derived(view.buckets.map(b => ({x: bucketLabel(b.from), v: b.future || !b.known ? null : b.total / (hours ? 60 : 1),
  parts: b.minutes.map(m => ({v: m.minutes / (hours ? 60 : 1), color: SPORT_COLORS[m.sport]})), mark: !b.known && !b.future ? 'unknown' as const : undefined,
  current: view.current && b.from <= view.today && b.to >= view.today})));
 async function markSeen(){
  const id=++request;
  try { await api.local_action({action:'milestonesSeen'}); if(id===request){view = {...view,milestones:view.milestones.map(m=>({...m,new:false}))};onChange(view);} }
  catch(e){error=String(e)}
 }
 const STATE: Record<string, string> = {active: 'active', rest: 'rest day', ill: 'ill', paused: 'paused', none: 'no activity', unknown: 'activity list incomplete', future: '', before: 'before your downloaded history'};
 let lead = $derived(view.calendar.length ? (new Date(view.calendar[0].date + 'T12:00:00Z').getUTCDay() + 6) % 7 : 0);
</script>
{@render header()}
<div class="prog-tabs" role="tablist" aria-label="Progress section">{#each SECTIONS as [k, l]}<button role="tab" aria-selected={section === k} onclick={() => section = k}>{l}</button>{/each}</div>
{#if section === 'overall'}
<div class="periods" role="group" aria-label="Period">{#each PERIODS as [p, name]}<button aria-pressed={view.period === p} disabled={busy || api.previewOnly} onclick={() => show(p, view.end)}>{name}</button>{/each}</div>
{#if api.previewOnly}<p class="small">Browser preview · fixed synthetic period. Period controls work in the app.</p>{/if}
{#if error}<p class="warning" role="alert">{error}</p>{/if}
<section class="card report" aria-busy={busy}>
 <div class="rep-h"><button class="nav" aria-label="Previous period" disabled={busy || api.previewOnly} onclick={() => shift(-1)}><Icon name="arrow-left" size={18}/></button><h2>{label}</h2><button class="nav" aria-label="Next period" disabled={view.current || busy || api.previewOnly} onclick={() => shift(1)}><Icon name="arrow-right" size={18}/></button></div>
 {#if view.previous}<p class="small cmp">Compared with {dateLabel(view.previousFrom!)} – {dateLabel(view.previousTo!)}</p>{/if}
 <div class="totals">{#each rows(view.totals) as [name, v, unit], i}{@const before = view.previous && (i!==3 || view.totals.intensityDays===view.totals.days && view.previous.intensityDays===view.previous.days) ? rows(view.previous)[i][1] : null}<div><span>{name}</span><b class="n">{fmt(v, unit)}{#if unit && unit !== 'h' && v != null}<small>{unit}</small>{/if}</b>{#if change(v, before, unit)}<em>{change(v, before, unit)}</em>{/if}</div>{/each}</div>
 <p class="small">Active on {view.totals.activeDays} of {view.totals.coveredDays} days{view.partial ? ' · known totals: Garmin’s activity list is incomplete for some days' : ''}</p>
 {#if view.totals.intensityDays < view.totals.days}<p class="small">Intensity: known total from {view.totals.intensityDays} of {view.totals.days} days.</p>{/if}
 {#if view.sleep}<p class="small">Sleep: {view.sleep.belowHour} of {view.sleep.nights} recorded nights an hour or more below Garmin’s need.</p>{/if}
 <AxisChart points={activity} unit={hours ? 'h' : 'min'} height={150} values={view.period === 'week' || view.period === 'year'} every={view.period === 'month' ? 5 : view.period === 'quarter' ? 3 : 1} fmt={v => hours ? num(v, v < 10 ? 1 : 0) : String(Math.round(v))}
  label={`Activity by ${view.period === 'year' ? 'month, hours' : view.period === 'quarter' ? 'week, hours' : 'day, minutes'}: ${view.buckets.filter(b => !b.future).map(b => dateLabel(b.from) + ': ' + (b.known ? (hours ? num(b.total / 60, 1) + ' h' : Math.round(b.total) + ' min') : 'incomplete')).join('; ')}`}/>
 {#if view.buckets.some(b => b.total)}<p class="legend">{#each [...new Set(view.buckets.flatMap(b => b.minutes.map(m => m.sport)))] as s}<span><i style:background={SPORT_COLORS[s]}></i>{SPORT_NAMES[s]}</span>{/each}</p>{/if}
</section>
{#if view.period === 'week' && view.current}
 <!-- This week's days are Plan's day strip. -->
{:else if view.ringDays.some(d => d.rings.length)}
 <section class="card ring-cal"><h3><span>Calendar</span><HelpButton topic="rings"/></h3>
  <DayCells days={view.ringDays.filter((_, i, all) => all.slice(Math.floor(i / 7) * 7, Math.floor(i / 7) * 7 + 7).some(d => d.date >= view.from && d.date <= view.to))} select={openCalendarDay} size={28} within={[view.from, view.to]}/>
  <RingLegend kinds={view.ringDays.flatMap(d => d.rings.map(r => r.kind))}/>
 </section>
{:else if view.calendar.length}
 <section class="card"><h3>Calendar</h3>
  <div class="cal">{#each Array(lead) as _}<span></span>{/each}{#each view.calendar as d}<button class="day {d.state}" class:uncovered={!d.covered && d.state === 'active'} disabled={d.state === 'future' || d.state === 'before'} aria-label={`${dateLabel(d.date)}${STATE[d.state] ? ', ' + STATE[d.state] : ''}${d.activities > 1 ? `, ${d.activities} activities` : ''}`} onclick={() => openCalendarDay(d.date)}>{Number(d.date.slice(8))}</button>{/each}</div>
  <p class="legend"><span><i class="k active"></i>Active</span><span><i class="k rest"></i>Rest</span><span><i class="k paused"></i>Ill or paused</span><span><i class="k unknown"></i>List incomplete</span></p>
 </section>
{/if}
{#if view.goalWeeks.length}
 <!-- Each passed week opens its review; this week opens Plan. Weeks that didn't count are faded and say why. -->
 <section class="card"><h3><span>Weekly goals over time</span><HelpButton topic="progress-overall"/></h3>
  <div class="gw">{#each view.goalWeeks as w}<button class:uncounted={!w.counted && !w.current} class:current={w.current} onclick={() => navigate(w.current ? 'plan' : 'review/' + w.start)} aria-label={`Week of ${dateLabel(w.start)}: ${w.current ? 'this week, open Plan' : w.counted ? `${w.met} of ${w.goals} goals met` : 'not counted' + (w.reasons.length ? ', ' + w.reasons.join(', ') : '')}`}><NestedRings rings={w.rings} size={34} stroke={3} mark={w.mark} dim={!w.counted && !w.current}/><small>{w.current ? 'This week' : `${Number(w.start.slice(8))} ${shortMonth(w.start)}`}</small></button>{/each}</div>
  <RingLegend kinds={view.goalWeeks.flatMap(w => w.rings.map(r => r.kind))}/>
  <button class="link-row all-weeks" onclick={() => navigate('reviews')}><span><b>All weekly reviews</b><small>Each week against its own goals</small></span><Icon name="chevron" size={16}/></button>
 </section>
{/if}
<div class="sec-h"><h2>Records and milestones</h2></div>
{#if view.milestones.length}
 {#if view.milestones.some(m=>m.new)}<button class="secondary" onclick={markSeen}>Mark records seen</button>{/if}
 <section class="card">{#each view.milestones as m}<button class="ms" disabled={!m.route} onclick={() => m.route && navigate(m.route)}><span><b>{m.title}</b><small>{[m.value, dateLabelIn(m.date, view.today)].filter(Boolean).join(' · ')}{m.corrected ? ' · corrected after a Garmin edit' : ''}</small></span>{#if m.new}<em class="new">New</em>{/if}{#if m.route}<Icon name="chevron" size={15}/>{/if}</button>{/each}
  {#if view.historyStart}<p class="small">In available history, since {dayMonthYear(view.historyStart)}.</p>{/if}<button class="link-row archive" onclick={() => navigate('progress/milestones')}><span><b>All records and milestones</b><small>{view.milestonesTotal > view.milestones.length ? `${view.milestonesTotal} in all, by month` : 'By month, with dates'}</small></span><Icon name="chevron" size={16}/></button></section>
{:else}<p class="small foot">Bests and other milestones appear here as Garmin records them.</p>{/if}
{:else}
<div class="periods" role="group" aria-label="Period">{#each weekly ? WEEK_SPANS : MONTH_SPANS as [n, name]}<button aria-pressed={(weekly ? domainQuery.weeks : domainQuery.months) === n} disabled={api.previewOnly} onclick={() => domainQuery = weekly ? {...domainQuery, weeks: n} : {...domainQuery, months: n}}>{name}</button>{/each}</div>
{#if api.previewOnly}<p class="small">Browser preview · fixed synthetic period. Period controls work in the app.</p>{/if}
{#if domainsError}<p class="warning" role="alert">{domainsError}</p>{/if}
{#if domains}<div class="rep-h dom-range" aria-busy={domainsBusy}><button class="nav" aria-label="Previous period" disabled={api.previewOnly} onclick={() => shiftDomain(-1)}><Icon name="arrow-left" size={18}/></button><h2>{domainLabel}</h2><button class="nav" aria-label="Next period" disabled={api.previewOnly || (weekly ? domainQuery.weeksEnd : domainQuery.monthsEnd) == null} onclick={() => shiftDomain(1)}><Icon name="arrow-right" size={18}/></button></div>{/if}
{#if !domains}{#if !domainsError}<p class="small">Reading your history…</p>{/if}
{:else if weekly && !domains.weeks.length}<p class="small foot">No weeks of downloaded history in these dates.</p>
{:else if section === 'cardio'}
 {#each cardioKinds as c (c.kind)}{@const now = latestGoal(c.goal)}{@const label = c.kind === 'intensity' && domains.goals?.intensityGarminGoal ? 'Garmin goal' : 'goal'}
  <section class="card dom" aria-label={c.title}><h3><span>{c.title}</span>{#if now != null}<small>{label} {now}</small>{/if}</h3>
   <AxisChart points={weekPoints(c.pick)} unit="min" goals={domains.weeks.map(c.goal)} color={GOALS[c.kind].color} height={140} every={weekEvery} values={domains.weeks.length <= 13} label={`${c.title} against each week's ${label}, ${domains.weeks.length} weeks: ${domains.weeks.map(w => `${wkLabel(w)} ${c.pick(w) ?? 'unknown'}${c.goal(w) != null ? ` of ${c.goal(w)}` : ''}`).join(', ')}`}/>
  </section>
 {/each}
 <section class="card dom" aria-label="VO₂ max, same weeks"><h3><span>VO₂ max, same weeks</span><button class="link" onclick={() => navigate('metric/vo2max')}>Details</button></h3>
  <AxisChart points={weekPoints(w => w.vo2max)} kind="dots" unit="ml/kg/min" color="var(--neutral)" height={120} every={weekEvery} label={`Garmin's VO₂ max estimates by week: ${weekPoints(w => w.vo2max).map(p => `${p.x}${p.current ? ' (so far)' : ''} ${p.v ?? 'none'}`).join(', ')}`}/>
 </section>
{:else if section === 'strength'}
 {@const now = latestGoal(w => w.strengthGoal)}
 <section class="card dom" aria-label="Strength sessions per week"><h3><span>Strength sessions per week</span>{#if now != null}<small>goal {now}</small>{/if}</h3>
  <AxisChart points={weekPoints(w => w.strength)} unit="sessions" goals={domains.weeks.map(w => w.strengthGoal)} integer color={GOALS.strength.color} height={130} every={weekEvery} values={domains.weeks.length <= 13} label={`Strength sessions per week against each week's goal: ${domains.weeks.map(w => `${wkLabel(w)} ${w.strength ?? 'unknown'}${w.strengthGoal != null ? ` of ${w.strengthGoal}` : ''}`).join(', ')}`}/>
 </section>
 <button class="card link-row" onclick={() => navigate('strength')}><span><b>Bests and lifts</b><small>Training · Strength: every exercise, its sets and its history</small></span><Icon name="chevron" size={16}/></button>
{:else}
 <section class="card dom" aria-label="Training per month"><h3><span>Training per month</span><button class="link" onclick={() => navigate('training')}>Details</button></h3>
  <AxisChart points={monthPoints(m => m.minutes == null ? null : m.minutes / 60)} unit="h" color="var(--goal-intensity)" height={120} fmt={v => num(v, 1)} every={monthEvery} values={!longMonths} label={`Hours recorded per month: ${monthPoints(m => m.minutes).map(p => `${p.x}${p.current ? ' (so far)' : ''} ${p.v == null ? 'unknown' : num(p.v / 60, 1)}`).join(', ')}`}/>
 </section>
 <section class="card dom" aria-label="Resting heart rate by month"><h3><span>Resting heart rate</span><button class="link" onclick={() => navigate('metric/heartRate')}>Details</button></h3><AxisChart points={monthPoints(m => m.restingHr)} kind="line" unit="bpm" color="var(--neutral)" height={120} every={monthEvery} label={`Resting heart rate monthly averages: ${monthPoints(m => m.restingHr).map(p => `${p.x}${p.current ? ' (so far)' : ''} ${p.v == null ? 'none' : num(p.v, 1)}`).join(', ')}`}/><p class="small">Monthly average of Garmin's daily readings.</p></section>
 <section class="card dom" aria-label="Overnight HRV by month"><h3><span>Overnight HRV</span><button class="link" onclick={() => navigate('metric/hrv')}>Details</button></h3><AxisChart points={monthPoints(m => m.hrv)} kind="line" unit="ms" color="var(--neutral)" height={120} every={monthEvery} label={`HRV monthly averages: ${monthPoints(m => m.hrv).map(p => `${p.x}${p.current ? ' (so far)' : ''} ${p.v == null ? 'none' : num(p.v, 1)}`).join(', ')}`}/><p class="small">Monthly average of Garmin's overnight readings.</p></section>
 <section class="card dom" aria-label="VO₂ max and Fitness Age by month"><h3><span>VO₂ max and Fitness Age</span><button class="link" onclick={() => navigate('health')}>Details</button></h3>
  <AxisChart points={monthPoints(m => m.vo2max)} kind="dots" unit="VO₂ max" color="var(--neutral)" height={110} every={monthEvery} label={`VO₂ max by month: ${monthPoints(m => m.vo2max).map(p => `${p.x}${p.current ? ' (so far)' : ''} ${p.v ?? 'none'}`).join(', ')}`}/>
  <AxisChart points={monthPoints(m => m.fitnessAge)} kind="dots" unit="years" color="var(--neutral)" height={110} every={monthEvery} label={`Fitness Age by month: ${monthPoints(m => m.fitnessAge).map(p => `${p.x}${p.current ? ' (so far)' : ''} ${p.v ?? 'none'}`).join(', ')}`}/>
  <p class="small">Garmin’s latest estimate each month; months without one stay empty.</p>
 </section>
{/if}
{#if domains}<p class="period-key">Outlined bar · current period so far{#if section==='cardio'||section==='strength'}<br/>Dashed line · each week's goal<br/>Hatched · ill or paused · dotted outline · incomplete data{/if}</p>{/if}
{/if}
<style>
 .period-key{font:400 12px/1.5 Inter;color:var(--text2);margin:16px 4px}
 .prog-tabs{display:grid;grid-template-columns:repeat(4,1fr);gap:4px;padding:4px;margin:8px 0 10px;border:1px solid var(--line);border-radius:14px;background:var(--card);max-width:560px}
 .prog-tabs button{min-height:40px;border-radius:10px;border:0;background:none;color:var(--text2);font:600 12.5px Inter}.prog-tabs button[aria-selected=true]{background:rgba(255,255,255,.1);color:var(--text)}
 .dom h3{display:flex;justify-content:space-between;align-items:baseline;gap:8px}.dom h3 small{font:500 12px Inter;color:var(--text2);text-transform:none;letter-spacing:0}
 .link{background:none;border:0;color:var(--ink);font:600 13px Inter;min-height:40px}
 .periods{display:flex;gap:6px;margin:6px 0 0}.periods button{flex:1;min-height:40px;border-radius:12px;border:1px solid var(--line);background:none;color:var(--text2);font:600 13px Inter}.periods button[aria-pressed=true]{background:var(--ink);color:var(--bg)}
 .rep-h{display:grid;grid-template-columns:44px 1fr 44px;align-items:center;text-align:center}.dom-range{margin:12px 0 0}.rep-h h2{font:600 18px Outfit}.nav{width:44px;height:44px;border-radius:50%;border:1px solid var(--line);background:none;color:var(--text);display:grid;place-items:center}.nav:disabled{color:var(--text3);border-color:transparent}
 .totals{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:12px;margin-top:12px}.totals span{font:500 12px Inter;color:var(--text2)}.totals b{display:block;font-size:24px;font-weight:600}.totals small{font:500 12px Inter;color:var(--text2);margin-left:3px}.totals em{display:block;font-style:normal;font-size:12px;color:var(--text2)}
 .cmp{margin:2px 0 0;text-align:center}
 @media (max-width:380px){.totals{grid-template-columns:minmax(0,1fr)}}
 .legend{display:flex;flex-wrap:wrap;gap:10px;margin:8px 0 0;font-size:12px;color:var(--text2)}.legend span{display:flex;align-items:center;gap:5px}.legend i{width:9px;height:9px;border-radius:50%;display:inline-block}
 .cal{display:grid;grid-template-columns:repeat(7,minmax(0,1fr));gap:4px}.day{display:grid;place-items:center;height:34px;border-radius:10px;border:0;padding:0;font:600 12px Outfit;color:var(--text2);background:rgba(255,255,255,.03);cursor:pointer}.day:disabled{cursor:default}
 .day.active{background:rgba(61,220,151,.22);color:var(--text)}.day.uncovered{outline:1px dashed var(--text3)}.day.rest{background:rgba(169,200,255,.14)}.day.ill,.day.paused{background:repeating-linear-gradient(45deg,rgba(169,200,255,.22) 0 3px,transparent 3px 6px)}.day.unknown{outline:1px dotted var(--text3);background:none}.day.future,.day.before{background:none;color:var(--text3)}
 .k{width:12px;height:12px;border-radius:4px}.k.active{background:rgba(61,220,151,.4)}.k.rest{background:rgba(169,200,255,.3)}.k.paused{background:repeating-linear-gradient(45deg,rgba(169,200,255,.5) 0 2px,transparent 2px 4px)}.k.unknown{outline:1px dotted var(--text3)}
 .gw{display:flex;gap:4px;overflow-x:auto;padding-bottom:4px}.gw button{display:grid;justify-items:center;gap:3px;background:none;border:0;border-radius:10px;color:var(--text2);min-width:46px;min-height:44px;padding:3px 0}.gw small{font-size:11px;white-space:nowrap}.gw .current{outline:1px solid rgba(255,255,255,.24);background:rgba(255,255,255,.05)}.gw .current small{color:var(--text)}.gw .uncounted small{color:var(--text3)}
 .all-weeks{border:0;border-top:1px solid var(--line);background:none;margin-top:10px;padding:12px 0 2px}
 .link-row{display:flex;justify-content:space-between;align-items:center;width:100%;text-align:left;color:var(--text)}.link-row b{display:block;font:600 16px Outfit}.link-row small{display:block;font-size:12px;color:var(--text2);margin-top:3px}
 .sec-h{margin:22px 4px 0}.sec-h h2{font:600 19px Outfit}
 .foot{margin:12px 4px}
 .ms{display:flex;justify-content:space-between;align-items:center;gap:8px;width:100%;padding:10px 0;border:0;border-bottom:1px solid var(--line);background:none;color:var(--text);text-align:left}.ms:disabled{cursor:default}.ms b{display:block;font:600 14px Inter}.ms small{display:block;font-size:12px;color:var(--text2);margin-top:2px}.new{font-style:normal;font:700 11px Inter;color:var(--bg);background:var(--ink);border-radius:999px;padding:3px 8px}
</style>
