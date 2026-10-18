<script lang="ts">
 import {dur, dayLabel} from '../lib/dates';
 // Recorded duration against that night's recorded need. A missing night is a
 // gap, not zero; each night stands alone, with no running total.
 type N = {date: string, asleep: number | null, need: number | null, label: string};
 let {nights, h = 96}: {nights: N[], h?: number} = $props();
 let max = $derived(Math.max(540, ...nights.map(n => Math.max(n.asleep ?? 0, n.need ?? 0))));
 let step = $derived(nights.length > 16 ? Math.ceil(nights.length / 7) : 1);
</script>
<div class="sn" style:--h={h + 'px'} role="img" aria-label={`Nightly sleep against need: ${nights.filter(n => n.asleep != null).map(n => `${dayLabel(n.date)} ${dur(n.asleep)} of ${dur(n.need)}`).join('; ')}`}>
 {#each nights as n, i}
  <div class="nt">
   <div class="plot">
    {#if n.asleep != null}<i class:short={n.need != null && n.need - n.asleep >= 60} style:height={n.asleep / max * 100 + '%'}></i>{:else}<span class="gap"></span>{/if}
    {#if n.need != null}<u style:bottom={n.need / max * 100 + '%'}></u>{/if}
   </div>
   <span class="lbl">{i % step === 0 ? n.label : ''}</span>
  </div>
 {/each}
</div>
<style>
 .sn{display:flex;gap:2px;align-items:stretch}
 .nt{flex:1;min-width:0;display:flex;flex-direction:column;align-items:center}
 .plot{position:relative;width:100%;max-width:18px;height:var(--h);display:flex;align-items:flex-end}
 .plot i{display:block;width:100%;border-radius:4px 4px 2px 2px;background:linear-gradient(180deg,#5C9BFF,#2F6FE0)}
 .plot i.short{background:linear-gradient(180deg,#A9C8FF,#5C9BFF);opacity:.7}
 .plot u{position:absolute;left:-1px;right:-1px;border-top:2px solid #EEF1F7;text-decoration:none;opacity:.8}
 .gap{display:block;width:100%;height:30%;border:1.5px dotted rgba(142,152,172,.55);border-radius:4px}
 .lbl{font:600 9.5px Inter;color:var(--text3);margin-top:4px;height:12px;white-space:nowrap}
</style>
