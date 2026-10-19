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
 import TodayHero from './lib/components/TodayHero.svelte';
 import AdaptSheet from './lib/components/AdaptSheet.svelte';
 import type {AdaptState} from './lib/generated/AdaptState';
 import AboutRaring from './lib/components/AboutRaring.svelte';
 import SleepPlanCard from './lib/components/SleepPlanCard.svelte';
 import NowCard from './lib/components/NowCard.svelte';
 import {phaseEnded} from './lib/tonight';
 import AlignmentCard from './lib/components/AlignmentCard.svelte';
 import WorkoutSplits from './lib/components/WorkoutSplits.svelte';
 import VitalsMonitor from './lib/components/VitalsMonitor.svelte';
 import StressTimeline from './lib/components/StressTimeline.svelte';
 import PlannedSessionCard from './lib/components/PlannedSessionCard.svelte';
 import PlanTab from './lib/screens/PlanTab.svelte';
 import ProgressTab from './lib/screens/ProgressTab.svelte';
 import ProfileScreen from './lib/screens/ProfileScreen.svelte';
 import GoalEditor from './lib/components/GoalEditor.svelte';
 import SearchSheet from './lib/components/SearchSheet.svelte';
 import DayCells from './lib/components/DayCells.svelte';
 import WeekSummary from './lib/components/WeekSummary.svelte';
 import AddSessionSheet,{type Draft} from './lib/components/AddSessionSheet.svelte';
 import ItemSheet from './lib/components/ItemSheet.svelte';
 import EventSheet from './lib/components/EventSheet.svelte';
 import WeekReviewCard from './lib/components/WeekReviewCard.svelte';
 import type {WeekReview} from './lib/generated/WeekReview';
 import type {ReviewRow} from './lib/generated/ReviewRow';
 import type {PlanItem} from './lib/generated/PlanItem';
 import type {PlanBoard} from './lib/generated/PlanBoard';
 import type {LocalAction} from './lib/generated/LocalAction';
 import IntensityCard from './lib/components/IntensityCard.svelte';
 import StrengthWorkout from './lib/components/StrengthWorkout.svelte';
 import StrengthHistory from './lib/components/StrengthHistory.svelte';
 import HelpSheet from './lib/components/HelpSheet.svelte';
 import BodyMap from './lib/components/BodyMap.svelte';
 import GearCard from './lib/components/GearCard.svelte';
 import {fitnessAgeColor} from './lib/fitness';
 import ReadinessDetail from './lib/screens/ReadinessDetail.svelte';
 import SleepDetail from './lib/screens/SleepDetail.svelte';
 import TrainingDetail from './lib/screens/TrainingDetail.svelte';
 import WorkoutDetail from './lib/screens/WorkoutDetail.svelte';
 import HealthDetail from './lib/screens/HealthDetail.svelte';
 import {help as helpContent,helpSections,GITHUB_QUESTIONS} from './lib/help';
 import type {StrengthView} from './lib/generated/StrengthView';
 import WhySheet from './lib/components/WhySheet.svelte';
 import Reconcile from './lib/components/Reconcile.svelte';
 import {closeSheet} from './lib/sheet';
 import * as api from './lib/api';
 import type {TodayView} from './lib/generated/TodayView';import type {SleepView} from './lib/generated/SleepView';import type {TrainingView} from './lib/generated/TrainingView';import type {WorkoutView} from './lib/generated/WorkoutView';import type {HealthView} from './lib/generated/HealthView';import type {CheckinView} from './lib/generated/CheckinView';import type {AppStatus} from './lib/generated/AppStatus';
 import Icon from './lib/components/Icon.svelte';import BrandMark from './lib/components/BrandMark.svelte';import ReadinessTrend from './lib/components/ReadinessTrend.svelte';
 import Gauge from './lib/components/Gauge.svelte';import Sparkline from './lib/components/Sparkline.svelte';import RangeBar from './lib/components/RangeBar.svelte';import ZoneStrip from './lib/components/ZoneStrip.svelte';import Hypnogram from './lib/components/Hypnogram.svelte';import ConsistencyChart from './lib/components/ConsistencyChart.svelte';import WeekBars from './lib/components/WeekBars.svelte';import LoadFocus from './lib/components/LoadFocus.svelte';import RouteMap from './lib/components/RouteMap.svelte';import HrChart from './lib/components/HrChart.svelte';import Card from './lib/components/Card.svelte';import Chip from './lib/components/Chip.svelte';import Header from './lib/components/Header.svelte';import TabBar from './lib/components/TabBar.svelte';import ActivityRow from './lib/components/ActivityRow.svelte';
 import {colors,toneColor,batteryColor,stressColors,stressNames} from './lib/charts';import {duration,elapsed,pace,clock,dateLabel,monthLabel,signed,num} from './lib/format';
 type StrengthPage={strengthPage:StrengthView|null};
 type ProgressPage={progress:import('./lib/generated/ProgressView').ProgressView;health:HealthView};
 type PageData=TodayView|SleepView|TrainingView|WorkoutView|HealthView|CheckinView|PlanBoard|StrengthPage|ProgressPage;
 let sheet=$state(false),search=$state(false),addDraft=$state<Draft|null>(null),itemSheet=$state<PlanItem|null>(null),eventSheet=$state(false),reviewData=$state<WeekReview|null>(null),reviewRows=$state<ReviewRow[]|null>(null),lastPhase=$state<string|null>(null),adaptOpen=$state(false),adaptState=$state<AdaptState|null>(null),weekBoard=$state<PlanBoard|null>(null),statusOffer=$state(false),savingStatus=$state(false);
 let checkingStatusOffer=false,settingsRevision=0;
 let settingsBusy=$state<Partial<Record<keyof api.Settings,boolean>>>({});
 function canApplySettingsRead(revision:number){return revision===settingsRevision && !Object.values(settingsBusy).some(Boolean) && !savingStatus}
 async function patchSettings(patch:api.SettingsPatch):Promise<boolean>{
  const keys=Object.keys(patch) as (keyof api.Settings)[];
  if(keys.some(key=>settingsBusy[key]))return false;
  const previous=Object.fromEntries(keys.map(key=>[key,settings[key]])) as api.SettingsPatch;
  settingsBusy={...settingsBusy,...Object.fromEntries(keys.map(key=>[key,true]))};
  settingsRevision++;
  settings={...settings,...patch};
  try{await api.patch_settings(patch);return true}
  catch(e){settings={...settings,...previous};error=String(e);return false}
  finally{settingsBusy={...settingsBusy,...Object.fromEntries(keys.map(key=>[key,false]))};settingsRevision++}
 }
 async function offerStatus(){
  if(!api.supportsStatusSurfaces || route!=='today' || !status?.lastSync || checkingStatusOffer)return;
  checkingStatusOffer=true;
  const revision=settingsRevision;
  try{
   const saved=await api.get_settings();
   if(!canApplySettingsRead(revision) || saved.statusOfferSeen || saved.notifyStatus || route!=='today')return;
   if(await patchSettings({statusOfferSeen:true}))statusOffer=true;
  }catch(e){error=String(e)}finally{checkingStatusOffer=false}
 }
 async function toggleStatus(enabled:boolean){
  if(savingStatus || settingsBusy.notifyStatus || settingsBusy.statusOfferSeen)return;
  savingStatus=true;
  settingsRevision++;
  try{
   if(enabled && !await api.request_notification_permission()){
    error='Notifications are blocked. Allow notifications for Raring in Android Settings, then try again.';
    return;
   }
   if(await patchSettings({notifyStatus:enabled,statusOfferSeen:true})){statusOffer=false;error=''}
  }catch(e){error=String(e)}finally{savingStatus=false;settingsRevision++}
 }
 let journalOpen=$state(false),journalDayOpen=$state<string|null>(null);
 async function toggleJournal(enabled:boolean){
  if(settingsBusy.journalEnabled)return;
  if(await patchSettings({journalEnabled:enabled})){
   journalOpen=false;journalDayOpen=null;clearViews();await refresh(true);
  }
 }
 function napWhen(local:string|null,label:string){
  if(!local || !/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}/.test(local) || clock(local)==='—')return `${label} date/time unavailable`;
  const day=dateLabel(local.slice(0,10));
  return day==='Date unavailable'?`${label} date/time unavailable`:`${day} ${clock(local)}`;
 }
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
  void tick().then(()=>{if(generation===cacheGeneration)return Promise.allSettled([...tabs,'sleep'].filter(tab=>tab!==route).map(tab=>readView(tab)))});
 }
 const fixture=new URLSearchParams(location.search).get('fixture');
 const live=isTauri() && fixture===null;
 let syncBusy=$derived(syncing || status?.syncing===true);
 // The plan-your-week prompt sits under every root header, never in wind-down or late night.
 let prompt=$derived(weekBoard?.prompt && !['windDown','lateNight'].includes(lastPhase??'')?weekBoard.prompt:null);
 let rootControls=$derived({onSearch:()=>search=true,onProfile:()=>navigate('profile'),onAdapt:()=>adaptOpen=true,adapt:adaptState?.label,below:prompt?promptCard:undefined});
 function answerPrompt(choice:'planned'|'dismissed'){if(weekBoard)act({action:'setPromptWeek',week:weekBoard.start,choice},choice==='planned'?()=>navigate('plan'):undefined)}
 // Every root header shows today's circumstances; they come with the live Today.
 function readAdapt(){void readView('today').then(v=>{adaptState=(v as TodayView).adapt??null}).catch(()=>{})}
 let activeTab=$derived([...trail].reverse().find(r=>(tabs as readonly string[]).includes(r))??'today');
 let parentLabel=$derived.by(()=>{const parent=trail.at(-2);if(!parent)return 'Today';if(parent.startsWith('workout/'))return 'Workout';if(parent.startsWith('help/'))return 'Help';return ({today:'Today',sleep:'Sleep',training:'Training',health:'Health',readiness:'Readiness',plan:'Plan',progress:'Progress',profile:'Profile',activities:'Activities','plan/commitments':'Goals',strength:'Strength',help:'Help',settings:'Settings'} as Record<string,string>)[parent]??'Back'});
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
   // The check-in is keyed by the Monday after its week; a review route names the week's start.
   case 'review':return api.checkin(id==='latest'?undefined:addDays(id,7));
   case 'plan':return api.plan_board();
   case 'progress':{const [progress,health]=await Promise.all([api.progress(),api.health()]);return {progress,health}}
   case 'profile':case 'gear':case 'activities':return api.training();
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
    const revision=settingsRevision;
    const nextSettings=await api.get_settings();
    if(request===requestId && canApplySettingsRead(revision))settings=nextSettings;
   }else if(current==='reviews'){
    reviewRows=null;void api.reviews().then(r=>{if(request===requestId)reviewRows=r}).catch(e=>error=String(e));
   }else if(current!=='onboarding' && current!=='help' && !current.startsWith('help/')){
    let next=await readView(current);
    if(request!==requestId)return;
    if(fixture==='missing' && current==='today'){
     next={...(next as TodayView),readiness:null,readinessTrend:[],sleep:null,sleepPlan:null,load:null,chips:[],factors:[],readinessWhy:null,bodyBatteryWhy:null,reconcile:null,call:null,callNote:null,bodyBattery:{now:null,charged:null,drained:null,timeline:[]},stress:{average:null,max:null,level:null,qualifier:null,tone:'neutral',timeline:[],active:[],sleep:null,time:[],activeS:null,trend:[]},hrv:{lastNight:null,status:null,tone:'neutral',baseline:null,week:[]},vitals:{readings:[],offCount:0,baselineReady:false},activities:[],fitnessAge:{fitnessAge:null,chronologicalAge:null,trend:[],change12m:null},headline:'',focus:{...(next as TodayView).focus,phase:'wake',cards:['focus'],effort:false,sleepMissing:true,lastNight:null}};
    }
    if(fixture==='stale' && current==='today')next={...(next as TodayView),watchLastUpload:new Date(Date.now()-7*3600_000).toISOString(),missingMetricReasons:{skinTemp:"Your watch doesn't record skin temperature"}};
    if(fixture==='fresh' && current==='today')next={...(next as TodayView),watchLastUpload:new Date(Date.now()-2*3600_000-120_000).toISOString()};
    if(fixture==='stale-sleep' && current==='today')next={...(next as TodayView),watchLastUpload:null,watchSleepUploadStale:true,missingMetricReasons:{}};
    const updated=fixture==='empty' && current==='today'?null:next;
    if(touching && data && updated)pendingSwap={route:current,data:updated};
    else data=updated;
    if(data){hasRendered=true;prefetchTabs()}
    if(current==='today' && updated){adaptState=(updated as TodayView).adapt??adaptState;lastPhase=(updated as TodayView).focus.phase}
    if(current==='plan' && updated)weekBoard=updated as PlanBoard;
    else if((tabs as readonly string[]).includes(current))void readView('plan').then(b=>{if(request===requestId)weekBoard=b as PlanBoard}).catch(()=>{});
    if(current.startsWith('review/') && updated){const week=(updated as CheckinView).weekStart;void api.week_review(week).then(r=>{if(request===requestId)reviewData=r}).catch(e=>error=String(e))}
    if(current==='reviews')void api.reviews().then(r=>{if(request===requestId)reviewRows=r}).catch(e=>error=String(e));
    else if(!adaptState)readAdapt();
   }
   if(hasRendered && !prefetched && current!=='onboarding')prefetchTabs();
   if(current==='today')void offerStatus();
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
   if(changed){scrollPositions.set(route,scroll?.scrollTop??0);pendingSwap=null;selectedFactor='';checkinIndex=0;sheet=false;search=false;addDraft=null;itemSheet=null;eventSheet=false;reviewData=null;journalOpen=false;journalDayOpen=null}
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
  const resume=()=>{if(document.visibilityState==='visible')void refresh(true,true);else {journalOpen=false;finishTouch()}};
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
 // Local changes run in order; each reloads every cached view, since Today shows the week too.
 let localChanges=Promise.resolve(),changing=$state(false);
 function act(action:LocalAction,then?:()=>void){changing=true;localChanges=localChanges.then(async()=>{try{await api.local_action(action);clearViews();then?.();await refresh(true);readAdapt()}catch(e){error=String(e)}finally{changing=false}})}
 // Today's layout depends on this switch: once it is stored, drop every view read before it and read
 // the visible Today again, wherever the user has gone meanwhile. A failed save puts the switch back.
 let savingTodayLayout=$state(false);
 async function saveTodayLayout(on:boolean){
  if(savingTodayLayout)return;
  savingTodayLayout=true;
  try{
   if(!await patchSettings({adaptiveToday:on}))return;
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

 let todayData=$derived(route==='today'||route==='readiness'?data as TodayView|null:null),sleepData=$derived(route==='sleep'?data as SleepView|null:null),trainingData=$derived(route==='training'?data as TrainingView|null:null),workoutData=$derived(route.startsWith('workout/')?data as WorkoutView|null:null),strengthData=$derived(route==='strength'?data as StrengthPage|null:null),healthData=$derived(route==='health'?data as HealthView|null:null),checkinData=$derived(route.startsWith('review/')?data as CheckinView|null:null),boardData=$derived(route==='plan'||route==='plan/commitments'?data as PlanBoard|null:null),progressData=$derived(route==='progress'?data as ProgressPage|null:null),profileData=$derived(route==='profile'||route==='gear'||route==='activities'?data as TrainingView|null:null);
 // Pages between the area cards and the closing summary, in order.
 let checkinExtra=$derived(checkinData?[...(checkinData.timing?['timing']:[]),...(checkinData.intensity?['intensity']:[])]:[]);
 let checkinPages=$derived(checkinData?checkinData.cards.length+checkinExtra.length:0);
 let checkinPage=$derived(checkinData?checkinExtra[checkinIndex-checkinData.cards.length]:undefined);
 let glow=$derived(route==='today' && todayData?.readiness?toneColor(todayData.readiness.tone):route==='sleep' && sleepData?.score?toneColor(sleepData.score.tone):route==='training' && trainingData?.acute?toneColor(trainingData.acute.tone):route==='health' && healthData?.fitnessAge.fitnessAge!=null && healthData.fitnessAge.chronologicalAge!=null?fitnessAgeColor(healthData.fitnessAge.fitnessAge,healthData.fitnessAge.chronologicalAge):null);
</script>
{#snippet promptCard()}{#if prompt}<section class="card prompt-card" aria-label="Plan your week"><b>Plan your week</b><p class="small">{prompt.review?'See how last week went, then choose this week’s sessions.':'Nothing is placed this week yet.'}</p><div class="acts">{#if prompt.review}<button class="primary" onclick={()=>navigate('review/'+prompt!.review)}>Review last week</button>{/if}<button class={prompt.review?'secondary':'primary'} onclick={()=>answerPrompt('planned')}>Plan now</button><button class="secondary" onclick={()=>answerPrompt('dismissed')}>Not this week</button></div><button class="linkish" onclick={()=>void patchSettings({planPrompt:false}).then(ok=>{if(ok){clearViews();void refresh(true)}})}>Don’t offer this again</button></section>{/if}{/snippet}
<div class="phone"><div class="glow" style:--glow={glow?`color-mix(in srgb, ${glow} 12%, transparent)`:'transparent'}></div><main class="scroll" bind:this={scroll} ontouchstart={(e)=>{touching=true;startY=e.touches[0].clientY;startedAtTop=e.currentTarget.scrollTop===0}} ontouchend={pullEnd} ontouchcancel={finishTouch}><div class="page" class:reading={route.startsWith('review/')||route==='onboarding'||route==='plan/commitments'}>
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
 <Header title="Settings" eyebrow="RARING" back={back} backLabel={parentLabel}/><div class="dashboard" {@attach masonry}><Card title="Notifications">{#each (['notifyMorningReadiness','notifyWeeklyCheckin','notifyMetricChanges','notifyVitals','notifyTrainingStatus','notifyBedtime'] as const) as key}<label class="setting"><span>{({notifyMorningReadiness:'Morning readiness',notifyWeeklyCheckin:'Weekly check-in',notifyMetricChanges:'VO₂ max or Fitness Age changed',notifyVitals:'Vitals off',notifyTrainingStatus:'Training status changed',notifyBedtime:'Bedtime'} as Record<string,string>)[key]}</span><input type="checkbox" checked={settings[key]} disabled={settingsBusy[key]} onchange={(e)=>void patchSettings({[key]:e.currentTarget.checked})}/></label>{#if key==='notifyBedtime' && settings.notifyBedtime}<label class="setting lead"><span>Wind-down reminder</span><select value={settings.bedtimeLeadMin} disabled={settingsBusy.bedtimeLeadMin} onchange={(e)=>void patchSettings({bedtimeLeadMin:Number(e.currentTarget.value)})}>{#each [15,30,45,60,90] as lead}<option value={lead}>{lead} min before</option>{/each}</select></label>{/if}{/each}{#if api.supportsStatusSurfaces}<label class="setting"><span>Persistent status<small class="setting-note">Silent readings and today's focus, with the watch's upload time. Uses the existing sync; no extra Garmin requests.</small></span><input type="checkbox" checked={settings.notifyStatus} disabled={savingStatus || settingsBusy.notifyStatus || settingsBusy.statusOfferSeen} onchange={(e)=>{const input=e.currentTarget,on=input.checked;input.checked=settings.notifyStatus;void toggleStatus(on)}}/></label><p class="small">Add Raring widgets from your home screen's widget picker. Updates follow sync; Android may delay them.</p>{/if}</Card><Card title="Today"><label class="setting"><span>Follow my day<small class="setting-note">Lead with this morning, the day so far or tonight's sleep. Turn off for shift work or an irregular schedule.</small></span><input type="checkbox" checked={settings.adaptiveToday} disabled={savingTodayLayout} onchange={(e)=>void saveTodayLayout(e.currentTarget.checked)}/></label></Card><Card title="Plan"><label class="setting"><span>Offer to plan the week<small class="setting-note">At the start of a week, after weeks with goals: review last week and place sessions. Never during wind-down.</small></span><input type="checkbox" checked={settings.planPrompt} disabled={settingsBusy.planPrompt} onchange={e=>void patchSettings({planPrompt:e.currentTarget.checked}).then(ok=>{if(ok)clearViews()})}/></label></Card><Card title="Journal privacy"><label class="setting"><span>Allow Garmin journal reads<small class="setting-note">Off by default. Last seven days only; never in notifications. Turning this off stops new reads. Already downloaded logs remain on this device and in future exports until you erase local data.</small></span><input type="checkbox" checked={settings.journalEnabled} disabled={settingsBusy.journalEnabled} onchange={e=>void toggleJournal(e.currentTarget.checked)}/></label></Card><Card title="Permission"><button class="secondary" onclick={async()=>{try{await api.request_notification_permission()}catch(e){error=String(e)}}}>Enable notifications</button></Card><Card title="Garmin history"><div class="big n">{status?.historyFrom?`Since ${monthLabel(status.historyFrom)}`:'—'}</div><p class="small">{status?.historyComplete?'All of your Garmin history is on this phone.':'Downloading older history in the background.'}</p></Card><Card title="Export"><p class="small">Download a ZIP of your saved Garmin responses, daily CSVs and Raring settings. Sign-in tokens are never included. Previously downloaded journal logs are included even if journal reads are now off; erase local data to remove them. The file leaves your phone only when you share it.</p><button class="secondary" disabled={exporting} onclick={exportData}>{exporting?'Preparing your export…':'Export your data'}</button>{#if exportMessage}<p class="small" role="status">{exportMessage}</p>{/if}</Card><Card title="Account"><button class="signout" onclick={async()=>{if(confirm('Sign out and erase all local Garmin history?')){try{await api.sign_out();clearViews();data=null;localStorage.removeItem('notifications-step-done');onboardStep=0;navigate('onboarding')}catch(e){error=String(e)}}}}>Sign out</button></Card><Card title="Help" onclick={()=>navigate('help')}><p class="small">How Raring reads strength sets, the body map and which watches work.</p></Card><Card title="How Raring works"><AboutRaring/></Card></div>
{:else if route==='help'}
 <Header title="Help" eyebrow="RARING" back={back} backLabel={parentLabel}/><div class="help-list">{#each helpSections as section}<Card title={section.title}>{#each section.pages as page}<button class="help-row" onclick={()=>navigate('help/'+page)}><span class="help-icon"><Icon name={helpContent[page].icon} size={18}/></span><b>{helpContent[page].title}</b><Icon name="chevron" size={16}/></button>{/each}</Card>{/each}<p class="small help-foot">Still stuck? Ask at {GITHUB_QUESTIONS}</p></div>
{:else if route.startsWith('help/') && route.slice(5) in helpContent}
 {@const page=helpContent[route.slice(5) as keyof typeof helpContent]}<Header title={page.title} eyebrow="HELP" back={back} backLabel={parentLabel}/><div class="help-list"><Card><p class="help-intro">{page.intro}</p><ol class="help-steps">{#each page.steps as step,i}<li><span class="n">{i+1}</span>{step}</li>{/each}</ol></Card><p class="small help-foot">Still stuck? Ask at {GITHUB_QUESTIONS}</p></div>
{:else if route==='reviews'}
 <Header title="Weekly reviews" eyebrow="Progress" back={back} backLabel={parentLabel}/>{#if reviewRows}{#each reviewRows as r}<button class="card review-row" onclick={()=>navigate('review/'+r.start)}><span><b>Week of {dateLabel(r.start)}</b><small>{r.counted?`${r.met} of ${r.goals} goals met`:'Not counted'}{r.reasons.length?' · '+r.reasons.join(' · '):''}</small></span><Icon name="chevron" size={16}/></button>{:else}<p class="small">Weeks appear here once you've chosen goals.</p>{/each}{:else}<p class="small">Loading…</p>{/if}
{:else if loading && !hasRendered}<div class="empty"><BrandMark/><h2>{status?.lastSync?'Loading your day…':'Syncing your Garmin data…'}</h2></div>
{:else if !data && hasRendered}<Header title={route[0].toUpperCase()+route.slice(1)} eyebrow="RARING"/><p class="small">{error || (loading?'Preparing this view…':'No data for this view yet.')}</p>
{:else if !data}<div class="empty"><BrandMark/><h2>Syncing your Garmin data…</h2><p>{error && !error.includes('not_ready')?error:'Your day will appear here soon.'}</p><button class="secondary" disabled={syncBusy} onclick={syncNow}>{syncBusy?'Syncing…':'Sync now'}</button>{#if syncError}<p>{syncError}</p>{/if}</div>
{:else if todayData && route==='today'}
 <div class="page-hero today-hero"><Header title="Today" eyebrow={dateLabel(todayData.date)} {...rootControls}/>
 <div class="sync-indicator"><span class="small" aria-live="polite">{#if todayData.watchLastUpload}{watchLabel(todayData.watchLastUpload,now)}{:else}Watch upload time unavailable{/if}{#if syncBusy} · Refreshing…{:else if syncError || status?.lastError} · Raring sync delayed · {syncError || status?.lastError}{/if}</span><button class="sync-button" disabled={syncBusy} onclick={syncNow}>Refresh</button></div>
 <TodayHero today={todayData} {navigate} {act} adapt={()=>adaptOpen=true} plan={()=>addDraft={date:todayData!.date}}/></div><div class="dashboard" {@attach masonry}>
 {#if statusOffer}<Card title="Your day, at a glance"><p>Keep Body Battery, stress and today's focus in a silent notification. Always labeled with the watch's upload time.</p><p class="small">Off unless you choose it. No extra Garmin requests. You can change this in Settings.</p><button class="primary" disabled={savingStatus} onclick={()=>void toggleStatus(true)}>Enable persistent status</button><button class="secondary" disabled={savingStatus} onclick={()=>statusOffer=false}>Not now</button></Card>{/if}
 {#if watchNeedsSync(todayData,now)}<Card title="Watch upload" extra="watch-notice"><p>Open Garmin Connect to sync your watch</p>{#if todayData.watchSleepUploadStale}<p class="small">Last night's sleep hasn't uploaded yet.</p>{/if}{#if connectUnavailable}<p class="small" role="status">Open Garmin Connect on your phone, then sync your watch there.</p>{:else}<button class="watch-open" onclick={openConnect}>Open Garmin Connect <Icon name="arrow-right" size={16}/></button>{/if}</Card>{/if}
 {#if Object.keys(todayData.missingMetricReasons).length}<Card title="About missing readings" extra="watch-notice"><div class="missing-reasons">{#each Object.values(todayData.missingMetricReasons) as reason}<p>{reason}</p>{/each}</div></Card>{/if}
 {#each todayData.focus.cards.filter(c=>c!=='focus') as card,i (card)}
  {#if card==='now'}<NowCard today={todayData} lead={i===0} open={navigate}/>
  {:else if card==='sleepPlan' && todayData.sleepPlan}<SleepPlanCard plan={todayData.sleepPlan} {now} lead={i===0} mode={todayData.focus.planOpen?(({evening:'evening',windDown:'windDown',lateNight:'lateNight'} as const)[todayData.focus.phase as 'evening']??'full'):'line'} open={()=>navigate('sleep')}/>
  {:else if card==='vitals' && todayData.vitals.readings.length}<VitalsMonitor vitals={todayData.vitals} compact open={(kind)=>navigate('metric/'+kind)}/>{/if}
 {/each}
 <PlannedSessionCard planned={todayData.planned} quiet={todayData.focus.phase==='windDown'||todayData.focus.phase==='lateNight'} open={()=>navigate('training')}/>
 {#if weekBoard && !['windDown','lateNight'].includes(todayData.focus.phase)}<Card title="This week" onclick={()=>navigate('plan')} extra="week-card">{#if weekBoard.ledger.length}<WeekSummary board={weekBoard} compact onGoals={()=>navigate('plan/commitments')}/>{:else}<p class="small">Recorded activities show here. Weekly goals are optional.</p>{/if}<div class="week-cells"><DayCells days={weekBoard.days} select={()=>navigate('plan')} size={30}/></div></Card>{/if}

 {#if todayData.bodyBattery.now!=null || todayData.hrv.lastNight!=null}<div class="row">{#if todayData.bodyBattery.now!=null}<Card title="Body Battery" onclick={()=>navigate('metric/bodyBattery')}><div class="bb"><div class="battery"><i style:height={Math.max(0,Math.min(100,todayData.bodyBattery.now))+'%'} style:background={batteryColor(todayData.bodyBattery.now)}></i></div><div><div class="big n" style:color={batteryColor(todayData.bodyBattery.now)}>{num(todayData.bodyBattery.now)}</div>{#if todayData.bodyBattery.charged!=null}<div class="small"><b>+{num(todayData.bodyBattery.charged)}</b> charged today</div>{/if}</div></div></Card>{/if}{#if todayData.hrv.lastNight!=null}<Card title="HRV" onclick={()=>navigate('metric/hrv')}>{#snippet aside()}{#if todayData.hrv.status}<span class="pill" style:color={toneColor(todayData.hrv.tone)}>{todayData.hrv.status}</span>{/if}{/snippet}<div class="big n">{num(todayData.hrv.lastNight)}<span class="unit">ms</span></div><div class="spark-space"><Sparkline values={todayData.hrv.week} band={todayData.hrv.baseline} w={138} h={34} color={toneColor(todayData.hrv.tone)}/></div></Card>{/if}</div>{/if}
 {#if todayData.stress.average!=null || todayData.stress.timeline.length}{@const stress=todayData.stress}<Card title="Stress" onclick={()=>navigate('metric/stress')}>{#snippet aside()}{#if stress.qualifier}<span class="pill" style:color={toneColor(stress.tone)}>{stress.qualifier}</span>{/if}{/snippet}<div class="stress-summary"><div><div class="big n" style:color={stress.level?stressColors[stress.level]:undefined}>{num(stress.average)}</div><div class="small">{stress.level?stressNames[stress.level]+' · ':''}average</div></div>{#if stress.timeline.length}<StressTimeline {stress} compact/>{/if}</div></Card>{/if}
 {#if todayData.vitals.readings.length && !todayData.focus.cards.includes('vitals')}<VitalsMonitor vitals={todayData.vitals} compact open={(kind)=>navigate('metric/'+kind)}/>{/if}
 {#if todayData.activities.length}<Card title="Activities">{#each todayData.activities as a}<ActivityRow activity={a} open={()=>navigate('workout/'+a.id)}/>{#if a.aerobicTe!=null || a.benefit}<div class="tags">{#if a.aerobicTe!=null}<span class="pill">Aerobic {a.aerobicTe} · {a.aerobicLabel}</span>{/if}{#if a.benefit}<span class="pill">{a.benefit}</span>{/if}</div>{/if}{/each}</Card>{/if}
 {#if todayData.checkinReady}<Card title="Weekly check-in" onclick={()=>navigate('review/latest')}><div class="checkin-tease"><span>See your week at a glance</span><b><Icon name="arrow-right" size={18}/></b></div></Card>{/if}
</div>
{:else if route==='readiness' && todayData}
 {#snippet readinessHeader()}<Header title="Readiness" eyebrow={dateLabel(todayData.date)} back={back} backLabel="Today"/>{/snippet}<ReadinessDetail {todayData} header={readinessHeader} bind:selectedFactor/>
{:else if sleepData}
 {#snippet sleepHeader()}<Header title="Sleep" eyebrow={sleepData.startLocal?`${clock(sleepData.startLocal)} – ${clock(sleepData.endLocal)}`:dateLabel(sleepData.date)} {...rootControls}/>{/snippet}<SleepDetail {sleepData} header={sleepHeader}/>
{:else if trainingData}
 {#snippet trainingHeader()}<Header title="Training" eyebrow="Last 7 days" {...rootControls}/>{/snippet}<TrainingDetail {trainingData} header={trainingHeader} {navigate}/>
{:else if workoutData}
 {@const sets=workoutData.strength}<Header title={workoutData.summary.name} eyebrow={(sets?'Strength · ':'')+dateLabel(workoutData.summary.startLocal.slice(0,10))+' · '+clock(workoutData.summary.startLocal)} back={back} backLabel={parentLabel} onHelp={sets?()=>sheet=true:undefined}/><WorkoutDetail {workoutData} {navigate} help={()=>sheet=true}/>
{:else if strengthData}
 <Header title="Strength" eyebrow="Last 12 weeks" back={back} backLabel={parentLabel} onHelp={()=>navigate('help')}/>{#if strengthData.strengthPage}<StrengthHistory view={strengthData.strengthPage} {navigate}/>{:else}<Card title="No sets yet"><p class="small untracked">Start a Strength activity on your watch and press lap after each set. Your sets appear here after Garmin Connect syncs.</p><button class="watch-open" onclick={()=>navigate('help/strength-start')}>Lift with your watch <Icon name="arrow-right" size={16}/></button></Card>{/if}
{:else if healthData}
 <Header title="Health" eyebrow={dateLabel(healthData.date)} {...rootControls}/><HealthDetail {healthData} {navigate} {toggleJournal} journalBusy={settingsBusy.journalEnabled} bind:journalOpen bind:journalDayOpen/>
{:else if boardData && route==='plan'}
 {#snippet planHeader()}<Header title="Plan" eyebrow="This week" {...rootControls}/>{/snippet}<PlanTab board={boardData} {navigate} {act} header={planHeader} openAdd={d=>addDraft=d as Draft} openItem={p=>itemSheet=p} openEvent={()=>eventSheet=true}/>
{:else if boardData && route==='plan/commitments'}
 <Header title="Weekly goals" eyebrow="Plan" back={back} backLabel={parentLabel}/><GoalEditor initial={boardData.goals?.targets??null} garminGoal={boardData.garminIntensityGoal} busy={changing} save={targets=>act({action:'setGoals',targets},back)}/>
{:else if progressData}
 {#snippet progressHeader()}<Header title="Progress" eyebrow="Trends and reviews" {...rootControls}/>{/snippet}<ProgressTab initial={progressData.progress} health={progressData.health} {navigate} header={progressHeader}/>
{:else if profileData && route==='profile'}
 {#snippet profileHeader()}<Header title="Profile" eyebrow="RARING" back={back} backLabel={parentLabel}/>{/snippet}<ProfileScreen {status} training={profileData} {navigate} header={profileHeader}/>
{:else if profileData && route==='gear'}
 <Header title="Gear" eyebrow="Garmin" back={back} backLabel={parentLabel}/>{#if profileData.gear}<GearCard gear={profileData.gear}/>{:else}<p class="small">No gear list from Garmin yet.</p>{/if}
{:else if profileData && route==='activities'}
 <Header title="Activities" eyebrow="Recorded" back={back} backLabel={parentLabel}/><Card title="Recent activities">{#each profileData.workouts as a}<ActivityRow activity={a} open={()=>navigate('workout/'+a.id)}/>{:else}<p class="small">No recorded activities yet.</p>{/each}</Card>
{:else if route.startsWith('metric/')}
 <Header title={route.split('/')[1]==='restingHr' && (data as HealthView).vitals?.readings.some(v=>v.kind==='restingHr' && (v.status || v.rangeSource==='garmin')) ? 'Sleep HR' : metricNames[route.split('/')[1]]} eyebrow="GARMIN METRIC" back={back} backLabel={parentLabel}/><MetricDetail metric={route.split('/')[1]} data={data as TodayView|HealthView|TrainingView}/>
{:else if checkinData}
 <Header title="Weekly review" eyebrow={'Week of '+dateLabel(checkinData.weekStart)} back={back} backLabel={parentLabel}/><div class="checkin-progress">{#each Array(checkinPages+1) as _,i}<span class:on={i===checkinIndex}></span>{/each}</div><div class="checkin-swiper" role="group" aria-label="Weekly check-in cards" ontouchstart={(e)=>swipeX=e.touches[0].clientX} ontouchend={(e)=>{let dx=e.changedTouches[0].clientX-swipeX;if(Math.abs(dx)>50)checkinIndex=Math.max(0,Math.min(checkinPages,checkinIndex+(dx<0?1:-1)))}}>{#if checkinIndex<checkinData.cards.length}{@const c=checkinData.cards[checkinIndex]}<Card title={c.area}><div class="checkin-content"><div class="eyebrow">YOUR WEEK IN {c.area.toUpperCase()}</div>{#if c.value!=null}<Gauge size={210} stroke={15} value={c.value} max={c.area==='training'?1000:c.area==='health'?50:100} color={toneColor(c.tone)} caption={c.unit}/>{/if}<h2>{c.headline}</h2><span class="pill" class:ok={c.previous!=null && c.value!=null && (c.area==='health'?c.value<=c.previous:c.value>=c.previous)}>{#if c.previous!=null && c.value!=null && Math.abs(c.value-c.previous)>=1}<Icon name={c.value<c.previous?'trend-down':'trend-up'} size={14}/>{signed(c.value-c.previous)} vs last week{:else if c.value==null}No readings this week{:else if c.previous!=null}Steady vs last week{:else}No previous week to compare{/if}</span><Sparkline values={c.series} w={240} h={60} color={colors.ink}/></div></Card>{:else if checkinPage==='timing' && checkinData.timing}{@const t=checkinData.timing}<Card title="Timing" extra="timing-review"><div class="checkin-content"><div class="eyebrow">YOUR WEEK IN SLEEP TIMING</div><ConsistencyChart nights={t.nights} date={addDays(checkinData.weekStart,6)} days={7}/><div class="timing-ranges">{#if t.bedRange}<div><span>Bedtimes</span><b class="n">{t.bedRange[0]}–{t.bedRange[1]}</b></div>{/if}{#if t.wakeRange}<div><span>Wake times</span><b class="n">{t.wakeRange[0]}–{t.wakeRange[1]}</b></div>{/if}</div>{#if t.aligned+t.ahead+t.behind>0}<span class="pill">Garmin: {[[t.aligned,'aligned'],[t.behind,'behind'],[t.ahead,'ahead']].filter(([n])=>Number(n)>0).map(([n,w])=>`${n} ${w}`).join(' · ')}</span>{/if}<h3>THE TIMING FOCUS</h3><p>{t.focus}</p></div></Card>{:else if checkinPage==='intensity' && checkinData.intensity}{@const im=checkinData.intensity}<Card title="Intensity" extra="intensity-review"><div class="checkin-content"><div class="eyebrow">YOUR WEEK IN INTENSITY MINUTES</div><Gauge size={210} stroke={15} value={im.total} max={im.goal??Math.max(im.total,1)} display={num(im.total)} caption={im.goal!=null?`of ${num(im.goal)} min`:'min'} color={im.goal!=null&&im.total>=im.goal?colors.good:colors.ink}/><h2>{im.goal==null?'Intensity minutes':im.total>=im.goal?"Garmin's goal reached":`${num(Math.ceil(im.goal-im.total))} min short`}</h2><span class="pill">{num(im.moderate)} moderate · {num(im.vigorous)} vigorous, counted twice</span></div></Card>{:else}<Card title="Your week ahead"><div class="checkin-summary"><div class="brand-mark"><Icon name="sparkle" size={70}/></div><h2>Keep the momentum</h2>{#if checkinData.vo2maxChange!=null && Math.abs(checkinData.vo2maxChange)>=1}<span class="pill ok">VO₂ max {signed(checkinData.vo2maxChange)}</span>{/if}{#if checkinData.fitnessAgeChange!=null && Math.abs(checkinData.fitnessAgeChange)>=1}<span class="pill ok">Fitness Age {signed(checkinData.fitnessAgeChange)}</span>{/if}{#if checkinData.win}<h3>THE WIN</h3><p>{checkinData.win}</p>{/if}{#if checkinData.focus}<h3>THE FOCUS</h3><p>{checkinData.focus}</p>{/if}</div></Card>{/if}</div><div class="checkin-controls"><button disabled={checkinIndex===0} onclick={()=>checkinIndex--}><Icon name="arrow-left" size={16}/> Previous</button><span>{checkinIndex+1} / {checkinPages+1}</span><button disabled={checkinIndex===checkinPages} onclick={()=>checkinIndex++}>Next <Icon name="arrow-right" size={16}/></button></div>{#if reviewData}<WeekReviewCard review={reviewData} {act} {navigate} busy={changing}/>{/if}
{/if}
</div></main>{#if sheet && workoutData?.strength}<HelpSheet close={()=>sheet=false} more={()=>{sheet=false;navigate('help/strength-sets')}}/>{/if}{#if addDraft && (boardData ?? weekBoard)}<AddSessionSheet board={(boardData ?? weekBoard)!} draft={addDraft} close={()=>addDraft=null} {act} busy={changing}/>{/if}{#if itemSheet && (boardData ?? weekBoard)}<ItemSheet board={(boardData ?? weekBoard)!} item={itemSheet} close={()=>itemSheet=null} {act} {navigate}/>{/if}{#if eventSheet}<EventSheet today={(boardData ?? weekBoard)?.today ?? new Date().toISOString().slice(0,10)} close={()=>eventSheet=false} {act}/>{/if}{#if adaptOpen}<AdaptSheet current={adaptState} close={()=>adaptOpen=false} {act} busy={changing}/>{/if}{#if search}<SearchSheet close={()=>search=false} {navigate}/>{/if}{#if route!=='onboarding'}<TabBar active={activeTab} {navigate}/>{/if}</div>
