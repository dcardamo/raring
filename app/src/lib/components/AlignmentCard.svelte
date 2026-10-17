<script lang="ts">
 import type {SleepAlignment} from '../generated/SleepAlignment';
 import {toneColor,colors} from '../charts';
 import {minuteClock,rhythmAxis,rhythmTicks} from '../rhythm';
 import Card from './Card.svelte';
 let {alignment}:{alignment:SleepAlignment}=$props();
 let latest=$derived(alignment.latest);
 let rows=$derived(alignment.state==='unavailable'?[]:alignment.history.filter(n=>n.window||n.sleep));
 let axis=$derived(rhythmAxis([...(latest?[latest]:[]),...rows]));
 let ticks=$derived(rhythmTicks(axis));
 // Labels left, the overnight plot in the middle, Garmin's status word right.
 const left=40,width=222;
 const x=(m:number)=>left+(m-axis[0])/(axis[1]-axis[0])*width;
 const span=(r:[number,number])=>Math.max(2,x(r[1])-x(r[0]));
 const weekday=(date:string)=>new Date(`${date}T12:00:00`).toLocaleDateString('en-GB',{weekday:'short'});
 const top=72,step=20;
 let height=$derived(rows.length>1?top+16+rows.length*step:72);
</script>

<Card title="Sleep alignment" extra="alignment">
 {#snippet aside()}{#if latest?.status}<span class="pill" style:color={toneColor(latest.tone)}>{latest.status}</span>{:else}<span class="pill">{alignment.state==='calibrating'?'Calibrating':'Unavailable'}</span>{/if}{/snippet}
 {#if latest && (latest.window || latest.sleep)}
  <svg viewBox="0 0 326 {height}" class="timeline" role="img" aria-label="Sleep against Garmin's optimal sleep window">
   {#each ticks as t}<line x1={x(t)} x2={x(t)} y1="2" y2={height-18} stroke="rgba(255,255,255,.06)"/><text x={x(t)} y={height-4} text-anchor="middle" fill="#8E98AC" font-size="11">{minuteClock(t)}</text>{/each}
   <text x="0" y="20" fill="#8E98AC" font-size="11">Optimal</text>
   {#if latest.window}<rect class="optimal" x={x(latest.window[0])} y="8" width={span(latest.window)} height="16" rx="8" fill="none" stroke={colors.ink} stroke-width="1.5" stroke-dasharray="4 3"/>{/if}
   <text x="0" y="48" fill="#8E98AC" font-size="11">Slept</text>
   {#if latest.sleep}<rect class="slept" x={x(latest.sleep[0])} y="36" width={span(latest.sleep)} height="16" rx="8" fill={colors.ink}/>{/if}
   {#if rows.length>1}
    <line x1="0" x2="326" y1={top-6} y2={top-6} stroke="rgba(255,255,255,.08)"/>
    {#each rows as n,i}
     <g class="night" data-date={n.date}>
      <text x="0" y={top+10+i*step} fill="#8E98AC" font-size="11">{weekday(n.date)}</text>
      {#if n.window}<rect x={x(n.window[0])} y={top+i*step} width={span(n.window)} height="12" rx="6" fill="none" stroke={colors.ink} stroke-opacity=".6" stroke-dasharray="3 3"/>{/if}
      {#if n.sleep}<rect x={x(n.sleep[0])} y={top+3+i*step} width={span(n.sleep)} height="6" rx="3" fill={colors.ink} opacity=".75"/>{/if}
      <text x="326" y={top+10+i*step} text-anchor="end" font-size="11" font-weight="600" fill={n.status?toneColor(n.tone):'#8E98AC'}>{n.status ?? '—'}</text>
     </g>
    {/each}
   {/if}
  </svg>
  <div class="times">
   {#if latest.window}<span>Optimal <b class="n">{minuteClock(latest.window[0])}–{minuteClock(latest.window[1])}</b></span>{/if}
   {#if latest.sleep}<span>Slept <b class="n">{minuteClock(latest.sleep[0])}–{minuteClock(latest.sleep[1])}</b></span>{/if}
  </div>
 {/if}
 <p>{alignment.detail}</p>
 <p class="note">Garmin's optimal window follows your longer-term rhythm; tonight's bedtime window is a separate recommendation.</p>
</Card>

<style>
 .timeline{display:block;width:100%}
 .times{display:flex;flex-wrap:wrap;gap:6px 16px;margin-top:8px;font-size:12px;color:var(--text2)}
 .times b{color:var(--text);font-weight:500;margin-left:4px}
 p{font-size:12px;line-height:1.5;color:var(--text2);margin-top:12px}
 .note{color:var(--text3);font-size:11px;margin-top:8px}
</style>
