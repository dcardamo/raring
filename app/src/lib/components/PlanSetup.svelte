<script lang="ts">
 import {untrack} from 'svelte';
 import type {PlanView} from '../generated/PlanView';
 import type {PlanTargets} from '../generated/PlanTargets';
 import type {PlanUpdate} from '../generated/PlanUpdate';
 import type {PlanPreset} from '../generated/PlanPreset';
 import {masonry} from '../masonry';
 import {num} from '../format';
 import {planIcons,weekOrder,weekdays} from '../plan';
 import Card from './Card.svelte';
 import Icon from './Icon.svelte';
 import PlanGoals from './PlanGoals.svelte';
 let {view,save,back}:{view:PlanView;save:(update:PlanUpdate)=>void;back:()=>void}=$props();
 // Edits stay local while the saved view reloads underneath.
 let targets=$state<PlanTargets>(untrack(()=>structuredClone($state.snapshot(view.targets))));
 let paused=$state(untrack(()=>view.week.paused));
 const presets:PlanPreset[]=['strength','endurance','sleep','custom'];
 const presetNames:Record<PlanPreset,string>={strength:'Strength',endurance:'Endurance',sleep:'Sleep',custom:'Custom'};
 let intensity=$derived(targets.intensityMin??view.garminIntensityGoal??150);
 function commit(next:PlanTargets){
  targets=next;
  save({targets:$state.snapshot(targets) as PlanTargets,paused});
 }
 function change(patch:Partial<PlanTargets>){commit({...targets,...patch,preset:'custom'})}
 function choose(preset:PlanPreset){
  const base=view.presets.find(p=>p.preset===preset);
  // Custom keeps the current numbers; a preset replaces them but keeps rest days.
  commit(base?{...structuredClone($state.snapshot(base)),restDays:[...targets.restDays]}:{...targets,preset});
 }
 const clamp=(n:number,max:number)=>Math.max(0,Math.min(max,n));
</script>

{#snippet stepper(label:string,value:number,display:string,step:number,max:number,set:(n:number)=>void)}
 <div class="stepper"><button aria-label={`Less ${label}`} disabled={value<=0} onclick={()=>set(clamp(value-step,max))}>−</button><b class="n" aria-live="polite">{display}</b><button aria-label={`More ${label}`} disabled={value>=max} onclick={()=>set(clamp(value+step,max))}>+</button></div>
{/snippet}

<button class="back" onclick={back}><Icon name="back" size={20}/> <span>This week</span></button>
<header class="top"><div><div class="eyebrow">Weekly plan</div><h1>Targets</h1></div></header>
<div class="dashboard" {@attach masonry}>
 <Card title="Start from" extra="plan-presets">
  <div class="presets" role="radiogroup" aria-label="Start from">{#each presets as preset}<button role="radio" aria-checked={targets.preset===preset} class:on={targets.preset===preset} onclick={()=>choose(preset)}>{presetNames[preset]}</button>{/each}</div>
  <p class="hint">Sets all four targets. Change any of them below.</p>
 </Card>
 <Card extra="plan-targets">
  <div class="target"><span class="icon"><Icon name={planIcons.strength} size={18}/></span><div class="label"><b>Strength</b><span>Sessions a week</span></div>{@render stepper('strength sessions',targets.strengthSessions,targets.strengthSessions?String(targets.strengthSessions):'Off',1,14,n=>change({strengthSessions:n}))}</div>
  <div class="target"><span class="icon"><Icon name={planIcons.intensity} size={18}/></span><div class="label"><b>Intensity</b><span>{targets.intensityMin==null?"Garmin's weekly goal":'Your weekly minutes'}</span>{#if targets.intensityMin!=null && view.garminIntensityGoal!=null}<button class="link" onclick={()=>change({intensityMin:null})}>Use Garmin's {num(view.garminIntensityGoal)}</button>{:else if targets.intensityMin==null}<span class="pill">Garmin</span>{/if}</div>{@render stepper('intensity minutes',intensity,targets.intensityMin==null&&view.garminIntensityGoal==null?'—':intensity?String(intensity):'Off',10,600,n=>change({intensityMin:n}))}</div>
  <div class="target"><span class="icon"><Icon name={planIcons.zone2} size={18}/></span><div class="label"><b>Zone 2</b><span>Minutes in heart-rate zone 2</span></div>{@render stepper('Zone 2 minutes',targets.zone2Min,targets.zone2Min?String(targets.zone2Min):'Off',10,600,n=>change({zone2Min:n}))}</div>
  <div class="target"><span class="icon"><Icon name={planIcons.asleepOnTime} size={18}/></span><div class="label"><b>Asleep on time</b><span>Nights within 15 min</span></div>{@render stepper('nights asleep on time',targets.asleepNights,targets.asleepNights?String(targets.asleepNights):'Off',1,7,n=>change({asleepNights:n}))}</div>
 </Card>
 <Card title="Rest days" extra="plan-rest">
  <div class="days">{#each weekOrder(view.weekStartsOn) as day}<button aria-label={weekdays[day]} aria-pressed={targets.restDays.includes(day)} class:on={targets.restDays.includes(day)} onclick={()=>change({restDays:targets.restDays.includes(day)?targets.restDays.filter(d=>d!==day):[...targets.restDays,day].sort()})}>{weekdays[day][0]}</button>{/each}</div>
  <p class="hint">Nothing is due on a rest day. Weeks start on {weekdays[view.weekStartsOn]}, like Garmin's intensity minutes.</p>
 </Card>
 <Card extra="plan-pause">
  <label class="target pause"><span class="icon"><Icon name="pause" size={18}/></span><div class="label"><b>Pause this week</b><span>Travel or illness. Left out of history.</span></div><input type="checkbox" role="switch" checked={paused} onchange={(e)=>{paused=e.currentTarget.checked;commit(targets)}}/></label>
 </Card>
 {#if view.configured}<Card title="Last week on these targets" extra="plan-last"><PlanGoals items={view.lastWeek.items} compact/></Card>{/if}
</div>

<style>
 .presets{display:grid;grid-template-columns:repeat(4,1fr);gap:4px;padding:4px;border-radius:14px;background:rgba(255,255,255,.05)}
 .presets button{min-height:44px;border:0;border-radius:11px;background:none;color:var(--text2);font-weight:600;padding:0 4px}
 .presets .on{background:rgba(255,255,255,.12);color:var(--text)}
 .hint{font-size:13px;color:var(--text2);margin-top:12px;line-height:1.5}
 .target{display:grid;grid-template-columns:34px 1fr auto;align-items:center;gap:12px;padding:14px 0;border-top:1px solid var(--line)}
 .target:first-child{border:0;padding-top:0}
 .target:last-child{padding-bottom:0}
 .icon{width:34px;height:34px;border-radius:11px;display:grid;place-items:center;background:rgba(255,255,255,.06);color:var(--text2)}
 .label{min-width:0;display:flex;flex-direction:column;align-items:flex-start;gap:3px}
 .target b{font:600 16px Inter}
 .label span:not(.pill){font-size:13px;color:var(--text2)}
 .pill{margin-top:4px}
 .link{background:none;border:0;padding:6px 0;color:var(--text);font-size:13px;font-weight:600;text-decoration:underline;text-underline-offset:3px;min-height:32px}
 .stepper{display:flex;align-items:center;gap:2px;padding:3px;border-radius:14px;background:rgba(255,255,255,.05)}
 .stepper button{width:44px;height:44px;border:0;border-radius:11px;background:rgba(255,255,255,.08);font-size:22px;color:var(--text)}
 .stepper button:disabled{color:var(--text3);background:none}
 .stepper b{min-width:46px;text-align:center;font-size:20px;font-weight:600}
 .days{display:grid;grid-template-columns:repeat(7,1fr);gap:6px}
 .days button{min-height:44px;border:0;border-radius:12px;background:rgba(255,255,255,.06);color:var(--text2);font-weight:600}
 .days .on{background:var(--ink);color:var(--bg)}
 .pause{cursor:pointer;padding:0}
 .pause input{appearance:none;width:52px;height:30px;border-radius:15px;background:rgba(255,255,255,.14);position:relative;cursor:pointer;margin:0}
 .pause input::after{content:'';position:absolute;top:3px;left:3px;width:24px;height:24px;border-radius:50%;background:var(--text2);transition:left .15s}
 .pause input:checked{background:var(--ink)}
 .pause input:checked::after{left:25px;background:var(--bg)}
</style>
