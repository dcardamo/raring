<script lang="ts">
 import {store} from '../lib/store.svelte';
 import {dayEvidence} from '../lib/allocation';
 import {addDays, dayLabel, narrowDay} from '../lib/dates';
 import {toneColor} from '../../lib/charts';
 // One cell per day: Sleep Score (a Garmin reading), recorded cardio and
 // strength against what was planned that day, and a vitals status.
 // Future days show only what is planned; unknown days are dotted.
 let {selected, select, from, count = 7, lead = 0, compact = false}: {selected?: string, select?: (date: string) => void, from?: string, count?: number, lead?: number, compact?: boolean} = $props();
 let start = $derived(from ?? store.ctx.weekFrom);
 let days = $derived(Array.from({length: count}, (_, i) => dayEvidence(store.ctx, addDays(start, i))));
 let cardioUnit = $derived(days[0]?.cardioLabel === 'Zone 2 minutes' ? 'Zone 2 min' : 'Cardio min');
 const ring = (value: number, target: number) => Math.min(100, value / Math.max(1, target) * 100);
 const click = (date: string) => select ? select(date) : store.openSheet('day', date);
 const sleepTone = (s: number) => toneColor(s >= 80 ? 'good' : s >= 60 ? 'fair' : 'poor');
</script>
<div class="day-cells" class:compact aria-label="Days">
 {#if lead}{#each Array(lead) as _}<span></span>{/each}{/if}
 {#each days as d}
  {@const outside = d.date < store.ctx.world.historyStart}
  <button class="dc" class:current={d.date === store.ctx.calendarDate} class:selected={selected === d.date} class:future={d.future} class:paused={d.paused} class:rest={d.rest} class:unknown={d.unknown} disabled={outside} onclick={() => click(d.date)}
   aria-label={`${dayLabel(d.date)}${d.future ? ', future' : ''}${d.rest ? ', rest day' : ''}${d.paused ? ', paused' : ''}${d.unknown ? ', awaiting data' : ''}. Sleep score ${d.night?.score ?? (d.sleepPending ? 'not yet' : 'unavailable')}. ${d.future ? `${d.cardioTarget ? `${d.cardioTarget} ${cardioUnit} planned. ` : ''}${d.strengthTarget ? `${d.strengthTarget} strength planned.` : ''}` : `${d.unknown ? 'Activity data awaiting.' : `${d.cardio} ${cardioUnit}${d.cardioTarget ? ` of ${d.cardioTarget} planned` : ''}. ${d.strength} strength${d.strengthTarget ? ` of ${d.strengthTarget} planned` : ''}.`} Vitals ${d.vitals}.`}`}>
   <span class="dow">{narrowDay(d.date)}</span><b class="date n">{Number(d.date.slice(8))}</b>
   {#if !outside}
   <span class="circles">
    {#if d.night}<span class="c sleep" style:--c={sleepTone(d.night.score)}><i>{d.night.score}</i></span>{:else if d.sleepPending}<span class="c nil"></span>{:else}<span class="c dot"><i>–</i></span>{/if}
    {#if d.future}{#if d.cardioTarget}<span class="c plan" style:--c="#5CA8FF"><i>{d.cardioTarget}</i></span>{:else}<span class="c nil"></span>{/if}
    {:else if d.unknown}<span class="c dot"><i>?</i></span>
    {:else if d.cardioTarget}<span class="c ring" style:--c="#5CA8FF" style:--p={ring(d.cardio, d.cardioTarget) + '%'}><i>{d.cardio}</i></span>
    {:else if d.cardio}<span class="c fact" style:--c="#5CA8FF"><i>{d.cardio}</i></span>{:else}<span class="c nil"></span>{/if}
    {#if d.future}{#if d.strengthTarget}<span class="c plan" style:--c="#FF7DB8"><i>{d.strengthTarget}</i></span>{:else}<span class="c nil"></span>{/if}
    {:else if d.unknown}<span class="c dot"><i>?</i></span>
    {:else if d.strengthTarget}<span class="c ring" style:--c="#FF7DB8" style:--p={ring(d.strength, d.strengthTarget) + '%'}><i>{d.strength}</i></span>
    {:else if d.strength}<span class="c fact" style:--c="#FF7DB8"><i>{d.strength}</i></span>{:else}<span class="c nil"></span>{/if}
    {#if d.future}<span class="c nil"></span>{:else if d.vitals === 'unknown'}<span class="c dot"><i>–</i></span>{:else}<span class="c vit" class:unusual={d.vitals === 'unusual'}><i>{d.vitals === 'unusual' ? '!' : '✓'}</i></span>{/if}
   </span>
   {/if}
   {#if d.rest}<span class="state">Rest</span>{:else if d.paused}<span class="state">Paused</span>{/if}
  </button>
 {/each}
</div>
<div class="cell-key" aria-hidden="true"><span><i class="k1">80</i>Sleep score</span><span><i class="k2">25</i>{cardioUnit}</span><span><i class="k3">1</i>Strength sessions</span><span><i class="k4">✓</i>Vitals in usual range</span><span><i class="k5"></i>Ring = against today's plan · outline = planned</span></div>
<style>
 .day-cells{display:grid;grid-template-columns:repeat(7,minmax(0,1fr));gap:4px}
 .dc{position:relative;min-width:0;padding:7px 1px 6px;border:1px solid transparent;border-radius:13px;background:rgba(255,255,255,.03);color:var(--text);display:flex;flex-direction:column;align-items:center;gap:2px;cursor:pointer}
 .dc:disabled{cursor:default;opacity:.35;background:none}
 .dc.current{background:rgba(255,255,255,.07);border-color:rgba(255,255,255,.24)}.dc.selected{border-color:var(--ink)}
 .dow{font:600 10px Inter;color:var(--text3)}.date{font-size:16px;font-weight:600}
 .dc.future{background:none}.dc.future .date{color:var(--text3)}
 .dc.paused{background-image:repeating-linear-gradient(45deg,rgba(169,200,255,.1) 0 3px,transparent 3px 7px)}
 .dc.rest{background-color:rgba(169,200,255,.09)}
 .dc.unknown{border:1px dotted #6B7689}
 .circles{display:grid;grid-template-columns:20px 20px;gap:3px;margin-top:3px}
 .c{height:20px;width:20px;display:grid;place-items:center;border-radius:50%}
 .c i{display:grid;place-items:center;width:16px;height:16px;border-radius:50%;font:600 9px Outfit;font-style:normal;color:var(--text)}
 .c.sleep{background:var(--c)}.c.sleep i{background:#121827;color:var(--c)}
 .c.ring{background:conic-gradient(var(--c) var(--p),rgba(255,255,255,.1) var(--p))}.c.ring i{background:#121827}
 .c.fact{background:color-mix(in srgb,var(--c) 22%,transparent)}.c.fact i{color:var(--c)}
 .c.plan{border:1.5px solid color-mix(in srgb,var(--c) 70%,transparent)}.c.plan i{color:color-mix(in srgb,var(--c) 80%,white)}
 .c.nil{background:none}
 .c.dot{border:1px dotted #6B7689}.c.dot i{color:var(--text3)}
 .c.vit{background:rgba(61,220,151,.14)}.c.vit i{color:#3DDC97}.c.vit.unusual{background:rgba(255,181,71,.16)}.c.vit.unusual i{color:#FFB547}
 .state{font:600 9px Inter;color:#A9C8FF;letter-spacing:.02em}
 .cell-key{display:flex;flex-wrap:wrap;gap:6px 12px;margin-top:10px;font:500 10.5px Inter;color:var(--text2)}
 .cell-key span{display:inline-flex;align-items:center;gap:5px}
 .cell-key i{display:grid;place-items:center;height:16px;min-width:16px;border-radius:50%;font:600 8.5px Outfit;font-style:normal}
 .k1{border:2px solid #3DDC97;color:#3DDC97}.k2{background:rgba(92,168,255,.22);color:#5CA8FF}.k3{background:rgba(255,125,184,.22);color:#FF7DB8}.k4{background:rgba(61,220,151,.14);color:#3DDC97}.k5{width:16px;background:conic-gradient(#E4E8F0 60%,rgba(255,255,255,.12) 60%)}
 @container(max-width:330px){.day-cells{grid-template-columns:repeat(4,minmax(0,1fr));gap:6px}}
</style>
