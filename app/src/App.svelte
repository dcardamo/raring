<script lang="ts">
 import {onMount,tick} from 'svelte';
 import {isTauri} from '@tauri-apps/api/core';
 import {createNavigation,tabs} from './lib/navigation';
 import {shouldSync,CACHE_POLL_MS} from './lib/freshness';
 import {metricSource,metricNames} from './lib/metrics';
 import MetricDetail from './lib/components/MetricDetail.svelte';
 import DailyFocus from './lib/components/DailyFocus.svelte';
 import SleepPlanCard from './lib/components/SleepPlanCard.svelte';
 import VitalsMonitor from './lib/components/VitalsMonitor.svelte';
 import * as api from './lib/api';
 import type {TodayView} from './lib/generated/TodayView';import type {SleepView} from './lib/generated/SleepView';import type {TrainingView} from './lib/generated/TrainingView';import type {WorkoutView} from './lib/generated/WorkoutView';import type {HealthView} from './lib/generated/HealthView';import type {CheckinView} from './lib/generated/CheckinView';import type {AppStatus} from './lib/generated/AppStatus';
 import Icon from './lib/components/Icon.svelte';import BrandMark from './lib/components/BrandMark.svelte';import ReadinessTrend from './lib/components/ReadinessTrend.svelte';
 import Gauge from './lib/components/Gauge.svelte';import Sparkline from './lib/components/Sparkline.svelte';import RangeBar from './lib/components/RangeBar.svelte';import ZoneStrip from './lib/components/ZoneStrip.svelte';import Hypnogram from './lib/components/Hypnogram.svelte';import ConsistencyChart from './lib/components/ConsistencyChart.svelte';import WeekBars from './lib/components/WeekBars.svelte';import LoadFocus from './lib/components/LoadFocus.svelte';import RouteMap from './lib/components/RouteMap.svelte';import HrChart from './lib/components/HrChart.svelte';import Card from './lib/components/Card.svelte';import Chip from './lib/components/Chip.svelte';import Header from './lib/components/Header.svelte';import TabBar from './lib/components/TabBar.svelte';import ActivityRow from './lib/components/ActivityRow.svelte';
 import {colors,toneColor} from './lib/charts';import {duration,elapsed,pace,clock,dateLabel,signed,num} from './lib/format';
 type PageData=TodayView|SleepView|TrainingView|WorkoutView|HealthView|CheckinView;
 let route=$state('today'),trail=$state<string[]>(['today']),data=$state<PageData|null>(null),status=$state<AppStatus|null>(null),settings=$state<api.Settings>({...api.defaultSettings}),loading=$state(true),error=$state(''),syncError=$state(''),syncing=$state(false),selectedFactor=$state(''),checkinIndex=$state(0),onboardStep=$state(0),startY=0,startedAtTop=false,swipeX=0;
 let navigation:ReturnType<typeof createNavigation>,scroll:HTMLElement;
 let requestId=0,refreshing=false,lastSyncAttempt:number|null=null;
 const scrollPositions=new Map<string,number>();
 const fixture=new URLSearchParams(location.search).get('fixture');
 const live=isTauri() && fixture===null;
 let syncBusy=$derived(syncing || status?.syncing===true);
 let activeTab=$derived([...trail].reverse().find(r=>(tabs as readonly string[]).includes(r))??'today');
 let parentLabel=$derived(trail.length>1?({today:'Today',sleep:'Sleep',training:'Training',health:'Health',readiness:'Readiness'} as Record<string,string>)[trail[trail.length-2]]??'Back':'Today');
 function navigate(to:string){navigation.navigate(to)}
 function back(){return navigation.back()}
 async function loadView(current:string):Promise<PageData>{
  const [key,id]=current.split('/');
  switch(key){
   case 'sleep':return api.sleep();
   case 'training':return api.training();
   case 'health':return api.health();
   case 'workout':return api.workout(id||'');
   case 'checkin':return api.checkin();
   case 'metric':return api[metricSource(id)]();
   default:return api.today();
  }
 }
 async function refresh(silent=false){
  if(silent && refreshing)return;
  const current=route,request=++requestId;
  refreshing=true;
  if(!silent)loading=true;
  try {
   const nextStatus=await api.app_status();
   if(request!==requestId)return;
   status=nextStatus;
   if(live){
    if(!status.signedIn){onboardStep=0;if(current!=='onboarding')navigate('onboarding');return}
    if(localStorage.getItem('notifications-step-done')!=='1'){onboardStep=1;if(current!=='onboarding')navigate('onboarding');return}
    if(current==='onboarding'){navigate('today');return}
   }
   if(current==='settings'){
    const nextSettings=await api.get_settings();
    if(request===requestId)settings=nextSettings;
   }else if(current!=='onboarding'){
    let next=await loadView(current);
    if(request!==requestId)return;
    if(fixture==='missing' && current==='today'){
     next={...(next as TodayView),readiness:null,readinessTrend:[],sleep:null,sleepPlan:null,load:null,chips:[],factors:[],call:null,callNote:null,bodyBattery:{now:null,charged:null,drained:null},hrv:{lastNight:null,status:null,tone:'neutral',baseline:null,week:[]},vitals:{readings:[],offCount:0,baselineReady:false},activities:[],fitnessAge:{fitnessAge:null,chronologicalAge:null,trend:[],change12m:null},headline:''};
    }
    data=fixture==='empty' && current==='today'?null:next;
   }
   if(request===requestId)error='';
  }catch(e){
   if(request===requestId){if(!silent)data=null;error=String(e)}
  }finally{
   if(request===requestId){loading=false;refreshing=false}
  }
  if(request===requestId && live && route!=='onboarding' && document.visibilityState==='visible' && shouldSync(status,lastSyncAttempt))void syncNow();
 }
 async function syncNow(){
  if(syncing || status?.syncing)return;
  syncing=true;syncError='';lastSyncAttempt=Date.now();
  try{await api.sync_now();await refresh(true)}catch(e){syncError=String(e)}
  finally{syncing=false}
 }
 onMount(()=>{
  let disposed=false,unlisten=()=>{};
  navigation=createNavigation(next=>{
   const changed=next[next.length-1]!==route;
   if(changed){scrollPositions.set(route,scroll?.scrollTop??0);data=null;selectedFactor='';checkinIndex=0}
   trail=next;route=next[next.length-1];
   void refresh().then(async()=>{
    await tick();
    if(changed && route===next[next.length-1] && scroll)scroll.scrollTop=scrollPositions.get(route)??0;
   });
  });
  navigation.start();
  const nativeWindow=window as Window & {__pepometerBack?:()=>boolean};
  nativeWindow.__pepometerBack=back;
  const resume=()=>{if(document.visibilityState==='visible')void refresh(true)};
  document.addEventListener('visibilitychange',resume);
  window.addEventListener('pepometer:resume',resume);
  void api.onStatusChanged(resume).then(fn=>{if(disposed)fn();else unlisten=fn});
  const poll=window.setInterval(resume,CACHE_POLL_MS);
  return()=>{disposed=true;requestId++;navigation.destroy();delete nativeWindow.__pepometerBack;window.clearInterval(poll);document.removeEventListener('visibilitychange',resume);window.removeEventListener('pepometer:resume',resume);unlisten()};
 });
 async function pullEnd(e:TouchEvent){if(route==='today' && startedAtTop && e.changedTouches[0].clientY-startY>75)await syncNow()}
 async function saveSettings(){try{await api.set_settings(settings)}catch(e){error=String(e)}}

 const driverScale=(key:string):[number,number]=>({vo2max:[20,70],restingHr:[35,85],bmi:[15,35],bodyFat:[5,40],vigorousMinutes:[0,200]} as Record<string,[number,number]>)[key]??[0,100];
 const driverRange=(key:string,target:number|null):[number,number]|null=>target==null?null:key==='bmi'?[18.5,target]:['restingHr','bodyFat'].includes(key)?[driverScale(key)[0],target]:[target,driverScale(key)[1]];
 let todayData=$derived(route==='today'||route==='readiness'?data as TodayView|null:null),sleepData=$derived(route==='sleep'?data as SleepView|null:null),trainingData=$derived(route==='training'?data as TrainingView|null:null),workoutData=$derived(route.startsWith('workout/')?data as WorkoutView|null:null),healthData=$derived(route==='health'?data as HealthView|null:null),checkinData=$derived(route==='checkin'?data as CheckinView|null:null);
 let glow=$derived(route==='sleep'?'rgba(155,140,255,.24)':route==='training'||route.startsWith('workout/')?'rgba(255,138,61,.20)':'rgba(61,220,151,.24)');
</script>
<div class="phone"><div class="glow" style:--glow={glow}></div><main class="scroll" bind:this={scroll} ontouchstart={(e)=>{startY=e.touches[0].clientY;startedAtTop=e.currentTarget.scrollTop===0}} ontouchend={pullEnd}><div class="page" class:reading={route==='checkin'||route==='onboarding'}>
{#if error && data}<p class="warning" role="status">{error}</p>{/if}
{#if route==='onboarding'}
 <section class="welcome"><BrandMark/><div class="eyebrow">YOUR GARMIN, AT A GLANCE</div><h1>Pepometer</h1><p>Know how you're doing. See what matters. Make today count.</p>{#if onboardStep===0}<button class="primary" onclick={async()=>{try{await api.begin_sign_in()}catch(e){error=String(e)}}}>Sign in with Garmin <Icon name="arrow-right" size={18}/></button>{:else}<h2>Stay in the loop</h2><p>Get a heads-up when your readiness and weekly check-in are ready.</p><button class="primary" onclick={async()=>{try{await api.request_notification_permission()}catch(e){error=String(e);return}localStorage.setItem('notifications-step-done','1');navigate('today')}}>Enable notifications</button><button class="secondary" onclick={()=>{localStorage.setItem('notifications-step-done','1');navigate('today')}}>Not now</button>{/if}{#if error || status?.lastError}<p class="small">{error || status?.lastError}</p>{/if}</section>
{:else if route==='settings'}
 <Header title="Settings" eyebrow="PEPOMETER" back={back} backLabel={parentLabel}/><div class="dashboard"><Card title="Notifications">{#each Object.keys(settings) as key}<label class="setting"><span>{({notifyMorningReadiness:'Morning readiness',notifyWeeklyCheckin:'Weekly check-in',notifyMetricChanges:'VO₂ max or Fitness Age changed',notifyVitals:'Vitals off',notifyTrainingStatus:'Training status changed',notifyBedtime:'Bedtime'} as Record<string,string>)[key]}</span><input type="checkbox" checked={settings[key as keyof api.Settings]} onchange={(e)=>{settings={...settings,[key]:e.currentTarget.checked};void saveSettings()}}/></label>{/each}</Card><Card title="Permission"><button class="secondary" onclick={async()=>{try{await api.request_notification_permission()}catch(e){error=String(e)}}}>Enable notifications</button></Card><Card title="Account"><button class="signout" onclick={async()=>{if(confirm('Sign out and erase all local Garmin history?')){await api.sign_out();localStorage.removeItem('notifications-step-done');onboardStep=0;navigate('onboarding')}}}>Sign out</button></Card></div>
{:else if loading}<div class="empty"><BrandMark/><h2>{status?.lastSync?'Loading your day…':'Syncing your Garmin data…'}</h2></div>
{:else if !data}<div class="empty"><BrandMark/><h2>Syncing your Garmin data…</h2><p>{error && !error.includes('not_ready')?error:'Your day will appear here soon.'}</p><button class="secondary" disabled={syncBusy} onclick={syncNow}>{syncBusy?'Syncing…':'Sync now'}</button>{#if syncError}<p>{syncError}</p>{/if}</div>
{:else if todayData && route==='today'}
 <div class="page-hero"><Header title="Today" eyebrow={dateLabel(todayData.date)} onSettings={()=>navigate('settings')}/>
 <div class="sync-indicator"><span class="small" aria-live="polite">{#if syncBusy}Syncing with Garmin…{:else if syncError || status?.lastError}Sync delayed · {syncError || status?.lastError}{:else if status?.lastSync}Last synced {clock(status.lastSync)}{:else}Waiting for first sync{/if}</span><button class="sync-button" disabled={syncBusy} onclick={syncNow}>Refresh</button></div>
 <section class="dials"><button class="dial-button" onclick={()=>navigate('sleep')}>{#if todayData.sleep}<Gauge size={100} value={todayData.sleep.value} max={todayData.sleep.max} color={colors.violet} second="#6E5BFF" caption="Sleep"/>{:else}<Gauge size={100} value={null} caption="Sleep"/>{/if}</button><button class="dial-button large" onclick={()=>navigate('readiness')}>{#if todayData.readiness}<Gauge size={156} stroke={13} value={todayData.readiness.value} max={todayData.readiness.max} color={toneColor(todayData.readiness.tone)} second="#2BB6D9" caption="Readiness" label={todayData.readiness.label}/>{:else}<Gauge size={156} stroke={13} value={null} caption="Readiness"/>{/if}</button><button class="dial-button" onclick={()=>navigate('training')}>{#if todayData.load}<Gauge size={100} value={todayData.load.value} max={todayData.load.max} color={colors.flame} second="#FF5E3A" caption="Load" band={todayData.load.band} marker/>{:else}<Gauge size={100} value={null} caption="Load"/>{/if}</button></section>
 <div class="headline"><h2>{todayData.headline||'Your day is taking shape'}</h2><div class="chips">{#each todayData.chips as chip}<Chip {chip} label={chip.icon==='moon'?'View sleep':chip.icon==='hrv'?'View HRV':'View training'} onclick={()=>navigate(chip.icon==='moon'?'sleep':chip.icon==='hrv'?'metric/hrv':'training')}/>{/each}</div></div></div><div class="dashboard">
 {#if todayData.call || todayData.factors.length}<DailyFocus today={todayData} open={()=>navigate('readiness')}/>{/if}
 {#if todayData.sleepPlan}<SleepPlanCard plan={todayData.sleepPlan} open={()=>navigate('sleep')}/>{/if}

 {#if todayData.bodyBattery.now!=null || todayData.hrv.lastNight!=null}<div class="row">{#if todayData.bodyBattery.now!=null}<Card title="Body Battery" onclick={()=>navigate('metric/bodyBattery')}><div class="bb"><div class="battery"><i style:height={Math.max(0,Math.min(100,todayData.bodyBattery.now))+'%'}></i></div><div><div class="big n">{num(todayData.bodyBattery.now)}</div>{#if todayData.bodyBattery.charged!=null}<div class="small"><b>+{num(todayData.bodyBattery.charged)}</b> charged today</div>{/if}</div></div></Card>{/if}{#if todayData.hrv.lastNight!=null}<Card title="HRV" onclick={()=>navigate('metric/hrv')}>{#snippet aside()}{#if todayData.hrv.status}<span class="pill" style:color={toneColor(todayData.hrv.tone)}>{todayData.hrv.status}</span>{/if}{/snippet}<div class="big n">{num(todayData.hrv.lastNight)}<span class="unit">ms</span></div><div class="spark-space"><Sparkline values={todayData.hrv.week} band={todayData.hrv.baseline} w={138} h={34}/></div></Card>{/if}</div>{/if}
 {#if todayData.vitals.readings.length}<VitalsMonitor vitals={todayData.vitals} compact open={(kind)=>navigate('metric/'+kind)}/>{/if}
 {#if todayData.activities.length}<Card title="Activities">{#each todayData.activities as a}<ActivityRow activity={a} open={()=>navigate('workout/'+a.id)}/>{#if a.aerobicTe!=null || a.benefit}<div class="tags">{#if a.aerobicTe!=null}<span class="pill ok">Aerobic {a.aerobicTe} · {a.aerobicLabel}</span>{/if}{#if a.benefit}<span class="pill flame">{a.benefit}</span>{/if}</div>{/if}{/each}</Card>{/if}
 {#if todayData.fitnessAge.fitnessAge!=null}<Card title="Fitness Age" onclick={()=>navigate('health')}><div class="age-small"><div><div class="big n">{num(todayData.fitnessAge.fitnessAge)}</div>{#if todayData.fitnessAge.chronologicalAge!=null}<span class="pill ok">{num(todayData.fitnessAge.chronologicalAge-todayData.fitnessAge.fitnessAge)} yrs younger</span>{/if}</div><Sparkline values={todayData.fitnessAge.trend} w={160} h={64}/></div></Card>{/if}
 {#if todayData.checkinReady}<Card title="Weekly check-in" onclick={()=>navigate('checkin')}><div class="checkin-tease"><span>See your week at a glance</span><b><Icon name="arrow-right" size={18}/></b></div></Card>{/if}
</div>
{:else if route==='readiness' && todayData}
 <div class="page-hero"><Header title="Readiness" eyebrow={dateLabel(todayData.date)} back={back} backLabel="Today"/><div class="detail-gauge">{#if todayData.readiness}<Gauge size={240} stroke={17} value={todayData.readiness.value} max={todayData.readiness.max} label={todayData.readiness.label} caption="Readiness" color={toneColor(todayData.readiness.tone)} second="#2BB6D9"/>{:else}<p>Waiting for readiness</p>{/if}<h2>{todayData.headline}</h2></div></div><div class="dashboard">
 {#if todayData.factors.length}<Card title="What's driving it"><div class="factor-list">{#each [...todayData.factors].sort((a,b)=>(a.percent??101)-(b.percent??101)) as factor}<button class="factor-row" aria-expanded={selectedFactor===factor.key} onclick={()=>selectedFactor=selectedFactor===factor.key?'':factor.key}><span>{factor.name}</span><b class="n">{num(factor.percent)}</b><div class="bar"><i style:width={(factor.percent??0)+'%'} style:background={toneColor(factor.tone)}></i></div>{#if selectedFactor===factor.key}<p>{factor.detail}</p>{/if}</button>{/each}</div></Card>{/if}
 {#if todayData.readinessTrend.length}<Card title="Readiness history"><ReadinessTrend values={todayData.readinessTrend} date={todayData.date}/><p class="small">Morning readings · latest synced reading today</p></Card>{/if}
 {#if todayData.call}<Card title="Today's call"><div class="segments">{#each ['rest','easy','steady','push'] as effort}<span class:on={todayData.call===effort}>{effort}</span>{/each}</div>{#if todayData.callNote}<div class="sub">{todayData.callNote}</div>{/if}</Card>{/if}
</div>
{:else if sleepData}
 <div class="page-hero"><Header title="Sleep" eyebrow={sleepData.startLocal?`${clock(sleepData.startLocal)} – ${clock(sleepData.endLocal)}`:dateLabel(sleepData.date)} onSettings={()=>navigate('settings')}/><section class="sleep-hero">{#if sleepData.score}<Gauge size={150} stroke={12} value={sleepData.score.value} max={sleepData.score.max} caption="Score" label={sleepData.score.label} color={colors.violet} second="#6E5BFF"/>{/if}<div class="need"><div class="eyebrow">Asleep</div><div class="big n">{duration(sleepData.asleepMin)}</div><div class="needbar"><i style:width={Math.min(100,(sleepData.asleepMin??0)/Math.max(540,sleepData.needMin??0)*100)+'%'}></i>{#if sleepData.needMin!=null}<u style:left={sleepData.needMin/Math.max(540,sleepData.needMin)*100+'%'}></u>{/if}</div><div class="need-sub">{#if sleepData.needMin!=null}Need <b>{duration(sleepData.needMin)}</b>{#if sleepData.asleepMin!=null && Math.abs(sleepData.asleepMin-sleepData.needMin)>=1} · <strong>{signed(sleepData.asleepMin-sleepData.needMin)} min</strong>{/if}{:else}Sleep need unavailable{/if}</div></div></section></div><div class="dashboard">
 {#if sleepData.stages.length}<Card title="Stages"><Hypnogram segments={sleepData.stages}/></Card>{/if}{#if sleepData.totals.length}<Card><div class="stage-stack">{#each sleepData.totals as s}<i style:width={s.minutes/sleepData.totals.reduce((a,b)=>a+b.minutes,0)*100+'%'} style:background={({deep:colors.deep,light:colors.light,rem:colors.rem,awake:colors.awake} as Record<string,string>)[s.stage]}></i>{/each}</div><div class="stage-legend">{#each sleepData.totals as s}<div><i style:background={({deep:colors.deep,light:colors.light,rem:colors.rem,awake:colors.awake} as Record<string,string>)[s.stage]}></i><span>{s.stage==='rem'?'REM':s.stage}</span><b class="n">{duration(s.minutes)}</b>{#if s.usualDeltaMin!=null && Math.abs(s.usualDeltaMin)>=1}<em>{signed(Math.round(s.usualDeltaMin),'m')}</em>{/if}</div>{/each}</div></Card>{/if}
 {#if sleepData.overnight.length}<Card title="Overnight"><div class="tiles">{#each sleepData.overnight as tile}<div class="tile"><span style:color={tile.kind==='heartRate'?colors.coral:tile.kind==='spo2'?colors.violet:tile.kind==='respiration'?colors.sky:colors.mint}><Icon name={tile.kind==='heartRate'?'heart':tile.kind==='spo2'?'drop':tile.kind==='respiration'?'lungs':'hrv'} size={14}/><span class="muted">{({heartRate:'Heart rate',hrv:'HRV',respiration:'Respiration',spo2:'SpO₂'} as Record<string,string>)[tile.kind]}</span></span><b class="n">{num(tile.value,tile.kind==='respiration'?1:0)}<small>{tile.unit}</small></b><Sparkline values={tile.series} w={128} h={30} color={tile.kind==='heartRate'?colors.coral:tile.kind==='spo2'?colors.violet:tile.kind==='respiration'?colors.sky:colors.mint} fill={false}/></div>{/each}</div></Card>{/if}
 {#if sleepData.consistency.length}<Card title="Consistency">{#snippet aside()}{#if sleepData.consistencySpreadMin!=null && sleepData.consistencySpreadMin>=1}<span class="pill violet">±{Math.round(sleepData.consistencySpreadMin)} min</span>{/if}{/snippet}<ConsistencyChart nights={sleepData.consistency} bedWindow={sleepData.bedWindow} wakeWindow={sleepData.wakeWindow}/></Card>{/if}
 {#if sleepData.sleepPlan}<SleepPlanCard plan={sleepData.sleepPlan}/>{/if}
</div>
{:else if trainingData}
 <div class="page-hero"><Header title="Training" eyebrow="Last 7 days" onSettings={()=>navigate('settings')}/>{#if trainingData.status}<span class="status-pill pill ok"><Icon name="trend-up" size={14}/>{trainingData.status}</span>{/if}
 {#if trainingData.acute}<section class="semi"><Gauge size={300} stroke={16} sweep={180} value={trainingData.acute.value} max={trainingData.acute.max} caption="Acute load" label={trainingData.acute.label} color={colors.flame} second="#FF5E3A" band={trainingData.acute.band} marker/><span class="semi-low">Low</span><span class="semi-high">High</span></section>{/if}<div class="headline training-head"><h2>{trainingData.headline}</h2><div class="chips">{#each trainingData.chips as chip}<Chip {chip}/>{/each}</div></div></div><div class="dashboard">
 {#if trainingData.week.length}<Card title="This week"><WeekBars days={trainingData.week}/></Card>{/if}{#if trainingData.loadFocus.length}<Card title="Load focus · 4 weeks"><LoadFocus bars={trainingData.loadFocus}/></Card>{/if}
 {#if trainingData.vo2max!=null || trainingData.races.length}<div class="row">{#if trainingData.vo2max!=null}<Card title="VO₂ max" onclick={()=>navigate('metric/vo2max')}><div class="big n">{num(trainingData.vo2max)}</div>{#if trainingData.vo2maxChange30d!=null && Math.abs(trainingData.vo2maxChange30d)>=1}<div class="small"><b>{signed(trainingData.vo2maxChange30d)}</b> in 30 days</div>{/if}<Sparkline values={trainingData.vo2maxTrend} w={138} h={30}/></Card>{/if}{#if trainingData.races.length}<Card title="Race times"><div class="race">{#each trainingData.races as race}<div><span>{race.label}</span><b class="n">{elapsed(race.seconds)}</b></div>{/each}</div></Card>{/if}</div>{/if}
 {#if trainingData.enduranceScore!=null || trainingData.hillScore!=null || trainingData.runningToleranceKm!=null}<Card title="More metrics"><div class="metrics">{#if trainingData.enduranceScore!=null}<button class="metric-link" onclick={()=>navigate('metric/endurance')}><b>{num(trainingData.enduranceScore)}</b><span>Endurance</span></button>{/if}{#if trainingData.hillScore!=null}<button class="metric-link" onclick={()=>navigate('metric/hill')}><b>{num(trainingData.hillScore)}</b><span>Hill score</span></button>{/if}{#if trainingData.runningToleranceKm!=null}<button class="metric-link" onclick={()=>navigate('metric/runningTolerance')}><b>{num(trainingData.runningToleranceKm)} km</b><span>Run tolerance</span></button>{/if}</div></Card>{/if}
 {#if trainingData.workouts.length}<Card title="Workouts">{#each trainingData.workouts as a}<div class="workout-item"><ActivityRow activity={a} open={()=>navigate('workout/'+a.id)}/></div>{/each}</Card>{/if}
</div>
{:else if workoutData}
 <Header title={workoutData.summary.name} eyebrow={dateLabel(workoutData.summary.startLocal.slice(0,10))+' · '+clock(workoutData.summary.startLocal)} back={back} backLabel={parentLabel}/><div class="dashboard"><Card extra="workout-overview"><div class="workout-hero"><div><div class="eyebrow">Training load</div><div class="load-number n">{num(workoutData.summary.load)}</div><div class="tags">{#if workoutData.summary.benefit}<span class="pill flame">{workoutData.summary.benefit}</span>{/if}{#if workoutData.recoveryHours!=null}<span class="pill"><Icon name="clock" size={13}/>{workoutData.recoveryHours}h recovery</span>{/if}</div></div><div class="effects">{#if workoutData.summary.aerobicTe!=null}<div><Gauge size={82} stroke={7} value={workoutData.summary.aerobicTe} max={5} caption="Aerobic" color={colors.mint}/><small>{workoutData.summary.aerobicLabel}</small></div>{/if}{#if workoutData.summary.anaerobicTe!=null}<div><Gauge size={82} stroke={7} value={workoutData.summary.anaerobicTe} max={5} caption="Anaer." color={colors.violet}/><small>{workoutData.summary.anaerobicLabel}</small></div>{/if}</div></div></Card>
 {#if workoutData.stats.length}<Card><div class="stats">{#each workoutData.stats as s}<div><b class="n">{s.key==='pace'?pace(s.value).replace('/km',''):s.key==='duration'?elapsed(s.value):num(s.value,s.key==='distance'?1:0)}{#if s.key==='pace'}<small>/km</small>{:else if s.unit!=='s' && s.unit!=='bpm' && s.unit!=='kcal'}<small>{s.unit}</small>{/if}</b><span>{s.label}</span></div>{/each}</div></Card>{/if}
 {#if workoutData.route.length}<Card extra="map-card"><RouteMap route={workoutData.route}/></Card>{/if}{#if workoutData.heartRate.length}<Card title="Heart rate"><HrChart samples={workoutData.heartRate} floors={workoutData.zoneFloors}/></Card>{/if}
 {#if workoutData.summary.zoneSeconds.length}<Card title="Time in zones">{@const maxZone=Math.max(...workoutData.summary.zoneSeconds,1)}{@const total=workoutData.summary.zoneSeconds.reduce((a,b)=>a+b,0)}{#each [...workoutData.summary.zoneSeconds].reverse() as seconds,i}<div class="zone-row"><b>Z{5-i}</b><div class="bar"><i style:width={seconds/maxZone*100+'%'} style:background={colors.z[4-i]}></i></div><span class="n">{elapsed(seconds)}</span><small>{Math.round(seconds/Math.max(total,1)*100)}%</small></div>{/each}</Card>{/if}
 {#if workoutData.splits.length}<Card title="Splits">{#snippet aside()}<span class="pill ok">Fastest km {workoutData.splits.reduce((a,b)=>a.seconds<b.seconds?a:b).index}</span>{/snippet}{@const slow=Math.max(...workoutData.splits.map(s=>s.seconds))}{@const fast=Math.min(...workoutData.splits.map(s=>s.seconds))}{#each workoutData.splits as s}<div class="split-row"><span>{s.index}</span><div class="bar"><i style:width={(slow-s.seconds+15)/(slow-fast+15)*92+'%'} style:background={s.seconds===fast?colors.mint:'rgba(255,138,61,.6)'}></i></div><b class="n">{pace(s.seconds).replace('/km','')}</b></div>{/each}</Card>{/if}
</div>
{:else if healthData}
 <Header title="Health" eyebrow={dateLabel(healthData.date)} onSettings={()=>navigate('settings')}/><div class="dashboard health-dashboard">
 <VitalsMonitor vitals={healthData.vitals} open={(kind)=>navigate('metric/'+kind)}/>
 <Card title="Fitness Age">{#snippet aside()}{#if healthData.fitnessAge.change12m!=null && Math.abs(healthData.fitnessAge.change12m)>=1}<span class="pill ok"><Icon name={healthData.fitnessAge.change12m<0?'trend-down':'trend-up'} size={13}/>{Math.round(Math.abs(healthData.fitnessAge.change12m))} this year</span>{/if}{/snippet}<div class="age-hero"><div><div class="age-number n">{num(healthData.fitnessAge.fitnessAge)}</div>{#if healthData.fitnessAge.fitnessAge!=null && healthData.fitnessAge.chronologicalAge!=null}<div class="age-caption"><b>{num(healthData.fitnessAge.chronologicalAge-healthData.fitnessAge.fitnessAge)} years younger</b> than your age</div>{/if}</div><Sparkline values={healthData.fitnessAge.trend} w={120} h={70}/></div>{#if healthData.fitnessAge.fitnessAge!=null && healthData.fitnessAge.chronologicalAge!=null}<div class="agebar"><div class="age-mark fit" style:left={Math.max(0,Math.min(100,(healthData.fitnessAge.fitnessAge-20)/50*100))+'%'}><span>YOU {num(healthData.fitnessAge.fitnessAge)}</span></div><div class="age-mark real" style:left={Math.max(0,Math.min(100,(healthData.fitnessAge.chronologicalAge-20)/50*100))+'%'}><span>AGE {num(healthData.fitnessAge.chronologicalAge)}</span></div></div>{/if}</Card>
 {#if healthData.lever}<Card title="Biggest lever"><div class="lever"><Gauge size={76} stroke={8} value={healthData.lever.current} max={healthData.lever.target} display={Math.round(healthData.lever.current/healthData.lever.target*100)+'%'} color={colors.amber} second={colors.flame}/><div><b>{healthData.lever.name}</b><span>{num(healthData.lever.current)} / {num(healthData.lever.target)} {healthData.lever.unit}</span>{#if healthData.lever.ageIfMet!=null}<span class="pill ok">Hit {healthData.lever.target} <Icon name="arrow-right" size={13}/> age {healthData.lever.ageIfMet}</span>{/if}</div></div></Card>{/if}
 {#if healthData.drivers.length}<Card title="What sets it"><div class="drivers">{#each healthData.drivers as d}<div class="driver"><span>{d.name} <i style:color={toneColor(d.tone)}><Icon name={d.met?'check':'trend-up'} size={16}/></i></span><b class="n">{num(d.value,d.key==='bmi'?1:0)}<small>{d.unit}</small></b><RangeBar value={d.value} scale={driverScale(d.key)} range={driverRange(d.key,d.target)} color={toneColor(d.tone)}/></div>{/each}</div></Card>{/if}

</div>
{:else if route.startsWith('metric/')}
 <Header title={metricNames[route.split('/')[1]]} eyebrow="GARMIN METRIC" back={back} backLabel={parentLabel}/><MetricDetail metric={route.split('/')[1]} data={data as TodayView|HealthView|TrainingView}/>
{:else if checkinData}
 <Header title="Weekly check-in" eyebrow={'Week of '+dateLabel(checkinData.weekStart)} back={back} backLabel="Today"/><div class="checkin-progress">{#each [...checkinData.cards, null] as _,i}<span class:on={i===checkinIndex}></span>{/each}</div><div class="checkin-swiper" role="group" aria-label="Weekly check-in cards" ontouchstart={(e)=>swipeX=e.touches[0].clientX} ontouchend={(e)=>{let dx=e.changedTouches[0].clientX-swipeX;if(Math.abs(dx)>50)checkinIndex=Math.max(0,Math.min(checkinData.cards.length,checkinIndex+(dx<0?1:-1)))}}>{#if checkinIndex<checkinData.cards.length}{@const c=checkinData.cards[checkinIndex]}<Card title={c.area}><div class="checkin-content"><div class="eyebrow">YOUR WEEK IN {c.area.toUpperCase()}</div>{#if c.value!=null}<Gauge size={210} stroke={15} value={c.value} max={c.area==='training'?1000:c.area==='health'?50:100} color={c.area==='sleep'?colors.violet:c.area==='training'?colors.flame:colors.mint} caption={c.unit}/>{/if}<h2>{c.headline}</h2><span class="pill" class:ok={c.previous!=null && c.value!=null && (c.area==='health'?c.value<=c.previous:c.value>=c.previous)}>{#if c.previous!=null && c.value!=null && Math.abs(c.value-c.previous)>=1}<Icon name={c.value<c.previous?'trend-down':'trend-up'} size={14}/>{signed(c.value-c.previous)} vs last week{:else if c.previous!=null}Steady vs last week{:else}First week{/if}</span><Sparkline values={c.series} w={240} h={60} color={c.area==='sleep'?colors.violet:c.area==='training'?colors.flame:colors.mint}/></div></Card>{:else}<Card title="Your week ahead"><div class="checkin-summary"><div class="brand-mark"><Icon name="sparkle" size={70}/></div><h2>Keep the momentum</h2>{#if checkinData.vo2maxChange!=null && Math.abs(checkinData.vo2maxChange)>=1}<span class="pill ok">VO₂ max {signed(checkinData.vo2maxChange)}</span>{/if}{#if checkinData.fitnessAgeChange!=null && Math.abs(checkinData.fitnessAgeChange)>=1}<span class="pill ok">Fitness Age {signed(checkinData.fitnessAgeChange)}</span>{/if}{#if checkinData.win}<h3>THE WIN</h3><p>{checkinData.win}</p>{/if}{#if checkinData.focus}<h3>THE FOCUS</h3><p>{checkinData.focus}</p>{/if}</div></Card>{/if}</div><div class="checkin-controls"><button disabled={checkinIndex===0} onclick={()=>checkinIndex--}><Icon name="arrow-left" size={16}/> Previous</button><span>{checkinIndex+1} / {checkinData.cards.length+1}</span><button disabled={checkinIndex===checkinData.cards.length} onclick={()=>checkinIndex++}>Next <Icon name="arrow-right" size={16}/></button></div>
{/if}
</div></main>{#if route!=='onboarding'}<TabBar active={activeTab} {navigate}/>{/if}</div>
