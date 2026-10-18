<script lang="ts">
 import Icon from '../../lib/components/Icon.svelte';
 import {colors, stressColors, stressLevel} from '../../lib/charts';
 import {SPORTS} from '../lib/sports';
 import {hm, toMin, dur} from '../lib/dates';
 import {stagesFor, type Activity} from '../lib/world';
 import type {Ctx} from '../lib/rules';
 import {timelines} from '../lib/views';
 // The signature "your day" strip: last night's stages, stress shading, heart
 // rate, activities and naps, now, and tonight's window ahead. Every part is an
 // existing view field (stages, stress and HR timelines, activities, naps, sleep plan).
 let {ctx, navigate}: {ctx: Ctx, navigate: (route: string) => void} = $props();
 const START = -150, END = 1530, W = 340;
 const x = (m: number) => (Math.max(START, Math.min(END, m)) - START) / (END - START) * W;
 const pct = (m: number) => x(m) / W * 100;
 let day = $derived(ctx.clock.phase === 'late' ? ctx.day : ctx.calendarDate);
 let now = $derived(toMin(ctx.clock.time) + (ctx.clock.phase === 'late' ? 1440 : 0));
 let waiting = $derived(ctx.phase === 'waiting');
 let night = $derived(waiting ? null : ctx.nightOf(day));
 let bed = $derived(night ? (toMin(night.bed) > 720 ? toMin(night.bed) - 1440 : toMin(night.bed)) : -60);
 let stages = $derived(night ? stagesFor(day, night.asleep) : []);
 let span = $derived(night ? toMin(night.wake) - bed : 0);
 let total = $derived(stages.length ? stages.at(-1)!.startMin + stages.at(-1)!.minutes : 1);
 let tl = $derived(timelines(ctx, day));
 let acts = $derived(ctx.world.activities.filter(a => a.date === day));
 let naps = $derived(ctx.napList.filter(n => n.start[0] === day));
 let windowStart = $derived(toMin(ctx.tonight.windowStart)), windowEnd = $derived(toMin(ctx.tonight.windowEnd)), windDown = $derived(toMin(ctx.tonight.windDown));
 const laneY = 8, laneH = 34;
 const stageY: Record<string, number> = {awake: 0, rem: 8, light: 16, deep: 24};
 let hrPath = $derived.by(() => {
  const v = tl.hr; let d = '', pen = false;
  const lo = 40, hi = 175;
  v.forEach((b, i) => { if (b == null) { pen = false; return; } const px = x(i * 2), py = 120 - (b - lo) / (hi - lo) * 46; d += `${pen ? 'L' : 'M'}${px.toFixed(1)},${py.toFixed(1)}`; pen = true; });
  return d;
 });
 let stressCells = $derived(tl.stress.map((v, i) => v == null ? null : {x: x(i * 3), w: Math.max(.6, x(i * 3 + 3) - x(i * 3)), c: stressColors[stressLevel(v)], o: .25 + v / 140}).filter(c => c != null));
 const actLabel = (a: Activity) => `${a.name}, ${a.start}, ${a.min} min`;
</script>
<section class="ribbon card" aria-labelledby="ribbon-t">
 <h3 id="ribbon-t"><span>Your day</span><span class="legend-btn"><button class="link" onclick={() => navigate('help/legend')}>Key</button></span></h3>
 <div class="rb-wrap">
  <svg viewBox={`0 0 ${W} 142`} class="rb" aria-hidden="true">
   <defs>
    <pattern id="future" width="5" height="5" patternUnits="userSpaceOnUse" patternTransform="rotate(45)"><rect width="1" height="5" fill="#fff" opacity=".05"/></pattern>
    <linearGradient id="wd" x1="0" x2="1"><stop stop-color="#A9C8FF" stop-opacity="0"/><stop offset="1" stop-color="#A9C8FF" stop-opacity=".28"/></linearGradient>
   </defs>
   {#each [0, 360, 720, 1080, 1440] as t}<line x1={x(t)} x2={x(t)} y1="4" y2="124" stroke="rgba(255,255,255,.06)"/>{/each}
   <!-- last night -->
   {#if night}
    {#each stages as s}<rect x={x(bed + s.startMin / total * span)} y={laneY + stageY[s.stage]} width={Math.max(.8, x(bed + (s.startMin + s.minutes) / total * span) - x(bed + s.startMin / total * span))} height={laneH - stageY[s.stage]} rx="1.5" fill={({deep: colors.deep, light: colors.light, rem: colors.rem, awake: colors.awake} as Record<string, string>)[s.stage]} opacity={s.stage === 'awake' ? .7 : 1}/>{/each}
   {:else}
    <rect x={x(-60)} y={laneY} width={x(400) - x(-60)} height={laneH} rx="6" fill="none" stroke="#8E98AC" stroke-dasharray="2 3"/>
   {/if}
   <!-- naps and activities -->
   {#each naps as n}<rect x={x(toMin(n.start[1]))} y={laneY + 10} width={Math.max(4, x(toMin(n.start[1]) + n.min) - x(toMin(n.start[1])))} height={laneH - 10} rx="3" fill={colors.rem} opacity=".85"/>{/each}
   {#each acts as a}<rect x={x(toMin(a.start))} y={laneY} width={Math.max(7, x(toMin(a.start) + a.min) - x(toMin(a.start)))} height={laneH} rx="5" fill={SPORTS[a.sport].color}/>{/each}
   <!-- tonight -->
   {#if !ctx.stale}
    <rect x={x(windDown)} y={laneY} width={x(windowStart) - x(windDown)} height={laneH} fill="url(#wd)"/>
    <rect x={x(windowStart)} y={laneY} width={Math.max(6, x(windowEnd) - x(windowStart))} height={laneH} rx="4" fill="rgba(169,200,255,.18)" stroke="#A9C8FF" stroke-width="1.2" stroke-dasharray="3 2"/>
    <rect x={x(windowEnd)} y={laneY + 14} width={x(END) - x(windowEnd)} height={laneH - 14} rx="2" fill="rgba(47,111,224,.18)"/>
   {/if}
   <!-- stress and heart rate -->
   <text x="0" y="55" font-size="7.5" fill="#8E98AC" font-weight="600">STRESS</text>
   {#each stressCells as c}<rect x={c.x} y="58" width={c.w} height="9" fill={c.c} opacity={c.o}/>{/each}
   <path d={hrPath} fill="none" stroke="#FF6B6B" stroke-opacity=".9" stroke-width="1.5" stroke-linejoin="round"/>
   <text x="0" y="80" font-size="7.5" fill="#8E98AC" font-weight="600">HR</text>
   <!-- future and now -->
   <rect x={x(now)} y="4" width={x(END) - x(now)} height="120" fill="url(#future)"/>
   <line x1={x(now)} x2={x(now)} y1="2" y2="126" stroke="#EEF1F7" stroke-width="1.4"/>
   <circle cx={x(now)} cy="2.5" r="2.5" fill="#EEF1F7"/>
   {#each [[0, '00'], [360, '06'], [720, '12'], [1080, '18'], [1440, '00']] as [t, l]}<text x={x(t as number)} y="139" font-size="9" fill="#8E98AC" text-anchor="middle">{l}</text>{/each}
  </svg>
  <!-- tappable parts -->
  {#if night}<button class="hit" style:left={pct(bed) + '%'} style:width={pct(toMin(night.wake)) - pct(bed) + '%'} aria-label={`Last night's sleep, ${night.bed} to ${night.wake}, ${dur(night.asleep)}`} onclick={() => navigate('sleep')}></button>
  {:else}<button class="hit" style:left={pct(-60) + '%'} style:width={pct(400) - pct(-60) + '%'} aria-label="Last night: waiting for your watch to sync" onclick={() => navigate('sleep')}></button>{/if}
  {#each naps as n}<button class="hit" style:left={pct(toMin(n.start[1])) + '%'} style:width={Math.max(4, pct(toMin(n.start[1]) + n.min) - pct(toMin(n.start[1]))) + '%'} aria-label={`Nap at ${n.start[1]}, ${n.min} min`} onclick={() => navigate('sleep/naps')}></button>{/each}
  {#each acts as a}<button class="hit act" style:left={pct(toMin(a.start)) + '%'} style:width={Math.max(5, pct(toMin(a.start) + a.min) - pct(toMin(a.start))) + '%'} aria-label={actLabel(a)} onclick={() => navigate(`workout/${a.id}`)}><Icon name={SPORTS[a.sport].icon} size={12}/></button>{/each}
  {#if !ctx.stale}<button class="hit" style:left={pct(windowStart - 20) + '%'} style:width={pct(windowEnd + 40) - pct(windowStart - 20) + '%'} aria-label={`Tonight: Garmin's window ${ctx.tonight.windowStart} to ${ctx.tonight.windowEnd}`} onclick={() => navigate('sleep/tonight')}></button>{/if}
  <button class="hit band" aria-label="Stress through the day" onclick={() => navigate('metric/stress')}></button>
  <button class="hit hrband" aria-label="Heart rate through the day" onclick={() => navigate('metric/heartRate')}></button>
 </div>
 <div class="rb-key" aria-hidden="true">
  {#if night}<span><i style:background={colors.light}></i>Slept {dur(night.asleep)}</span>{:else}<span><i class="dot"></i>{waiting ? 'Night syncing' : 'No night'}</span>{/if}
  {#if acts.length}<span><i style:background={SPORTS[acts[0].sport].color}></i>{acts.length === 1 ? acts[0].name : `${acts.length} activities`}</span>{/if}
  <span class="now">Now {hm(now)}</span>
  {#if !ctx.stale}<span><i class="win"></i>Tonight {ctx.tonight.windowStart}</span>{/if}
 </div>
</section>
<style>
 .ribbon h3{margin-bottom:6px}
 .legend-btn .link{font:600 11px Inter;letter-spacing:.06em;color:var(--text3);background:none;border:0;padding:6px 0;min-height:32px;text-transform:uppercase}
 .rb-wrap{position:relative}
 .rb{display:block;width:100%;height:auto;overflow:visible}
 .hit{position:absolute;top:5%;height:26%;background:none;border:0;padding:0;border-radius:6px;cursor:pointer}
 .hit:focus-visible{outline:2px solid var(--ink)}
 .hit.act{display:grid;place-items:center;color:#0B0F1A;min-width:24px;transform:translateX(0)}
 .hit.band{left:0;right:0;width:auto;top:38%;height:10%}
 .hit.hrband{left:0;right:0;width:auto;top:50%;height:36%}
 .rb-key{display:flex;flex-wrap:wrap;gap:6px 14px;margin-top:8px;font:500 12px Inter;color:var(--text2)}
 .rb-key span{display:inline-flex;align-items:center;gap:6px}
 .rb-key i{width:10px;height:10px;border-radius:3px;display:inline-block}
 .rb-key i.dot{border:1.5px dashed #8E98AC;background:none}
 .rb-key i.win{border:1.5px dashed #A9C8FF;background:rgba(169,200,255,.18)}
 .rb-key .now{color:var(--text);font-weight:600}
</style>
