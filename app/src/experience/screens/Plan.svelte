<script lang="ts">
 import Icon from '../../lib/components/Icon.svelte';
 import RootHeader from '../components/RootHeader.svelte';
 import WeekStrip, {marksFor, isPaused} from '../components/WeekStrip.svelte';
 import Glyph from '../components/Glyph.svelte';
 import Ring from '../components/Ring.svelte';
 import XIcon from '../components/XIcon.svelte';
 import {store} from '../lib/store.svelte';
 import {A} from '../lib/actions';
 import {addDays, dayLabel, daysBetween, shortDay} from '../lib/dates';
 import {SPORTS} from '../lib/sports';
 import {contribution, versionFor} from '../lib/rules';
 import {activityById} from '../lib/world';
 let ctx = $derived(store.ctx);
 const go = (r: string) => store.navigate(r);
 let sel = $derived((store.ui.day as string) ?? ctx.calendarDate);
 let agenda = $derived(marksFor(ctx, sel));
 let ambitions = $derived([...ctx.world.ambitions, ...store.state.ambitionsAdded.map(a => ({...a, sport: null}))]);
 let earlier = $derived(store.state.intentions.filter(i => i.date < ctx.weekFrom || (i.date < ctx.calendarDate && !i.link)));
 let coming = $derived(Array.from({length: 7}, (_, i) => addDays(ctx.calendarDate, i + 1)).flatMap(d => marksFor(ctx, d).filter(m => m.mark !== 'recorded').map(m => ({...m, d}))).slice(0, 5));
 let versions = $derived(versionFor(ctx));
 const conflict = (sport: string) => store.state.limits.exclude.includes(sport as never);
 const sub = (m: ReturnType<typeof marksFor>[number], d: string) => {
  if (m.mark === 'recorded') { const a = activityById(m.go!.slice(8)); return a ? contribution(ctx, a).short : 'Recorded'; }
  if (m.mark === 'garmin') { const s = ctx.world.schedule.find(x => `scheduled/${x.id}` === m.go); return conflict(m.sport) ? 'Garmin · conflicts with your limits' : s?.routine && store.state.limits.note ? 'Garmin · review needed' : `Garmin schedule${ctx.paused ? ' · paused for you' : ''}`; }
  if (m.mark === 'plan') { const i = store.state.intentions.find(x => `intention/${x.id}` === m.go); return i?.link ? 'Your plan · linked by you' : d < ctx.calendarDate ? 'Your plan · not linked' : `Your plan${i?.time ? ' · ' + i.time : ''}`; }
  return 'No exercise prompts';
 };
</script>

<RootHeader title="Plan" adapt/>

{#if store.state.savedSleepTarget && !store.state.seen.timingMigration}
 <section class="card mig"><span class="mig-i"><Icon name="moon" size={18}/></span><div><b>Sleep timing is now an observation</b><p class="small">Nights show against your saved plan, without verdicts. Earlier weeks keep their results.</p><div class="btns"><button class="secondary" onclick={() => A.seenMigration()}>Got it</button><button class="link" onclick={() => A.seenMigration('winddown')}>Add a wind-down intention</button></div></div></section>
{/if}
{#if ctx.paused || ctx.restActive || ctx.hasLimits}
 <button class="circ" onclick={() => go('adapt')}>{#if ctx.paused}<Glyph mark="paused" size={30}/>{:else if ctx.restActive}<Glyph mark="rest" size={30}/>{:else}<span class="lim"><XIcon name="adapt" size={18}/></span>{/if}<span><b>{ctx.paused ? `Paused since ${dayLabel(ctx.paused.since)}` : ctx.restActive ? 'Rest day · through tonight' : 'Your limits'}</b><small>{ctx.paused ? 'Commitments kept for when you resume' : ctx.restActive ? 'Plan and data unchanged' : [store.state.limits.exclude.map(x => 'no ' + SPORTS[x].name.toLowerCase()).join(', '), store.state.limits.note && `“${store.state.limits.note}”`].filter(Boolean).join(' · ')}</small></span><Icon name="chevron" size={16}/></button>
{/if}

<section class="card week" id="sec-week">
 <div class="wk-h"><h3>This week</h3><button class="link quiet" onclick={() => store.openSheet('legend')}>Key</button></div>
 <WeekStrip {ctx} selected={sel} onSelect={d => store.setUi({day: d})}/>
 <div class="agenda" aria-live="polite">
  <h4>{sel === ctx.calendarDate ? 'Today' : dayLabel(sel)}{isPaused(ctx, sel) ? ' · paused' : ''}</h4>
  {#each agenda as m}
   <button class="ag" style:--c={m.mark === 'rest' ? '#A9C8FF' : SPORTS[m.sport].color} class:warn={m.mark === 'garmin' && conflict(m.sport)} disabled={!m.go} onclick={() => m.go && go(m.go)}>
    <Glyph sport={m.sport} mark={m.mark} size={32} label={m.label}/><span><b>{m.label.replace(/^(Recorded|Garmin|Your plan): ?/, '')}</b><small>{sub(m, sel)}</small></span>{#if m.go}<Icon name="chevron" size={16}/>{/if}
   </button>
  {:else}<p class="small none">Nothing planned{sel < ctx.calendarDate ? ' or recorded' : ''}.</p>{/each}
 </div>
 <button class="primary" onclick={() => store.openSheet('add')}>Add to my plan</button>
</section>

<div class="sec-h"><h2>Commitments</h2>{#if ctx.plan}<button class="link" onclick={() => go('plan/commitments')}>Edit</button>{/if}</div>
<section class="card" id="sec-commitments">
 {#if ctx.plan}
  <div class="rings">
   {#each ctx.plan.tracks as t}
    <div class="cring">
     <Ring value={t.have} target={t.target} size={84} stroke={8} state={t.state} color={t.key === 'strength' ? SPORTS.strength.color : '#36C9C6'} label={`${t.key === 'strength' ? 'Strength sessions' : 'Intensity minutes'}: ${t.label}`}>{#if t.state !== 'reached' && t.state !== 'reached-adjusted'}<b class="n rv">{t.have}<small>/{t.target}</small></b>{/if}</Ring>
     <b>{t.key === 'strength' ? 'Strength sessions' : 'Intensity minutes'}</b>
     <small class="st s-{t.state}">{t.label}</small>
     <small>{t.state === 'open' ? `${ctx.plan.daysLeft} days left` : t.detail}</small>
    </div>
   {/each}
  </div>
  <div class="vers" aria-label="Commitment versions">
   {#each versions.versions.filter(v => v.from <= ctx.plan!.wk) as v}<span class="ver" class:cur={v === versions.base}>{v.from.replace(/\d{4}-W/, 'W')} · {v.strength} strength · {v.im} min</span>{/each}
   {#each ctx.plan.revisions as r}<span class="ver cur">{shortDay(r.from)} · {r.strength} strength · {r.im} min</span>{/each}
   {#if ctx.plan.pause}<span class="ver hatch">Paused {shortDay(ctx.plan.pause.since)}{ctx.plan.pause.until ? '–' + shortDay(ctx.plan.pause.until) : ''}</span>{/if}
  </div>
  <p class="small">Plan week {dayLabel(ctx.weekFrom)}–{dayLabel(ctx.weekTo)} · Garmin’s intensity week</p>
 {:else}
  <div class="nocom"><Ring value={0} target={1} size={46} stroke={5} center={false} label="No commitments"/><div><b>No commitments</b><small>Garmin’s weekly intensity goal: 150 min</small></div></div>
  <button class="secondary" onclick={() => go('plan/commitments')}>Add a commitment</button>
 {/if}
</section>

<div class="sec-h"><h2>Ambitions</h2><button class="link" onclick={() => store.openSheet('ambition')}>Add</button></div>
{#if ambitions.length}
 <div class="amb-grid" id="sec-ambitions">
  {#each ambitions as a}
   {@const days = a.date ? daysBetween(ctx.calendarDate, a.date) : null}
   <div class="amb" style:--c={a.sport ? SPORTS[a.sport].color : '#A48BFF'}>
    {#if days != null}<Ring value={Math.max(0, 60 - days)} target={60} size={60} stroke={5} color={a.sport ? SPORTS[a.sport].color : '#A48BFF'} label={`${days} days to ${a.name}`}><b class="n dd">{days}<small>d</small></b></Ring>
    {:else}<span class="amb-i">{#if a.sport}<Glyph sport={a.sport} mark="plan" size={40}/>{:else}<XIcon name="flag" size={22}/>{/if}</span>{/if}
    <b>{a.name}</b><small>{a.date ? `${dayLabel(a.date)} · ${days} days` : 'No date'}</small><em>{a.source}</em>
   </div>
  {/each}
 </div>
{:else}<p class="small amb-empty">Optional. Events and directions can sit side by side.</p>{/if}
{#if ctx.world.schedule.length}<p class="small cov">{store.state.calendar === 'complete' ? 'Garmin schedule checked 07:12 · covers 18 Nov–1 Dec' : store.state.calendar === 'partial' ? 'Garmin returned a full page of events; later items may be missing' : 'Garmin schedule last checked Mon 16 Nov; may be out of date'}</p>{/if}

{#if coming.length}
 <div class="sec-h"><h2>Coming up</h2></div>
 <section class="card">
  {#each coming as m}<button class="rowlink" onclick={() => m.go && go(m.go)}><Glyph sport={m.sport} mark={m.mark} size={30} label={m.label}/><span><b>{m.label.replace(/^(Garmin|Your plan): ?/, '')}</b><small>{dayLabel(m.d)} · {sub(m, m.d)}</small></span><Icon name="chevron" size={16}/></button>{/each}
 </section>
{/if}

{#if earlier.length}
 <div class="sec-h"><h2>Earlier intentions</h2></div>
 <section class="card">{#each earlier as i}<button class="rowlink" onclick={() => go(`intention/${i.id}`)}><Glyph sport={i.sport} mark="plan" size={28}/><span><b>{SPORTS[i.sport].name} · {dayLabel(i.date)}</b><small>{i.link ? 'Linked by you' : 'Not linked'}</small></span><Icon name="chevron" size={16}/></button>{/each}</section>
{/if}

{#if store.state.rehab.items.length || ctx.hasLimits}
 <button class="circ" onclick={() => go('adapt/routines')}><span class="lim"><Icon name="check" size={18}/></span><span><b>My routines</b><small>{store.state.rehab.items.length ? `${store.state.rehab.items.length} in your checklist · self-reported` : 'Garmin routines need your review'}</small></span><Icon name="chevron" size={16}/></button>
{/if}

<style>
 .mig{display:flex;gap:12px;border-color:rgba(169,200,255,.3);background:rgba(169,200,255,.06)}
 .mig-i{width:34px;height:34px;flex:none;border-radius:50%;display:grid;place-items:center;background:rgba(169,200,255,.16);color:#A9C8FF}
 .mig b{display:block}
 .mig .btns{align-items:center}
 .circ{display:grid;grid-template-columns:auto 1fr auto;gap:12px;align-items:center;width:100%;margin-top:12px;padding:12px 14px;border-radius:18px;background:rgba(169,200,255,.07);border:1px solid rgba(169,200,255,.25);color:var(--text);text-align:left}
 .circ b{display:block}.circ small{display:block;font-size:12px;color:var(--text2)}
 .lim{width:30px;height:30px;border-radius:50%;display:grid;place-items:center;background:rgba(255,181,71,.14);color:#FFB547}
 .week{padding:14px 10px}
 .wk-h{display:flex;justify-content:space-between;align-items:center;padding:0 6px;margin-bottom:6px}
 .wk-h h3{margin:0}
 .agenda{margin:10px 4px 0;padding-top:12px;border-top:1px solid var(--line)}
 .agenda h4{margin:0 0 6px;font:600 15px Outfit}
 .ag{display:grid;grid-template-columns:auto 1fr auto;gap:12px;align-items:center;width:100%;margin-top:8px;padding:10px 12px 10px 10px;border-radius:16px;background:rgba(255,255,255,.04);border:1px solid var(--line);border-left:4px solid var(--c);color:var(--text);text-align:left}
 .ag:disabled{opacity:1;cursor:default}
 .ag.warn{border-color:rgba(255,181,71,.45);border-left-color:#FFB547}
 .ag b{display:block;font-weight:600}.ag small{display:block;font-size:12px;color:var(--text2)}
 .none{margin:8px 0 0}
 .week .primary{margin:14px 4px 0;width:calc(100% - 8px)}
 .rings{display:grid;grid-template-columns:1fr 1fr;gap:12px}
 .cring{display:flex;flex-direction:column;align-items:center;text-align:center;gap:3px}
 .cring>b{margin-top:8px;font-weight:600;font-size:14px}
 .cring small{font-size:12px;color:var(--text2)}
 .st{font-weight:600}
 .s-reached,.s-reached-adjusted{color:var(--good)!important}.s-unknown,.s-paused,.s-adjusted-paused{color:var(--text3)!important}
 .rv{font-size:20px}.rv small{font-size:12px;color:var(--text3)}
 .vers{display:flex;flex-wrap:wrap;gap:6px;margin-top:16px}
 .ver{font:600 11px Inter;padding:5px 9px;border-radius:999px;background:rgba(255,255,255,.04);color:var(--text3);border:1px solid var(--line)}
 .ver.cur{color:var(--text);border-color:rgba(255,255,255,.2)}
 .ver.hatch{background:repeating-linear-gradient(45deg,rgba(169,200,255,.14) 0 3px,transparent 3px 7px);color:#D8E6FF}
 .nocom{display:flex;gap:12px;align-items:center}
 .nocom b{display:block}.nocom small{display:block;color:var(--text2);font-size:12px}
 .amb-grid{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:10px;margin-top:10px}
 .amb{display:flex;flex-direction:column;align-items:flex-start;gap:3px;padding:14px;border-radius:20px;background:linear-gradient(160deg,color-mix(in srgb,var(--c) 14%,transparent),var(--card) 60%);border:1px solid var(--line)}
 .amb>b{margin-top:8px;font:600 16px Outfit}
 .amb small{font-size:12px;color:var(--text2)}
 .amb em{font-style:normal;font-size:11px;color:var(--text3)}
 .amb-i{height:60px;display:grid;place-items:center;color:#C9BEFF}
 .dd{font-size:18px}.dd small{font-size:11px;color:var(--text3)}
 .amb-empty,.cov{margin:8px 4px 0}
</style>
