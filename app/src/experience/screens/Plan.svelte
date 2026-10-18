<script lang="ts">
 import Icon from '../../lib/components/Icon.svelte';
 import RootHeader from '../components/RootHeader.svelte';
 import GoalLedger from '../components/GoalLedger.svelte';
 import DayCells from '../components/DayCells.svelte';
 import Glyph from '../components/Glyph.svelte';
 import {store} from '../lib/store.svelte';
 import {A} from '../lib/actions';
 import {isOpen,intentionState,candidates,allocationFor,goalsFor} from '../lib/allocation';
 import {SPORTS} from '../lib/sports';
 import {COACH} from '../lib/world';
 import {dayLabel,weekRange,daysBetween,shortDay} from '../lib/dates';
 let ctx=$derived(store.ctx), selected=$derived((store.ui.day as string)??ctx.calendarDate);
 let goals=$derived(goalsFor(ctx));
 let local=$derived(ctx.state.intentions.filter(i=>i.date===selected&&isOpen(i)));
 let recorded=$derived(ctx.world.activities.filter(a=>a.date===selected));
 let scheduled=$derived(ctx.world.schedule.filter(s=>s.date===selected&&!local.some(i=>i.garminId===s.id)));
 let unfinished=$derived(ctx.state.intentions.filter(i=>['review','not-recorded','awaiting'].includes(intentionState(ctx,i))));
 const labels={strength:'strength',zone2:'Zone 2 min',im:'intensity min',sleep:'night'};
 // Pause intervals touching this week stay visible after an explicit resume.
 let weekPauses=$derived(ctx.state.pauses.filter(p=>p.since<=ctx.weekTo&&(!p.until||p.until>=ctx.weekFrom)&&p.until));
 const go=(r:string)=>store.navigate(r);
</script>
<RootHeader title="Plan" adapt/>
<div class="plan-heading"><div><h2>Your week, your way</h2><p>{weekRange(ctx.weekFrom)} · choose goals, then place them</p></div><button class="small-edit" onclick={()=>store.openSheet('goals')}><Icon name="pencil" size={14}/>Goals</button></div>
{#if ctx.paused||ctx.restActive||ctx.hasLimits}<button class="adapt-state" onclick={()=>go('adapt')}><Icon name={ctx.paused?'pause':ctx.restActive?'moon':'info'} size={16}/>{ctx.paused?`Training paused since ${shortDay(ctx.paused.since)}`:ctx.restActive?'Rest day · through tonight':'Your limits apply'}<Icon name="chevron" size={14}/></button>{/if}
{#each weekPauses as p}<p class="pause-note"><Icon name="pause" size={13}/>Paused {shortDay(p.since)}–{shortDay(p.until!)}</p>{/each}
<GoalLedger/>

<div class="sec-h"><h2>Place it in your week</h2><button class="link" onclick={()=>store.openSheet('allocate')}>Add a session</button></div>
<section class="card schedule-week"><DayCells selected={selected} select={d=>store.setUi({day:d})}/>
 <div class="agenda-h"><h3>{selected===ctx.calendarDate?'Today':dayLabel(selected)}</h3><button class="link" onclick={()=>store.openSheet('day',selected)}>Day details</button></div>
 {#each local as i}
  {@const alloc=allocationFor(i)}
  <button class="agenda-row local" onclick={()=>go(`intention/${i.id}`)} style:--ac={i.kind==='sleep'?'#A9C8FF':SPORTS[i.sport].color}><Glyph sport={i.kind==='sleep'?'other':i.sport} mark="plan" size={30}/><span><small>Your plan{i.garminId?' · counts a Garmin session':''}</small><b>{i.name??SPORTS[i.sport].name}</b><em>{i.time??'Any time'} · {i.link?'linked by you':Object.entries(alloc).filter(([,n])=>n).map(([k,n])=>`${n} ${labels[k as keyof typeof labels]}`).join(' · ')||'Unallocated session'}</em></span><Icon name="chevron" size={15}/></button>
 {/each}
 {#each scheduled as s}<button class="agenda-row imported" onclick={()=>go(`scheduled/${s.id}`)}><Glyph sport={s.sport} mark="garmin" size={30}/><span><small>Garmin schedule · read-only</small><b>{s.name}</b><em>{ctx.limits.exclude.includes(s.sport)?'Conflicts with your limits':ctx.limits.note?'Review needed':'Completion unknown'}</em></span><Icon name="chevron" size={15}/></button>{/each}
 {#each recorded as a}<button class="agenda-row done" onclick={()=>go(`workout/${a.id}`)}><Glyph sport={a.sport} size={30}/><span><small>Recorded · Garmin</small><b>{a.name}</b><em>{a.start} · {a.min} min{a.km?' · '+a.km+' km':''}</em></span><Icon name="chevron" size={15}/></button>{/each}
 {#if !local.length&&!recorded.length&&!scheduled.length}<div class="empty-day"><Icon name={ctx.state.rest?.day===selected?'moon':'clock'} size={24}/><p>{ctx.state.rest?.day===selected?'A day of rest':'Space for what suits you'}</p></div>{/if}
 <button class="secondary" onclick={()=>store.openSheet('allocate')}>Place a session</button>
</section>

{#if unfinished.length}<div class="sec-h"><h2>Unfinished plans</h2><span class="small">Your choice next</span></div>
 {#each unfinished as i}{@const matches=candidates(ctx,i)}{@const st=intentionState(ctx,i)}<section class="card unfinished" class:review={st==='review'}><div class="uf-head"><Glyph sport={i.sport} mark="plan" size={30}/><div><b>{i.name??SPORTS[i.sport].name}</b><small>{dayLabel(i.date)} · {st==='review'?'Needs review':st==='awaiting'?'Awaiting data':'Not recorded'}</small></div></div>
  {#if st==='review'}<p class="small">{matches.length===1?`${matches[0].name} at ${matches[0].start} may be this session`:`${matches.length} recordings may be this session`}</p>{/if}
  <div class="uf-actions">{#if st==='review'}<button onclick={()=>matches.length===1?A.link(i.id,matches[0].id):go(`intention/${i.id}`)}>This was my session</button>{/if}<button onclick={()=>store.openSheet('reschedule',i.id)}>Reschedule</button><button onclick={()=>A.finishIntention(i.id,'released')}>Let it go</button></div>
 </section>{/each}
{/if}

{#if ctx.world.schedule.length}
 <div class="sec-h"><h2>From Garmin</h2><span class="small">Read-only</span></div>
 <section class="garmin-plan">
  <button class="garmin-h" aria-label={`Garmin Coach: ${COACH.name}, read-only`} onclick={()=>go('plan/garmin')}><span class="diamond" aria-hidden="true">◇</span><span><small>GARMIN COACH · ACTIVE</small><b class="gname">{COACH.name}</b><span class="gsub">Since {dayLabel(COACH.since)} · managed in Garmin Connect</span></span><Icon name="chevron" size={16}/></button>
  <h4>Garmin schedule</h4>
  {#each ctx.world.schedule as s}<div class="garmin-item"><button onclick={()=>go(`scheduled/${s.id}`)}><span class="date-box"><b class="n">{Number(s.date.slice(8))}</b><small>{dayLabel(s.date).slice(0,3)}</small></span><span><b>{s.name}</b><small>{s.min} min · completion unknown</small></span></button><button class="allocate-garmin" onclick={()=>store.openSheet('allocate','garmin:'+s.id)}>Count toward my goals <Icon name="arrow-right" size={14}/></button></div>{/each}
  <p class="coverage">{ctx.state.calendar==='partial'?'Garmin returned a full page; later sessions may be missing':ctx.state.calendar==='stale'?'Last checked Mon 16 Nov · may be out of date':'Checked 18 Nov 07:12 · covers 18 Nov–1 Dec'}</p>
 </section>
{/if}
{#if ctx.state.savedSleepTarget&&!ctx.state.seen.timingMigration}<button class="timing-note" onclick={()=>{A.seenMigration();go('sleep/timing');}}><Icon name="moon" size={17}/><span>Sleep timing is now an observation<small>Earlier results are kept. Review timing.</small></span><Icon name="chevron" size={14}/></button>{/if}
{#if ctx.world.ambitions.length}<div class="sec-h"><h2>Looking ahead</h2></div><div class="ambitions">{#each ctx.world.ambitions as a}<div><b>{a.name}</b><small>{a.date?`${daysBetween(ctx.calendarDate,a.date)} days · ${dayLabel(a.date)}`:'No date'}</small><small>{a.source}</small></div>{/each}</div>{/if}
<style>
 .plan-heading{display:flex;align-items:center;justify-content:space-between;gap:12px;margin:18px 2px 12px}.plan-heading h2{font:600 23px Outfit}.plan-heading p{font-size:12px;color:var(--text2);margin-top:4px}.small-edit{background:rgba(255,255,255,.06);border:1px solid var(--line);border-radius:12px;display:flex;gap:6px;align-items:center;padding:10px 12px;color:var(--text);font:600 12px Inter}.adapt-state{display:flex;align-items:center;gap:9px;width:100%;padding:10px 12px;margin:8px 0 12px;border:1px solid rgba(169,200,255,.25);border-radius:14px;background:rgba(169,200,255,.08);color:var(--text);text-align:left;font-size:13px}
 .pause-note{display:flex;gap:6px;align-items:center;margin:0 4px 10px;font-size:12px;color:#A9C8FF}
 .schedule-week{padding:12px 10px}.agenda-h{display:flex;justify-content:space-between;align-items:center;border-top:1px solid var(--line);margin:14px 2px 4px;padding-top:8px}.agenda-h h3{margin:0;font:600 17px Outfit;color:var(--text);text-transform:none;letter-spacing:0}.agenda-h .link{font-size:12px;min-height:32px}.agenda-row{display:grid;grid-template-columns:30px 1fr 15px;align-items:center;gap:12px;width:100%;margin-top:8px;padding:10px;border:1px solid var(--line);border-radius:16px;background:rgba(255,255,255,.03);color:var(--text);text-align:left}.agenda-row small{font:500 10px Inter;color:var(--text3)}.agenda-row b{display:block;font:600 16px Outfit;margin:3px 0}.agenda-row em{font-style:normal;font-size:11px;color:var(--text2)}.agenda-row.local{border-left:3px solid var(--ac)}.agenda-row.imported{border-color:rgba(164,139,255,.25);background:rgba(164,139,255,.06)}.agenda-row.imported small{color:#C9BEFF}.done{border-left:3px solid #3DDC97}.empty-day{display:flex;gap:10px;align-items:center;color:var(--text3);padding:24px 8px;font-size:13px}
 .uf-head{display:flex;align-items:center;gap:12px}.uf-head b{display:block;font:600 17px Outfit}.uf-head small{display:block;font-size:12px;color:var(--text2);margin-top:3px}.uf-actions{display:flex;flex-wrap:wrap;gap:6px;margin-top:12px}.uf-actions button{font:600 12px Inter;min-height:38px;padding:8px 10px;border:1px solid var(--line);border-radius:10px;background:rgba(255,255,255,.05);color:var(--text)}
 .garmin-plan{margin-top:10px;padding:16px;border:1px solid rgba(164,139,255,.3);border-radius:24px;background:linear-gradient(160deg,rgba(164,139,255,.12),var(--card));position:relative}.garmin-h{display:flex;gap:14px;align-items:center;width:100%;padding:0;background:none;border:0;color:var(--text);text-align:left}.garmin-h>span:nth-child(2){flex:1}.gname{display:block;font:600 24px Outfit}.gsub{display:block;font-size:12px;color:var(--text2);margin-top:2px}.garmin-h small{display:block}.diamond{font-size:42px;color:#BDAAFF}.garmin-h small{font:600 10px Inter;letter-spacing:.1em;color:#BDAAFF}.garmin-item{border-top:1px solid rgba(164,139,255,.2);padding-top:12px;margin-top:14px}.garmin-item>button:first-child{display:flex;gap:12px;align-items:center;width:100%;background:none;border:0;color:var(--text);text-align:left;padding:0}.date-box{display:flex;flex-direction:column;align-items:center;justify-content:center;width:40px;height:46px;border:1px solid rgba(164,139,255,.3);border-radius:10px;flex:none}.date-box b{font-size:21px}.date-box small{font-size:10px}.garmin-item b{display:block;font-weight:600}.garmin-item small{display:block;font-size:11px;color:var(--text2);margin-top:3px}.allocate-garmin{display:flex;gap:8px;align-items:center;border:0;background:none;color:#D6CCFF;font:600 12px Inter;padding:8px 0;min-height:34px}.garmin-plan h4{font:600 10px Inter;letter-spacing:.1em;text-transform:uppercase;color:#BDAAFF;margin:18px 0 0}.coverage{font-size:11px;color:var(--text2);margin:12px 0 0}.timing-note{display:flex;gap:10px;align-items:center;padding:12px;border:1px solid var(--line);border-radius:16px;width:100%;background:var(--card);color:var(--text2);text-align:left;font-size:12px;margin-top:14px}.timing-note small{display:block;color:var(--text3);margin-top:3px}.ambitions{display:grid;grid-template-columns:repeat(auto-fit,minmax(130px,1fr));gap:10px}.ambitions>div{min-width:0}.ambitions>div{padding:14px;background:var(--card);border:1px solid var(--line);border-radius:18px}.ambitions b{font:600 17px Outfit;display:block}.ambitions small{display:block;font-size:12px;color:var(--text2);margin-top:3px}
</style>
