<script lang="ts">
 import type {WorkoutCount} from '../generated/WorkoutCount';
 import {GOALS, amount} from '../goals';
 import {num} from '../format';
 // What this recording added to the week's goals, from the core. Garmin's
 // estimates (VO₂ max, Fitness Age) aren't credited to one workout.
 let {counted}: {counted: WorkoutCount} = $props();
</script>
<section class="card hic" aria-labelledby="hic-h">
 <h3 id="hic-h">How it counted</h3>
 <ul>
  {#each counted.lines as l (l.kind)}
   <li>{#if l.kind === 'strength'}<b>{l.position ?? 1} of {num(l.target)} strength sessions</b> this week{:else}<b>{amount(l.kind, l.amount)} {GOALS[l.kind].name}</b> · {l.recorded != null ? `${num(l.recorded)} of ${num(l.target)} this week` : `goal ${num(l.target)}`}{/if}</li>
  {/each}
  {#if counted.planItem}<li>Done: <b>{counted.planItem}</b> from your plan</li>{/if}
  {#each counted.bests as b}<li><b>Best in available history</b> · {b}</li>{/each}
  {#if !counted.lines.length && !counted.planItem && !counted.bests.length}<li>Recorded. No weekly goal counts this kind of activity.</li>{/if}
 </ul>
</section>
<style>
 .hic{margin-top:12px}.hic ul{list-style:none;margin:8px 0 0;padding:0;display:grid;gap:6px;font-size:14px}.hic b{font-weight:600}
</style>
