<script lang="ts">
 import type {WorkoutView} from '../generated/WorkoutView';
 import {masonry} from '../masonry';
 import Card from '../components/Card.svelte';
 import Icon from '../components/Icon.svelte';
 import Gauge from '../components/Gauge.svelte';
 import StrengthWorkout from '../components/StrengthWorkout.svelte';
import StrengthRefresh from '../components/StrengthRefresh.svelte';
import type {StrengthSession} from '../generated/StrengthSession';
 import RouteMap from '../components/RouteMap.svelte';
 import HrChart from '../components/HrChart.svelte';
 import WorkoutSplits from '../components/WorkoutSplits.svelte';
 import WorkoutMore from '../components/WorkoutMore.svelte';
 import HowItCounted from '../components/HowItCounted.svelte';
 import {colors} from '../charts';
 import {elapsed,pace,num} from '../format';
 let {workoutData,navigate,help,showMap=true,refresh}:{workoutData:WorkoutView,navigate:(route:string)=>void,help:()=>void,showMap?:boolean,refresh:(id:string)=>Promise<StrengthSession|null>}=$props();
 let sets=$derived(workoutData.strength),lifted=$derived(sets?.setsRecorded===true);
</script>
{#if sets && lifted}<StrengthWorkout workout={workoutData} strength={sets} {help} refresh={()=>refresh(workoutData.summary.id)}/>{/if}<div class="dashboard" {@attach masonry}>{#if sets && !lifted}<Card title="No sets recorded"><p class="small untracked">Your watch timed this session as one set, without reps or exercises. Press lap after each set next time.</p><button class="watch-open" onclick={()=>navigate('help/strength-start')}>Lift with your watch <Icon name="arrow-right" size={16}/></button><div class="sets-refresh-wrap"><StrengthRefresh checkedAt={sets.checkedAt} refresh={()=>refresh(workoutData.summary.id)}/></div></Card>{/if}{#if !lifted && (workoutData.summary.load!=null || workoutData.summary.benefit || workoutData.recoveryHours!=null || workoutData.summary.aerobicTe!=null || workoutData.summary.anaerobicTe!=null)}<Card extra="workout-overview"><div class="workout-hero"><div>{#if workoutData.summary.load!=null}<div class="eyebrow">Training load</div><div class="load-number n">{num(workoutData.summary.load)}</div>{/if}<div class="tags">{#if workoutData.summary.benefit}<span class="pill">{workoutData.summary.benefit}</span>{/if}{#if workoutData.recoveryHours!=null}<span class="pill"><Icon name="clock" size={13}/>{num(workoutData.recoveryHours,1)}h recovery</span>{/if}</div></div><div class="effects">{#if workoutData.summary.aerobicTe!=null}<div><Gauge size={82} stroke={7} value={workoutData.summary.aerobicTe} max={5} caption="Aerobic" color={colors.ink}/><small>{workoutData.summary.aerobicLabel}</small></div>{/if}{#if workoutData.summary.anaerobicTe!=null}<div><Gauge size={82} stroke={7} value={workoutData.summary.anaerobicTe} max={5} caption="Anaer." color={colors.ink}/><small>{workoutData.summary.anaerobicLabel}</small></div>{/if}</div></div></Card>{/if}
 {#if workoutData.stats.length && !lifted}<Card><div class="stats">{#each workoutData.stats as s}<div><b class="n">{s.key==='pace'?pace(s.value).replace('/km',''):s.key==='duration'?elapsed(s.value):num(s.value,s.key==='distance'||s.key==='speed'?1:0)}{#if s.key==='pace'}<small>/km</small>{:else if s.unit!=='s' && s.unit!=='bpm' && s.unit!=='kcal'}<small>{s.unit}</small>{/if}</b><span>{s.label}</span></div>{/each}</div></Card>{/if}
 {#if showMap && workoutData.route.length}<Card extra="map-card"><RouteMap route={workoutData.route}/></Card>{/if}{#if workoutData.heartRate.length}<Card title="Heart rate"><HrChart samples={workoutData.heartRate} floors={workoutData.zoneFloors} durationS={workoutData.summary.durationS}/></Card>{/if}
 {#if workoutData.summary.zoneSeconds.length}<Card title="Time in zones">{@const maxZone=Math.max(...workoutData.summary.zoneSeconds,1)}{@const total=workoutData.summary.zoneSeconds.reduce((a,b)=>a+b,0)}{#each [...workoutData.summary.zoneSeconds].reverse() as seconds,i}<div class="zone-row"><b>Z{5-i}</b><div class="bar"><i style:width={seconds/maxZone*100+'%'} style:background={colors.z[4-i]}></i></div><span class="n">{elapsed(seconds)}</span><small>{Math.round(seconds/Math.max(total,1)*100)}%</small></div>{/each}</Card>{/if}
 {#if workoutData.splits.length && !lifted}<WorkoutSplits splits={workoutData.splits}/>{/if}
</div>{#if workoutData.counted && (workoutData.counted.goals || workoutData.counted.planItem || workoutData.counted.bests.length)}<HowItCounted counted={workoutData.counted}/>{/if}{#if !lifted}<WorkoutMore workout={workoutData} {navigate}/>{/if}
