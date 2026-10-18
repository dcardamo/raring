<script lang="ts">
 import {untrack} from 'svelte';
 import {store} from '../lib/store.svelte';
 import {A} from '../lib/actions';
 import {goalsFor} from '../lib/allocation';
 import {backlog, planPrompt, proposeGoals, takeItem, weekOutcome} from '../lib/week';
 import {addDays, dayLabel, weekId, weekRange} from '../lib/dates';
 import GoalEditor from './GoalEditor.svelte';
 import DayCells from './DayCells.svelte';
 import WeekSummary from './WeekSummary.svelte';
 import type {GoalTargets} from '../lib/rules';
 // Review last week (when there is one) → choose goals → place them → done.
 // Cancel leaves everything as it was; the reminder can be turned off here.
 let ctx = $derived(store.ctx);
 const reviewFrom = untrack(() => planPrompt(store.ctx)?.review ?? null);
 let step = $state<'review' | 'goals' | 'place' | 'done'>(reviewFrom ? 'review' : 'goals');
 const proposals = untrack(() => reviewFrom ? proposeGoals(store.ctx, store.ctx.weekFrom) : []);
 let accepted = $state<Record<string, boolean>>({});
 let base = $derived<GoalTargets>({...goalsFor(ctx), ...Object.fromEntries(proposals.filter(p => accepted[p.key]).map(p => [p.key, p.proposed]))});
 let outcome = $derived(reviewFrom ? weekOutcome(ctx, reviewFrom) : null);
 let items = $derived(backlog(ctx).items);
 let days = $derived(Array.from({length: 7}, (_, i) => addDays(ctx.weekFrom, i)).filter(d => d >= ctx.calendarDate));
 const STATE = {met: 'Met', partly: 'Partly met', none: 'Not recorded', 'not-counted': 'Not counted'};
 const finish = () => { store.set(s => ({planWeeks: {...(s.planWeeks ?? {}), [weekId(ctx.calendarDate)]: 'done'}}), 'Week planned'); store.closeSheet(); };
</script>
<ol class="steps" aria-label="Steps">{#each (reviewFrom ? ['review', 'goals', 'place', 'done'] : ['goals', 'place', 'done']) as s, i}<li class:on={s === step}>{i + 1}. {({review: 'Last week', goals: 'Goals', place: 'Place', done: 'Done'} as Record<string, string>)[s]}</li>{/each}</ol>

{#if step === 'review' && outcome}
 <h3>{weekRange(outcome.from)}</h3>
 {#if outcome.reasons.length}<p class="why">{outcome.counted ? 'Also that week' : 'Not counted toward suggestions'}: {outcome.reasons.join(' · ')}</p>{/if}
 <ul class="outcome">{#each outcome.goals as o}<li class="o-{o.state}" style:--c={({strength: '#FF7DB8', zone2: '#5CA8FF', hi: '#FF8A5C', im: '#36C9C6', sleep: '#A9C8FF'} as Record<string, string>)[o.key]}><i></i><b>{o.name}</b><span class="n">{o.recorded} of {o.target}</span><em>{STATE[o.state]}</em></li>{/each}</ul>
 {#if outcome.notRecorded}<p class="why">{outcome.notRecorded} planned session{outcome.notRecorded === 1 ? '' : 's'} not recorded</p>{/if}
 <h3>Suggested for this week</h3>
 {#each proposals as p}
  <label class="prop"><input type="checkbox" disabled={p.proposed === p.current} bind:checked={accepted[p.key]}/><span><b>{p.name}: {p.current} → <span class="n">{p.proposed}</span> {p.unit}</b><small>{p.reason}</small></span></label>
 {/each}
 <p class="small">Nothing changes unless you tick it.</p>
 <button class="primary" onclick={() => step = 'goals'}>Next: goals</button>
{:else if step === 'goals'}
 {#key base}<GoalEditor initial={base} label="Next: place them" save={(t: GoalTargets) => A.saveGoals(t)} done={() => step = 'place'}/>{/key}
{:else if step === 'place'}
 <DayCells size={30}/>
 {#each items as it, n}
  <div class="bl"><span><b>{it.label}</b></span><span class="chips">{#each days as d}<button class="chipb" onclick={() => A.allocate(takeItem(ctx, it, d), false)}>{d === ctx.calendarDate ? 'Today' : dayLabel(d).slice(0, 3)}</button>{/each}</span></div>
 {:else}<p class="small">Everything is placed or recorded.</p>{/each}
 <p class="small">Anything you leave stays in this week’s backlog on Today and Plan.</p>
 <button class="primary" onclick={() => step = 'done'}>Next</button>
{:else}
 <WeekSummary/>
 <button class="primary" onclick={finish}>Done</button>
{/if}
<div class="foot">
 <label class="remind"><input type="checkbox" checked={ctx.state.planReminder !== false} onchange={e => store.set({planReminder: e.currentTarget.checked}, e.currentTarget.checked ? 'Weekly planning reminder on' : 'Weekly planning reminder off')}/>Remind me each week</label>
 <button class="link" onclick={() => store.closeSheet()}>Cancel</button>
</div>
<style>
 .steps{display:flex;gap:6px;list-style:none;padding:0;margin:0 0 10px;font:600 11px Inter;color:var(--text3)}.steps li{padding:4px 8px;border-radius:999px;background:rgba(255,255,255,.05)}.steps li.on{background:var(--ink);color:var(--bg)}
 h3{font:600 11px Inter;letter-spacing:.1em;text-transform:uppercase;color:var(--text3);margin:12px 0 6px}
 .why{font-size:12px;color:#A9C8FF;margin:0 0 6px}
 .outcome{list-style:none;padding:0;margin:0;display:grid;gap:6px}.outcome li{display:grid;grid-template-columns:10px 1fr auto auto;gap:8px;align-items:center;padding:8px 10px;border:1px solid var(--line);border-radius:12px;font-size:13px}.outcome i{width:9px;height:9px;border-radius:50%;background:var(--c)}.outcome em{font-style:normal;font-size:11px;color:var(--text2)}.o-met em{color:#3DDC97}.o-not-counted{border-style:dotted}
 .prop{display:grid;grid-template-columns:22px 1fr;gap:8px;align-items:start;padding:9px 0;border-bottom:1px solid var(--line)}.prop input{width:18px;height:18px;accent-color:var(--ink)}.prop b{display:block;font-weight:600;font-size:14px}.prop small{font-size:12px;color:var(--text2)}
 .bl{padding:9px 0;border-bottom:1px solid var(--line)}.bl b{font-size:14px}.chips{display:flex;flex-wrap:wrap;gap:5px;margin-top:6px}.chips .chipb{min-height:34px;padding:6px 10px}
 .foot{display:flex;justify-content:space-between;align-items:center;margin-top:12px;font-size:12px;color:var(--text2)}.remind{display:flex;gap:6px;align-items:center}.remind input{accent-color:var(--ink);width:16px;height:16px}
</style>
