<script lang="ts">
 import type {SleepPlan} from '../generated/SleepPlan';
 import {duration} from '../format';
 import {nightStrip,wakeLabel} from '../tonight';
 import Card from './Card.svelte';
 import Icon from './Icon.svelte';
 import SleepPlanEvidence from './SleepPlanEvidence.svelte';
 import {declutter} from '../declutter';
 // `evidence` adds the "Why this plan" sheet; Today opens Sleep instead. Today's `mode` follows
 // the time of day: one line until the plan leads, then the evening, wind-down or late-night card.
 type Mode='full'|'line'|'evening'|'windDown'|'lateNight';
 let {plan,open,evidence=false,mode='full',now=new Date(),lead=false}:{plan:SleepPlan;open?:()=>void;evidence?:boolean;mode?:Mode;now?:Date;lead?:boolean}=$props();
 let why=$state(false);
 let strip=$derived(mode==='evening'||mode==='windDown'?nightStrip(plan,now):null);
 let bedWindow=$derived(plan.bedtimeStart?`${plan.bedtimeStart}–${plan.bedtime}`:plan.bedtime);
 let garmin=$derived(plan.bedtimeSource==='garmin');
 let wake=$derived(wakeLabel(plan.wakeNights)==='reference'?'Wake reference':'Usual wake');
 let needSource=$derived(plan.source==='tonight'?'Garmin\'s need for tonight':plan.source==='average'?'14-day average of Garmin\'s sleep need':'Garmin\'s latest sleep need');
 let wakes=$derived(`median of ${plan.wakeNights} recorded wake ${plan.wakeNights===1?'time':'times'}`);
</script>

{#snippet stripView()}
 {#if strip}
  <div class="strip" aria-hidden="true" data-chart="night-strip" {@attach declutter}>
   <div class="strip-bar"><i class="wd" style:left={strip.windDown+'%'} style:width={strip.start-strip.windDown+'%'}></i><i class="bw" style:left={strip.start+'%'} style:width={Math.max(1,strip.end-strip.start)+'%'}></i>{#if strip.now!=null}<u data-mark="now" style:left={strip.now+'%'}></u>{/if}</div>
   <div class="strip-ticks">{#each strip.ticks as tick,i}<span data-thin class:first={i===0} class:last={i===strip.ticks.length-1} style:left={tick.left+'%'}>{tick.label}</span>{/each}</div>
   <div class="strip-key"><span><i class="wd"></i>Wind down</span><span><i class="bw"></i>{garmin?'Bedtime window':'Asleep by'}</span>{#if strip.now!=null}<span><i class="now"></i>Now</span>{/if}</div>
  </div>
 {/if}
{/snippet}

{#if mode==='line'}
 <button class="card plan-line" onclick={open}>
  <span class="line-icon"><Icon name="moon" size={18}/></span>
  <span class="line-main"><b>Tonight <span>· {garmin?'Garmin plan':'Raring estimate'}</span></b><span>Wind down <b class="n">{plan.windDown}</b> · {garmin?'bed':'asleep by'} <b class="n">{bedWindow}</b></span></span>
  <Icon name="arrow-right" size={14}/>
 </button>
{:else if mode==='windDown' || mode==='lateNight'}
 <Card title={mode==='windDown'?'Wind down':'Tonight'} onclick={open} extra="sleep-plan{lead?' lead':''}">
  {#snippet aside()}<span class="pill"><Icon name="moon" size={13}/> {garmin?'Garmin plan':'Raring estimate'}</span>{/snippet}
  <div class="calm">
   <span>{mode==='lateNight'?(garmin?'Tonight\'s window was':'Tonight\'s asleep-by was'):(garmin?'Bedtime window':'Asleep by')}</span>
   <b class="n">{bedWindow}</b>
   {#if mode==='windDown'}<small>Sleep need <b class="n">{duration(plan.needMin)}</b></small>{/if}
  </div>
  {@render stripView()}
  {#if mode!=='windDown'}<p>Tomorrow's plan follows your next sleep.</p>{/if}
  <div class="foot"><span>Tonight’s plan</span><Icon name="arrow-right" size={16}/></div>
 </Card>
{:else}
<Card title="Tonight's sleep" onclick={open} extra="sleep-plan{lead?' lead':''}">
 {#snippet aside()}<span class="pill"><Icon name="moon" size={13}/> {garmin?'Garmin plan':'Raring estimate'}</span>{/snippet}
 <div class="plan-row">
  <div><span>Wind down</span><b class="n">{plan.windDown}</b></div>
  {#if plan.bedtimeStart}
   <div class="window"><span>Bedtime</span><b class="n">{plan.bedtimeStart}–{plan.bedtime}</b></div>
  {:else}
   <div class="window"><span>Asleep by</span><b class="n">{plan.bedtime}</b></div>
  {/if}
  <div><span>Sleep need</span><b class="n">{duration(plan.needMin)}</b></div>
 </div>
 {@render stripView()}
 {#if plan.stale}
  <p>No sleep has synced today; this estimate uses older nights.</p>
 {:else if garmin}
  <p>{plan.reason ?? 'Garmin\'s bedtime window and sleep need for tonight.'}</p>
 {:else}
  <p>{wake} {plan.wakeTime} ({wakes}) minus {plan.source==='tonight'?'tonight\'s need':plan.source==='average'?'your 14-day average need':'your latest need'}.</p>
 {/if}
 {#if evidence}
  <button class="why" aria-expanded={why} onclick={()=>why=!why}>Why this plan <span class="chevron" class:open={why}><Icon name="arrow-right" size={13}/></span></button>
  {#if why}
   <SleepPlanEvidence {plan}/>
  {/if}
 {/if}
 {#if mode==='evening'}<div class="foot"><span>Tonight’s plan</span><Icon name="arrow-right" size={16}/></div>{/if}
</Card>
{/if}

<style>
 .plan-row{display:grid;grid-template-columns:auto 1fr auto;align-items:end;gap:12px}
 .plan-row>div{display:grid;gap:6px}
 .plan-row>.window{text-align:center}
 .plan-row>div:last-child{text-align:right}
 .plan-row span{font-size:11px;color:var(--text2);white-space:nowrap}
 .plan-row b{font-size:26px;font-weight:500;line-height:1;white-space:nowrap}
 p{font-size:12px;line-height:1.5;color:var(--text2);margin-top:14px}
 .why{display:inline-flex;align-items:center;gap:4px;margin-top:10px;padding:6px 0;background:none;border:0;color:var(--text);font:600 12px Inter}
 .chevron{display:inline-flex;transform:rotate(90deg);transition:transform .15s}
 .chevron.open{transform:rotate(-90deg)}
 :global(.sleep-plan){container-type:inline-size}
 .plan-line{display:flex;align-items:center;gap:10px;width:100%;padding:12px 14px;color:var(--text);text-align:left;font:inherit}
 .plan-line>:global(svg){color:var(--text3);flex:none}
 .line-icon{display:flex;color:var(--text2)}
 .line-main{flex:1;min-width:0;display:grid;gap:2px}
 .line-main>b{font:600 13px Inter}
 .line-main>b span{font-weight:500;color:var(--text2)}
 .line-main>span{font-size:12px;color:var(--text2)}
 .line-main>span b{font:600 13px Outfit;color:var(--text)}
 .calm{display:grid;gap:6px;justify-items:start}
 .calm span{font-size:12px;color:var(--text2)}
 .calm>b{font-size:34px;font-weight:500;line-height:1}
 .calm small{font-size:12px;color:var(--text2)}
 .calm small b{font:600 13px Outfit;color:var(--text)}
 .strip{margin-top:16px}
 .strip-bar{position:relative;height:10px;border-radius:5px;background:rgba(255,255,255,.06)}
 .strip-bar i{position:absolute;top:0;bottom:0;border-radius:5px}
 .wd{background:rgba(228,232,240,.22)}
 .bw{background:var(--ink)}
 .strip-bar u{position:absolute;top:-4px;width:3px;height:18px;border-radius:2px;background:var(--text);transform:translateX(-50%);box-shadow:0 0 0 2px var(--bg)}
 .strip-ticks{position:relative;height:1.5em;margin-top:6px;font:500 11px Inter}
 .strip-ticks span{position:absolute;transform:translateX(-50%);color:var(--text3)}
 .strip-ticks .first{transform:none}
 .strip-ticks .last{transform:translateX(-100%)}
 .strip-key{display:flex;flex-wrap:wrap;gap:4px 14px;margin-top:6px;font-size:12px;color:var(--text2)}
 .strip-key span{display:flex;align-items:center;gap:5px}
 .strip-key i{width:10px;height:10px;border-radius:3px}
 .strip-key i.now{width:3px;height:12px;border-radius:2px;background:var(--text)}
 .foot{display:flex;justify-content:space-between;align-items:center;margin-top:14px;padding-top:12px;border-top:1px solid var(--line);font-size:12px;color:var(--ink)}
 @container (max-width:310px){.plan-row{gap:8px}.plan-row b{font-size:21px}}
</style>
