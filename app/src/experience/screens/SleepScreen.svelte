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
 // Canonical Sleep: tonight first (the prototype's plan semantics), then the
 // shipped Sleep detail in full, then timing observations.
 let ctx = $derived(store.ctx);
 let view = $derived(sleepView(ctx));
 let tn = $derived(ctx.tonight);
 let marked = $derived(store.state.winddown[tn.nightOf]);
 let woundDown = $derived(store.state.winddown[addDays(ctx.displayedDay, -1)]);
 let waiting = $derived(ctx.phase === 'waiting');
</script>
{#snippet empty()}{/snippet}
{#snippet head()}<Header title="Sleep" eyebrow={ctx.night && !waiting ? `Night ending ${dayLabel(ctx.displayedDay)}` : waiting ? 'Last night syncing' : 'No recorded night'} back={() => store.back()} backLabel={store.parentLabel}/>{/snippet}



{@render head()}
<button class="jump" onclick={() => document.getElementById('sec-tonight')?.scrollIntoView({behavior: matchMedia('(prefers-reduced-motion: reduce)').matches ? 'auto' : 'smooth', block: 'start'})}><Icon name="moon" size={16}/><span>Tonight · {tn.window ? `window ${tn.window}` : 'asleep by 23:20'} · need {dur(tn.need)}</span><Icon name="chevron-down" size={16}/></button>
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
<section class="card tn-card" id="sec-tonight" aria-labelledby="tn-t">
 <h3 id="tn-t"><span>Tonight · {tn.nightLabel}</span><span class="src">{tn.source}</span></h3>
 <div class="tn">
  <TonightDial now={ctx.clock.time} windDown={tn.windDown} windowStart={tn.window ? tn.windowStart : '23:20'} windowEnd={tn.window ? tn.windowEnd : '23:20'} estimate={!tn.window} wake={tn.wake} lastBed={ctx.night?.bed} lastWake={ctx.night?.wake} need={dur(tn.need)} size={200}/>
  <div class="tn-t">
   <small>{tn.window ? 'Garmin’s window' : 'Asleep by · estimate'}</small><b class="n">{tn.window ?? '23:20'}</b>
   <small>Need</small><b class="n sm">{dur(tn.need)}</b>
   <small>Wind down</small><b class="n sm">{tn.windDown}</b>
  </div>
 </div>
 <p class="small">{tn.estimate ?? tn.reason}</p>
 {#if tn.updated}<p class="small upd"><Icon name="info" size={13}/> {tn.updated}</p>{/if}
 {#if ctx.phase === 'late'}<p class="small">The window has passed. Rest when you’re ready.</p>{/if}
 <div class="btns">
  <button class="secondary" aria-pressed={!!marked} onclick={A.winddown}>{#if marked}<Icon name="check" size={16}/>Wound down · self-reported{:else}I wound down{/if}</button>
  <button class="link" onclick={A.bedtime}>{store.state.notifyBedtime ? 'Wind-down reminder on' : 'Remind me to wind down'}</button>
 </div>
</section>

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
<style>
 .jump{display:grid;grid-template-columns:auto 1fr auto;gap:10px;align-items:center;width:100%;margin:2px 0 6px;padding:10px 14px;border-radius:16px;border:1px solid rgba(92,124,255,.3);background:rgba(92,124,255,.08);color:var(--text);text-align:left;font-size:13px}
 .jump :global(svg:first-child){color:#A9C8FF}
 .tn-card{border-color:rgba(92,124,255,.3);background:linear-gradient(180deg,rgba(92,124,255,.1),var(--card))}
 .src{font-size:10px;padding:3px 8px;border-radius:999px;background:rgba(169,200,255,.14);color:#D8E6FF;letter-spacing:.06em}
 .tn{display:flex;flex-direction:column;gap:12px;align-items:center}
 .tn-t{text-align:center;max-width:100%}
 .tn-t{display:grid;gap:1px}
 .tn-t small{font-size:11px;color:var(--text3);margin-top:6px}
 .tn-t b{font-size:26px;font-weight:600;line-height:1.1}
 .tn-t b.sm{font-size:18px}
 .upd{display:flex;gap:6px;align-items:flex-start}
 .obs{list-style:none;padding:0;margin:10px 0 0;display:grid;gap:6px}
 .obs li{display:grid;grid-template-columns:44px auto 1fr;gap:10px;align-items:baseline;font-size:14px}
 .obs .pl{color:var(--text2);font-size:12px}
</style>
