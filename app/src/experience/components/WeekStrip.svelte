<script lang="ts" module>
 import type {Ctx} from '../lib/rules';
 import type {Sport} from '../lib/sports';
 import type {Mark} from './Glyph.svelte';
 import {SPORTS} from '../lib/sports';
 export type DayMark = {mark: Mark, sport: Sport, label: string, go?: string};
 /** Everything that belongs to a day, in a stable order: recorded, Garmin, yours, rest. */
 export function marksFor(ctx: Ctx, d: string): DayMark[] {
  const out: DayMark[] = [];
  for (const a of ctx.world.activities.filter(a => a.date === d && d <= ctx.calendarDate)) out.push({mark: 'recorded', sport: a.sport, label: `Recorded ${SPORTS[a.sport].name} ${a.start}`, go: `workout/${a.id}`});
  for (const s of ctx.world.schedule.filter(s => s.date === d)) out.push({mark: 'garmin', sport: s.sport, label: `Garmin: ${s.name}`, go: `scheduled/${s.id}`});
  for (const i of ctx.state.intentions.filter(i => i.date === d)) out.push({mark: 'plan', sport: i.sport, label: `Your plan: ${SPORTS[i.sport].name}${i.link ? ', linked' : ''}`, go: `intention/${i.id}`});
  if (ctx.state.rest?.day === d) out.push({mark: 'rest', sport: 'other', label: 'Rest day'});
  return out;
 }
 export const isPaused = (ctx: Ctx, d: string) => [...ctx.state.pauses, ...(ctx.state.pause ? [ctx.state.pause] : [])].some(p => d >= p.since && (!p.until || d <= p.until));
</script>
<script lang="ts">
 import Glyph from './Glyph.svelte';
 import {addDays, dayLabel, narrowDay} from '../lib/dates';
 let {ctx, selected = null, onSelect, from = ctx.weekFrom, max = 3}: {ctx: Ctx, selected?: string | null, onSelect?: (d: string) => void, from?: string, max?: number} = $props();
 let days = $derived(Array.from({length: 7}, (_, i) => addDays(from, i)));
</script>
<ol class="wstrip" aria-label="This week">
 {#each days as d}
  {@const marks = marksFor(ctx, d)}{@const paused = isPaused(ctx, d)}{@const today = d === ctx.calendarDate}
  <li>
   <button class="wday" class:today class:sel={selected === d} class:past={d < ctx.calendarDate} class:paused disabled={!onSelect} aria-pressed={onSelect ? selected === d : undefined} onclick={() => onSelect?.(d)}
    aria-label={`${dayLabel(d)}${today ? ', today' : ''}${paused ? ', paused' : ''}: ${marks.map(m => m.label).join('; ') || 'nothing'}`}>
    <span class="wl">{narrowDay(d)}</span><span class="wn n">{Number(d.slice(8))}</span>
    <span class="wm">
     {#each marks.slice(0, max) as m}<Glyph sport={m.sport} mark={m.mark} size={22} label={m.label}/>{/each}
     {#if marks.length > max}<span class="more">+{marks.length - max}</span>{/if}
     {#if !marks.length && paused}<Glyph mark="paused" size={22}/>{/if}
     {#if !marks.length && !paused}<span class="nil"></span>{/if}
    </span>
   </button>
  </li>
 {/each}
</ol>
<style>
 .wstrip{list-style:none;margin:0;padding:0;display:grid;grid-template-columns:repeat(7,minmax(0,1fr));gap:4px}
 .wday{width:100%;display:flex;flex-direction:column;align-items:center;gap:3px;padding:8px 0 10px;border-radius:14px;border:1px solid transparent;background:none;color:var(--text2);min-height:112px}
 .wday:disabled{cursor:default;opacity:1;color:var(--text2)}
 .wday.past .wn{color:var(--text2)}
 .wday.today{background:rgba(255,255,255,.07);border-color:rgba(255,255,255,.14)}
 .wday.today .wl,.wday.today .wn{color:var(--text)}
 .wday.sel{border-color:var(--ink)}
 .wday.paused{background-image:repeating-linear-gradient(45deg,rgba(169,200,255,.08) 0 3px,transparent 3px 7px)}
 .wl{font:600 11px Inter;color:var(--text3)}
 .wn{font-size:16px;font-weight:600;color:var(--text)}
 .wm{display:flex;flex-direction:column;align-items:center;gap:3px;margin-top:4px}
 .more{font:600 10px Inter;color:var(--text3)}
 .nil{width:6px;height:6px;border-radius:50%;background:rgba(255,255,255,.1);margin-top:8px}
</style>
