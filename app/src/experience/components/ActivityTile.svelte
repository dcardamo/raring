<script lang="ts">
 import SportIcon from './SportIcon.svelte';
 import ZoneStrip from '../../lib/components/ZoneStrip.svelte';
 import BodyMap from '../../lib/components/BodyMap.svelte';
 import Icon from '../../lib/components/Icon.svelte';
 import RouteHero from './RouteHero.svelte';
 import Ring from './Ring.svelte';
 import {SPORTS} from '../lib/sports';
 import {contribution, type Ctx} from '../lib/rules';
 import {routeFor, setsFor, type Activity} from '../lib/world';
 import {strengthSession} from '../lib/views';
 import {elapsed} from '../../lib/format';
 import {dayLabel} from '../lib/dates';
 // A data-bearing activity preview: picture, the key numbers including heart
 // rate, zones, Garmin's benefit and effect, and what it did for the person's plan.
 let {a, ctx, navigate, showDate = false, contributes = false}: {a: Activity, ctx: Ctx, navigate: (route: string) => void, showDate?: boolean, contributes?: boolean} = $props();
 // Garmin's own labels for what a session worked: benefit, training effect,
 // recorded zone time, and the muscle groups of recorded sets.
 let worked = $derived.by(() => {
  if (a.timerOnly) return [];
  const out = [`aerobic ${a.aerobic.toFixed(1)} · anaerobic ${a.anaerobic.toFixed(1)}`];
  const z2 = Math.floor((a.zones[1] ?? 0) / 60), hi = Math.floor(((a.zones[3] ?? 0) + (a.zones[4] ?? 0)) / 60);
  if (z2) out.push(`Zone 2 ${z2} min`); if (hi) out.push(`Zone 4–5 ${hi} min`);
  if (session?.setsRecorded) out.push(session.muscles.filter(m => m.sets > 0).sort((x, y) => y.sets - x.sets).slice(0, 4).map(m => m.muscle).join(', '));
  return out;
 });
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
  if (sets) out.push([String(sets.reduce((n, l) => n + l.sets.length, 0)), '', 'Sets']);
  out.push([String(a.avgHr), 'bpm', 'Avg HR']);
  return out;
 });
</script>
<button class="atile" onclick={() => navigate(`workout/${a.id}`)} aria-label={`${a.name}, ${showDate ? dayLabel(a.date) + ' ' : ''}${a.start}. ${stats.map(s => `${s[2]} ${s[0]}${s[1] ? ' ' + s[1] : ''}`).join(', ')}. Garmin: ${a.timerOnly ? 'no sets recorded' : `${a.benefit} benefit, aerobic effect ${a.aerobic.toFixed(1)}`}. Open workout`}>
 <span class="pic" style:--c={SPORTS[a.sport].color}>
  {#if route.length}<RouteHero {route} color={SPORTS[a.sport].color} w={120} h={96} compact label={`${a.name} route`}/>
  {:else if session?.setsRecorded}<span class="bm"><BodyMap figure="female" muscles={session.muscles} width={56} max={6} legend={false}/></span>
  {:else}<span class="art"><SportIcon sport={a.sport} size={36}/></span>{/if}
  <span class="badge" style:background={SPORTS[a.sport].color}><SportIcon sport={a.sport} size={13}/></span>
 </span>
 <span class="head">
  <span class="top"><b>{a.name}</b><small>{showDate ? dayLabel(a.date) + ' · ' : ''}{a.start}</small></span>
  <span class="effect">{#if a.timerOnly}<Icon name="clock" size={12}/>No sets recorded{:else}<em>Garmin</em><b>{a.benefit}</b><span class="nw">· aerobic <b class="n">{a.aerobic.toFixed(1)}</b></span>{/if}</span>
  <span class="foot">
   {#if a.record}<span class="tag best"><Icon name="star" size={12}/>Best</span>{/if}
   {#if c.ring && a.date >= ctx.weekFrom}<span class="contrib"><Ring value={c.ring[0]} target={c.ring[1]} size={22} stroke={3} center={false} state={c.ring[0] >= c.ring[1] ? 'reached' : 'open'} label={c.short}/>{c.short}</span>{/if}
  </span>
 </span>
 <span class="stats">{#each stats as [v, u, l]}<span><b class="n">{v}<small>{u}</small></b><em>{l}</em></span>{/each}</span>
 <span class="zones"><ZoneStrip seconds={a.zones}/></span>
 {#if contributes && worked.length}<span class="worked"><em>Worked</em>{a.benefit} · {worked.join(' · ')}</span>{/if}
</button>
<style>
 .atile{display:grid;grid-template-columns:96px 1fr;grid-template-areas:'pic head' 'stats stats' 'zones zones' 'worked worked';column-gap:12px;width:100%;text-align:left;background:var(--card);border:1px solid var(--line);border-radius:20px;padding:10px 12px 12px 10px;margin-top:10px;color:var(--text)}
 .pic{grid-area:pic;position:relative;border-radius:14px;overflow:hidden;height:80px;background:radial-gradient(120% 90% at 30% 20%,color-mix(in srgb,var(--c) 26%,#121829),#0E1424);display:grid;place-items:center}
 .pic :global(.routehero){position:absolute;inset:0}
 .art{color:var(--c);opacity:.9}
 .bm{display:grid;place-items:center;padding-top:2px}
 .badge{position:absolute;left:6px;top:6px;width:22px;height:22px;border-radius:50%;display:grid;place-items:center;color:#0B0F1A;box-shadow:0 0 0 2px #0E1424}
 .head{grid-area:head;display:flex;flex-direction:column;gap:4px;min-width:0;padding-top:2px}
 .top{display:flex;justify-content:space-between;gap:8px;align-items:baseline}
 .top b{font:600 16px Outfit,Inter;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
 .top small{color:var(--text3);font-size:12px;flex:none}
 .effect{display:flex;align-items:center;gap:4px;flex-wrap:wrap;font-size:12px;color:var(--text2)}
 .effect em{font-style:normal;font:700 9.5px Inter;letter-spacing:.08em;text-transform:uppercase;color:var(--text3);border:1px solid var(--line);border-radius:5px;padding:1px 4px;margin-right:2px}
 .effect b{color:var(--text);font-weight:600}
 .nw{white-space:nowrap}
 .foot{display:flex;flex-wrap:wrap;gap:6px 10px;align-items:center;font-size:12px;color:var(--text2)}
 .tag{display:inline-flex;align-items:center;gap:4px;padding:3px 8px;border-radius:999px;font-weight:600;font-size:11px}
 .tag.best{color:#FFD666;background:rgba(255,214,102,.12)}
 .contrib{display:inline-flex;align-items:center;gap:6px;font-weight:600;color:var(--text)}
 .stats{grid-area:stats;display:grid;grid-template-columns:repeat(4,minmax(0,1fr));gap:6px;margin-top:10px}
 .stats b{display:block;color:var(--text);font-size:18px;font-weight:600;line-height:1.1;white-space:nowrap}
 .stats b small{font:500 10.5px Inter;color:var(--text3);margin-left:1px}
 .stats em{font-style:normal;font-size:11px;color:var(--text3)}
 .zones{grid-area:zones}
 .worked{grid-area:worked;margin-top:8px;font-size:11.5px;color:var(--text2);text-transform:none}.worked em{font-style:normal;font:700 9.5px Inter;letter-spacing:.08em;text-transform:uppercase;color:var(--text3);margin-right:6px}
 .zones :global(.zbar){margin-top:10px;height:6px}
 @media(max-width:360px){.stats b{font-size:16px}}
</style>
