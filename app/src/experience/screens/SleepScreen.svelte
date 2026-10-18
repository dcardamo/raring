<script lang="ts">
 import Header from '../../lib/components/Header.svelte';
 import Icon from '../../lib/components/Icon.svelte';
 import SleepDetail from '../../lib/screens/SleepDetail.svelte';
 import TonightDial from '../components/TonightDial.svelte';
 import {store} from '../lib/store.svelte';
 import {A} from '../lib/actions';
 import {addDays, dayLabel, dur} from '../lib/dates';
 import {sleepView} from '../lib/views';
 import {clock} from '../../lib/format';
 import AxisChart from '../components/AxisChart.svelte';
 import HelpButton from '../components/HelpButton.svelte';
 import {goalsFor} from '../lib/allocation';
 import {weekRings, sleepMet} from '../lib/week';
 import {hm, toMin} from '../lib/dates';
 // Canonical Sleep: tonight first (the prototype's plan semantics), then the
 // shipped Sleep detail in full, then timing observations.
 let ctx = $derived(store.ctx);
 let view = $derived(sleepView(ctx));
 let tn = $derived(ctx.tonight);
 let marked = $derived(store.state.winddown[tn.nightOf]);
 let woundDown = $derived(store.state.winddown[addDays(ctx.displayedDay, -1)]);
 let waiting = $derived(ctx.phase === 'waiting');
 // Two tabs; a plain #sleep opens whichever is closer in time.
 let sub = $derived(store.route.split('/')[1]);
 let tab = $derived(sub === 'tonight' ? 'tonight' : sub ? 'last' : ['evening', 'winddown', 'late'].includes(ctx.phase) ? 'tonight' : 'last');
 let g = $derived(goalsFor(ctx));
 let wk = $derived(weekRings(ctx));
 let bedtime = $derived(tn.window ? tn.windowEnd : hm((toMin(tn.wake) - tn.need + 1440) % 1440));
 let placed = $derived(store.state.intentions.some(i => i.kind === 'sleep' && i.date === tn.nightOf && (!i.status || i.status === 'open')));
 let nights = $derived(Array.from({length: 7}, (_, i) => addDays(ctx.displayedDay, i - 6)).map(d => ({d, n: ctx.nightOf(d)})));
 let avgNeed = $derived(Math.round(nights.filter(x => x.n).reduce((t, x) => t + x.n!.need, 0) / Math.max(1, nights.filter(x => x.n).length)));
 const fmtH = (m: number) => `${Math.floor(m / 60)}:${String(Math.round(m % 60)).padStart(2, '0')}`;
</script>
{#snippet empty()}{/snippet}
{#snippet head()}<Header title="Sleep" eyebrow={ctx.night && !waiting ? `Night ending ${dayLabel(ctx.displayedDay)}` : waiting ? 'Last night syncing' : 'No recorded night'} back={() => store.back()} backLabel={store.parentLabel}/>{/snippet}



{@render head()}
<div class="seg" role="tablist" aria-label="Sleep"><button role="tab" aria-selected={tab === 'last'} onclick={() => store.replace('sleep/last')}>Last night</button><button role="tab" aria-selected={tab === 'tonight'} onclick={() => store.replace('sleep/tonight')}>Tonight</button></div>
{#if tab === 'last'}
<div class="tab-h"><span class="small">{ctx.night && !waiting ? `Night ending ${dayLabel(ctx.displayedDay)}` : ''}</span><HelpButton topic="sleep-last"/></div>
{#if woundDown && ctx.night}<p class="selfrep"><Icon name="check" size={15}/>Wound down · self-reported, the evening before this night</p>{/if}
{#if waiting}<section class="card"><p>Last night hasn’t synced yet.</p><button class="link" onclick={() => store.navigate('help/sync')}>How syncing works</button></section>
{:else if !ctx.night}<section class="card"><p>No sleep recorded for this night.</p><p class="small">A gap in the record.</p></section>{/if}

{#if ctx.night && !waiting}<SleepDetail sleepData={view} header={empty}/>
{:else}<section class="card naps"><h3>Recorded naps</h3>
 {#if view.napsState === 'stale'}<p class="small">Naps may be out of date.</p>{/if}
 {#if view.napsState === 'unavailable' || view.napsState === 'unknown'}<p class="small">Nap data unavailable.</p>
 {:else if !view.naps.length}<p class="small">{view.napsState === 'stale' ? 'No cached naps.' : `No naps recorded for ${dayLabel(ctx.calendarDate)}.`}</p>
 {:else}{#each view.naps as nap}<div class="nap-row"><span>{nap.startLocal ? `${dayLabel(nap.startLocal.slice(0,10))} ${clock(nap.startLocal)}` : 'Start date/time unavailable'} – {nap.endLocal ? `${dayLabel(nap.endLocal.slice(0,10))} ${clock(nap.endLocal)}` : 'End date/time unavailable'}</span><b class="n">{dur(nap.durationMin)}</b></div>{/each}{/if}
 </section>{/if}
<section class="card" id="sec-patterns">
 <button class="rowlink" onclick={() => store.navigate('sleep/patterns')}><Icon name="moon" size={18}/><span><b>Patterns</b><small>Duration against need, timing, alignment · 14 nights</small></span><Icon name="chevron" size={16}/></button>
 <button class="rowlink" onclick={() => store.navigate('metric/sleepScore')}><Icon name="trend-up" size={18}/><span><b>Sleep score trend</b><small>Ranges and comparisons</small></span><Icon name="chevron" size={16}/></button>
</section>
{#if store.state.winddownIntention}<section class="card"><h3>Your wind-down intention</h3><p class="small">Chosen by you. Marking it done is a self-report, not inferred from sleep.</p></section>{/if}
{#if store.state.savedSleepTarget}
 <section class="card" id="sec-timing"><h3>Sleep timing · observation</h3>
  <p class="small">Asleep times against the plan saved each evening (reference: 5 of 7 nights).</p>
  <ul class="obs">
   {#each [['Mon', '22:56', '23:15'], ['Tue', '23:18', '23:15'], ['Wed', '00:52', '23:10'], ['Thu', null, null]] as [d, a, p]}<li><b>{d}</b>{#if a}<span class="n">{a}</span><span class="pl">plan {p}</span>{:else}<span class="pl">No saved plan that evening</span>{/if}</li>{/each}
  </ul>
  <p class="small">Week 46 keeps its original result: 4 of 5 nights. This week is marked adjusted.</p>
 </section>
{/if}
{:else}
<section class="card tn-card" id="sec-tonight" aria-labelledby="tn-t">
 <h3 id="tn-t"><span>Tonight · {tn.nightLabel} · {tn.source}</span><HelpButton topic="sleep-tonight"/></h3>
 <div class="tn">
  <TonightDial now={ctx.clock.time} windDown={tn.windDown} windowStart={tn.window ? tn.windowStart : bedtime} windowEnd={tn.window ? tn.windowEnd : bedtime} estimate={!tn.window} wake={tn.wake} lastBed={ctx.night?.bed} lastWake={ctx.night?.wake} need={dur(tn.need)} size={124}/>
  <dl class="plan">
   <dt><i style:background="#FFB547"></i>Wind down</dt><dd class="n">{g.sleep > 0 && g.sleepKind === 'winddown' ? g.sleepTime : tn.windDown}<small>{g.sleep > 0 && g.sleepKind === 'winddown' ? 'your goal' : 'suggested'}</small></dd>
   <dt><i style:background="#5C9BFF"></i>Asleep by</dt><dd class="n">{g.sleep > 0 && g.sleepKind === 'bedtime' ? g.sleepTime : bedtime}<small>{g.sleep > 0 && g.sleepKind === 'bedtime' ? 'your goal' : tn.window ? 'Garmin window' : 'need before wake'}</small></dd>
   <dt><i style:background="#EEF1F7"></i>Wake</dt><dd class="n">{tn.wake}<small>usual</small></dd>
   <dt>Need</dt><dd class="n">{dur(tn.need)}<small>Garmin</small></dd>
  </dl>
 </div>
 <p class="small">{tn.estimate ?? tn.reason}</p>
 {#if tn.updated}<p class="small upd"><Icon name="info" size={13}/> {tn.updated}</p>{/if}
 {#if ctx.phase === 'late'}<p class="small">The window has passed. Rest when you’re ready.</p>{/if}
 <div class="btns">
  <button class="secondary" aria-pressed={!!marked} onclick={A.winddown}>{#if marked}<Icon name="check" size={16}/>Wound down · self-reported{:else}I wound down{/if}</button>
  <button class="link" onclick={A.bedtime}>{store.state.notifyBedtime ? 'Wind-down reminder on' : 'Remind me to wind down'}</button>
 </div>
</section>
<section class="card"><h3>Your sleep goals this week</h3>
 {#if g.sleep > 0}{@const r = wk.rings.find(x => x.key === 'sleep')}<p class="goal-l"><b>{g.sleepKind === 'winddown' ? 'Wind down at' : 'Asleep by'} {g.sleepTime}</b> · {r?.recorded ?? 0} of {g.sleep} nights{#if placed} · tonight is one{/if}</p>
  {#if !placed}<button class="secondary" onclick={() => A.allocate({sport: 'other', date: tn.nightOf, time: g.sleepTime, kind: 'sleep', name: `${g.sleepKind === 'winddown' ? 'Wind down' : 'Asleep by'} ${g.sleepTime}`, allocation: {sleep: 1}}, false)}>Make tonight one of them</button>{/if}
 {/if}
 {#if wk.score}<p class="goal-l"><b>Average Sleep Score {wk.score.average ?? '—'}</b> · target {wk.score.target} · Garmin reading over {wk.score.nights} night{wk.score.nights === 1 ? '' : 's'}</p>{/if}
 {#if !g.sleep && !wk.score}<p class="small">No sleep goal. <button class="link inline" onclick={() => store.openSheet('goals')}>Choose one</button></p>{/if}
</section>
<section class="card"><h3>Last 7 nights against need</h3>
 <AxisChart points={nights.map(x => ({x: dayLabel(x.d).slice(0, 3), v: x.n?.asleep ?? null}))} unit="h" goal={avgNeed} goalLabel="avg need" fmt={fmtH} color="#5C9BFF" height={140} label={`Hours asleep, last 7 nights: ${nights.map(x => x.n ? fmtH(x.n.asleep) : 'none').join(', ')}; average need ${fmtH(avgNeed)}`}/>
 <p class="small">{nights.filter(x => x.n && x.n.asleep < x.n.need).length} of {nights.filter(x => x.n).length} nights below Garmin’s need. No running debt is computed.</p>
</section>
{/if}
<style>
 .seg{display:grid;grid-template-columns:1fr 1fr;gap:4px;padding:4px;border-radius:14px;background:rgba(255,255,255,.05);margin:6px 0 8px}.seg button{height:38px;border-radius:10px;border:0;background:none;color:var(--text2);font:600 13px Inter}.seg button[aria-selected="true"]{background:var(--ink);color:var(--bg)}
 .tab-h{display:flex;justify-content:space-between;align-items:center;margin:0 2px}
 .tn{display:grid;grid-template-columns:124px 1fr;gap:12px;align-items:center}.tn :global(.dial text){display:none}
 .plan{display:grid;grid-template-columns:auto 1fr;gap:4px 10px;margin:0;align-items:center}.plan dt{font-size:12px;color:var(--text2);white-space:nowrap;display:flex;gap:6px;align-items:center}.plan dt i{width:8px;height:8px;border-radius:50%}.plan dd{margin:0;font-size:19px;font-weight:600;line-height:1.1}.plan small{display:block;font:500 10px Inter;color:var(--text3)}
 .goal-l{font-size:13px;color:var(--text2);margin:4px 0 8px}.goal-l b{color:var(--text)}
 @container(max-width:300px){.tn{grid-template-columns:1fr;justify-items:center}}
 :global(.xp .stage-legend){grid-template-columns:repeat(auto-fill,minmax(140px,1fr))!important}
 .tn-card{border-color:rgba(92,124,255,.3);background:linear-gradient(180deg,rgba(92,124,255,.1),var(--card))}
 .upd{display:flex;gap:6px;align-items:flex-start}
 .obs{list-style:none;padding:0;margin:10px 0 0;display:grid;gap:6px}
 .obs li{display:grid;grid-template-columns:44px auto 1fr;gap:10px;align-items:baseline;font-size:14px}
 .obs .pl{color:var(--text2);font-size:12px}
</style>
