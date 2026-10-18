<script lang="ts">
 import Icon from '../../lib/components/Icon.svelte';
 import {store} from '../lib/store.svelte';
 import {planPrompt, weekOutcome} from '../lib/week';
 import {weekId, weekRange} from '../lib/dates';
 // Offered on every tab at the start of a week or while the week has no plan.
 // Never a wall: later, plan now, or stop the reminder right here.
 let ctx = $derived(store.ctx);
 let p = $derived(planPrompt(ctx));
 let last = $derived(p?.review ? weekOutcome(ctx, p.review) : null);
 const later = () => store.set(s => ({planWeeks: {...(s.planWeeks ?? {}), [weekId(ctx.calendarDate)]: 'dismissed'}}), 'Not this week');
</script>
{#if p}
 <section class="pp" aria-label="Plan your week">
  <span class="pp-i"><Icon name="clock" size={18}/></span>
  <div class="pp-t"><b>{last ? 'Review last week, plan this one' : 'Plan your week'}</b>
   <small>{weekRange(ctx.weekFrom)}{last ? ` · last week ${last.goals.filter(g => g.state === 'met').length} of ${last.goals.length} goals met` : ' · goals, then days'}</small></div>
  <div class="pp-a"><button class="go" onclick={() => store.openSheet('planweek')}>{last ? 'Start' : 'Plan'}</button><button class="later" onclick={later}>Not now</button></div>
  <button class="off" onclick={() => store.set({planReminder: false}, 'Weekly planning reminder off · Settings turns it back on')}>Stop weekly reminders</button>
 </section>
{/if}
<style>
 .pp{display:grid;grid-template-columns:auto 1fr auto;gap:4px 10px;align-items:center;margin:8px 0 4px;padding:10px 12px;border:1px solid rgba(92,168,255,.3);border-radius:16px;background:linear-gradient(120deg,rgba(92,168,255,.12),var(--card))}
 .pp-i{color:#8FC2FF}.pp-t b{display:block;font:600 14px Outfit}.pp-t small{font-size:11.5px;color:var(--text2)}
 .pp-a{display:flex;gap:6px}.pp-a button{min-height:34px;border-radius:10px;font:600 12px Inter;padding:6px 11px;border:1px solid var(--line)}.go{background:var(--ink);color:var(--bg)}.later{background:none;color:var(--text)}
 .off{grid-column:2/-1;justify-self:start;background:none;border:0;padding:4px 0;color:var(--text3);font-size:11px;text-decoration:underline;min-height:28px}
 @container(max-width:320px){.pp{grid-template-columns:auto 1fr}.pp-a{grid-column:1/-1}}
</style>
