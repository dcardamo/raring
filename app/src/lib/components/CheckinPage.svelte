<script lang="ts">
 import type {CheckinView} from '../generated/CheckinView';
 import type {CheckinCard} from '../generated/CheckinCard';
 import type {WeekReview} from '../generated/WeekReview';
 import type {LocalAction} from '../generated/LocalAction';
 import Card from './Card.svelte';
 import Gauge from './Gauge.svelte';
 import Icon from './Icon.svelte';
 import ConsistencyChart from './ConsistencyChart.svelte';
 import WeekReviewCard from './WeekReviewCard.svelte';
 import {colors, toneColor} from '../charts';
 import {num, signed} from '../format';
 import {addDays} from '../history';
 // The weekly review: the goals decision first, then the week's readings in
 // one grid (value, change against the week before, each day labelled), then
 // intensity, timing, and one win and one focus.
 let {checkin, review, act, navigate, busy = false}: {checkin: CheckinView, review: WeekReview | null, act: (a: LocalAction) => void, navigate: (route: string) => void, busy?: boolean} = $props();
 const DAYS = ['M', 'T', 'W', 'T', 'F', 'S', 'S'];
 // Scores share a 0–100 scale; other readings scale to the week's highest.
 const top = (c: CheckinCard) => c.area === 'readiness' || c.area === 'sleep' ? 100 : Math.max(1, ...c.series.map(v => v ?? 0));
 const fmt = (c: CheckinCard, v: number | null) => v == null ? '—' : num(v, c.area === 'health' ? 1 : 0);
 const change = (c: CheckinCard) => c.value == null ? 'No readings this week' : c.previous == null ? 'No week before to compare' : Math.abs(c.value - c.previous) < 1 ? 'Steady on the week before' : `${signed(c.value - c.previous)} on the week before`;
</script>
{#if review}<WeekReviewCard {review} {act} {navigate} {busy}/>{/if}
<section class="card cw" aria-label="Your week">
 <h3>Your week</h3>
 <div class="cw-grid">
  {#each checkin.cards as c (c.area)}
   <div class="cw-row" role="group" aria-label={`${c.headline}: ${c.value == null ? 'no readings' : `${fmt(c, c.value)} ${c.unit}`.trim()}, ${change(c)}`}>
    <div class="cw-h"><b>{c.headline}</b><span class="n" style:color={c.tone === 'neutral' ? undefined : toneColor(c.tone)}>{fmt(c, c.value)}</span>{#if c.value != null}<small>{c.area === 'health' ? 'years' : 'daily avg'}</small>{/if}
     <em>{#if c.value != null && c.previous != null && Math.abs(c.value - c.previous) >= 1}<Icon name={c.value < c.previous ? 'trend-down' : 'trend-up'} size={13}/>{/if}{change(c)}</em></div>
    <div class="cw-bars" aria-hidden="true">{#each c.series as v, i}<span class="cw-bar"><small>{fmt(c, v)}</small><span class="cw-col"><i class:gap={v == null} style:height={v == null ? '0' : Math.max(2, v / top(c) * 100) + '%'} style:background={c.tone === 'neutral' ? colors.ink : toneColor(c.tone)}></i></span><b>{DAYS[i]}</b></span>{/each}</div>
   </div>
  {/each}
 </div>
</section>
{#if checkin.intensity}{@const im = checkin.intensity}
 <Card title="Intensity minutes" extra="intensity-review"><div class="cw-im"><Gauge size={150} stroke={12} value={im.total} max={im.goal ?? Math.max(im.total, 1)} display={num(im.total)} caption={im.goal != null ? `of ${num(im.goal)} min` : 'min'} color={im.goal != null && im.total >= im.goal ? colors.good : colors.ink}/>
  <div><b>{im.goal == null ? 'Garmin’s weekly total' : im.total >= im.goal ? "Garmin's goal reached" : `${num(Math.ceil(im.goal - im.total))} min short of Garmin's goal`}</b><span class="pill">{num(im.moderate)} moderate · {num(im.vigorous)} vigorous, counted twice</span></div></div></Card>
{/if}
{#if checkin.timing}{@const t = checkin.timing}
 <Card title="Sleep timing" extra="timing-review"><div class="checkin-content">
  <ConsistencyChart nights={t.nights} date={addDays(checkin.weekStart, 6)} days={7}/>
  <div class="timing-ranges">{#if t.bedRange}<div><span>Bedtimes</span><b class="n">{t.bedRange[0]}–{t.bedRange[1]}</b></div>{/if}{#if t.wakeRange}<div><span>Wake times</span><b class="n">{t.wakeRange[0]}–{t.wakeRange[1]}</b></div>{/if}</div>
  {#if t.aligned + t.ahead + t.behind > 0}<span class="pill">Garmin: {[[t.aligned, 'aligned'], [t.behind, 'behind'], [t.ahead, 'ahead']].filter(([n]) => Number(n) > 0).map(([n, w]) => `${n} ${w}`).join(' · ')}</span>{/if}
  <h3>THE TIMING FOCUS</h3><p>{t.focus}</p></div></Card>
{/if}
{#if checkin.win || checkin.focus || (checkin.vo2maxChange != null && Math.abs(checkin.vo2maxChange) >= 1) || (checkin.fitnessAgeChange != null && Math.abs(checkin.fitnessAgeChange) >= 1)}
 <Card title="Win and focus"><div class="cw-wf">
  {#if checkin.vo2maxChange != null && Math.abs(checkin.vo2maxChange) >= 1}<span class="pill">VO₂ max {signed(checkin.vo2maxChange)}</span>{/if}{#if checkin.fitnessAgeChange != null && Math.abs(checkin.fitnessAgeChange) >= 1}<span class="pill">Fitness Age {signed(checkin.fitnessAgeChange)}</span>{/if}
  {#if checkin.win}<h3>THE WIN</h3><p>{checkin.win}</p>{/if}
  {#if checkin.focus}<h3>THE FOCUS</h3><p>{checkin.focus}</p>{/if}
 </div></Card>
{/if}
<style>
 .cw h3{margin:0 0 10px}
 .cw-grid{display:grid;grid-template-columns:repeat(auto-fill,minmax(260px,1fr));gap:16px 20px}
 .cw-row{display:grid;gap:8px}
 .cw-h{display:flex;flex-wrap:wrap;align-items:baseline;gap:4px 8px}.cw-h b{font:600 14px Inter;color:var(--text);flex-basis:100%}
 .cw-h .n{font:600 26px Outfit;line-height:1}.cw-h small{font-size:12px;color:var(--text2)}
 .cw-h em{display:flex;align-items:center;gap:4px;font-style:normal;font-size:12px;color:var(--text2);margin-left:auto}
 .cw-bars{display:grid;grid-template-columns:repeat(7,minmax(0,1fr));gap:4px;height:84px}
 .cw-bar{display:flex;flex-direction:column;align-items:center;min-width:0;height:100%}
 .cw-bar small{font:500 11px Inter;color:var(--text2);white-space:nowrap}
 .cw-col{flex:1;width:100%;display:flex;align-items:flex-end;justify-content:center;border-bottom:1px solid var(--line)}
 .cw-bar i{display:block;width:70%;max-width:22px;border-radius:3px 3px 0 0;opacity:.85}.cw-bar i.gap{opacity:0}
 .cw-bar b{font:600 11px Inter;color:var(--text3);margin-top:3px}
 .cw-im{display:flex;flex-wrap:wrap;gap:12px 18px;align-items:center}.cw-im b{display:block;font:600 16px Outfit;margin-bottom:8px}
 .cw-wf h3{color:var(--text3);font:600 11px Inter;letter-spacing:.14em;margin:12px 0 4px}.cw-wf p{margin:0;color:var(--text)}.cw-wf .pill{margin-right:6px}
</style>
