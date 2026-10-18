<script lang="ts">
 import Icon from '../../lib/components/Icon.svelte';
 import {colors, stressColors, stressLevel} from '../../lib/charts';
 import {SPORTS} from '../lib/sports';
 import {hm, toMin, dur, dayLabel} from '../lib/dates';
 import {stagesFor} from '../lib/world';
 import type {Ctx} from '../lib/rules';
 import {bodyBatteryDay, timelines} from '../lib/views';
 // "Your day" at a glance: last night as one band in stage colours, Body Battery
 // as the line that tells the charge and drain, stress as a soft band, activities
 // and naps on their own lane, now, and tonight's window ahead. Each part opens
 // its detail. Sources: sleep stages, the stress timeline, activities and naps are
 // view fields; the Body Battery line is stored but not yet exposed (see the
 // provenance table in docs/experience-design.md).
 let {ctx, navigate}: {ctx: Ctx, navigate: (route: string) => void} = $props();
 const START = -120, END = 1500, W = 340, H = 132;
 const x = (m: number) => (Math.max(START, Math.min(END, m)) - START) / (END - START) * W;
 const px = (m: number) => x(m) / W * 100;
 const SLEEP = [4, 18], LANE = [26, 46], BB = [54, 104], STRESS = [108, 116];
 const py = (y: number) => y / H * 100;
 let day = $derived(ctx.clock.phase === 'late' ? ctx.day : ctx.calendarDate);
 let now = $derived(toMin(ctx.clock.time) + (ctx.clock.phase === 'late' ? 1440 : 0));
 let waiting = $derived(ctx.phase === 'waiting');
 let night = $derived(waiting ? null : ctx.nightOf(day));
 let bed = $derived(night ? (toMin(night.bed) > 720 ? toMin(night.bed) - 1440 : toMin(night.bed)) : -60);
 let wake = $derived(night ? toMin(night.wake) : 400);
 let stages = $derived(night ? stagesFor(day, night.asleep) : []);
 let total = $derived(stages.length ? stages.at(-1)!.startMin + stages.at(-1)!.minutes : 1);
 const stageColor: Record<string, string> = {deep: colors.deep, light: colors.light, rem: colors.rem, awake: '#C9D3E6'};
 let bb = $derived(waiting ? null : bodyBatteryDay(ctx, day, now));
 const by = (v: number) => BB[1] - v / 100 * (BB[1] - BB[0]);
 let bbLine = $derived(bb ? bb.points.map(([m, v], i) => `${i ? 'L' : 'M'}${x(m).toFixed(1)},${by(v).toFixed(1)}`).join('') : '');
 let bbArea = $derived(bb && bb.points.length ? `${bbLine}L${x(bb.points.at(-1)![0]).toFixed(1)},${BB[1]}L${x(bb.points[0][0]).toFixed(1)},${BB[1]}Z` : '');
 let nowV = $derived(bb?.now ?? null);
 let sampleMinute = $derived(bb?.points.at(-1)?.[0] ?? now);
 // Stress in 15-minute buckets, Garmin's category colours; gaps stay empty.
 let stress = $derived.by(() => {
  const s = timelines(ctx, day).stress, out: {x: number, w: number, c: string}[] = [];
  for (let i = 0; i < s.length; i += 5) {
   const vals = s.slice(i, i + 5).filter((v): v is number => v != null);
   if (vals.length < 2) continue;
   const avg = vals.reduce((a, b) => a + b, 0) / vals.length;
   out.push({x: x(i * 3), w: x(i * 3 + 15) - x(i * 3) + .4, c: stressColors[stressLevel(avg)]});
  }
  return out;
 });
 // Activities and naps share a lane; short blocks get room for their glyph.
 let blocks = $derived.by(() => {
  const items = [
   ...ctx.world.activities.filter(a => a.date === day).map(a => ({m: toMin(a.start), len: a.min, color: SPORTS[a.sport].color, icon: SPORTS[a.sport].icon, label: `${a.name}, ${a.start}, ${a.min} min`, go: `workout/${a.id}`})),
   ...ctx.napList.filter(n => n.start[0] === day).map(n => ({m: toMin(n.start[1]), len: n.min, color: colors.rem, icon: 'moon' as const, label: `Nap at ${n.start[1]}, ${n.min} min`, go: 'sleep/naps'})),
  ].sort((a, b) => a.m - b.m);
  let end = -Infinity;
  return items.map(it => { const x0 = Math.max(x(it.m), end + 2), w = Math.max(20, x(it.m + it.len) - x(it.m)); end = x0 + w; return {...it, x0, w}; });
 });
 let win0 = $derived(toMin(ctx.tonight.window ? ctx.tonight.windowStart : '23:20')), windDown = $derived(toMin(ctx.tonight.windDown));
</script>
<section class="ribbon card" aria-labelledby="ribbon-t">
 <h3 id="ribbon-t"><span>Your day{day !== ctx.calendarDate ? ` · ${dayLabel(day)}` : ''}</span><button class="key" onclick={() => navigate('help/legend')}>Key</button></h3>
 <div class="rb-wrap">
  <svg viewBox={`0 0 ${W} ${H}`} class="rb" aria-hidden="true">
   <defs>
    <linearGradient id="bbfill" x1="0" y1={BB[0]} x2="0" y2={BB[1]} gradientUnits="userSpaceOnUse"><stop stop-color="#3DDC97" stop-opacity=".32"/><stop offset="1" stop-color="#3DDC97" stop-opacity=".02"/></linearGradient>
    <filter id="soft" x="-5%" y="-50%" width="110%" height="200%"><feGaussianBlur stdDeviation="1.4 .6"/></filter>
    <clipPath id="sleepclip"><rect x={x(bed)} y={SLEEP[0]} width={x(wake) - x(bed)} height={SLEEP[1] - SLEEP[0]} rx="7"/></clipPath>
   </defs>
   {#each [0, 360, 720, 1080, 1440] as t}<line x1={x(t)} x2={x(t)} y1={SLEEP[0]} y2={STRESS[1]} stroke="rgba(255,255,255,.05)"/>{/each}
   <line x1="0" x2={W} y1={by(50)} y2={by(50)} stroke="rgba(255,255,255,.05)" stroke-dasharray="2 3"/>
   <!-- last night: one continuous band -->
   {#if night}
    <g clip-path="url(#sleepclip)">{#each stages as s}<rect x={x(bed + s.startMin / total * (wake - bed))} y={SLEEP[0]} width={x(bed + (s.startMin + s.minutes) / total * (wake - bed)) - x(bed + s.startMin / total * (wake - bed)) + .3} height={SLEEP[1] - SLEEP[0]} fill={stageColor[s.stage]}/>{/each}</g>
   {:else}<rect x={x(-60)} y={SLEEP[0]} width={x(420) - x(-60)} height={SLEEP[1] - SLEEP[0]} rx="7" fill="none" stroke="#8E98AC" stroke-dasharray="2 3"/>{/if}
   <!-- activity and nap lane -->
   <rect x="0" y={LANE[0]} width={W} height={LANE[1] - LANE[0]} rx="6" fill="rgba(255,255,255,.025)"/>
   {#each blocks as b}<rect x={b.x0} y={LANE[0] + 1} width={b.w} height={LANE[1] - LANE[0] - 2} rx="6" fill={b.color}/>{/each}
   <!-- stress: a soft band -->
   <g filter="url(#soft)" opacity=".8">{#each stress as c}<rect x={c.x} y={STRESS[0]} width={c.w} height={STRESS[1] - STRESS[0]} fill={c.c}/>{/each}</g>
   <!-- Body Battery -->
   {#if bb}<path d={bbArea} fill="url(#bbfill)"/><path d={bbLine} fill="none" stroke="#3DDC97" stroke-width="2" stroke-linejoin="round"/>{/if}
   <!-- the rest of the day is quieter, not hatched -->
   <rect x={x(now)} y="0" width={W - x(now)} height={STRESS[1] + 2} fill="#141a27" opacity=".55"/>
   <!-- tonight -->
   {#if !ctx.stale}
    <rect x={x(windDown)} y={SLEEP[0]} width={Math.max(0, x(win0) - x(windDown))} height={SLEEP[1] - SLEEP[0]} fill="#A9C8FF" opacity=".1"/>
    <rect x={x(win0)} y={SLEEP[0]} width={W - x(win0) + 8} height={SLEEP[1] - SLEEP[0]} rx="7" fill="rgba(169,200,255,.16)" stroke="rgba(169,200,255,.45)" stroke-dasharray="3 2"/>
   {/if}
   <!-- now -->
   <line x1={x(now)} x2={x(now)} y1={SLEEP[0]} y2={STRESS[1] + 2} stroke="#EEF1F7" stroke-opacity=".35"/>
   {#if nowV != null}<circle cx={x(sampleMinute)} cy={by(nowV)} r="7" fill="#3DDC97" opacity=".22"/><circle cx={x(sampleMinute)} cy={by(nowV)} r="4.2" fill="#EEF1F7" stroke="#0B0F1A" stroke-width="1.5"/>{/if}
   {#each [[0, '00'], [360, '06'], [720, '12'], [1080, '18'], [1440, '00']] as [t, l]}<text x={x(t as number)} y={H - 2} font-size="9" fill="#8E98AC" text-anchor="middle">{l}</text>{/each}
  </svg>
  {#if nowV != null}<span class="nowtag" style:left={px(sampleMinute) + '%'} style:top={py(by(nowV) - 16) + '%'}><b class="n">{nowV}</b></span>{/if}
  <!-- tappable parts -->
  <button class="hit" style:left={px(night ? bed : -60) + '%'} style:width={px(night ? wake : 420) - px(night ? bed : -60) + '%'} style:top="0" style:height={py(22) + '%'}
   aria-label={night ? `Last night's sleep, ${night.bed} to ${night.wake}, ${dur(night.asleep)}` : 'Last night: not synced yet'} onclick={() => navigate('sleep')}></button>
  {#each blocks as b}<button class="hit blk" style:left={b.x0 / W * 100 + '%'} style:width={b.w / W * 100 + '%'} style:top={py(LANE[0]) + '%'} style:height={py(LANE[1] - LANE[0]) + '%'} aria-label={b.label} onclick={() => navigate(b.go)}><Icon name={b.icon} size={12}/></button>{/each}
  {#if !ctx.stale}<button class="hit moon" style:left={px(win0) + '%'} style:right="0" style:top="0" style:height={py(22) + '%'} aria-label={`Tonight: ${ctx.tonight.window ? `Garmin's window ${ctx.tonight.window}` : 'asleep by 23:20'}`} onclick={() => navigate('sleep/tonight')}><Icon name="moon" size={11}/></button>{/if}
  {#if bb}<button class="hit" style:left="0" style:width={px(now) + '%'} style:top={py(BB[0]) + '%'} style:height={py(BB[1] - BB[0]) + '%'} aria-label={`Body Battery through the day, ${nowV} at watch upload ${ctx.upload.at}`} onclick={() => navigate('metric/bodyBattery')}></button>{/if}
  <button class="hit" style:left="0" style:width={px(now) + '%'} style:top={py(STRESS[0] - 2) + '%'} style:height={py(STRESS[1] - STRESS[0] + 6) + '%'} aria-label="Stress through the day" onclick={() => navigate('metric/stress')}></button>
 </div>
 <div class="rb-key">
  <button onclick={() => navigate('sleep')}><i class="sw sleep"></i>{night ? `Slept ${dur(night.asleep)}` : waiting ? 'Night syncing' : 'No night'}</button>
  {#if bb}<button onclick={() => navigate('metric/bodyBattery')}><i class="sw bb"></i>Body Battery</button>{/if}
  <button onclick={() => navigate('metric/stress')}><i class="sw st"></i>Stress</button>
  <span class="now">Now {hm(now)}</span>
 </div>
</section>
<style>
 .ribbon h3{margin-bottom:8px}
 .key{font:600 11px Inter;letter-spacing:.06em;color:var(--text3);background:none;border:0;padding:6px 0;min-height:32px;text-transform:uppercase;text-decoration:underline;text-underline-offset:3px}
 .rb-wrap{position:relative}
 .rb{display:block;width:100%;height:auto;overflow:visible}
 .hit{position:absolute;background:none;border:0;padding:0;border-radius:6px;cursor:pointer;min-height:24px}
 .hit:focus-visible{outline:2px solid var(--ink)}
 .hit.blk{display:grid;place-items:center;color:#0B0F1A}
 .hit.moon{display:flex;align-items:center;justify-content:flex-start;padding-left:6px;color:#A9C8FF}
 .nowtag{position:absolute;transform:translate(-50%,-50%);pointer-events:none;font-size:11px;color:#3DDC97;background:#141a27;padding:0 4px;border-radius:6px;line-height:1.3}
 .rb-key{display:flex;flex-wrap:wrap;gap:0 12px;margin-top:6px;font:500 12px Inter;color:var(--text2);align-items:center}
 .rb-key button{display:inline-flex;align-items:center;gap:6px;background:none;border:0;color:inherit;font:inherit;padding:6px 0;min-height:32px}
 .sw{width:12px;height:8px;border-radius:3px;display:inline-block}
 .sw.sleep{background:linear-gradient(90deg,#2F6FE0,#5C9BFF,#A9C8FF)}
 .sw.bb{height:3px;background:#3DDC97}
 .sw.st{background:linear-gradient(90deg,#B4BCCB,#3DDC97,#FFB547);filter:blur(.4px)}
 .rb-key .now{color:var(--text);font-weight:600;margin-left:auto}
</style>
