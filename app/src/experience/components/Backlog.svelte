<script lang="ts">
 import Icon from '../../lib/components/Icon.svelte';
 import Glyph from './Glyph.svelte';
 import {store} from '../lib/store.svelte';
 import {A} from '../lib/actions';
 import {backlog, takeItem} from '../lib/week';
 import {GOAL_INFO} from '../lib/allocation';
 import {addDays, dayLabel} from '../lib/dates';
 // What's left of this week's goals. One tap takes an item for today; on Plan
 // it can also go on any remaining day. Paused or awaiting data: no remainder.
 let {days = false}: {days?: boolean} = $props();
 let ctx = $derived(store.ctx);
 let b = $derived(backlog(ctx));
 let future = $derived(Array.from({length: 7}, (_, i) => addDays(ctx.weekFrom, i)).filter(d => d > ctx.calendarDate));
 let blocked = $derived(b.paused || ctx.restActive || ['evening', 'winddown', 'late'].includes(ctx.phase) && !days);
</script>
{#if b.paused}<p class="bl-note"><Icon name="pause" size={14}/>Backlog paused while {ctx.paused?.reason === 'ill' ? 'you’re ill' : 'training is paused'}. Nothing carries over.</p>
{:else if b.awaiting}<p class="bl-note dotted">Awaiting Garmin data · what’s left shows once this week’s activities are complete</p>
{:else if b.items.length}
 <ul class="bl" aria-label="Left this week">
  {#each b.items as it}
   <li style:--c={GOAL_INFO[it.key].color}>
    <Glyph sport={it.key === 'sleep' ? 'other' : it.sport} mark="plan" size={26}/>
    <span class="t"><b>{it.label}</b><small>{it.key === 'strength' ? 'Not placed on a day yet' : it.key === 'sleep' ? 'Choose the nights' : it.key === 'im' ? 'Garmin’s intensity minutes from any activity count' : `Only recorded ${GOAL_INFO[it.key].name} heart-rate time counts`}</small></span>
    <span class="acts">
     {#if !blocked}<button class="do" onclick={() => A.allocate(takeItem(ctx, it, ctx.calendarDate), false)}>{it.key === 'sleep' ? 'Tonight' : 'Do it today'}</button>{/if}
     {#if days}{#each future as d}<button class="chipb" aria-label={`Place ${it.label} on ${dayLabel(d)}`} onclick={() => A.allocate(takeItem(ctx, it, d), false)}>{dayLabel(d).slice(0, 3)}</button>{/each}{/if}
    </span>
   </li>
  {/each}
 </ul>
{:else}<p class="bl-note">Nothing left to place this week.</p>{/if}
<style>
 .bl{list-style:none;margin:0;padding:0;display:grid;gap:6px}
 .bl li{display:grid;grid-template-columns:26px 1fr;gap:4px 10px;align-items:center;padding:9px 10px;border:1px dashed color-mix(in srgb,var(--c) 45%,transparent);border-radius:14px;background:rgba(255,255,255,.02)}
 .t b{display:block;font:600 14px Outfit}.t small{font-size:11px;color:var(--text2)}
 .acts{grid-column:1/-1;display:flex;flex-wrap:wrap;gap:5px;padding-left:36px}.acts:empty{display:none}
 .do{min-height:34px;padding:6px 12px;border-radius:10px;border:0;background:var(--ink);color:var(--bg);font:600 12px Inter}
 .acts .chipb{min-height:34px;padding:6px 10px}
 .bl-note{display:flex;gap:6px;align-items:center;font-size:12px;color:#A9C8FF;margin:4px 2px}.bl-note.dotted{border:1px dotted #6B7689;border-radius:12px;padding:8px 10px;color:var(--text2)}
</style>
