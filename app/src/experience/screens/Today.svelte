<script lang="ts">
 import Icon from '../../lib/components/Icon.svelte';
 import Gauge from '../../lib/components/Gauge.svelte';
 import RootHeader from '../components/RootHeader.svelte';
 import ActivityTile from '../components/ActivityTile.svelte';
 import DayRows from '../components/DayRows.svelte';
 import DayCells from '../components/DayCells.svelte';
 import WeekSummary from '../components/WeekSummary.svelte';
 import Backlog from '../components/Backlog.svelte';
 import {napCharge} from '../lib/views';
 import {backlog} from '../lib/week';
 import Glyph from '../components/Glyph.svelte';
 import {store} from '../lib/store.svelte';
 import {A} from '../lib/actions';
 import {SPORTS} from '../lib/sports';
 import {dayLabel,dur} from '../lib/dates';
 import {effortFor} from '../lib/rules';
 import {isOpen} from '../lib/allocation';
 import {itemMatch, localItem, garminItem} from '../lib/match';
 import {acuteLoad} from '../lib/views';
 import {toneColor} from '../../lib/charts';
 let ctx=$derived(store.ctx),lead=$derived(ctx.lead);
 let waiting=$derived(ctx.stale||ctx.phase==='waiting'), n=$derived(waiting?null:ctx.night),r=$derived(waiting?null:ctx.readiness);
 let blocked=$derived(ctx.restActive||!!ctx.paused||ctx.hasLimits);
 let evening=$derived(['evening','winddown','late'].includes(ctx.phase));
 let effort=$derived(!blocked&&!waiting&&!evening&&!ctx.fixedReason?effortFor(r):null);
 let choice=$derived(ctx.state.todayChoice?.day===ctx.calendarDate?ctx.state.todayChoice.id:null);
 let local=$derived(ctx.intentionsToday.find(i=>i.id===choice&&i.kind!=='sleep'&&isOpen(i)));
 let garmin=$derived(local?null:ctx.scheduleToday.find(s=>s.id===choice)??ctx.scheduleToday[0]??null);
 let own=$derived(local??(!garmin?ctx.intentionsToday.find(i=>i.kind!=='sleep'&&isOpen(i)):undefined));
 let selectedSport=$derived(own?.sport??garmin?.sport);
 let selectedName=$derived(own?.name??(own?SPORTS[own.sport].name:garmin?.name));
 let otherLocal=$derived(ctx.intentionsToday.find(i=>i.kind!=='sleep'&&isOpen(i)&&i.id!==own?.id&&!i.link));
 let done=$derived(lead.kind==='done');
 // The chosen session's state: done (matched to a recording), planned, or not recorded.
 let selItem=$derived(garmin?garminItem(garmin):own?localItem(own):null);
 let selMatch=$derived(selItem?itemMatch(ctx,selItem):null);
 let sessDone=$derived(selMatch?.state==='done'?selMatch.activity:null);
 // One plain line from Garmin's readiness inputs; the gauge opens the full factors.
 let reason=$derived.by(()=>{
  if(waiting||!r||blocked||evening)return null;
  const parts:string[]=[];
  if(n&&n.asleep<n.need-30)parts.push(`Short sleep (${dur(n.asleep)} of ${dur(n.need)})`);
  const rec=Math.round(acuteLoad(ctx,ctx.calendarDate)/30);
  if(r.limit==='Recovery time'||r.value<50)parts.push(`${rec}h recovery left`);
  else if(r.limit==='Acute load')parts.push('high recent training load');
  else if(r.limit==='HRV status')parts.push('HRV below your usual');
  if(!parts.length)parts.push(r.value>=75?'Good sleep and recovery':`Garmin readiness ${r.value}`);
  return parts.slice(0,2).join(' and ').replace(/^./,c=>c.toUpperCase());
 });
 const go=(r:string)=>store.navigate(r);
 const keep=()=>own?A.selectToday(own.id):garmin?A.keepGarmin(garmin.id):undefined;
</script>
<RootHeader title="Today" help="today"/>
{#if ctx.notices.length}<div class="notice-strip" role="status"><Icon name="alert" size={16}/><p>{ctx.notices[0].text}</p><button class="link inline" onclick={()=>go(ctx.notices[0].action.go)}>{ctx.notices[0].action.label}</button></div>{/if}
{#if ctx.state.dst}<p class="small">Clocks changed by an hour overnight</p>{/if}

<section class="morning" aria-label="Last night and readiness">
 <button class="sleep-fact" onclick={()=>go('sleep/last')} aria-label="Last night’s sleep">
  <span class="ml"><Icon name="moon" size={16}/>Last night</span>
  <b class="n sleep-num">{n?dur(n.asleep):'Waiting'}</b>
  <div class="sleep-need"><i style:width={n?Math.min(100,n.asleep/n.need*100)+'%':'0%'}></i><u></u></div>
  <small>{n?`Need ${dur(n.need)} · score ${n.score}`:ctx.stale?'Last upload '+ctx.upload.label:'Not synced yet'}</small>
 </button>
 <button class="ready-fact" onclick={()=>go('readiness')} aria-label={`Readiness ${r?.value??'unavailable'}`}>
  <Gauge value={r?.value??null} size={80} stroke={7} color={toneColor(r?r.value>=75?'good':r.value>=50?'fair':'poor':'neutral')}/>
  <span><b>Readiness</b><small>{r?.level??(ctx.world.readinessSupported?'Waiting':'Not on this watch')}</small></span>
 </button>
</section>

<section class="brief hero" aria-labelledby="hero-h">
 <div class="brief-h"><h2 id="hero-h">{waiting?lead.headline:ctx.paused?(ctx.paused.reason==='ill'?'Ill · goals paused':'Training is paused'):ctx.restActive?'Rest day':ctx.hasLimits?'Your limits apply':done?lead.headline:evening?lead.headline:ctx.fixedReason?'Your recorded day':r&&r.value<50?'An easier day may fit':r?'Ready when you are':'Make room for sleep'}</h2></div>
 {#if reason}<p class="reason">{reason}</p>{/if}
 {#if effort}<div class="effort" role="img" aria-label={`Raring effort: ${effort}, from Garmin readiness ${r?.value ?? ''}`}><small>Raring effort</small>
  <div class="ebar">{#each ['rest','easy','steady','push'] as e, i}<span class="sw" class:on={e===effort}><i class="seg s{i}"></i><em>{e}</em></span>{/each}</div></div>{/if}
 {#if ctx.fixedReason}<p class="small">{ctx.fixedReason}</p>{/if}
 {#if waiting}<button class="secondary" onclick={()=>go('help/sync')}>Sync help</button>
 {:else if ctx.paused||ctx.restActive||ctx.hasLimits}<div class="calm-row"><Icon name={ctx.paused?'pause':ctx.restActive?'moon':'info'} size={24}/><span>{ctx.paused?`Since ${dayLabel(ctx.paused.since)}`:ctx.restActive?'Through tonight':ctx.limits.note||ctx.limits.exclude.map(s=>SPORTS[s].name).join(', ')+' excluded'}</span><button class="secondary" onclick={()=>go(ctx.paused?'adapt/pause':ctx.hasLimits?'adapt/limits':'plan')}>{ctx.paused?'Review pause':ctx.hasLimits?'Review limits':'View plan'}</button></div>
  {#each ctx.scheduleToday as g}<button class="ctx-row garmin" onclick={()=>go(`scheduled/${g.id}`)}><Glyph sport={g.sport} mark="garmin" size={28}/><span><small>Garmin schedule</small><b>{g.name}</b><em>{ctx.limits.exclude.includes(g.sport)?'Conflicts with your limits':ctx.limits.note?'Review needed':'Completion unknown'}</em></span><Icon name="chevron" size={15}/></button>{/each}
  {#each ctx.intentionsToday.filter(i=>i.kind!=='sleep'&&isOpen(i)&&!i.link) as i}<button class="ctx-row" onclick={()=>go(`intention/${i.id}`)}><Glyph sport={i.sport} mark="plan" size={28}/><span><small>Your plan</small><b>{i.name??SPORTS[i.sport].name}{i.time?' · '+i.time:''}</b><em>{ctx.limits.exclude.includes(i.sport)?'Conflicts with your limits':ctx.limits.note?'Review needed':ctx.paused?'Paused':'Rest today'}</em></span><Icon name="chevron" size={15}/></button>{/each}
 {:else if done && lead.activity}<ActivityTile a={lead.activity} {ctx} navigate={go}/>
 {:else if evening}<button class="tonight-line" onclick={()=>go('sleep/tonight')}><Icon name="moon" size={22}/><span><small>{ctx.tonight.nightLabel} · {ctx.tonight.source}</small><b class="n">{ctx.tonight.window??'23:20'}</b><small>Wind down {ctx.tonight.windDown} · need {dur(ctx.tonight.need)}</small></span><Icon name="chevron" size={16}/></button>
 {:else}
  <div class="session" class:garmin={!!garmin} class:done={!!sessDone}>
   <div class="session-top">{#if selectedSport}<Glyph sport={selectedSport} mark={sessDone?'recorded':garmin?'garmin':'plan'} size={40}/>{:else}<span class="choose-glyph"><Icon name="run" size={26}/></span>{/if}{#if garmin||own}<button class="session-open" onclick={()=>go(sessDone?`workout/${sessDone.id}`:garmin?`scheduled/${garmin.id}`:`intention/${own!.id}`)}><span class="source">{sessDone?'✓ Done':'Planned'} · {garmin?'Garmin schedule':'Your plan'}</span><b class="sname">{selectedName}</b><small>{#if sessDone}{sessDone.min} min{sessDone.km?` · ${sessDone.km} km`:''} · avg HR {sessDone.avgHr} · recorded {sessDone.start}{:else}{garmin?`${garmin.min} min`:own?.durationMin?`${own.durationMin} min`:'Any duration'} · {own?.time??'any time'}{/if}</small></button>{:else}<div><span class="source">Your plan</span><h3>Choose how to move</h3><small>Choose a workout or a sport</small></div>{/if}</div>
   {#if !sessDone}<div class="session-actions">{#if selectedName}<button class="keep primary" onclick={keep}>{choice===own?.id||!!garmin&&ctx.state.keptGarmin?.[garmin.id]?'Kept':'Keep'}</button>{/if}<button class="secondary choose" onclick={()=>store.openSheet('choose')}>{selectedName?'Choose another':'Choose a session'}</button></div>{/if}
   {#if otherLocal}<button class="also" onclick={()=>A.selectToday(otherLocal!.id)}>Also yours: {otherLocal.name??SPORTS[otherLocal.sport].name}{otherLocal.time?' · '+otherLocal.time:''}<Icon name="arrow-right" size={14}/></button>{/if}
  </div>
 {/if}
 {#if evening && otherLocal}<button class="also" onclick={()=>go(`intention/${otherLocal!.id}`)}>{otherLocal.name??SPORTS[otherLocal.sport].name} {otherLocal.time??''} · still in your plan<Icon name="chevron" size={14}/></button>{/if}
</section>

<DayRows/>
{#if ctx.actsToday.length||ctx.napsToday.length||ctx.lateArrivals.length||waiting}
 <div class="sec-h"><h2>Recorded today</h2><button class="link" onclick={()=>go('activities')}>Activities</button></div>
 {#each ctx.actsToday.filter(a=>a.id!==lead.activity?.id).slice(0,2) as a}<ActivityTile {a} {ctx} navigate={go}/>{/each}
 {#if waiting}{#each ctx.world.activities.filter(a=>a.date===ctx.displayedDay).slice(-1) as a}<ActivityTile {a} {ctx} navigate={go} showDate/>{/each}{/if}
 {#each ctx.napsToday as nap}{@const dl=napCharge(ctx,nap)}<button class="event" onclick={()=>go('sleep/naps')}><Icon name="moon" size={17}/><span><b>Nap · {nap.start[1]}{nap.end?'–'+nap.end[1]:''}</b><small>{nap.min} min{dl!=null?` · Body Battery ${dl>=0?'+':''}${dl}`:''} · separate from last night</small></span><Icon name="chevron" size={15}/></button>{/each}
 {#if ctx.lateArrivals.length}<button class="event" onclick={()=>go(`workout/${ctx.lateArrivals[0].id}`)}><Glyph sport={ctx.lateArrivals[0].sport} size={24}/><span><b>Arrived late · {ctx.lateArrivals[0].name}</b><small>Counts on {dayLabel(ctx.lateArrivals[0].date)}</small></span><Icon name="chevron" size={15}/></button>{/if}
{/if}
<div class="sec-h"><h2>This week</h2><button class="link" onclick={()=>go('plan')}>Your plan</button></div>
<section class="card week-overview" aria-label="This week and what's left">
 <div class="wk2"><div class="wk2-l"><WeekSummary compact/></div><div class="wk2-r"><h3>Left this week</h3><Backlog compact/></div></div>
 <div class="wk-cells"><DayCells size={32}/></div></section>
{#if !evening}<button class="event tonight-ahead" onclick={()=>go('sleep/tonight')}><Icon name="moon" size={17}/><span>Tonight · wind down {ctx.tonight.windDown}</span><Icon name="chevron" size={15}/></button>{/if}
<p class="end-note">That’s today.</p>
<style>
 .morning{display:grid;grid-template-columns:1.1fr 1fr;gap:12px;padding:11px 14px;margin:8px 0 6px;border-radius:22px;background:linear-gradient(115deg,rgba(92,155,255,.1),var(--card));border:1px solid var(--line)}.morning button{padding:0;background:none;border:0;color:var(--text);text-align:left;min-width:0}.ml{display:flex;gap:6px;align-items:center;font:600 11px Inter;color:#A9C8FF}.sleep-num{display:block;font-size:30px;font-weight:600;margin-top:4px}.morning small{font-size:10.5px;color:var(--text2)}.sleep-need{height:5px;background:rgba(255,255,255,.1);position:relative;border-radius:3px;margin:8px 0 6px}.sleep-need i{display:block;height:100%;background:#5C9BFF;border-radius:3px}.sleep-need u{position:absolute;right:0;top:-2px;height:9px;width:2px;background:var(--ink)}.ready-fact{display:flex;gap:4px;align-items:center;justify-content:center;border-left:1px solid var(--line)!important}.ready-fact b{display:block;font:600 12px Inter}.ready-fact small{display:block;margin-top:4px}
 .brief{margin-top:4px}.brief-h{display:flex;justify-content:space-between;gap:10px;align-items:center;margin:0 4px}.brief-h h2{font:600 21px Outfit;line-height:1.15}
 
 .reason{margin:2px 4px 6px;font-size:13px;color:var(--text2)}
 .effort{display:grid;gap:3px;margin:2px 4px 8px}.effort>small{font:500 10px Inter;color:var(--text3)}
 .ebar{display:flex;align-items:flex-end}.sw{flex:1;display:grid;gap:5px;justify-items:center;margin-left:-6px}.sw:first-child{margin-left:0}
 .seg{display:block;width:100%;height:16px;clip-path:polygon(0 0,calc(100% - 9px) 0,100% 50%,calc(100% - 9px) 100%,0 100%,9px 50%);opacity:.5}.sw:first-child .seg{clip-path:polygon(0 0,calc(100% - 9px) 0,100% 50%,calc(100% - 9px) 100%,0 100%);border-radius:6px 0 0 6px}.sw:last-child .seg{clip-path:polygon(0 0,100% 0,100% 100%,0 100%,9px 50%);border-radius:0 6px 6px 0}
 .s0{background:linear-gradient(90deg,#6F95FF,#5FB8D8)}.s1{background:linear-gradient(90deg,#5FB8D8,#3DDC97)}.s2{background:linear-gradient(90deg,#3DDC97,#FFB547)}.s3{background:linear-gradient(90deg,#FFB547,#FF6B6B)}
 .sw em{font:600 11px Inter;font-style:normal;text-transform:capitalize;color:var(--text3)}.sw.on .seg{opacity:1;height:21px;box-shadow:0 0 0 2px var(--ink)}.sw.on em{color:var(--text);font-weight:700}.sw.on em::before{content:'▲ ';font-size:9px;color:var(--ink)}
 .wk2{display:grid;grid-template-columns:minmax(0,1fr) minmax(0,1.1fr);gap:12px;align-items:start}.wk2-r h3{font:600 10.5px Inter;letter-spacing:.08em;text-transform:uppercase;color:var(--text3);margin:0 0 6px}
 .session.done{border-color:rgba(61,220,151,.35);background:linear-gradient(115deg,rgba(61,220,151,.09),var(--card))}.session.done .source{color:#3DDC97}
 .wk-cells{margin-top:12px;padding-top:10px;border-top:1px solid var(--line)}
 .session{border:1px solid var(--line);border-radius:22px;background:var(--card);padding:14px}.session.garmin{border-color:rgba(164,139,255,.25);background:linear-gradient(115deg,rgba(164,139,255,.09),var(--card))}.session-top{display:flex;gap:12px;align-items:center}.source{font:600 10px Inter;letter-spacing:.06em;text-transform:uppercase;color:var(--text3)}.garmin .source{color:#BDAAFF}.session h3,.sname{display:block;font:600 20px Outfit;margin:3px 0 4px}.session-top small{font-size:11px;color:var(--text2)}.session-actions{display:flex;gap:8px;align-items:center;margin-top:12px}.session-actions button{min-height:42px;margin:0;border-radius:13px;font-size:12px}.keep{width:auto;padding:10px 16px}.choose{flex:1;width:auto;padding:10px}.also{display:flex;justify-content:space-between;align-items:center;width:100%;background:none;border:0;border-top:1px solid var(--line);color:var(--text2);font-size:12px;margin-top:12px;padding:10px 0 0;text-align:left;min-height:32px}.choose-glyph{height:40px;width:40px;border-radius:12px;background:rgba(255,255,255,.08);display:grid;place-items:center;flex:none}.calm-row{display:flex;gap:12px;align-items:center;padding:12px 14px;border:1px solid var(--line);border-radius:18px;background:rgba(169,200,255,.07)}.calm-row span{flex:1;color:var(--text2);font-size:13px}.calm-row .secondary{width:auto;margin:0;padding:10px}
 .session-open{display:block;flex:1;min-width:0;padding:0;background:none;border:0;color:var(--text);text-align:left}.ctx-row{display:grid;grid-template-columns:28px 1fr 15px;gap:12px;align-items:center;width:100%;margin-top:8px;padding:10px 12px;border:1px solid var(--line);border-radius:16px;background:rgba(255,255,255,.03);color:var(--text);text-align:left}.ctx-row.garmin{border-color:rgba(164,139,255,.25);background:rgba(164,139,255,.06)}.ctx-row small{display:block;font:600 10px Inter;color:var(--text3);text-transform:uppercase;letter-spacing:.06em}.ctx-row.garmin small{color:#BDAAFF}.ctx-row b{display:block;font:600 15px Outfit;margin:2px 0}.ctx-row em{font-style:normal;font-size:11.5px;color:var(--text2)}
 .tonight-line,.event{display:grid;grid-template-columns:auto 1fr auto;align-items:center;gap:12px;width:100%;text-align:left;color:var(--text);padding:12px 14px;border:1px solid var(--line);border-radius:18px;background:var(--card);margin-top:10px}.tonight-line{background:rgba(92,124,255,.1)}.tonight-line b{display:block;font-size:28px;margin:5px 0}.tonight-line small{font-size:12px;color:var(--text2)}.event b{display:block;font:600 14px Inter}.event small{display:block;font-size:11px;color:var(--text2);margin-top:3px}.event>:global(svg:first-child){color:#A9C8FF}.tonight-ahead{font-size:13px;color:var(--text2)}.week-overview{padding:12px 8px}
 @container(max-width:300px){.wk2{grid-template-columns:1fr}.morning{grid-template-columns:1fr}.ready-fact{border-left:0!important;justify-content:flex-start}.session-actions{flex-wrap:wrap}.session-actions .choose{flex-basis:100%}.calm-row{flex-wrap:wrap}}
</style>
