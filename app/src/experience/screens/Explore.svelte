<script lang="ts">
 import Icon from '../../lib/components/Icon.svelte';
 import Gauge from '../../lib/components/Gauge.svelte';
 import Hypnogram from '../../lib/components/Hypnogram.svelte';
 import BodyMap from '../../lib/components/BodyMap.svelte';
 import Sparkline from '../../lib/components/Sparkline.svelte';
 import {toneColor} from '../../lib/charts';
 import RootHeader from '../components/RootHeader.svelte';
 import RouteHero from '../components/RouteHero.svelte';
 import XIcon from '../components/XIcon.svelte';
 import {store} from '../lib/store.svelte';
 import {METRICS} from '../lib/metrics';
 import {addDays, dur, range} from '../lib/dates';
 import {EXERCISES, routeFor, hrTrace, stagesFor, restingHrFor, hrvFor, stressAvgFor, bodyBatteryHigh, vo2maxFor, fitnessAgeFor, sleepHrFor, respirationFor, spo2For, skinTempFor, weightFor} from '../lib/world';
 import {acuteLoad, musclesBetween, todayView} from '../lib/views';
 import {SPORTS} from '../lib/sports';
 let ctx = $derived(store.ctx);
 const go = (r: string) => store.navigate(r);
 let isRoot = $derived(store.roots.includes('explore'));
 let q = $derived((store.ui.q as string) ?? '');
 let n = $derived(ctx.phase === 'waiting' ? null : ctx.night);
 let gps = $derived(ctx.world.activities.findLast(a => routeFor(a).length));
 let last = $derived(ctx.world.activities.at(-1));
 let loads = $derived(range(addDays(ctx.calendarDate, -6), ctx.calendarDate).map(d => ctx.world.activities.filter(a => a.date === d).reduce((t, a) => t + a.load, 0)));
 let vitals = $derived(todayView(ctx).vitals.readings);
 const series: Record<string, (d: string) => number | null> = {
  readiness: d => ctx.readinessOf(d)?.value ?? null, sleepScore: d => ctx.nightOf(d)?.score ?? null, sleepTime: d => ctx.nightOf(d)?.asleep ?? null,
  sleepHr: d => ctx.nightOf(d) ? sleepHrFor(d) : null, bodyBattery: bodyBatteryHigh, stress: stressAvgFor, heartRate: restingHrFor, restingHr: restingHrFor,
  hrv: d => ctx.nightOf(d) ? hrvFor(d) : null, respiration: d => ctx.nightOf(d) ? respirationFor(d) : null, spo2: d => ctx.nightOf(d) ? spo2For(d) : null,
  skinTemp: d => ctx.nightOf(d) ? skinTempFor(d) : null, vo2max: vo2maxFor, fitnessAge: fitnessAgeFor, acuteLoad: d => acuteLoad(ctx, d), weight: weightFor,
  endurance: d => 5800 + Math.round(Math.sin(d.length) * 60), hill: () => 61, runningTolerance: () => 31,
 };
 const spark = (id: string) => range(addDays(ctx.calendarDate, -27), ctx.calendarDate).map(d => d < ctx.world.historyStart ? null : series[id]?.(d) ?? null);
 const latest = (id: string) => { for (let d = ctx.calendarDate; d >= addDays(ctx.calendarDate, -90); d = addDays(d, -1)) { const v = series[id]?.(d); if (v != null) return v; } return null; };
 const fmt = (id: string, v: number | null) => v == null ? '—' : id === 'sleepTime' ? dur(v) : `${id === 'skinTemp' && v > 0 ? '+' : ''}${Math.round(v * (['respiration', 'skinTemp', 'weight'].includes(id) ? 10 : 1)) / (['respiration', 'skinTemp', 'weight'].includes(id) ? 10 : 1)}`;
 const INDEX: [string, string, string, string | null][] = ([
  ...Object.entries(METRICS).map(([id, m]) => [m.name, `metric/${id}`, m.area, id] as [string, string, string, string]),
  ['Race predictions', 'training/performance', 'Training', null], ['Thresholds (lactate, FTP)', 'training/performance', 'Training', null], ['Load focus', 'training', 'Training', null],
  ['Gear (shoes, bikes)', 'gear', 'Training', null], ['Naps', 'sleep/naps', 'Sleep', null], ['Sleep stages', 'sleep/stages', 'Sleep', null], ['Sleep alignment', 'sleep/patterns', 'Sleep', null],
  ['Sleep timing', 'sleep/timing', 'Sleep', null], ['Garmin journal', 'health/journal', 'Health · private', null], ['Intensity minutes', 'plan', 'Plan', null],
  ['Weekly reviews', 'reviews', 'Progress', null], ['Muscle coverage', 'strength/muscles', 'Strength', null], ['Export your data', 'settings/export', 'Settings', null], ['Help', 'help', 'Help', null],
  ...EXERCISES.map(e => [e.name, `strength/${e.id}`, 'Strength exercise', null] as [string, string, string, null]),
 ] as [string, string, string, string | null][]).sort((a, b) => a[0].localeCompare(b[0]));
 const SYN: Record<string, string> = {'pulse ox': 'spo2', spo2: 'pulse ox', vo2: 'vo₂', shoes: 'gear', bike: 'gear', hr: 'heart', 'heart rate variability': 'hrv', temperature: 'skin', lifestyle: 'journal', backup: 'export', weight: 'weight', energy: 'body battery'};
 let results = $derived.by(() => { const t = q.trim().toLowerCase(); if (!t) return []; const terms = [t, SYN[t]].filter(Boolean); return INDEX.filter(([nm, , s]) => terms.some(x => `${nm} ${s}`.toLowerCase().includes(x))); });
</script>

{#if isRoot}<RootHeader title="Explore"/>{:else}<button class="back" onclick={() => store.back()}><Icon name="back" size={20}/> <span>{store.parentLabel}</span></button><header class="top"><div><div class="eyebrow">All of your Garmin data</div><h1 tabindex="-1">Explore</h1></div></header>{/if}

<label class="search"><XIcon name="search" size={18}/><span class="sr">Search metrics and exercises</span><input type="search" placeholder="Search metrics and exercises" value={q} oninput={e => store.setUi({q: e.currentTarget.value})} autocomplete="off"/></label>
{#if q.trim()}
 <section class="card"><h3>{results.length} result{results.length === 1 ? '' : 's'}</h3>
  {#each results as [nm, route, area, id]}<button class="rowlink" onclick={() => go(route)}><span class="az-i">{#if id}<Sparkline values={spark(id)} w={56} h={22} fill={false}/>{:else}<Icon name="chevron" size={14}/>{/if}</span><span><b>{nm}</b><small>{area}</small></span>{#if id}<span class="val">{fmt(id, latest(id))}</span>{/if}</button>{:else}<p class="small">Nothing by that name. Every metric lives in one of the areas below.</p>{/each}
 </section>
{/if}

<nav class="lib" aria-label="Areas">
 <button class="lt sleep" onclick={() => go('sleep')}><span class="lt-h"><b>Sleep</b><small>{n ? dur(n.asleep) : 'Syncing'}</small></span><span class="lt-v hyp">{#if n}<Hypnogram segments={stagesFor(ctx.displayedDay, n.asleep)} startLocal={null}/>{/if}</span><em>Last night, tonight, naps, patterns</em></button>
 <button class="lt" onclick={() => go('readiness')}><span class="lt-h"><b>Readiness</b><small>{ctx.readiness?.level ?? (ctx.world.readinessSupported ? 'Waiting' : 'Not on watch')}</small></span><span class="lt-v center"><Gauge size={74} stroke={7} value={ctx.readiness?.value ?? null} color={toneColor(ctx.readiness ? (ctx.readiness.value >= 75 ? 'good' : ctx.readiness.value >= 50 ? 'fair' : 'poor') : 'neutral')}/></span><em>Garmin’s factors and trend</em></button>
 <button class="lt" onclick={() => go('training')}><span class="lt-h"><b>Training</b><small>Load {acuteLoad(ctx, ctx.calendarDate)}</small></span><span class="lt-v bars">{#each loads as l, i}<i style:height={Math.max(4, l / Math.max(1, ...loads) * 48) + 'px'} class:today={i === 6}></i>{/each}</span><em>Load, VO₂ max, race times, gear</em></button>
 <button class="lt" onclick={() => go('strength')}><span class="lt-h"><b>Strength</b><small>{ctx.world.activities.filter(a => a.sport === 'strength' && a.date >= ctx.weekFrom).length} this week</small></span><span class="lt-v center"><BodyMap figure="female" muscles={musclesBetween(ctx, addDays(ctx.weekFrom, -21), ctx.calendarDate)} width={60} max={16} legend={false}/></span><em>Sessions, exercises, bests</em></button>
 <button class="lt act" onclick={() => go('activities')}><span class="lt-h"><b>Activities</b><small>{last ? `${SPORTS[last.sport].name} · ${last.date === ctx.calendarDate ? 'today' : last.date.slice(8) + '/' + last.date.slice(5, 7)}` : 'None yet'}</small></span><span class="lt-v map">{#if gps}<RouteHero route={routeFor(gps)} samples={hrTrace(gps)} floors={[118, 136, 150, 163]} color={SPORTS[gps.sport].color} w={160} h={70} compact label={`${gps.name} route`}/>{/if}</span><em>Every workout, routes, filters</em></button>
 <button class="lt" onclick={() => go('health')}><span class="lt-h"><b>Health</b><small>Resting HR {restingHrFor(ctx.displayedDay)}</small></span><span class="lt-v dots">{#each vitals as v}<i style:background={v.value != null ? toneColor(v.tone) : 'transparent'} class:off={v.status && v.status !== 'IN_RANGE'}></i>{/each}</span><em>Vitals, heart rate, stress, weight, journal</em></button>
</nav>

<section class="card more">
 <button class="rowlink" onclick={() => go('trends')}><Icon name="trend-up" size={18}/><span><b>Trends</b><small>Every chart, ranges and comparisons</small></span><Icon name="chevron" size={16}/></button>
 <button class="rowlink" onclick={() => go('gear')}><Icon name="run" size={18}/><span><b>Gear</b><small>Shoes and bikes, Garmin distances</small></span><Icon name="chevron" size={16}/></button>
 <button class="rowlink" onclick={() => go('reviews')}><XIcon name="calendar" size={18}/><span><b>Weekly reviews</b><small>Past weeks</small></span><Icon name="chevron" size={16}/></button>
 <button class="rowlink" onclick={() => go('help')}><Icon name="help" size={18}/><span><b>Help</b><small>Offline guides</small></span><Icon name="chevron" size={16}/></button>
 <button class="rowlink" onclick={() => go('settings')}><XIcon name="settings" size={18}/><span><b>Settings</b><small>Notifications, privacy, export, sign out</small></span><Icon name="chevron" size={16}/></button>
</section>

<div class="sec-h"><h2>Everything A–Z</h2></div>
<section class="card az">
 {#each INDEX as [nm, route, area, id]}<button class="rowlink" onclick={() => go(route)}><span class="az-i">{#if id}<Sparkline values={spark(id)} w={56} h={22} fill={false} color={id === 'fitnessAge' || id === 'vo2max' || id === 'weight' ? '#B4BCCB' : '#E4E8F0'} points={id === 'fitnessAge' || id === 'vo2max' || id === 'weight'}/>{/if}</span><span><b>{nm}</b><small>{area}</small></span>{#if id}<span class="val">{fmt(id, latest(id))}<small>{METRICS[id].unit && METRICS[id].unit !== 'h' ? ' ' + METRICS[id].unit : ''}</small></span>{:else}<Icon name="chevron" size={16}/>{/if}</button>{/each}
</section>

<style>
 .search{display:flex;align-items:center;gap:10px;margin-top:12px;padding:0 14px;height:48px;border-radius:16px;background:rgba(255,255,255,.06);border:1px solid var(--line);color:var(--text3)}
 .search input{flex:1;background:none;border:0;color:var(--text);font:500 15px Inter;outline:none;min-width:0;height:100%}
 .sr{position:absolute;width:1px;height:1px;overflow:hidden;clip:rect(0 0 0 0)}
 .lib{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:10px;margin-top:12px}
 .lt{display:flex;flex-direction:column;gap:6px;padding:12px;border-radius:20px;background:var(--card);border:1px solid var(--line);color:var(--text);text-align:left;min-height:156px;min-width:0}
 .lt-h{display:flex;justify-content:space-between;align-items:baseline;gap:6px}
 .lt-h b{font:600 17px Outfit}
 .lt-h small{font-size:11px;color:var(--text2);white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
 .lt-v{flex:1;display:flex;align-items:center;min-height:74px}
 .lt-v.center{justify-content:center}
 .lt-v.hyp :global(text){display:none}
 .lt-v.hyp :global(.plot){height:70px}
 .lt-v.bars{align-items:flex-end;gap:4px;padding:0 4px 8px}
 .lt-v.bars i{flex:1;border-radius:4px 4px 2px 2px;background:rgba(228,232,240,.3)}
 .lt-v.bars i.today{background:#E4E8F0}
 .lt-v.map{border-radius:12px;overflow:hidden;height:74px}
 .lt-v.dots{justify-content:center;gap:7px}
 .lt-v.dots i{width:14px;height:14px;border-radius:50%}
 .lt-v.dots i.off{outline:2px solid #FFB547;outline-offset:2px}
 .lt em{font-style:normal;font-size:11.5px;color:var(--text3);line-height:1.3}
 .more{margin-top:12px;padding-top:6px;padding-bottom:6px}
 .more .rowlink>:global(svg:first-child){color:var(--text2)}
 .az{padding-top:4px;padding-bottom:4px}
 .az-i{width:56px;display:grid;place-items:center;color:var(--text3)}
 .val small{font:500 11px Inter;color:var(--text3)}
</style>
