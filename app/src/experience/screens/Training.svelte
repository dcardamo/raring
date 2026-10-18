<script lang="ts">
 import TrainingDetail from '../../lib/screens/TrainingDetail.svelte';
 import StrengthHistory from '../../lib/components/StrengthHistory.svelte';
 import BodyMap from '../../lib/components/BodyMap.svelte';
 import Icon from '../../lib/components/Icon.svelte';
 import RootHeader from '../components/RootHeader.svelte';
 import ActivityTile from '../components/ActivityTile.svelte';
 import Glyph from '../components/Glyph.svelte';
 import {store} from '../lib/store.svelte';
 import {trainingView, strengthView, musclesBetween} from '../lib/views';
 import {recordedFor} from '../lib/allocation';
 import {SPORTS, type Sport} from '../lib/sports';
 import {addDays, between, dayMonth, narrowDay, weekRange} from '../lib/dates';
 // Training is fitness and strength together: one week view, then focused sections.
 let ctx = $derived(store.ctx);
 const go = (r: string) => store.navigate(r);
 let tab = $derived((store.ui.section as string) ?? 'fitness');
 let week = $derived(ctx.world.activities.filter(a => between(a.date, ctx.weekFrom, ctx.calendarDate)));
 let days = $derived(Array.from({length: 7}, (_, i) => addDays(ctx.weekFrom, i)));
 let max = $derived(Math.max(60, ...days.map(d => week.filter(a => a.date === d).reduce((t, a) => t + a.min, 0))));
 let bySport = $derived.by(() => { const m = new Map<Sport, number>(); for (const a of week) m.set(a.sport, (m.get(a.sport) ?? 0) + a.min); return [...m.entries()].sort((a, b) => b[1] - a[1]); });
 let sv = $derived(strengthView(ctx));
 let all = $derived(ctx.world.activities.slice().reverse());
 let filter = $derived((store.ui.filter as string) ?? 'all');
 let list = $derived(all.filter(a => filter === 'all' || a.sport === filter).slice(0, (store.ui.limit as number) ?? 12));
 let sports = $derived([...new Set(all.map(a => a.sport))]);
</script>
<RootHeader title="Training"/>

<section class="tw" aria-label="This week's training">
 <div class="tw-h"><div><small>This week · {weekRange(ctx.weekFrom)}</small><b class="n">{week.length}<span> sessions</span></b></div>
  <div class="tw-k"><span><b class="n">{Math.round(week.reduce((t, a) => t + a.min, 0) / 6) / 10}</b>h</span><span><b class="n">{recordedFor(week, 'zone2')}</b>Zone 2 min</span><span><b class="n">{recordedFor(week, 'strength')}</b>strength</span></div></div>
 <div class="tw-bars" role="img" aria-label={`Minutes per day: ${days.map(d => week.filter(a => a.date === d).reduce((t, a) => t + a.min, 0)).join(', ')}`}>
  {#each days as d}{@const acts = week.filter(a => a.date === d)}
   <button class="tb" class:future={d > ctx.calendarDate} class:today={d === ctx.calendarDate} onclick={() => store.openSheet('day', d)} aria-label={`${dayMonth(d)}: ${acts.map(a => a.name).join(', ') || (d > ctx.calendarDate ? 'future' : 'nothing recorded')}`}>
    <span class="stack">{#each acts as a}<i style:height={a.min / max * 100 + '%'} style:background={SPORTS[a.sport].color}></i>{/each}</span><small>{narrowDay(d)}</small>
   </button>
  {/each}
 </div>
 {#if bySport.length}<div class="tw-mix">{#each bySport as [s, m]}<span><Glyph sport={s} size={18}/>{SPORTS[s].name}<b class="n">{m} min</b></span>{/each}</div>{/if}
</section>

<div class="seg" role="tablist" aria-label="Training section">
 {#each [['fitness', 'Fitness'], ['strength', 'Strength'], ['activities', 'Activities']] as [k, l]}<button role="tab" aria-selected={tab === k} onclick={() => store.setUi({section: k})}>{l}</button>{/each}
</div>

{#if tab === 'fitness'}
 {#snippet none()}{/snippet}
 <div class="fitness"><TrainingDetail trainingData={trainingView(ctx)} header={none} navigate={go}/></div>
 <section class="card"><button class="rowlink" onclick={() => go('help/acute-load')}><Icon name="info" size={18}/><span><b>What acute load is</b><small>Context, not a budget</small></span><Icon name="chevron" size={16}/></button><button class="rowlink" onclick={() => go('training/performance')}><Icon name="flame" size={18}/><span><b>Performance</b><small>Thresholds and predictions · Garmin dates</small></span><Icon name="chevron" size={16}/></button><button class="rowlink" onclick={() => go('gear')}><Icon name="run" size={18}/><span><b>Gear</b><small>Garmin distances</small></span><Icon name="chevron" size={16}/></button></section>
{:else if tab === 'strength'}
 {#if sv}
  <section class="card st-sum"><BodyMap figure="female" muscles={musclesBetween(ctx, ctx.weekFrom, ctx.calendarDate)} width={110} max={12} legend={false}/><div><small>This week</small><b class="n">{recordedFor(week, 'strength')}<span> sessions</span></b><small>{sv.thisWeek.reduce((t, m) => t + m.sets, 0)} sets by muscle</small><button class="link" onclick={() => go('strength/muscles')}>Muscles over time</button></div></section>
  <StrengthHistory view={sv} navigate={go}/>
 {:else}<section class="card"><p>No strength sessions in the last 12 weeks.</p><button class="link" onclick={() => go('help/strength-start')}>Lift with your watch</button></section>{/if}
{:else}
 <div class="chiprow filt" role="group" aria-label="Filter by sport">{#each ['all', ...sports] as s}<button class="chipb" aria-pressed={filter === s} onclick={() => store.setUi({filter: s, limit: 12})}>{#if s !== 'all'}<Glyph sport={s as Sport} size={20}/>{/if}{s === 'all' ? 'All' : SPORTS[s as Sport].name}</button>{/each}</div>
 {#each list as a (a.id)}<ActivityTile {a} {ctx} navigate={go} showDate/>{/each}
 <button class="secondary" onclick={() => go('activities')}>All activities</button>
{/if}

<style>
 .tw{margin-top:12px;padding:16px;border-radius:24px;background:linear-gradient(160deg,rgba(54,201,198,.1),var(--card) 60%);border:1px solid var(--line)}
 .tw-h{display:flex;justify-content:space-between;align-items:flex-end;gap:10px;flex-wrap:wrap}.tw-h small{font-size:11px;color:var(--text3)}.tw-h b{display:block;font-size:32px;font-weight:600}.tw-h b span{font:500 13px Inter;color:var(--text3);margin-left:4px}
 .tw-k{display:flex;gap:14px}.tw-k span{font-size:11px;color:var(--text2);display:flex;flex-direction:column;align-items:flex-end}.tw-k b{font-size:18px;color:var(--text)}
 .tw-bars{display:flex;gap:6px;height:96px;margin-top:14px}.tb{flex:1;display:flex;flex-direction:column;align-items:center;gap:4px;background:none;border:0;padding:0;color:var(--text3);min-width:0}.stack{flex:1;width:100%;max-width:30px;display:flex;flex-direction:column-reverse;gap:1px;border-radius:6px;overflow:hidden;background:rgba(255,255,255,.04)}.stack i{display:block;width:100%}.tb small{font:600 10px Inter}.tb.today small{color:var(--text)}.tb.future .stack{background:none;border:1px dashed rgba(255,255,255,.08)}
 .tw-mix{display:flex;flex-wrap:wrap;gap:6px 14px;margin-top:12px;font-size:12px;color:var(--text2)}.tw-mix span{display:inline-flex;gap:6px;align-items:center}.tw-mix b{color:var(--text);font-weight:600}
 .seg{display:grid;grid-template-columns:repeat(3,1fr);gap:4px;padding:4px;border-radius:14px;background:rgba(255,255,255,.05);margin:14px 0 4px;position:sticky;top:0;z-index:3;backdrop-filter:blur(8px)}.seg button{height:38px;border-radius:10px;border:0;background:none;color:var(--text2);font:600 13px Inter}.seg button[aria-selected="true"]{background:var(--ink);color:var(--bg)}
 .fitness :global(.page-hero){margin-top:4px}
 .st-sum{display:flex;gap:16px;align-items:center}.st-sum small{display:block;font-size:12px;color:var(--text2)}.st-sum b{display:block;font-size:30px;font-weight:600;margin:4px 0}.st-sum b span{font:500 13px Inter;color:var(--text3);margin-left:4px}.filt{margin-top:10px}
</style>
