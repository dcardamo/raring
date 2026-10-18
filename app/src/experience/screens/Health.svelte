<script lang="ts">
 import Icon from '../../lib/components/Icon.svelte';
 import Gauge from '../../lib/components/Gauge.svelte';
 import RangeBar from '../../lib/components/RangeBar.svelte';
 import VitalsMonitor from '../../lib/components/VitalsMonitor.svelte';
 import {toneColor} from '../../lib/charts';
 import {fitnessAgeColor, driverScale, driverRange} from '../../lib/fitness';
 import RootHeader from '../components/RootHeader.svelte';
 import TrendTile from '../components/TrendTile.svelte';
 import DayRows from '../components/DayRows.svelte';
 import XIcon from '../components/XIcon.svelte';
 import {store} from '../lib/store.svelte';
 import {healthView, todayView} from '../lib/views';
 import {TREND_TILES, trendTile, shift} from '../lib/progress';
 import {addDays, dayLabel, dayMonth, dur} from '../lib/dates';
 import {stagesFor, fitnessAgeFor, CHRONOLOGICAL_AGE} from '../lib/world';
 // Health leads with the long view, then sleep and recovery, then today's readings.
 let ctx = $derived(store.ctx);
 const go = (r: string) => store.navigate(r);
 let hv = $derived(healthView(ctx));
 let n = $derived(ctx.phase === 'waiting' || ctx.stale ? null : ctx.night);
 let r = $derived(ctx.phase === 'waiting' || ctx.stale ? null : ctx.readiness);
 let from = $derived(addDays(ctx.calendarDate, -89));
 let prevFrom = $derived(addDays(from, -90)), prevEnd = $derived(addDays(from, -1));
 let tiles = $derived(['restingHr', 'hrv', 'vo2max', 'sleepTime'].map(id => TREND_TILES.find(t => t.id === id)!).filter(t => !t.needs || t.needs(ctx)).map(t => ({t, v: trendTile(ctx, t, from, ctx.calendarDate, prevFrom, prevEnd)})));
 let faDate = $derived.by(() => { for (let d = ctx.calendarDate; d >= ctx.world.historyStart; d = addDays(d, -1)) if (fitnessAgeFor(d) != null) return d; return null; });
 const palette: Record<string, string> = {deep: '#2F6FE0', light: '#5C9BFF', rem: '#A9C8FF', awake: '#E4E8F0'};
</script>
<RootHeader title="Health"/>

<section class="lt" aria-labelledby="lt-h">
 <div class="lt-top">
  <div><small id="lt-h">Long-term health · Fitness Age</small><b class="n age" style:color={fitnessAgeColor(hv.fitnessAge.fitnessAge ?? 0, CHRONOLOGICAL_AGE)}>{hv.fitnessAge.fitnessAge ?? '—'}</b><span class="sub">Age {CHRONOLOGICAL_AGE} · Garmin estimate{faDate ? ` · ${dayMonth(faDate)}` : ''}</span></div>
  <button class="link" onclick={() => go('metric/fitnessAge')}>History</button>
 </div>
 <div class="drivers">
  {#each hv.drivers as d}<button onclick={() => go(d.key === 'restingHr' ? 'metric/restingHr' : d.key === 'vo2max' ? 'metric/vo2max' : 'metric/fitnessAge')}><span>{d.name}<b class="n">{d.value ?? '—'}<small>{d.unit}</small></b></span><RangeBar value={d.value} scale={driverScale(d.key)} range={driverRange(d.key, d.target)} color={toneColor(d.tone)}/></button>{/each}
 </div>
</section>

<div class="sec-h"><h2>Over 90 days</h2><button class="link" onclick={() => go('trends')}>All trends</button></div>
<div class="tgrid">{#each tiles as {t, v}}<TrendTile name={t.name} value={v.value} unit={t.unit} change={v.change} points={v.points} sparse={t.sparse} noNewEstimate={!!t.sparse && v.count === 0} note={t.sparse && v.latestDate ? dayMonth(v.latestDate) : undefined} color={t.id === 'sleepTime' ? '#5C9BFF' : t.id === 'vo2max' ? '#E4E8F0' : '#FF8A8A'} onclick={() => go(`metric/${t.id}`)}/>{/each}</div>

<div class="sec-h"><h2>Sleep and recovery</h2></div>
<div class="sr">
 <button class="sr-sleep" onclick={() => go('sleep')}><small><Icon name="moon" size={14}/>Last night</small><b class="n">{n ? dur(n.asleep) : '—'}</b>{#if n}<span class="stage">{#each stagesFor(ctx.displayedDay, n.asleep) as s}<i style:flex={s.minutes} style:background={palette[s.stage]}></i>{/each}</span><em>Score {n.score} · need {dur(n.need)}</em>{:else}<em>{ctx.phase === 'waiting' ? 'Not synced yet' : 'No recorded night'}</em>{/if}</button>
 <button class="sr-ready" aria-label={`Readiness ${r?.value ?? 'unavailable'}`} onclick={() => go('readiness')}><Gauge value={r?.value ?? null} size={74} stroke={7} color={toneColor(r ? r.value >= 75 ? 'good' : r.value >= 50 ? 'fair' : 'poor' : 'neutral')}/><span><b>Readiness</b><small>{r?.level ?? (ctx.world.readinessSupported ? 'Waiting' : 'Not on this watch')}</small></span></button>
</div>
<button class="rowlink tonight" onclick={() => go('sleep/tonight')}><Icon name="moon" size={18}/><span><b>Tonight · {ctx.tonight.window ?? 'asleep by 23:20'}</b><small>Wind down {ctx.tonight.windDown} · need {dur(ctx.tonight.need)} · {ctx.tonight.source}</small></span><Icon name="chevron" size={16}/></button>

<div class="sec-h"><h2>Today’s readings</h2><button class="link" onclick={() => go('health/vitals')}>All readings</button></div>
<VitalsMonitor vitals={todayView(ctx).vitals} compact open={(k: string) => go('metric/' + k)}/>
<DayRows/>

<section class="card private">
 <button class="rowlink" onclick={() => go('health/weight')}><Icon name="scale" size={18}/><span><b>Weight</b><small>Private to Health · no targets</small></span><Icon name="chevron" size={16}/></button>
 <button class="rowlink" onclick={() => go('health/journal')}><XIcon name="lock" size={18}/><span><b>Garmin journal</b><small>{ctx.state.journal.consent ? 'Allowed · opens on request' : 'Off · reads only with your consent'}</small></span><Icon name="chevron" size={16}/></button>
</section>

<style>
 .lt{margin-top:12px;padding:16px;border-radius:24px;background:linear-gradient(160deg,rgba(61,220,151,.1),var(--card) 60%);border:1px solid var(--line)}
 .lt-top{display:flex;justify-content:space-between;align-items:flex-start}.lt-top small{font:600 11px Inter;letter-spacing:.06em;color:var(--text3)}.age{display:block;font-size:64px;font-weight:600;line-height:1}.sub{font-size:12px;color:var(--text2)}
 .drivers{display:grid;grid-template-columns:1fr 1fr;gap:12px 14px;margin-top:16px}.drivers button{background:none;border:0;padding:0;text-align:left;color:var(--text);display:grid;gap:7px}.drivers span{display:flex;justify-content:space-between;align-items:baseline;font-size:12px;color:var(--text2)}.drivers b{font-size:16px;color:var(--text);font-weight:600}.drivers small{font:500 10px Inter;color:var(--text3);margin-left:2px}
 .tgrid{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:10px;margin-top:10px}
 .sr{display:grid;grid-template-columns:1.2fr 1fr;gap:10px;margin-top:10px}.sr button{border:1px solid var(--line);border-radius:20px;background:var(--card);color:var(--text);text-align:left;padding:14px;min-width:0}.sr small{display:flex;gap:6px;align-items:center;font:600 11px Inter;color:#A9C8FF}.sr-sleep b{display:block;font-size:28px;font-weight:600;margin:4px 0 8px}.stage{display:flex;height:8px;border-radius:4px;overflow:hidden}.stage i{display:block}.sr em{display:block;font-style:normal;font-size:11px;color:var(--text2);margin-top:6px}.sr-ready{display:flex;flex-wrap:wrap;align-items:center;gap:8px}.sr-ready span{min-width:0}.sr-ready b{display:block;font:600 13px Inter}.sr-ready span small{color:var(--text2);font-weight:500;margin-top:3px}
 .tonight{margin-top:10px;border:1px solid var(--line);border-radius:18px;background:rgba(92,124,255,.08);padding:12px 14px}.private{margin-top:14px}
 @container(max-width:300px){.drivers,.sr,.tgrid{grid-template-columns:1fr}}
</style>
