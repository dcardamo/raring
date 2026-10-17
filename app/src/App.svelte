<script lang="ts">
 import {onMount,tick,setContext} from 'svelte';
 import {isTauri} from '@tauri-apps/api/core';
 import {createNavigation,tabs} from './lib/navigation';
 import {shouldSync,CACHE_POLL_MS} from './lib/freshness';
 import {watchLabel,watchNeedsSync} from './lib/watch';
 import {masonry} from './lib/masonry';
 import {addDays} from './lib/history';
 import {metricSource,metricNames} from './lib/metrics';
 import MetricDetail from './lib/components/MetricDetail.svelte';
 import HistoryChart from './lib/components/HistoryChart.svelte';
 import DailyFocus from './lib/components/DailyFocus.svelte';
 import AboutRaring from './lib/components/AboutRaring.svelte';
 import SleepPlanCard from './lib/components/SleepPlanCard.svelte';
 import NowCard from './lib/components/NowCard.svelte';
 import {phaseEnded} from './lib/tonight';
 import AlignmentCard from './lib/components/AlignmentCard.svelte';
 import WorkoutSplits from './lib/components/WorkoutSplits.svelte';
 import VitalsMonitor from './lib/components/VitalsMonitor.svelte';
 import StressTimeline from './lib/components/StressTimeline.svelte';
 import PlanCard from './lib/components/PlanCard.svelte';
 import {missingPlanEvidence} from './lib/sample/plan-fixtures';
 import PlanScreen from './lib/components/PlanScreen.svelte';
 import PlanSetup from './lib/components/PlanSetup.svelte';
 import PlanCheckinCard from './lib/components/PlanCheckinCard.svelte';
 import IntensityCard from './lib/components/IntensityCard.svelte';
 import StrengthWorkout from './lib/components/StrengthWorkout.svelte';
 import StrengthHistory from './lib/components/StrengthHistory.svelte';
 import HelpSheet from './lib/components/HelpSheet.svelte';
 import BodyMap from './lib/components/BodyMap.svelte';
 import {help as helpContent,helpSections,GITHUB_QUESTIONS} from './lib/help';
 import type {StrengthView} from './lib/generated/StrengthView';
 import WhySheet from './lib/components/WhySheet.svelte';
 import Reconcile from './lib/components/Reconcile.svelte';
 import {closeSheet} from './lib/sheet';
 import * as api from './lib/api';
 import type {TodayView} from './lib/generated/TodayView';import type {SleepView} from './lib/generated/SleepView';import type {TrainingView} from './lib/generated/TrainingView';import type {WorkoutView} from './lib/generated/WorkoutView';import type {HealthView} from './lib/generated/HealthView';import type {CheckinView} from './lib/generated/CheckinView';import type {AppStatus} from './lib/generated/AppStatus';import type {PlanView} from './lib/generated/PlanView';import type {PlanUpdate} from './lib/generated/PlanUpdate';
 import Icon from './lib/components/Icon.svelte';import BrandMark from './lib/components/BrandMark.svelte';import ReadinessTrend from './lib/components/ReadinessTrend.svelte';
 import Gauge from './lib/components/Gauge.svelte';import Sparkline from './lib/components/Sparkline.svelte';import RangeBar from './lib/components/RangeBar.svelte';import ZoneStrip from './lib/components/ZoneStrip.svelte';import Hypnogram from './lib/components/Hypnogram.svelte';import ConsistencyChart from './lib/components/ConsistencyChart.svelte';import WeekBars from './lib/components/WeekBars.svelte';import LoadFocus from './lib/components/LoadFocus.svelte';import RouteMap from './lib/components/RouteMap.svelte';import HrChart from './lib/components/HrChart.svelte';import Card from './lib/components/Card.svelte';import Chip from './lib/components/Chip.svelte';import Header from './lib/components/Header.svelte';import TabBar from './lib/components/TabBar.svelte';import ActivityRow from './lib/components/ActivityRow.svelte';
 import {colors,toneColor,batteryColor,stressColors,stressNames} from './lib/charts';import {duration,elapsed,pace,clock,dateLabel,monthLabel,signed,num} from './lib/format';
 type StrengthPage={strengthPage:StrengthView|null};
 type PageData=TodayView|SleepView|TrainingView|WorkoutView|HealthView|CheckinView|PlanView|StrengthPage;
 let sheet=$state(false);
 let now=$state(new Date()),connectUnavailable=$state(false),exporting=$state(false),exportMessage=$state('');
 let route=$state('today'),trail=$state<string[]>(['today']),data=$state<PageData|null>(null),status=$state<AppStatus|null>(null),settings=$state<api.Settings>({...api.defaultSettings}),loading=$state(true),error=$state(''),syncError=$state(''),syncing=$state(false),selectedFactor=$state(''),checkinIndex=$state(0),onboardStep=$state(0),startY=0,startedAtTop=false,swipeX=0;
 let navigation:ReturnType<typeof createNavigation>,scroll:HTMLElement;
 let requestId=0,statusReadId=0,appliedStatusReadId=0,lastSyncAttempt:number|null=null,prefetched=false;
 let cacheGeneration=$state(0);
 setContext('trend-generation',()=>cacheGeneration);
 let hasRendered=$state(false);
 let touching=false,pendingSwap:{route:string;data:PageData}|null=null;
 const views=new Map<string,PageData>();
 const pendingViews=new Map<string,Promise<PageData>>();
 let cacheDay=new Date().toDateString();
 function clearViews(){views.clear();pendingViews.clear();pendingSwap=null;prefetched=false;cacheGeneration++}
 const scrollPositions=new Map<string,number>();
 function checkDay(){
  const day=new Date().toDateString();
  if(day!==cacheDay){clearViews();cacheDay=day}
 }
 function cached(current:string){
  checkDay();
  const view=views.get(current)??null;
  // A Today whose phase has ended would swap its leading card right after showing.
  if(current==='today' && view && phaseEnded(view as TodayView)){views.delete(current);return null}
  return view;
 }
 async function readView(current:string):Promise<PageData>{
  for(;;){
   checkDay();
   const generation=cacheGeneration;
   let read=pendingViews.get(current);
   if(!read){
    read=loadView(current).then(next=>{if(generation===cacheGeneration)views.set(current,next);return next})
     .finally(()=>{if(pendingViews.get(current)===read)pendingViews.delete(current)});
    pendingViews.set(current,read);
   }
   try{
    const next=await read;
    checkDay();
    if(generation===cacheGeneration)return next;
   }catch(e){
    if(generation===cacheGeneration)throw e;
   }
  }
 }
 function prefetchTabs(){
  if(prefetched)return;
  prefetched=true;
  const generation=cacheGeneration;
  void tick().then(()=>{if(generation===cacheGeneration)return Promise.allSettled([...tabs,'plan'].filter(tab=>tab!==route).map(tab=>readView(tab)))});
 }
 const fixture=new URLSearchParams(location.search).get('fixture');
 const live=isTauri() && fixture===null;
 let syncBusy=$derived(syncing || status?.syncing===true);
 let activeTab=$derived([...trail].reverse().find(r=>(tabs as readonly string[]).includes(r))??'today');
 let parentLabel=$derived.by(()=>{const parent=trail.at(-2);if(!parent)return 'Today';if(parent.startsWith('workout/'))return 'Workout';if(parent.startsWith('help/'))return 'Help';return ({today:'Today',sleep:'Sleep',training:'Training',health:'Health',readiness:'Readiness',plan:'This week',checkin:'Check-in',strength:'Strength',help:'Help',settings:'Settings'} as Record<string,string>)[parent]??'Back'});
 function navigate(to:string){navigation.navigate(to)}
 function back(){return closeSheet() || navigation.back()}
 async function loadView(current:string):Promise<PageData>{
  const [key,id]=current.split('/');
  switch(key){
   case 'sleep':return api.sleep();
   case 'training':return api.training();
   case 'health':return api.health();
   case 'workout':return api.workout(id||'');
   case 'strength':return api.strength().then(strengthPage=>({strengthPage}));
   case 'checkin':return api.checkin();
   case 'plan':case 'plan-setup':return api.plan();
   case 'metric':return api[metricSource(id)]();
   default:return api.today();
  }
 }
 async function refresh(silent=false,withStatus=false){
  checkDay();
  const current=route,request=++requestId;
  if(!silent)loading=!data;
  try {
   if(withStatus){
    const statusRead=++statusReadId;
    const nextStatus=await api.app_status();
    if(statusRead>appliedStatusReadId){
     appliedStatusReadId=statusRead;
     // A queued or slow Android sync can succeed after the foreground wait ends.
     if(lastSyncAttempt!==null && nextStatus.lastSync && Date.parse(nextStatus.lastSync)>=lastSyncAttempt && !nextStatus.lastError)syncError='';
     const changedSync=status!==null && status.lastSync!==nextStatus.lastSync;
     if(changedSync)clearViews();
     status=nextStatus;
     checkDay();
     if(live){
      if(!status.signedIn){onboardStep=0;clearViews();data=null;if(route!=='onboarding')navigate('onboarding');return}
      if(localStorage.getItem('notifications-step-done')!=='1'){onboardStep=1;if(route!=='onboarding')navigate('onboarding');return}
      if(route==='onboarding'){navigate('today');return}
     }
     if(changedSync && request!==requestId)void refresh(true);
    }
    if(request!==requestId)return;
   }
   if(current==='settings'){
    const nextSettings=await api.get_settings();
    if(request===requestId)settings=nextSettings;
   }else if(current!=='onboarding' && current!=='help' && !current.startsWith('help/')){
    let next=await readView(current);
    if(request!==requestId)return;
    if(fixture==='missing' && current==='today'){
     next={...(next as TodayView),readiness:null,readinessTrend:[],sleep:null,sleepPlan:null,load:null,chips:[],factors:[],readinessWhy:null,bodyBatteryWhy:null,reconcile:null,call:null,callNote:null,bodyBattery:{now:null,charged:null,drained:null},stress:{average:null,max:null,level:null,qualifier:null,tone:'neutral',timeline:[],active:[],sleep:null,time:[],activeS:null,trend:[]},hrv:{lastNight:null,status:null,tone:'neutral',baseline:null,week:[]},vitals:{readings:[],offCount:0,baselineReady:false},activities:[],fitnessAge:{fitnessAge:null,chronologicalAge:null,trend:[],change12m:null},headline:'',focus:{...(next as TodayView).focus,phase:'wake',cards:['focus'],effort:false,sleepMissing:true,lastNight:null}};
    }
    if(fixture==='stale' && current==='today')next={...(next as TodayView),watchLastUpload:new Date(Date.now()-7*3600_000).toISOString(),missingMetricReasons:{skinTemp:"Your watch doesn't record skin temperature"}};
    if(fixture==='fresh' && current==='today')next={...(next as TodayView),watchLastUpload:new Date(Date.now()-2*3600_000-120_000).toISOString()};
    if(fixture==='stale-sleep' && current==='today')next={...(next as TodayView),watchLastUpload:null,watchSleepUploadStale:true,missingMetricReasons:{}};
    if(fixture==='plan-partial'||fixture==='plan-unavailable'){
     const all=fixture==='plan-unavailable';
     if(current==='plan'||current==='plan-setup'){
      const p=next as PlanView;
      next={...p,week:missingPlanEvidence(p.week,all),lastWeek:missingPlanEvidence(p.lastWeek,all),history:p.history.map(w=>({...w,completion:w.current?(all?null:1):null,partial:w.current,paused:false}))};
     }else if(current==='today'){
      const t=next as TodayView;next={...t,plan:t.plan?missingPlanEvidence(t.plan,all):null};
     }else if(current==='checkin'){
      const c=next as CheckinView;next={...c,plan:c.plan?{...c.plan,week:missingPlanEvidence(c.plan.week,all)}:null};
     }
    }
    const updated=fixture==='empty' && current==='today'?null:next;
    if(touching && data && updated)pendingSwap={route:current,data:updated};
    else data=updated;
    if(data){hasRendered=true;prefetchTabs()}
   }
   if(hasRendered && !prefetched && current!=='onboarding')prefetchTabs();
   if(request===requestId)error='';
  }catch(e){
   if(request===requestId){error=String(e)}
  }finally{
   if(request===requestId)loading=false;
  }
  if(request===requestId && withStatus && live && route!=='onboarding' && document.visibilityState==='visible' && shouldSync(status,lastSyncAttempt))void syncNow();
 }
 async function syncNow(){
  if(syncing || status?.syncing)return;
  syncing=true;syncError='';lastSyncAttempt=Date.now();
  try{await api.sync_now();clearViews();await refresh(true,true)}catch(e){syncError=String(e)}
  finally{syncing=false}
 }
 onMount(()=>{
  let disposed=false,unlisten=()=>{};
  navigation=createNavigation(next=>{
   const from=route;
   const changed=next[next.length-1]!==route;
   if(changed){scrollPositions.set(route,scroll?.scrollTop??0);pendingSwap=null;selectedFactor='';checkinIndex=0;sheet=false}
   trail=next;route=next[next.length-1];
   const available=changed?cached(route):null;
   if(changed){
    data=available;
    if(available)void tick().then(()=>restoreScroll(next));
   }
   void refresh(false,status===null || from==='onboarding' && route!=='onboarding').then(async()=>{
    if(changed && !available){await tick();restoreScroll(next)}
   });
  });
  navigation.start();
  const nativeWindow=window as Window & {__raringBack?:()=>boolean};
  nativeWindow.__raringBack=back;
  const resume=()=>{if(document.visibilityState==='visible')void refresh(true,true);else finishTouch()};
  document.addEventListener('visibilitychange',resume);
  window.addEventListener('raring:resume',resume);
  void api.onStatusChanged(resume).then(fn=>{if(disposed)fn();else unlisten=fn});
  const poll=window.setInterval(resume,CACHE_POLL_MS);
  const clockTick=window.setInterval(()=>now=new Date(),60_000);
  return()=>{disposed=true;requestId++;navigation.destroy();delete nativeWindow.__raringBack;window.clearInterval(poll);window.clearInterval(clockTick);document.removeEventListener('visibilitychange',resume);window.removeEventListener('raring:resume',resume);unlisten()};
 });
 // Read Today again when its phase ends; a touch defers the swap like any refresh.
 $effect(()=>{
  const until=todayData?.focus.until,wait=until?Date.parse(until)-Date.now()+1000:NaN;
  if(!(wait>0 && wait<2**31))return;
  const timer=window.setTimeout(()=>{if(route==='today')void refresh(true);else views.delete('today')},wait);
  return()=>window.clearTimeout(timer);
 });
 function restoreScroll(next:string[]){
  if(route===next[next.length-1] && scroll)scroll.scrollTop=scrollPositions.get(route)??0;
 }
 function finishTouch(){
  touching=false;
  if(pendingSwap?.route===route)data=pendingSwap.data;
  pendingSwap=null;
 }
 async function pullEnd(e:TouchEvent){
  finishTouch();
  if(route==='today' && startedAtTop && e.changedTouches[0].clientY-startY>75)await syncNow();
 }
 // Saves run in order; each reloads every cached view, since Today shows the plan too.
 let planSaves=Promise.resolve();
 function savePlan(update:PlanUpdate){planSaves=planSaves.then(async()=>{try{await api.set_plan(update);clearViews();await refresh(true)}catch(e){error=String(e)}})}
 async function saveSettings():Promise<boolean>{try{await api.set_settings($state.snapshot(settings));return true}catch(e){error=String(e);return false}}
 // Today's layout depends on this switch: once it is stored, drop every view read before it and read
 // the visible Today again, wherever the user has gone meanwhile. A failed save puts the switch back.
 let savingTodayLayout=$state(false);
 async function saveTodayLayout(on:boolean){
  if(savingTodayLayout)return;
  savingTodayLayout=true;
  const previous=settings.adaptiveToday;
  settings={...settings,adaptiveToday:on};
  try{
   if(!await saveSettings()){settings={...settings,adaptiveToday:previous};return}
   clearViews();
   if(route==='today')void refresh(true);
  }finally{savingTodayLayout=false}
 }
 async function openConnect(){try{connectUnavailable=!(await api.open_garmin_connect())}catch{connectUnavailable=true}}
 async function exportData(){
  if(exporting)return;
  exporting=true;exportMessage='';
  try{exportMessage=await api.export_archive()}catch(e){exportMessage=`Export failed: ${String(e)}`}
  finally{exporting=false}
 }

 const driverScale=(key:string):[number,number]=>({vo2max:[20,70],restingHr:[35,85],bmi:[15,35],bodyFat:[5,40],vigorousMinutes:[0,200],vigorousDays:[0,7]} as Record<string,[number,number]>)[key]??[0,100];
 const driverRange=(key:string,target:number|null):[number,number]|null=>target==null?null:key==='bmi'?[18.5,target]:['restingHr','bodyFat'].includes(key)?[driverScale(key)[0],target]:[target,driverScale(key)[1]];
 let todayData=$derived(route==='today'||route==='readiness'?data as TodayView|null:null),sleepData=$derived(route==='sleep'?data as SleepView|null:null),trainingData=$derived(route==='training'?data as TrainingView|null:null),workoutData=$derived(route.startsWith('workout/')?data as WorkoutView|null:null),strengthData=$derived(route==='strength'?data as StrengthPage|null:null),healthData=$derived(route==='health'?data as HealthView|null:null),checkinData=$derived(route==='checkin'?data as CheckinView|null:null),planData=$derived(route==='plan'||route==='plan-setup'?data as PlanView|null:null);
 // Pages between the area cards and the closing summary, in order.
 let checkinExtra=$derived(checkinData?[...(checkinData.timing?['timing']:[]),...(checkinData.intensity?['intensity']:[]),...(checkinData.plan?['plan']:[])]:[]);
 let checkinPages=$derived(checkinData?checkinData.cards.length+checkinExtra.length:0);
 let checkinPage=$derived(checkinData?checkinExtra[checkinIndex-checkinData.cards.length]:undefined);
 const fitnessAgeColor=(age:number,actual:number|null)=>actual==null?colors.ink:age<actual?colors.good:age===actual?colors.fair:colors.poor;
 let glow=$derived(route==='today' && todayData?.readiness?toneColor(todayData.readiness.tone):route==='sleep' && sleepData?.score?toneColor(sleepData.score.tone):route==='training' && trainingData?.acute?toneColor(trainingData.acute.tone):route==='health' && healthData?.fitnessAge.fitnessAge!=null && healthData.fitnessAge.chronologicalAge!=null?fitnessAgeColor(healthData.fitnessAge.fitnessAge,healthData.fitnessAge.chronologicalAge):null);
</script>
<div class="phone"><div class="glow" style:--glow={glow?`color-mix(in srgb, ${glow} 12%, transparent)`:'transparent'}></div><main class="scroll" bind:this={scroll} ontouchstart={(e)=>{touching=true;startY=e.touches[0].clientY;startedAtTop=e.currentTarget.scrollTop===0}} ontouchend={pullEnd} ontouchcancel={finishTouch}><div class="page" class:reading={route==='checkin'||route==='onboarding'}>
{#if error && (data || route==='settings')}<p class="warning" role="status">{error}</p>{/if}
{#if route==='onboarding'}
 <section class="welcome" class:intro={onboardStep===0}>
  <div class="welcome-brand"><BrandMark/><h1>Raring</h1></div>
  {#if onboardStep===0}
   <p class="welcome-tagline">A clearer view of your readiness, sleep and training.</p>
   <AboutRaring/>
   <button class="primary" onclick={async()=>{try{await api.begin_sign_in()}catch(e){error=String(e)}}}>Sign in with Garmin <Icon name="arrow-right" size={18}/></button>
  {:else}
   <h2>Stay in the loop</h2><p>Get a heads-up when your readiness and weekly check-in are ready.</p>
   <button class="primary" onclick={async()=>{try{await api.request_notification_permission()}catch(e){error=String(e);return}localStorage.setItem('notifications-step-done','1');navigate('today')}}>Enable notifications</button>
   <button class="secondary" onclick={()=>{localStorage.setItem('notifications-step-done','1');navigate('today')}}>Not now</button>
  {/if}
  {#if error || status?.lastError}<p class="small" role="alert">{error || status?.lastError}</p>{/if}
 </section>
{:else if route==='settings'}
 <Header title="Settings" eyebrow="RARING" back={back} backLabel={parentLabel}/><div class="dashboard" {@attach masonry}><Card title="Notifications">{#each (Object.keys(settings) as (keyof api.Settings)[]).filter(key=>typeof settings[key]==='boolean' && key!=='adaptiveToday') as key}<label class="setting"><span>{({notifyMorningReadiness:'Morning readiness',notifyWeeklyCheckin:'Weekly check-in',notifyMetricChanges:'VO₂ max or Fitness Age changed',notifyVitals:'Vitals off',notifyTrainingStatus:'Training status changed',notifyBedtime:'Bedtime'} as Record<string,string>)[key]}</span><input type="checkbox" checked={settings[key]===true} onchange={(e)=>{settings={...settings,[key]:e.currentTarget.checked};void saveSettings()}}/></label>{#if key==='notifyBedtime' && settings.notifyBedtime}<label class="setting lead"><span>Wind-down reminder</span><select value={settings.bedtimeLeadMin} onchange={(e)=>{settings={...settings,bedtimeLeadMin:Number(e.currentTarget.value)};void saveSettings()}}>{#each [15,30,45,60,90] as lead}<option value={lead}>{lead} min before</option>{/each}</select></label>{/if}{/each}</Card><Card title="Today"><label class="setting"><span>Follow my day<small class="setting-note">Lead with this morning, the day so far or tonight's sleep. Turn off for shift work or an irregular schedule.</small></span><input type="checkbox" checked={settings.adaptiveToday} disabled={savingTodayLayout} onchange={(e)=>void saveTodayLayout(e.currentTarget.checked)}/></label></Card><Card title="Permission"><button class="secondary" onclick={async()=>{try{await api.request_notification_permission()}catch(e){error=String(e)}}}>Enable notifications</button></Card><Card title="Garmin history"><div class="big n">{status?.historyFrom?`Since ${monthLabel(status.historyFrom)}`:'—'}</div><p class="small">{status?.historyComplete?'All of your Garmin history is on this phone.':'Downloading older history in the background.'}</p></Card><Card title="Export"><p class="small">Download a ZIP of your saved Garmin responses, daily CSVs and Raring settings. Sign-in tokens are never included. The file leaves your phone only when you share it.</p><button class="secondary" disabled={exporting} onclick={exportData}>{exporting?'Preparing your export…':'Export your data'}</button>{#if exportMessage}<p class="small" role="status">{exportMessage}</p>{/if}</Card><Card title="Account"><button class="signout" onclick={async()=>{if(confirm('Sign out and erase all local Garmin history?')){try{await api.sign_out();clearViews();data=null;localStorage.removeItem('notifications-step-done');onboardStep=0;navigate('onboarding')}catch(e){error=String(e)}}}}>Sign out</button></Card><Card title="Help" onclick={()=>navigate('help')}><p class="small">How Raring reads strength sets, the body map and which watches work.</p></Card><Card title="How Raring works"><AboutRaring/></Card></div>
{:else if route==='help'}
 <Header title="Help" eyebrow="RARING" back={back} backLabel={parentLabel}/><div class="help-list">{#each helpSections as section}<Card title={section.title}>{#each section.pages as page}<button class="help-row" onclick={()=>navigate('help/'+page)}><span class="help-icon"><Icon name={helpContent[page].icon} size={18}/></span><b>{helpContent[page].title}</b><Icon name="chevron" size={16}/></button>{/each}</Card>{/each}<p class="small help-foot">Still stuck? Ask at {GITHUB_QUESTIONS}</p></div>
{:else if route.startsWith('help/') && route.slice(5) in helpContent}
 {@const page=helpContent[route.slice(5) as keyof typeof helpContent]}<Header title={page.title} eyebrow="HELP" back={back} backLabel={parentLabel}/><div class="help-list"><Card><p class="help-intro">{page.intro}</p><ol class="help-steps">{#each page.steps as step,i}<li><span class="n">{i+1}</span>{step}</li>{/each}</ol></Card><p class="small help-foot">Still stuck? Ask at {GITHUB_QUESTIONS}</p></div>
{:else if loading && !hasRendered}<div class="empty"><BrandMark/><h2>{status?.lastSync?'Loading your day…':'Syncing your Garmin data…'}</h2></div>
{:else if !data && hasRendered}<Header title={route[0].toUpperCase()+route.slice(1)} eyebrow="RARING"/><p class="small">{error || (loading?'Preparing this view…':'No data for this view yet.')}</p>
{:else if !data}<div class="empty"><BrandMark/><h2>Syncing your Garmin data…</h2><p>{error && !error.includes('not_ready')?error:'Your day will appear here soon.'}</p><button class="secondary" disabled={syncBusy} onclick={syncNow}>{syncBusy?'Syncing…':'Sync now'}</button>{#if syncError}<p>{syncError}</p>{/if}</div>
{:else if todayData && route==='today'}
 <div class="page-hero today-hero"><Header title="Today" eyebrow={dateLabel(todayData.date)} onSettings={()=>navigate('settings')}/>
 <div class="sync-indicator"><span class="small" aria-live="polite">{#if todayData.watchLastUpload}{watchLabel(todayData.watchLastUpload,now)}{:else}Watch upload time unavailable{/if}{#if syncBusy} · Refreshing…{:else if syncError || status?.lastError} · Raring sync delayed · {syncError || status?.lastError}{/if}</span><button class="sync-button" disabled={syncBusy} onclick={syncNow}>Refresh</button></div>
 <section class="dials"><button class="dial-button" onclick={()=>navigate('sleep')}>{#if todayData.sleep}<Gauge size={100} value={todayData.sleep.value} max={todayData.sleep.max} color={toneColor(todayData.sleep.tone)} caption="Sleep"/>{:else}<Gauge size={100} value={null} caption="Sleep"/>{/if}</button><button class="dial-button large" onclick={()=>navigate('readiness')}>{#if todayData.readiness}<Gauge size={156} stroke={13} value={todayData.readiness.value} max={todayData.readiness.max} color={toneColor(todayData.readiness.tone)} caption="Readiness" label={todayData.readiness.label}/>{:else}<Gauge size={156} stroke={13} value={null} caption="Readiness"/>{/if}</button><button class="dial-button" onclick={()=>navigate('training')}>{#if todayData.load}<Gauge size={100} value={todayData.load.value} max={todayData.load.max} color={toneColor(todayData.load.tone)} caption="Load" band={todayData.load.band} marker/>{:else}<Gauge size={100} value={null} caption="Load"/>{/if}</button></section>
 <div class="headline"><h2>{todayData.headline||'Your day is taking shape'}</h2><div class="chips">{#each todayData.chips as chip}<Chip {chip} label={chip.icon==='moon'?'View sleep':chip.icon==='hrv'?'View HRV':'View training'} onclick={()=>navigate(chip.icon==='moon'?'sleep':chip.icon==='hrv'?'metric/hrv':'training')}/>{/each}</div>{#if todayData.reconcile}<Reconcile reconcile={todayData.reconcile}/>{/if}{#if todayData.focus.note}<p class="phase-note"><Icon name="clock" size={14}/>{todayData.focus.note}</p>{/if}</div></div><div class="dashboard" {@attach masonry}>
 {#if watchNeedsSync(todayData,now)}<Card title="Watch upload" extra="watch-notice"><p>Open Garmin Connect to sync your watch</p>{#if todayData.watchSleepUploadStale}<p class="small">Last night's sleep hasn't uploaded yet.</p>{/if}{#if connectUnavailable}<p class="small" role="status">Open Garmin Connect on your phone, then sync your watch there.</p>{:else}<button class="watch-open" onclick={openConnect}>Open Garmin Connect <Icon name="arrow-right" size={16}/></button>{/if}</Card>{/if}
 {#if Object.keys(todayData.missingMetricReasons).length}<Card title="About missing readings" extra="watch-notice"><div class="missing-reasons">{#each Object.values(todayData.missingMetricReasons) as reason}<p>{reason}</p>{/each}</div></Card>{/if}
 {#each todayData.focus.cards as card,i (card)}
  {#if card==='focus' && (todayData.call || todayData.factors.length || todayData.focus.phase==='wake')}<DailyFocus today={todayData} {now} lead={i===0} open={()=>navigate('readiness')}/>
  {:else if card==='now'}<NowCard today={todayData} lead={i===0} open={navigate}/>
  {:else if card==='sleepPlan' && todayData.sleepPlan}<SleepPlanCard plan={todayData.sleepPlan} {now} lead={i===0} mode={todayData.focus.planOpen?(({evening:'evening',windDown:'windDown',lateNight:'lateNight'} as const)[todayData.focus.phase as 'evening']??'full'):'line'} open={()=>navigate('sleep')}/>
  {:else if card==='vitals' && todayData.vitals.readings.length}<VitalsMonitor vitals={todayData.vitals} compact open={(kind)=>navigate('metric/'+kind)}/>{/if}
 {/each}
 {#if todayData.plan}<PlanCard week={todayData.plan} open={()=>navigate('plan')}/>{/if}

 {#if todayData.bodyBattery.now!=null || todayData.hrv.lastNight!=null}<div class="row">{#if todayData.bodyBattery.now!=null}<Card title="Body Battery" onclick={()=>navigate('metric/bodyBattery')}><div class="bb"><div class="battery"><i style:height={Math.max(0,Math.min(100,todayData.bodyBattery.now))+'%'} style:background={batteryColor(todayData.bodyBattery.now)}></i></div><div><div class="big n" style:color={batteryColor(todayData.bodyBattery.now)}>{num(todayData.bodyBattery.now)}</div>{#if todayData.bodyBattery.charged!=null}<div class="small"><b>+{num(todayData.bodyBattery.charged)}</b> charged today</div>{/if}</div></div></Card>{/if}{#if todayData.hrv.lastNight!=null}<Card title="HRV" onclick={()=>navigate('metric/hrv')}>{#snippet aside()}{#if todayData.hrv.status}<span class="pill" style:color={toneColor(todayData.hrv.tone)}>{todayData.hrv.status}</span>{/if}{/snippet}<div class="big n">{num(todayData.hrv.lastNight)}<span class="unit">ms</span></div><div class="spark-space"><Sparkline values={todayData.hrv.week} band={todayData.hrv.baseline} w={138} h={34} color={toneColor(todayData.hrv.tone)}/></div></Card>{/if}</div>{/if}
 {#if todayData.stress.average!=null || todayData.stress.timeline.length}{@const stress=todayData.stress}<Card title="Stress" onclick={()=>navigate('metric/stress')}>{#snippet aside()}{#if stress.qualifier}<span class="pill" style:color={toneColor(stress.tone)}>{stress.qualifier}</span>{/if}{/snippet}<div class="stress-summary"><div><div class="big n" style:color={stress.level?stressColors[stress.level]:undefined}>{num(stress.average)}</div><div class="small">{stress.level?stressNames[stress.level]+' · ':''}average</div></div>{#if stress.timeline.length}<StressTimeline {stress} compact/>{/if}</div></Card>{/if}
 {#if todayData.vitals.readings.length && !todayData.focus.cards.includes('vitals')}<VitalsMonitor vitals={todayData.vitals} compact open={(kind)=>navigate('metric/'+kind)}/>{/if}
 {#if todayData.activities.length}<Card title="Activities">{#each todayData.activities as a}<ActivityRow activity={a} open={()=>navigate('workout/'+a.id)}/>{#if a.aerobicTe!=null || a.benefit}<div class="tags">{#if a.aerobicTe!=null}<span class="pill">Aerobic {a.aerobicTe} · {a.aerobicLabel}</span>{/if}{#if a.benefit}<span class="pill">{a.benefit}</span>{/if}</div>{/if}{/each}</Card>{/if}
 {#if todayData.fitnessAge.fitnessAge!=null}<Card title="Fitness Age" onclick={()=>navigate('health')}><div class="age-small"><div><div class="big n" style:color={fitnessAgeColor(todayData.fitnessAge.fitnessAge,todayData.fitnessAge.chronologicalAge)}>{num(todayData.fitnessAge.fitnessAge)}</div>{#if todayData.fitnessAge.chronologicalAge!=null}<span class="pill" style:color={fitnessAgeColor(todayData.fitnessAge.fitnessAge!,todayData.fitnessAge.chronologicalAge)}>{num(todayData.fitnessAge.chronologicalAge-todayData.fitnessAge.fitnessAge)} yrs younger</span>{/if}</div><Sparkline values={todayData.fitnessAge.trend} w={160} h={64} color={colors.ink}/></div></Card>{/if}
 {#if !todayData.plan}<Card title="Weekly plan" onclick={()=>navigate('plan')} extra="plan-invite"><div class="checkin-tease"><span>Set targets for strength, intensity, Zone 2 and sleep timing</span><b><Icon name="arrow-right" size={18}/></b></div></Card>{/if}
 {#if todayData.checkinReady}<Card title="Weekly check-in" onclick={()=>navigate('checkin')}><div class="checkin-tease"><span>See your week at a glance</span><b><Icon name="arrow-right" size={18}/></b></div></Card>{/if}
</div>
{:else if route==='readiness' && todayData}
 <div class="page-hero"><Header title="Readiness" eyebrow={dateLabel(todayData.date)} back={back} backLabel="Today"/><div class="detail-gauge">{#if todayData.readiness}<Gauge size={240} stroke={17} value={todayData.readiness.value} max={todayData.readiness.max} label={todayData.readiness.label} caption="Readiness" color={toneColor(todayData.readiness.tone)}/>{:else}<p>Waiting for readiness</p>{/if}<h2>{todayData.headline}</h2>{#if todayData.readinessWhy}<div class="why-line"><WhySheet why={todayData.readinessWhy} label={todayData.readiness?.label?`Why ${todayData.readiness.label}?`:'Why?'}/></div>{/if}</div></div><div class="dashboard" {@attach masonry}>
 {#if todayData.factors.length}<Card title="What's driving it"><div class="factor-list">{#each [...todayData.factors].sort((a,b)=>(a.percent??101)-(b.percent??101)) as factor}<button class="factor-row" aria-expanded={selectedFactor===factor.key} onclick={()=>selectedFactor=selectedFactor===factor.key?'':factor.key}><span>{factor.name}</span><b class="n">{num(factor.percent)}</b><div class="bar"><i style:width={(factor.percent??0)+'%'} style:background={toneColor(factor.tone)}></i></div>{#if selectedFactor===factor.key}<p>{factor.detail}</p>{/if}</button>{/each}</div></Card>{/if}
 {#if todayData.readinessTrend.length}<Card title="Readiness history"><ReadinessTrend values={todayData.readinessTrend} date={todayData.date}/></Card>{/if}
 {#if todayData.call}<Card title="Today's call"><div class="segments">{#each ['rest','easy','steady','push'] as effort}<span class:on={todayData.call===effort}>{effort}</span>{/each}</div>{#if todayData.callNote}<div class="sub">{todayData.callNote}</div>{/if}</Card>{/if}
</div>
{:else if sleepData}
 <div class="page-hero"><Header title="Sleep" eyebrow={sleepData.startLocal?`${clock(sleepData.startLocal)} – ${clock(sleepData.endLocal)}`:dateLabel(sleepData.date)} onSettings={()=>navigate('settings')}/><section class="sleep-hero">{#if sleepData.score}<Gauge size={150} stroke={12} value={sleepData.score.value} max={sleepData.score.max} caption="Score" label={sleepData.score.label} color={toneColor(sleepData.score.tone)}/>{/if}<div class="need"><div class="eyebrow">Asleep</div><div class="big n">{duration(sleepData.asleepMin)}</div><div class="needbar"><i style:width={Math.min(100,(sleepData.asleepMin??0)/Math.max(540,sleepData.needMin??0)*100)+'%'}></i>{#if sleepData.needMin!=null}<u style:left={sleepData.needMin/Math.max(540,sleepData.needMin)*100+'%'}></u>{/if}</div><div class="need-sub">{#if sleepData.needMin!=null}Last night's need <b>{duration(sleepData.needMin)}</b>{#if sleepData.asleepMin!=null && Math.abs(sleepData.asleepMin-sleepData.needMin)>=1} <span class="nowrap">&nbsp;· <strong style:color={sleepData.asleepMin<sleepData.needMin?colors.poor:colors.good}>{signed(sleepData.asleepMin-sleepData.needMin)} min</strong></span>{/if}{:else}Sleep need unavailable{/if}</div></div></section></div><div class="dashboard" {@attach masonry}>
 {#if sleepData.stages.length}<Card title="Stages"><Hypnogram segments={sleepData.stages} startLocal={sleepData.startLocal}/></Card>{/if}{#if sleepData.totals.length}<Card><div class="stage-stack">{#each sleepData.totals as s}<i style:width={s.minutes/sleepData.totals.reduce((a,b)=>a+b.minutes,0)*100+'%'} style:background={({deep:colors.deep,light:colors.light,rem:colors.rem,awake:colors.awake} as Record<string,string>)[s.stage]}></i>{/each}</div><div class="stage-legend">{#each sleepData.totals as s}<div><i style:background={({deep:colors.deep,light:colors.light,rem:colors.rem,awake:colors.awake} as Record<string,string>)[s.stage]}></i><span>{s.stage==='rem'?'REM':s.stage}</span><b class="n">{duration(s.minutes)}</b>{#if s.usualDeltaMin!=null && Math.abs(s.usualDeltaMin)>=1}<em style:color={s.stage==='light'?colors.secondary:s.stage==='awake'?(s.usualDeltaMin<0?colors.good:colors.poor):(s.usualDeltaMin>0?colors.good:colors.poor)}>{signed(Math.round(s.usualDeltaMin),'m')}</em>{/if}</div>{/each}</div></Card>{/if}
 {#if sleepData.overnight.length}<Card title="Overnight"><div class="tiles">{#each sleepData.overnight as tile}<div class="tile"><span><Icon name={tile.kind==='heartRate'?'heart':tile.kind==='spo2'?'drop':tile.kind==='respiration'?'lungs':'hrv'} size={14}/><span class="muted">{({heartRate:'Heart rate',hrv:'HRV',respiration:'Respiration',spo2:'SpO₂'} as Record<string,string>)[tile.kind]}</span></span><b class="n">{num(tile.value,tile.kind==='respiration'?1:0)}<small>{tile.unit}</small></b><Sparkline values={tile.series} w={128} h={30} color={colors.ink} fill={false}/></div>{/each}</div></Card>{/if}
 {#if sleepData.consistency.length}<Card title="Consistency" extra="consistency">{#snippet aside()}{#if sleepData.consistencySpreadMin!=null && sleepData.consistencySpreadMin>=1}<span class="pill">±{Math.round(sleepData.consistencySpreadMin)} min</span>{/if}{/snippet}{#if sleepData.garminAverages}{@const avg=sleepData.garminAverages}<div class="averages"><div><span>Avg bedtime</span><b class="n">{avg.bedtime}</b></div><div><span>Avg wake</span><b class="n">{avg.wakeTime}</b></div><small>Garmin · last {avg.nights} {avg.nights===1?'night':'nights'}</small></div>{/if}<ConsistencyChart nights={sleepData.consistency} date={sleepData.date} bedWindow={sleepData.bedWindow} wakeWindow={sleepData.wakeWindow}/>{#if sleepData.bedWindow || sleepData.wakeWindow}<p class="small">Shaded: your usual bed and wake times.</p>{/if}</Card>{/if}
 <Card title="Sleep history"><HistoryChart values={[]} endDate={sleepData.date} label="Sleep score" metric="sleepScore" initialRange="days30" nights/></Card>
 {#if sleepData.alignment}<AlignmentCard alignment={sleepData.alignment}/>{/if}
 {#if sleepData.sleepPlan}<SleepPlanCard plan={sleepData.sleepPlan} evidence/>{/if}
</div>
{:else if trainingData}
 <div class="page-hero"><Header title="Training" eyebrow="Last 7 days" onSettings={()=>navigate('settings')}/>{#if trainingData.status}<span class="status-pill pill" style:color={toneColor(trainingData.statusTone)}>{trainingData.status}</span>{/if}
 {#if trainingData.acute}<section class="semi"><Gauge size={300} stroke={16} sweep={180} value={trainingData.acute.value} max={trainingData.acute.max} caption="Acute load" label={trainingData.acute.label} color={toneColor(trainingData.acute.tone)} band={trainingData.acute.band} marker/><span class="semi-low">Low</span><span class="semi-high">High</span></section>{/if}<div class="headline training-head"><h2>{trainingData.headline}</h2><div class="chips">{#each trainingData.chips as chip}<Chip {chip}/>{/each}</div>{#if trainingData.statusWhy}<div class="why-line"><WhySheet why={trainingData.statusWhy} label={trainingData.status?`Why ${trainingData.status}?`:'Why?'}/></div>{/if}</div></div><div class="dashboard" {@attach masonry}>
 {#if trainingData.week.length}<Card title="This week"><WeekBars days={trainingData.week}/></Card>{/if}{#if trainingData.loadFocus.length}<Card title="Load focus · 4 weeks"><LoadFocus bars={trainingData.loadFocus}/></Card>{/if}
 {#if trainingData.vo2max!=null || trainingData.races.length}<div class="row">{#if trainingData.vo2max!=null}<Card title="VO₂ max" onclick={()=>navigate('metric/vo2max')}><div class="big n">{num(trainingData.vo2max)}</div>{#if trainingData.vo2maxChange30d!=null && Math.abs(trainingData.vo2maxChange30d)>=1}<div class="small"><b style:color={trainingData.vo2maxChange30d>0?colors.good:colors.poor}>{signed(trainingData.vo2maxChange30d)}</b> in 30 days</div>{/if}<Sparkline values={trainingData.vo2maxTrend} w={138} h={30} color={colors.ink}/></Card>{/if}{#if trainingData.races.length}<Card title="Race times"><div class="race">{#each trainingData.races as race}<div><span>{race.label}</span><b class="n">{elapsed(race.seconds)}</b></div>{/each}</div></Card>{/if}</div>{/if}
 {#if trainingData.enduranceScore!=null || trainingData.hillScore!=null || trainingData.runningToleranceKm!=null}<Card title="More metrics"><div class="metrics">{#if trainingData.enduranceScore!=null}<button class="metric-link" onclick={()=>navigate('metric/endurance')}><b>{num(trainingData.enduranceScore)}</b><span>Endurance</span></button>{/if}{#if trainingData.hillScore!=null}<button class="metric-link" onclick={()=>navigate('metric/hill')}><b>{num(trainingData.hillScore)}</b><span>Hill score</span></button>{/if}{#if trainingData.runningToleranceKm!=null}<button class="metric-link" onclick={()=>navigate('metric/runningTolerance')}><b>{num(trainingData.runningToleranceKm)} km</b><span>Run tolerance</span></button>{/if}</div></Card>{/if}
 <Card title="Load history"><HistoryChart values={[]} label="Acute load" metric="acuteLoad" initialRange="days30"/></Card>
 {#if trainingData.strength}{@const st=trainingData.strength}<Card title="Strength · this week" onclick={()=>navigate('strength')}>{#snippet aside()}{#if st.records}<span class="pill ok"><Icon name="star" size={12}/>{st.records===1?'1 record':`${st.records} records`}</span>{/if}{/snippet}<div class="strength-week"><div><div class="big n">{st.sets}</div><div class="small">sets · {st.sessionsWithSets===1?'1 session':`${st.sessionsWithSets} sessions`} with sets{#if st.strengthActivities>st.sessionsWithSets}<br>{st.strengthActivities-st.sessionsWithSets} without sets{/if}</div></div><BodyMap figure={st.figure} muscles={st.muscles} width={96} max={12} legend={false}/></div></Card>{/if}
 {#if trainingData.workouts.length}<Card title="Recent workouts">{#each trainingData.workouts as a}<div class="workout-item"><ActivityRow activity={a} open={()=>navigate('workout/'+a.id)}/></div>{/each}</Card>{/if}
</div>
{:else if workoutData}
 {@const sets=workoutData.strength}{@const lifted=sets?.setsRecorded===true}
 <Header title={workoutData.summary.name} eyebrow={(sets?'Strength · ':'')+dateLabel(workoutData.summary.startLocal.slice(0,10))+' · '+clock(workoutData.summary.startLocal)} back={back} backLabel={parentLabel} onHelp={sets?()=>sheet=true:undefined}/>{#if sets && lifted}<StrengthWorkout workout={workoutData} strength={sets} help={()=>sheet=true}/>{/if}<div class="dashboard" {@attach masonry}>{#if sets && !lifted}<Card title="No sets recorded"><p class="small untracked">Your watch timed this session as one set, without reps or exercises. Press lap after each set next time.</p><button class="watch-open" onclick={()=>navigate('help/strength-start')}>Lift with your watch <Icon name="arrow-right" size={16}/></button></Card>{/if}{#if !lifted && (workoutData.summary.load!=null || workoutData.summary.benefit || workoutData.recoveryHours!=null || workoutData.summary.aerobicTe!=null || workoutData.summary.anaerobicTe!=null)}<Card extra="workout-overview"><div class="workout-hero"><div>{#if workoutData.summary.load!=null}<div class="eyebrow">Training load</div><div class="load-number n">{num(workoutData.summary.load)}</div>{/if}<div class="tags">{#if workoutData.summary.benefit}<span class="pill">{workoutData.summary.benefit}</span>{/if}{#if workoutData.recoveryHours!=null}<span class="pill"><Icon name="clock" size={13}/>{num(workoutData.recoveryHours,1)}h recovery</span>{/if}</div></div><div class="effects">{#if workoutData.summary.aerobicTe!=null}<div><Gauge size={82} stroke={7} value={workoutData.summary.aerobicTe} max={5} caption="Aerobic" color={colors.ink}/><small>{workoutData.summary.aerobicLabel}</small></div>{/if}{#if workoutData.summary.anaerobicTe!=null}<div><Gauge size={82} stroke={7} value={workoutData.summary.anaerobicTe} max={5} caption="Anaer." color={colors.ink}/><small>{workoutData.summary.anaerobicLabel}</small></div>{/if}</div></div></Card>{/if}
 {#if workoutData.stats.length && !lifted}<Card><div class="stats">{#each workoutData.stats as s}<div><b class="n">{s.key==='pace'?pace(s.value).replace('/km',''):s.key==='duration'?elapsed(s.value):num(s.value,s.key==='distance'||s.key==='speed'?1:0)}{#if s.key==='pace'}<small>/km</small>{:else if s.unit!=='s' && s.unit!=='bpm' && s.unit!=='kcal'}<small>{s.unit}</small>{/if}</b><span>{s.label}</span></div>{/each}</div></Card>{/if}
 {#if workoutData.route.length}<Card extra="map-card"><RouteMap route={workoutData.route}/></Card>{/if}{#if workoutData.heartRate.length}<Card title="Heart rate"><HrChart samples={workoutData.heartRate} floors={workoutData.zoneFloors} durationS={workoutData.summary.durationS}/></Card>{/if}
 {#if workoutData.summary.zoneSeconds.length}<Card title="Time in zones">{@const maxZone=Math.max(...workoutData.summary.zoneSeconds,1)}{@const total=workoutData.summary.zoneSeconds.reduce((a,b)=>a+b,0)}{#each [...workoutData.summary.zoneSeconds].reverse() as seconds,i}<div class="zone-row"><b>Z{5-i}</b><div class="bar"><i style:width={seconds/maxZone*100+'%'} style:background={colors.z[4-i]}></i></div><span class="n">{elapsed(seconds)}</span><small>{Math.round(seconds/Math.max(total,1)*100)}%</small></div>{/each}</Card>{/if}
 {#if workoutData.splits.length && !lifted}<WorkoutSplits splits={workoutData.splits}/>{/if}
</div>
{:else if strengthData}
 <Header title="Strength" eyebrow="Last 12 weeks" back={back} backLabel={parentLabel} onHelp={()=>navigate('help')}/>{#if strengthData.strengthPage}<StrengthHistory view={strengthData.strengthPage} {navigate}/>{:else}<Card title="No sets yet"><p class="small untracked">Start a Strength activity on your watch and press lap after each set. Your sets appear here after Garmin Connect syncs.</p><button class="watch-open" onclick={()=>navigate('help/strength-start')}>Lift with your watch <Icon name="arrow-right" size={16}/></button></Card>{/if}
{:else if healthData}
 <Header title="Health" eyebrow={dateLabel(healthData.date)} onSettings={()=>navigate('settings')}/><div class="dashboard health-dashboard" {@attach masonry}>
 <VitalsMonitor vitals={healthData.vitals} open={(kind)=>navigate('metric/'+kind)}/>
 <Card title="Heart rate · all day" onclick={()=>navigate('metric/heartRate')}><div class="big n">{num(healthData.heartRateDay.resting)}<small> bpm</small></div><div class="small">Garmin resting HR</div>{#if healthData.heartRateDay.timeline.length}<StressTimeline heartRate={healthData.heartRateDay} compact/>{/if}<p class="small">Sleep HR in overnight vitals is a separate value.</p></Card>
 {#if healthData.stress.average!=null || healthData.stress.trend.some(v=>v!=null)}{@const stress=healthData.stress}<Card title="Stress" onclick={()=>navigate('metric/stress')}>{#snippet aside()}{#if stress.qualifier}<span class="pill" style:color={toneColor(stress.tone)}>{stress.qualifier}</span>{/if}{/snippet}<div class="age-small"><div><div class="big n" style:color={stress.level?stressColors[stress.level]:undefined}>{num(stress.average)}</div><div class="small">Today's average</div></div><Sparkline values={stress.trend} w={160} h={64} color={colors.ink} fill={false}/></div><p class="small">Daily average · last 30 days</p></Card>{/if}
 {#if healthData.bodyWeight}{@const weight=healthData.bodyWeight}<Card title="Weight"><div class="age-small"><div><div class="big n">{num(weight.value,1)}<small>&nbsp;{weight.unit}</small></div><div class="small">Last measured {dateLabel(weight.measuredOn)}</div></div><Sparkline values={weight.trend} w={160} h={64} color={colors.ink} fill={false} points/></div><p class="small">{#if weight.bodyFat!=null}Body fat {num(weight.bodyFat,1)}% ·&nbsp;{/if}{#if weight.bmi!=null}BMI {num(weight.bmi,1)} ·&nbsp;{/if}Last 12 weeks · recorded weigh-ins only</p></Card>{/if}
 <Card title="Fitness Age">{#snippet aside()}{#if healthData.fitnessAge.change12m!=null && Math.abs(healthData.fitnessAge.change12m)>=1}<span class="pill" style:color={healthData.fitnessAge.change12m<0?colors.good:colors.poor}><Icon name={healthData.fitnessAge.change12m<0?'trend-down':'trend-up'} size={13}/>{Math.round(Math.abs(healthData.fitnessAge.change12m))} this year</span>{/if}{/snippet}<div class="age-hero"><div><div class="age-number n" style:color={healthData.fitnessAge.fitnessAge!=null?fitnessAgeColor(healthData.fitnessAge.fitnessAge,healthData.fitnessAge.chronologicalAge):undefined}>{num(healthData.fitnessAge.fitnessAge)}</div>{#if healthData.fitnessAge.fitnessAge!=null && healthData.fitnessAge.chronologicalAge!=null}<div class="age-caption"><b>{num(healthData.fitnessAge.chronologicalAge-healthData.fitnessAge.fitnessAge)} years younger</b> than your age</div>{/if}</div><Sparkline values={healthData.fitnessAge.trend} w={120} h={70} color={colors.ink}/></div>{#if healthData.fitnessAge.fitnessAge!=null && healthData.fitnessAge.chronologicalAge!=null}<div class="agebar" style:--age-status={fitnessAgeColor(healthData.fitnessAge.fitnessAge,healthData.fitnessAge.chronologicalAge)}><div class="age-mark fit" style:left={Math.max(0,Math.min(100,(healthData.fitnessAge.fitnessAge-20)/50*100))+'%'}><span>YOU {num(healthData.fitnessAge.fitnessAge)}</span></div><div class="age-mark real" style:left={Math.max(0,Math.min(100,(healthData.fitnessAge.chronologicalAge-20)/50*100))+'%'}><span>AGE {num(healthData.fitnessAge.chronologicalAge)}</span></div></div>{/if}</Card>
 <Card title="Fitness Age history"><HistoryChart values={[]} endDate={healthData.date} label="Fitness Age" unit="years" metric="fitnessAge" initialRange="year"/></Card>
 {#if healthData.intensity}<IntensityCard intensity={healthData.intensity} open={()=>navigate('plan')}/>{/if}
 {#if healthData.lever}<Card title="Biggest lever"><div class="lever">{#if ['restingHr','bmi','bodyFat'].includes(healthData.lever.key)}<Icon name="trend-down" size={60}/>{:else}<Gauge size={76} stroke={8} value={healthData.lever.current} max={healthData.lever.target} display={Math.round(healthData.lever.current/healthData.lever.target*100)+'%'} color={healthData.lever.current>=healthData.lever.target?colors.good:colors.fair}/>{/if}<div><b>{healthData.lever.name}</b><span>{num(healthData.lever.current)} / {num(healthData.lever.target)} {healthData.lever.unit}</span>{#if healthData.lever.ageIfMet!=null}<span class="pill">Hit {num(healthData.lever.target,1)} <Icon name="arrow-right" size={13}/> age {num(healthData.lever.ageIfMet,1)}</span>{/if}</div></div></Card>{/if}
 {#if healthData.drivers.length}<Card title="What sets it"><div class="drivers">{#each healthData.drivers as d}<div class="driver"><span>{d.name} {#if d.target!=null}<i style:color={toneColor(d.tone)}><Icon name={d.met?'check':['restingHr','bmi','bodyFat'].includes(d.key)?'trend-down':'trend-up'} size={16}/></i>{/if}</span><b class="n">{num(d.value,d.key==='bmi'||d.key==='vigorousDays'?1:0)}<small>{d.unit}</small></b><RangeBar value={d.value} scale={driverScale(d.key)} range={driverRange(d.key,d.target)} color={toneColor(d.tone)}/></div>{/each}</div></Card>{/if}

</div>
{:else if planData && route==='plan'}
 <PlanScreen view={planData} back={back} edit={()=>navigate('plan-setup')}/>
{:else if planData && route==='plan-setup'}
 <PlanSetup view={planData} save={savePlan} back={back}/>
{:else if route.startsWith('metric/')}
 <Header title={route.split('/')[1]==='restingHr' && (data as HealthView).vitals?.readings.some(v=>v.kind==='restingHr' && (v.status || v.rangeSource==='garmin')) ? 'Sleep HR' : metricNames[route.split('/')[1]]} eyebrow="GARMIN METRIC" back={back} backLabel={parentLabel}/><MetricDetail metric={route.split('/')[1]} data={data as TodayView|HealthView|TrainingView}/>
{:else if checkinData}
 <Header title="Weekly check-in" eyebrow={'Week of '+dateLabel(checkinData.weekStart)} back={back} backLabel="Today"/><div class="checkin-progress">{#each Array(checkinPages+1) as _,i}<span class:on={i===checkinIndex}></span>{/each}</div><div class="checkin-swiper" role="group" aria-label="Weekly check-in cards" ontouchstart={(e)=>swipeX=e.touches[0].clientX} ontouchend={(e)=>{let dx=e.changedTouches[0].clientX-swipeX;if(Math.abs(dx)>50)checkinIndex=Math.max(0,Math.min(checkinPages,checkinIndex+(dx<0?1:-1)))}}>{#if checkinIndex<checkinData.cards.length}{@const c=checkinData.cards[checkinIndex]}<Card title={c.area}><div class="checkin-content"><div class="eyebrow">YOUR WEEK IN {c.area.toUpperCase()}</div>{#if c.value!=null}<Gauge size={210} stroke={15} value={c.value} max={c.area==='training'?1000:c.area==='health'?50:100} color={toneColor(c.tone)} caption={c.unit}/>{/if}<h2>{c.headline}</h2><span class="pill" class:ok={c.previous!=null && c.value!=null && (c.area==='health'?c.value<=c.previous:c.value>=c.previous)}>{#if c.previous!=null && c.value!=null && Math.abs(c.value-c.previous)>=1}<Icon name={c.value<c.previous?'trend-down':'trend-up'} size={14}/>{signed(c.value-c.previous)} vs last week{:else if c.value==null}No readings this week{:else if c.previous!=null}Steady vs last week{:else}No previous week to compare{/if}</span><Sparkline values={c.series} w={240} h={60} color={colors.ink}/></div></Card>{:else if checkinPage==='timing' && checkinData.timing}{@const t=checkinData.timing}<Card title="Timing" extra="timing-review"><div class="checkin-content"><div class="eyebrow">YOUR WEEK IN SLEEP TIMING</div><ConsistencyChart nights={t.nights} date={addDays(checkinData.weekStart,6)} days={7}/><div class="timing-ranges">{#if t.bedRange}<div><span>Bedtimes</span><b class="n">{t.bedRange[0]}–{t.bedRange[1]}</b></div>{/if}{#if t.wakeRange}<div><span>Wake times</span><b class="n">{t.wakeRange[0]}–{t.wakeRange[1]}</b></div>{/if}</div>{#if t.aligned+t.ahead+t.behind>0}<span class="pill">Garmin: {[[t.aligned,'aligned'],[t.behind,'behind'],[t.ahead,'ahead']].filter(([n])=>Number(n)>0).map(([n,w])=>`${n} ${w}`).join(' · ')}</span>{/if}<h3>THE TIMING FOCUS</h3><p>{t.focus}</p></div></Card>{:else if checkinPage==='intensity' && checkinData.intensity}{@const im=checkinData.intensity}<Card title="Intensity" extra="intensity-review"><div class="checkin-content"><div class="eyebrow">YOUR WEEK IN INTENSITY MINUTES</div><Gauge size={210} stroke={15} value={im.total} max={im.goal??Math.max(im.total,1)} display={num(im.total)} caption={im.goal!=null?`of ${num(im.goal)} min`:'min'} color={im.goal!=null&&im.total>=im.goal?colors.good:colors.ink}/><h2>{im.goal==null?'Intensity minutes':im.total>=im.goal?"Garmin's goal reached":`${num(Math.ceil(im.goal-im.total))} min short`}</h2><span class="pill">{num(im.moderate)} moderate · {num(im.vigorous)} vigorous, counted twice</span></div></Card>{:else if checkinPage==='plan' && checkinData.plan}<PlanCheckinCard plan={checkinData.plan} keep={()=>checkinIndex++} change={()=>navigate('plan-setup')}/>{:else}<Card title="Your week ahead"><div class="checkin-summary"><div class="brand-mark"><Icon name="sparkle" size={70}/></div><h2>Keep the momentum</h2>{#if checkinData.vo2maxChange!=null && Math.abs(checkinData.vo2maxChange)>=1}<span class="pill ok">VO₂ max {signed(checkinData.vo2maxChange)}</span>{/if}{#if checkinData.fitnessAgeChange!=null && Math.abs(checkinData.fitnessAgeChange)>=1}<span class="pill ok">Fitness Age {signed(checkinData.fitnessAgeChange)}</span>{/if}{#if checkinData.win}<h3>THE WIN</h3><p>{checkinData.win}</p>{/if}{#if checkinData.focus}<h3>THE FOCUS</h3><p>{checkinData.focus}</p>{/if}</div></Card>{/if}</div><div class="checkin-controls"><button disabled={checkinIndex===0} onclick={()=>checkinIndex--}><Icon name="arrow-left" size={16}/> Previous</button><span>{checkinIndex+1} / {checkinPages+1}</span><button disabled={checkinIndex===checkinPages} onclick={()=>checkinIndex++}>Next <Icon name="arrow-right" size={16}/></button></div>
{/if}
</div></main>{#if sheet && workoutData?.strength}<HelpSheet close={()=>sheet=false} more={()=>{sheet=false;navigate('help/strength-sets')}}/>{/if}{#if route!=='onboarding'}<TabBar active={activeTab} {navigate}/>{/if}</div>
