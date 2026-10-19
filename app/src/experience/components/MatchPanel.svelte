<script lang="ts">
 import Icon from '../../lib/components/Icon.svelte';
 import ActivityTile from './ActivityTile.svelte';
 import {store} from '../lib/store.svelte';
 import {A} from '../lib/actions';
 import {itemMatch, type PlanItem} from '../lib/match';
 // What was recorded for a plan item: the match (automatic or yours) with
 // Unlink, or the candidates when Raring has to ask.
 let {p}: {p: PlanItem} = $props();
 let ctx = $derived(store.ctx);
 let m = $derived(itemMatch(ctx, p));
 const go = (r: string) => store.navigate(r);
</script>
<div class="sec-h"><h2>Recorded that day</h2></div>
{#if m.state === 'done' && m.activity}
 <ActivityTile a={m.activity} {ctx} navigate={go}/>
 <p class="how"><Icon name="check" size={15}/>{m.auto ? 'Matched by Raring: same day and sport, the only candidate' : 'Linked by you'}{p.source === 'garmin' ? ' · Garmin’s own completion flag isn’t read' : ''}</p>
 <button class="link" onclick={() => A.unlink(p.id)}>Unlink</button>
{:else if m.state === 'review'}
 <p class="small">{m.candidates.some(a => a.sport === p.sport) ? 'More than one recording could be this session.' : 'A different sport was recorded that day.'} Each recording already counts once toward your goals.</p>
 {#each m.candidates as a}<ActivityTile {a} {ctx} navigate={go}/><div class="cand"><button class="secondary" onclick={() => A.link(p.id, a.id)}>This was my session</button></div>{/each}
{:else}<p class="small">{p.date < ctx.calendarDate ? 'No matching recording.' : 'Nothing recorded yet. A recording of the same sport that day will count here automatically.'}</p>{/if}
<style>
 .how{display:flex;gap:6px;align-items:center;margin:8px 4px 0;font-size:12.5px;color:#3DDC97}
 .cand{display:flex;justify-content:flex-end;margin-top:6px}.cand .secondary{width:auto;margin:0;padding:10px 14px}
</style>
