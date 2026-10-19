<script lang="ts">
 import Icon from '../../lib/components/Icon.svelte';
 import RootHeader from '../components/RootHeader.svelte';
 import DayCells from '../components/DayCells.svelte';
 import WeekSummary from '../components/WeekSummary.svelte';
 import Backlog from '../components/Backlog.svelte';
 import Glyph from '../components/Glyph.svelte';
 import ItemRow from '../components/ItemRow.svelte';
 import {planItems, itemMatch, localItem} from '../lib/match';
 import {eventWeek} from '../lib/week';
 import {trainingView} from '../lib/views';
 import {store} from '../lib/store.svelte';
 import {A} from '../lib/actions';
 import {isOpen,intentionState,candidates,allocationFor,allocations,goalChanged,GOAL_INFO} from '../lib/allocation';
 import {SPORTS} from '../lib/sports';
 import {COACH} from '../lib/world';
 import {dayLabel,weekRange,daysBetween,shortDay} from '../lib/dates';
 // Plan: the week at a glance (rings + one line per goal), days to place
 // sessions on with Garmin's items inline, what's left, and longer goals.
 let ctx=$derived(store.ctx), selected=$derived((store.ui.day as string)??ctx.calendarDate);
 let recorded=$derived(ctx.world.activities.filter(a=>a.date===selected));
 let dayItems=$derived(planItems(ctx,selected,selected).filter(p=>itemMatch(ctx,p).state!=='released'));
 let unmatched=$derived.by(()=>{const used=new Set(dayItems.map(p=>itemMatch(ctx,p).activity?.id).filter(Boolean));return recorded.filter(a=>!used.has(a.id));});
 let evw=$derived(eventWeek(ctx));
 let pred=$derived.by(()=>{const r=trainingView(ctx).races.find(x=>x.label==='10K');return r&&evw&&/10k/i.test(evw.ev.name)?`${Math.floor(r.seconds/60)}:${String(r.seconds%60).padStart(2,'0')}`:null;});
 let unfinished=$derived(ctx.state.intentions.filter(i=>['review','not-recorded','awaiting'].includes(intentionState(ctx,i))));
 let reviews=$derived(allocations(ctx).flatMap(r=>r.review));
 let changed=$derived(goalChanged(ctx));
 let coachWeek=$derived(ctx.world.schedule.filter(s=>s.date>=ctx.weekFrom&&s.date<=ctx.weekTo).length);
 let ambitions=$derived([...ctx.world.ambitions,...ctx.state.ambitionsAdded]);
 const labels:Record<string,string>={strength:'strength',zone2:'Zone 2 min',hi:'Zone 4–5 min',im:'intensity min',sleep:'night'};
 // Pause intervals touching this week stay visible after an explicit resume.
 let weekPauses=$derived(ctx.state.pauses.filter(p=>p.since<=ctx.weekTo&&(!p.until||p.until>=ctx.weekFrom)&&p.until));
 const go=(r:string)=>store.navigate(r);
</script>
<RootHeader title="Plan" help="plan"/>
<section class="card wk">
 <div class="wk-h"><div><h2>{weekRange(ctx.weekFrom)}</h2>{#if changed}<small>Targets changed {shortDay(changed)}</small>{/if}</div><span class="wk-b"><button class="small-edit" onclick={()=>store.openSheet('planweek')}>Plan week</button><button class="small-edit" onclick={()=>store.openSheet('goals')}><Icon name="pencil" size={13}/>Goals</button></span></div>
 {#if ctx.paused||ctx.restActive||ctx.hasLimits}<button class="adapt-state" onclick={()=>go('adapt')}><Icon name={ctx.paused?'pause':ctx.restActive?'moon':'info'} size={15}/>{ctx.paused?`${ctx.paused.reason==='ill'?'Ill':'Training paused'} since ${shortDay(ctx.paused.since)} · rings hatched, no catch-up`:ctx.restActive?'Rest day · through tonight':'Your limits apply'}<Icon name="chevron" size={14}/></button>{/if}
 {#each weekPauses as p}<p class="pause-note"><Icon name="pause" size={13}/>{p.reason==='ill'?'Ill':'Paused'} {shortDay(p.since)}–{shortDay(p.until!)}</p>{/each}
 <WeekSummary/>
 {#each reviews as i}<button class="review" onclick={()=>go(`intention/${i.id}`)}><span><b>Needs review</b> · {i.name??SPORTS[i.sport].name}, {dayLabel(i.date)} may match a recording</span><Icon name="chevron" size={14}/></button>{/each}
 {#if ctx.world.schedule.length}<button class="coach-line" aria-label={`Garmin Coach: ${COACH.name}, read-only`} onclick={()=>go('plan/garmin')}><span class="gmark" aria-hidden="true">◇</span><span><b>Garmin Coach · {COACH.name}</b><small>{ctx.state.calendar==='partial'?'Garmin returned a full page; later sessions may be missing':ctx.state.calendar==='stale'?'Last checked Mon 16 Nov · may be out of date':`${coachWeek} Garmin session${coachWeek===1?'':'s'} this week · shown in your days`}</small></span><Icon name="chevron" size={14}/></button>{/if}
 {#if evw && evw.items.length}
  <div class="evb" style:--ev="#FFB547">
   <button class="ev-h" onclick={()=>go(`plan/race/${evw.ev.id}`)} aria-label={`${evw.ev.name}, ${evw.days} days, ${dayLabel(evw.ev.date!)}`}><span class="cd"><b class="n">{evw.days}</b><small>days</small></span><span><b>{evw.ev.name}</b><small>{dayLabel(evw.ev.date!)} · {SPORTS[evw.sport].plural} this week: {evw.recordedMin} min recorded{evw.scheduledMin ? ` · Garmin Coach scheduled ${evw.scheduledMin} min` : ''}{pred ? ` · Garmin 10K prediction ${pred}` : ''}</small></span><Icon name="chevron" size={14}/></button>
   {#each evw.items as p (p.id)}<ItemRow {p} showDay event={evw.ev.name} actions={false}/>{/each}
  </div>
 {:else}{#each ambitions.filter(a=>a.date) as a}<button class="race-line" aria-label={`${a.name}, ${daysBetween(ctx.calendarDate,a.date!)} days, ${dayLabel(a.date!)}, ${a.source}`} onclick={()=>go(`plan/race/${a.id}`)}><span class="cd"><b class="n">{daysBetween(ctx.calendarDate,a.date!)}</b><small>days</small></span><span><b>{a.name}</b><small>{dayLabel(a.date!)} · {a.source}</small></span><Icon name="chevron" size={14}/></button>{/each}{/if}
</section>

<div class="sec-h"><h2>Days</h2><button class="link" onclick={()=>store.openSheet('legend')}>Key</button></div>
<section class="card schedule-week"><DayCells selected={selected} select={d=>store.setUi({day:d})}/>
 <div class="agenda-h"><h3>{selected===ctx.calendarDate?'Today':dayLabel(selected)}</h3><button class="link" onclick={()=>store.openSheet('day',selected)}>Planned, recorded, left</button></div>
 {#each dayItems as p (p.id)}<ItemRow {p} event={evw && evw.items.some(x => x.id === p.id) ? evw.ev.name : null}/>{/each}
 {#each unmatched as a}<button class="agenda-row done" onclick={()=>go(`workout/${a.id}`)}><Glyph sport={a.sport} size={30}/><span><small>✓ Recorded · not in your plan</small><b>{a.name}</b><em>{a.start} · {a.min} min{a.km?' · '+a.km+' km':''}</em></span><Icon name="chevron" size={15}/></button>{/each}
 {#if !dayItems.length&&!unmatched.length}<div class="empty-day"><Icon name={ctx.state.rest?.day===selected?'moon':'clock'} size={22}/><p>{ctx.state.rest?.day===selected?'A day of rest':'Nothing on this day'}</p></div>{/if}
 <button class="secondary" onclick={()=>store.openSheet('allocate')}>Add a session</button>
</section>

<div class="sec-h"><h2>Left this week</h2></div>
<Backlog days/>

{#if unfinished.length}<div class="sec-h"><h2>Unfinished plans</h2><span class="small">Your choice next</span></div>
 {#each unfinished as i}{@const matches=itemMatch(ctx,localItem(i)).candidates}{@const st=intentionState(ctx,i)}<section class="card unfinished" class:uf-review={st==='review'}><div class="uf-head"><Glyph sport={i.sport} mark="plan" size={30}/><div><b>{i.name??SPORTS[i.sport].name}</b><small>{dayLabel(i.date)} · {st==='review'?'Needs review':st==='awaiting'?'Awaiting data':'Not recorded'}</small></div></div>
  {#if st==='review'}<p class="small">{matches.length===1?`${matches[0].name} at ${matches[0].start} may be this session`:`${matches.length} recordings may be this session`}</p>{/if}
  <div class="uf-actions">{#if st==='review'}<button onclick={()=>matches.length===1?A.link(i.id,matches[0].id):go(`intention/${i.id}`)}>This was my session</button>{/if}<button onclick={()=>store.openSheet('reschedule',i.id)}>Reschedule</button><button onclick={()=>A.finishIntention(i.id,'released')}>Let it go</button></div>
 </section>{/each}
{/if}

{#if ctx.state.savedSleepTarget&&!ctx.state.seen.timingMigration}<button class="timing-note" onclick={()=>{A.seenMigration();go('sleep/timing');}}><Icon name="moon" size={17}/><span>Sleep timing is now an observation<small>Earlier results are kept. Review timing.</small></span><Icon name="chevron" size={14}/></button>{/if}
{#if ambitions.some(a=>!a.date)}<div class="sec-h"><h2>Longer goals</h2><button class="link" onclick={()=>store.openSheet('ambition')}>Add an event</button></div><div class="ambitions">{#each ambitions.filter(a=>!a.date) as a}<div><b>{a.name}</b><small>No date · {a.source}</small></div>{/each}</div>{:else}<button class="link add-event" onclick={()=>store.openSheet('ambition')}>Add a race or event</button>{/if}
<style>
 .wk{padding:14px}.wk-h{display:flex;flex-wrap:wrap;justify-content:space-between;align-items:flex-start;gap:8px;margin-bottom:12px}.wk-h h2{font:600 20px Outfit}.wk-h small{font-size:11.5px;color:var(--text3)}.wk-b{display:flex;gap:6px}
 .small-edit{background:rgba(255,255,255,.06);border:1px solid var(--line);border-radius:11px;display:flex;gap:5px;align-items:center;padding:8px 10px;color:var(--text);font:600 12px Inter;min-height:34px}
 .adapt-state{display:flex;align-items:center;gap:8px;width:100%;padding:8px 10px;margin:0 0 10px;border:1px solid rgba(169,200,255,.25);border-radius:12px;background:rgba(169,200,255,.08);color:var(--text);text-align:left;font-size:12px}
 .review{display:flex;justify-content:space-between;align-items:center;gap:8px;width:100%;margin-top:10px;padding:8px 10px;border:1px dashed rgba(255,181,71,.45);border-radius:12px;background:rgba(255,181,71,.06);color:var(--text2);font-size:12px;text-align:left;min-height:36px}.review b{color:#FFB547;font-weight:600}
 .evb{margin-top:10px;padding:8px 8px 10px;border-radius:14px;border:1px solid rgba(255,181,71,.3);background:rgba(255,181,71,.05)}.evb :global(.ir-main){border-left-color:var(--ev)}.ev-h{display:grid;grid-template-columns:34px 1fr 14px;gap:10px;align-items:center;width:100%;padding:2px 2px 4px;background:none;border:0;color:var(--text);text-align:left}.ev-h b{display:block;font:600 14px Inter}.ev-h small{font-size:11px;color:var(--text2)}
 .coach-line,.race-line{display:grid;grid-template-columns:34px 1fr 14px;gap:10px;align-items:center;width:100%;margin-top:10px;padding:8px 10px;border-radius:12px;border:1px solid rgba(164,139,255,.28);background:rgba(164,139,255,.07);color:var(--text);text-align:left}
 .coach-line b,.race-line b{display:block;font:600 13.5px Inter}.coach-line small,.race-line small{font-size:11px;color:var(--text2)}.gmark{font-size:24px;color:#BDAAFF;text-align:center}
 .race-line{border-color:rgba(255,181,71,.3);background:rgba(255,181,71,.06)}.cd{display:grid;place-items:center;line-height:1}.cd b{font-size:18px;color:#FFB547}.cd small{font-size:9px;color:var(--text3)}
 
 .add-event{display:block;margin:14px 2px}
 .pause-note{display:flex;gap:6px;align-items:center;margin:0 4px 10px;font-size:12px;color:#A9C8FF}
 .schedule-week{padding:12px 10px}.agenda-h{display:flex;justify-content:space-between;align-items:center;border-top:1px solid var(--line);margin:14px 2px 4px;padding-top:8px}.agenda-h h3{margin:0;font:600 17px Outfit;color:var(--text);text-transform:none;letter-spacing:0}.agenda-h .link{font-size:12px;min-height:32px}.agenda-row{display:grid;grid-template-columns:30px 1fr 15px;align-items:center;gap:12px;width:100%;margin-top:8px;padding:10px;border:1px solid var(--line);border-radius:16px;background:rgba(255,255,255,.03);color:var(--text);text-align:left}.agenda-row small{font:500 10px Inter;color:var(--text3)}.agenda-row b{display:block;font:600 16px Outfit;margin:3px 0}.agenda-row em{font-style:normal;font-size:11px;color:var(--text2)}.done{border-left:3px solid #3DDC97}.empty-day{display:flex;gap:10px;align-items:center;color:var(--text3);padding:24px 8px;font-size:13px}
 .uf-head{display:flex;align-items:center;gap:12px}.uf-head b{display:block;font:600 17px Outfit}.uf-head small{display:block;font-size:12px;color:var(--text2);margin-top:3px}.uf-actions{display:flex;flex-wrap:wrap;gap:6px;margin-top:12px;min-width:0}.unfinished{min-width:0}.uf-head>div{min-width:0}.uf-actions button{max-width:100%}.uf-actions button{font:600 12px Inter;min-height:38px;padding:8px 10px;border:1px solid var(--line);border-radius:10px;background:rgba(255,255,255,.05);color:var(--text)}
</style>
