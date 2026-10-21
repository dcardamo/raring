<script lang="ts">
 import {onMount,tick,setContext} from 'svelte';
 import GarminChanges from './lib/components/GarminChanges.svelte';
 import type {GarminChange} from './lib/generated/GarminChange';
 import {isTauri} from '@tauri-apps/api/core';
 import {createNavigation,datedDetail,knownRoute,tabs} from './lib/navigation';
 import {shouldSync,CACHE_POLL_MS} from './lib/freshness';
 import {watchLabel,watchNeedsSync} from './lib/watch';
 import {masonry} from './lib/masonry';
 import {addDays} from './lib/history';
 import {metricSource,metricNames} from './lib/metrics';
 import {isSleepHr} from './lib/signals';
 import MetricDetail from './lib/components/MetricDetail.svelte';
 import HistoryChart from './lib/components/HistoryChart.svelte';
 import TodayHero from './lib/components/TodayHero.svelte';
 import AdaptSheet from './lib/components/AdaptSheet.svelte';
 import ChooseSheet from './lib/components/ChooseSheet.svelte';
 import GarminSheet from './lib/components/GarminSheet.svelte';
 import type {GarminSession} from './lib/generated/GarminSession';
 import YourDay from './lib/components/YourDay.svelte';
 import SleepTonight from './lib/components/SleepTonight.svelte';
 import QuickHelpSheet from './lib/components/QuickHelpSheet.svelte';
 import {quickHelp} from './lib/quickHelp.svelte';
 import TrainingTab from './lib/screens/TrainingTab.svelte';
 import type {AdaptState} from './lib/generated/AdaptState';
 import type {ProfileView} from './lib/generated/ProfileView';
 import type {Milestone} from './lib/generated/Milestone';
 import AboutRaring from './lib/components/AboutRaring.svelte';
 import SleepPlanCard from './lib/components/SleepPlanCard.svelte';
 import {phaseEnded} from './lib/tonight';
 import AlignmentCard from './lib/components/AlignmentCard.svelte';
 import WorkoutSplits from './lib/components/WorkoutSplits.svelte';
 import VitalsMonitor from './lib/components/VitalsMonitor.svelte';
 import {GOALS,SPORT_NAMES,amount,backlogDraft} from './lib/goals';
 import StressTimeline from './lib/components/StressTimeline.svelte';
 import PlanTab from './lib/screens/PlanTab.svelte';
 import ProgressTab from './lib/screens/ProgressTab.svelte';
 import ProfileScreen from './lib/screens/ProfileScreen.svelte';
 import GoalEditor from './lib/components/GoalEditor.svelte';
 import SearchSheet from './lib/components/SearchSheet.svelte';
 import DayCells from './lib/components/DayCells.svelte';
 import WeekSummary from './lib/components/WeekSummary.svelte';
 import WeightCard from './lib/components/WeightCard.svelte';
 import type {WeightView} from './lib/generated/WeightView';
 let weightRange=$state<TrendRange>('days30');
 import AddSessionSheet,{type Draft} from './lib/components/AddSessionSheet.svelte';
 import ItemSheet from './lib/components/ItemSheet.svelte';
 import EventSheet from './lib/components/EventSheet.svelte';
 import CheckinPage from './lib/components/CheckinPage.svelte';
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
 import WorkoutDetail from './lib/screens/WorkoutDetail.svelte';
 import HealthDetail from './lib/screens/HealthDetail.svelte';
 import {help as helpContent,helpSections,GITHUB_QUESTIONS} from './lib/help';
 import {importRowText,importSummary,importDone} from './lib/import';
 import type {StrengthView} from './lib/generated/StrengthView';
 import type {FigureChoice} from './lib/generated/FigureChoice';
 import type {TrendRange} from './lib/generated/TrendRange';
 import WhySheet from './lib/components/WhySheet.svelte';
 import Reconcile from './lib/components/Reconcile.svelte';
 import {closeSheet} from './lib/sheet';
 import * as api from './lib/api';
 import type {TodayView} from './lib/generated/TodayView';import type {SleepView} from './lib/generated/SleepView';import type {SleepPlan} from './lib/generated/SleepPlan';import type {TrainingView} from './lib/generated/TrainingView';import type {WorkoutView} from './lib/generated/WorkoutView';import type {HealthView} from './lib/generated/HealthView';import type {CheckinView} from './lib/generated/CheckinView';import type {AppStatus} from './lib/generated/AppStatus';
 import Icon from './lib/components/Icon.svelte';import BrandMark from './lib/components/BrandMark.svelte';import ReadinessTrend from './lib/components/ReadinessTrend.svelte';
 import RangeBar from './lib/components/RangeBar.svelte';import ZoneStrip from './lib/components/ZoneStrip.svelte';import Hypnogram from './lib/components/Hypnogram.svelte';import WeekBars from './lib/components/WeekBars.svelte';import LoadFocus from './lib/components/LoadFocus.svelte';import RouteMap from './lib/components/RouteMap.svelte';import HrChart from './lib/components/HrChart.svelte';import Card from './lib/components/Card.svelte';import Chip from './lib/components/Chip.svelte';import Header from './lib/components/Header.svelte';import TabBar from './lib/components/TabBar.svelte';import ActivityRow from './lib/components/ActivityRow.svelte';
 import {colors,toneColor,batteryColor,stressColors,stressNames} from './lib/charts';import {duration,elapsed,pace,clock,dateLabel,monthLabel,signed,num,minuteClock} from './lib/format';
 type StrengthPage={strengthPage:StrengthView|null};
 type ProgressView=import('./lib/generated/ProgressView').ProgressView;
 type ProgressPage={progress:ProgressView};
 let progressQuery:{period:import('./lib/generated/Period').Period,date?:string}={period:'month'};
 // Progress's Cardio, Strength and Long-term windows, kept like its period for the session.
 type DomainQuery=import('./lib/generated/DomainQuery').DomainQuery;
 const FIRST_DOMAINS:DomainQuery={weeks:12,weeksEnd:null,months:12,monthsEnd:null};
 let progressRevision=0;
 type PageData=TodayView|SleepView|TrainingView|WorkoutView|HealthView|CheckinView|PlanBoard|StrengthPage|ProgressPage|WeightView|ProfileView;
 let sheet=$state(false),search=$state(false),addDraft=$state<Draft|null>(null),itemSheet=$state<PlanItem|null>(null),eventSheet=$state(false),reviewData=$state<WeekReview|null>(null),reviewRows=$state<ReviewRow[]|null>(null),milestoneRows=$state<Milestone[]|null>(null),garminChanges=$state<GarminChange[]|null>(null),garminChangesFailed=$state(false),lastPhase=$state<string|null>(null),adaptOpen=$state(false),invalidLink=$state<string|null>(null),trainingSection=$state<'fitness'|'strength'|'activities'>('fitness'),liftPicked=$state<string|null>(null),liftMetric=$state<'max'|'volume'>('max'),liftRange=$state<TrendRange>('days90'),muscleRange=$state<TrendRange>('days30'),activityDay=$state<string|null>(null),progressSection=$state<'overall'|'cardio'|'strength'|'long'>('overall'),progressDomains=$state<DomainQuery>({...FIRST_DOMAINS}),adaptLimits=$state(false),chooseOpen=$state(false),garminSheet=$state<GarminSession|null>(null),adaptState=$state<AdaptState|null>(null),weekBoard=$state<PlanBoard|null>(null),statusOffer=$state(false),savingStatus=$state(false);
 let checkingStatusOffer=false,settingsRevision=0;
 let settingsBusy=$state<Partial<Record<keyof api.Settings,boolean>>>({});
 function canApplySettingsRead(revision:number){return revision===settingsRevision && !Object.values(settingsBusy).some(Boolean) && !savingStatus}
 const FALL_ASLEEP=Array.from({length:13},(_,i)=>i*5);
 // Tonight's plan reads these, so every view that shows it is read again.
 async function patchPlan(patch:api.SettingsPatch){if(await patchSettings(patch)){clearViews();void refresh(true)}}
 async function patchSettings(patch:api.SettingsPatch):Promise<boolean>{
  const keys=Object.keys(patch) as (keyof api.Settings)[];
  if(keys.some(key=>settingsBusy[key]))return false;
  const previous=Object.fromEntries(keys.map(key=>[key,settings[key]])) as api.SettingsPatch;
  settingsBusy={...settingsBusy,...Object.fromEntries(keys.map(key=>[key,true]))};
  settingsRevision++;
  settings={...settings,...patch};
  try{await api.patch_settings(patch);if(api.desktopReminders && keys.some(key=>key.startsWith('notify') || key==='bedtimeLeadMin' || key==='fallAsleepMin')){void api.check_reminders();void readReminderStatus()}return true}
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
 // A declined permission is remembered so Raring doesn't ask again; Android Settings can still allow it.
 let notificationsDeclined=$state(localStorage.getItem('notifications-declined')==='1');
 function markDeclined(declined:boolean){notificationsDeclined=declined;if(declined)localStorage.setItem('notifications-declined','1');else localStorage.removeItem('notifications-declined')}
 // Desktop reminders are shown by the app itself while it runs; Settings says why one could not be.
 let reminderStatus=$state<api.ReminderStatus|null>(null);
 async function readReminderStatus(){
  if(!api.desktopReminders)return;
  try{reminderStatus=await api.reminders_status()}catch{reminderStatus=null}
 }
 function reminderNote(){
  const s=reminderStatus;
  if(!s)return 'Reminders appear while Raring is open. Your system’s notification settings can still hide them.';
  if(!s.supported)return 'This build can’t show notifications, so reminders stay unmarked.';
  if(!s.granted)return 'Notifications are blocked for Raring in your system settings, so reminders stay unmarked until you allow them.';
  if(s.error)return `Raring couldn’t show a reminder (${s.error}). It stays unmarked and Raring tries again.`;
  return s.nextAt?`Reminders appear while Raring is open · next wind-down ${clock(s.nextAt)}.`:`Reminders appear while Raring is open. Your system’s notification settings can still hide them.`;
 }
 async function askNotifications():Promise<boolean>{
  try{const granted=await api.request_notification_permission();markDeclined(!granted);if(notificationsAllowed!==null)notificationsAllowed=granted;return true}catch(e){error=String(e);return false}
 }
 // Read from Android each time Settings opens; null where the platform has no such permission.
 let notificationsAllowed=$state<boolean|null>(null);
 const NOTIFICATIONS=[
  ['notifyMorningReadiness','Morning readiness','After last night’s sleep and readiness sync'],
  ['notifyWeeklyCheckin','Weekly review','Monday 08:00'],
  ['notifyMetricChanges','VO₂ max or Fitness Age changed','When Garmin’s estimate moves either way'],
  ['notifyVitals','Vitals outside your ranges','When two or more are outside last night'],
  ['notifyTrainingStatus','Training status changed','When Garmin’s status changes after a sync'],
  ['notifyBedtime','Wind-down reminder','At the start of tonight’s wind-down'],
 ] as const;
 let confirmSignOut=$state(false);
 async function signOut(){
  try{await api.sign_out();confirmSignOut=false;clearSessionViews();data=null;localStorage.removeItem('notifications-step-done');onboardStep=0;navigate('onboarding')}catch(e){error=String(e)}
 }
 async function toggleStatus(enabled:boolean){
  if(savingStatus || settingsBusy.notifyStatus || settingsBusy.statusOfferSeen)return;
  savingStatus=true;
  settingsRevision++;
  try{
   const granted=!enabled || await api.request_notification_permission();
   if(enabled)markDeclined(!granted);
   if(enabled && notificationsAllowed!==null)notificationsAllowed=granted;
   if(!granted){
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
 let importing=$state(false),importPreview=$state<api.ImportPreview|null>(null),importMessage=$state('');
 let route=$state('today'),trail=$state<string[]>(['today']),data=$state<PageData|null>(null),status=$state<AppStatus|null>(null),settings=$state<api.Settings>({...api.defaultSettings}),loading=$state(true),error=$state(''),syncError=$state(''),syncing=$state(false),selectedFactor=$state(''),onboardStep=$state(0),startY=0,startedAtTop=false;
 let navigation:ReturnType<typeof createNavigation>,scroll:HTMLElement;
 let requestId=0,statusReadId=0,appliedStatusReadId=0,lastSyncAttempt:number|null=null,prefetched=false;
 let cacheGeneration=$state(0);
 setContext('trend-generation',()=>cacheGeneration);
 let hasRendered=$state(false);
 let touching=false,pendingSwap:{route:string;data:PageData}|null=null;
 const views=new Map<string,PageData>();
 const pendingViews=new Map<string,Promise<PageData>>();
 // How long a tap keeps the shown page while the next is read; past it the next opens empty and fills in.
 const HOLD_MS=200;
 let cacheDay=new Date().toDateString();
 // An explicit open asks Garmin once per page; retry clears the record for that page.
 let openedAttempts=new Set<string>(),detailState=$state<{kind:'loading'|'failed';message:string}|null>(null);
 // A sync keeps cached views, which still open at once and are read again as they open. A local change, a new
 // day or sign-out drops them: they would show the page before the change, another day or another account.
 function clearViews(keepCached=false){if(!keepCached)views.clear();pendingViews.clear();pendingSwap=null;prefetched=false;cacheGeneration++;openedAttempts.clear();detailState=null}
 const scrollPositions=new Map<string,number>();
 function checkDay(){
  const day=new Date().toDateString();
  if(day!==cacheDay){clearViews();cacheDay=day}
 }
 // These pages read their tab's view, so the prefetched tab serves them: Sleep's Last night and Tonight,
 // Readiness and Today's metrics, Gear and Training's metrics, Health's metrics, and the weekly goals.
 function viewKey(route:string){
  if(route==='sleep/last'||route==='sleep/tonight')return 'sleep';
  if(route==='readiness')return 'today';
  if(route==='gear')return 'training';
  if(route==='plan/commitments')return 'plan';
  const metric=/^metric\/(\w+)$/.exec(route);
  return metric?metricSource(metric[1]):route;
 }
 // Settings, Help and the review and milestone lists load their own parts rather than one view.
 const readsView=(route:string)=>!['onboarding','settings','help','reviews','progress/milestones'].includes(route) && !route.startsWith('help/');
 function cached(current:string){
  checkDay();
  const view=views.get(viewKey(current))??null;
  // A Today whose phase has ended would swap its leading card right after showing.
  if(current==='today' && view && phaseEnded(view as TodayView)){views.delete(current);return null}
  return view;
 }
 async function readView(route:string):Promise<PageData>{
  const current=viewKey(route);
  for(;;){
   checkDay();
   const generation=cacheGeneration,selection=progressRevision;
   const currentRead=()=>generation===cacheGeneration && (current!=='progress'||selection===progressRevision);
   let read=pendingViews.get(current);
   if(!read){
    read=loadView(current).then(next=>{if(currentRead())views.set(current,next);return next})
     .finally(()=>{if(pendingViews.get(current)===read)pendingViews.delete(current)});
    pendingViews.set(current,read);
   }
   try{
    const next=await read;
    checkDay();
    if(currentRead())return next;
   }catch(e){
    if(currentRead())throw e;
   }
  }
 }
 function prefetchTabs(){
  if(prefetched)return;
  prefetched=true;
  const generation=cacheGeneration;
  void tick().then(()=>{if(generation===cacheGeneration)return Promise.allSettled([...tabs,'sleep'].filter(tab=>viewKey(tab)!==viewKey(route)).map(tab=>readView(tab)))});
 }
 const fixture=new URLSearchParams(location.search).get('fixture');
 const live=isTauri() && fixture===null;
 let syncBusy=$derived(syncing || status?.syncing===true);
 // The plan-your-week prompt sits under every root header, never in wind-down or late night.
 let prompt=$derived(weekBoard?.prompt && !['windDown','lateNight'].includes(lastPhase??'')?weekBoard.prompt:null);
 let rootControls=$derived({onSearch:()=>search=true,onProfile:()=>navigate('profile'),onAdapt:()=>adaptOpen=true,adapt:adaptState?.label,below:prompt?promptCard:undefined});
 function answerPrompt(choice:'planned'|'dismissed'){if(weekBoard)act({action:'setPromptWeek',week:weekBoard.start,choice},choice==='planned'?()=>navigate('plan'):undefined)}
 // Every root header shows today's circumstances; they come with the live Today.
 // It also brings Today's phase, which gates the plan prompt on every page.
 function readAdapt(){void readView('today').then(v=>{adaptState=(v as TodayView).adapt??null;lastPhase=(v as TodayView).focus.phase}).catch(()=>{})}
 let activeTab=$derived([...trail].reverse().find(r=>(tabs as readonly string[]).includes(r))??'today');
 let parentLabel=$derived.by(()=>{const parent=trail.at(-2);if(!parent)return 'Today';if(parent.startsWith('workout/'))return 'Workout';if(parent.startsWith('health/')&&parent!=='health/weight')return 'Health';if(parent.startsWith('help/'))return 'Help';return ({today:'Today',sleep:'Sleep',training:'Training',health:'Health',readiness:'Readiness',plan:'Plan',progress:'Progress',profile:'Profile','plan/commitments':'Goals',strength:'Strength',help:'Help',settings:'Settings'} as Record<string,string>)[parent]??'Back'});
 // Keep Progress's selection with its cached page, not a component that is
 // destroyed on a detail visit. Older reads may not replace a newer selection.
 function saveProgress(progress:ProgressView){
  if(route!=='progress' || !data || !('progress' in data))return;
  progressQuery={period:progress.period,date:progress.current?undefined:progress.end};
  progressRevision++;requestId++;
  pendingViews.delete('progress');
  const next:ProgressPage={progress};
  views.set('progress',next);data=next;pendingSwap=null;loading=false;
 }
 function clearSessionViews(){progressQuery={period:'month'};progressRevision++;progressDomains={...FIRST_DOMAINS};weightRange='days30';clearViews()}
 function navigate(to:string){navigation.navigate(to)}
 function back(){return closeSheet() || navigation.back()}
 function openedDetail(current:string){return datedDetail(current) ?? (/^workout\/\d+$/.test(current)?{page:'workout',date:current.slice(8)}:null)}
 function detailKey(opened:{page:string,date:string}){return (opened.page==='workout'?'workout:':'day:')+opened.date}
 function detailMissing(view:PageData){return 'detail' in view && view.detail.missing.length>0}
 // Only a page whose route names its own day or workout fetches. Prefetched tabs never do.
 async function fetchOpenedDetail(current:string){
  const opened=openedDetail(current);
  // The dated heart-rate page reads only its heart-rate day, inside MetricDetail.
  if(!opened || opened.page==='heartRate')return;
  const key=detailKey(opened);
  if(openedAttempts.has(key))return;
  openedAttempts.add(key);
  detailState={kind:'loading',message:''};
  try{
   if(opened.page==='workout')await api.fetch_workout_detail(opened.date);
   else await api.fetch_day_detail(opened.date,opened.page==='sleep'?'sleep':'day');
   const next=await loadView(current);
   if(route===current){data=next;views.set(current,next);pendingSwap=null}
   if(detailMissing(next))detailState={kind:'failed',message:'Garmin didn’t return this detail. Try again later.'};
   else detailState=null;
  }catch(e){detailState={kind:'failed',message:String(e)}}
  if(route!==current)detailState=null;
 }
 function retryOpenedDetail(){const opened=openedDetail(route);if(opened){openedAttempts.delete(detailKey(opened));detailState=null;void fetchOpenedDetail(route)}}
 async function loadView(current:string):Promise<PageData>{
  const opened=datedDetail(current);
  // A dated page reads only its own day; a metric's dated page reads the day's Health view.
  if(opened)return opened.page==='sleep'?api.sleep(opened.date):api.health(opened.date);
  const [key,id]=current.split('/');
  switch(key){
   case 'sleep':case 'sleep/last':case 'sleep/tonight':return api.sleep();
   case 'training':return api.training();
   case 'health':return id==='weight'?api.weight_trend(weightRange):api.health();
   case 'workout':return api.workout(id||'');
   case 'strength':return api.strength().then(strengthPage=>({strengthPage}));
   // The check-in is keyed by the Monday after its week; a review route names the week's start.
   case 'review':return api.checkin(id==='latest'?undefined:addDays(id,7));
   case 'plan':return api.plan_board();
   case 'progress':return {progress:await api.progress(progressQuery.period,progressQuery.date)};
   case 'profile':return api.profile();
   case 'gear':return api.training();
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
     if(changedSync)clearViews(true);
     status=nextStatus;
     checkDay();
     if(live){
      if(!status.signedIn){onboardStep=0;clearSessionViews();data=null;if(route!=='onboarding')navigate('onboarding');return}
      if(localStorage.getItem('notifications-step-done')!=='1'){onboardStep=1;if(route!=='onboarding')navigate('onboarding');return}
      if(route==='onboarding'){navigate('today');return}
     }
     if(changedSync && request!==requestId)void refresh(true);
    }
    if(request!==requestId)return;
   }
   if(current==='settings'){
    garminChanges=null;garminChangesFailed=false;confirmSignOut=false;
    void api.notification_permission().then(v=>{if(request===requestId)notificationsAllowed=v}).catch(()=>{});
    void api.garmin_changes().then(r=>{if(request===requestId)garminChanges=r}).catch(()=>{if(request===requestId)garminChangesFailed=true});
    const revision=settingsRevision;
    const nextSettings=await api.get_settings();
    if(request===requestId && canApplySettingsRead(revision))settings=nextSettings;
   }else if(current==='reviews'){
    reviewRows=null;void api.reviews().then(r=>{if(request===requestId)reviewRows=r}).catch(e=>error=String(e));
   }else if(current==='progress/milestones'){
    milestoneRows=null;void api.milestone_archive().then(r=>{if(request===requestId)milestoneRows=r}).catch(e=>error=String(e));
   }else if(readsView(current)){
    let next=await readView(current);
    if(request!==requestId)return;
    if(fixture==='missing' && current==='today'){
     next={...(next as TodayView),readiness:null,readinessTrend:[],sleep:null,sleepPlan:null,load:null,factors:[],readinessWhy:null,bodyBatteryWhy:null,reconcile:null,call:null,callNote:null,bodyBattery:{now:null,charged:null,drained:null,timeline:[]},stress:{average:null,max:null,level:null,qualifier:null,tone:'neutral',timeline:[],active:[],sleep:null,time:[],activeS:null,trend:[]},hrv:{lastNight:null,status:null,tone:'neutral',baseline:null,week:[]},vitals:{readings:[],offCount:0,baselineReady:false},activities:[],fitnessAge:{fitnessAge:null,chronologicalAge:null,trend:[],change12m:null,recordedOn:null},headline:'',focus:{...(next as TodayView).focus,phase:'wake',cards:['focus'],effort:false,sleepMissing:true,lastNight:null}};
    }
    if(fixture==='stale' && current==='today')next={...(next as TodayView),watchLastUpload:new Date(Date.now()-7*3600_000).toISOString(),missingMetricReasons:{skinTemp:"Your watch doesn't record skin temperature",readiness:"Your watch doesn't record training readiness"}};
    if(fixture==='fresh' && current==='today')next={...(next as TodayView),watchLastUpload:new Date(Date.now()-2*3600_000-120_000).toISOString()};
    if(fixture==='stale-sleep' && current==='today')next={...(next as TodayView),watchLastUpload:null,watchSleepUploadStale:true,missingMetricReasons:{}};
    const updated=fixture==='empty' && current==='today'?null:next;
    if(touching && data && updated)pendingSwap={route:current,data:updated};
    else data=updated;
    if(data){hasRendered=true;prefetchTabs()}
    if(updated && detailMissing(updated))void fetchOpenedDetail(current);
    if(current==='today' && updated){adaptState=(updated as TodayView).adapt??adaptState;lastPhase=(updated as TodayView).focus.phase}
    if(current==='plan' && updated)weekBoard=updated as PlanBoard;
    else if((tabs as readonly string[]).includes(current)||current.startsWith('sleep'))void readView('plan').then(b=>{if(request===requestId)weekBoard=b as PlanBoard}).catch(()=>{});
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
  try{await api.sync_now();clearViews(true);await refresh(true,true)}catch(e){syncError=String(e)}
  finally{syncing=false}
 }
 onMount(()=>{
  let disposed=false,unlisten=()=>{};
  // A link this version can't open says so instead of silently showing Today.
  const unknownLink=()=>{const h=location.hash;if(h.length>1&&!knownRoute(h))invalidLink=h.slice(1)};
  unknownLink();window.addEventListener('hashchange',unknownLink);
  const show=(next:string[])=>{
   const from=route;
   if(next[next.length-1]!=='today')invalidLink=null;
   const changed=next[next.length-1]!==route;
   if(changed){scrollPositions.set(route,scroll?.scrollTop??0);pendingSwap=null;detailState=null;selectedFactor='';sheet=false;search=false;chooseOpen=false;garminSheet=null;quickHelp.topic=null;addDraft=null;itemSheet=null;eventSheet=false;reviewData=null;journalOpen=false;journalDayOpen=null}
   trail=next;route=next[next.length-1];
   // Today's one-day activity list holds into a workout and back, not elsewhere.
   if(route!=='training' && !route.startsWith('workout/'))activityDay=null;
   const available=changed?cached(route):null;
   if(changed){
    data=available;
    if(available)void tick().then(()=>restoreScroll(next));
   }
   void refresh(false,status===null || from==='onboarding' && route!=='onboarding').then(async()=>{
    if(changed && !available){await tick();restoreScroll(next)}
   });
  };
  // An uncached page is read before it replaces the shown one, for up to HOLD_MS: most reads end sooner,
  // so the page opens whole instead of empty. A later navigation supersedes a held one.
  let navigations=0;
  navigation=createNavigation(next=>{
   const target=next[next.length-1],token=++navigations;
   if(target===route || route==='onboarding' || !hasRendered || !readsView(target) || cached(target))return show(next);
   const hold=new Promise(done=>window.setTimeout(done,HOLD_MS));
   void Promise.race([readView(target).catch(()=>{}),hold]).then(()=>{if(token===navigations)show(next)});
  });
  navigation.start();
  const nativeWindow=window as Window & {__raringBack?:()=>boolean};
  nativeWindow.__raringBack=back;
  const resume=()=>{
   if(document.visibilityState==='visible'){void refresh(true,true);if(api.desktopReminders){void api.check_reminders();if(route==='settings')void readReminderStatus()}}
   else {journalOpen=false;finishTouch()}
  };
  document.addEventListener('visibilitychange',resume);
  window.addEventListener('raring:resume',resume);
  void api.onStatusChanged(resume).then(fn=>{if(disposed)fn();else unlisten=fn});
  const poll=window.setInterval(resume,CACHE_POLL_MS);
  const clockTick=window.setInterval(()=>now=new Date(),60_000);
  return()=>{disposed=true;requestId++;navigation.destroy();delete nativeWindow.__raringBack;window.clearInterval(poll);window.clearInterval(clockTick);document.removeEventListener('visibilitychange',resume);window.removeEventListener('raring:resume',resume);unlisten()};
 });
 // Settings states why a desktop reminder could not be shown.
 $effect(()=>{if(route==='settings')void readReminderStatus()});
 // Read Today again when its phase ends; a touch defers the swap like any refresh.
 $effect(()=>{
  const until=todayData?.focus.until,wait=until?Date.parse(until)-Date.now()+1000:NaN;
  if(!(wait>0 && wait<2**31))return;
  const timer=window.setTimeout(()=>{if(route==='today')void refresh(true);else views.delete('today')},wait);
  return()=>window.clearTimeout(timer);
 });
 // A search result can name a card; the page opens scrolled to it instead of its last position.
 let pendingAnchor:string|null=null;
 function restoreScroll(next:string[]){
  if(route!==next[next.length-1] || !scroll)return;
  const card=pendingAnchor?document.getElementById(pendingAnchor):null;
  pendingAnchor=null;
  if(card)followCard(card);else scroll.scrollTop=scrollPositions.get(route)??0;
 }
 // Charts above the card finish loading after it opens; keep it in place until they settle or the person scrolls.
 function followCard(card:HTMLElement){
  const until=performance.now()+2500;let stopped=false;
  const stop=()=>{stopped=true};
  for(const kind of ['touchstart','wheel','keydown'])scroll.addEventListener(kind,stop,{once:true,passive:true});
  const step=()=>{if(stopped || !card.isConnected)return;card.scrollIntoView({block:'start'});if(performance.now()<until)requestAnimationFrame(step)};
  step();
 }
 function openFound(target:string,anchor?:string){
  pendingAnchor=anchor??null;
  // Activities is a Training section, not a page of its own.
  if(anchor==='training-sections'){trainingSection='activities';activityDay=null}
  if(target===route){if(anchor)void tick().then(()=>restoreScroll(trail))}else navigate(target);
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
 // A chosen export is validated before anything is written; the person confirms the preview.
 async function chooseImport(){
  if(importing)return;
  importing=true;importMessage='';
  try{importPreview=await api.choose_import()}catch(e){importPreview=null;importMessage=`Import failed: ${String(e)}`}
  finally{importing=false}
 }
 async function applyImport(){
  if(importing || !importPreview)return;
  importing=true;
  try{
   importMessage=importDone(await api.apply_import());
   importPreview=null;
   clearViews();
   // The import is already written; a failed settings reread is reported, not retried.
   try{settings=await api.get_settings()}catch(e){error=String(e)}
   void refresh(true);
  }catch(e){importMessage=`Import failed: ${String(e)}`}
  finally{importing=false}
 }

 // Strength's figure picker is a saved preference; the core applies it to every body map.
 async function saveFigure(figure:FigureChoice){
  if(await patchSettings({bodyFigure:figure})){clearViews();void refresh(true)}
 }
 // An explicit Garmin read of one strength workout's sets, never on navigation.
 async function refreshStrength(id:string){
  const session=await api.refresh_strength(id);
  if(session && workoutData?.summary.id===id)data={...workoutData,strength:session};
  return session;
 }
 let openedDay=$derived(datedDetail(route));
 let todayData=$derived(route==='today'||route==='readiness'?data as TodayView|null:null),sleepData=$derived(route.startsWith('sleep')?data as SleepView|null:null),trainingData=$derived(route==='training'?data as TrainingView|null:null),workoutData=$derived(route.startsWith('workout/')?data as WorkoutView|null:null),strengthData=$derived(route==='strength'?data as StrengthPage|null:null),healthData=$derived(route==='health'||openedDay?.page==='health'?data as HealthView|null:null),checkinData=$derived(route.startsWith('review/')?data as CheckinView|null:null),boardData=$derived(route==='plan'||route==='plan/commitments'?data as PlanBoard|null:null),progressData=$derived(route==='progress'?data as ProgressPage|null:null),profileData=$derived(route==='gear'?data as TrainingView|null:null),profilePage=$derived(route==='profile'?data as ProfileView|null:null);
 // Recorded today is the calendar day's (Your day's), not the displayed night's day: after midnight yesterday's run isn't today's.
 let todaysActivities=$derived((todayData?.activities??[]).filter(a=>a.startLocal.slice(0,10)===(todayData?.day?.date??todayData?.date)));
 let todayNaps=$derived((todayData?.day?.events??[]).filter(e=>e.kind==='nap'));
 // One list for everything recorded today, workouts and naps in the order they happened.
 const RECORDED_SHOWN=4;
 let recordedItems=$derived([...todaysActivities.map(a=>({at:+a.startLocal.slice(11,13)*60+ +a.startLocal.slice(14,16),activity:a,nap:null})),...todayNaps.map(n=>({at:n.startMin??n.endMin??0,activity:null,nap:n}))].sort((x,y)=>x.at-y.at));
 // The core orders Today's cards; Your day stays with the hero until a card such as tonight's plan comes first.
 let dayInHero=$derived(todayData?.focus.cards.find(c=>c!=='focus' && c!=='vitals')==='day');
 // Wind-down and late night: nothing about the week competes with tonight's plan.
 let calmNight=$derived(['windDown','lateNight'].includes(todayData?.focus.phase??''));
 // A dated Sleep route is that night's Last night.
 let sleepTab=$derived<'tonight'|'last'>(openedDay?.page==='sleep'?'last':route==='sleep/tonight'||(route==='sleep'&&!!sleepData?.tonightFirst)?'tonight':'last');
 // Tonight's header names the evening the plan's night starts and its window, not last night's times.
 const tonightLabel=(plan:SleepPlan|null)=>plan?`${dateLabel(addDays(plan.night,-1))} · ${plan.bedtimeStart?`${plan.bedtimeStart}–${plan.bedtime}`:`asleep by ${plan.bedtime}`}`:'Tonight';
 // Today shows readiness and the sleep score; other metrics' reasons belong with their own pages.
 let todayReasons=$derived(Object.entries(todayData?.missingMetricReasons??{}).filter(([k])=>k==='readiness'||k==='sleep').map(([,r])=>r));
 // Your day with nothing recorded (after midnight, or before the night syncs) would be a card of blanks.
 let dayShown=$derived(!!todayData?.day && (todayData.day.sleep.length>0 || todayData.day.events.length>0 || todayData.day.bodyBatteryNow!=null || [todayData.day.bodyBattery,todayData.day.stress,todayData.day.heartRate].some(s=>s.some(v=>v!=null))));
 let excludedSports=$derived(todayData?.adapt?.limits.excluded??[]);
 // Pages between the area cards and the closing summary, in order.
 let glow=$derived(route==='today' && todayData?.readiness && todayData.lead?.kind!=='waiting'?toneColor(todayData.readiness.tone):route.startsWith('sleep') && sleepData?.score?toneColor(sleepData.score.tone):route==='training' && trainingData?.acute?toneColor(trainingData.acute.tone):route==='health' && healthData?.fitnessAge.fitnessAge!=null && healthData.fitnessAge.chronologicalAge!=null?fitnessAgeColor(healthData.fitnessAge.fitnessAge,healthData.fitnessAge.chronologicalAge):null);
</script>
{#snippet recordedToday(compact:boolean)}{#if recordedItems.length}<Card title={!compact && todayData?.lead?.kind==='done'?'':'Recorded today'} extra="recorded">{#each recordedItems.slice(0,RECORDED_SHOWN) as item}<div class="rec-item">{#if item.activity}{@const a=item.activity}<ActivityRow activity={a} open={()=>navigate('workout/'+a.id)}/>{#if !compact && (a.aerobicTe!=null || a.benefit)}<div class="tags">{#if a.aerobicTe!=null}<span class="pill">Aerobic {num(a.aerobicTe,1)}{a.aerobicLabel?` · ${a.aerobicLabel}`:''}</span>{/if}{#if a.benefit}<span class="pill">{a.benefit}</span>{/if}</div>{/if}{:else if item.nap}{@const n=item.nap}<button class="activity" onclick={()=>navigate('sleep/last')}><span class="act-ico"><Icon name="moon" size={24}/></span><span class="act-main"><b>Nap</b><small>{n.startMin!=null?minuteClock(n.startMin):'Start not recorded'}{n.endMin!=null?`–${minuteClock(n.endMin)}`:''}{n.durationMin!=null?` · ${Math.round(n.durationMin)} min`:''}</small><small>{n.bodyBatteryChange!=null?`Body Battery ${n.bodyBatteryChange>=0?'+':''}${n.bodyBatteryChange}`:''}</small></span></button>{/if}</div>{/each}{#if recordedItems.length>RECORDED_SHOWN}<button class="link-row" onclick={()=>{trainingSection='activities';activityDay=todayData?.day?.date??todayData?.date??null;navigate('training')}}>All today’s activities ({todaysActivities.length})<Icon name="chevron" size={15}/></button>{/if}</Card>{/if}{/snippet}
{#snippet promptCard()}{#if prompt}<section class="card prompt-card" aria-label="Plan your week"><b>Plan your week</b><p class="small">{prompt.review?'See how last week went, then choose this week’s sessions.':'Nothing is placed this week yet.'}</p><div class="acts">{#if prompt.review}<button class="primary" onclick={()=>navigate('review/'+prompt!.review)}>Review last week</button>{/if}<button class={prompt.review?'secondary':'primary'} onclick={()=>answerPrompt('planned')}>Plan now</button><button class="secondary" onclick={()=>answerPrompt('dismissed')}>Not this week</button></div><button class="linkish" onclick={()=>void patchSettings({planPrompt:false}).then(ok=>{if(ok){clearViews();void refresh(true)}})}>Don’t offer this again</button></section>{/if}{/snippet}
<div class="phone"><div class="glow" style:--glow={glow?`color-mix(in srgb, ${glow} 12%, transparent)`:'transparent'}></div><main class="scroll" bind:this={scroll} ontouchstart={(e)=>{touching=true;startY=e.touches[0].clientY;startedAtTop=e.currentTarget.scrollTop===0}} ontouchend={pullEnd} ontouchcancel={finishTouch}><div class="page" class:reading={route.startsWith('review/')||route==='onboarding'||route==='plan/commitments'}>
{#if error && (data || route==='settings')}<p class="warning" role="status">{error}</p>{/if}
{#if detailState}<p class="warning" role="status">{#if detailState.kind==='loading'}Fetching this {openedDetail(route)?.page==='workout'?'workout':'day'} from Garmin…{:else}{detailState.message} <button class="link-inline" onclick={retryOpenedDetail}>Try again</button>{/if}</p>{/if}
{#if route==='onboarding'}
 <section class="welcome" class:intro={onboardStep===0}>
  <div class="welcome-brand"><BrandMark/><h1>Raring</h1></div>
  {#if onboardStep===0}
   <p class="welcome-tagline">A clearer view of your readiness, sleep and training.</p>
   <AboutRaring/>
   <button class="primary" onclick={async()=>{try{await api.begin_sign_in()}catch(e){error=String(e instanceof Error?e.message:e).replace(/^Error:\s*/,'')}}}>Sign in with Garmin <Icon name="arrow-right" size={18}/></button>
  {:else}
   <h2>Stay in the loop</h2><p>Get a heads-up when your readiness and weekly check-in are ready.</p>
   <button class="primary" onclick={async()=>{if(!await askNotifications())return;localStorage.setItem('notifications-step-done','1');navigate('today')}}>Enable notifications</button>
   <button class="secondary" onclick={()=>{localStorage.setItem('notifications-step-done','1');navigate('today')}}>Not now</button>
  {/if}
  {#if error || status?.lastError}<p class="small" role="alert">{error || status?.lastError}</p>{/if}
 </section>
{:else if route==='settings'}
 <Header title="Settings" eyebrow="RARING" back={back} backLabel={parentLabel}/><div class="dashboard" {@attach masonry}><Card title="Notifications" id="notifications">{#if reminderStatus}<p class="small" role="note">{reminderNote()}</p>{/if}{#if notificationsAllowed===false}<div class="blocked" role="note"><p>Android is blocking Raring’s notifications.{notificationsDeclined?' Allow them for Raring in Android Settings.':''}</p>{#if !notificationsDeclined}<button class="secondary" onclick={()=>void askNotifications()}>Allow</button>{/if}</div>{/if}{#each NOTIFICATIONS as [key,name,when]}<label class="setting" class:blocked-setting={notificationsAllowed===false}><span>{name}<small class="setting-note">{when}</small></span><input type="checkbox" checked={settings[key]} disabled={settingsBusy[key] || notificationsAllowed===false} onchange={(e)=>void patchSettings({[key]:e.currentTarget.checked})}/></label>{#if key==='notifyBedtime' && settings.notifyBedtime}<label class="setting lead"><span>Wind-down starts</span><select value={settings.bedtimeLeadMin} disabled={settingsBusy.bedtimeLeadMin || notificationsAllowed===false} onchange={(e)=>void patchPlan({bedtimeLeadMin:Number(e.currentTarget.value)})}>{#each [15,30,45,60,90] as lead}<option value={lead}>{lead} min before bedtime</option>{/each}</select></label>{/if}{/each}{#if api.supportsStatusSurfaces}<label class="setting"><span>Persistent status<small class="setting-note">Silent readings and today's focus, with the watch's upload time. Uses the existing sync; no extra Garmin requests.</small></span><input type="checkbox" checked={settings.notifyStatus} disabled={savingStatus || settingsBusy.notifyStatus || settingsBusy.statusOfferSeen} onchange={(e)=>{const input=e.currentTarget,on=input.checked;input.checked=settings.notifyStatus;void toggleStatus(on)}}/></label><p class="small">Add Raring widgets from your home screen's widget picker. Updates follow sync; Android may delay them.</p>{/if}</Card><Card title="Sleep"><label class="setting"><span>Time to fall asleep<small class="setting-note">Raring plans you in bed this long before Garmin's asleep-by time, and the wind-down before that. Garmin's window and need don't change.</small></span><select value={settings.fallAsleepMin} disabled={settingsBusy.fallAsleepMin} onchange={(e)=>void patchPlan({fallAsleepMin:Number(e.currentTarget.value)})}>{#each FALL_ASLEEP as m}<option value={m}>{m} min</option>{/each}</select></label></Card><Card title="Today"><label class="setting"><span>Follow my day<small class="setting-note">Lead with this morning, the day so far or tonight's sleep. Turn off for shift work or an irregular schedule. Raring doesn’t learn from how you use the app.</small></span><input type="checkbox" checked={settings.adaptiveToday} disabled={savingTodayLayout} onchange={(e)=>void saveTodayLayout(e.currentTarget.checked)}/></label></Card><Card title="Plan"><label class="setting"><span>Offer to plan the week<small class="setting-note">At the start of a week, after weeks with goals: review last week and place sessions. Never during wind-down.</small></span><input type="checkbox" checked={settings.planPrompt} disabled={settingsBusy.planPrompt} onchange={e=>void patchSettings({planPrompt:e.currentTarget.checked}).then(ok=>{if(ok)clearViews()})}/></label></Card><Card title="Journal privacy"><label class="setting"><span>Allow Garmin journal reads<small class="setting-note">Off by default. Last seven days only; never in notifications. Turning this off stops new reads. Already downloaded logs remain on this device and in future exports until you erase local data.</small></span><input type="checkbox" checked={settings.journalEnabled} disabled={settingsBusy.journalEnabled} onchange={e=>void toggleJournal(e.currentTarget.checked)}/></label></Card>{#if garminChangesFailed || garminChanges?.length}<GarminChanges changes={garminChanges} failed={garminChangesFailed}/>{/if}<Card title="Export" id="export"><p class="small">A ZIP of your Garmin history and Raring settings, never your sign-in. It leaves the phone only when you share it.</p><details class="small"><summary>What’s included</summary><p>Your saved Garmin responses, daily CSVs and Raring settings. Previously downloaded journal logs are included even if journal reads are now off; erase local data to remove them.</p></details><button class="secondary" disabled={exporting} onclick={exportData}>{exporting?'Preparing your export…':'Export your data'}</button>{#if exportMessage}<p class="small" role="status">{exportMessage}</p>{/if}</Card><Card title="Import" id="import">{#if !importPreview}<p class="small">Restore your Raring settings from an export of this Garmin account.</p><details class="small"><summary>What’s included</summary><p>Weekly plan history, goals, your sessions, plan links, rest days, pauses, limits, routines, events, plan-week answers and the plan prompt. An export from another Garmin account is refused. Garmin data isn’t imported: a sync downloads it again for this phone, and journal logs stay this phone’s own. Nothing leaves the phone.</p></details>{/if}{#if importPreview}<p class="small" role="status">{importSummary(importPreview)}</p><div class="acts"><button class="primary" disabled={importing} onclick={applyImport}>{importing?'Importing…':'Import these settings'}</button><button class="secondary" disabled={importing} onclick={()=>{importPreview=null;importMessage=''}}>Cancel</button></div><ul class="import-list">{#each importPreview.rows as row}<li><span>{row.kind}</span><small>{importRowText(row)}</small></li>{/each}</ul><ul class="small import-skips">{#each importPreview.skipped as skip}<li><b>{skip.what}</b> — {skip.reason}</li>{/each}</ul>{:else}<button class="secondary" disabled={importing} onclick={chooseImport}>{importing?'Opening your files…':'Choose a Raring export'}</button>{/if}{#if importMessage}<p class="small" role="status">{importMessage}</p>{/if}</Card><Card title="Account" id="account">{#if !confirmSignOut}<button class="signout" onclick={()=>confirmSignOut=true}>Sign out and erase this phone’s data</button>{:else}<div class="signout-confirm" role="group" aria-label="Sign out"><p>Signing out erases your Garmin history, journal logs, goals, plans, limits and routines from this phone. Nothing is deleted from Garmin. Export first to keep your settings.</p><div class="acts"><button class="secondary" disabled={exporting} onclick={exportData}>{exporting?'Preparing your export…':'Export first'}</button><button class="secondary danger" onclick={signOut}>Sign out and erase</button><button class="secondary" onclick={()=>confirmSignOut=false}>Cancel</button></div>{#if exportMessage}<p class="small" role="status">{exportMessage}</p>{/if}</div>{/if}</Card><Card title="Help" onclick={()=>navigate('help')}><p class="small">Lifting with your watch, sets, the body map and which watches work.</p></Card><Card title="How Raring works"><AboutRaring/></Card></div>
{:else if route==='help'}
 <Header title="Help" eyebrow="RARING" back={back} backLabel={parentLabel}/><div class="help-list">{#each helpSections as section}<Card title={section.title}>{#each section.pages as page}<button class="help-row" onclick={()=>navigate('help/'+page)}><span class="help-icon"><Icon name={helpContent[page].icon} size={18}/></span><b>{helpContent[page].title}</b><Icon name="chevron" size={16}/></button>{/each}</Card>{/each}<p class="small help-foot">Still stuck? Ask at {GITHUB_QUESTIONS}</p></div>
{:else if route.startsWith('help/') && route.slice(5) in helpContent}
 {@const page=helpContent[route.slice(5) as keyof typeof helpContent]}<Header title={page.title} eyebrow="HELP" back={back} backLabel={parentLabel}/><div class="help-list"><Card><p class="help-intro">{page.intro}</p><ol class="help-steps">{#each page.steps as step,i}<li><span class="n">{i+1}</span>{step}</li>{/each}</ol></Card><p class="small help-foot">Still stuck? Ask at {GITHUB_QUESTIONS}</p></div>
{:else if route==='progress/milestones'}
 <Header title="Records and milestones" eyebrow="Progress" back={back} backLabel={parentLabel}/>{#if milestoneRows}{#each [...new Set(milestoneRows.map(m=>m.date.slice(0,7)))] as month}<section class="card archive-month" aria-label={monthLabel(month+'-01')}><h3>{monthLabel(month+'-01')}</h3>{#each milestoneRows.filter(m=>m.date.startsWith(month)) as m (m.id)}<button class="ms" disabled={!m.route} onclick={()=>m.route&&navigate(m.route)}><span><b>{m.title}</b><small>{[m.value,dateLabel(m.date)].filter(Boolean).join(' · ')}{m.corrected?' · corrected after a Garmin edit':''}</small></span>{#if m.route}<Icon name="chevron" size={15}/>{/if}</button>{/each}</section>{:else}<p class="small">Bests and other milestones appear here as Garmin records them.</p>{/each}{:else}<p class="small">Reading your records…</p>{/if}
{:else if route==='reviews'}
 <Header title="Weekly reviews" eyebrow="Progress" back={back} backLabel={parentLabel}/>{#if reviewRows}{#each reviewRows as r}<button class="card review-row" onclick={()=>navigate('review/'+r.start)}><span><b>Week of {dateLabel(r.start)}</b><small>{r.counted?`${r.met} of ${r.goals} goals met`:'Not counted'}{r.reasons.length?' · '+r.reasons.join(' · '):''}</small></span><Icon name="chevron" size={16}/></button>{:else}<p class="small">Weeks appear here once you've chosen goals.</p>{/each}{:else}<p class="small">Loading…</p>{/if}
{:else if loading && !hasRendered}<div class="empty"><BrandMark/><h2>{status?.lastSync?'Loading your day…':'Syncing your Garmin data…'}</h2></div>
{:else if !data && hasRendered}<Header title={route[0].toUpperCase()+route.slice(1)} eyebrow="RARING"/><p class="small">{error || (loading?'Preparing this view…':'No data for this view yet.')}</p>
{:else if !data}<div class="empty"><BrandMark/><h2>Syncing your Garmin data…</h2><p>{error && !error.includes('not_ready')?error:'Your day will appear here soon.'}</p><button class="secondary" disabled={syncBusy} onclick={syncNow}>{syncBusy?'Syncing…':'Sync now'}</button>{#if syncError}<p>{syncError}</p>{/if}</div>
{:else if todayData && route==='today'}
 <div class="page-hero today-hero"><Header title="Today" eyebrow={dateLabel(todayData.date)} {...rootControls}/>
 {#if invalidLink}<p class="invalid-link" role="status">That link isn’t in this version of Raring, so Today opened. <button class="link-inline" onclick={()=>invalidLink=null}>Dismiss</button></p>{/if}<div class="sync-indicator"><span class="small" aria-live="polite">{#if todayData.watchLastUpload}{watchLabel(todayData.watchLastUpload,now)}{:else}Watch upload time unavailable{/if}{#if status && !status.historyComplete && status.historyFrom} · History since {monthLabel(status.historyFrom)}{/if}{#if syncBusy} · Refreshing…{:else if syncError || status?.lastError} · Raring sync delayed · {syncError || status?.lastError}{/if}</span><button class="sync-button" disabled={syncBusy} onclick={syncNow}>Refresh</button></div>
 {#if watchNeedsSync(todayData,now)}<Card title="Watch upload" extra="watch-notice"><p>Open Garmin Connect to sync your watch</p>{#if todayData.watchSleepUploadStale}<p class="small">Last night's sleep hasn't uploaded yet.</p>{/if}{#if connectUnavailable}<p class="small" role="status">Open Garmin Connect on your phone, then sync your watch there.</p>{:else}<button class="watch-open" onclick={openConnect}>Open Garmin Connect <Icon name="arrow-right" size={16}/></button>{/if}</Card>{/if}
 {#if todayReasons.length}<Card title="About missing readings" extra="watch-notice"><div class="missing-reasons">{#each todayReasons as reason}<p>{reason}</p>{/each}</div></Card>{/if}
 <TodayHero today={todayData} {navigate} {act} adapt={()=>adaptOpen=true} plan={()=>addDraft={date:todayData!.date}} choose={weekBoard?()=>chooseOpen=true:undefined} stillPlanned={(weekBoard?.items??[]).filter(p=>p.intention.date===todayData!.date&&p.state==='planned'&&p.intention.kind==='workout').map(p=>({name:p.intention.name??SPORT_NAMES[p.intention.sport],time:p.intention.time}))}/>{#if dayInHero}{@render recordedToday(false)}{/if}{#if todayData.day && dayShown && dayInHero}<YourDay day={todayData.day} {navigate}/>{/if}</div><div class="dashboard" {@attach masonry}>
 {#if statusOffer}<Card title="Persistent status"><p>Keep Body Battery, stress and today's focus in a silent notification. Always labeled with the watch's upload time.</p><p class="small">You can change this in Settings.</p><button class="primary" disabled={savingStatus} onclick={()=>void toggleStatus(true)}>Enable persistent status</button><button class="secondary" disabled={savingStatus} onclick={()=>statusOffer=false}>Not now</button></Card>{/if}
 {#each todayData.focus.cards.filter(c=>c!=='focus' && !(c==='day' && dayInHero)) as card,i (card)}
  {#if card==='day'}{#if todayData.day && dayShown}<YourDay day={todayData.day} {navigate} compact/>{/if}{@render recordedToday(true)}
  {:else if card==='sleepPlan' && todayData.sleepPlan}<SleepPlanCard plan={todayData.sleepPlan} {now} lead={i===0} mode={todayData.focus.planOpen?(({evening:'evening',windDown:'windDown',lateNight:'lateNight'} as const)[todayData.focus.phase as 'evening']??'full'):'line'} open={()=>navigate('sleep/tonight')}/>
  {:else if card==='vitals' && todayData.vitals.readings.length}<VitalsMonitor vitals={todayData.vitals} compact open={(kind)=>navigate('metric/'+kind)}/>{/if}
 {/each}
 {#if weekBoard && !calmNight}<Card title="This week" onclick={()=>navigate('plan')} extra="week-card"><div class="wk2"><div>{#if weekBoard.ledger.length}<WeekSummary board={weekBoard} compact onGoals={()=>navigate('plan/commitments')}/>{:else}<p class="small">No weekly goals · set them in Plan</p>{/if}</div>{#if weekBoard.backlogPaused || weekBoard.backlog.length}<div class="left-week" role="group" aria-label="Left this week"><h3>Left this week</h3>{#if weekBoard.backlogPaused}<p class="small">Paused · nothing to catch up.</p>{:else}{#each weekBoard.backlog as b}<div class="left-item" style:--c={GOALS[b.kind].color}><span><i></i>{b.kind==='strength'?'Strength session':`${amount(b.kind,b.amount)} ${GOALS[b.kind].name}`}</span>{#if b.kind!=='sleepTiming' && todayData.focus.phase!=='evening'}{@const d=backlogDraft(b,weekBoard!.today)}{#if excludedSports.includes(d.sport)}<small class="excluded">Excluded by your limits</small>{:else}<button class="small-edit" onclick={(e)=>{e.stopPropagation();addDraft=d}}>Do it today</button>{/if}{/if}</div>{/each}{/if}</div>{/if}</div><div class="week-cells"><DayCells days={weekBoard.days} select={()=>navigate('plan')} size={30}/></div></Card>{/if}
 {#if !todayData.day && todayData.bodyBattery.now!=null}<div class="row">{#if !todayData.day && todayData.bodyBattery.now!=null}<Card title="Body Battery" onclick={()=>navigate('metric/bodyBattery')}><div class="bb"><div class="battery"><i style:height={Math.max(0,Math.min(100,todayData.bodyBattery.now))+'%'} style:background={batteryColor(todayData.bodyBattery.now)}></i></div><div><div class="big n" style:color={batteryColor(todayData.bodyBattery.now)}>{num(todayData.bodyBattery.now)}</div>{#if todayData.bodyBattery.charged!=null}<div class="small"><b>+{num(todayData.bodyBattery.charged)}</b> charged today</div>{/if}</div></div></Card>{/if}</div>{/if}
 {#if !todayData.day && (todayData.stress.average!=null || todayData.stress.timeline.length)}{@const stress=todayData.stress}<Card title="Stress" onclick={()=>navigate('metric/stress')}>{#snippet aside()}{#if stress.qualifier}<span class="pill" style:color={toneColor(stress.tone)}>{stress.qualifier}</span>{/if}{/snippet}<div class="stress-summary"><div><div class="big n" style:color={stress.level?stressColors[stress.level]:undefined}>{num(stress.average)}</div><div class="small">{stress.level?stressNames[stress.level]+' · ':''}average</div></div>{#if stress.timeline.length}<StressTimeline {stress} compact/>{/if}</div></Card>{/if}
 {#if todayData.checkinReady && !calmNight}<Card title="Weekly check-in" onclick={()=>navigate('review/latest')}><p class="checkin-tease">See your week at a glance</p></Card>{/if}
</div><p class="end-note">That’s today.</p>
{:else if route==='readiness' && todayData}
 {#snippet readinessHeader()}<Header title="Readiness" eyebrow={dateLabel(todayData.date)} back={back} backLabel="Today"/>{/snippet}<ReadinessDetail {todayData} header={readinessHeader} bind:selectedFactor/>
{:else if sleepData}
 {@const historical=openedDay?.page==='sleep'}
 {#snippet sleepHeader()}{#if historical}<Header title="Sleep" eyebrow={dateLabel(sleepData.date)} back={back} backLabel={parentLabel}/>{:else}<Header title="Sleep" eyebrow={sleepTab==='tonight'?tonightLabel(sleepData.sleepPlan):sleepData.startLocal?`${clock(sleepData.startLocal)} – ${clock(sleepData.endLocal)}`:dateLabel(sleepData.date)} {...rootControls}/>{/if}{/snippet}{#snippet tonightTab()}<SleepTonight sleep={sleepData} board={weekBoard} {act} {navigate} {now} reminder={settings.notifyBedtime} setReminder={on=>void patchSettings({notifyBedtime:on})} fallAsleep={settings.fallAsleepMin} setFallAsleep={m=>void patchPlan({fallAsleepMin:m})}/>{/snippet}<SleepDetail {sleepData} header={sleepHeader} {navigate} {historical} tab={sleepTab} setTab={historical?undefined:t=>navigate('sleep/'+(t==='last'?'last':'tonight'))} tonight={tonightTab}/>
{:else if trainingData}
 {#snippet trainingHeader()}<Header title="Training" eyebrow={dateLabel(trainingData.lanes?.today ?? trainingData.week.at(-1)?.date ?? '')} {...rootControls}/>{/snippet}<TrainingTab {trainingData} header={trainingHeader} {navigate} bind:section={trainingSection} bind:muscleRange bind:day={activityDay} muscles={api.muscles} activityPage={api.activity_page}/>
{:else if workoutData}
 {@const sets=workoutData.strength}<Header title={workoutData.summary.name} eyebrow={(sets?'Strength · ':'')+dateLabel(workoutData.summary.startLocal.slice(0,10))+' · '+clock(workoutData.summary.startLocal)} back={back} backLabel={parentLabel} onHelp={sets?()=>sheet=true:undefined}/><WorkoutDetail {workoutData} {navigate} help={()=>sheet=true} refresh={refreshStrength}/>
{:else if strengthData}
 <Header title="Strength" eyebrow="Lifts and muscles" back={back} backLabel={parentLabel} onHelp={()=>sheet=true}/>{#if strengthData.strengthPage}<StrengthHistory view={strengthData.strengthPage} {navigate} liftSeries={api.lift_series} muscles={api.muscles} figureChoice={settings.bodyFigure} setFigure={saveFigure} bind:picked={liftPicked} bind:metric={liftMetric} bind:range={liftRange} bind:muscleRange/>{:else}<Card title="No sets yet"><p class="small untracked">Start a Strength activity on your watch and press lap after each set. Your sets appear here after Garmin Connect syncs.</p><button class="watch-open" onclick={()=>navigate('help/strength-start')}>Lift with your watch <Icon name="arrow-right" size={16}/></button></Card>{/if}
{:else if healthData}
 {@const historical=openedDay?.page==='health'}
 {#if historical}<Header title="Health" eyebrow={dateLabel(healthData.date)} back={back} backLabel={parentLabel}/>{:else}<Header title="Health" eyebrow={dateLabel(healthData.date)} {...rootControls}/>{/if}<HealthDetail {healthData} {navigate} {toggleJournal} {historical} journalBusy={settingsBusy.journalEnabled} bind:journalOpen bind:journalDayOpen>{#snippet weightContent()}<WeightCard bind:range={weightRange} {navigate} endDate={historical?healthData.date:undefined}/>{/snippet}</HealthDetail>
{:else if boardData && route==='plan'}
 {#snippet planHeader()}<Header title="Plan" eyebrow="This week" {...rootControls}/>{/snippet}<PlanTab board={boardData} {navigate} {act} header={planHeader} openAdd={d=>addDraft=d as Draft} openItem={p=>itemSheet=p} openEvent={()=>eventSheet=true} openGarmin={g=>garminSheet=g} limited={!!adaptState&&(adaptState.limits.excluded.length>0||!!adaptState.limits.note)}/>
{:else if boardData && route==='plan/commitments'}
 <Header title="Weekly goals" eyebrow="Plan" back={back} backLabel={parentLabel}/><GoalEditor initial={boardData.goals?.targets??null} garminGoal={boardData.garminIntensityGoal} busy={changing} save={targets=>act({action:'setGoals',targets},back)}/>
{:else if progressData}
 {#snippet progressHeader()}<Header title="Progress" eyebrow="Trends and reviews" {...rootControls}/>{/snippet}<ProgressTab initial={progressData.progress} {navigate} header={progressHeader} onChange={saveProgress} bind:section={progressSection} bind:domainQuery={progressDomains}/>
{:else if profilePage && route==='profile'}
 {#snippet profileHeader()}<Header title="Profile" eyebrow="RARING" back={back} backLabel={parentLabel}/>{/snippet}<ProfileScreen {status} profile={profilePage} {navigate} limits={()=>{adaptLimits=true;adaptOpen=true}} header={profileHeader}/>
{:else if profileData && route==='gear'}
 <Header title="Gear" eyebrow="Garmin" back={back} backLabel={parentLabel}/>{#if profileData.gear}<GearCard gear={profileData.gear}/>{:else}<p class="small">No gear list from Garmin yet.</p>{/if}
{:else if route==='health/weight'}
 <Header title="Weight" eyebrow="Garmin weigh-ins · Raring trend" back={back} backLabel={parentLabel}/><WeightCard initial={data as WeightView} bind:range={weightRange}/>
{:else if route.startsWith('metric/')}
 <Header title={route.split('/')[1]==='restingHr' && (data as HealthView).vitals?.readings.some(isSleepHr) ? 'Sleep HR' : metricNames[route.split('/')[1]]} eyebrow={openedDay?dateLabel(openedDay.date):'GARMIN METRIC'} back={back} backLabel={parentLabel}/><MetricDetail metric={route.split('/')[1]} data={data as TodayView|HealthView|TrainingView} dayDate={openedDay?.page==='heartRate'?openedDay.date:null}/>
{:else if checkinData}
 <Header title="Weekly review" eyebrow={'Week of '+dateLabel(checkinData.weekStart)} back={back} backLabel={parentLabel}/><CheckinPage checkin={checkinData} review={reviewData} {act} {navigate} busy={changing}/>
{/if}
</div></main>{#if sheet && (workoutData?.strength || strengthData)}<HelpSheet close={()=>sheet=false} more={()=>{sheet=false;navigate('help/strength-sets')}}/>{/if}{#if addDraft && (boardData ?? weekBoard)}<AddSessionSheet board={(boardData ?? weekBoard)!} draft={addDraft} close={()=>addDraft=null} {act} busy={changing}/>{/if}{#if itemSheet && (boardData ?? weekBoard)}<ItemSheet board={(boardData ?? weekBoard)!} item={itemSheet} close={()=>itemSheet=null} {act} {navigate} limits={adaptState?.limits??null}/>{/if}{#if eventSheet}<EventSheet today={(boardData ?? weekBoard)?.today ?? new Date().toISOString().slice(0,10)} close={()=>eventSheet=false} {act}/>{/if}{#if garminSheet && (boardData ?? weekBoard)}<GarminSheet session={garminSheet} today={(boardData ?? weekBoard)!.today} limits={adaptState?.limits??null} close={()=>garminSheet=null} {act} {navigate}/>{/if}{#if quickHelp.topic}<QuickHelpSheet {navigate}/>{/if}{#if chooseOpen && weekBoard}<ChooseSheet board={weekBoard} kept={todayData?.lead?.kept?(todayData.lead.intention??todayData.lead.garmin):null} excluded={excludedSports} limited={!!adaptState&&(adaptState.limits.excluded.length>0||!!adaptState.limits.note)} close={()=>chooseOpen=false} {act} add={d=>addDraft=d}/>{/if}{#if adaptOpen}<AdaptSheet current={adaptState} startLimits={adaptLimits} goals={(weekBoard?.ledger??[]).filter(l=>!l.observation).map(l=>`${GOALS[l.kind].name}: ${amount(l.kind,l.target)}`)} changeGoals={()=>navigate('plan/commitments')} close={()=>{adaptOpen=false;adaptLimits=false}} {act} busy={changing} garminWorkouts={[...(weekBoard?.garmin??[]).map(g=>({name:g.name,source:'Garmin schedule'})),...(weekBoard?.items??[]).filter(p=>p.coach&&p.state==='planned').map(p=>({name:p.intention.name??'Garmin Coach session',source:'Garmin Coach'}))]}/>{/if}{#if search}<SearchSheet close={()=>search=false} navigate={openFound} sleepHr={()=>readView('health').then(h=>(h as HealthView).vitals.readings.some(isSleepHr))}/>{/if}{#if route!=='onboarding'}<TabBar active={activeTab} {navigate}/>{/if}</div>
