<script lang="ts">
 import type {Snippet} from 'svelte';
 import type {TrainingView} from '../generated/TrainingView';
 import {masonry} from '../masonry';
 import Card from '../components/Card.svelte';
 import Icon from '../components/Icon.svelte';
 import Gauge from '../components/Gauge.svelte';
 import Chip from '../components/Chip.svelte';
 import WhySheet from '../components/WhySheet.svelte';
 import PlannedSessionCard from '../components/PlannedSessionCard.svelte';
 import WeekBars from '../components/WeekBars.svelte';
 import LaneChart from '../components/LaneChart.svelte';
 import LoadFocus from '../components/LoadFocus.svelte';
 import Sparkline from '../components/Sparkline.svelte';
 import HistoryChart from '../components/HistoryChart.svelte';
 import BodyMap from '../components/BodyMap.svelte';
 import GearCard from '../components/GearCard.svelte';
 import ActivityRow from '../components/ActivityRow.svelte';
 import {colors,toneColor} from '../charts';
 import {dateLabel,elapsed,pace,signed,num} from '../format';
 let {trainingData,header,navigate}:{trainingData:TrainingView,header:Snippet,navigate:(route:string)=>void}=$props();
</script>
 <div class="page-hero">{@render header()}{#if trainingData.status}<span class="status-pill pill" style:color={toneColor(trainingData.statusTone)}>{trainingData.status}</span>{/if}
 {#if trainingData.acute}<section class="semi"><Gauge size={300} stroke={16} sweep={180} value={trainingData.acute.value} max={trainingData.acute.max} caption="Acute load" label={trainingData.acute.label} color={toneColor(trainingData.acute.tone)} band={trainingData.acute.band} marker/><span class="semi-low">Low</span><span class="semi-high">High</span></section>{/if}<div class="headline training-head"><h2>{trainingData.headline}</h2><div class="chips">{#each trainingData.chips as chip}<Chip {chip}/>{/each}</div>{#if trainingData.statusWhy}<div class="why-line"><WhySheet why={trainingData.statusWhy} label={trainingData.status?`Why ${trainingData.status}?`:'Why?'}/></div>{/if}</div></div><div class="dashboard" {@attach masonry}>
 <PlannedSessionCard planned={trainingData.planned}/>
 {#if trainingData.race}<Card title="Upcoming race" extra="upcoming-race"><div class="planned-race"><b>{trainingData.race.name}</b><span>{trainingData.race.daysUntil===0?'Today':trainingData.race.daysUntil===1?'Tomorrow':`${trainingData.race.daysUntil} days`} · {dateLabel(trainingData.race.date)}</span>{#if trainingData.raceState==='partial'}<span>Only the first 10 calendar events checked.</span>{/if}</div></Card>{:else}<Card title="Upcoming race" extra="upcoming-race"><p class="small">{trainingData.raceState==='empty'?'No upcoming race in Garmin’s next 90 days.':trainingData.raceState==='stale'?'Race calendar needs a refresh.':trainingData.raceState==='partial'?'Race schedule incomplete: Garmin returned the first 10 events.':'Race calendar unavailable.'}</p></Card>{/if}
 {#if trainingData.lanes?.lanes.length}{@const tw=trainingData.lanes}{@const today=Math.round((Date.parse(tw.today)-Date.parse(tw.start))/864e5)}<Card title="This week" extra="week-lanes">{#each tw.lanes as lane (lane.kind)}<LaneChart {lane} {today}/>{/each}<p class="small">Line: so far this week. {tw.lanes.some(l=>l.usual)?`Band: your usual week, the middle half of ${Math.max(...tw.lanes.map(l=>l.weeks))} recent complete weeks; dotted: its middle.`:'Your usual week appears after three complete weeks of history.'} Dashed: your goal.</p></Card>{:else if trainingData.week.length}<Card title="This week"><WeekBars days={trainingData.week}/></Card>{/if}{#if trainingData.loadFocus.length}<Card title="Load focus · 4 weeks"><LoadFocus bars={trainingData.loadFocus}/></Card>{/if}
 {#if trainingData.vo2max!=null || trainingData.races.length}<div class="row">{#if trainingData.vo2max!=null}<Card title="VO₂ max" onclick={()=>navigate('metric/vo2max')}><div class="big n">{num(trainingData.vo2max)}</div>{#if trainingData.vo2maxChange30d!=null && Math.abs(trainingData.vo2maxChange30d)>=1}<div class="small"><b style:color={trainingData.vo2maxChange30d>0?colors.good:colors.poor}>{signed(trainingData.vo2maxChange30d)}</b> in 30 days</div>{/if}<Sparkline values={trainingData.vo2maxTrend} w={138} h={30} color={colors.ink}/></Card>{/if}{#if trainingData.races.length}<Card title="Race times"><div class="race">{#each trainingData.races as race}<div><span>{race.label}</span><b class="n">{elapsed(race.seconds)}</b></div>{/each}</div></Card>{/if}</div>{/if}
 {#if trainingData.thresholds?.length}<Card title="Thresholds & acclimation"><p class="small">Garmin readings · available with supported watches and recorded activities</p><div class="race threshold-list">{#each trainingData.thresholds as reading}<div><span>{({runningHr:'Lactate threshold HR',runningPace:'Lactate threshold pace',runningPower:'Running threshold power',cyclingFtp:'Cycling FTP',heat:'Heat acclimation',altitude:'Acclimated altitude'} as Record<string,string>)[reading.kind]??reading.kind}<small>Recorded {dateLabel(reading.recordedOn)}{reading.stale?' · stale':''}</small></span><b class="n">{reading.unit.startsWith('s/')?pace(reading.value).split('/')[0]:num(reading.value)}{reading.unit==='%'?'':' '}{reading.unit.startsWith('s/')?reading.unit.slice(1):reading.unit}</b></div>{/each}</div></Card>{/if}
 {#if trainingData.enduranceScore!=null || trainingData.hillScore!=null || trainingData.runningToleranceKm!=null}<Card title="More metrics"><div class="metrics">{#if trainingData.enduranceScore!=null}<button class="metric-link" onclick={()=>navigate('metric/endurance')}><b>{num(trainingData.enduranceScore)}</b><span>Endurance</span></button>{/if}{#if trainingData.hillScore!=null}<button class="metric-link" onclick={()=>navigate('metric/hill')}><b>{num(trainingData.hillScore)}</b><span>Hill score</span></button>{/if}{#if trainingData.runningToleranceKm!=null}<button class="metric-link" onclick={()=>navigate('metric/runningTolerance')}><b>{num(trainingData.runningToleranceKm)} km</b><span>Run tolerance</span></button>{/if}</div></Card>{/if}
 <Card title="Load history"><HistoryChart values={[]} label="Acute load" metric="acuteLoad" initialRange="days30"/></Card>
 {#if trainingData.strength}{@const st=trainingData.strength}<Card title="Strength · this week" onclick={()=>navigate('strength')}>{#snippet aside()}{#if st.records}<span class="pill ok"><Icon name="star" size={12}/>{st.records===1?'1 record':`${st.records} records`}</span>{/if}{/snippet}<div class="strength-week"><div><div class="big n">{st.sets}</div><div class="small">sets · {st.sessionsWithSets===1?'1 session':`${st.sessionsWithSets} sessions`} with sets{#if st.strengthActivities>st.sessionsWithSets}<br>{st.strengthActivities-st.sessionsWithSets} without sets{/if}</div></div><BodyMap figure={st.figure} muscles={st.muscles} width={96} max={12} legend={false}/></div></Card>{/if}
 {#if trainingData.gear}<GearCard gear={trainingData.gear}/>{/if}
 {#if trainingData.workouts.length}<Card title="Recent workouts">{#each trainingData.workouts as a}<div class="workout-item"><ActivityRow activity={a} open={()=>navigate('workout/'+a.id)}/></div>{/each}</Card>{/if}
</div>
<style>
 .planned-race{display:flex;flex-direction:column;gap:5px}
 .planned-race b{font:600 17px Outfit,Inter,sans-serif}
 .planned-race span{color:var(--text2);font-size:13px}
</style>
