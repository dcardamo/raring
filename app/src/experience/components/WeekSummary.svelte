<script lang="ts">
 import {store} from '../lib/store.svelte';
 import {weekRings, backlog} from '../lib/week';
 import NestedRings from './NestedRings.svelte';
 // The whole week in one nested ring, then one line per goal: recorded of
 // target, what's placed and what's left. Sleep Score is a Garmin reading.
 let {onGoals, compact = false}: {onGoals?: () => void, compact?: boolean} = $props();
 let w = $derived(weekRings(store.ctx));
 let left = $derived(backlog(store.ctx));
 const leftOf = (key: string) => left.items.filter(i => i.key === key).reduce((n, i) => n + i.amount, 0);
 const unit = (u: string, n: number) => u === 'sessions' ? (n === 1 ? 'session' : 'sessions') : u === 'nights' ? (n === 1 ? 'night' : 'nights') : u;
</script>
{#if w.rings.length || w.score}
 <div class="ws" class:compact>
  <NestedRings rings={w.rings} size={compact ? 76 : 92} stroke={compact ? 5.5 : 6.5} gap={2} label={`This week: ${w.rings.map(r => `${r.name} ${r.recorded} of ${w.goals[r.key]}`).join(', ')}`}>
   {#if w.score}<span class="sc"><b class="n">{w.score.average ?? '—'}</b><small>score</small></span>{/if}
  </NestedRings>
  <ul>
   {#each w.rings as r}
    <li style:--c={r.color}><i></i><span><b>{r.name}</b> <span class="n">{r.recorded}</span> of {w.goals[r.key]} {unit(r.unit, w.goals[r.key])}{r.planned ? ` · ${r.planned} placed` : ''}{left.paused ? ' · paused' : left.awaiting ? ' · awaiting data' : ''}{#if !left.paused && !left.awaiting && leftOf(r.key)}{' · '}<em>{leftOf(r.key)} left</em>{:else if r.recorded >= w.goals[r.key]}{' · reached'}{/if}</span></li>
   {/each}
   {#if w.score}<li style:--c="#A9C8FF"><i class="o"></i><span><b>Sleep Score</b> avg <span class="n">{w.score.average ?? '—'}</span> · target {w.score.target} <small>Garmin reading</small></span></li>{/if}
  </ul>
 </div>
{:else}
 <button class="ws-empty" onclick={() => onGoals ? onGoals() : store.openSheet('planweek')}><b>No weekly goals yet</b><span>Choose a few and they'll appear here as rings</span></button>
{/if}
<style>
 .ws{display:grid;grid-template-columns:92px 1fr;gap:14px;align-items:center}
 ul{list-style:none;margin:0;padding:0;display:grid;gap:5px;font-size:12.5px;color:var(--text2)}
 li{display:grid;grid-template-columns:10px 1fr;gap:7px;align-items:baseline}
 li i{width:9px;height:9px;border-radius:50%;background:var(--c);transform:translateY(1px)}li i.o{background:none;border:2px solid var(--c)}
 li b{color:var(--text);font-weight:600}li .n{color:var(--text);font-weight:600}li em{font-style:normal;color:var(--c)}li small{color:var(--text3);font-size:10.5px}
 .sc{display:grid;place-items:center}.sc b{font-size:15px;color:#A9C8FF}.sc small{font-size:8px;color:var(--text3)}
 .ws-empty{width:100%;text-align:left;border:1px dashed var(--line);border-radius:16px;background:none;color:var(--text);padding:12px 14px}.ws-empty b{display:block;font:600 15px Outfit}.ws-empty span{font-size:12px;color:var(--text2)}
 .compact{grid-template-columns:1fr;justify-items:center;gap:8px}.compact ul{font-size:11.5px;justify-self:stretch}
 @container(max-width:300px){.ws{grid-template-columns:1fr}}
</style>
