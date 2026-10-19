<script lang="ts" module>
 export const RANGES = [[7, '7D'], [14, '14D'], [30, '30D'], [90, '90D'], [365, '1Y'], [0, 'Max']] as const;
 /** Days covered by a range; Max is everything since downloaded history began. */
 export const rangeDays = (n: number, today: string, historyStart: string) => n || Math.round((Date.parse(today) - Date.parse(historyStart)) / 864e5) + 1;
</script>
<script lang="ts">
 // The one range picker: 7D · 14D · 30D · 90D · 1Y · Max, everywhere a chart has a range.
 let {value = $bindable(30), label}: {value?: number, label: string} = $props();
</script>
<div class="rp" role="group" aria-label={label}>{#each RANGES as [n, l]}<button aria-pressed={value === n} onclick={() => value = n}>{l}</button>{/each}</div>
<style>
 .rp{display:grid;grid-template-columns:repeat(6,minmax(0,1fr));gap:2px;padding:3px;border-radius:10px;background:rgba(255,255,255,.05);max-width:100%}
 .rp button{min-height:32px;min-width:0;padding:0 2px;border:0;border-radius:8px;background:none;color:var(--text2);font:600 11px Inter}
 .rp button[aria-pressed=true]{background:var(--ink);color:var(--bg)}
</style>
