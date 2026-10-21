<script lang="ts">
 import type {WorkoutView} from '../generated/WorkoutView';
 import Card from '../components/Card.svelte';
 import Icon from '../components/Icon.svelte';
 import Gauge from '../components/Gauge.svelte';
 import StrengthWorkout from '../components/StrengthWorkout.svelte';
import StrengthRefresh from '../components/StrengthRefresh.svelte';
import type {StrengthSession} from '../generated/StrengthSession';
 import RouteMap from '../components/RouteMap.svelte';
 import HrChart from '../components/HrChart.svelte';
 import AmountRows from '../components/AmountRows.svelte';
 import WorkoutSplits from '../components/WorkoutSplits.svelte';
 import WorkoutMore from '../components/WorkoutMore.svelte';
 import HowItCounted from '../components/HowItCounted.svelte';
 import HowItFelt from '../components/HowItFelt.svelte';
 import {colors} from '../charts';
 import {elapsed,num} from '../format';
 import {distance,pace,paceUnit,speed,speedUnit,elevation,elevationUnit} from '../units';
 let {workoutData,navigate,help,showMap=true,refresh}:{workoutData:WorkoutView,navigate:(route:string)=>void,help:()=>void,showMap?:boolean,refresh:(id:string)=>Promise<StrengthSession|null>}=$props();
 let sets=$derived(workoutData.strength),lifted=$derived(sets?.setsRecorded===true);
 let unit=$derived(workoutData.summary.distanceUnit);
 // Heart rate is drawn once: at the top of Along the way when streams exist, otherwise in its own card.
 let alongShown=$derived(!lifted && !!workoutData.along && workoutData.heartRate.length>0);
 // Each zone's bpm range from Garmin's zone floors (Z1 runs up to Z2's floor).
 const zoneRange=(zone:number)=>{const f=workoutData.zoneFloors;if(f.length!==4)return '';return zone===1?`< ${f[0]}`:zone===5?`${f[3]}+`:`${f[zone-2]}–${f[zone-1]-1}`};
 const stat=(key:string,value:number,u:string)=>key==='distance'?distance(value,unit):key==='pace'?pace(value,unit):key==='speed'?num(speed(value,unit),1):key==='elevationGain'?num(elevation(value,unit)):key==='duration'?elapsed(value):num(value);
 const statUnit=(key:string,u:string)=>key==='distance'?'':key==='pace'?paceUnit(unit):key==='speed'?speedUnit(unit):key==='elevationGain'?elevationUnit(unit):u==='s'?'':u;
</script>
{#if sets && lifted}<StrengthWorkout workout={workoutData} strength={sets} {help} refresh={()=>refresh(workoutData.summary.id)}/>{/if}<div class="dashboard">{#if sets && !lifted}<Card title="No sets recorded"><p class="small untracked">Your watch timed this session as one set, without reps or exercises. Press lap after each set next time.</p><button class="watch-open" onclick={()=>navigate('help/strength-start')}>Lift with your watch <Icon name="arrow-right" size={16}/></button><div class="sets-refresh-wrap"><StrengthRefresh checkedAt={sets.checkedAt} refresh={()=>refresh(workoutData.summary.id)}/></div></Card>{/if}{#if !lifted && (workoutData.summary.load!=null || workoutData.summary.benefit || workoutData.recoveryHours!=null || workoutData.summary.aerobicTe!=null || workoutData.summary.anaerobicTe!=null)}<Card extra="workout-overview"><div class="workout-hero"><div>{#if workoutData.summary.load!=null}<div class="eyebrow">Training load</div><div class="load-number n">{num(workoutData.summary.load)}</div>{/if}<div class="tags">{#if workoutData.summary.benefit}<span class="pill">{workoutData.summary.benefit}</span>{/if}{#if workoutData.recoveryHours!=null}<span class="pill"><Icon name="clock" size={13}/>{num(workoutData.recoveryHours,1)}h recovery</span>{/if}</div></div><div class="effects">{#if workoutData.summary.aerobicTe!=null}<div><Gauge size={82} stroke={7} value={workoutData.summary.aerobicTe} max={5} caption="Aerobic" color={colors.ink}/><small>{workoutData.summary.aerobicLabel}</small></div>{/if}{#if workoutData.summary.anaerobicTe!=null}<div><Gauge size={82} stroke={7} value={workoutData.summary.anaerobicTe} max={5} caption="Anaer." color={colors.ink}/><small>{workoutData.summary.anaerobicLabel}</small></div>{/if}</div></div></Card>{/if}
 {#if workoutData.stats.length && !lifted}<Card><div class="stats">{#each workoutData.stats as s}<div><b class="n">{stat(s.key,s.value,s.unit)}{#if statUnit(s.key,s.unit)}<small>{statUnit(s.key,s.unit)}</small>{/if}</b><span>{s.label}</span></div>{/each}</div></Card>{/if}
 {#if workoutData.selfEvaluation}<HowItFelt rating={workoutData.selfEvaluation}/>{/if}
 {#if showMap && workoutData.route.length}<Card extra="map-card"><RouteMap route={workoutData.route} map={workoutData.map} id={workoutData.summary.id}/></Card>{/if}{#if workoutData.heartRate.length && !alongShown}<Card title="Heart rate"><HrChart samples={workoutData.heartRate} floors={workoutData.zoneFloors} durationS={workoutData.summary.durationS} avg={workoutData.summary.avgHr ?? null}/></Card>{/if}
 {#if workoutData.summary.zoneSeconds.length}<Card title="Time in zones"><AmountRows label="Garmin heart-rate zones" unit="min:sec" format={v=>elapsed(v*60)} rows={[...workoutData.summary.zoneSeconds].map((seconds,i)=>({label:'Zone '+(i+1),value:seconds==null?null:seconds/60,text:elapsed(seconds),detail:zoneRange(i+1)?zoneRange(i+1)+' bpm':'Bpm range unavailable',color:colors.z[i]})).reverse()}/></Card>{/if}
 {#if workoutData.splits.length && !lifted}<WorkoutSplits splits={workoutData.splits} ride={workoutData.summary.icon==='bike'} {unit}/>{/if}
</div>{#if workoutData.counted && (workoutData.counted.goals || workoutData.counted.planItem || workoutData.counted.bests.length || workoutData.counted.fitness)}<HowItCounted counted={workoutData.counted}/>{/if}{#if !lifted}<WorkoutMore workout={workoutData} {navigate}/>{/if}
