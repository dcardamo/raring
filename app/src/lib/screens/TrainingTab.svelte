<script lang="ts">
 import type {Snippet} from 'svelte';
 import type {TrainingView} from '../generated/TrainingView';
 import type {MuscleRange} from '../generated/MuscleRange';
 import type {TrendRange} from '../generated/TrendRange';
 import type {IconName} from '../components/Icon.svelte';
 import {masonry} from '../masonry';
 import Card from '../components/Card.svelte';
 import Icon from '../components/Icon.svelte';
 import WhySheet from '../components/WhySheet.svelte';
 import PlannedSessionCard from '../components/PlannedSessionCard.svelte';
 import WeekBars from '../components/WeekBars.svelte';
 import LaneChart from '../components/LaneChart.svelte';
 import LoadFocus from '../components/LoadFocus.svelte';
 import HistoryChart from '../components/HistoryChart.svelte';
 import BodyMap from '../components/BodyMap.svelte';
 import ActivityRow from '../components/ActivityRow.svelte';
 import RangePicker from '../components/RangePicker.svelte';
 import {muscleNames} from '../strength';
 import {SPORT_ICONS, SPORT_NAMES} from '../goals';
 import {colors, toneColor} from '../charts';
 import {dateLabel, elapsed, pace, signed, num} from '../format';
 // Training: this week's lanes, then one story per section. Fitness answers
 // three questions (enough or too much? improving? ready for what?);
 // Strength is led by the goal and muscle gaps; Activities shows what each
 // recording worked, in Garmin's words.
 let {trainingData, header, navigate, section = $bindable('fitness'), muscles}: {trainingData: TrainingView, header: Snippet, navigate: (route: string) => void,
  section?: 'fitness' | 'strength' | 'activities', muscles: (range: TrendRange) => Promise<MuscleRange | null>} = $props();
 const SECTIONS = [['fitness', 'Fitness'], ['strength', 'Strength'], ['activities', 'Activities']] as const;
 const TH: Record<string, string> = {runningHr: 'Lactate threshold HR', runningPace: 'Lactate threshold pace', runningPower: 'Running threshold power', cyclingFtp: 'Cycling FTP', heat: 'Heat acclimation', altitude: 'Acclimated altitude'};
 let acute = $derived(trainingData.acute);
 let tw = $derived(trainingData.lanes);
 let todayIndex = $derived(tw ? Math.round((Date.parse(tw.today) - Date.parse(tw.start)) / 864e5) : 0);
 let strengthLane = $derived(tw?.lanes.find(l => l.kind === 'strength') ?? null);
 // Muscles over the chosen range, read from the core.
 let muscleRange = $state<TrendRange>('days30'), muscleView = $state<MuscleRange | null>(null), muscleError = $state(false);
 let request = 0;
 $effect(() => { if (section !== 'strength') return; const r = muscleRange, id = ++request; muscleError = false; void muscles(r).then(v => { if (id === request) muscleView = v }).catch(() => { if (id === request) muscleError = true }); });
 // Activities, filtered by Garmin's activity icon.
 const ICON_NAMES = Object.fromEntries(Object.entries(SPORT_ICONS).map(([s, i]) => [i, SPORT_NAMES[s as keyof typeof SPORT_NAMES]])) as Record<string, string>;
 let filter = $state<IconName | 'all'>('all'), limit = $state(12);
 let kinds = $derived([...new Set(trainingData.workouts.map(a => a.icon))]);
 let shown = $derived(trainingData.workouts.filter(a => filter === 'all' || a.icon === filter));
 const te = (v: number | null, label: string | null, kind: string) => v == null ? null : `${kind} ${num(v, 1)}${label ? ` · ${label}` : ''}`;
</script>
<div class="page-hero">{@render header()}{#if trainingData.status}<span class="status-pill pill" style:color={toneColor(trainingData.statusTone)}>{trainingData.status}</span>{/if}
 <div class="headline training-head"><h2>{trainingData.headline}</h2>{#if trainingData.statusWhy}<div class="why-line"><WhySheet why={trainingData.statusWhy} label={trainingData.status ? `Why ${trainingData.status}?` : 'Why?'}/></div>{/if}</div></div>
<div class="dashboard" {@attach masonry}>
 {#if tw?.lanes.length}<Card title="This week" extra="week-lanes" help="training-week">{#each tw.lanes as lane (lane.kind)}<LaneChart {lane} today={todayIndex}/>{/each}
  {#if tw.bySport.length}<p class="mix">{#each tw.bySport as m}<span><Icon name={SPORT_ICONS[m.sport]} size={14}/>{SPORT_NAMES[m.sport]} <b class="n">{Math.round(m.minutes)} min</b></span>{/each}</p>{/if}
  <p class="small">Line: so far this week. {tw.lanes.some(l => l.usual) ? `Band: your usual week, the middle half of ${Math.max(...tw.lanes.map(l => l.weeks))} recent complete weeks; dotted: its middle.` : 'Your usual week appears after three complete weeks of history.'} Dashed: your goal.</p></Card>
 {:else if trainingData.week.length}<Card title="This week"><WeekBars days={trainingData.week}/></Card>{/if}
</div>
<div class="train-tabs" role="tablist" aria-label="Training section">{#each SECTIONS as [k, l]}<button role="tab" aria-selected={section === k} onclick={() => section = k}>{l}</button>{/each}</div>
<div class="dashboard" {@attach masonry}>
{#if section === 'fitness'}
 <Card title="Enough, or too much?" extra="q-load" help="acute-load">
  {#if acute}
   <div class="load"><div><small>Acute load · Garmin</small><b class="n">{num(acute.value)}</b><span style:color={toneColor(acute.tone)}>{acute.label}</span></div>
    <div class="lbar" role="img" aria-label={`Acute load ${num(acute.value)}${acute.band ? `; Garmin's optimal range ${acute.band[0]}–${acute.band[1]}` : ''}; scale to ${acute.max}`}>
     {#if acute.band}<i class="band" style:left={acute.band[0] / acute.max * 100 + '%'} style:width={(acute.band[1] - acute.band[0]) / acute.max * 100 + '%'}></i>{/if}<u style:left={Math.min(100, acute.value / acute.max * 100) + '%'}></u></div>
    <div class="lticks"><span>0</span>{#if acute.band}<span>Optimal {acute.band[0]}–{acute.band[1]}</span>{/if}<span>{acute.max}</span></div></div>
  {:else}<p class="small">Garmin hasn’t sent an acute load yet.</p>{/if}
  {#if trainingData.loadFocus.length}<h4>Load focus · last 4 weeks</h4><LoadFocus bars={trainingData.loadFocus}/>{/if}
 </Card>
 <Card title="Load history"><HistoryChart values={[]} label="Acute load" metric="acuteLoad" initialRange="days30"/></Card>
 <Card title="Is my fitness improving?" extra="q-fitness" help="vo2max">
  {#if trainingData.vo2max != null}<button class="vo" onclick={() => navigate('metric/vo2max')} aria-label={`VO₂ max ${num(trainingData.vo2max)}, open its history`}><span><small>VO₂ max · Garmin</small><b class="n">{num(trainingData.vo2max)}</b>{#if trainingData.vo2maxChange30d != null && Math.abs(trainingData.vo2maxChange30d) >= 1}<em style:color={trainingData.vo2maxChange30d > 0 ? colors.good : colors.poor}>{signed(trainingData.vo2maxChange30d)} in 30 days</em>{/if}</span><span class="go">History<Icon name="chevron" size={14}/></span></button>
  {:else}<p class="small">No VO₂ max estimate from Garmin yet.</p>{/if}
  {#if trainingData.enduranceScore != null || trainingData.hillScore != null || trainingData.runningToleranceKm != null}<div class="metrics">{#if trainingData.enduranceScore != null}<button class="metric-link" onclick={() => navigate('metric/endurance')}><b>{num(trainingData.enduranceScore)}</b><span>Endurance</span></button>{/if}{#if trainingData.hillScore != null}<button class="metric-link" onclick={() => navigate('metric/hill')}><b>{num(trainingData.hillScore)}</b><span>Hill score</span></button>{/if}{#if trainingData.runningToleranceKm != null}<button class="metric-link" onclick={() => navigate('metric/runningTolerance')}><b>{num(trainingData.runningToleranceKm)} km</b><span>Run tolerance</span></button>{/if}</div>{/if}
 </Card>
 <Card title="What am I ready for?" extra="q-ready" help="ready-for">
  {#if trainingData.races.length}<h4>Garmin’s race predictions</h4><div class="race">{#each trainingData.races as race}<div><span>{race.label}</span><b class="n">{elapsed(race.seconds)}</b></div>{/each}</div>{/if}
  {#if trainingData.thresholds?.length}<h4>Thresholds & acclimation</h4><div class="race threshold-list">{#each trainingData.thresholds as reading}<div><span>{TH[reading.kind] ?? reading.kind}<small>Recorded {dateLabel(reading.recordedOn)}{reading.stale ? ' · stale' : ''}</small></span><b class="n">{reading.unit.startsWith('s/') ? pace(reading.value).split('/')[0] : num(reading.value)}{reading.unit === '%' ? '' : ' '}{reading.unit.startsWith('s/') ? reading.unit.slice(1) : reading.unit}</b></div>{/each}</div>{/if}
  <h4>Upcoming race</h4>
  {#if trainingData.race}<div class="planned-race"><b>{trainingData.race.name}</b><span>{trainingData.race.daysUntil === 0 ? 'Today' : trainingData.race.daysUntil === 1 ? 'Tomorrow' : `${trainingData.race.daysUntil} days`} · {dateLabel(trainingData.race.date)}</span>{#if trainingData.raceState === 'partial'}<span>Only the first 10 calendar events checked.</span>{/if}</div>
  {:else}<p class="small">{trainingData.raceState === 'empty' ? 'No upcoming race in Garmin’s next 90 days.' : trainingData.raceState === 'stale' ? 'Race calendar needs a refresh.' : trainingData.raceState === 'partial' ? 'Race schedule incomplete: Garmin returned the first 10 events.' : 'Race calendar unavailable.'}</p>{/if}
 </Card>
 <PlannedSessionCard planned={trainingData.planned}/>
{:else if section === 'strength'}
 {#if trainingData.strength || strengthLane}{@const st = trainingData.strength}
  <Card title="Strength this week" onclick={() => navigate('strength')} extra="q-strength">
   <div class="sw"><div><b class="n big">{strengthLane?.cumulative[todayIndex] ?? st?.strengthActivities ?? 0}{#if strengthLane?.goal}<small> of {strengthLane.goal}</small>{/if}</b><span class="small">sessions{strengthLane?.goal ? ' · your goal' : ''}</span></div>
    {#if st}<div><b class="n big">{st.sets}</b><span class="small">sets · {st.sessionsWithSets === 1 ? '1 session' : `${st.sessionsWithSets} sessions`} with sets{st.strengthActivities > st.sessionsWithSets ? ` · ${st.strengthActivities - st.sessionsWithSets} without` : ''}</span></div>{/if}
    {#if st?.records}<span class="pill ok"><Icon name="star" size={12}/>{st.records === 1 ? '1 record' : `${st.records} records`}</span>{/if}</div>
  </Card>
  <Card title="Muscles" extra="q-muscles" help="strength-gaps">
   <RangePicker value={muscleRange} label="Muscle range" onchange={r => muscleRange = r}/>
   {#if muscleView}
    <div class="mus"><BodyMap figure={muscleView.figure} muscles={muscleView.muscles} width={120} max={Math.max(8, Math.round(muscleView.days / 2))} legend={false}/>
     <ul aria-label="Sets per muscle group">{#each [...muscleView.muscles].sort((a, b) => a.sets - b.sets) as m}<li class:gap={muscleView.gaps.includes(m.muscle)}><span>{muscleNames[m.muscle]}</span><b class="n">{num(m.sets)}</b><small>sets</small></li>{/each}</ul></div>
    <p class="small">{muscleView.gaps.length ? `Few or no sets since ${dateLabel(muscleView.from)}: ${muscleView.gaps.map(m => muscleNames[m]).join(', ')}.` : `Every group had more than ${muscleView.gapAt} set${muscleView.gapAt === 1 ? '' : 's'} since ${dateLabel(muscleView.from)}.`} Counted from Garmin’s exercise names; each set counts for each primary muscle.</p>
   {:else if muscleError}<p class="small">Muscles couldn’t be read. Try again.</p>
   {:else}<p class="small">Muscles appear once a strength session has sets.</p>{/if}
  </Card>
  <Card title="Lifts and exercises" onclick={() => navigate('strength')}><p class="small">Max weight or volume for each lift, bests at each rep count, and every session’s sets.</p></Card>
 {:else}<Card title="No strength sessions yet"><p class="small">Start a Strength activity on your watch and press lap after each set.</p><button class="secondary" onclick={() => navigate('help/strength-start')}>Lift with your watch</button></Card>{/if}
{:else}
 <Card title="What each activity worked" extra="q-activities" help="activity-contribution">
  <p class="small">Garmin’s benefit and training effect for each recording; open one for more.</p>
  {#if kinds.length > 1}<div class="chips2" role="group" aria-label="Filter by sport"><button aria-pressed={filter === 'all'} onclick={() => { filter = 'all'; limit = 12; }}>All</button>{#each kinds as k}<button aria-pressed={filter === k} onclick={() => { filter = k; limit = 12; }}><Icon name={k} size={14}/>{ICON_NAMES[k] ?? 'Other'}</button>{/each}</div>{/if}
  {#each shown.slice(0, limit) as a (a.id)}
   {@const lines = [a.benefit && `Primary benefit · ${a.benefit}`, te(a.aerobicTe, a.aerobicLabel, 'Aerobic'), te(a.anaerobicTe, a.anaerobicLabel, 'Anaerobic'), a.intensityMin != null && `${num(a.intensityMin)} intensity min`].filter(Boolean)}
   <div class="workout-item"><ActivityRow activity={a} open={() => navigate('workout/' + a.id)}/>
    {#if lines.length}<div class="tags">{#each lines as l}<span class="pill">{l}</span>{/each}</div>{/if}</div>
  {:else}<p class="small">No recorded activities yet.</p>{/each}
  {#if shown.length > limit}<button class="secondary" onclick={() => limit += 12}>Show more</button>{/if}
 </Card>
{/if}
</div>
<style>
 .train-tabs{display:grid;grid-template-columns:repeat(3,1fr);gap:4px;padding:4px;margin:10px 0;border:1px solid var(--line);border-radius:14px;background:var(--card);max-width:520px}
 .train-tabs button{min-height:40px;border-radius:10px;border:0;background:none;color:var(--text2);font:600 13px Inter}
 .train-tabs button[aria-selected=true]{background:rgba(255,255,255,.1);color:var(--text)}
 .mix{display:flex;flex-wrap:wrap;gap:6px 14px;margin:8px 0 0;font-size:12.5px;color:var(--text2)}.mix span{display:inline-flex;gap:5px;align-items:center}.mix b{color:var(--text)}
 h4{font:600 11px Inter;letter-spacing:.08em;text-transform:uppercase;color:var(--text2);margin:14px 0 6px}
 .load{display:grid;gap:6px}.load>div:first-child{display:flex;gap:10px;align-items:baseline}.load small{font-size:12px;color:var(--text2)}.load b{font-size:24px}.load span{font:600 13px Inter}
 .lbar{position:relative;height:10px;border-radius:5px;background:rgba(255,255,255,.07)}.lbar .band{position:absolute;top:0;bottom:0;background:rgba(61,220,151,.35);border-radius:5px}
 .lbar u{position:absolute;top:-4px;bottom:-4px;width:3px;margin-left:-1.5px;background:var(--ink);border-radius:2px}
 .lticks{display:flex;justify-content:space-between;font-size:11px;color:var(--text2)}
 .vo{display:flex;justify-content:space-between;align-items:center;width:100%;background:none;border:0;color:var(--text);text-align:left;min-height:52px;padding:0}
 .vo small{display:block;font-size:12px;color:var(--text2)}.vo b{font-size:28px;margin-right:8px}.vo em{font:600 12px Inter;font-style:normal}.go{display:inline-flex;gap:4px;align-items:center;font:600 13px Inter;color:var(--text2)}
 .planned-race{display:flex;flex-direction:column;gap:5px}.planned-race b{font:600 17px Outfit,Inter,sans-serif}.planned-race span{color:var(--text2);font-size:13px}
 .sw{display:flex;flex-wrap:wrap;gap:16px;align-items:center}.sw>div{display:grid}.big{font-size:26px}.big small{font-size:14px;color:var(--text2)}
 .mus{display:flex;gap:14px;align-items:flex-start;margin-top:10px}.mus ul{list-style:none;margin:0;padding:0;flex:1;display:grid;gap:3px;font-size:13px}
 .mus li{display:grid;grid-template-columns:1fr auto 30px;gap:6px;align-items:baseline}.mus li small{color:var(--text2);font-size:11px}.mus li.gap span{color:var(--fair);font-weight:600}
 .chips2{display:flex;flex-wrap:wrap;gap:6px;margin:8px 0}.chips2 button{display:inline-flex;gap:5px;align-items:center;min-height:36px;padding:6px 12px;border-radius:999px;border:1px solid var(--line);background:none;color:var(--text2);font:600 12.5px Inter}
 .chips2 button[aria-pressed=true]{background:var(--ink);color:var(--bg)}
 .tags{display:flex;flex-wrap:wrap;gap:6px;margin:4px 0 10px}
 @container(max-width:340px){.mus{flex-direction:column;align-items:center}.mus ul{width:100%}}
</style>
