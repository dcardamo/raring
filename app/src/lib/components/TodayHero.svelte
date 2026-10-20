<script lang="ts">
 import Gauge from './Gauge.svelte';
 import Icon from './Icon.svelte';
 import Chip from './Chip.svelte';
 import Reconcile from './Reconcile.svelte';
 import ActivityRow from './ActivityRow.svelte';
 import type {TodayView} from '../generated/TodayView';
 import type {LocalAction} from '../generated/LocalAction';
 import {toneColor} from '../charts';
 import {duration} from '../format';
 // Last night and readiness first, then one leading message with at most one
 // action. The effort scale is Garmin's call, shown only when nothing holds it back.
 // `stillPlanned` lists today's open sessions; in the evening they stay as a quiet line, never a prompt.
 let {today, navigate, act, adapt, plan, choose, stillPlanned = []}: {today: TodayView, navigate: (route: string) => void, act: (a: LocalAction) => void, adapt: () => void, plan: () => void, choose?: () => void, stillPlanned?: {name: string, time: string | null}[]} = $props();
 let lead = $derived(today.lead);
 let night = $derived(today.focus.lastNight);
 let asleep = $derived(night?.asleepMin ?? null);
 let need = $derived(night?.needMin ?? null);
 let effort = $derived(today.call && today.focus.effort ? today.call : null);
 // Waiting for last night: the previous day's readiness and score stay off the hero, not shown as this morning's.
 let waiting = $derived(lead ? lead.kind === 'waiting' : today.focus.sleepMissing && !today.focus.effort && today.focus.phase === 'wake');
 let readiness = $derived(waiting ? null : today.readiness);
 // Tonight's plan card below already carries the window and opens Tonight.
 let planCard = $derived(lead?.kind === 'sleep' && !!today.sleepPlan && today.focus.planOpen);
 let showReason = $derived(!!effort && (!lead || ['suggest', 'easy', 'session', 'garmin', 'done', 'fixed'].includes(lead.kind)));
 let recognised = $derived(lead?.activity ? today.activities.find(a => a.id === lead!.activity) : undefined);
 // Keep · Choose another for a session that leads today; Rest is inside Choose.
 let keepId = $derived(lead && ['session', 'garmin'].includes(lead.kind) ? lead.intention ?? lead.garmin ?? null : null);
 const EFFORTS = ['rest', 'easy', 'steady', 'push'] as const;
 function run() {
  const a = lead?.action;
  if (!a) return;
  if (a.adapt) adapt(); else if (lead?.kind === 'suggest') plan(); else if (a.route) navigate(a.route);
 }
</script>
<section class="morning" aria-label="Last night and readiness">
 <button class="sleep-fact" onclick={() => navigate('sleep/last')} aria-label="Last night’s sleep">
  <span class="ml"><Icon name="moon" size={15}/>Last night</span>
  <b class="n">{asleep != null ? duration(asleep) : '—'}</b>
  {#if asleep != null && need}<span class="need-bar"><i style:width={Math.min(100, asleep / need * 100) + '%'}></i></span>{/if}
  <small>{asleep != null ? `${need ? `Need ${duration(need)}` : 'Need not recorded'}${today.sleep ? ` · score ${today.sleep.value}` : ''}` : today.focus.sleepMissing ? 'Not synced yet' : today.sleep && !waiting ? `Score ${today.sleep.value} · duration not recorded` : 'No recorded night'}</small>
 </button>
 <button class="ready-fact" onclick={() => navigate('readiness')} aria-label={`Readiness ${readiness?.value ?? (waiting ? 'waiting for last night' : 'unavailable')}`}>
  <Gauge value={readiness?.value ?? null} max={readiness?.max ?? 100} size={66} stroke={6} color={readiness ? toneColor(readiness.tone) : undefined}/>
  <span><b>Readiness</b><small>{readiness?.label ?? (today.missingMetricReasons.readiness && !waiting ? 'Not on this watch' : 'Waiting')}</small></span>
 </button>
</section>
<section class="hero" aria-labelledby="lead-h">
 <h2 id="lead-h">{lead?.headline ?? today.headline ?? 'Your day is taking shape'}</h2>
 {#if lead?.line && !planCard}<p class="line">{lead.line}</p>{/if}
 {#if today.focus.note && lead?.kind !== 'fixed'}<p class="reason"><Icon name="clock" size={13}/> {today.focus.note}</p>{/if}
 {#if today.focus.schedule}<p class="reason"><Icon name="clock" size={13}/> Timed by your {today.focus.schedule} until a few more nights are recorded.</p>{/if}
 {#if showReason && today.readinessWhy}<p class="reason">{today.readinessWhy.summary}</p>{/if}
 {#if effort}
  <div class="effort" role="img" aria-label={`Suggested effort from Garmin readiness: ${effort}`}><small>Suggested effort · from Garmin readiness</small>
   <div class="ebar">{#each EFFORTS as e, i}<span class="sw" class:on={e === effort}><i class="seg s{i}"></i><em>{e}</em></span>{/each}</div></div>
 {/if}
 {#if lead?.kind === 'sleep'}{#each stillPlanned as s}<button class="still" onclick={() => navigate('plan')}>{s.name}{s.time ? ` ${s.time}` : ''} · still in your plan<Icon name="chevron" size={14}/></button>{/each}{/if}
 {#if recognised}<div class="recognised"><ActivityRow activity={recognised} open={() => navigate('workout/' + recognised!.id)}/></div>{/if}
 {#if today.chips.length && !['sleep', 'waiting'].includes(lead?.kind ?? '')}<div class="chips">{#each today.chips as chip}<Chip {chip} label={chip.icon === 'moon' ? 'View sleep' : chip.icon === 'hrv' ? 'View HRV' : 'View training'} onclick={() => navigate(chip.icon === 'moon' ? 'sleep/last' : chip.icon === 'hrv' ? 'metric/hrv' : 'training')}/>{/each}</div>{/if}
 {#if today.reconcile && !waiting}<Reconcile reconcile={today.reconcile}/>{/if}
 {#if lead && !planCard && (lead.action || lead.dismissable || ['session', 'garmin', 'review'].includes(lead.kind))}
  <div class="acts">
   {#if keepId}<button class="primary" aria-pressed={lead.kept} onclick={() => { if (!lead!.kept) act({action: 'keepToday', planId: keepId!}); }}>{lead.kept ? 'Kept for today' : 'Keep'}</button>{#if choose}<button class="secondary" onclick={choose}>Choose another</button>{/if}
   {:else if lead.action && !recognised}<button class="primary" onclick={run}>{lead.action.label}</button>{/if}
   {#if lead.kind === 'review' || (keepId && !choose)}<button class="secondary" onclick={() => act({action: 'restToday'})}>Rest today</button>{/if}
   {#if lead.dismissable}<button class="secondary" onclick={() => act({action: 'dismiss', id: lead!.id})}>Not today</button>{/if}
  </div>
 {/if}
</section>
<style>
 .morning{display:grid;grid-template-columns:minmax(0,1.1fr) minmax(0,1fr);gap:12px;padding:11px 14px;margin:4px 0 6px;border-radius:22px;background:linear-gradient(115deg,rgba(92,155,255,.1),var(--card));border:1px solid var(--line);max-width:620px}
 .morning button{padding:0;background:none;border:0;color:var(--text);text-align:left;min-width:0;min-height:44px}
 .ml{display:flex;gap:6px;align-items:center;font:600 11px Inter;color:#A9C8FF}
 .sleep-fact b{display:block;font-size:30px;font-weight:600;margin:4px 0 2px}.sleep-fact small,.ready-fact small{display:block;font-size:12px;color:var(--text2)}
 .need-bar{display:block;height:6px;border-radius:3px;background:rgba(255,255,255,.08);overflow:hidden;margin:4px 0}.need-bar i{display:block;height:100%;background:#A9C8FF}
 .ready-fact{display:flex;align-items:center;gap:10px;border-left:1px solid var(--line)!important;padding-left:12px!important}.ready-fact span{min-width:0}.ready-fact b{display:block;font:600 15px Outfit}
 .hero{margin:6px 0 4px;max-width:620px}.hero h2{font:600 23px/1.15 Outfit;margin:8px 4px 2px}
 .line{margin:2px 4px;font-size:14px;color:var(--text)}.reason{margin:2px 4px 6px;font-size:13px;color:var(--text2)}
 .effort{display:grid;gap:4px;margin:8px 4px 6px}.effort>small{font:500 11px Inter;color:var(--text2)}
 .ebar{display:flex;align-items:flex-end}.sw{flex:1;display:grid;gap:5px;justify-items:center;margin-left:-6px}.sw:first-child{margin-left:0}
 .seg{display:block;width:100%;height:16px;clip-path:polygon(0 0,calc(100% - 9px) 0,100% 50%,calc(100% - 9px) 100%,0 100%,9px 50%);opacity:.5}
 .sw:first-child .seg{clip-path:polygon(0 0,calc(100% - 9px) 0,100% 50%,calc(100% - 9px) 100%,0 100%);border-radius:6px 0 0 6px}.sw:last-child .seg{clip-path:polygon(0 0,100% 0,100% 100%,0 100%,9px 50%);border-radius:0 6px 6px 0}
 .s0{background:linear-gradient(90deg,#6F95FF,#5FB8D8)}.s1{background:linear-gradient(90deg,#5FB8D8,#3DDC97)}.s2{background:linear-gradient(90deg,#3DDC97,#FFB547)}.s3{background:linear-gradient(90deg,#FFB547,#FF6B6B)}
 .sw em{font:600 12px Inter;font-style:normal;text-transform:capitalize;color:var(--text2)}.sw.on .seg{opacity:1;height:21px;box-shadow:0 0 0 2px var(--ink)}.sw.on em{color:var(--text);font-weight:700}.sw.on em::before{content:'▲ ';font-size:9px;color:var(--ink)}
 .still{display:flex;justify-content:space-between;align-items:center;gap:8px;width:100%;min-height:44px;margin:6px 0 0;padding:6px 4px;border:0;border-top:1px solid var(--line);background:none;color:var(--text2);font:500 13px Inter;text-align:left}
 .recognised{margin:10px 0 4px;padding:12px;border:1px solid rgba(61,220,151,.3);border-radius:18px;background:rgba(61,220,151,.06)}
 .chips{justify-content:flex-start;margin:10px 4px 0}
 .acts{display:flex;gap:8px;flex-wrap:wrap;margin:10px 0 0}.acts button{width:auto;flex:1 1 140px;margin-top:0}
 @container(max-width:300px){.morning{grid-template-columns:1fr}.ready-fact{border-left:0!important;padding-left:0!important}}
</style>
