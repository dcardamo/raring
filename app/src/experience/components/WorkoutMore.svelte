<script lang="ts">
 import AxisChart from './AxisChart.svelte';
 import HelpButton from './HelpButton.svelte';
 import {store} from '../lib/store.svelte';
 import {summaryFor, streamsFor, effectLine} from '../lib/streams';
 import type {Activity} from '../lib/world';
 // More of what Garmin stored for this activity: time-aligned charts, effort in
 // plain words, and only the fields this sport and watch recorded.
 let {a}: {a: Activity} = $props();
 let s = $derived(summaryFor(a));
 let st = $derived(streamsFor(a));
 let ride = $derived(a.sport === 'cycling');
 const pace = (kmh: number) => { const sec = 3600 / kmh; return `${Math.floor(sec / 60)}:${String(Math.round(sec % 60)).padStart(2, '0')}`; };
 let pts = $derived(st ? st.filter((_, i) => i % Math.max(1, Math.ceil(st!.length / 36)) === 0) : []);
 const x = (t: number) => `${Math.round(t)}′`;
 let aer = $derived(effectLine('aerobic', a.aerobic)), ana = $derived(effectLine('anaerobic', a.anaerobic));
 let rows = $derived([
  ['Moving time', `${s.movingMin} min`],
  s.elevationGain != null ? ['Elevation', `+${s.elevationGain} m / −${s.elevationLoss} m`] : null,
  s.cadence != null ? ['Cadence', `${s.cadence} ${s.cadenceUnit}`] : null,
  s.strideM != null ? ['Stride length', `${s.strideM} m`] : null,
  s.gctMs != null ? ['Ground contact', `${s.gctMs} ms`] : null,
  s.verticalOscCm != null ? ['Vertical oscillation', `${s.verticalOscCm} cm`] : null,
  s.avgPower != null ? ['Average power', `${s.avgPower} W`] : null,
  s.np != null ? ['Normalized power', `${s.np} W`] : null,
  s.intensityFactor != null ? ['Intensity factor', `${s.intensityFactor} (FTP ${s.ftp} W)`] : null,
  s.maxPower20 != null ? ['Best 20 min power', `${s.maxPower20} W`] : null,
  s.avgRespiration != null ? ['Breathing rate', `${s.avgRespiration} brpm`] : null,
  s.recoveryHr != null ? ['Recovery heart rate', `−${s.recoveryHr} bpm in 2 min`] : null,
  s.temperature != null ? ['Temperature', `${s.temperature} °C`] : null,
 ].filter((r): r is string[] => !!r));
</script>
{#if st}
 <section class="card"><div class="wm-h"><h3>Along the way</h3><span class="small">same time axis</span></div>
  <AxisChart points={pts.map(p => ({x: x(p.t), v: p.bpm}))} kind="line" unit="bpm" color="#FF6B8A" height={100} label={`Heart rate over ${a.min} min`}/>
  <AxisChart points={pts.map(p => ({x: x(p.t), v: p.speed}))} kind="line" unit={ride ? 'km/h' : 'km/h · pace'} color="#36C9C6" height={100} fmt={v => ride ? String(Math.round(v)) : pace(v)} label={`${ride ? 'Speed' : 'Pace'} over ${a.min} min`}/>
  <AxisChart points={pts.map(p => ({x: x(p.t), v: p.elev}))} kind="line" unit="m" color="#D9C089" height={90} label={`Elevation over ${a.min} min`}/>
 </section>
{/if}
{#if !a.timerOnly}
 <section class="card"><div class="wm-h"><h3>What it did</h3><HelpButton topic="training-effect"/></div>
  <div class="te"><span><small>Aerobic · Garmin</small><b class="n">{a.aerobic.toFixed(1)}</b><em>{aer.band}</em></span><p>{aer.what}</p></div>
  <div class="te"><span><small>Anaerobic · Garmin</small><b class="n">{a.anaerobic.toFixed(1)}</b><em>{ana.band}</em></span><p>{ana.what}</p></div>
  <dl class="kv">
   <dt>Primary benefit</dt><dd>{a.benefit}</dd>
   <dt>Intensity minutes</dt><dd>{s.moderate + 2 * s.vigorous} <small>{s.moderate} moderate + 2 × {s.vigorous} vigorous</small></dd>
   {#if s.bodyBatteryDiff != null}<dt>Body Battery</dt><dd>{s.bodyBatteryDiff}</dd>{/if}
   {#if s.staminaStart != null}<dt>Stamina</dt><dd>{s.staminaStart}% → {s.staminaEnd}%</dd>{/if}
   {#if s.performanceCondition != null}<dt>Performance condition</dt><dd>{s.performanceCondition > 0 ? '+' : ''}{s.performanceCondition}</dd>{/if}
  </dl>
  {#if s.vo2Estimate}<button class="link" onclick={() => store.navigate('metric/vo2max')}>Garmin recorded a new VO₂ max estimate today: {s.vo2Estimate}</button>{/if}
 </section>
 {#if rows.length > 1}<section class="card"><h3>Recorded by your watch</h3><dl class="kv">{#each rows as [k, v]}<dt>{k}</dt><dd>{v}</dd>{/each}</dl></section>{/if}
{/if}
<style>
 .wm-h{display:flex;justify-content:space-between;align-items:center;margin-bottom:6px}.wm-h h3{margin:0}
 .te{display:grid;grid-template-columns:auto 1fr;gap:4px 12px;align-items:center;padding:8px 0;border-bottom:1px solid var(--line)}.te span{display:grid}.te small{font-size:10.5px;color:var(--text3)}.te b{font-size:24px;font-weight:600}.te em{font-style:normal;font-size:12px;color:#3DDC97}.te p{margin:0;font-size:14px}
 .kv small{display:block;font:500 10.5px Inter;color:var(--text3)}
</style>
