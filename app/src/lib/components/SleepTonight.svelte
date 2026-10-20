<script lang="ts">
 import Card from './Card.svelte';
 import Icon from './Icon.svelte';
 import SleepClock from './SleepClock.svelte';
 import RecentNights from './RecentNights.svelte';
 import SleepPlanEvidence from './SleepPlanEvidence.svelte';
 import type {SleepView} from '../generated/SleepView';
 import type {PlanBoard} from '../generated/PlanBoard';
 import type {LocalAction} from '../generated/LocalAction';
 import {dateLabel, duration, readAt} from '../format';
 import {wakeLabel} from '../tonight';
 // Tonight as a plan: when to wind down and be asleep from Garmin's window or
 // need and the usual wake, the week's sleep goal, and recent nights against
 // Garmin's need as plain facts. No sleep debt is computed.
 let {sleep, board, act, reminder, setReminder, navigate, now = new Date(), fallAsleep, setFallAsleep}: {sleep: SleepView, board: PlanBoard | null, act: (a: LocalAction) => void,
  reminder: boolean, setReminder: (on: boolean) => void, navigate: (route: string) => void, now?: Date,
  fallAsleep: number, setFallAsleep: (minutes: number) => void} = $props();
 let plan = $derived(sleep.sleepPlan);
 let garmin = $derived(plan?.bedtimeSource === 'garmin');
 let why = $state(false);
 // Only tonight's window can have passed; an older plan is labelled stale by the core.
 let passed = $derived.by(() => { const t = plan?.bedtimeAt ? new Date(plan.bedtimeAt).getTime() : NaN; return now.getTime() > t && now.getTime() - t < 12 * 3600_000; });
 let goals = $derived(board?.goals?.targets ?? null);
 let timing = $derived(board?.ledger.find(l => l.kind === 'sleepTiming') ?? null);
 // Tonight's evening, the day before the morning the night ends on.
 let evening = $derived(plan ? new Date(Date.parse(plan.night + 'T12:00:00Z') - 864e5).toISOString().slice(0, 10) : null);
 let placed = $derived(!!board && !!evening && board.items.some(p => p.intention.kind === 'windDown' && p.intention.date === evening && p.state !== 'letGo'));
 let canPlace = $derived(!!board && !!evening && !!goals && goals.sleepNights > 0 && goals.sleepKind !== 'savedPlan' && !!goals.sleepTime && evening >= board.today && !placed && !board.readOnly);
 function placeTonight() { if (!goals?.sleepTime || !evening) return; act({action: 'addIntention', date: evening, time: goals.sleepTime, sport: 'other', typeKey: null, name: `${goals.sleepKind === 'windDown' ? 'Wind down' : 'Asleep by'} ${goals.sleepTime}`, durationMin: null, allocation: {strengthSessions: 0, zone2Min: 0, highIntensityMin: 0, intensityMin: 0, sleepNights: 1}, kind: 'windDown', garminId: null}); }
 let ownTime = $derived(goals && goals.sleepNights > 0 && goals.sleepTime ? goals.sleepTime : null);
 let nights = $derived(sleep.recent);
 // The usual wake beside the plan's own wake only when they differ.
 let up = $derived(plan?.wakeAt ? plan.wakeAt.slice(11, 16) : null);
 let recorded = $derived(nights.filter(n => n.asleepMin != null));
 let below = $derived(recorded.filter(n => n.needMin != null && n.asleepMin! < n.needMin));
</script>
{#if plan}
 <Card title="Tonight" extra="tonight-plan" help="sleep-tonight">
  {#snippet aside()}<span class="pill"><Icon name="moon" size={13}/> {dateLabel(plan.night)} · {garmin ? 'Garmin plan' : 'Raring estimate'}</span>{/snippet}
  <div class="tn">
   <SleepClock {plan} {now}/>
   <dl class="facts">
    <div><dt><i style:background="var(--fair)"></i>Wind down</dt><dd class="n">{ownTime && goals?.sleepKind === 'windDown' ? ownTime : plan.windDown}<small>{ownTime && goals?.sleepKind === 'windDown' ? 'your goal' : `Raring · ${plan.windDownLeadMin} min before ${plan.bedtimeStart && plan.bedtimeStartAt && plan.bedtimeStartAt < plan.inBedAt ? 'Garmin’s window' : 'in bed'}`}</small></dd></div>
    <div><dt><i style:background="var(--text)"></i>{garmin ? 'Bedtime window' : 'Asleep by'}</dt><dd class="n">{ownTime && goals?.sleepKind === 'asleepBy' ? ownTime : plan.bedtimeStart ? `${plan.bedtimeStart}–${plan.bedtime}` : plan.bedtime}<small>{ownTime && goals?.sleepKind === 'asleepBy' ? 'your goal' : garmin ? 'Garmin' : 'Raring estimate · need before wake'}</small></dd></div>
    <div><dt><i style:background="var(--light)"></i>Need</dt><dd class="n">{duration(plan.needMin)} asleep<small>Garmin{plan.needReadAt ? ` · read ${readAt(plan.needReadAt, now)}` : ''}</small><small>so about {duration(plan.inBedMin || plan.needMin)} in bed with {plan.fallAsleepMin} min to fall asleep (Raring)</small></dd></div>
    {#if plan.wakeTime && plan.wakeTime !== up}<div><dt>Usual wake</dt><dd class="n">{plan.wakeTime}<small>{wakeLabel(plan.wakeNights)}</small></dd></div>{/if}
   </dl>
   <label class="setting fall"><span>Time to fall asleep<small>Raring’s; in bed this long before asleep-by</small></span><select value={fallAsleep} onchange={e => setFallAsleep(Number(e.currentTarget.value))}>{#each Array.from({length: 13}, (_, i) => i * 5) as m}<option value={m}>{m} min</option>{/each}</select></label>
  </div>
  <p class="small">{plan.stale ? 'No sleep has synced today; this estimate uses older nights.' : passed ? 'The window has passed. Rest when you’re ready.' : plan.reason ?? (garmin ? 'Garmin’s bedtime window and sleep need for tonight.' : 'Your usual wake minus Garmin’s sleep need.')}</p>
  <div class="btns">
   <button class="secondary" aria-pressed={sleep.woundDown} disabled={sleep.woundDown || !sleep.windDownOpen} onclick={() => act({action: 'reportWindDown'})}>{#if sleep.woundDown}<Icon name="check" size={16}/>Wound down · self-reported{:else if sleep.windDownOpen}I wound down{:else}I wound down · from the evening{/if}</button>
   <label class="setting reminder"><span>Remind me to wind down</span><input type="checkbox" checked={reminder} onchange={e => setReminder(e.currentTarget.checked)}/></label>
  </div>
  <button class="why" aria-expanded={why} onclick={() => why = !why}>Why this plan</button>
  {#if why}<SleepPlanEvidence {plan}/>{/if}
 </Card>
{:else}
 <Card title="Tonight"><p class="small">No plan for tonight yet: Raring needs Garmin’s sleep need and a few recorded wake times.</p></Card>
{/if}
<Card title="Sleep goals this week" extra="tonight-goals">
 {#if timing && goals}
  <p class="goal"><b>{timing.observation ? 'Asleep on time · observation' : `${goals.sleepKind === 'windDown' ? 'Wind down by' : 'Asleep by'} ${goals.sleepTime ?? '—'}`}</b> · {timing.recorded ?? '—'} of {timing.target} nights{timing.unavailable ? ` · ${timing.unavailable}` : ''}{placed ? ' · tonight is one' : ''}</p>
  {#if canPlace}<button class="secondary" onclick={placeTonight}>Make tonight one of them</button>{/if}
 {/if}
 {#if board?.sleepScore}<p class="goal"><b>Average Sleep Score {board.sleepScore.average ?? '—'}</b> · target {board.sleepScore.target} · Garmin’s score over {board.sleepScore.nights} night{board.sleepScore.nights === 1 ? '' : 's'}</p>{/if}
 {#if !timing && !board?.sleepScore}<p class="small">No sleep goal this week.</p>{/if}
 <button class="link" onclick={() => navigate('plan/commitments')}>{timing || board?.sleepScore ? 'Change goals' : 'Choose a sleep goal'}</button>
</Card>
<Card title="Recent nights vs tonight’s window" extra="tonight-nights">
 <RecentNights date={sleep.date} windows={sleep.consistency} {plan} {navigate}/>
 {#if recorded.length}<p class="small">{below.length} of {recorded.length} recorded nights in the last 7 were below Garmin’s need for that night.</p>{/if}
</Card>
<style>
 .tn{display:grid;gap:14px;margin:4px 0 6px}
 /* Each time wraps under its name rather than leaving the card at large text. */
 .facts{display:grid;gap:6px;margin:0;min-width:0;font-size:13px}.facts>div{display:flex;flex-wrap:wrap;justify-content:space-between;align-items:baseline;gap:2px 10px}
 .facts dt{display:flex;gap:6px;align-items:center;color:var(--text2);font-weight:600}.facts dt i{width:8px;height:8px;border-radius:50%}
 .facts dd{margin:0 0 0 auto;text-align:right;font-weight:600;font-size:15px;white-space:nowrap}.facts dd small{display:block;font:500 12px Inter;color:var(--text2);white-space:normal}
 .btns{display:flex;gap:8px;flex-wrap:wrap;align-items:center;margin-top:8px}.btns .secondary{width:auto;flex:1 1 160px;margin:0}
 .link{background:none;border:0;color:var(--ink);font:600 13px Inter;min-height:44px;padding:0 4px}
 .reminder{flex:1 1 100%;gap:12px;min-height:44px;padding:8px 0 0;font-size:14px;color:var(--text)}
 .fall{gap:12px;padding:10px 0 0;font-size:14px;color:var(--text)}.fall small{display:block;font:500 12px Inter;color:var(--text2)}.fall select{flex:none}
 .why{background:none;border:0;color:var(--text2);font:600 12px Inter;min-height:40px;padding:0}
 .goal{margin:4px 0;font-size:14px}
</style>
