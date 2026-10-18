<script lang="ts">
 import Icon from '../../lib/components/Icon.svelte';
 import SportIcon from './SportIcon.svelte';
 import {stressColors, stressLevel} from '../../lib/charts';
 import {store} from '../lib/store.svelte';
 import {stressCard, bodyBatteryDay} from '../lib/views';
 import {stagesFor} from '../lib/world';
 import {SPORTS} from '../lib/sports';
 import {dur, dayLabel, hm, toMin} from '../lib/dates';
 // Sleep, Body Battery, stress and recorded events share one midnight-to-midnight
 // axis, so any moment lines up across rows. Data stops at the watch's last upload.
 let ctx = $derived(store.ctx);
 let day = $derived(ctx.clock.phase === 'late' ? ctx.day : ctx.calendarDate);
 let waiting = $derived(ctx.phase === 'waiting' || ctx.stale);
 let night = $derived(waiting ? null : ctx.nightOf(day));
 let now = $derived(day === ctx.calendarDate ? toMin(ctx.clock.time) : null);
 let uploadMin = $derived(day === ctx.upload.date ? toMin(ctx.upload.at) : day < ctx.upload.date ? 1440 : 0);
 const pct = (m: number) => Math.max(0, Math.min(100, m / 1440 * 100));
 const palette: Record<string, string> = {deep: '#2F6FE0', light: '#5C9BFF', rem: '#A9C8FF', awake: '#E4E8F0'};
 // Last night's stages placed at their clock times; the part before midnight is clipped.
 let stages = $derived.by(() => {
  if (!night) return [];
  const bed = toMin(night.bed) > 720 ? toMin(night.bed) - 1440 : toMin(night.bed), wake = toMin(night.wake);
  const st = stagesFor(day, night.asleep), total = st.reduce((t, s) => t + s.minutes, 0) || 1;
  return st.map(s => ({stage: s.stage, a: bed + s.startMin / total * (wake - bed), b: bed + (s.startMin + s.minutes) / total * (wake - bed)})).filter(s => s.b > 0);
 });
 let bb = $derived(waiting || uploadMin === 0 ? null : bodyBatteryDay(ctx, day));
 let bbPath = $derived(bb ? bb.points.filter(([m]) => m >= 0 && m <= uploadMin).map(([m, v], i) => `${i ? 'L' : 'M'}${(m / 1440 * 1000).toFixed(1)},${(48 - v / 100 * 44).toFixed(1)}`).join('') : '');
 let bbNow = $derived(bb ? bb.points.filter(([m]) => m <= uploadMin).at(-1)?.[1] ?? null : null);
 let stress = $derived(stressCard(ctx, day));
 // Fifteen-minute averages of Garmin's three-minute readings; missing slots stay empty.
 let bars = $derived.by(() => {
  const out: {m: number, v: number}[] = [];
  for (let i = 0; i < stress.timeline.length; i += 5) {
   const vals = stress.timeline.slice(i, i + 5).filter((v): v is number => v != null);
   if (vals.length) out.push({m: i * 3, v: vals.reduce((a, b) => a + b, 0) / vals.length});
  }
  return out;
 });
 let events = $derived([
  ...ctx.world.activities.filter(a => a.date === day).map(a => ({m: toMin(a.start), len: a.min, color: SPORTS[a.sport].color, sport: a.sport, label: `${a.name} ${a.start}, ${a.min} min`, go: `workout/${a.id}`, nap: false})),
  ...ctx.napList.filter(n => n.start[0] === day).map(n => ({m: toMin(n.start[1]), len: n.min, color: '#A9C8FF', sport: 'other' as const, label: `Nap ${n.start[1]}, ${n.min} min`, go: 'sleep/naps', nap: true})),
 ].sort((a, b) => a.m - b.m));
 const go = (r: string) => store.navigate(r);
</script>
<section class="day-rows" aria-label="Your day on one timeline">
 <div class="dr-h"><h2>Your day{day !== ctx.calendarDate ? ` · ${dayLabel(day)}` : ''}</h2><span>Updated {ctx.upload.at}</span></div>

 <button class="row" onclick={() => go('sleep')} aria-label={night ? `Sleep ${night.bed} to ${night.wake}, ${dur(night.asleep)}` : 'Sleep: not synced yet'}>
  <span class="lab"><Icon name="moon" size={15}/><b>Sleep</b></span>
  <span class="track sleep">{#each stages as s}<i style:left={pct(s.a) + '%'} style:width={pct(s.b) - pct(s.a) + '%'} style:background={palette[s.stage]}></i>{/each}{#if !night}<em class="none">{waiting ? 'Not synced yet' : 'No recorded night'}</em>{/if}{#if now != null}<u style:left={pct(now) + '%'}></u>{/if}</span>
  <span class="val n">{night ? dur(night.asleep) : '—'}</span>
 </button>

 <button class="row" onclick={() => go('metric/bodyBattery')} aria-label={`Body Battery ${bbNow ?? 'unavailable'} at ${ctx.upload.at}`}>
  <span class="lab"><Icon name="bolt" size={15}/><b>Body Battery</b></span>
  <span class="track chart">{#if bbPath}<svg viewBox="0 0 1000 50" preserveAspectRatio="none" aria-hidden="true"><path d={`${bbPath}L${(uploadMin / 1440 * 1000).toFixed(1)},50L0,50Z`} fill="rgba(61,220,151,.16)"/><path d={bbPath} fill="none" stroke="#3DDC97" stroke-width="2" vector-effect="non-scaling-stroke"/></svg>{:else}<em class="none">Awaiting upload</em>{/if}{#if now != null}<u style:left={pct(now) + '%'}></u>{/if}</span>
  <span class="val n">{bbNow ?? '—'}</span>
 </button>

 <button class="row" onclick={() => go('metric/stress')} aria-label={`Stress, day average ${stress.average ?? 'unavailable'}`}>
  <span class="lab"><Icon name="pulse" size={15}/><b>Stress</b></span>
  <span class="track chart">{#if bars.length}<svg viewBox="0 0 1440 50" preserveAspectRatio="none" aria-hidden="true">{#each bars as b}<rect x={b.m + 1} y={50 - Math.max(2, b.v / 100 * 48)} width="13" height={Math.max(2, b.v / 100 * 48)} fill={stressColors[stressLevel(b.v)]}/>{/each}</svg>{:else}<em class="none">Awaiting upload</em>{/if}{#if now != null}<u style:left={pct(now) + '%'}></u>{/if}</span>
  <span class="val n">{stress.average ?? '—'}<small>avg</small></span>
 </button>

 <div class="row ev" role="group" aria-label="Recorded activities and naps">
  <span class="lab"><Icon name="run" size={15}/><b>Recorded</b></span>
  <span class="track events">{#each events as e}<button class="evb" class:nap={e.nap} style:left={pct(e.m) + '%'} style:width={`max(26px, ${pct(e.m + e.len) - pct(e.m)}%)`} style:background={e.color} aria-label={e.label} onclick={() => go(e.go)}>{#if e.nap}<Icon name="moon" size={11}/>{:else}<SportIcon sport={e.sport} size={12}/>{/if}</button>{/each}{#if !events.length}<em class="none">Nothing recorded yet</em>{/if}{#if now != null}<u style:left={pct(now) + '%'}></u>{/if}</span>
  <span class="val n">{events.filter(e => !e.nap).length || ''}</span>
 </div>

 <div class="row axis" aria-hidden="true"><span></span><span class="track ticks">{#each [0, 6, 12, 18, 24] as h}<i style:left={h / 24 * 100 + '%'}>{String(h).padStart(2, '0')}</i>{/each}{#if now != null}<b style:left={pct(now) + '%'}>Now {hm(now)}</b>{/if}</span><span></span></div>
 <p class="key"><span><i style:background={stressColors.rest}></i>Rest</span><span><i style:background={stressColors.low}></i>Low</span><span><i style:background={stressColors.medium}></i>Medium</span><span><i style:background={stressColors.high}></i>High stress</span></p>
</section>
<style>
 .day-rows{margin-top:14px;border:1px solid var(--line);border-radius:22px;background:var(--card);padding:14px 14px 10px}
 .dr-h{display:flex;justify-content:space-between;align-items:baseline;margin-bottom:6px}.dr-h h2{font:600 11px Inter;letter-spacing:.13em;text-transform:uppercase;color:var(--text3)}.dr-h span{font-size:11px;color:var(--text3)}
 .row{display:grid;grid-template-columns:96px 1fr 46px;gap:8px;align-items:center;width:100%;padding:6px 0;background:none;border:0;color:var(--text);text-align:left;min-height:44px}
 .lab{display:flex;align-items:center;gap:6px;min-width:0}.lab :global(svg){color:var(--text3);flex:none}.lab b{font:600 12.5px/1.15 Inter;overflow-wrap:anywhere}
 .track{position:relative;display:block;height:34px;border-radius:6px;background:rgba(255,255,255,.035)}
 .track.sleep{height:16px}.track.sleep i{position:absolute;top:0;bottom:0}
 .track.chart svg{position:absolute;inset:0;width:100%;height:100%}
 .track.events{height:34px;background:none;border-bottom:1px solid var(--line);border-radius:0}
 .evb{position:absolute;top:1px;height:32px;transform:translateX(0);border:0;border-radius:6px;display:grid;place-items:center;color:#0B0F1A;padding:0;min-width:22px}
 .track u{position:absolute;top:-4px;bottom:-4px;width:2px;margin-left:-1px;background:var(--ink);border-radius:1px;text-decoration:none;opacity:.85;pointer-events:none}
 .none{position:absolute;inset:0;display:grid;place-items:center;font:500 10.5px Inter;color:var(--text3);font-style:normal}
 .val{text-align:right;font-size:17px;font-weight:600}.val small{display:block;font:500 9.5px Inter;color:var(--text3)}
 .axis{padding:0;min-height:22px}.ticks{height:20px;background:none}.ticks i{position:absolute;top:2px;transform:translateX(-50%);font:500 10px Inter;color:var(--text3);font-style:normal}.ticks i:first-child{transform:none}.ticks i:nth-child(5){transform:translateX(-100%)}.ticks b{position:absolute;top:-2px;transform:translateX(-50%);font:600 10px Inter;color:var(--text);background:#141a27;padding:1px 4px;border-radius:4px;white-space:nowrap}
 .key{display:flex;flex-wrap:wrap;gap:4px 12px;margin:6px 0 0;font-size:10.5px;color:var(--text3)}.key span{display:inline-flex;gap:5px;align-items:center}.key i{width:8px;height:8px;border-radius:2px}
 @container(max-width:300px){.row{grid-template-columns:72px 1fr 40px}.lab b{font-size:11px}}
</style>
