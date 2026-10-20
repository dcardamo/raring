<script lang="ts">
 import Card from './Card.svelte';
 import Icon from './Icon.svelte';
 import TonightLine from './TonightLine.svelte';
 import SleepPlanEvidence from './SleepPlanEvidence.svelte';
 import {declutter} from '../declutter';
 import type {SleepView} from '../generated/SleepView';
 import type {PlanBoard} from '../generated/PlanBoard';
 import type {LocalAction} from '../generated/LocalAction';
 import {dateLabel, duration, readAt} from '../format';
 import {wakeLabel} from '../tonight';
 // Tonight as a plan: when to wind down and be asleep from Garmin's window or
 // need and the usual wake, the week's sleep goal, and recent nights against
 // Garmin's need as plain facts. No sleep debt is computed.
 let {sleep, board, act, reminder, setReminder, navigate, now = new Date()}: {sleep: SleepView, board: PlanBoard | null, act: (a: LocalAction) => void,
  reminder: boolean, setReminder: (on: boolean) => void, navigate: (route: string) => void, now?: Date} = $props();
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
 let recorded = $derived(nights.filter(n => n.asleepMin != null));
 let below = $derived(recorded.filter(n => n.needMin != null && n.asleepMin! < n.needMin));
 // A whole number of hours, even so the gridlines fall every two hours.
 let topH = $derived(Math.ceil(Math.max(9 * 60, ...nights.flatMap(n => [n.asleepMin ?? 0, n.needMin ?? 0])) / 120) * 2);
 let top = $derived(topH * 60);
 const h = (m: number) => `${Math.floor(m / 60)}:${String(Math.round(m % 60)).padStart(2, '0')}`;
</script>
{#if plan}
 <Card title="Tonight" extra="tonight-plan" help="sleep-tonight">
  {#snippet aside()}<span class="pill"><Icon name="moon" size={13}/> {dateLabel(plan.night)} · {garmin ? 'Garmin plan' : 'Raring estimate'}</span>{/snippet}
  <div class="tn">
   <TonightLine {plan} {now} {garmin}/>
   <dl class="facts">
    <div><dt><i style:background="var(--fair)"></i>Wind down</dt><dd class="n">{ownTime && goals?.sleepKind === 'windDown' ? ownTime : plan.windDown}<small>{ownTime && goals?.sleepKind === 'windDown' ? 'your goal' : 'suggested'}</small></dd></div>
    <div><dt><i style:background="var(--goal-sleep)"></i>{garmin ? 'Bedtime' : 'Asleep by'}</dt><dd class="n">{ownTime && goals?.sleepKind === 'asleepBy' ? ownTime : plan.bedtimeStart ? `${plan.bedtimeStart}–${plan.bedtime}` : plan.bedtime}<small>{ownTime && goals?.sleepKind === 'asleepBy' ? 'your goal' : garmin ? 'Garmin window' : 'need before wake'}</small></dd></div>
    {#if plan.wakeTime}<div><dt><i style:background="var(--text)"></i>Wake</dt><dd class="n">{plan.wakeTime}<small>{wakeLabel(plan.wakeNights)}</small></dd></div>{/if}
    <div><dt>Need</dt><dd class="n">{duration(plan.needMin)}<small>Garmin{plan.needReadAt ? ` · read ${readAt(plan.needReadAt, now)}` : ''}</small></dd></div>
   </dl>
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
<Card title="Last 7 nights against need" extra="tonight-nights">
 {#if recorded.length}
 <!-- A group, not an image: each night is a button assistive tech can reach, labelled with its hours. -->
 <div class="nights" data-chart="nights" {@attach declutter} role="group" aria-label="Hours asleep, last 7 nights, against Garmin’s need">
  <span class="yl" aria-hidden="true">{#each Array.from({length: topH / 2 + 1}, (_, i) => i * 2) as t}<i data-thin={t && t < topH ? 2 : 3} style:bottom={t / topH * 100 + '%'}>{t ? `${t} h` : '0'}</i>{/each}</span>
  {#each nights as n}
   <button class="col" aria-label={`Open ${dateLabel(n.date)}: ${n.asleepMin != null ? `${h(n.asleepMin)} asleep` : 'not recorded'}${n.needMin != null ? `, need ${h(n.needMin)}` : ''}`} onclick={() => navigate('sleep/' + n.date)}>
    <div class="bar-area" style:--step={200 / topH + '%'}>
     {#if n.asleepMin != null}<b class="night-bar" class:below={n.needMin != null && n.asleepMin < n.needMin} style:height={n.asleepMin / top * 100 + '%'}></b>{:else}<span class="gap"></span>{/if}
     {#if n.needMin != null}<u style:bottom={n.needMin / top * 100 + '%'}></u>{/if}
    </div>
    <small><span data-thin="1">{new Date(n.date + 'T12:00:00Z').toLocaleDateString('en-GB', {weekday: 'short', timeZone: 'UTC'})}</span>{#if n.asleepMin != null}<span class="n" data-thin>{h(n.asleepMin)}</span>{/if}</small>
   </button>
  {/each}
 </div>
 <p class="key"><span><i class="k-bar"></i>Asleep</span>{#if below.length}<span><i class="k-bar below"></i>Below need</span>{/if}<span><i class="k-need"></i>Garmin’s need that night</span>{#if recorded.length<nights.length}<span><i class="k-gap"></i>Not recorded</span>{/if}</p>
 <p class="small">{below.length} of {recorded.length} recorded nights below Garmin’s need. Open a night for its stages.</p>
 {:else}<p class="small">No recorded nights in the last 7.</p>{/if}
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
 .why{background:none;border:0;color:var(--text2);font:600 12px Inter;min-height:40px;padding:0}
 .goal{margin:4px 0;font-size:14px}
 .nights{position:relative;display:grid;grid-template-columns:auto repeat(7,minmax(0,1fr));gap:4px;height:150px;align-items:end;font-size:11px;padding-bottom:calc(2.6em + 6px)}
 .yl{position:relative;height:100%;min-width:2.6em;font:500 11px Inter;color:var(--text2)}.yl i{position:absolute;right:2px;transform:translateY(50%);font-style:normal;white-space:nowrap}
 .col{position:relative;height:100%;display:flex;flex-direction:column;justify-content:flex-end;align-items:center;border:0;padding:0;background:none;color:inherit;font:inherit;cursor:pointer}
 .bar-area{position:relative;width:100%;height:100%;display:flex;align-items:flex-end;justify-content:center;border-bottom:1px solid var(--line);background:repeating-linear-gradient(to top,transparent 0 calc(var(--step) - 1px),rgba(255,255,255,.07) calc(var(--step) - 1px) var(--step))}
 .night-bar{position:relative;display:block;width:70%;max-width:30px;border-radius:5px 5px 0 0;background:var(--light)}.night-bar.below{background:#3E6FC2}
 .bar-area u{position:absolute;left:8%;right:8%;height:0;border-top:2px dashed var(--text);opacity:.8}
 .gap,.k-gap{display:block;border:1.5px dotted var(--text3);border-radius:4px}.gap{width:70%;max-width:30px;height:24px}
 .col small{position:absolute;top:100%;margin-top:3px;display:grid;justify-items:center;line-height:1.3;font:500 11px Inter;color:var(--text2)}.col small .n{font:600 12px Outfit,Inter;color:var(--text)}
 .key{display:flex;flex-wrap:wrap;gap:4px 14px;font-size:12px;color:var(--text2);margin:8px 0 0}.key span{display:inline-flex;gap:5px;align-items:center}
 .k-bar{width:10px;height:8px;border-radius:2px;background:var(--light)}.k-bar.below{background:#3E6FC2}.k-gap{width:10px;height:8px}.k-need{width:12px;border-top:2px dashed var(--text)}
</style>
