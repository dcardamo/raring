<script lang="ts">
 import type {PlanHistoryWeek} from '../generated/PlanHistoryWeek';
 import {colors} from '../charts';
 import Icon from './Icon.svelte';
 let {weeks}:{weeks:PlanHistoryWeek[]}=$props();
 const day=(d:string)=>String(Number(d.slice(8,10)));
</script>

<div class="weeks" role="list" aria-label="Share of targets reached each week">
 {#each weeks as week (week.start)}
  {@const pct=week.completion==null?null:Math.round(week.completion*100)}
  <div class="week" class:now={week.current} role="listitem" aria-label={`Week of ${week.start}: ${week.paused?'paused':pct==null?'no plan or evidence':pct+'%'+(week.partial?' of known targets only':'')}`}>
   <div class="track">{#if week.paused}<span class="paused"><Icon name="pause" size={14}/></span>{:else if pct!=null}<i style:height={Math.max(pct,3)+'%'} style:background={pct>=100&&!week.partial?colors.good:week.current?colors.ink:'rgba(228,232,240,.32)'}></i>{/if}</div>
   <b class="n">{pct==null?'–':`${pct}${week.partial?'*':''}`}</b><span>{week.current?'Now':day(week.start)}</span>
  </div>
 {/each}
</div>
<div class="legend"><span>% of targets reached</span>{#if weeks.some(w=>w.partial)}<span>* Known targets only</span>{/if}<span><i style:background={colors.good}></i>All reached</span><span><Icon name="pause" size={12}/>Paused</span></div>

<style>
 .weeks{display:grid;grid-template-columns:repeat(8,1fr);gap:8px}
 .week{display:grid;justify-items:center;gap:6px;min-width:0}
 .track{width:100%;height:64px;border-radius:10px;background:rgba(255,255,255,.05);display:flex;align-items:flex-end;overflow:hidden;position:relative}
 .now .track{box-shadow:inset 0 0 0 1.5px rgba(228,232,240,.5)}
 .track i{display:block;width:100%;border-radius:8px}
 .paused{position:absolute;inset:0;display:grid;place-items:center;color:var(--text3)}
 b{font-size:15px}
 span{font-size:12px;color:var(--text2)}
 .legend{display:flex;flex-wrap:wrap;gap:6px 16px;margin-top:16px}
 .legend span{display:inline-flex;align-items:center;gap:6px}
 .legend i{width:9px;height:9px;border-radius:2px}
</style>
