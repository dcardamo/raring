<script lang="ts">
 import Icon from '../../lib/components/Icon.svelte';
 import Hypnogram from '../../lib/components/Hypnogram.svelte';
 import {batteryColor, toneColor} from '../../lib/charts';
 import RootHeader from '../components/RootHeader.svelte';
 import Hero from '../components/Hero.svelte';
 import DayRibbon from '../components/DayRibbon.svelte';
 import ActivityTile from '../components/ActivityTile.svelte';
 import WeekStrip from '../components/WeekStrip.svelte';
 import Ring from '../components/Ring.svelte';
 import Glyph from '../components/Glyph.svelte';
 import {store} from '../lib/store.svelte';
 import {A} from '../lib/actions';
 import {dayLabel, dur} from '../lib/dates';
 import {SPORTS} from '../lib/sports';
 import {stagesFor} from '../lib/world';
 import {todayView} from '../lib/views';
 let ctx = $derived(store.ctx);
 const go = (r: string) => store.navigate(r);
 let lead = $derived(ctx.lead);
 // The hero already shows the lead activity; previews list the rest, two at most.
 let acts = $derived(ctx.actsToday.slice().reverse());
 let shown = $derived(acts.filter(a => a.id !== lead.activity?.id).slice(0, 2));
 let waiting = $derived(ctx.phase === 'waiting' || ctx.stale);
 let earlier = $derived(ctx.phase === 'waiting' ? ctx.world.activities.filter(a => a.date === ctx.displayedDay) : []);
 let n = $derived(ctx.night);
 let heroSleep = $derived(lead.kind === 'sleep');
 let vitals = $derived(waiting ? null : todayView(ctx).vitals);
 let off = $derived(vitals?.readings.filter(v => v.status && v.status !== 'IN_RANGE').length ?? 0);
</script>

<RootHeader title="Today" adapt/>
{#if ctx.notices.length}
 <div class="notice-strip" role="status"><Icon name="alert" size={18}/><p>{ctx.notices[0].text}{#if ctx.notices.length > 1} · {ctx.notices.length - 1} more{/if}</p><button class="link inline" onclick={() => ctx.notices.length > 1 ? store.openSheet('notices') : go(ctx.notices[0].action.go)}>{ctx.notices.length > 1 ? 'View all' : ctx.notices[0].action.label}</button></div>
{/if}
{#if ctx.state.dst}<p class="small dst"><Icon name="clock" size={13}/> Clocks changed by an hour overnight</p>{/if}

<Hero {ctx} navigate={go} openSheet={(n, a) => store.openSheet(n, a)} dismiss={A.dismiss}/>

{#if !ctx.stale}<DayRibbon {ctx} navigate={go}/>{/if}

{#if shown.length || ctx.napsToday.length || ctx.lateArrivals.length || earlier.length}
 <div class="sec-h"><h2>{ctx.calendarDate === ctx.displayedDay ? 'Today’s activities' : `${dayLabel(ctx.calendarDate)}`}</h2>{#if acts.length > 2}<button class="link" onclick={() => { go('activities'); setTimeout(() => store.setUi({date: ctx.calendarDate}), 50); }}>All today’s activities ({acts.length})</button>{/if}</div>
 {#each shown as a (a.id)}<ActivityTile {a} {ctx} navigate={go}/>{/each}
 {#each ctx.napsToday as nap}<button class="nap" onclick={() => go('sleep/naps')}><span class="napico"><Icon name="moon" size={18}/></span><span><b>Nap · {nap.start[1]}{nap.end ? `–${nap.end[1]}` : ''}</b><small>{nap.min} min · separate from overnight sleep</small></span><Icon name="chevron" size={16}/></button>{/each}
 {#each earlier as a (a.id)}<ActivityTile {a} {ctx} navigate={go} showDate/>{/each}
 {#if ctx.lateArrivals.length}<button class="late" onclick={() => go(`workout/${ctx.lateArrivals[0].id}`)}><Glyph sport={ctx.lateArrivals[0].sport} size={24}/><span><b>Arrived late · {SPORTS[ctx.lateArrivals[0].sport].name}</b><small>Counts on {dayLabel(ctx.lateArrivals[0].date)}</small></span><Icon name="chevron" size={16}/></button>{/if}
{/if}

<div class="tiles">
 {#if !heroSleep}
  <button class="tile" onclick={() => go('sleep')} aria-label={n && !waiting ? `Sleep ${dur(n.asleep)}, need ${dur(n.need)}` : 'Sleep: waiting'}>
   <small>Sleep</small>
   {#if n && !waiting}<b class="n">{dur(n.asleep)}</b><span class="mini-hyp"><Hypnogram segments={stagesFor(ctx.displayedDay, n.asleep)} startLocal={null}/></span><em>{n.asleep >= n.need ? 'Met need' : `${dur(n.need - n.asleep)} under need`}</em>
   {:else}<b class="n">—</b><em>{waiting ? 'Syncing' : 'No night'}</em>{/if}
  </button>
 {:else}
  <button class="tile" onclick={() => go('readiness')}><small>Readiness</small><b class="n">{ctx.readiness?.value ?? '—'}</b><em>{ctx.world.readinessSupported ? (ctx.readiness?.level ?? 'Waiting') : 'Not on this watch'}</em></button>
 {/if}
 <button class="tile" onclick={() => go('metric/bodyBattery')} aria-label={ctx.bodyBattery != null ? `Body Battery ${ctx.bodyBattery}` : 'Body Battery unavailable'}>
  <small>Body Battery</small>
  <span class="bbrow"><span class="batt"><i style:height={(ctx.bodyBattery ?? 0) + '%'} style:background={ctx.bodyBattery != null ? batteryColor(ctx.bodyBattery) : 'transparent'}></i></span><b class="n">{ctx.bodyBattery ?? '—'}</b></span>
  <em>{ctx.bodyBattery != null ? `At ${ctx.upload.at}` : 'Waiting'}</em>
 </button>
 <button class="tile" onclick={() => go('health')} aria-label={vitals ? `Overnight vitals: ${off ? `${off} outside usual range` : 'all within usual range'}` : 'Vitals waiting'}>
  <small>Vitals</small>
  <span class="dots" aria-hidden="true">{#each vitals?.readings ?? Array(5).fill(null) as v}<i class:off={v?.status && v.status !== 'IN_RANGE'} class:none={!v || v.value == null} style:background={v && v.value != null ? toneColor(v.tone) : undefined}></i>{/each}</span>
  <em>{vitals ? (off ? `${off} outside range` : 'Usual range') : 'Waiting'}</em>
 </button>
</div>

<div class="sec-h"><h2>This week</h2><button class="link" onclick={() => go('plan')}>Plan</button></div>
<section class="card week-card">
 <WeekStrip {ctx}/>
 {#if ctx.plan}
  <div class="wk-rings">
   {#each ctx.plan.tracks as t}<button class="wk-ring" onclick={() => go('plan')}><Ring value={t.have} target={t.target} size={44} stroke={5} state={t.state} color={t.key === 'strength' ? SPORTS.strength.color : '#36C9C6'} label={`${t.key === 'strength' ? 'Strength sessions' : 'Intensity minutes'}: ${t.label}`}>{#if t.state !== 'reached' && t.state !== 'reached-adjusted'}<b class="n rn">{t.key === 'im' ? Math.round(t.have / Math.max(1, t.target) * 100) + '%' : t.have}</b>{/if}</Ring><span><b>{t.key === 'strength' ? 'Strength' : 'Intensity'}</b><small>{t.label}{t.state === 'open' ? ` · ${ctx.plan.daysLeft}d left` : ''}</small></span></button>{/each}
  </div>
 {/if}
</section>

{#if lead.kind !== 'sleep' || !['evening', 'winddown', 'late'].includes(ctx.phase)}
 <button class="ahead" onclick={() => go('sleep/tonight')}><Icon name="moon" size={18}/><span>{waiting && ctx.stale ? 'Tonight · asleep by 23:20 (estimate)' : waiting ? 'Tonight’s plan after last night syncs' : `Tonight · wind down ${ctx.tonight.windDown} · window ${ctx.tonight.window}`}</span><Icon name="chevron" size={16}/></button>
{/if}
<p class="end-note">That’s today</p>

<style>
 .dst{display:flex;gap:6px;align-items:center;margin:8px 4px 0}
 .nap,.late,.ahead{display:grid;grid-template-columns:auto 1fr auto;gap:12px;align-items:center;width:100%;margin-top:10px;padding:12px 14px;border-radius:18px;background:var(--card);border:1px solid var(--line);color:var(--text);text-align:left}
 .nap b,.late b{display:block}
 .nap small,.late small{display:block;font-size:12px;color:var(--text2)}
 .napico{width:28px;height:28px;border-radius:50%;display:grid;place-items:center;background:rgba(169,200,255,.15);color:#A9C8FF}
 .ahead{color:var(--text2);font-size:13px;margin-top:12px}
 .ahead :global(svg:first-child){color:#A9C8FF}
 .mini-hyp{display:block;margin:2px 0 -2px;height:30px;overflow:hidden}
 .mini-hyp :global(text){display:none}
 .mini-hyp :global(.plot){height:34px;width:100%}
 .bbrow{display:flex;align-items:center;gap:8px;margin-top:4px}
 .bbrow b{margin-top:0}
 .batt{width:16px;height:28px;border:2px solid rgba(228,232,240,.5);border-radius:4px;position:relative;display:flex;align-items:flex-end;padding:2px}
 .batt i{display:block;width:100%;border-radius:1.5px}
 .dots{display:flex;gap:5px;margin:12px 0 9px}
 .dots i{width:11px;height:11px;border-radius:50%;opacity:.9}
 .dots i.off{outline:2px solid #FFB547;outline-offset:1px}
 .dots i.none{background:none;border:1.5px dashed #8E98AC}
 .week-card{padding:12px 10px 12px}
 .wk-rings{display:grid;grid-template-columns:1fr 1fr;gap:10px;margin-top:10px;padding-top:12px;border-top:1px solid var(--line)}
 .wk-ring{display:flex;align-items:center;gap:10px;background:none;border:0;color:var(--text);text-align:left;padding:2px 4px}
 .wk-ring b{display:block;font-weight:600;font-size:14px}
 .wk-ring small{display:block;color:var(--text2);font-size:12px}
 .rn{font-size:12px}
</style>
