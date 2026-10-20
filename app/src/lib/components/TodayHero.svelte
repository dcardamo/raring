<script lang="ts">
 import MorningStats from './MorningStats.svelte';
 import Icon from './Icon.svelte';
 import Reconcile from './Reconcile.svelte';
 import EffortBar from './EffortBar.svelte';
 import TodaySessionCard from './TodaySessionCard.svelte';
 import type {TodayView} from '../generated/TodayView';
 import type {LocalAction} from '../generated/LocalAction';
 // Last night and readiness first, then one leading message with at most one
 // action. The effort scale is Garmin's call, shown only when nothing holds it back.
 // `stillPlanned` lists today's open sessions; in the evening they stay as a quiet line, never a prompt.
 let {today, navigate, act, adapt, plan, choose, stillPlanned = []}: {today: TodayView, navigate: (route: string) => void, act: (a: LocalAction) => void, adapt: () => void, plan: () => void, choose?: () => void, stillPlanned?: {name: string, time: string | null}[]} = $props();
 let lead = $derived(today.lead);
 let effort = $derived(today.call && today.focus.effort ? today.call : null);
 // Waiting for last night: the previous day's readiness and score stay off the stats, not shown as this morning's.
 let waiting = $derived(lead ? lead.kind === 'waiting' : today.focus.sleepMissing && !today.focus.effort && today.focus.phase === 'wake');
 // Tonight's plan card below already carries the window and opens Tonight.
 let planCard = $derived(lead?.kind === 'sleep' && !!today.sleepPlan && today.focus.planOpen);
 let showReason = $derived(!!effort && (!lead || ['suggest', 'easy', 'session', 'garmin', 'done', 'fixed'].includes(lead.kind)));
 // A recorded workout is listed with everything else recorded today, right below; the headline only counts it.
 let done = $derived(lead?.kind === 'done');
 // Today's session sits below the day's context, with Keep · Choose another
 // inside it so they clearly act on the session; Rest is inside Choose.
 let session = $derived(lead && ['session', 'garmin'].includes(lead.kind) ? lead.session ?? null : null);
 let keepId = $derived(lead && ['session', 'garmin'].includes(lead.kind) ? lead.intention ?? lead.garmin ?? null : null);
 function run() {
  const a = lead?.action;
  if (!a) return;
  if (a.adapt) adapt(); else if (lead?.kind === 'suggest') plan(); else if (a.route) navigate(a.route);
 }
</script>
<MorningStats {today} {waiting} {navigate}/>
<section class="hero" aria-labelledby="lead-h">
 <h2 id="lead-h">{lead?.headline || today.headline || 'Your day is taking shape'}</h2>
 {#if lead?.line && !planCard}<p class="line">{lead.line}</p>{/if}
 {#if today.focus.note && lead?.kind !== 'fixed'}<p class="reason"><Icon name="clock" size={13}/> {today.focus.note}</p>{/if}
 {#if today.focus.schedule}<p class="reason"><Icon name="clock" size={13}/> Timed by your {today.focus.schedule} until a few more nights are recorded.</p>{/if}
 {#if effort}<EffortBar {effort}/>{/if}
 {#if showReason && today.readinessWhy}<p class="reason">{today.readinessWhy.summary}</p>{/if}
 {#if lead?.kind === 'sleep'}{#each stillPlanned as s}<button class="still" onclick={() => navigate('plan')}>{s.name}{s.time ? ` ${s.time}` : ''} · still in your plan<Icon name="chevron" size={14}/></button>{/each}{/if}
 {#if today.reconcile && !waiting}<Reconcile reconcile={today.reconcile}/>{/if}
 {#if lead && !planCard && !session && ((lead.action && !done) || lead.dismissable || lead.kind === 'review')}
  <div class="acts">
   {#if lead.action && !done}<button class="primary" onclick={run}>{lead.action.label}</button>{/if}
   {#if lead.kind === 'review'}<button class="secondary" onclick={() => act({action: 'restToday'})}>Rest today</button>{/if}
   {#if lead.dismissable}<button class="secondary" onclick={() => act({action: 'dismiss', id: lead!.id})}>Not today</button>{/if}
  </div>
 {/if}
</section>
{#if lead && session && !planCard}
 <TodaySessionCard {session}>
  {#if keepId}<button class="primary" aria-pressed={lead.kept} onclick={() => { if (!lead!.kept) act({action: 'keepToday', planId: keepId!}); }}>{lead.kept ? 'Kept for today' : 'Keep'}</button>{#if choose}<button class="secondary" onclick={choose}>Choose another</button>{:else}<button class="secondary" onclick={() => act({action: 'restToday'})}>Rest today</button>{/if}{/if}
  {#if lead.dismissable}<button class="secondary" onclick={() => act({action: 'dismiss', id: lead!.id})}>Not today</button>{/if}
 </TodaySessionCard>
{/if}
<style>
 .hero{margin:6px 0 4px;max-width:620px}.hero h2{font:600 26px/1.2 var(--font-body);letter-spacing:-.035em;margin:16px 4px 6px}
 .line{margin:2px 4px;font-size:14px;color:var(--text)}.reason{margin:2px 4px 6px;font-size:13px;color:var(--text2)}
 .still{display:flex;justify-content:space-between;align-items:center;gap:8px;width:100%;min-height:44px;margin:6px 0 0;padding:6px 4px;border:0;border-top:1px solid var(--line);background:none;color:var(--text2);font:500 13px Inter;text-align:left}
 .acts{display:flex;gap:8px;flex-wrap:wrap;margin:10px 0 0}.acts button{width:auto;flex:1 1 140px;margin-top:0}
</style>
