<script lang="ts">
 import type {VigorousWindow} from '../generated/VigorousWindow';
 import {num} from '../format';
 import {dayMonth, describe, windowStart} from '../vigorous';
 // Garmin's vigorous minutes in the six seven-day blocks its Fitness Age driver
 // averages, oldest first, against Garmin's weekly target (dashed). The last
 // block ends today and is outlined; today's minutes are its green top. A block
 // with a day Garmin hasn't sent is a gap. Compact drops the labels.
 let {window: w, compact = false, live = true}: {window: VigorousWindow, compact?: boolean, live?: boolean} = $props();
 let max = $derived(Math.max(w.target * 1.12, ...w.weeks.map(v => v ?? 0)));
 const pct = (v: number) => `${Math.min(100, v / max * 100)}%`;
 let today = $derived(live ? w.today ?? 0 : 0);
</script>
<div class="vb" class:compact data-chart="vigorous" role="img" aria-label={describe(w, live)}>
 <div class="plot">
  <div class="target" style:bottom={pct(w.target)}><span class="n">{num(w.target)}</span></div>
  {#each w.weeks as v, i}
   {@const last = i === w.weeks.length - 1}
   <div class="col" class:last>
    {#if !compact}<b class="n" aria-hidden="true">{v == null ? '' : num(v)}</b>{/if}
    {#if v != null}<i class="bar" style:height={pct(v)}>{#if last && today > 0}<i class="today" style:height={`${Math.min(100, today / Math.max(v, 1) * 100)}%`}></i>{/if}</i>{/if}
   </div>
  {/each}
 </div>
 {#if !compact}<div class="ticks" aria-hidden="true"><span>{dayMonth(windowStart(w))}</span><span>Last 7 days</span></div>{/if}
</div>
<style>
 .vb{--plot:88px;--gutter:30px;width:100%}
 .vb.compact{--plot:40px;--gutter:24px}
 .plot{position:relative;height:var(--plot);display:flex;align-items:flex-end;gap:6px;padding-right:var(--gutter);border-bottom:1px solid var(--line-strong);margin-top:16px}
 .compact .plot{gap:4px;margin-top:6px}
 .col{flex:1;min-width:0;height:100%;display:flex;flex-direction:column;justify-content:flex-end;align-items:stretch}
 .col b{display:block;margin-bottom:3px;text-align:center;font:600 12px/1 Outfit,Inter;color:var(--text2);white-space:nowrap}
 .col.last b{color:var(--text)}
 .bar{display:flex;flex-direction:column;justify-content:flex-start;background:var(--amount);opacity:.78;border-radius:4px 4px 2px 2px;min-height:2px;overflow:hidden}
 .last .bar{opacity:1;box-shadow:0 0 0 1.5px var(--card),0 0 0 3px var(--ink)}
 .today{display:block;width:100%;background:var(--good);flex:none}
 .target{position:absolute;left:0;right:0;height:0;border-top:1.5px dashed color-mix(in srgb,var(--ink) 75%,transparent);z-index:1;pointer-events:none}
 .target span{position:absolute;right:0;top:0;transform:translateY(-50%);font:600 12px/1 Outfit,Inter;color:var(--text);background:var(--card);padding-left:4px}
 .compact .target span{font-size:11px}
 .ticks{display:flex;justify-content:space-between;gap:8px;padding-right:var(--gutter);margin-top:5px;font:500 12px/1.2 Inter;color:var(--text3)}
 .ticks span:last-child{color:var(--text);font-weight:600;text-align:right}
</style>
