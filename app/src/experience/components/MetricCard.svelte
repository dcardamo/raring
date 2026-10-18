<script lang="ts">
 import AxisChart from './AxisChart.svelte';
 import Icon from '../../lib/components/Icon.svelte';
 import {store} from '../lib/store.svelte';
 import {addDays, dayMonth} from '../lib/dates';
 // One Garmin daily metric with an in-place range picker, its usual range and
 // a link to the canonical detail for comparisons and all history.
 let {id, name, unit, read, band = null, color = '#FF8A8A', fmt = (v: number) => String(Math.round(v))}: {id: string, name: string, unit: string, read: (d: string) => number | null, band?: [number, number] | null, color?: string, fmt?: (v: number) => string} = $props();
 const RANGES = [[7, '7D'], [30, '30D'], [90, '90D'], [365, '1Y']] as const;
 let span = $state<number>(30);
 let ctx = $derived(store.ctx);
 let end = $derived(ctx.stale ? ctx.upload.date : ctx.displayedDay);
 let pts = $derived.by(() => {
  const step = span > 90 ? 7 : 1, out: {x: string, v: number | null}[] = [];
  for (let i = span - step; i >= 0; i -= step) { const d = addDays(end, -i); out.push({x: dayMonth(d), v: d < ctx.world.historyStart ? null : read(d)}); }
  return out;
 });
 let latest = $derived(pts.findLast(p => p.v != null)?.v ?? null);
 let vals = $derived(pts.map(p => p.v).filter((v): v is number => v != null));
 let avg = $derived(vals.length ? vals.reduce((a, b) => a + b, 0) / vals.length : null);
</script>
<section class="card mc">
 <div class="mc-h"><div><small>{name} · Garmin</small><b class="n">{latest != null ? fmt(latest) : '—'}<span>{unit}</span></b>{#if avg != null}<em>avg {fmt(avg)} over {span === 365 ? 'a year' : `${span} days`}</em>{/if}</div>
  <div class="rp" role="group" aria-label={`${name} range`}>{#each RANGES as [n, l]}<button aria-pressed={span === n} onclick={() => span = n}>{l}</button>{/each}</div></div>
 <AxisChart points={pts} kind={span === 7 ? 'dots' : 'line'} {unit} {band} {color} {fmt} height={130} label={`${name}, last ${span} days: latest ${latest ?? 'none'} ${unit}${band ? `, usual ${band[0]}–${band[1]}` : ''}`}/>
 <button class="link more" onclick={() => store.navigate(`metric/${id}`)}>Comparisons and all history<Icon name="chevron" size={13}/></button>
</section>
<style>
 .mc-h{display:flex;justify-content:space-between;align-items:flex-start;gap:8px;flex-wrap:wrap;margin-bottom:4px}.mc-h small{display:block;font-size:11px;color:var(--text3)}.mc-h b{font-size:26px;font-weight:600}.mc-h b span{font:500 12px Inter;color:var(--text3);margin-left:3px}.mc-h em{display:block;font-style:normal;font-size:11.5px;color:var(--text2)}
 .rp{display:grid;grid-template-columns:repeat(4,minmax(0,1fr));gap:3px;padding:3px;border-radius:10px;background:rgba(255,255,255,.05);max-width:100%}.rp button{min-height:32px;min-width:0;padding:0 6px;border:0;border-radius:8px;background:none;color:var(--text2);font:600 11px Inter}.rp button[aria-pressed=true]{background:var(--ink);color:var(--bg)}
 .more{display:inline-flex;gap:4px;align-items:center;font-size:12px;margin-top:4px}
</style>
