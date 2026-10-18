<script lang="ts">
 import ZoneStrip from '../../lib/components/ZoneStrip.svelte';
 import BodyMap from '../../lib/components/BodyMap.svelte';
 import Icon from '../../lib/components/Icon.svelte';
 import RouteHero from './RouteHero.svelte';
 import Ring from './Ring.svelte';
 import {SPORTS} from '../lib/sports';
 import {contribution, type Ctx} from '../lib/rules';
 import {routeFor, hrTrace, setsFor, type Activity} from '../lib/world';
 import {strengthSession} from '../lib/views';
 import {elapsed} from '../../lib/format';
 import {dayLabel} from '../lib/dates';
 // A data-bearing activity preview: picture, three or four numbers, zones and
 // what Garmin and the person's plan say about it. Opens the canonical workout.
 let {a, ctx, navigate, showDate = false}: {a: Activity, ctx: Ctx, navigate: (route: string) => void, showDate?: boolean} = $props();
 let route = $derived(routeFor(a));
 let sets = $derived(setsFor(a));
 let session = $derived(a.sport === 'strength' ? strengthSession(a) : null);
 let c = $derived(contribution(ctx, a));
 let stats = $derived.by(() => {
  const out: [string, string, string][] = [];
  if (a.km) out.push([String(a.km), 'km', 'Distance']);
  out.push([elapsed(a.min * 60), '', 'Time']);
  if (a.km && a.sport === 'cycling') out.push([(a.km / (a.min / 60)).toFixed(1), 'km/h', 'Speed']);
  else if (a.km) out.push([elapsed(a.min * 60 / a.km), '/km', 'Pace']);
  if (sets) return [[String(sets.filter(l => l.key).length), '', 'Exercises'], [String(sets.reduce((n, l) => n + l.sets.length, 0)), '', 'Sets'], [elapsed(a.min * 60), '', 'Time']] as [string, string, string][];
  out.push([String(a.avgHr), 'bpm', 'Avg HR']);
  return out.slice(0, 3);
 });
</script>
<button class="atile" onclick={() => navigate(`workout/${a.id}`)} aria-label={`${a.name}, ${showDate ? dayLabel(a.date) + ' ' : ''}${a.start}. ${stats.map(s => `${s[2]} ${s[0]}${s[1] ? ' ' + s[1] : ''}`).join(', ')}. Open workout`}>
 <span class="pic" style:--c={SPORTS[a.sport].color}>
  {#if route.length}<RouteHero {route} samples={hrTrace(a)} floors={[118, 136, 150, 163]} color={SPORTS[a.sport].color} w={120} h={96} compact label={`${a.name} route`}/>
  {:else if session?.setsRecorded}<span class="bm"><BodyMap figure="female" muscles={session.muscles} width={58} max={6} legend={false}/></span>
  {:else}<span class="art"><Icon name={SPORTS[a.sport].icon} size={40}/></span>{/if}
  <span class="badge" style:background={SPORTS[a.sport].color}><Icon name={SPORTS[a.sport].icon} size={13}/></span>
 </span>
 <span class="body">
  <span class="top"><b>{a.name}</b><small>{showDate ? dayLabel(a.date) + ' · ' : ''}{a.start}</small></span>
  <span class="stats">{#each stats as [v, u, l]}<span><b class="n">{v}<small>{u}</small></b><em>{l}</em></span>{/each}</span>
  <ZoneStrip seconds={a.zones}/>
  <span class="foot">
   {#if a.timerOnly}<span class="tag"><Icon name="clock" size={12}/>No sets recorded</span>{:else}<span class="tag">Garmin · {a.benefit} {a.aerobic.toFixed(1)}</span>{/if}
   <span class="tag"><Icon name="heart" size={11}/>{a.avgHr}</span>
   {#if a.record}<span class="tag best"><Icon name="star" size={12}/>Best</span>{/if}
   {#if c.ring && a.date >= ctx.weekFrom}<span class="contrib"><Ring value={c.ring[0]} target={c.ring[1]} size={22} stroke={3} center={false} state={c.ring[0] >= c.ring[1] ? 'reached' : 'open'} label={c.short}/>{c.short}</span>{/if}
  </span>
 </span>
</button>
<style>
 .atile{display:grid;grid-template-columns:112px 1fr;gap:14px;width:100%;text-align:left;background:var(--card);border:1px solid var(--line);border-radius:20px;padding:10px;margin-top:10px;align-items:stretch}
 .pic{position:relative;border-radius:14px;overflow:hidden;min-height:104px;background:radial-gradient(120% 90% at 30% 20%,color-mix(in srgb,var(--c) 26%,#121829),#0E1424);display:grid;place-items:center}
 .pic :global(.routehero){position:absolute;inset:0}
 .art{color:var(--c);opacity:.9}
 .bm{display:grid;place-items:center;padding-top:4px}
 .badge{position:absolute;left:6px;top:6px;width:22px;height:22px;border-radius:50%;display:grid;place-items:center;color:#0B0F1A;box-shadow:0 0 0 2px #0E1424}
 .body{display:flex;flex-direction:column;min-width:0;padding:2px 2px 0 0}
 .top{display:flex;justify-content:space-between;gap:8px;align-items:baseline}
 .top b{font:600 16px Outfit,Inter;color:var(--text);white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
 .top small{color:var(--text3);font-size:12px;flex:none}
 .stats{display:grid;grid-template-columns:repeat(auto-fit,minmax(54px,1fr));gap:4px 8px;margin-top:8px}
 .stats b{display:block;font-size:17px;font-weight:600;color:var(--text);line-height:1.1}
 .stats b small{font:500 11px Inter;color:var(--text3);margin-left:2px}
 .stats em{font-style:normal;font-size:11px;color:var(--text3)}
 .body :global(.zbar){margin-top:9px;height:6px}
 .foot{display:flex;flex-wrap:wrap;gap:6px 10px;align-items:center;margin-top:9px;font-size:12px;color:var(--text2)}
 .tag{display:inline-flex;align-items:center;gap:4px;padding:3px 8px;border-radius:999px;background:rgba(255,255,255,.06);font-weight:600;font-size:11px;color:var(--text2)}
 .tag.best{color:#FFD666;background:rgba(255,214,102,.12)}
 .contrib{display:inline-flex;align-items:center;gap:6px;font-weight:600;font-size:12px;color:var(--text)}
 @media(max-width:360px){.atile{grid-template-columns:92px 1fr;gap:10px}.pic{min-height:96px}}
</style>
