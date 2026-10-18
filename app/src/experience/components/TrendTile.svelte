<script lang="ts">
 import Sparkline from '../../lib/components/Sparkline.svelte';
 import Icon from '../../lib/components/Icon.svelte';
 // A metric tile: big value, the period's line and change against the previous
 // period. Change is shown neutrally; no arrow colour implies better or worse.
 let {name, value, unit, change, points, decimals = 0, sparse = false, note, onclick, color = '#E4E8F0'}: {
  name: string, value: number | null, unit: string, change: number | null, points: (number | null)[], decimals?: number, sparse?: boolean, note?: string, onclick: () => void, color?: string,
 } = $props();
 const fmt = (v: number) => unit === 'h' ? `${Math.floor(v / 60)}:${String(Math.round(v % 60)).padStart(2, '0')}` : v.toFixed(decimals);
</script>
<button class="tt" {onclick} aria-label={`${name}: ${value == null ? 'no reading' : fmt(value) + ' ' + unit}${change != null && Math.abs(change) >= Math.pow(10, -decimals) ? `, ${change > 0 ? 'up' : 'down'} ${fmt(Math.abs(change))} from the previous period` : ', no change from the previous period'}`}>
 <span class="tt-n">{name}</span>
 <span class="tt-v"><b class="n">{value == null ? '—' : fmt(value)}</b>{#if unit && unit !== 'h'}<small>{unit}</small>{/if}</span>
 <span class="tt-c">{#if change == null}{sparse ? 'Few readings' : 'No earlier period'}{:else if Math.abs(change) < Math.pow(10, -decimals) * (unit === 'h' ? 5 : 1)}No change{:else}<Icon name={change > 0 ? 'trend-up' : 'trend-down'} size={12}/>{change > 0 ? '+' : '−'}{fmt(Math.abs(change))}{/if}{#if note}<em> · {note}</em>{/if}</span>
 <span class="tt-s"><Sparkline values={points} w={140} h={36} {color} fill={!sparse} points={sparse}/></span>
</button>
<style>
 .tt{display:flex;flex-direction:column;align-items:flex-start;gap:2px;padding:12px;border-radius:18px;background:var(--card);border:1px solid var(--line);color:var(--text);text-align:left;min-width:0}
 .tt-n{font:600 11px Inter;letter-spacing:.08em;text-transform:uppercase;color:var(--text3)}
 .tt-v{margin-top:4px}
 .tt-v b{font-size:26px;font-weight:600}
 .tt-v small{font:500 12px Inter;color:var(--text3);margin-left:3px}
 .tt-c{display:inline-flex;align-items:center;gap:3px;font:600 12px Inter;color:var(--text2)}
 .tt-c em{font-style:normal;font-weight:500;color:var(--text3)}
 .tt-s{width:100%;margin-top:8px}
 .tt-s :global(svg){width:100%}
</style>
