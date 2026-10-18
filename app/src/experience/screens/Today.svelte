<script lang="ts">
 import Icon from '../../lib/components/Icon.svelte';
 import Gauge from '../../lib/components/Gauge.svelte';
 import RootHeader from '../components/RootHeader.svelte';
 import ActivityTile from '../components/ActivityTile.svelte';
 import DayRows from '../components/DayRows.svelte';
 import DayCells from '../components/DayCells.svelte';
 import Glyph from '../components/Glyph.svelte';
 import {store} from '../lib/store.svelte';
 import {A} from '../lib/actions';
 import {SPORTS} from '../lib/sports';
 import {dayLabel,dur} from '../lib/dates';
 import {effortFor} from '../lib/rules';
 import {isOpen} from '../lib/allocation';
 import {toneColor} from '../../lib/charts';
 let ctx=$derived(store.ctx),lead=$derived(ctx.lead);
 let waiting=$derived(ctx.stale||ctx.phase==='waiting'), n=$derived(waiting?null:ctx.night),r=$derived(waiting?null:ctx.readiness);
 let blocked=$derived(ctx.restActive||!!ctx.paused||ctx.hasLimits);
 let evening=$derived(['evening','winddown','late'].includes(ctx.phase));
 let effort=$derived(!blocked&&!waiting&&!evening&&!ctx.fixedReason?effortFor(r):null);
 let choice=$derived(ctx.state.todayChoice?.day===ctx.calendarDate?ctx.state.todayChoice.id:null);
 let local=$derived(ctx.intentionsToday.find(i=>i.id===choice&&i.kind!=='sleep'&&isOpen(i)));
 let garmin=$derived(local?null:ctx.scheduleToday.find(s=>s.id===choice)??ctx.scheduleToday[0]??null);
 let own=$derived(local??(!garmin?ctx.intentionsToday.find(i=>i.kind!=='sleep'&&!i.link&&isOpen(i)):undefined));
 let selectedSport=$derived(own?.sport??garmin?.sport);
 let selectedName=$derived(own?.name??(own?SPORTS[own.sport].name:garmin?.name));
 let otherLocal=$derived(ctx.intentionsToday.find(i=>i.kind!=='sleep'&&isOpen(i)&&i.id!==own?.id&&!i.link));
 let done=$derived(lead.kind==='done');
 const go=(r:string)=>store.navigate(r);
 const keep=()=>own?A.selectToday(own.id):garmin?A.keepGarmin(garmin.id):undefined;
</script>
<RootHeader title="Today" adapt/>
{#if ctx.notices.length}<div class="notice-strip" role="status"><Icon name="alert" size={16}/><p>{ctx.notices[0].text}</p><button class="link inline" onclick={()=>go(ctx.notices[0].action.go)}>{ctx.notices[0].action.label}</button></div>{/if}
{#if ctx.state.dst}<p class="small">Clocks changed by an hour overnight</p>{/if}

<section class="morning" aria-label="Last night and readiness">
 <button class="sleep-fact" onclick={()=>go('sleep')} aria-label="Last night’s sleep">
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
 <div class="brief-h"><h2 id="hero-h">{waiting?lead.headline:ctx.paused?'Training is paused':ctx.restActive?'Rest day':ctx.hasLimits?'Your limits apply':done?lead.headline:evening?lead.headline:ctx.fixedReason?'Your recorded day':r&&r.value<50?'An easier day may fit':r?'Ready when you are':'Make room for sleep'}</h2><button class="why link" onclick={()=>store.openSheet('why')}>Why?</button></div>
 {#if effort}<div class="effort-line"><small>Raring effort</small><div class="effort-scale" role="img" aria-label={`Raring effort ${effort}, based on Garmin readiness`}>{#each ['rest','easy','steady','push'] as e}<span class:on={e===effort}>{e}</span>{/each}</div></div>{/if}
 {#if ctx.fixedReason}<p class="small">{ctx.fixedReason}</p>{/if}
 {#if waiting}<button class="secondary" onclick={()=>go('help/sync')}>Sync help</button>
 {:else if ctx.paused||ctx.restActive||ctx.hasLimits}<div class="calm-row"><Icon name={ctx.paused?'pause':ctx.restActive?'moon':'info'} size={24}/><span>{ctx.paused?`Since ${dayLabel(ctx.paused.since)}`:ctx.restActive?'Through tonight':ctx.limits.note||ctx.limits.exclude.map(s=>SPORTS[s].name).join(', ')+' excluded'}</span><button class="secondary" onclick={()=>go(ctx.paused?'adapt/pause':ctx.hasLimits?'adapt/limits':'plan')}>{ctx.paused?'Review pause':ctx.hasLimits?'Review limits':'View plan'}</button></div>
 {:else if done && lead.activity}<ActivityTile a={lead.activity} {ctx} navigate={go}/>
 {:else if evening}<button class="tonight-line" onclick={()=>go('sleep/tonight')}><Icon name="moon" size={22}/><span><small>{ctx.tonight.nightLabel} · {ctx.tonight.source}</small><b class="n">{ctx.tonight.window??'23:20'}</b><small>Wind down {ctx.tonight.windDown} · need {dur(ctx.tonight.need)}</small></span><Icon name="chevron" size={16}/></button>
 {:else}
  <div class="session" class:garmin={!!garmin}>
   <div class="session-top">{#if selectedSport}<Glyph sport={selectedSport} mark={garmin?'garmin':'plan'} size={40}/>{:else}<span class="choose-glyph"><Icon name="run" size={26}/></span>{/if}<div><span class="source">{garmin?'Garmin schedule':own?.garminId?'Garmin schedule · counts toward your goals':'Your plan'}</span><h3>{selectedName??'Choose how to move'}</h3><small>{garmin?`${garmin.min} min · read-only · completion unknown`:own?.time??'Choose a workout or a sport'}</small></div></div>
   <div class="session-actions">{#if selectedName}<button class="keep primary" onclick={keep}>{choice===own?.id||!!garmin&&ctx.state.keptGarmin?.[garmin.id]?'Kept':'Keep'}</button>{/if}<button class="secondary choose" onclick={()=>store.openSheet('choose')}>{selectedName?'Choose another':'Choose a session'}</button><button class="rest-button" onclick={A.rest}><Icon name="moon" size={15}/>Rest</button></div>
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
 {#each ctx.napsToday as nap}<button class="event" onclick={()=>go('sleep/naps')}><Icon name="moon" size={17}/><span><b>Nap · {nap.start[1]}{nap.end?'–'+nap.end[1]:''}</b><small>{nap.min} min · recorded separately</small></span><Icon name="chevron" size={15}/></button>{/each}
 {#if ctx.lateArrivals.length}<button class="event" onclick={()=>go(`workout/${ctx.lateArrivals[0].id}`)}><Glyph sport={ctx.lateArrivals[0].sport} size={24}/><span><b>Arrived late · {ctx.lateArrivals[0].name}</b><small>Counts on {dayLabel(ctx.lateArrivals[0].date)}</small></span><Icon name="chevron" size={15}/></button>{/if}
{/if}
<div class="sec-h"><h2>This week</h2><button class="link" onclick={()=>go('plan')}>Your plan</button></div>
<section class="card week-overview"><DayCells compact/></section>
{#if !evening}<button class="event tonight-ahead" onclick={()=>go('sleep/tonight')}><Icon name="moon" size={17}/><span>Tonight · wind down {ctx.tonight.windDown}</span><Icon name="chevron" size={15}/></button>{/if}
<p class="end-note">That’s today.</p>
<style>
 .morning{display:grid;grid-template-columns:1.1fr 1fr;gap:12px;padding:14px;margin:12px 0 10px;border-radius:22px;background:linear-gradient(115deg,rgba(92,155,255,.1),var(--card));border:1px solid var(--line)}.morning button{padding:0;background:none;border:0;color:var(--text);text-align:left;min-width:0}.ml{display:flex;gap:6px;align-items:center;font:600 11px Inter;color:#A9C8FF}.sleep-num{display:block;font-size:30px;font-weight:600;margin-top:4px}.morning small{font-size:10.5px;color:var(--text2)}.sleep-need{height:5px;background:rgba(255,255,255,.1);position:relative;border-radius:3px;margin:8px 0 6px}.sleep-need i{display:block;height:100%;background:#5C9BFF;border-radius:3px}.sleep-need u{position:absolute;right:0;top:-2px;height:9px;width:2px;background:var(--ink)}.ready-fact{display:flex;gap:4px;align-items:center;justify-content:center;border-left:1px solid var(--line)!important}.ready-fact b{display:block;font:600 12px Inter}.ready-fact small{display:block;margin-top:4px}
 .brief{margin-top:8px}.brief-h{display:flex;justify-content:space-between;gap:10px;align-items:center;margin:0 4px}.brief-h h2{font:600 21px Outfit;line-height:1.15}.why{font-size:12px;flex:none;min-height:32px;padding:8px 0}.effort-line{display:flex;align-items:center;gap:12px;margin:2px 4px 10px}.effort-line>small{font:500 10px Inter;color:var(--text3);white-space:nowrap}.effort-scale{display:flex;flex:1;gap:3px}.effort-scale span{flex:1;padding:5px 3px;border-radius:6px;font:600 11px Inter;text-transform:capitalize;color:var(--text3);text-align:center;background:rgba(255,255,255,.035)}.effort-scale span.on{color:var(--bg);background:var(--ink)}
 .session{border:1px solid var(--line);border-radius:22px;background:var(--card);padding:14px}.session.garmin{border-color:rgba(164,139,255,.25);background:linear-gradient(115deg,rgba(164,139,255,.09),var(--card))}.session-top{display:flex;gap:12px;align-items:center}.source{font:600 10px Inter;letter-spacing:.06em;text-transform:uppercase;color:var(--text3)}.garmin .source{color:#BDAAFF}.session h3{font:600 20px Outfit;margin:3px 0 4px}.session-top small{font-size:11px;color:var(--text2)}.session-actions{display:flex;gap:8px;align-items:center;margin-top:12px}.session-actions button{min-height:42px;margin:0;border-radius:13px;font-size:12px}.keep{width:auto;padding:10px 16px}.choose{flex:1;width:auto;padding:10px}.rest-button{display:flex;gap:5px;align-items:center;background:none;border:1px solid var(--line);padding:10px 12px;color:var(--text)}.also{display:flex;justify-content:space-between;align-items:center;width:100%;background:none;border:0;border-top:1px solid var(--line);color:var(--text2);font-size:12px;margin-top:12px;padding:10px 0 0;text-align:left;min-height:32px}.choose-glyph{height:40px;width:40px;border-radius:12px;background:rgba(255,255,255,.08);display:grid;place-items:center;flex:none}.calm-row{display:flex;gap:12px;align-items:center;padding:12px 14px;border:1px solid var(--line);border-radius:18px;background:rgba(169,200,255,.07)}.calm-row span{flex:1;color:var(--text2);font-size:13px}.calm-row .secondary{width:auto;margin:0;padding:10px}
 .tonight-line,.event{display:grid;grid-template-columns:auto 1fr auto;align-items:center;gap:12px;width:100%;text-align:left;color:var(--text);padding:12px 14px;border:1px solid var(--line);border-radius:18px;background:var(--card);margin-top:10px}.tonight-line{background:rgba(92,124,255,.1)}.tonight-line b{display:block;font-size:28px;margin:5px 0}.tonight-line small{font-size:12px;color:var(--text2)}.event b{display:block;font:600 14px Inter}.event small{display:block;font-size:11px;color:var(--text2);margin-top:3px}.event>:global(svg:first-child){color:#A9C8FF}.tonight-ahead{font-size:13px;color:var(--text2)}.week-overview{padding:12px 8px}
 @container(max-width:300px){.morning{grid-template-columns:1fr}.ready-fact{border-left:0!important;justify-content:flex-start}.session-actions{flex-wrap:wrap}.session-actions .choose{flex-basis:100%}.effort-line{flex-wrap:wrap}.calm-row{flex-wrap:wrap}}
</style>
