<script lang="ts">
 import AxisChart from './AxisChart.svelte';
 import RangePicker, {rangeDays} from './RangePicker.svelte';
 import HelpButton from './HelpButton.svelte';
 import Icon from '../../lib/components/Icon.svelte';
 import {store} from '../lib/store.svelte';
 import {addDays, dayLabel, dayMonth} from '../lib/dates';
 import {changes, trendSeries, weighIn, ageDays} from '../lib/weight';
 // Scale weigh-ins as points with the trend through them, how the trend moved
 // over 1–90 days, and how fresh the last weigh-in is. No goals or verdicts.
 let {detail = false}: {detail?: boolean} = $props();
 let ctx = $derived(store.ctx);
 let span = $state(30);
 let c = $derived(changes(ctx));
 let age = $derived(ageDays(c.last, ctx.calendarDate));
 let stale = $derived(age != null && age > 7);
 let end = $derived(ctx.upload.date);
 let series = $derived(trendSeries(ctx, c.last ?? end));
 let pts = $derived.by(() => {
  const days = rangeDays(span, end, ctx.world.historyStart), step = days > 400 ? 14 : days > 90 ? 7 : 1, out: {x: string, v: number | null, t: number | null}[] = [];
  for (let i = days - step; i >= 0; i -= step) {
   const d = addDays(end, -i); let v: number | null = null;
   for (let k = 0; k < step && v == null; k++) v = weighIn(ctx, addDays(d, -k));
   out.push({x: dayMonth(d), v, t: c.last && d <= c.last ? series.get(d) ?? null : null});
  }
  return out;
 });
 // A tight scale so real change is visible: half a kilo either side of the data.
 let vals = $derived(pts.flatMap(p => [p.v, p.t]).filter((v): v is number => v != null));
 let yMin = $derived(vals.length ? Math.floor((Math.min(...vals) - .5) * 2) / 2 : undefined), yMax = $derived(vals.length ? Math.ceil((Math.max(...vals) + .5) * 2) / 2 : undefined);
 let scale = $derived(c.last ? weighIn(ctx, c.last) : null);
 const signed = (n: number) => `${n > 0 ? '+' : n < 0 ? '−' : '±'}${Math.abs(n).toFixed(1)}`;
 const SPAN = (d: number) => d === 1 ? '1 day' : `${d} days`;
</script>
<section class="card wc" aria-label="Weight">
 <div class="wc-h"><div><small>Trend weight</small><b class="n">{c.trend != null ? c.trend.toFixed(1) : '—'}<span>kg</span></b>
  <em>Scale {scale != null ? `${scale.toFixed(1)} kg` : '—'} · {c.last ? dayLabel(c.last) : 'no weigh-ins'}</em></div><HelpButton topic="trend-weight"/></div>
 {#if c.last}<p class="fresh" class:stale>{#if stale}<Icon name="clock" size={13}/>{/if}Last weigh-in {age === 0 ? 'today' : age === 1 ? 'yesterday' : `${age} days ago`}{stale ? ' · trend holds until the next one' : ''}</p>{/if}
 <RangePicker bind:value={span} label="Weight range"/>
 <AxisChart points={pts.map(p => ({x: p.x, v: p.v}))} line={pts.map(p => p.t)} kind="dots" unit="kg" {yMin} {yMax} color="#A9B4C8" lineColor="#E4E8F0" fmt={v => v.toFixed(1)} height={140}
  label={`Weight, ${span ? `last ${span} days` : 'all history'}: weigh-ins as points, trend ${c.trend?.toFixed(1) ?? 'none'} kg`}/>
 <p class="key"><span><i class="d"></i>Scale weigh-in</span><span><i class="l"></i>Trend</span></p>
 <h3 class="chg-h">Trend change</h3>
 <ul class="chg">{#each c.rows as r}<li class:none={r.kg == null}><span class="p">{SPAN(r.days)}</span><b class="n">{r.kg == null ? '—' : `${signed(r.kg)} kg`}</b><span class="w">{r.word ?? 'Not enough weigh-ins'}</span>{#if r.perWeek != null && r.days >= 7}<span class="rate">{signed(r.perWeek)} kg/wk</span>{/if}</li>{/each}</ul>
 {#if detail}<p class="small">Garmin weigh-ins; body fat appears where the scale records it.</p>{/if}
 <button class="na" disabled aria-describedby="w-na"><Icon name="scale" size={15}/>Add a weight</button><p class="small" id="w-na">Not available yet: adding a weight is a Garmin write.</p>
 {#if !detail}<button class="link more" onclick={() => store.navigate('metric/weight')}>Comparisons and all history<Icon name="chevron" size={13}/></button>{/if}
</section>
<style>
 .wc-h{display:flex;justify-content:space-between;align-items:flex-start}.wc-h small{display:block;font-size:11px;color:var(--text3)}.wc-h b{font-size:28px;font-weight:600}.wc-h b span{font:500 12px Inter;color:var(--text3);margin-left:3px}.wc-h em{display:block;font-style:normal;font-size:12px;color:var(--text2)}
 .fresh{margin:4px 0 8px;font-size:12px;color:var(--text3);display:flex;gap:5px;align-items:center}.fresh.stale{color:#FFB547}
 .key{display:flex;gap:14px;margin:2px 0 6px;font-size:11px;color:var(--text3)}.key span{display:inline-flex;gap:5px;align-items:center}.key .d{width:6px;height:6px;border-radius:50%;background:#A9B4C8}.key .l{width:14px;height:2px;background:#E4E8F0}
 .chg-h{font:600 10.5px Inter;letter-spacing:.08em;text-transform:uppercase;color:var(--text3);margin:8px 0 2px}
 .chg{list-style:none;margin:0;padding:0}.chg li{display:flex;flex-wrap:wrap;align-items:baseline;gap:2px 12px;padding:7px 2px;border-top:1px solid var(--line);font-size:13px}
 .chg .p{min-width:56px;color:var(--text2)}.chg b{font-size:15px;font-weight:600;min-width:64px}.chg .w{flex:1;color:var(--text)}.chg .rate{color:var(--text3);font-size:11.5px}.chg li.none b,.chg li.none .w{color:var(--text3);font-size:12px}
 .na{display:flex;gap:8px;align-items:center;justify-content:center;width:100%;margin-top:10px;min-height:40px;border-radius:12px;border:1px dashed var(--line);background:none;color:var(--text3);font:600 13px Inter}
 .more{display:inline-flex;gap:4px;align-items:center;font-size:12px;margin-top:4px}
</style>
