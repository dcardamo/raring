<script lang="ts">
 import AxisChart from './AxisChart.svelte';
 import RangePicker, {rangeDays} from './RangePicker.svelte';
 import Icon from '../../lib/components/Icon.svelte';
 import {store} from '../lib/store.svelte';
 import {addDays, dayMonth} from '../lib/dates';
 // One Garmin daily metric with an in-place range picker, its usual range and
 // a link to the canonical detail for comparisons and all history.
 let {id, name, unit, read, band = null, color = '#FF8A8A', fmt = (v: number) => String(Math.round(v)), route, sparse = false, children}: {id: string, name: string, unit: string, read: (d: string) => number | null, band?: [number, number] | null, color?: string, fmt?: (v: number) => string, route?: string, sparse?: boolean, children?: import('svelte').Snippet} = $props();
 let span = $state<number>(30);
 let ctx = $derived(store.ctx);
 let end = $derived(ctx.stale ? ctx.upload.date : ctx.displayedDay);
 let pts = $derived.by(() => {
  const days = rangeDays(span, end, ctx.world.historyStart), step = days > 400 ? 14 : days > 90 ? 7 : 1, out: {x: string, v: number | null}[] = [];
  // Sparse readings (weigh-ins) take the latest within each step.
  for (let i = days - step; i >= 0; i -= step) if (sparse && step > 1) { let v: number | null = null; for (let k = 0; k < step && v == null; k++) { const d = addDays(end, -i + k); if (d <= end && d >= ctx.world.historyStart) v = read(d); } out.push({x: dayMonth(addDays(end, -i)), v}); } else { const d = addDays(end, -i); out.push({x: dayMonth(d), v: d < ctx.world.historyStart ? null : read(d)}); }
  return out;
 });
 let latest = $derived(pts.findLast(p => p.v != null)?.v ?? null);
 let vals = $derived(pts.map(p => p.v).filter((v): v is number => v != null));
 let avg = $derived(vals.length ? vals.reduce((a, b) => a + b, 0) / vals.length : null);
 let lastDate = $derived.by(() => { for (let i = 0; i < 400; i++) { const d = addDays(end, -i); if (d < ctx.world.historyStart) break; if (read(d) != null) return d; } return null; });
</script>
<section class="card mc">
 <div class="mc-h"><div><small>{name} · Garmin</small><b class="n">{latest != null ? fmt(latest) : '—'}<span>{unit}</span></b>{#if avg != null}<em>{sparse ? `last ${lastDate ? dayMonth(lastDate) : '—'}` : `avg ${fmt(avg)} over ${span === 0 ? 'all history' : span === 365 ? 'a year' : `${span} days`}`}</em>{/if}</div></div>
 <RangePicker bind:value={span} label={`${name} range`}/>
 <AxisChart points={pts} kind={span === 7 || sparse ? 'dots' : 'line'} {unit} {band} {color} {fmt} height={130} label={`${name}, ${span ? `last ${span} days` : 'all history'}: latest ${latest ?? 'none'} ${unit}${band ? `, usual ${band[0]}–${band[1]}` : ''}`}/>
 {#if children}{@render children()}{/if}
 <button class="link more" onclick={() => store.navigate(route ?? `metric/${id}`)}>Comparisons and all history<Icon name="chevron" size={13}/></button>
</section>
<style>
 .mc-h{display:flex;justify-content:space-between;align-items:flex-start;gap:8px;flex-wrap:wrap;margin-bottom:4px}.mc-h small{display:block;font-size:11px;color:var(--text3)}.mc-h b{font-size:26px;font-weight:600}.mc-h b span{font:500 12px Inter;color:var(--text3);margin-left:3px}.mc-h em{display:block;font-style:normal;font-size:11.5px;color:var(--text2)}
 .more{display:inline-flex;gap:4px;align-items:center;font-size:12px;margin-top:4px}
</style>
