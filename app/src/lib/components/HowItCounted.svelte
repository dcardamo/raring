<script lang="ts">
 import type {WorkoutCount} from '../generated/WorkoutCount';
 import {GOALS, amount} from '../goals';
 import {dateLabel, num} from '../format';
 import Icon from './Icon.svelte';
 // What this recording added to the week's goals, from the core, and its
 // vigorous minutes toward Fitness Age's driver while Garmin's six-week
 // average holds them. Garmin's estimates (VO₂ max, Fitness Age) aren't
 // credited to one workout.
 let {counted}: {counted: WorkoutCount} = $props();
</script>
<section class="card hic" aria-labelledby="hic-h">
 <h3 id="hic-h">How it counted</h3>
 <ul>
  {#each counted.lines as l (l.kind)}
   <li>{#if l.kind === 'strength'}<b>{l.position != null ? `${l.position} of ` : ''}{num(l.target)} strength sessions</b> this week{:else}<b>{amount(l.kind, l.amount)} {GOALS[l.kind].name}</b> · {l.recorded != null ? `${num(l.recorded)} of ${num(l.target)} this week` : `goal ${num(l.target)}`}{/if}</li>
  {/each}
  {#if counted.planItem}<li>Done: <b>{counted.planItem}</b> from your plan</li>{/if}
  {#each counted.bests as b}<li><b>Best in available history</b> · {b}</li>{/each}
  {#if !counted.lines.length && !counted.planItem && !counted.bests.length && (counted.goals || !counted.fitness)}<li>Recorded. {counted.goals ? 'No weekly goal counts this kind of activity.' : 'No weekly goals set.'}</li>{/if}
  {#if counted.fitness}{@const f = counted.fitness}<li class="fa"><Icon name="heart" size={14}/><span><b>{num(f.minutes)} vigorous min</b> toward Fitness Age{f.week != null ? ` · ${num(f.week)} of Garmin’s ${num(f.target)} ${f.thisWeek ? 'this' : 'that'} week` : ''}<small>Counts in Garmin’s six-week average through {dateLabel(f.until)}</small></span></li>{/if}
 </ul>
</section>
<style>
 .fa{display:grid;grid-template-columns:16px 1fr;gap:6px;align-items:baseline}.fa:not(:first-child){margin-top:4px;padding-top:8px;border-top:1px solid var(--line)}.fa :global(svg){color:var(--good);transform:translateY(2px)}.fa small{display:block;margin-top:3px;font-size:12px;color:var(--text2)}
 .hic{margin-top:12px}.hic ul{list-style:none;margin:8px 0 0;padding:0;display:grid;gap:6px;font-size:14px}.hic b{font-weight:600}
</style>
