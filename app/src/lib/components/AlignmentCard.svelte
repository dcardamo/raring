<script lang="ts">
 import type {SleepAlignment} from '../generated/SleepAlignment';
 import {toneColor} from '../charts';
 import {minuteClock,rhythmAxis,rhythmTicks} from '../rhythm';
 import Card from './Card.svelte';
 import {declutter} from '../declutter';
 let {alignment}:{alignment:SleepAlignment}=$props();
 let latest=$derived(alignment.latest);
 let rows=$derived(alignment.state==='unavailable'?[]:alignment.history.filter(n=>n.window||n.sleep));
 let axis=$derived(rhythmAxis([...(latest?[latest]:[]),...rows]));
 let ticks=$derived(rhythmTicks(axis));
 // Labels left, the overnight plot in the middle, Garmin's status word right; HTML rows
 // grow with the system font size instead of colliding.
 const pct=(m:number)=>(m-axis[0])/(axis[1]-axis[0])*100;
 const span=(r:[number,number])=>Math.max(1,pct(r[1])-pct(r[0]));
 const weekday=(date:string)=>new Date(`${date}T12:00:00`).toLocaleDateString('en-GB',{weekday:'short'});
</script>

<Card title="Sleep alignment" extra="alignment">
 {#snippet aside()}{#if latest?.status}<span class="pill" style:color={toneColor(latest.tone)}>{latest.status}</span>{:else}<span class="pill">{alignment.state==='calibrating'?'Calibrating':'Unavailable'}</span>{/if}{/snippet}
 {#if latest && (latest.window || latest.sleep)}
  <div class="timeline" role="img" aria-label="Sleep against Garmin's optimal sleep window" data-chart="alignment" {@attach declutter}>
   {#snippet grid()}{#each ticks as t}<b class="gl" style:left={pct(t)+'%'}></b>{/each}{/snippet}
   <div class="ar"><span class="al">Optimal</span><span class="at">{@render grid()}{#if latest.window}<i class="optimal" style:left={pct(latest.window[0])+'%'} style:width={span(latest.window)+'%'}></i>{/if}</span><span></span></div>
   <div class="ar"><span class="al">Slept</span><span class="at">{@render grid()}{#if latest.sleep}<i class="slept" style:left={pct(latest.sleep[0])+'%'} style:width={span(latest.sleep)+'%'}></i>{/if}</span><span></span></div>
   {#if rows.length>1}
    <hr>
    {#each rows as n}
     <div class="ar night" data-date={n.date}>
      <span class="al">{weekday(n.date)}</span>
      <span class="at past">{@render grid()}{#if n.window}<i class="window" style:left={pct(n.window[0])+'%'} style:width={span(n.window)+'%'}></i>{/if}{#if n.sleep}<i class="sleep" style:left={pct(n.sleep[0])+'%'} style:width={span(n.sleep)+'%'}></i>{/if}</span>
      <span class="st" style:color={n.status?toneColor(n.tone):'var(--text2)'}>{n.status ?? '—'}</span>
     </div>
    {/each}
   {/if}
   <div class="ar"><span></span><span class="ticks">{#each ticks as t}<span data-thin style:left={pct(t)+'%'}>{minuteClock(t)}</span>{/each}</span><span></span></div>
  </div>
  <div class="times">
   {#if latest.window}<span>Optimal <b class="n">{minuteClock(latest.window[0])}–{minuteClock(latest.window[1])}</b></span>{/if}
   {#if latest.sleep}<span>Slept <b class="n">{minuteClock(latest.sleep[0])}–{minuteClock(latest.sleep[1])}</b></span>{/if}
  </div>
 {/if}
 <p>{alignment.detail}</p>
 <p class="note">Garmin's optimal window follows your longer-term rhythm; tonight's bedtime window is a separate recommendation.</p>
</Card>

<style>
 .timeline{display:grid;grid-template-columns:max-content minmax(0,1fr) max-content;gap:6px 8px;align-items:center;font-size:12px;color:var(--text2)}
 .ar{display:contents}
 .al{min-width:32px}
 .st{font-weight:600;text-align:right}
 hr{grid-column:1/-1;width:100%;margin:2px 0;border:0;border-top:1px solid rgba(255,255,255,.08)}
 .at{position:relative;height:16px}
 .at.past{height:12px}
 .gl{position:absolute;top:-3px;bottom:-3px;width:1px;background:rgba(255,255,255,.06)}
 .at i{position:absolute;top:0;bottom:0;border-radius:8px;box-sizing:border-box}
 .optimal{border:1.5px dashed var(--ink)}
 .slept{background:var(--ink)}
 .window{border:1px dashed color-mix(in srgb,var(--ink) 60%,transparent)}
 .at .sleep{top:3px;bottom:3px;border-radius:3px;background:var(--ink);opacity:.75}
 .ticks{position:relative;height:1.4em;font-size:11px}
 .ticks span{position:absolute;transform:translateX(-50%);white-space:nowrap}
 .times{display:flex;flex-wrap:wrap;gap:6px 16px;margin-top:8px;font-size:12px;color:var(--text2)}
 .times b{color:var(--text);font-weight:500;margin-left:4px}
 p{font-size:12px;line-height:1.5;color:var(--text2);margin-top:12px}
 .note{color:var(--text3);font-size:11px;margin-top:8px}
</style>
