<script lang="ts">
 import {untrack, type Snippet} from 'svelte';
 import Icon from '../components/Icon.svelte';
 import NestedRings from '../components/NestedRings.svelte';
 import type {HealthView} from '../generated/HealthView';
 import type {ProgressView} from '../generated/ProgressView';
 import type {Period} from '../generated/Period';
 import type {PeriodTotals} from '../generated/PeriodTotals';
 import {SPORT_COLORS, SPORT_NAMES} from '../goals';
 import {dateLabel, duration, num, signed} from '../format';
 import * as api from '../api';
 // Progress: a report for a chosen period, the calendar, goals over time, trends
 // and milestones. Follow-through and measured change stay apart; nothing is a grade.
 let {initial, health, navigate, header}: {initial: ProgressView, health: HealthView, navigate: (route: string) => void, header: Snippet} = $props();
 let view = $state<ProgressView>(untrack(() => initial));
 let busy = $state(false), error = $state('');
 let request = 0;
 $effect(() => { view = initial; request++; busy = false; });
 const PERIODS: [Period, string][] = [['week', 'Week'], ['month', 'Month'], ['quarter', '3 months'], ['year', 'Year']];
 async function show(period: Period, date: string) {
  const id = ++request;
  busy = true; error = '';
  try { const next = await api.progress(period, date); if(id === request)view = next; }
  catch(e){ if(id === request)error = String(e); }
  finally { if(id === request)busy = false; }
 }
 const shift = (n: number) => {
  const d = new Date(view.to + 'T12:00:00Z');
  if(view.period==='week')d.setUTCDate(d.getUTCDate()+7*n);
  else { d.setUTCDate(1); d.setUTCMonth(d.getUTCMonth()+n*({month:1,quarter:3,year:12}[view.period])); }
  const date = d.toISOString().slice(0,10);
  void show(view.period,date>initial.end?initial.end:date);
 };
 let label = $derived(view.period === 'week' ? `${dateLabel(view.from)} – ${dateLabel(view.to)}` : view.period === 'month' ? new Date(view.from + 'T12:00:00Z').toLocaleDateString('en-GB', {month: 'long', year: 'numeric', timeZone: 'UTC'}) : `${new Date(view.from + 'T12:00:00Z').toLocaleDateString('en-GB', {month: 'short', year: 'numeric', timeZone: 'UTC'})} – ${new Date(view.to + 'T12:00:00Z').toLocaleDateString('en-GB', {month: 'short', year: 'numeric', timeZone: 'UTC'})}`);
 let max = $derived(Math.max(1, ...view.buckets.map(b => b.total)));
 const rows = (t: PeriodTotals) => [['Activities', t.activities, ''], ['Active time', t.activeMin, 'h'], ['Distance', t.distanceKm, 'km'], ['Intensity', t.intensityMin, 'min']] as const;
 const fmt = (v: number | null, unit: string) => v == null ? '—' : unit === 'h' ? duration(v) : unit === 'km' ? num(v, 1) : num(v);
 const change = (now: number | null, before: number | null, unit: string) => now == null || before == null ? null : unit === 'h' ? `${now - before >= 0 ? '+' : '−'}${duration(Math.abs(now - before))}` : signed(Math.round((now - before) * 10) / 10);
 const vital = (kind: string) => health.vitals.readings.find(r => r.kind === kind);
 let tiles = $derived([
  {route: 'metric/vo2max', name: 'VO₂ max', value: null as number | null, unit: ''},
  {route: 'health', name: 'Fitness Age', value: health.fitnessAge.fitnessAge, unit: 'yrs'},
  {route: 'metric/heartRate', name: 'Resting HR', value: health.heartRateDay.resting, unit: 'bpm'},
  {route: 'metric/hrv', name: 'HRV', value: vital('hrv')?.value ?? null, unit: 'ms'},
  {route: 'metric/bodyBattery', name: 'Body Battery', value: null, unit: ''},
  {route: 'metric/stress', name: 'Stress', value: health.stress.average, unit: 'avg'},
 ]);
 async function markSeen(){
  try { await api.local_action({action:'milestonesSeen'}); view = {...view,milestones:view.milestones.map(m=>({...m,new:false}))}; }
  catch(e){error=String(e)}
 }
 const STATE: Record<string, string> = {active: 'active', rest: 'rest day', ill: 'ill', paused: 'paused', none: 'no activity', unknown: 'activity list incomplete', future: '', before: 'before your downloaded history'};
 let lead = $derived(view.calendar.length ? (new Date(view.calendar[0].date + 'T12:00:00Z').getUTCDay() + 6) % 7 : 0);
</script>
{@render header()}
<div class="periods" role="group" aria-label="Period">{#each PERIODS as [p, name]}<button aria-pressed={view.period === p} disabled={busy || api.previewOnly} onclick={() => show(p, view.end)}>{name}</button>{/each}</div>
{#if api.previewOnly}<p class="small">Browser preview · fixed synthetic period. Period controls work in the app.</p>{/if}
{#if error}<p class="warning" role="alert">{error}</p>{/if}
<section class="card report" aria-busy={busy}>
 <div class="rep-h"><button class="nav" aria-label="Previous period" disabled={busy || api.previewOnly} onclick={() => shift(-1)}><Icon name="arrow-left" size={18}/></button><h2>{label}</h2><button class="nav" aria-label="Next period" disabled={view.current || busy || api.previewOnly} onclick={() => shift(1)}><Icon name="arrow-right" size={18}/></button></div>
 <div class="totals">{#each rows(view.totals) as [name, v, unit], i}{@const before = view.previous && (i!==3 || view.totals.intensityDays===view.totals.days && view.previous.intensityDays===view.previous.days) ? rows(view.previous)[i][1] : null}<div><span>{name}</span><b class="n">{fmt(v, unit)}{#if unit && unit !== 'h' && v != null}<small>{unit}</small>{/if}</b>{#if change(v, before, unit)}<em>{change(v, before, unit)} vs {dateLabel(view.previousFrom!)}–{dateLabel(view.previousTo!)}</em>{/if}</div>{/each}</div>
 <p class="small">Active on {view.totals.activeDays} of {view.totals.coveredDays} days{view.partial ? ' · known totals: Garmin’s activity list is incomplete for some days' : ''}</p>
 {#if view.totals.intensityDays < view.totals.days}<p class="small">Intensity: known total from {view.totals.intensityDays} of {view.totals.days} days.</p>{/if}
 <p class="small">Activity minutes · scale 0–{Math.ceil(max)} min</p>
 <div class="bars" role="img" aria-label={`Minutes of activity by ${view.period === 'year' ? 'month' : view.period === 'quarter' ? 'week' : 'day'}: 0 to ${Math.ceil(max)} min. ${view.buckets.filter(b=>!b.future).map(b=>dateLabel(b.from)+': '+(b.known?Math.round(b.total)+' min':'incomplete')).join('; ')}`}>
  {#each view.buckets as b}<span class="bar" class:unknown={!b.known && !b.future} title={`${dateLabel(b.from)}: ${b.total} min`}>{#each b.minutes as m}<i style:height={m.minutes / max * 100 + '%'} style:background={SPORT_COLORS[m.sport]}></i>{/each}</span>{/each}
 </div>
 <p class="axis"><span>{dateLabel(view.from)}</span><span>{dateLabel(view.to)}</span></p>
 {#if view.buckets.some(b => b.total)}<p class="legend">{#each [...new Set(view.buckets.flatMap(b => b.minutes.map(m => m.sport)))] as s}<span><i style:background={SPORT_COLORS[s]}></i>{SPORT_NAMES[s]}</span>{/each}</p>{/if}
</section>
{#if view.calendar.length}
 <section class="card"><h3>Calendar</h3>
  <div class="cal">{#each Array(lead) as _}<span></span>{/each}{#each view.calendar as d}<span class="day {d.state}" class:uncovered={!d.covered && d.state === 'active'} aria-label={`${dateLabel(d.date)}${STATE[d.state] ? ', ' + STATE[d.state] : ''}${d.activities > 1 ? `, ${d.activities} activities` : ''}`}>{Number(d.date.slice(8))}</span>{/each}</div>
  <p class="legend"><span><i class="k active"></i>Active</span><span><i class="k rest"></i>Rest</span><span><i class="k paused"></i>Ill or paused</span><span><i class="k unknown"></i>List incomplete</span></p>
 </section>
{/if}
{#if view.goalWeeks.length}
 <section class="card"><h3>Weekly goals over time</h3>
  <div class="gw">{#each view.goalWeeks as w}<button class:uncounted={!w.counted} onclick={() => navigate('review/' + w.start)} aria-label={`Week of ${dateLabel(w.start)}: ${w.counted ? `${w.met} of ${w.goals} goals met` : 'not counted' + (w.reasons.length ? ', ' + w.reasons.join(', ') : '')}`}><NestedRings rings={w.rings} size={34} stroke={3} mark={w.counted ? null : 'unknown'}/><small>{Number(w.start.slice(8))}/{Number(w.start.slice(5, 7))}</small></button>{/each}</div>
 </section>
{/if}
<button class="card link-row" onclick={() => navigate('review/latest')}><span><b>Last week</b><small>What was met, what changed and next week’s goals</small></span><Icon name="chevron" size={16}/></button>
<button class="card link-row" onclick={() => navigate('reviews')}><span><b>Earlier weeks</b><small>Each week against its own goals</small></span><Icon name="chevron" size={16}/></button>
<div class="sec-h"><h2>Latest readings</h2></div>
<p class="small">{dateLabel(health.date)} · open a reading for its chart and dates.</p>
<div class="tiles2">{#each tiles as t}<button class="card tile2" onclick={() => navigate(t.route)}><span>{t.name}</span>{#if t.value != null}<b class="n">{num(t.value)}<small>{t.unit}</small></b>{:else}<em>Open chart</em>{/if}</button>{/each}</div>
{#if view.sleep}<p class="small foot">Sleep: {view.sleep.belowHour} of {view.sleep.nights} recorded nights an hour or more below Garmin’s need.</p>{/if}
<div class="sec-h"><h2>Records and milestones</h2></div>
{#if view.milestones.length}
 {#if view.milestones.some(m=>m.new)}<button class="secondary" onclick={markSeen}>Mark records seen</button>{/if}
 <section class="card">{#each view.milestones as m}<button class="ms" disabled={!m.route} onclick={() => m.route && navigate(m.route)}><span><b>{m.title}</b><small>{m.value} · {dateLabel(m.date)}{m.corrected ? ' · corrected after a Garmin edit' : ''}</small></span>{#if m.new}<em class="new">New</em>{/if}</button>{/each}
  {#if view.historyStart}<p class="small">In available history, since {dateLabel(view.historyStart)}.</p>{/if}</section>
{:else}<p class="small foot">Bests and other milestones appear here as Garmin records them.</p>{/if}
<style>
 .periods{display:flex;gap:6px;margin:6px 0 0}.periods button{flex:1;min-height:40px;border-radius:12px;border:1px solid var(--line);background:none;color:var(--text2);font:600 13px Inter}.periods button[aria-pressed=true]{background:var(--ink);color:var(--bg)}
 .rep-h{display:grid;grid-template-columns:44px 1fr 44px;align-items:center;text-align:center}.rep-h h2{font:600 18px Outfit}.nav{width:44px;height:44px;border-radius:50%;border:1px solid var(--line);background:none;color:var(--text);display:grid;place-items:center}.nav:disabled{color:var(--text3);border-color:transparent}
 .totals{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:12px;margin-top:12px}.totals span{font:500 12px Inter;color:var(--text2)}.totals b{display:block;font-size:24px;font-weight:600}.totals small{font:500 12px Inter;color:var(--text2);margin-left:3px}.totals em{display:block;font-style:normal;font-size:11.5px;color:var(--text2)}
 .axis{display:flex;justify-content:space-between;color:var(--text2);font-size:12px;margin-top:4px}
 .bars{display:flex;align-items:flex-end;gap:2px;height:90px;margin-top:14px;border-bottom:1px solid var(--line)}.bar{flex:1;height:100%;display:flex;flex-direction:column-reverse;min-width:0}.bar i{display:block;width:100%;border-radius:2px 2px 0 0}.bar.unknown{background:repeating-linear-gradient(45deg,transparent 0 3px,rgba(255,255,255,.07) 3px 5px)}
 .legend{display:flex;flex-wrap:wrap;gap:10px;margin:8px 0 0;font-size:12px;color:var(--text2)}.legend span{display:flex;align-items:center;gap:5px}.legend i{width:9px;height:9px;border-radius:50%;display:inline-block}
 .cal{display:grid;grid-template-columns:repeat(7,minmax(0,1fr));gap:4px}.day{display:grid;place-items:center;height:34px;border-radius:10px;font:600 12px Outfit;color:var(--text2);background:rgba(255,255,255,.03)}
 .day.active{background:rgba(61,220,151,.22);color:var(--text)}.day.uncovered{outline:1px dashed var(--text3)}.day.rest{background:rgba(169,200,255,.14)}.day.ill,.day.paused{background:repeating-linear-gradient(45deg,rgba(169,200,255,.22) 0 3px,transparent 3px 6px)}.day.unknown{outline:1px dotted var(--text3);background:none}.day.future,.day.before{background:none;color:var(--text3)}
 .k{width:12px;height:12px;border-radius:4px}.k.active{background:rgba(61,220,151,.4)}.k.rest{background:rgba(169,200,255,.3)}.k.paused{background:repeating-linear-gradient(45deg,rgba(169,200,255,.5) 0 2px,transparent 2px 4px)}.k.unknown{outline:1px dotted var(--text3)}
 .gw{display:flex;gap:4px;overflow-x:auto;padding-bottom:4px}.gw button{display:grid;justify-items:center;gap:3px;background:none;border:0;color:var(--text2);min-width:40px;min-height:44px}.gw small{font-size:10px}
 .link-row{display:flex;justify-content:space-between;align-items:center;width:100%;text-align:left;color:var(--text)}.link-row b{display:block;font:600 16px Outfit}.link-row small{display:block;font-size:12px;color:var(--text2);margin-top:3px}
 .sec-h{margin:22px 4px 0}.sec-h h2{font:600 19px Outfit}
 .tiles2{display:grid;grid-template-columns:repeat(auto-fill,minmax(150px,1fr));gap:10px}.tile2{display:flex;flex-direction:column;align-items:flex-start;gap:6px;text-align:left;color:var(--text);margin-top:10px}.tile2 span{font:500 12px Inter;color:var(--text2)}.tile2 b{font:600 26px Outfit}.tile2 small{font:500 12px Inter;color:var(--text2);margin-left:3px}.tile2 em{font-style:normal;font-size:13px;color:var(--text2)}
 .foot{margin:12px 4px}
 .ms{display:flex;justify-content:space-between;align-items:center;gap:8px;width:100%;padding:10px 0;border:0;border-bottom:1px solid var(--line);background:none;color:var(--text);text-align:left}.ms:disabled{cursor:default}.ms b{display:block;font:600 14px Inter}.ms small{display:block;font-size:12px;color:var(--text2);margin-top:2px}.new{font-style:normal;font:700 11px Inter;color:var(--bg);background:var(--ink);border-radius:999px;padding:3px 8px}
</style>
