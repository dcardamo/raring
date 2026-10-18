<script lang="ts">
 import BodyMap from '../../lib/components/BodyMap.svelte';
 import Icon from '../../lib/components/Icon.svelte';
 import RootHeader from '../components/RootHeader.svelte';
 import ActivityTile from '../components/ActivityTile.svelte';
 import Glyph from '../components/Glyph.svelte';
 import AxisChart from '../components/AxisChart.svelte';
 import HelpButton from '../components/HelpButton.svelte';
 import NestedRings from '../components/NestedRings.svelte';
 import {store} from '../lib/store.svelte';
 import {trainingView, strengthView} from '../lib/views';
 import {recordedFor, goalsFor, GOAL_INFO} from '../lib/allocation';
 import {weekRings} from '../lib/week';
 import {SPORTS, type Sport} from '../lib/sports';
 import {addDays, between, dayLabel, dayMonth, narrowDay, weekRange, dur} from '../lib/dates';
 import {MUSCLES} from '../lib/views';
 // Training answers three questions (enough or too much? improving? ready for
 // what?) after one labelled week chart; strength and activities have their own tabs.
 let ctx = $derived(store.ctx);
 const go = (r: string) => store.navigate(r);
 let tab = $derived((store.ui.section as string) ?? 'fitness');
 let tv = $derived(trainingView(ctx));
 let week = $derived(ctx.world.activities.filter(a => between(a.date, ctx.weekFrom, ctx.calendarDate)));
 let days = $derived(Array.from({length: 7}, (_, i) => addDays(ctx.weekFrom, i)));
 let usualDay = $derived(Math.round(ctx.world.activities.filter(a => between(a.date, addDays(ctx.weekFrom, -56), addDays(ctx.weekFrom, -1))).reduce((t, a) => t + a.min, 0) / 56));
 let elapsedDays = $derived(days.filter(d => d <= ctx.calendarDate).length);
 let total = $derived(week.reduce((t, a) => t + a.min, 0));
 let bySport = $derived.by(() => { const m = new Map<Sport, number>(); for (const a of week) m.set(a.sport, (m.get(a.sport) ?? 0) + a.min); return [...m.entries()].sort((a, b) => b[1] - a[1]); });
 let bars = $derived(days.map(d => { const acts = week.filter(a => a.date === d); return {x: narrowDay(d), v: d > ctx.calendarDate ? null : acts.reduce((t, a) => t + a.min, 0), parts: acts.map(a => ({v: a.min, color: SPORTS[a.sport].color}))}; }));
 let wr = $derived(weekRings(ctx));
 // Fitness
 let acute = $derived(tv.acute);
 let vo2 = $derived(tv.vo2maxTrend.map((v, i) => ({x: dayMonth(addDays(ctx.calendarDate, -7 * (tv.vo2maxTrend.length - 1 - i))), v})));
 const TH: Record<string, [string, (v: number) => string]> = {runningHr: ['Lactate threshold HR', v => `${v} bpm`], runningPace: ['Threshold pace', v => `${Math.floor(v / 60)}:${String(v % 60).padStart(2, '0')} /km`], runningPower: ['Threshold power', v => `${v} W`], cyclingFtp: ['Cycling FTP', v => `${v} W`], heat: ['Heat acclimation', v => `${v}%`], altitude: ['Altitude acclimation', v => `${v}%`]};
 const mmss = (s: number) => s >= 3600 ? `${Math.floor(s / 3600)}:${String(Math.floor(s % 3600 / 60)).padStart(2, '0')}:${String(Math.round(s % 60)).padStart(2, '0')}` : `${Math.floor(s / 60)}:${String(Math.round(s % 60)).padStart(2, '0')}`;
 // Strength
 let sv = $derived(strengthView(ctx));
 let g = $derived(goalsFor(ctx));
 let strengthRing = $derived(wr.rings.filter(r => r.key === 'strength'));
 let gaps = $derived(sv ? MUSCLES.map(m => ({m, sets: sv!.last4w.find(x => x.muscle === m)?.sets ?? 0})).sort((a, b) => a.sets - b.sets) : []);
 let lifts = $derived(sv ? sv.exercises.filter(e => e.sessions.length >= 3).sort((a, b) => b.sessions.length - a.sessions.length).slice(0, 3) : []);
 // Activities
 let all = $derived(ctx.world.activities.slice().reverse());
 let filter = $derived((store.ui.filter as string) ?? 'all');
 let list = $derived(all.filter(a => filter === 'all' || a.sport === filter).slice(0, (store.ui.limit as number) ?? 12));
 let sports = $derived([...new Set(all.map(a => a.sport))]);
</script>
<RootHeader title="Training" help="training-week"/>

<section class="tw" aria-label="This week's training">
 <div class="tw-h"><div><small>This week · {weekRange(ctx.weekFrom)}</small><b class="n">{dur(total)}</b><span class="vs">usual by {dayLabel(ctx.calendarDate).slice(0, 3)}: {dur(usualDay * elapsedDays)}</span></div>
  {#if wr.rings.length}<button class="goals" onclick={() => go('plan')} aria-label="Weekly goals in Plan">{#each wr.rings.filter(r => r.key !== 'sleep') as r}<span style:--c={r.color}><i></i>{r.name} <b class="n">{r.recorded}/{g[r.key]}</b></span>{/each}</button>{/if}</div>
 <AxisChart points={bars} unit="min" goal={usualDay} goalLabel="usual day" height={150} label={`Minutes per day this week: ${bars.map(b => `${b.x} ${b.v ?? 'future'}`).join(', ')}; usual day ${usualDay} min`}/>
 {#if bySport.length}<div class="tw-mix">{#each bySport as [s, m]}<span><Glyph sport={s} size={18}/>{SPORTS[s].name}<b class="n">{m} min</b></span>{/each}</div>{/if}
</section>

<div class="seg" role="tablist" aria-label="Training section">
 {#each [['fitness', 'Fitness'], ['strength', 'Strength'], ['activities', 'Activities']] as [k, l]}<button role="tab" aria-selected={tab === k} onclick={() => store.setUi({section: k})}>{l}</button>{/each}
</div>

{#if tab === 'fitness'}
 <section class="card q"><div class="q-h"><h2>Enough, or too much?</h2><HelpButton topic="acute-load"/></div>
  {#if acute}
   <div class="load"><div><small>Acute load · Garmin</small><b class="n">{acute.value}</b><span class="st" class:good={acute.label === 'Optimal'}>{acute.label}{tv.status ? ` · ${tv.status}` : ''}</span></div>
    <div class="lbar" role="img" aria-label={`Acute load ${acute.value}; Garmin's optimal band ${acute.band?.[0]}–${acute.band?.[1]}`}>
     {#if acute.band}<i class="band" style:left={acute.band[0] / acute.max * 100 + '%'} style:width={(acute.band[1] - acute.band[0]) / acute.max * 100 + '%'}></i>{/if}<u style:left={Math.min(100, acute.value / acute.max * 100) + '%'}></u>
     <span class="lt"><em>0</em>{#if acute.band}<em style:left={acute.band[0] / acute.max * 100 + '%'}>{acute.band[0]}</em><em style:left={acute.band[1] / acute.max * 100 + '%'}>{acute.band[1]}</em>{/if}<em style:left="100%">{acute.max}</em></span></div></div>
  {/if}
  <div class="q-sub"><h3>Load focus · last 4 weeks</h3><HelpButton topic="load-focus"/></div>
  {#each tv.loadFocus as f}{@const mx = Math.max(f.value, f.target?.[1] ?? 0) * 1.15}
   <div class="lf"><span>{f.name}<b class="n">{f.value}</b>{#if f.shortage}<em>short</em>{/if}</span>
    <div class="lfb" role="img" aria-label={`${f.name} ${f.value}, target ${f.target?.[0]}–${f.target?.[1]}`}>{#if f.target}<i class="band" style:left={f.target[0] / mx * 100 + '%'} style:width={(f.target[1] - f.target[0]) / mx * 100 + '%'}></i>{/if}<i class="val" style:width={f.value / mx * 100 + '%'}></i></div></div>
  {/each}
 </section>

 <section class="card q"><div class="q-h"><h2>Is my fitness improving?</h2><HelpButton topic="vo2max"/></div>
  <div class="vo"><span><small>VO₂ max · Garmin</small><b class="n">{tv.vo2max ?? '—'}</b>{#if tv.vo2maxChange30d}<em>{tv.vo2maxChange30d > 0 ? '+' : ''}{tv.vo2maxChange30d} in 30 days</em>{/if}</span><button class="link" onclick={() => go('metric/vo2max')}>All history</button></div>
  <AxisChart points={vo2} kind="dots" unit="ml/kg/min" color="#E4E8F0" height={120} fmt={v => v.toFixed(0)} label={`VO₂ max weekly estimates: ${vo2.map(p => p.v ?? 'none').join(', ')}`}/>
  <div class="scores">
   <button onclick={() => go('metric/endurance')}><small>Endurance score</small><b class="n">{tv.enduranceScore ?? '—'}</b></button>
   <button onclick={() => go('metric/hill')}><small>Hill score</small><b class="n">{tv.hillScore ?? '—'}</b></button>
   {#if tv.runningToleranceKm}<button onclick={() => go('metric/runningTolerance')}><small>Running tolerance</small><b class="n">{tv.runningToleranceKm} km</b></button>{/if}
  </div>
 </section>

 <section class="card q"><div class="q-h"><h2>What am I ready for?</h2><HelpButton topic="ready-for"/></div>
  {#if tv.races.length}<div class="races">{#each tv.races as r}<span><small>{r.label}</small><b class="n">{mmss(r.seconds)}</b></span>{/each}</div><p class="small">Garmin’s race predictions</p>{/if}
  <dl class="kv th">{#each tv.thresholds as t}{@const d = TH[t.kind]}{#if d}<dt>{d[0]}</dt><dd>{d[1](t.value)} <small>{dayMonth(t.recordedOn)}{t.stale ? ' · old' : ''}</small></dd>{/if}{/each}</dl>
  <button class="link" onclick={() => go('training/performance')}>Thresholds in detail</button>
 </section>

{:else if tab === 'strength'}
 <section class="card st-goal">
  {#if strengthRing.length}<NestedRings rings={strengthRing} size={64} stroke={7}/><div><small>Strength this week</small><b class="n">{strengthRing[0].recorded} of {g.strength}</b><span>{strengthRing[0].planned ? `${strengthRing[0].planned} placed · ` : ''}<button class="link inline" onclick={() => go('plan')}>Plan</button></span></div>
  {:else}<div><small>Strength this week</small><b class="n">{recordedFor(week, 'strength')} sessions</b><span>No strength goal · <button class="link inline" onclick={() => store.openSheet('goals')}>Add one</button></span></div>{/if}
 </section>
 {#if sv}
  <section class="card q"><div class="q-h"><h2>Muscles, last 4 weeks</h2><HelpButton topic="strength-gaps"/></div>
   <div class="mus"><BodyMap figure="female" muscles={sv.last4w} width={120} max={16} legend={false}/>
    <ul>{#each gaps as x, i}<li class:gap={x.sets <= 2}><span>{x.m}</span><b class="n">{x.sets}</b><small>sets</small></li>{#if i === 4}<li class="more"><button class="link" onclick={() => go('strength/muscles')}>All groups over time</button></li>{/if}{/each}</ul></div>
   {#if gaps.some(x => x.sets <= 2)}<p class="small">Few or no sets: {gaps.filter(x => x.sets <= 2).map(x => x.m).join(', ')}</p>{/if}
  </section>
  {#if lifts.length}<section class="card q"><div class="q-h"><h2>Lifts</h2><button class="link" onclick={() => go('strength')}>All exercises</button></div>
   {#each lifts as e}{@const pts = e.sessions.map(s => ({x: dayMonth(s.date), v: s.top?.weight ?? null}))}<div class="lift"><button class="lift-h" onclick={() => go(`strength/${e.name.toLowerCase().replace(/[^a-z]+/g, '-')}`)}><b>{e.name}</b><small>heaviest set each session</small><Icon name="chevron" size={14}/></button>
    <AxisChart points={pts} kind="line" unit="kg" color="#FF7DB8" height={110} fmt={v => String(Math.round(v * 2) / 2)} label={`${e.name}: heaviest set per session, latest ${pts.findLast(p => p.v != null)?.v ?? 'none'} kg`}/></div>{/each}
  </section>{/if}
 {:else}<section class="card"><p>No strength sessions in the last 12 weeks.</p><button class="link" onclick={() => go('help/strength-start')}>Lift with your watch</button></section>{/if}

{:else}
 <div class="q-h act-h"><span class="small">Garmin’s benefit, training effect, zones and muscles for each</span><HelpButton topic="activity-contribution"/></div>
 <div class="chiprow filt" role="group" aria-label="Filter by sport">{#each ['all', ...sports] as s}<button class="chipb" aria-pressed={filter === s} onclick={() => store.setUi({filter: s, limit: 12})}>{#if s !== 'all'}<Glyph sport={s as Sport} size={20}/>{/if}{s === 'all' ? 'All' : SPORTS[s as Sport].name}</button>{/each}</div>
 {#each list as a (a.id)}<ActivityTile {a} {ctx} navigate={go} showDate contributes/>{/each}
 <button class="secondary" onclick={() => go('activities')}>All activities</button>
{/if}

<style>
 .tw{margin-top:12px;padding:14px;border-radius:22px;background:linear-gradient(160deg,rgba(54,201,198,.08),var(--card) 60%);border:1px solid var(--line)}
 .tw-h{display:flex;justify-content:space-between;align-items:flex-start;gap:10px;flex-wrap:wrap;margin-bottom:6px}.tw-h small{font-size:11px;color:var(--text3)}.tw-h b{display:block;font-size:28px;font-weight:600}.vs{font-size:12px;color:var(--text2)}
 .goals{display:grid;gap:4px;background:none;border:0;color:var(--text2);font-size:12px;text-align:right;padding:0}.goals span{display:flex;gap:5px;align-items:center;justify-content:flex-end}.goals i{width:8px;height:8px;border-radius:50%;background:var(--c)}.goals b{color:var(--text);font-size:13px;display:inline}
 .tw-mix{display:flex;flex-wrap:wrap;gap:6px 14px;margin-top:8px;font-size:12px;color:var(--text2)}.tw-mix span{display:inline-flex;gap:6px;align-items:center}.tw-mix b{color:var(--text);font-weight:600}
 .seg{display:grid;grid-template-columns:repeat(3,1fr);gap:4px;padding:4px;border-radius:14px;background:rgba(255,255,255,.05);margin:14px 0 4px;position:sticky;top:0;z-index:3;backdrop-filter:blur(8px)}.seg button{height:38px;border-radius:10px;border:0;background:none;color:var(--text2);font:600 13px Inter}.seg button[aria-selected="true"]{background:var(--ink);color:var(--bg)}
 .q-h{display:flex;justify-content:space-between;align-items:center;gap:8px;margin-bottom:10px}.q-h h2{font:600 18px Outfit}.q-sub{display:flex;justify-content:space-between;align-items:center;margin:14px 0 6px}.q-sub h3{margin:0}
 .load{display:grid;gap:10px}.load small{font-size:11px;color:var(--text3)}.load b{display:block;font-size:30px;font-weight:600;line-height:1.1}.st{font-size:12px;color:var(--text2)}.st.good{color:#3DDC97}
 .lbar{position:relative;height:10px;border-radius:5px;background:rgba(255,255,255,.08);margin:4px 0 18px}.lbar .band{position:absolute;top:0;bottom:0;background:rgba(61,220,151,.35);border-radius:5px}.lbar u{position:absolute;top:-4px;bottom:-4px;width:3px;margin-left:-1.5px;background:var(--ink);border-radius:2px}
 .lt{position:absolute;left:0;right:0;top:14px;font:500 9.5px Inter;color:var(--text3)}.lt em{position:absolute;font-style:normal;transform:translateX(-50%)}.lt em:first-child{transform:none}.lt em:last-child{transform:translateX(-100%)}
 .lf{margin-top:8px}.lf span{display:flex;gap:8px;align-items:baseline;font-size:12.5px;color:var(--text2)}.lf b{color:var(--text);margin-left:auto}.lf em{font-style:normal;font-size:11px;color:#FFB547}
 .lfb{position:relative;height:8px;border-radius:4px;background:rgba(255,255,255,.06);margin-top:4px}.lfb .band{position:absolute;top:-2px;bottom:-2px;border:1px dashed rgba(61,220,151,.6);border-radius:4px}.lfb .val{position:absolute;left:0;top:0;bottom:0;background:#5CA8FF;border-radius:4px}
 .vo{display:flex;justify-content:space-between;align-items:flex-end}.vo small{display:block;font-size:11px;color:var(--text3)}.vo b{font-size:30px;font-weight:600;margin-right:8px}.vo em{font-style:normal;font-size:12px;color:var(--text2)}
 .scores{display:grid;grid-template-columns:repeat(auto-fit,minmax(90px,1fr));gap:8px;margin-top:10px}.scores button{background:rgba(255,255,255,.04);border:1px solid var(--line);border-radius:12px;padding:8px 10px;color:var(--text);text-align:left}.scores small{display:block;font-size:10.5px;color:var(--text3)}.scores b{font-size:18px;font-weight:600}
 .races{display:grid;grid-template-columns:repeat(4,minmax(0,1fr));gap:6px}.races span{display:grid;text-align:center;padding:8px 2px;border-radius:12px;background:rgba(255,181,71,.08)}.races small{font-size:10.5px;color:var(--text3)}.races b{font-size:15px;font-weight:600}
 .th{margin-top:10px}.th small{color:var(--text3);font:500 10.5px Inter}
 .st-goal{display:flex;gap:14px;align-items:center}.st-goal small{display:block;font-size:11px;color:var(--text3)}.st-goal b{display:block;font-size:26px;font-weight:600}.st-goal span{font-size:12px;color:var(--text2)}
 .mus{display:grid;grid-template-columns:120px 1fr;gap:14px;align-items:center}.mus ul{list-style:none;margin:0;padding:0;display:grid;gap:3px}.mus li{display:grid;grid-template-columns:1fr auto auto;gap:5px;align-items:baseline;font-size:12.5px;text-transform:capitalize;color:var(--text2)}.mus li b{color:var(--text)}.mus li small{font-size:10px;color:var(--text3)}.mus li.gap span,.mus li.gap b{color:#FFB547}.mus li.more{display:block}.mus li:nth-child(n+7){display:none}
 .lift{border-top:1px solid var(--line);padding:8px 0 4px}.lift:first-of-type{border-top:0}.lift-h{display:flex;align-items:baseline;gap:6px;width:100%;background:none;border:0;padding:4px 0;color:var(--text);text-align:left;min-height:32px}.lift-h b{font:600 14px Inter}.lift-h small{flex:1;font-size:11px;color:var(--text3)}
 .act-h{margin:12px 2px 0}.filt{margin-top:8px}
 @container(max-width:300px){.races{grid-template-columns:repeat(2,1fr)}.mus{grid-template-columns:1fr}}
</style>
