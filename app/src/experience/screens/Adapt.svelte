<script lang="ts">
 import Header from '../../lib/components/Header.svelte';
 import Icon from '../../lib/components/Icon.svelte';
 import Glyph from '../components/Glyph.svelte';
 import XIcon from '../components/XIcon.svelte';
 import {store} from '../lib/store.svelte';
 import {A} from '../lib/actions';
 import {addDays, dayLabel} from '../lib/dates';
 import {EXCLUDABLE, SPORTS} from '../lib/sports';
 // Circumstances: what to change, never why. Day choices are quick; longer
 // constraints are progressive and last until the person changes them.
 let {route}: {route: string} = $props();
 let ctx = $derived(store.ctx);
 let L = $derived(store.state.limits);
 let review = $state('');
 let rehabName = $state('');
 let note = $state(store.state.limits.note);
 let routines = $derived(ctx.world.schedule.filter(s => s.routine));
 let affected = $derived(ctx.world.schedule.filter(s => L.exclude.includes(s.sport) || (s.routine && L.note)));
</script>
<Header title="Adapt" eyebrow="Your circumstances · this phone" back={() => store.back()} backLabel={store.parentLabel}/>
{#if route === 'adapt'}<p class="small intro">Raring asks what to change, never why. <button class="link inline" onclick={() => store.navigate('help/adapt')}>How this works</button></p>{/if}

<section class="card ad" id="sec-today">
 <div class="ad-h"><Glyph mark="rest" size={40}/><div><h3>Rest today</h3><p class="small">Today and tonight · no exercise prompts</p></div></div>
 {#if ctx.restActive}<p class="on-state"><Icon name="check" size={15}/>On · through tonight</p><button class="secondary" onclick={A.unrest}>Undo rest</button>{:else}<button class="primary" onclick={A.rest}>Rest today</button>{/if}
</section>

<section class="card ad" id="sec-pause">
 <div class="ad-h"><Glyph mark="paused" size={40}/><div><h3>Pause training</h3><p class="small">Illness, recovery or a break · until you resume</p></div></div>
 {#if store.state.pause}
  <p class="on-state"><Icon name="pause" size={15}/>Paused since {dayLabel(store.state.pause.since)}{store.state.pause.review ? ` · review ${dayLabel(store.state.pause.review)}` : ''}</p>
  <p class="small">Commitments kept for when you return.</p>
  <button class="primary" onclick={() => store.openSheet('resume')}>Resume training…</button>
 {:else}
  <label class="field"><span>Review date (optional)</span><select bind:value={review}><option value="">None</option><option value={addDays(ctx.calendarDate, 3)}>{dayLabel(addDays(ctx.calendarDate, 3))}</option><option value={addDays(ctx.calendarDate, 7)}>{dayLabel(addDays(ctx.calendarDate, 7))}</option></select></label>
  <p class="small">A review date reminds you; it doesn’t end the pause.</p>
  <button class="secondary" onclick={() => A.pause(review || null)}>Pause from today</button>
 {/if}
</section>

<section class="card ad" id="sec-limits">
 <div class="ad-h"><span class="ad-i"><XIcon name="adapt" size={20}/></span><div><h3>Avoid activities or movements</h3><p class="small">Excluded activities are enforced in suggestions</p></div></div>
 <div class="chiprow" role="group" aria-label="Activities to avoid">{#each EXCLUDABLE as s}<button class="chipb" aria-pressed={L.exclude.includes(s)} onclick={() => A.exclude(s)}><Glyph sport={s} mark={L.exclude.includes(s) ? 'recorded' : 'plan'} size={20} label={SPORTS[s].name}/>{SPORTS[s].name}</button>{/each}</div>
 <label class="field col"><span>Movements to avoid (your words)</span><textarea rows="2" bind:value={note} onchange={() => A.note(note)} placeholder="For example: no deep knee bends"></textarea></label>
 {#if L.note}<p class="small">Raring can’t read workout contents, so routines it can’t check show “Review needed”.</p>{/if}
 {#each affected as s}<button class="rowlink" onclick={() => store.navigate(`scheduled/${s.id}`)}><Glyph sport={s.sport} mark="garmin" size={28}/><span><b>{s.name}</b><small>{dayLabel(s.date)} · {L.exclude.includes(s.sport) ? 'Conflict' : L.routines[s.id] === 'chosen' ? 'Your choice' : 'Review needed'}</small></span><Icon name="chevron" size={16}/></button>{/each}
 {#if L.exclude.length || L.note}<button class="link" onclick={() => { A.clearLimits(); note = ''; }}>Remove all limits</button>{/if}
</section>

<section class="card ad" id="sec-routines">
 <div class="ad-h"><span class="ad-i"><Icon name="check" size={20}/></span><div><h3>My routines</h3><p class="small">A checklist you or your clinician supply</p></div></div>
 {#each routines as s}<div class="rowlink static"><Glyph sport={s.sport} mark="garmin" size={28}/><span><b>{s.name}</b><small>Garmin workout · Contents unknown to Raring</small></span><span class="st" class:warn={!!L.note}>{L.note ? (L.routines[s.id] === 'chosen' ? 'Your choice' : 'Review needed') : 'No limits set'}</span></div>{/each}
 {#each store.state.rehab.items as it}
  {@const done = store.state.rehab.done[`${ctx.calendarDate}:${it.id}`]}
  <div class="rowlink static"><Glyph sport="yoga" mark={done ? 'recorded' : 'self'} size={28} label="Self-reported"/><span><b>{it.name}</b>{#if it.how}<small>{it.how}</small>{/if}</span><button class="secondary small-btn" aria-pressed={!!done} onclick={() => A.rehabDone(it.id)}>{done ? 'Done today · Self-reported' : 'Mark done today'}</button></div>
 {/each}
 <div class="addrow"><label class="field col"><span>Exercise name</span><input bind:value={rehabName} placeholder="For example: wall sit, 3 × 30 s"/></label><button class="secondary" onclick={() => { A.rehabAdd(rehabName); rehabName = ''; }}>Add to checklist</button></div>
 <p class="small">Self-reported · no Garmin activity, intensity minutes or records are created.</p>
</section>

<section class="card ad" id="sec-lighter">
 <div class="ad-h"><span class="ad-i"><Icon name="clock" size={20}/></span><div><h3>Busy or travelling</h3><p class="small">A lighter week, no reason needed</p></div></div>
 {#if ctx.plan}<p class="small">Original targets stay on record; nothing is prorated.</p><div class="btns"><button class="secondary" onclick={A.lighter}>Lighter week: 1 strength · 60 min</button><button class="secondary" onclick={() => A.pause()}>Pause this week</button></div>
 {:else}<p class="small">No commitments to lighten.</p>{/if}
</section>
<style>
 .intro{margin:2px 4px 6px}
 .ad h3{margin:0;color:var(--text);font:600 17px Outfit;letter-spacing:0;text-transform:none;min-height:0}
 .ad-h{display:flex;gap:12px;align-items:center;margin-bottom:6px}
 .ad-i{width:40px;height:40px;border-radius:50%;display:grid;place-items:center;background:rgba(255,181,71,.12);color:#FFB547;flex:none}
 .on-state{display:flex;gap:6px;align-items:center;color:#A9C8FF;font-weight:600;margin-top:8px}
 .static{cursor:default}
 .st{font:600 11px Inter;padding:4px 8px;border-radius:999px;background:rgba(255,255,255,.06);color:var(--text2)}
 .st.warn{background:rgba(255,181,71,.14);color:#FFB547}
 .small-btn{width:auto;margin:0;padding:8px 12px;font-size:12px}
 .addrow{display:grid;gap:4px}
</style>
