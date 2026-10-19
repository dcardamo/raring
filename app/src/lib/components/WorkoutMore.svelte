<script lang="ts">
 import AxisChart from './AxisChart.svelte';
 import HelpButton from './HelpButton.svelte';
 import type {WorkoutView} from '../generated/WorkoutView';
 import type {StreamKind} from '../generated/StreamKind';
 import {elapsed, num} from '../format';
 // What Garmin stored for this activity beyond the summary: streams on one
 // time axis with heart rate, the training effect in plain words, and only
 // the fields this sport and watch recorded. Values come from the core.
 let {workout, navigate}: {workout: WorkoutView, navigate: (route: string) => void} = $props();
 const NAME: Record<StreamKind, string> = {speed: 'Speed', elevation: 'Elevation', cadence: 'Cadence', power: 'Power', respiration: 'Breathing rate', stamina: 'Stamina', performanceCondition: 'Performance condition', temperature: 'Watch temperature'};
 const COLOR: Record<StreamKind, string> = {speed: '#36C9C6', elevation: '#D9C089', cadence: '#A48BFF', power: '#FFB547', respiration: '#A9C8FF', stamina: '#3DDC97', performanceCondition: '#E4E8F0', temperature: '#FF8A5C'};
 let along = $derived(workout.along);
 let ride = $derived(workout.summary.icon === 'bike');
 // At most ~40 points per chart keeps labels readable; the core's samples are kept in order.
 let step = $derived(along ? Math.max(1, Math.ceil(along.tS.length / 40)) : 1);
 let idx = $derived(along ? along.tS.map((_, i) => i).filter(i => i % step === 0) : []);
 const minute = (t: number) => `${Math.round(t / 60)}′`;
 // Heart rate on the same axis: the nearest HR sample at or before each time.
 let hrAt = $derived(along ? idx.map(i => { const t = along!.tS[i]; let v: number | null = null; for (const s of workout.heartRate) { if (s.tS > t) break; v = s.bpm; } return v; }) : []);
 const pace = (kmh: number) => { const s = 3600 / kmh; return `${Math.floor(s / 60)}:${String(Math.round(s % 60)).padStart(2, '0')}`; };
 const fmtMore = (key: string, v: number, unit: string) => key === 'movingDuration' ? elapsed(v) : key === 'differenceBodyBattery' ? `${v > 0 ? '+' : ''}${num(v)}` : key === 'intensityFactor' ? num(v, 2) : `${num(v, unit === 'cm' || unit === 'brpm' || unit === '°C' ? 1 : 0)}${unit ? ' ' + unit : ''}`;
 let im = $derived.by(() => { const m = workout.more.find(x => x.key === 'moderateIntensityMinutes')?.value, v = workout.more.find(x => x.key === 'vigorousIntensityMinutes')?.value; return m != null && v != null ? {m, v} : null; });
 let rows = $derived(workout.more.filter(x => !['moderateIntensityMinutes', 'vigorousIntensityMinutes', 'differenceBodyBattery', 'beginPotentialStamina', 'endPotentialStamina'].includes(x.key)));
 let bb = $derived(workout.more.find(x => x.key === 'differenceBodyBattery')?.value ?? null);
 let stamina = $derived.by(() => { const a = workout.more.find(x => x.key === 'beginPotentialStamina')?.value, b = workout.more.find(x => x.key === 'endPotentialStamina')?.value; return a != null && b != null ? [a, b] : null; });
</script>
{#if along && workout.heartRate.length}
 <section class="card wm" aria-labelledby="along-h"><div class="wm-h"><h3 id="along-h">Along the way</h3><span class="small">same time axis</span></div>
  <AxisChart points={idx.map((i, n) => ({x: minute(along.tS[i]), v: hrAt[n]}))} kind="line" unit="bpm" color="#FF6B8A" height={100} label={`Heart rate over ${elapsed(workout.summary.durationS)}`}/>
  {#each along.series as s (s.kind)}
   {#if s.kind === 'speed' && !ride}
    <AxisChart points={idx.map(i => ({x: minute(along.tS[i]), v: s.values[i] != null && s.values[i]! > 1 ? 60 / s.values[i]! : null}))} kind="line" unit="min/km · lower is faster" color={COLOR.speed} height={100} fmt={v => pace(60 / v)} label={`Pace over ${elapsed(workout.summary.durationS)}, minutes per km`}/>
   {:else}
    <AxisChart points={idx.map(i => ({x: minute(along.tS[i]), v: s.values[i]}))} kind="line" unit={s.unit} color={COLOR[s.kind]} height={90} fmt={v => num(v, 0)} label={`${NAME[s.kind]} over ${elapsed(workout.summary.durationS)}, ${s.unit}`}/>
   {/if}
  {/each}
  <p class="small">Only what your watch recorded for this activity. Gaps stay gaps.{along.series.some(x => x.kind === 'temperature') ? ' Temperature is the watch’s own sensor and reads warm on the wrist; it isn’t the weather.' : ''}</p>
 </section>
{/if}
{#if workout.effects.length || im || bb != null || stamina || workout.vo2maxNew != null}
 <section class="card wm" aria-labelledby="did-h"><div class="wm-h"><h3 id="did-h">What it did</h3><HelpButton topic="training-effect"/></div>
  {#each workout.effects as e (e.kind)}
   <div class="te"><span><small>{e.kind === 'aerobic' ? 'Aerobic' : 'Anaerobic'} · Garmin</small><b class="n">{num(e.value, 1)}</b><em>{e.band}</em></span><p>{e.line}{e.garminLabel ? ` · Garmin: ${e.garminLabel}` : ''}</p></div>
  {/each}
  <dl class="kv">
   {#if workout.summary.benefit}<dt>Primary benefit</dt><dd>{workout.summary.benefit}</dd>{/if}
   {#if im}<dt>Intensity minutes</dt><dd>{num(im.m + 2 * im.v)}<small>{num(im.m)} moderate + 2 × {num(im.v)} vigorous · Garmin</small></dd>{/if}
   {#if bb != null}<dt>Body Battery</dt><dd>{bb > 0 ? '+' : ''}{num(bb)}</dd>{/if}
   {#if stamina}<dt>Stamina</dt><dd>{num(stamina[0])}% → {num(stamina[1])}%</dd>{/if}
  </dl>
  {#if workout.vo2maxNew != null}<button class="link" onclick={() => navigate('metric/vo2max')}>Garmin recorded a new VO₂ max estimate that day: {num(workout.vo2maxNew)}</button>{/if}
 </section>
{/if}
{#if rows.length}
 <section class="card wm" aria-labelledby="rec-h"><h3 id="rec-h">Recorded by your watch</h3><dl class="kv">{#each rows as r (r.key)}<dt>{r.label}</dt><dd>{fmtMore(r.key, r.value, r.unit)}</dd>{/each}</dl>{#if rows.some(r => r.unit === '°C')}<p class="small">The watch’s own sensor, warmer than the air when worn.</p>{/if}</section>
{/if}
<style>
 .wm{margin-top:12px}.wm-h{display:flex;justify-content:space-between;align-items:center;margin-bottom:6px}.wm-h h3{margin:0}
 .te{display:grid;grid-template-columns:auto 1fr;gap:4px 12px;align-items:center;padding:8px 0;border-bottom:1px solid var(--line)}.te span{display:grid}
 .te small{font-size:11px;color:var(--text2)}.te b{font-size:24px;font-weight:600}.te em{font-style:normal;font-size:12px;color:#3DDC97}.te p{margin:0;font-size:13.5px}
 .kv{display:grid;grid-template-columns:auto 1fr;gap:8px 14px;margin:10px 0 0;font-size:14px}.kv dt{color:var(--text2)}.kv dd{margin:0;text-align:right;font-weight:600}
 .kv small{display:block;font:500 11px Inter;color:var(--text2)}
 .link{background:none;border:0;color:var(--ink);font:600 13px Inter;min-height:44px;padding:0;text-align:left}
</style>
