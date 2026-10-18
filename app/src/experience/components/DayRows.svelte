<script lang="ts">
 import StressTimeline from '../../lib/components/StressTimeline.svelte';
 import TrendSpark from './TrendSpark.svelte';
 import Icon from '../../lib/components/Icon.svelte';
 import {store} from '../lib/store.svelte';
 import {stressCard,bodyBatteryDay} from '../lib/views';
 import {stagesFor} from '../lib/world';
 import {dur,dayLabel} from '../lib/dates';
 let ctx=$derived(store.ctx),night=$derived(ctx.phase==='waiting'||ctx.stale?null:ctx.night);
 let stress=$derived(stressCard(ctx));
 let bb=$derived(ctx.bodyBattery==null?null:bodyBatteryDay(ctx));
 const palette:Record<string,string>={deep:'#2F6FE0',light:'#5C9BFF',rem:'#A9C8FF',awake:'#E4E8F0'};
</script>
<section class="day-rows" aria-label="Your day, sleep energy and stress">
 <h2>Your day</h2>
 <button class="dr" onclick={()=>store.navigate('sleep')} aria-label="Sleep: open last night">
  <span class="dr-label"><Icon name="moon" size={17}/><b>Sleep</b><em>{night?`${night.bed}–${night.wake}`:'Awaiting upload'}</em></span>
  <span class="sleep-track">{#if night}{#each stagesFor(ctx.displayedDay,night.asleep) as s}<i style:flex={s.minutes} style:background={palette[s.stage]}></i>{/each}{/if}</span>
  <span class="dr-value n">{night?dur(night.asleep):'—'}<small>{dayLabel(ctx.displayedDay)}</small></span>
 </button>
 <button class="dr" onclick={()=>store.navigate('metric/bodyBattery')} aria-label="Body Battery: open energy through the day">
  <span class="dr-label"><Icon name="bolt" size={17}/><b>Body Battery</b><em>At watch upload</em></span>
  <span class="trace">{#if bb}<TrendSpark values={bb.points.map(p=>p[1])} w={145} h={42} color="#3DDC97"/>{:else}<span class="gap">Awaiting data</span>{/if}</span>
  <span class="dr-value n">{ctx.bodyBattery??'—'}<small>{bb?`+${bb.charged} overnight`:''}</small></span>
 </button>
 <button class="dr stress" onclick={()=>store.navigate('metric/stress')} aria-label="Stress: open the day chart">
  <span class="dr-label"><Icon name="pulse" size={17}/><b>Stress</b><em>{stress.level?stress.level[0].toUpperCase()+stress.level.slice(1):'Unavailable'} · day so far</em></span>
  <span class="dr-value n">{stress.average??'—'}<small>Garmin</small></span>
  <span class="stress-chart"><StressTimeline {stress} compact/></span>
 </button>
</section>
<style>
 .day-rows{margin-top:18px;border:1px solid var(--line);border-radius:22px;background:var(--card);padding:14px}.day-rows h2{font:600 11px Inter;letter-spacing:.13em;text-transform:uppercase;color:var(--text3);margin-bottom:5px}.dr{display:grid;grid-template-columns:118px 1fr auto;gap:10px;align-items:center;width:100%;padding:12px 0;background:none;border:0;border-bottom:1px solid var(--line);color:var(--text);text-align:left}.dr:last-child{border-bottom:0;padding-bottom:0}.dr-label{display:grid;grid-template-columns:18px 1fr;gap:3px 6px}.dr-label>:global(svg){color:var(--text3)}.dr-label b{font:600 13px Inter}.dr-label em{grid-column:1/-1;font:400 10.5px Inter;color:var(--text3);font-style:normal}.dr-value{font-size:24px;font-weight:600;text-align:right;line-height:1.15}.dr-value small{display:block;font:500 10px Inter;color:var(--text3);margin-top:4px}.sleep-track{display:flex;height:14px;border-radius:7px;overflow:hidden;background:rgba(255,255,255,.04)}.sleep-track i{height:100%;display:block}.trace{min-width:0}.trace :global(svg){width:100%}.gap{font-size:11px;color:var(--text3)}.dr.stress{grid-template-columns:1fr auto;row-gap:4px}.stress-chart{grid-column:1/-1;display:block;min-width:0}.stress-chart :global(svg){max-height:94px;width:100%}
 @container(max-width:300px){.dr{grid-template-columns:1fr auto}.trace,.sleep-track{grid-row:2;grid-column:1/-1}.dr-label{grid-template-columns:18px 1fr}}
</style>
