<script lang="ts">
 import type {WeekReview} from '../generated/WeekReview';
 import type {GoalKind} from '../generated/GoalKind';
 import type {LocalAction} from '../generated/LocalAction';
 import {GOALS, unitFor} from '../goals';
 import {dateLabel} from '../format';
 // What was met and why, then next week's goals proposed from history. Nothing
 // changes until the person accepts; Keep all is always there.
 let {review, act, navigate, busy = false}: {review: WeekReview, act: (a: LocalAction) => void, navigate: (route: string) => void, busy?: boolean} = $props();
 let changes = $derived(review.proposals.filter(p => p.proposed !== p.current));
 let chosen = $state<GoalKind[]>([]);
 const WORD = {met: 'Met', partly: 'Partly', none: 'Not met', notCounted: 'Not counted', observed: 'Observed'} as const;
 const n = (v: number | null) => v == null ? '—' : String(Math.round(v));
 const toggle = (k: GoalKind) => chosen = chosen.includes(k) ? chosen.filter(x => x !== k) : [...chosen, k];
 let decided = $derived(review.decision);
 // A decision keeps the proposals it saw; an old or ongoing week offers none.
 let shown = $derived(decided ? decided.proposals : review.decidable ? review.proposals : []);
 let from = $derived(decided ? (decided.at.slice(0, 10) > review.nextStart ? decided.at.slice(0, 10) : review.nextStart) : review.appliesFrom);
 let fromLabel = $derived(from === review.nextStart ? `Next week, from ${dateLabel(from)}` : `From today, ${dateLabel(from)}`);
</script>
<section class="card wr" aria-labelledby="wr-h">
 <h3 id="wr-h">Goals · week of {dateLabel(review.start)}</h3>
 {#if review.goals.length}
  <ul class="out">{#each review.goals as g}<li style:--c={GOALS[g.kind].color}><i></i><span><b>{GOALS[g.kind].name}</b> {n(g.recorded)} of {n(g.target)} {unitFor(g.kind, g.target)}</span><em class={g.state}>{WORD[g.state]}</em></li>{/each}</ul>
 {:else}<p class="small">No weekly goals that week.</p>{/if}
 {#if review.reasons.length}<p class="small">{review.counted ? '' : 'Not counted toward next week: '}{review.reasons.join(' · ')}</p>{/if}
 {#if review.notRecorded}<p class="small">{review.notRecorded} planned session{review.notRecorded === 1 ? '' : 's'} not recorded</p>{/if}
 {#if shown.length}
  <h4>{fromLabel}</h4>
  <ul class="props">{#each shown as p}{@const accepted = decided?.accepted.includes(p.kind)}<li>
   {#if p.proposed !== p.current && !decided}<label><input type="checkbox" checked={chosen.includes(p.kind)} onchange={() => toggle(p.kind)}/><b>{GOALS[p.kind].name}</b> {n(p.current)} → {n(p.proposed)} {unitFor(p.kind, p.proposed)}</label>
   {:else}<span><b>{GOALS[p.kind].name}</b> {p.proposed !== p.current && accepted ? `${n(p.current)} → ${n(p.proposed)}` : `keep ${n(p.current)}`} {unitFor(p.kind, p.current)}</span>{/if}
   {#if decided && !accepted && p.proposed !== p.current}<small>Kept (proposed {n(p.proposed)})</small>{:else}<small>{p.reason}</small>{/if}</li>{/each}</ul>
  {#if decided}<p class="small">Decided{decided.accepted.length ? `: ${decided.accepted.map(k => GOALS[k].name).join(', ')} changed from ${dateLabel(from)}` : ': goals kept'}.</p>
  {:else}<div class="acts">{#if changes.length}<button class="primary" disabled={busy || !chosen.length} onclick={() => act({action: 'decideReview', week: review.start, accepted: chosen})}>Accept {chosen.length || ''} change{chosen.length === 1 ? '' : 's'}</button>{/if}<button class="secondary" disabled={busy} onclick={() => act({action: 'decideReview', week: review.start, accepted: []})}>Keep all</button></div>{/if}
 {/if}
 <button class="edit" onclick={() => navigate('plan/commitments')}>Edit goals</button>
</section>
<style>
 .wr h4{font:600 13px Inter;margin:16px 0 6px;color:var(--text)}
 .out,.props{list-style:none;margin:0;padding:0;display:grid;gap:8px}
 .out li{display:grid;grid-template-columns:10px 1fr auto;gap:8px;align-items:baseline;font-size:14px}.out i{width:9px;height:9px;border-radius:50%;background:var(--c)}
 .out em{font-style:normal;font:600 12px Inter;color:var(--text2)}.out em.met{color:var(--good)}
 .props li{display:grid;gap:2px;font-size:14px}.props small{font-size:12px;color:var(--text2)}.props label{display:flex;gap:8px;align-items:center;min-height:36px}.props input{width:20px;height:20px;accent-color:var(--ink)}
 .edit{background:none;border:0;color:var(--ink);font:600 13px Inter;min-height:40px;margin-top:6px;padding:0}
 .acts{display:flex;gap:8px;flex-wrap:wrap;margin-top:12px}.acts button{flex:1 1 140px;width:auto;margin-top:0}
</style>
