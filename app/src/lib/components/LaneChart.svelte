<script lang="ts">
 import type {WeekLane} from '../generated/WeekLane';
 import {GOALS} from '../goals';
 import {weekLetters} from '../format';
 // One kind of training through the week: the cumulative line so far, the
 // goal as a dashed line, and the usual week as a band (middle half) with its
 // median. All values come from the core.
 let {lane: core, today, start}: {lane: WeekLane, today: number, start: string} = $props();
 const INFO = {cardio: {name: 'Cardio', unit: 'min', color: '#36C9C6'}, zone2: {name: GOALS.zone2.name, unit: 'min', color: GOALS.zone2.color},
  highIntensity: {name: GOALS.highIntensity.name, unit: 'min', color: GOALS.highIntensity.color}, intensity: {name: GOALS.intensity.name, unit: 'min', color: GOALS.intensity.color},
  strength: {name: 'Strength', unit: 'sessions', color: GOALS.strength.color}} as const;
 let lane = $derived({...INFO[core.kind], goal: core.goal, cum: core.cumulative, usual: core.usual ? {lo: core.usual.low, mid: core.usual.median, hi: core.usual.high} : null});
 let unknown = $derived(core.cumulative.slice(0, today + 1).some(v => v == null));
 let w = $state(320);
 const H = 92, L = 30, R = 44, T = 8, B = 18;
 let max = $derived(Math.max(1, lane.goal ?? 0, ...(lane.usual?.hi ?? []), ...lane.cum.map(v => v ?? 0)) * 1.08);
 const x = (i: number) => L + (w - L - R) * (i / 6);
 const y = (v: number) => T + (1 - v / max) * (H - T - B);
 let band = $derived(lane.usual ? `M${lane.usual.hi.map((v, i) => `${x(i)},${y(v)}`).join('L')}L${[...lane.usual.lo].reverse().map((v, i) => `${x(6 - i)},${y(v)}`).join('L')}Z` : '');
 let mid = $derived(lane.usual ? lane.usual.mid.map((v, i) => `${i ? 'L' : 'M'}${x(i)},${y(v)}`).join('') : '');
 // Sessions are whole: the strength lane steps up on the day; minutes slope.
 const steps = (vals: number[]) => vals.map((v, i) => i ? `H${x(i)}V${y(v)}` : `M${x(0)},${y(v)}`).join('');
 let line = $derived(lane.unit === 'sessions' ? steps(lane.cum.slice(0, lane.cum.findIndex(v => v == null) < 0 ? 7 : lane.cum.findIndex(v => v == null)) as number[]) : lane.cum.map((v, i) => v == null ? '' : `${i ? 'L' : 'M'}${x(i)},${y(v)}`).join(''));
 let now = $derived(lane.cum[today] ?? null);
 let usualNow = $derived(lane.usual ? Math.round(lane.usual.mid[today]) : null);
 const fmt = (v: number) => lane.unit === 'sessions' ? String(v) : `${Math.round(v)}`;
</script>
<div class="lane" bind:clientWidth={w}>
 <div class="lh"><b style:color={lane.color}>{lane.name}</b><span><b class="n">{now != null ? fmt(now) : '—'}</b>{lane.goal ? ` of ${lane.goal}` : ''} {lane.unit}{usualNow != null ? ` · usual by now ${usualNow}` : ''}</span></div>
 {#if unknown}<p class="unk">Awaiting data: an activity has no Garmin zones or a day hasn't synced.</p>{/if}
 <svg width={w} height={H} role="img" aria-label={`${lane.name}: ${now != null ? `${fmt(now)} ${lane.unit}` : 'unknown'} so far${lane.goal ? ` of a ${lane.goal} goal` : ''}${usualNow != null ? `; usual week by now ${usualNow}` : ''}`}>
  {#if band}<path d={band} fill={lane.color} opacity=".13"/><path d={mid} fill="none" stroke={lane.color} stroke-opacity=".5" stroke-dasharray="1.5 3"/>{/if}
  {#if lane.goal}<line x1={L} x2={w - R} y1={y(lane.goal)} y2={y(lane.goal)} stroke="#E4E8F0" stroke-dasharray="4 3"/><text class="gl" x={w - R + 4} y={y(lane.goal) + 3}>goal {lane.goal}</text>{/if}
  <path d={line} fill="none" stroke={lane.color} stroke-width="2.4" stroke-linejoin="round"/>
  {#each lane.cum as v, i}{#if v != null}<circle cx={x(i)} cy={y(v)} r={i === today ? 4 : 2.5} fill={lane.color}/>{/if}{/each}
  {#if now != null}<text class="vl" x={x(today)} y={y(now) - 7} text-anchor="middle">{fmt(now)}</text>{/if}
  <text class="yt" x={L - 4} y={y(0) + 3} text-anchor="end">0</text><text class="yt" x={L - 4} y={y(max / 1.08) + 3} text-anchor="end">{fmt(max / 1.08)}</text>
  {#each weekLetters(start) as d, i}<text class="xt" x={x(i)} y={H - 4} text-anchor="middle" class:on={i === today}>{d}</text>{/each}
 </svg>
</div>
<style>
 .lane{padding:6px 0;border-top:1px solid var(--line)}.lane:first-child{border-top:0}
 .lh{display:flex;justify-content:space-between;align-items:baseline;gap:8px;font-size:12px;color:var(--text2)}.lh>b{font:600 13px Inter}.lh .n{color:var(--text);font-size:15px}
 svg{display:block}text{font-family:Inter,sans-serif}.yt,.xt{font-size:10px;fill:var(--text2)}.xt.on{fill:var(--text);font-weight:700}.vl{font:600 10.5px Outfit;fill:var(--text)}.gl{font:600 9.5px Inter;fill:var(--text2)}
 .unk{margin:2px 0 0;font-size:11.5px;color:var(--text2)}
</style>
