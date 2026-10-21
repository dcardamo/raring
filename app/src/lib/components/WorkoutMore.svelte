<script lang="ts">
 import AxisChart from './AxisChart.svelte';
 import HrChart from './HrChart.svelte';
 import HelpButton from './HelpButton.svelte';
 import type {WorkoutView} from '../generated/WorkoutView';
 import type {StreamKind} from '../generated/StreamKind';
 import {elapsed, num} from '../format';
 import {paceSeconds, paceUnit, speed, speedUnit, elevation, elevationUnit, swimUnit} from '../units';
 import {fieldValue, groupName, streamScale} from '../workoutFields';
 // What Garmin stored for this activity beyond the summary: streams on one
 // time axis with heart rate, the training effect in plain words, and only
 // the fields this sport and watch recorded. Values come from the core.
 let {workout, navigate}: {workout: WorkoutView, navigate: (route: string) => void} = $props();
 const NAME: Record<StreamKind, string> = {speed: 'Speed', elevation: 'Elevation', cadence: 'Cadence', strideLength: 'Stride length', groundContactTime: 'Ground contact time', verticalOscillation: 'Vertical oscillation', verticalRatio: 'Vertical ratio', strokeRate: 'Stroke rate', strokeDistance: 'Distance per stroke', power: 'Power', respiration: 'Breathing rate', stamina: 'Stamina', performanceCondition: 'Performance condition', temperature: 'Watch temperature'};
 // Running form and strokes have no Garmin rating: the neutral blue (Rules: Chart colours).
 const COLOR: Record<StreamKind, string> = {speed: 'var(--goal-intensity)', elevation: '#D9C089', cadence: '#A48BFF', strideLength: 'var(--amount)', groundContactTime: 'var(--amount)', verticalOscillation: 'var(--amount)', verticalRatio: 'var(--amount)', strokeRate: 'var(--amount)', strokeDistance: 'var(--amount)', power: 'var(--fair)', respiration: 'var(--goal-sleep)', stamina: 'var(--good)', performanceCondition: 'var(--ink)', temperature: 'var(--goal-high)'};
 let along = $derived(workout.along);
 let ride = $derived(workout.summary.icon === 'bike');
 let unit = $derived(workout.summary.distanceUnit);
 // At most ~40 points per chart keeps labels readable; the core's samples are kept in order.
 let step = $derived(along ? Math.max(1, Math.ceil(along.tS.length / 40)) : 1);
 let idx = $derived(along ? along.tS.map((_, i) => i).filter(i => i % step === 0) : []);
 // Round minute ticks, the same as the heart-rate chart above; only the last chart is labelled.
 let tickStep = $derived.by(() => { const m = Math.max(1, workout.summary.durationS, ...(along?.tS ?? [])) / 60; return [1, 2, 5, 10, 15, 20, 30, 60].find(s => m / s <= 6) ?? 120; });
 let xs = $derived(along ? idx.map((i, n) => { const k = Math.floor(along!.tS[i] / 60 / tickStep); return n === 0 || k !== Math.floor(along!.tS[idx[n - 1]] / 60 / tickStep) ? String(k * tickStep) : ''; }) : []);
 let lastKind = $derived(along?.series.at(-1)?.kind);
 // Pace is drawn faster-up: negative seconds per unit on a fixed scale.
 const paceOf = (kmh: number | null) => kmh != null && kmh > (swimUnit(unit) ? .5 : 1) ? -paceSeconds(3600 / kmh, unit) : null;
 const paceScale = (values: (number | null)[]) => { const p = values.filter((v): v is number => v != null).map(v => -v); return {yMin: -Math.ceil(Math.max(...p) / 30) * 30, yMax: -Math.floor(Math.min(...p) / 30) * 30}; };
 const did = (key: string) => workout.more.find(x => x.key === key)?.value ?? null;
 let im = $derived.by(() => { const m = did('moderateIntensityMinutes'), v = did('vigorousIntensityMinutes'); return m != null && v != null ? {m, v} : null; });
 let bb = $derived(did('differenceBodyBattery'));
 // Garmin's potential stamina at start and end, and its lowest available stamina: two series.
 let stamina = $derived.by(() => { const a = did('beginPotentialStamina'), b = did('endPotentialStamina'), low = did('minAvailableStamina'); return (a != null && b != null) || low != null ? {range: a != null && b != null ? [a, b] : null, low} : null; });
 let groups = $derived(workout.recorded ?? []);
</script>
{#if along && workout.heartRate.length}
 <section class="card wm" aria-labelledby="along-h"><div class="wm-h"><h3 id="along-h">Along the way</h3>{#if along.series.some(x => x.kind === 'temperature')}<HelpButton topic="watch-temperature"/>{/if}</div>
  <HrChart samples={workout.heartRate} floors={workout.zoneFloors} durationS={workout.summary.durationS} avg={workout.summary.avgHr ?? null} axis={false} name="Heart rate"/>
  {#each along.series as s (s.kind)}
   {@const label = (n: number) => s.kind === lastKind ? xs[n] : ''}
   {#if s.kind === 'speed' && !ride}
    {@const values = idx.map(i => paceOf(s.values[i]))}
    <AxisChart points={idx.map((_, n) => ({x: label(n), v: values[n]}))} kind="line" unit={`Pace · min${paceUnit(unit)}`} color={COLOR.speed} height={100} every={1} {...paceScale(values)} fmt={v => elapsed(-v)} label={`Pace over ${elapsed(workout.summary.durationS)}, minutes ${paceUnit(unit).replace('/', 'per ')}`}/>
   {:else if s.kind === 'speed' || s.kind === 'elevation'}
    {@const conv = (v: number) => s.kind === 'speed' ? speed(v, unit) : elevation(v, unit)}
    {@const u = s.kind === 'speed' ? speedUnit(unit) : elevationUnit(unit)}
    <AxisChart points={idx.map((i, n) => ({x: label(n), v: s.values[i] != null ? conv(s.values[i]!) : null}))} kind="line" unit={`${NAME[s.kind]} · ${u}`} color={COLOR[s.kind]} height={90} every={1} fmt={v => num(v, 0)} label={`${NAME[s.kind]} over ${elapsed(workout.summary.durationS)}, ${u}`}/>
   {:else}
    {@const scale = streamScale(s.kind, unit)}
    {@const u = scale?.unit ?? s.unit}
    <AxisChart points={idx.map((i, n) => ({x: label(n), v: s.values[i] != null && scale ? scale.conv(s.values[i]!) : s.values[i]}))} kind="line" unit={`${NAME[s.kind]} · ${u}`} color={COLOR[s.kind]} height={90} every={1} fmt={v => num(v, scale?.digits ?? 0)} label={`${NAME[s.kind]} over ${elapsed(workout.summary.durationS)}, ${u}`}/>
   {/if}
  {/each}
  <p class="small axis-note">Minutes from the start</p>
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
   {#if stamina}<dt>Stamina</dt><dd>{#if stamina.range}{num(stamina.range[0])}% → {num(stamina.range[1])}%<small>potential{stamina.low != null ? ` · lowest available ${num(stamina.low)}%` : ''}</small>{:else}{num(stamina.low)}%<small>lowest available</small>{/if}</dd>{/if}
  </dl>
  {#if workout.vo2maxNew != null}<button class="link" onclick={() => navigate('metric/vo2max')}>Garmin recorded a new VO₂ max estimate that day: {num(workout.vo2maxNew)}</button>{/if}
 </section>
{/if}
{#if groups.length}
 <section class="card wm" aria-labelledby="rec-h"><div class="wm-h"><h3 id="rec-h">Recorded by your watch</h3>{#if groups.some(g => g.group === 'temperature')}<HelpButton topic="watch-temperature"/>{/if}</div>
  {#each groups as g (g.group)}
   <div class="rg" role="group" aria-labelledby={`rg-${g.group}`}><div class="rg-h"><h4 id={`rg-${g.group}`}>{groupName(g.group, workout.summary.icon)}</h4>{#if g.group === 'form' && workout.summary.icon === 'run' && g.fields.some(f => ['groundContactTime', 'verticalOscillation', 'verticalRatio', 'avgStepSpeedLoss'].includes(f.key))}<HelpButton topic="running-form"/>{/if}</div><dl class="kv">{#each g.fields as r (r.key)}<dt>{r.label}</dt><dd>{fieldValue(r, unit)}</dd>{/each}</dl></div>
  {/each}
 </section>
{/if}
<style>
 .wm{margin-top:12px}.axis-note{margin:2px 0 0;text-align:right}.wm-h{display:flex;justify-content:space-between;align-items:center;margin-bottom:6px}.wm-h h3{margin:0}
 .te{display:grid;grid-template-columns:auto 1fr;gap:4px 12px;align-items:center;padding:8px 0;border-bottom:1px solid var(--line)}.te span{display:grid}
 .te small{font-size:11px;color:var(--text2)}.te b{font-size:24px;font-weight:600}.te em{font-style:normal;font-size:12px;color:var(--ink)}.te p{margin:0;font-size:13.5px}
 .kv{display:grid;grid-template-columns:auto 1fr;gap:8px 14px;margin:10px 0 0;font-size:14px}.kv dt{color:var(--text2)}.kv dd{margin:0;text-align:right;font-weight:600}
 .kv small{display:block;font:500 11px Inter;color:var(--text2)}
 .rg+.rg{margin-top:14px;padding-top:12px;border-top:1px solid var(--line)}.rg-h{display:flex;justify-content:space-between;align-items:center;gap:8px}.rg h4{margin:0;font:600 12px/1.4 Inter;color:var(--text2)}.rg .kv{margin-top:6px}
 .link{background:none;border:0;color:var(--ink);font:600 13px Inter;min-height:44px;padding:0;text-align:left}
</style>
