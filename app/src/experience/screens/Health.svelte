<script lang="ts">
 import Icon from '../../lib/components/Icon.svelte';
 import Gauge from '../../lib/components/Gauge.svelte';
 import RangeBar from '../../lib/components/RangeBar.svelte';
 import VitalsMonitor from '../../lib/components/VitalsMonitor.svelte';
 import {toneColor} from '../../lib/charts';
 import {fitnessAgeColor, driverScale, driverRange} from '../../lib/fitness';
 import RootHeader from '../components/RootHeader.svelte';
 import MetricCard from '../components/MetricCard.svelte';
 import HelpButton from '../components/HelpButton.svelte';
 import XIcon from '../components/XIcon.svelte';
 import {store} from '../lib/store.svelte';
 import {healthView, todayView} from '../lib/views';
 import {dayMonth, dur} from '../lib/dates';
 import {stagesFor, fitnessAgeFor, restingHrFor, hrvFor, stressAvgFor, bodyBatteryHigh, nightFor, weightFor, CHRONOLOGICAL_AGE} from '../lib/world';
 import {addDays} from '../lib/dates';
 // Health: the long view first, then this morning's readings, then each daily
 // metric with its own range picker. Your day lives on Today only.
 let ctx = $derived(store.ctx);
 const go = (r: string) => store.navigate(r);
 let hv = $derived(healthView(ctx));
 let tv = $derived(todayView(ctx));
 let n = $derived(ctx.phase === 'waiting' || ctx.stale ? null : ctx.night);
 let r = $derived(ctx.phase === 'waiting' || ctx.stale ? null : ctx.readiness);
 let faDate = $derived.by(() => { for (let d = ctx.calendarDate; d >= ctx.world.historyStart; d = addDays(d, -1)) if (fitnessAgeFor(d) != null) return d; return null; });
 const usual = (k: string): [number, number] | null => { const v = tv.vitals?.readings.find(x => x.kind === k); return v?.normal ? [v.normal[0], v.normal[1]] : null; };
 const withNight = (f: (d: string) => number) => (d: string) => nightFor(d) ? f(d) : null;
 const palette: Record<string, string> = {deep: '#2F6FE0', light: '#5C9BFF', rem: '#A9C8FF', awake: '#E4E8F0'};
</script>
<RootHeader title="Health" help="fitness-age"/>

<section class="lt" aria-labelledby="lt-h">
 <div class="lt-top">
  <div><small id="lt-h">Long-term health · Fitness Age</small><b class="n age" style:color={fitnessAgeColor(hv.fitnessAge.fitnessAge ?? 0, CHRONOLOGICAL_AGE)}>{hv.fitnessAge.fitnessAge ?? '—'}</b><span class="sub">Age {CHRONOLOGICAL_AGE} · Garmin estimate{faDate ? ` · ${dayMonth(faDate)}` : ''}</span></div>
  <button class="link" onclick={() => go('metric/fitnessAge')}>History</button>
 </div>
 <div class="drivers">
  {#each hv.drivers as d}<button onclick={() => go(d.key === 'restingHr' ? 'metric/restingHr' : d.key === 'vo2max' ? 'metric/vo2max' : 'metric/fitnessAge')}><span>{d.name}<b class="n">{d.value ?? '—'}<small>{d.unit}</small></b></span><RangeBar value={d.value} scale={driverScale(d.key)} range={driverRange(d.key, d.target)} color={toneColor(d.tone)}/></button>{/each}
 </div>
</section>

<div class="sec-h"><h2>This morning’s readings</h2><span class="hh"><HelpButton topic="vitals"/><button class="link" onclick={() => go('health/vitals')}>All</button></span></div>
<VitalsMonitor vitals={tv.vitals} compact open={(k: string) => go('metric/' + k)}/>

<div class="sr">
 <button class="sr-sleep" onclick={() => go('sleep')}><small><Icon name="moon" size={14}/>Last night</small><b class="n">{n ? dur(n.asleep) : '—'}</b>{#if n}<span class="stage">{#each stagesFor(ctx.displayedDay, n.asleep) as s}<i style:flex={s.minutes} style:background={palette[s.stage]}></i>{/each}</span><em>Score {n.score} · need {dur(n.need)}</em>{:else}<em>{ctx.phase === 'waiting' ? 'Not synced yet' : 'No recorded night'}</em>{/if}</button>
 <button class="sr-ready" aria-label={`Readiness ${r?.value ?? 'unavailable'}`} onclick={() => go('readiness')}><Gauge value={r?.value ?? null} size={70} stroke={7} color={toneColor(r ? r.value >= 75 ? 'good' : r.value >= 50 ? 'fair' : 'poor' : 'neutral')}/><span><b>Readiness</b><small>{r?.level ?? (ctx.world.readinessSupported ? 'Waiting' : 'Not on this watch')}</small></span></button>
</div>

<div class="sec-h"><h2>Over time</h2><HelpButton topic="metric-range"/></div>
<MetricCard id="restingHr" name="Resting heart rate" unit="bpm" read={withNight(restingHrFor)} band={usual('restingHr')} color="#FF8A8A"/>
<MetricCard id="hrv" name="Overnight HRV" unit="ms" read={withNight(hrvFor)} band={usual('hrv')} color="#A48BFF"/>
<MetricCard id="stress" name="Average stress" unit="" read={stressAvgFor} color="#FFB547"/>
<MetricCard id="bodyBattery" name="Body Battery high" unit="" read={bodyBatteryHigh} color="#3DDC97"/>
<MetricCard id="weight" name="Weight" unit="kg" read={weightFor} color="#E4E8F0" sparse fmt={v => v.toFixed(1)} route="health/weight">
 <button class="na" disabled aria-describedby="w-na"><Icon name="scale" size={15}/>Add a weight</button><p class="small" id="w-na">Not available yet: adding a weight is a Garmin write.</p>
</MetricCard>

<section class="card private">
 <button class="rowlink" onclick={() => go('health/journal')}><XIcon name="lock" size={18}/><span><b>Garmin journal</b><small>{ctx.state.journal.consent ? 'Allowed · opens on request' : 'Off · reads only with your consent'}</small></span><Icon name="chevron" size={16}/></button>
</section>

<style>
 .lt{margin-top:12px;padding:16px;border-radius:24px;background:linear-gradient(160deg,rgba(61,220,151,.1),var(--card) 60%);border:1px solid var(--line)}
 .lt-top{display:flex;justify-content:space-between;align-items:flex-start}.lt-top small{font:600 11px Inter;letter-spacing:.06em;color:var(--text3)}.age{display:block;font-size:60px;font-weight:600;line-height:1}.sub{font-size:12px;color:var(--text2)}
 .drivers{display:grid;grid-template-columns:1fr 1fr;gap:12px 14px;margin-top:14px}.drivers button{background:none;border:0;padding:0;text-align:left;color:var(--text);display:grid;gap:7px}.drivers span{display:flex;justify-content:space-between;align-items:baseline;font-size:12px;color:var(--text2)}.drivers b{font-size:16px;color:var(--text);font-weight:600}.drivers small{font:500 10px Inter;color:var(--text3);margin-left:2px}
 .hh{display:flex;gap:8px;align-items:center}
 .sr{display:grid;grid-template-columns:1.2fr 1fr;gap:10px;margin-top:12px}.sr button{border:1px solid var(--line);border-radius:20px;background:var(--card);color:var(--text);text-align:left;padding:14px;min-width:0}.sr small{display:flex;gap:6px;align-items:center;font:600 11px Inter;color:#A9C8FF}.sr-sleep b{display:block;font-size:26px;font-weight:600;margin:4px 0 8px}.stage{display:flex;height:8px;border-radius:4px;overflow:hidden}.stage i{display:block}.sr em{display:block;font-style:normal;font-size:11px;color:var(--text2);margin-top:6px}.sr-ready{display:flex;flex-wrap:wrap;align-items:center;gap:8px}.sr-ready span{min-width:0}.sr-ready b{display:block;font:600 13px Inter}.sr-ready span small{color:var(--text2);font-weight:500;margin-top:3px}
 .private{margin-top:14px}
 .na{display:flex;gap:8px;align-items:center;justify-content:center;width:100%;margin-top:10px;min-height:40px;border-radius:12px;border:1px dashed var(--line);background:none;color:var(--text3);font:600 13px Inter}
 @container(max-width:300px){.drivers,.sr{grid-template-columns:1fr}}
</style>
