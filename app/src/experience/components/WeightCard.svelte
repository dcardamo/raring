<script lang="ts">
 import RangePicker, {rangeDays} from './RangePicker.svelte';
 import HelpButton from './HelpButton.svelte';
 import WeightChart from './WeightChart.svelte';
 import Spark from './Spark.svelte';
 import Icon from '../../lib/components/Icon.svelte';
 import {store} from '../lib/store.svelte';
 import {addDays, dayLabel, dayMonth} from '../lib/dates';
 import {changes, trendSeries, weighIn, ageDays} from '../lib/weight';
 // Weight in one accent colour: trend and scale weight, the range's average
 // and change, how the trend moved over 1–90 days, and how fresh it is. The
 // colour marks the metric, not good or bad; Raring has no weight goal.
 let {detail = false}: {detail?: boolean} = $props();
 const C = '#8FA8FF';
 let ctx = $derived(store.ctx);
 let span = $state(30);
 let c = $derived(changes(ctx));
 let age = $derived(ageDays(c.last, ctx.calendarDate));
 let stale = $derived(age != null && age > 7);
 let end = $derived(ctx.upload.date);
 let series = $derived(trendSeries(ctx, c.last ?? end));
 let pts = $derived.by(() => {
  const days = rangeDays(span, end, ctx.world.historyStart), step = days > 400 ? 14 : days > 90 ? 7 : 1, out: {x: string, scale: number | null, trend: number | null}[] = [];
  for (let i = days - step; i >= 0; i -= step) {
   const d = addDays(end, -i); let v: number | null = null;
   for (let k = 0; k < step && v == null; k++) v = weighIn(ctx, addDays(d, -k));
   out.push({x: dayMonth(d), scale: v, trend: c.last && d <= c.last ? series.get(d) ?? null : null});
  }
  return out;
 });
 let inRange = $derived(pts.map(p => p.trend).filter((v): v is number => v != null));
 let avg = $derived(inRange.length ? inRange.reduce((a, b) => a + b, 0) / inRange.length : null);
 // Same measure as the change rows: trend at the last weigh-in minus the trend that many days before.
 let diff = $derived.by(() => { if (!c.last || c.trend == null) return null; const from = span ? series.get(addDays(c.last, -span)) : series.values().next().value; return from == null ? null : c.trend - from; });
 let scale = $derived(c.last ? weighIn(ctx, c.last) : null);
 const signed = (n: number) => `${n > 0 ? '+' : n < 0 ? '−' : '±'}${Math.abs(n).toFixed(1)}`;
 const SPAN = (d: number) => d === 1 ? '1 day' : `${d} days`;
 const window = (days: number) => c.last ? Array.from({length: Math.max(2, days + 1)}, (_, i) => series.get(addDays(c.last!, i - days)) ?? null) : [];
</script>
<section class="card wc" aria-label="Weight" style:--wc={C}>
 <div class="wc-h"><div><small><i class="kl"></i>Trend weight</small><b class="n">{c.trend != null ? c.trend.toFixed(1) : '—'}<span>kg</span></b>
  <em><i class="ks"></i>Scale {scale != null ? `${scale.toFixed(1)} kg` : '—'} · {c.last ? dayLabel(c.last) : 'no weigh-ins'}</em></div><HelpButton topic="trend-weight"/></div>
 {#if c.last}<p class="fresh" class:stale>{#if stale}<Icon name="clock" size={13}/>{/if}Last weigh-in {age === 0 ? 'today' : age === 1 ? 'yesterday' : `${age} days ago`}{stale ? ' · trend holds until the next one' : ''}</p>{/if}
 <div class="sum"><span><small>Average</small><b class="n">{avg != null ? avg.toFixed(1) : '—'}<em>kg</em></b></span><span><small>Change</small><b class="n">{diff != null ? signed(diff) : '—'}<em>kg</em></b></span><span class="per">{span ? `${span} days to ${c.last ? dayMonth(c.last) : '—'}` : 'all history'}</span></div>
 <WeightChart points={pts} color={C} fmt={v => v.toFixed(1)} label={`Weight, ${span ? `last ${span} days` : 'all history'}: scale weigh-ins and trend, trend ${c.trend?.toFixed(1) ?? 'none'} kg`}/>
 <p class="key"><span><i class="ks"></i>Scale weight</span><span><i class="kl"></i>Trend weight</span></p>
 <RangePicker bind:value={span} label="Weight range"/>
 <h3 class="chg-h">Trend change</h3>
 <ul class="chg">{#each c.rows as r}{@const dir = r.word === 'Losing' ? 'down' : r.word === 'Gaining' ? 'up' : 'flat'}
  <li class:none={r.kg == null}><span class="p">{SPAN(r.days)}</span><Spark values={window(r.days)} color={C} w={56} label={`Trend over ${SPAN(r.days)}`}/><b class="n">{r.kg == null ? '—' : `${signed(r.kg)} kg`}</b>
   <span class="w">{#if r.word}<span class="wd"><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">{#if dir === 'down'}<path d="M3 7l6 6 4-4 8 8M21 11v6h-6"/>{:else if dir === 'up'}<path d="M3 17l6-6 4 4 8-8M21 13V7h-6"/>{:else}<path d="M3 12h16M15 8l4 4-4 4"/>{/if}</svg>{r.word}</span>{#if r.perWeek != null && r.days >= 7}<small>{signed(r.perWeek)} kg/wk</small>{/if}{:else}Not enough weigh-ins{/if}</span></li>
 {/each}</ul>
 {#if detail}<p class="small">Garmin weigh-ins; body fat appears where the scale records it.</p>{/if}
 <button class="na" disabled aria-describedby="w-na"><Icon name="scale" size={15}/>Add a weight</button><p class="small" id="w-na">Not available yet: adding a weight is a Garmin write.</p>
 {#if !detail}<button class="link more" onclick={() => store.navigate('metric/weight')}>Comparisons and all history<Icon name="chevron" size={13}/></button>{/if}
</section>
<style>
 .wc{background:linear-gradient(160deg,color-mix(in srgb,var(--wc) 10%,transparent),var(--card) 55%)}
 .wc-h{display:flex;justify-content:space-between;align-items:flex-start}.wc-h small{display:flex;gap:6px;align-items:center;font-size:11px;color:var(--text3)}.wc-h b{font-size:30px;font-weight:600;color:var(--wc)}.wc-h b span{font:500 12px Inter;color:var(--text3);margin-left:3px}.wc-h em{display:flex;gap:6px;align-items:center;font-style:normal;font-size:12px;color:var(--text2)}
 .kl{display:inline-block;width:14px;height:3px;border-radius:2px;background:var(--wc)}.ks{display:inline-block;width:14px;height:2px;border-radius:2px;background:var(--wc);opacity:.45}
 .fresh{margin:4px 0 8px;font-size:12px;color:var(--text3);display:flex;gap:5px;align-items:center}.fresh.stale{color:#FFB547}
 .sum{display:flex;flex-wrap:wrap;gap:4px 22px;align-items:flex-end;margin:6px 0 4px}.sum small{display:block;font-size:11px;color:var(--text3)}.sum b{font-size:20px;font-weight:600}.sum em{font:500 11px Inter;font-style:normal;color:var(--text3);margin-left:3px}.per{font-size:11px;color:var(--text3);padding-bottom:3px}
 .key{display:flex;gap:14px;margin:0 0 8px;font-size:11px;color:var(--text3)}.key span{display:inline-flex;gap:6px;align-items:center}
 .chg-h{font:600 10.5px Inter;letter-spacing:.08em;text-transform:uppercase;color:var(--text3);margin:12px 0 2px}
 .chg{list-style:none;margin:0;padding:0}.chg li{display:grid;grid-template-columns:52px 56px minmax(0,1fr) auto;align-items:center;gap:4px 10px;padding:8px 2px;border-top:1px solid var(--line);font-size:13px}
 .chg .p{color:var(--text2)}.chg b{font-size:15px;font-weight:600;text-align:right;white-space:nowrap}.chg .w{display:grid;justify-items:end;color:var(--wc);font-weight:600;min-width:74px}.chg .wd{display:inline-flex;gap:5px;align-items:center}.chg .w small{font-weight:500;color:var(--text3);font-size:10.5px}
 @container(max-width:300px){.chg li{grid-template-columns:1fr auto}.chg li :global(svg[role=img]),.chg li :global(.nos){display:none}}.chg li.none b,.chg li.none .w{color:var(--text3);font-size:12px;font-weight:500}
 .na{display:flex;gap:8px;align-items:center;justify-content:center;width:100%;margin-top:10px;min-height:40px;border-radius:12px;border:1px dashed var(--line);background:none;color:var(--text3);font:600 13px Inter}
 .more{display:inline-flex;gap:4px;align-items:center;font-size:12px;margin-top:4px}
</style>
