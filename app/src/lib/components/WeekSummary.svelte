<script lang="ts">
 import type {PlanBoard} from '../generated/PlanBoard';
 import {GOALS, unitFor} from '../goals';
 import NestedRings from './NestedRings.svelte';
 // The week in one nested ring, then one line per goal: recorded of target,
 // what's placed and what's left. Sleep Score is a Garmin reading.
 let {board, onGoals, compact = false}: {board: PlanBoard, onGoals: () => void, compact?: boolean} = $props();
 const n = (v: number) => Math.round(v);
 let lines = $derived(board.ledger.map(l => {
  const recorded = l.recorded == null ? null : n(l.recorded);
  const parts = [recorded == null ? `${l.unavailable ?? 'Awaiting data'}` : `${recorded} of ${n(l.target)} ${unitFor(l.kind, l.target)}`];
  if (l.original != null) parts.push(`was ${n(l.original)} until ${new Date(l.revisedOn + 'T12:00:00Z').toLocaleDateString('en-GB', {weekday: 'short', timeZone: 'UTC'})}`);
  if (l.observation) parts.push('observed, no target verdict');
  else if (l.planned) parts.push(`${n(l.planned)} placed`);
  if (l.review) parts.push(`${l.review} to review`);
  if (!l.observation) {
   if (l.reached) parts.push(l.original != null ? 'reached · adjusted' : 'reached');
   else if (l.remaining) parts.push(`${n(l.remaining)} left`);
   else if (board.pause && l.recorded != null) parts.push('paused');
  }
  return {l, text: parts.join(' · ')};
 }));
</script>
{#if board.ledger.length || board.sleepScore}
 <div class="ws" class:compact>
  <NestedRings rings={board.weekRings} size={compact ? 76 : 92} stroke={compact ? 5.5 : 6.5} gap={2} label={`This week: ${lines.map(x => `${GOALS[x.l.kind].name} ${x.text}`).join('; ')}`}>
   {#if board.sleepScore}<span class="sc"><b class="n">{board.sleepScore.average ?? '—'}</b><small>score</small></span>{/if}
  </NestedRings>
  <ul>
   {#each lines as {l, text}}<li style:--c={GOALS[l.kind].color}><i class:o={l.observation}></i><span><b>{GOALS[l.kind].name}</b> {text}</span></li>{/each}
   {#if board.sleepScore}<li style:--c="#A9C8FF"><i class="o"></i><span><b>Sleep Score</b> avg <span class="n">{board.sleepScore.average ?? '—'}</span> · target {board.sleepScore.target} <small>Garmin reading</small></span></li>{/if}
  </ul>
 </div>
{:else}
 <button class="ws-empty" onclick={onGoals}><b>No weekly goals yet</b><span>Choose a few and they'll appear here as rings</span></button>
{/if}
<style>
 .ws{display:grid;grid-template-columns:92px 1fr;gap:14px;align-items:center}
 ul{list-style:none;margin:0;padding:0;display:grid;gap:5px;font-size:12.5px;color:var(--text2)}
 li{display:grid;grid-template-columns:10px 1fr;gap:7px;align-items:baseline}
 li i{width:9px;height:9px;border-radius:50%;background:var(--c);transform:translateY(1px)}li i.o{background:none;border:2px solid var(--c)}
 li b{color:var(--text);font-weight:600}li .n{color:var(--text);font-weight:600}li small{color:var(--text3);font-size:10.5px}
 .sc{display:grid;place-items:center}.sc b{font-size:15px;color:#A9C8FF}.sc small{font-size:8px;color:var(--text3)}
 .ws-empty{width:100%;text-align:left;border:1px dashed var(--line);border-radius:16px;background:none;color:var(--text);padding:12px 14px}.ws-empty b{display:block;font:600 15px Outfit}.ws-empty span{font-size:12px;color:var(--text2)}
 .compact{grid-template-columns:1fr;justify-items:center;gap:8px}.compact ul{font-size:11.5px;justify-self:stretch}
 @container(max-width:300px){.ws{grid-template-columns:1fr}}
</style>
