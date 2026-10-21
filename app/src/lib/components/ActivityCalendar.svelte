<script lang="ts">
 import type {ActivityCard} from '../generated/ActivityCard';
 import type {ActivityMonth} from '../generated/ActivityMonth';
 import Icon from './Icon.svelte';
 import {dateLabel, monthName, weekLetters} from '../format';
 import {monthDays, shiftMonth} from '../calendar';
 // A month of stored recordings: each day shows its sports in start order (two
 // icons, then one and a count) and opens that day's list. The month's name
 // opens a year of months; nothing opens before the oldest stored recording
 // or after this month.
 let {view, activities, selected, select, go, busy = false}: {view: ActivityMonth, activities: ActivityCard[], selected: string | null,
  select: (day: string) => void, go: (month: string) => void, busy?: boolean} = $props();
 let picking = $state(false), year = $state(0);
 let lowest = $derived((view.first ?? view.today).slice(0, 7) + '-01'), highest = $derived(view.today.slice(0, 7) + '-01');
 let title = $derived(new Date(view.month + 'T12:00:00Z').toLocaleDateString('en-GB', {month: 'long', year: 'numeric', timeZone: 'UTC'}));
 let lead = $derived(Math.max(0, Math.round((Date.parse(view.month) - Date.parse(view.gridStart)) / 864e5)));
 let byDay = $derived.by(() => { const days = new Map<string, ActivityCard[]>(); for (const a of [...activities].reverse()) { const d = a.startLocal.slice(0, 10); days.set(d, [...(days.get(d) ?? []), a]); } return days; });
 const open = () => { year = Number(view.month.slice(0, 4)); picking = !picking; };
 const pick = (month: string) => { picking = false; go(month); };
</script>
<div class="cal">
<div class="cal-h">
 <button class="nav" aria-label="Previous month" disabled={busy || view.month <= lowest} onclick={() => go(shiftMonth(view.month, -1))}><Icon name="arrow-left" size={18}/></button>
 <button class="title" aria-expanded={picking} aria-label={`${title}, choose a month`} onclick={open}>{title}<Icon name="chevron-down" size={14}/></button>
 <button class="nav" aria-label="Next month" disabled={busy || view.month >= highest} onclick={() => go(shiftMonth(view.month, 1))}><Icon name="arrow-right" size={18}/></button>
</div>
{#if picking}
 <div class="years"><button class="nav" aria-label="Previous year" disabled={year <= Number(lowest.slice(0, 4))} onclick={() => year--}><Icon name="arrow-left" size={16}/></button><b class="n">{year}</b><button class="nav" aria-label="Next year" disabled={year >= Number(highest.slice(0, 4))} onclick={() => year++}><Icon name="arrow-right" size={16}/></button></div>
 <div class="months" role="group" aria-label={`Months of ${year}`}>{#each Array(12) as _, i}{@const m = `${year}-${String(i + 1).padStart(2, '0')}-01`}<button aria-pressed={m === view.month} disabled={busy || m < lowest || m > highest} onclick={() => pick(m)}>{monthName(i)}</button>{/each}</div>
{:else}
 <div class="grid" role="group" aria-label={title} aria-busy={busy}>
  {#each weekLetters(view.gridStart) as letter}<span class="dow" aria-hidden="true">{letter}</span>{/each}
  {#each Array(lead) as _}<span aria-hidden="true"></span>{/each}
  {#each monthDays(view.month) as d (d)}{@const list = byDay.get(d) ?? []}
   {#if list.length}<button class="day" class:today={d === view.today} aria-pressed={selected === d} aria-label={`${dateLabel(d)}: ${list.map(a => a.name).join(', ')}`} onclick={() => select(d)}><b class="n">{Number(d.slice(8))}</b><span class="marks">{#each list.slice(0, list.length > 2 ? 1 : 2) as a}<Icon name={a.icon} size={14}/>{/each}{#if list.length > 2}<small class="n">+{list.length - 1}</small>{/if}</span></button>
   {:else}<span class="day" class:today={d === view.today} class:closed={d > view.today || !view.first || d < view.first} aria-hidden="true"><b class="n">{Number(d.slice(8))}</b></span>{/if}
  {/each}
 </div>
{/if}
</div>
<style>
 .cal{max-width:560px}
 .cal-h,.years{display:grid;grid-template-columns:44px 1fr 44px;align-items:center;text-align:center}
 .cal-h{margin:4px 0 8px}.years{margin-bottom:8px;font:600 15px Outfit}
 .nav{width:44px;height:44px;border-radius:50%;border:1px solid var(--line);background:none;color:var(--text);display:grid;place-items:center;padding:0}
 .nav:disabled{color:var(--text3);border-color:transparent;cursor:default}
 .title{justify-self:center;display:inline-flex;align-items:center;gap:6px;min-height:44px;padding:0 10px;border:0;border-radius:var(--radius-control);background:none;color:var(--text);font:600 16px Outfit}
 .title[aria-expanded=true]{background:var(--surface3)}
 .months{display:grid;grid-template-columns:repeat(4,minmax(0,1fr));gap:6px}
 .months button{min-height:44px;border:1px solid var(--line);border-radius:var(--radius-control);background:none;color:var(--text);font:600 13px Inter}
 .months button[aria-pressed=true]{background:var(--surface3);border-color:var(--ink)}.months button:disabled{color:var(--text3);border-color:transparent;cursor:default}
 .grid{display:grid;grid-template-columns:repeat(7,minmax(0,1fr));gap:3px}
 .dow{font:600 11px Inter;color:var(--text2);text-align:center;padding-bottom:2px}
 .day{display:flex;flex-direction:column;align-items:center;gap:3px;min-width:0;min-height:48px;padding:6px 0 5px;border:1px solid transparent;border-radius:var(--radius-control);background:none;color:var(--text2)}
 .day b{font-size:13px;font-weight:600}
 button.day{background:var(--surface2);color:var(--text)}
 .day.today{border-color:var(--line-strong)}.day[aria-pressed=true]{background:var(--surface3);border-color:var(--ink)}.day.closed{color:var(--text3)}
 .marks{display:flex;align-items:center;justify-content:center;gap:2px;height:14px}.marks small{font-size:11px;font-weight:600;color:var(--text2)}
</style>
