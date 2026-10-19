<script lang="ts">
 import Card from './Card.svelte';
 import Icon from './Icon.svelte';
 import TonightDial from './TonightDial.svelte';
 import SleepPlanEvidence from './SleepPlanEvidence.svelte';
 import type {SleepView} from '../generated/SleepView';
 import type {PlanBoard} from '../generated/PlanBoard';
 import type {LocalAction} from '../generated/LocalAction';
 import {clock, dateLabel, duration} from '../format';
 // Tonight as a plan: when to wind down and be asleep from Garmin's window or
 // need and the usual wake, the week's sleep goal, and recent nights against
 // Garmin's need as plain facts. No sleep debt is computed.
 let {sleep, board, act, reminder, setReminder, navigate, now = new Date()}: {sleep: SleepView, board: PlanBoard | null, act: (a: LocalAction) => void,
  reminder: boolean, setReminder: (on: boolean) => void, navigate: (route: string) => void, now?: Date} = $props();
 let plan = $derived(sleep.sleepPlan);
 let garmin = $derived(plan?.bedtimeSource === 'garmin');
 let why = $state(false);
 let hhmm = $derived(`${String(now.getHours()).padStart(2, '0')}:${String(now.getMinutes()).padStart(2, '0')}`);
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
 let top = $derived(Math.max(9 * 60, ...nights.flatMap(n => [n.asleepMin ?? 0, n.needMin ?? 0])) + 30);
 const h = (m: number) => `${Math.floor(m / 60)}:${String(Math.round(m % 60)).padStart(2, '0')}`;
</script>
{#if plan}
 <Card title="Tonight" extra="tonight-plan" help="sleep-tonight">
  {#snippet aside()}<span class="pill"><Icon name="moon" size={13}/> {dateLabel(plan.night)} · {garmin ? 'Garmin plan' : 'Raring estimate'}</span>{/snippet}
  <div class="tn">
   <TonightDial now={hhmm} windDown={plan.windDown} windowStart={plan.bedtimeStart ?? plan.bedtime} windowEnd={plan.bedtime} estimate={!garmin} wake={plan.wakeTime}
    lastBed={sleep.startLocal ? clock(sleep.startLocal) : null} lastWake={sleep.endLocal ? clock(sleep.endLocal) : null} need={duration(plan.needMin)} size={124}/>
   <dl class="facts">
    <dt><i style:background="#FFB547"></i>Wind down</dt><dd class="n">{ownTime && goals?.sleepKind === 'windDown' ? ownTime : plan.windDown}<small>{ownTime && goals?.sleepKind === 'windDown' ? 'your goal' : 'suggested'}</small></dd>
    <dt><i style:background="#A9C8FF"></i>{garmin ? 'Bedtime' : 'Asleep by'}</dt><dd class="n">{ownTime && goals?.sleepKind === 'asleepBy' ? ownTime : plan.bedtimeStart ? `${plan.bedtimeStart}–${plan.bedtime}` : plan.bedtime}<small>{ownTime && goals?.sleepKind === 'asleepBy' ? 'your goal' : garmin ? 'Garmin window' : 'need before wake'}</small></dd>
    {#if plan.wakeTime}<dt><i style:background="#EEF1F7"></i>Wake</dt><dd class="n">{plan.wakeTime}<small>{plan.wakeNights === 14 ? 'usual' : 'reference'}</small></dd>{/if}
    <dt>Need</dt><dd class="n">{duration(plan.needMin)}<small>Garmin{plan.needReadAt ? ` · read ${new Date(plan.needReadAt).toLocaleTimeString('en-GB', {hour: '2-digit', minute: '2-digit'})}` : ''}</small></dd>
   </dl>
  </div>
  <p class="small">{plan.stale ? 'No sleep has synced today; this estimate uses older nights.' : passed ? 'The window has passed. Rest when you’re ready.' : plan.reason ?? (garmin ? 'Garmin’s bedtime window and sleep need for tonight.' : 'Your usual wake minus Garmin’s sleep need.')}</p>
  <div class="btns">
   <button class="secondary" aria-pressed={sleep.woundDown} disabled={sleep.woundDown} onclick={() => act({action: 'reportWindDown'})}>{#if sleep.woundDown}<Icon name="check" size={16}/>Wound down · self-reported{:else}I wound down{/if}</button>
   <button class="link" aria-pressed={reminder} onclick={() => setReminder(!reminder)}>{reminder ? 'Wind-down reminder on' : 'Remind me to wind down'}</button>
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
 <div class="nights" role="img" aria-label={`Hours asleep, last 7 nights: ${nights.map(n => `${dateLabel(n.date)} ${n.asleepMin != null ? h(n.asleepMin) : 'not recorded'}${n.needMin != null ? `, need ${h(n.needMin)}` : ''}`).join('; ')}`}>
  <span class="yl"><i>{Math.round(top / 60)} h</i><i>0</i></span>
  {#each nights as n}
   <div class="col">
    <div class="bar-area">
     {#if n.asleepMin != null}<b class="bar" class:below={n.needMin != null && n.asleepMin < n.needMin} style:height={n.asleepMin / top * 100 + '%'}><em class="n">{h(n.asleepMin)}</em></b>{:else}<span class="gap">—</span>{/if}
     {#if n.needMin != null}<u style:bottom={n.needMin / top * 100 + '%'}></u>{/if}
    </div>
    <small>{new Date(n.date + 'T12:00:00Z').toLocaleDateString('en-GB', {weekday: 'short', timeZone: 'UTC'})}</small>
   </div>
  {/each}
 </div>
 <p class="key"><span><i class="k-bar"></i>Asleep</span><span><i class="k-need"></i>Garmin’s need that night</span></p>
 <p class="small">{recorded.length ? `${below.length} of ${recorded.length} recorded nights below Garmin’s need.` : 'No recorded nights this week.'} No running debt is computed.</p>
</Card>
<style>
 .tn{display:flex;gap:14px;align-items:center;margin:4px 0 6px}
 .facts{display:grid;grid-template-columns:auto 1fr;gap:6px 10px;margin:0;flex:1;min-width:0;font-size:13px}
 .facts dt{display:flex;gap:6px;align-items:center;color:var(--text2);font-weight:600}.facts dt i{width:8px;height:8px;border-radius:50%}
 .facts dd{margin:0;text-align:right;font-weight:600;font-size:15px}.facts dd small{display:block;font:500 10.5px Inter;color:var(--text2)}
 .btns{display:flex;gap:8px;flex-wrap:wrap;align-items:center;margin-top:8px}.btns .secondary{width:auto;flex:1 1 160px;margin:0}
 .link{background:none;border:0;color:var(--ink);font:600 13px Inter;min-height:44px;padding:0 4px}
 .why{background:none;border:0;color:var(--text2);font:600 12px Inter;min-height:40px;padding:0}
 .goal{margin:4px 0;font-size:14px}
 .nights{position:relative;display:grid;grid-template-columns:28px repeat(7,minmax(0,1fr));gap:4px;height:150px;align-items:end;padding-bottom:18px}
 .yl{position:relative;height:100%;display:flex;flex-direction:column;justify-content:space-between;font:500 10px Inter;color:var(--text2);padding-bottom:0}.yl i{font-style:normal}
 .col{position:relative;height:100%;display:flex;flex-direction:column;justify-content:flex-end;align-items:center}
 .bar-area{position:relative;width:100%;height:100%;display:flex;align-items:flex-end;justify-content:center;border-bottom:1px solid var(--line)}
 .bar{position:relative;display:block;width:70%;max-width:30px;border-radius:5px 5px 0 0;background:#5C9BFF}.bar.below{background:#3E6FC2}
 .bar em{position:absolute;bottom:100%;left:50%;transform:translateX(-50%);font:600 10px Inter;font-style:normal;color:var(--text);white-space:nowrap;margin-bottom:2px}
 .bar-area u{position:absolute;left:8%;right:8%;height:0;border-top:2px dashed #EEF1F7;opacity:.8}
 .gap{color:var(--text2);font-size:12px;margin-bottom:4px}
 .col small{position:absolute;bottom:-18px;font:500 10.5px Inter;color:var(--text2)}
 .key{display:flex;gap:14px;font-size:11px;color:var(--text2);margin:8px 0 0}.key span{display:inline-flex;gap:5px;align-items:center}
 .k-bar{width:10px;height:8px;border-radius:2px;background:#5C9BFF}.k-need{width:12px;border-top:2px dashed #EEF1F7}
 @container(max-width:330px){.tn{flex-direction:column;align-items:stretch}.tn :global(.dial){align-self:center}}
</style>
