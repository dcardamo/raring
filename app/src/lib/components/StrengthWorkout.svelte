<script lang="ts">
 import Icon from './Icon.svelte';
 import BodyMap from './BodyMap.svelte';
 import {elapsed,dateLabel,num} from '../format';
 import {muscleNames,weight,lift,unitLabel,flagSummary,repsLabel} from '../strength';
 import type {WorkoutView} from '../generated/WorkoutView';
 import type {StrengthSession} from '../generated/StrengthSession';
 import type {StrengthExercise} from '../generated/StrengthExercise';
import StrengthRefresh from './StrengthRefresh.svelte';
 let {workout,strength,help,refresh}:{workout:WorkoutView,strength:StrengthSession,help:()=>void,refresh:()=>Promise<StrengthSession|null>}=$props();
 let flags=$derived(flagSummary(strength));
 let reps=$derived(repsLabel(strength));
 let avgHr=$derived(workout.stats.find(s=>s.key==='averageHR' || s.key==='avgHr')?.value??null);
 const height=(reps:number|null,max:number)=>Math.max((reps??0)/max*56,3);
 const maxReps=(e:StrengthExercise)=>Math.max(1,...e.sets.map(s=>s.reps??0),...e.lastSets.map(s=>s.reps??0));
 const toCheck=(e:StrengthExercise)=>e.sets.filter(s=>s.flags.length).length;
</script>
<div class="strength-cols">
 <div class="col">
  <section class="card"><div class="hero">
   <div><div class="eyebrow">Sets</div><div class="hero-n n">{strength.sets}</div>{#if strength.records}<span class="pill ok"><Icon name="star" size={12}/>{strength.records===1?'1 record':`${strength.records} records`}</span>{/if}</div>
   <div class="grid2"><div><b class="n">{elapsed(workout.summary.durationS)}</b><span>Time</span></div><div><b class="n">{reps.value}</b><span class:wrap={reps.label!=='Reps'}>{reps.label}</span></div><div><b class="n">{num(workout.summary.load)}</b><span>Load</span></div><div><b class="n">{num(avgHr)}</b><span>Avg HR</span></div></div>
  </div>
   <div class="checked"><StrengthRefresh checkedAt={strength.checkedAt} {refresh}/></div></section>
  {#if flags}<button class="card alert" onclick={help}><span class="al-i"><Icon name="alert" size={19}/></span><span class="al-t"><b>{flags.title}</b><span>{flags.detail}</span></span><Icon name="help" size={18}/></button>{/if}
  <section class="card"><h3>Worked</h3><BodyMap figure={strength.figure} muscles={strength.muscles} max={9}/></section>
 </div>
 <div class="col">
  {#each strength.exercises as e}
   {@const max=maxReps(e)}{@const check=toCheck(e)}
   <section class="card ex">
    <div class="ex-h"><div><b>{e.name??'Not recognized'}</b><span>{e.key==null?"Garmin couldn't name it":e.muscles.map(m=>muscleNames[m]).join(' · ')||'Muscles unknown'}</span></div>
     {#if e.key==null}<span class="pill warn">Unnamed</span>{:else if check}<span class="pill warn">{check} to check</span>{:else if e.record}<span class="pill ok"><Icon name="star" size={12}/>{weight(e.record.weight)} × {e.record.reps}</span>{/if}</div>
    <div class="setbars" role="list" aria-label="Sets: reps and weight">
     {#each e.sets as s,i}
      {@const ghost=e.lastSets[i]}
      <div class="sbar" class:flag={s.flags.length>0} role="listitem" aria-label={`Set ${i+1}: ${s.reps??'no'} reps, ${s.weight==null?'no weight':`${weight(s.weight)} ${unitLabel(strength.unit)}`}${s.record?', record':''}${s.flags.length?', looks off':''}`}>
       <b class="n">{s.reps??'—'}</b>
       <div class="sbar-t">{#if ghost}<u style:height={height(ghost.reps,max)+'px'}></u>{/if}<i style:height={height(s.reps,max)+'px'}>{#if s.record}<Icon name="star" size={12}/>{/if}</i></div>
       <span class="n">{weight(s.weight)}</span>
      </div>
     {/each}
     <div class="sbar key" aria-hidden="true"><b>reps</b><div class="sbar-t"></div><span>{unitLabel(strength.unit)}</span></div>
    </div>
    {#if e.lastDate}<div class="lastl">Last · {dateLabel(e.lastDate)} · <b class="n">{e.lastSets.map(lift).join('  ')}</b></div>{/if}
   </section>
  {/each}
 </div>
</div>
<style>
 .strength-cols{display:grid;gap:0 16px}
 .col{min-width:0}
 .hero{display:flex;justify-content:space-between;gap:12px;align-items:center}
 .hero-n{font:600 60px/1 Outfit;margin:6px 0 10px}
 .grid2{display:grid;grid-template-columns:auto auto;gap:14px 22px}
 .grid2 b{display:block;font:600 21px Outfit}
 .grid2 span{font:500 12px Inter;color:var(--text3);white-space:nowrap}
 .grid2 span.wrap{display:block;max-width:96px;white-space:normal}
 .checked{margin-top:14px;padding-top:12px;border-top:1px solid var(--line)}
 .pill :global(svg){flex:none}
 .pill.warn{background:rgba(255,181,71,.14);color:var(--fair)}
 .alert{display:flex;align-items:center;gap:12px;width:100%;text-align:left;border-color:rgba(255,181,71,.3);color:var(--text3)}
 .al-i{width:36px;height:36px;border-radius:11px;display:grid;place-items:center;background:rgba(255,181,71,.13);color:var(--fair);flex:none}
 .al-t{flex:1;min-width:0}
 .al-t b{display:block;font:600 15px Inter;color:var(--text)}
 .al-t span{font-size:12px;color:var(--text2)}
 .ex-h{display:flex;justify-content:space-between;align-items:flex-start;gap:10px;margin-bottom:12px}
 .ex-h b{display:block;font:600 16px Inter}
 .ex-h span:not(.pill){font-size:12px;color:var(--text3)}
 .setbars{display:flex;gap:8px;align-items:flex-end}
 .sbar{flex:1;max-width:58px;display:flex;flex-direction:column;align-items:center;gap:5px;min-width:0}
 .sbar>b{font:600 15px Outfit}
 .sbar-t{position:relative;width:100%;height:56px;display:flex;align-items:flex-end;justify-content:center}
 .sbar-t i{position:relative;display:grid;place-items:start center;width:100%;border-radius:7px 7px 4px 4px;background:rgba(228,232,240,.78);color:#0B0F1A}
 .sbar-t i :global(svg){margin-top:4px}
 .sbar-t u{position:absolute;left:-3px;right:-3px;bottom:0;border:1.5px dashed rgba(255,255,255,.2);border-radius:9px 9px 6px 6px}
 .sbar>span{font:500 12px Outfit;color:var(--text2);white-space:nowrap}
 .sbar.flag i{background:var(--fair)}
 .sbar.flag>b{color:var(--fair)}
 .sbar.key{flex:0 0 30px}
 .sbar.key b,.sbar.key span{font:600 10px Inter;color:var(--text3)}
 .lastl{margin-top:12px;font-size:12px;color:var(--text3)}
 .lastl b{color:var(--text2);font-weight:600;font-size:13px;word-spacing:3px}
 @media(min-width:660px){
  .strength-cols{grid-template-columns:minmax(0,1fr) minmax(0,1fr);margin-bottom:16px}
  .col>:global(.card:first-child){margin-top:0}
 }
</style>
