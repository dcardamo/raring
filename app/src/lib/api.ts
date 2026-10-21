import { invoke, isTauri } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
// data.ts is generated alongside these JSON fixtures and checks each value with `satisfies`.
import sampleToday from './sample/today.json';
import sampleSleep from './sample/sleep.json';
import sampleTraining from './sample/training.json';
import sampleWorkout from './sample/workout.json';
import sampleStrengthWorkout from './sample/strengthWorkout.json';
import sampleStrengthWorkoutHistory from './sample/strengthWorkoutHistory.json';
import sampleOtherWorkouts from './sample/otherWorkouts.json';
import sampleStrength from './sample/strength.json';
import sampleHealth from './sample/health.json';
import type {HeartRateDay} from './generated/HeartRateDay';
import sampleCheckin from './sample/checkin.json';
import sampleStatus from './sample/status.json';
import sampleBoard from './sample/board.json';
import sampleReview from './sample/review.json';
import sampleReviews from './sample/reviews.json';
import sampleProfile from './sample/profile.json';
import sampleDomains from './sample/progressDomains.json';
import type { ProgressDomains } from './generated/ProgressDomains';
import type { DomainQuery } from './generated/DomainQuery';
import type { Milestone } from './generated/Milestone';
import type { ProfileView } from './generated/ProfileView';
import type { LiftSeries } from './generated/LiftSeries';
import type { ActivityPage } from './generated/ActivityPage';
import type { ActivityMonth } from './generated/ActivityMonth';
import type { Icon as IconName } from './generated/Icon';
import type { MuscleRange } from './generated/MuscleRange';
import type { AppStatus } from './generated/AppStatus';
import type { TodayView } from './generated/TodayView';
import type { SleepView } from './generated/SleepView';
import type { TrainingView } from './generated/TrainingView';
import type { WorkoutView } from './generated/WorkoutView';
import type { RouteMapView } from './generated/RouteMapView';
import type { StrengthView } from './generated/StrengthView';
import type { StrengthSession } from './generated/StrengthSession';
import type { HealthView } from './generated/HealthView';
import type { CheckinView } from './generated/CheckinView';
import type { Settings } from './generated/Settings';
import type { TrendQuery } from './generated/TrendQuery';
import type { TrendView } from './generated/TrendView';
import { sampleTrend, sampleStressMonth } from './sample/trend';
import type { StressMonth } from './generated/StressMonth';
import type { LocalAction } from './generated/LocalAction';
import type { PlanBoard } from './generated/PlanBoard';
import type { WeekReview } from './generated/WeekReview';
import type { ReviewRow } from './generated/ReviewRow';
import type { WeightView } from './generated/WeightView';
import type { TrendRange } from './generated/TrendRange';
import {weightFixtures} from './sample/weight';
import type { ProgressView } from './generated/ProgressView';
import type { Period } from './generated/Period';
import sampleProgress from './sample/progress.json';
import type { GarminChange } from './generated/GarminChange';
import { sampleGarminChanges } from './sample/garminChanges';
import type { ImportPreview } from './generated/ImportPreview';
export const previewOnly = !isTauri();

export type { Settings } from './generated/Settings';
export type { ImportPreview } from './generated/ImportPreview';
export type SettingsPatch = Partial<Settings>;
export const supportsStatusSurfaces = (isTauri() && /Android/.test(navigator.userAgent)) || new URLSearchParams(location.search).get('fixture')==='status-offer';
// Desktop delivers reminders itself while the app runs; Android schedules them natively.
export const desktopReminders = isTauri() && !/Android/.test(navigator.userAgent);
export type ReminderStatus = {supported:boolean;granted:boolean;error:string|null;nextAt:string|null};
export const defaultSettings = {notifyMorningReadiness:true,notifyWeeklyCheckin:true,notifyMetricChanges:true,notifyVitals:true,notifyTrainingStatus:false,notifyBedtime:false,notifyStatus:false,statusOfferSeen:false,bedtimeLeadMin:30,fallAsleepMin:15,adaptiveToday:true,journalEnabled:false,planPrompt:true,bodyFigure:'profile',streetMaps:true} satisfies Settings;
let previewSettings:Settings = {...defaultSettings};
const copy = <T>(value:T):T => structuredClone(value);
const call = <T>(name:string,args:Record<string,unknown>|undefined,fallback:T):Promise<T> => isTauri()?invoke<T>(name,args):Promise.resolve(copy(fallback));
export const app_status = ():Promise<AppStatus> => call('app_status',undefined,sampleStatus as AppStatus);
export const today = (date?:string):Promise<TodayView> => call('today',{date},sampleToday as unknown as TodayView);
export const sleep = (date?:string):Promise<SleepView> => call('sleep',{date},date?{...copy(sampleSleep as unknown as SleepView),date}:sampleSleep as unknown as SleepView);
// The browser preview uses the sample's week or four weeks; the app asks the core.
export const muscles = (range:TrendRange):Promise<MuscleRange|null> => {
 if(isTauri())return invoke<MuscleRange|null>('muscles',{range});
 const v=sampleStrength as unknown as StrengthView,short=range==='days7'||range==='days14';
 const days=({days7:7,days14:14,days30:30,days90:90,year:365} as Record<string,number>)[range]??120;
 const list=short?v.thisWeek:v.last4w,gapAt=Math.round(days/14);
 return Promise.resolve({range,from:new Date(Date.parse(v.date)-(days-1)*864e5).toISOString().slice(0,10),to:v.date,days,figure:v.figure,muscles:list,
  gaps:list.filter(m=>m.sets<=gapAt).sort((a,b)=>a.sets-b.sets).map(m=>m.muscle),gapAt});
};
// Older stored recordings; the browser preview pages through the sample's activities and strength workouts.
const sampleActivities = () => {
 const all=[...(sampleTraining as unknown as TrainingView).workouts,...(sampleStrength as unknown as StrengthView).workouts.map(w=>w.activity)];
 const seen=new Set<string>();
 return all.filter(a=>!seen.has(a.id)&&seen.add(a.id)).sort((a,b)=>b.startLocal.localeCompare(a.startLocal)||b.id.localeCompare(a.id));
};
export const activity_page = (beforeStart:string,beforeId:string,icon:IconName|null):Promise<ActivityPage> => {
 if(isTauri())return invoke<ActivityPage>('activity_page',{beforeStart,beforeId,icon});
 const older=sampleActivities().filter(a=>a.startLocal<beforeStart||a.startLocal===beforeStart&&a.id<beforeId).filter(a=>!icon||a.icon===icon);
 return Promise.resolve({activities:older.slice(0,20),more:older.length>20});
};
// One month of stored recordings (this month without a date); the preview's weeks start on Monday.
export const activity_month = (month:string|null):Promise<ActivityMonth> => {
 if(isTauri())return invoke<ActivityMonth>('activity_month',{month});
 const today=sampleToday.date,from=(month??today).slice(0,7)+'-01',grid=new Date(from+'T12:00:00Z'),all=sampleActivities();
 grid.setUTCDate(grid.getUTCDate()-(grid.getUTCDay()+6)%7);
 return Promise.resolve(copy({month:from,gridStart:grid.toISOString().slice(0,10),today,first:all.at(-1)?.startLocal.slice(0,10)??null,activities:all.filter(a=>a.startLocal.startsWith(from.slice(0,7)))}));
};
// The browser preview derives a series from the sample's sessions; the app asks the core.
export const lift_series = (key:string,range:TrendRange):Promise<LiftSeries|null> => {
 if(isTauri())return invoke<LiftSeries|null>('lift_series',{key,range});
 const e=(sampleStrength as unknown as StrengthView).exercises.find(x=>x.key===key);
 if(!e)return Promise.resolve(null);
 const days=({days7:7,days14:14,days30:30,days90:90,year:365} as Record<string,number>)[range];
 const from=days?new Date(Date.parse(sampleToday.date)-(days-1)*864e5).toISOString().slice(0,10):'';
 const points=[...e.sessions].reverse().filter(s=>s.date>=from).map(s=>({date:s.date,activityId:s.activityId,max:s.top?.weight??null,volume:s.top?.weight!=null?Math.round(s.top.weight*s.reps.reduce<number>((t,r)=>t+(r??0),0)):null,excluded:s.top?0:s.reps.length,
  mostReps:s.top?null:Math.max(0,...s.reps.map(r=>r??0))||null}));
 return Promise.resolve({key,name:e.name,unit:(sampleStrength as unknown as StrengthView).unit,range,from:from||(points[0]?.date??sampleToday.date),to:sampleToday.date,points});
};
export const milestone_archive = ():Promise<Milestone[]> => call('milestone_archive',undefined,(sampleProgress as unknown as {milestones:Milestone[]}).milestones);
export const progress_domains = (query:DomainQuery):Promise<ProgressDomains> => call('progress_domains',{query},sampleDomains as unknown as ProgressDomains);
export const profile = ():Promise<ProfileView> => call('profile',undefined,sampleProfile as unknown as ProfileView);
export const training = (date?:string):Promise<TrainingView> => call('training',{date},sampleTraining as unknown as TrainingView);
export const workout = (id:string):Promise<WorkoutView> => {
 if(isTauri())return invoke<WorkoutView>('workout',{id});
 const history={...sampleStrengthWorkoutHistory,...sampleOtherWorkouts} as unknown as Record<string,WorkoutView>;
 const listed=sampleTraining.workouts.find(a=>a.id===id);
 // Other listed preview activities keep their own summary, never a run's charts.
 const summaryOnly=listed?{summary:listed,recoveryHours:null,stats:[],route:[],heartRate:[],zoneFloors:[],splits:[],strength:null,along:null,more:[],effects:[],vo2maxNew:null,counted:null,map:null}:null;
 const value=id===sampleStrengthWorkout.summary.id?sampleStrengthWorkout:id===sampleWorkout.summary.id?sampleWorkout:Object.hasOwn(history,id)?history[id]:summaryOnly;
 if(!value)return Promise.reject(new Error('No preview workout for this ID'));
 const view=copy(value as unknown as WorkoutView);
 return Promise.resolve(previewSettings.streetMaps?view:{...view,map:null});
};
// The route card's street map: the core downloads the workout's area once if needed and draws the card once.
export const route_map = async (id:string):Promise<RouteMapView|null> => {
 if(isTauri())return invoke<RouteMapView|null>('route_map',{id});
 if(!previewSettings.streetMaps || id!==sampleWorkout.summary.id)return null;
 // The preview's drawn sample, in its own chunk: only a card still waiting for its map loads it.
 const drawn=(await import('./sample/workoutMap.json')).default as unknown as RouteMapView;
 return {...copy(drawn),state:'ready'};
};
export const strength = (date?:string):Promise<StrengthView|null> => call('strength',{date},sampleStrength as unknown as StrengthView);
// One explicit Garmin read of a workout's sets; navigation never calls it.
export const refresh_strength = (id:string):Promise<StrengthSession|null> => isTauri()?invoke<StrengthSession|null>('refresh_strength',{id}):Promise.reject(new Error('Refreshing sets needs the Raring app'));
export const health = (date?:string):Promise<HealthView> => isTauri()?invoke<HealthView>('health',{date}):Promise.resolve({...copy(sampleHealth as unknown as HealthView),...(date?{date}:{}),journalEnabled:previewSettings.journalEnabled});
export const heart_rate_day = (date:string):Promise<HeartRateDay> => isTauri()?invoke<HeartRateDay>('heart_rate_day',{date}):Promise.resolve({...((sampleHealth as unknown as HealthView).heartRateDay),date});
// An explicit open of a past day or an older workout; navigation prefetch never calls these.
export const fetch_day_detail = (date:string,scope:'sleep'|'day'|'stress'):Promise<void> => isTauri()?invoke<void>('fetch_day_detail',{date,scope}):Promise.resolve();
export const fetch_workout_detail = (id:string):Promise<void> => isTauri()?invoke<void>('fetch_workout_detail',{id}):Promise.resolve();
export const checkin = (weekStart?:string):Promise<CheckinView> => call('checkin',{weekStart},sampleCheckin as unknown as CheckinView);
export const trend = (query:TrendQuery):Promise<TrendView> => isTauri()?invoke<TrendView>('trend',{query}):Promise.resolve().then(()=>sampleTrend(query));
export const stress_month = (month:string):Promise<StressMonth> => isTauri()?invoke<StressMonth>('stress_month',{month}):Promise.resolve().then(()=>sampleStressMonth(month));
// The preview keeps local changes in memory; only the Rust core applies the plan rules.
let previewBoard:PlanBoard = copy(sampleBoard as unknown as PlanBoard);
export const plan_board = (date?:string):Promise<PlanBoard> => call('plan_board',{date},previewBoard);
export const week_review = (date:string):Promise<WeekReview> => call('week_review',{date},sampleReview as unknown as WeekReview);
export const reviews = ():Promise<ReviewRow[]> => call('reviews',undefined,sampleReviews as unknown as ReviewRow[]);
export const progress = (period:Period='month',date?:string):Promise<ProgressView> => call('progress',{period,date},sampleProgress as unknown as ProgressView);
export const weight_trend = (range:TrendRange='days30',date?:string):Promise<WeightView> => {
 if(isTauri())return invoke<WeightView>('weight_trend',{range,date});
 if(range==='custom')return Promise.reject(new Error('Use a weight range from 7D to Max'));
 return Promise.resolve(copy(weightFixtures.kg[range]));
};
// Every local change (goals, sessions, links, rest, pauses, limits) goes through one command.
export const local_action = (action:LocalAction):Promise<void> => {
 if(!isTauri()){
  if(action.action==='setGoals')previewBoard={...previewBoard,goals:{from:previewBoard.today,targets:copy(action.targets),source:'revision',savedAt:`${previewBoard.today}T12:00:00`}};
  if(action.action==='observationExplained')previewBoard={...previewBoard,explainObservation:false};
  if(action.action==='setPromptWeek')previewBoard={...previewBoard,prompt:null};
  return Promise.resolve();
 }
 return invoke<void>('local_action',{action});
};
// Recent Garmin writes; the preview has none unless the garmin-changes fixture asks.
export const garmin_changes = ():Promise<GarminChange[]> => call('garmin_changes',undefined,new URLSearchParams(location.search).get('fixture')==='garmin-changes'?sampleGarminChanges:[]);
export const sync_now = ():Promise<void> => call('sync_now',undefined,undefined);
export const open_garmin_connect = ():Promise<boolean> => call('open_garmin_connect',undefined,false);
export const begin_sign_in = ():Promise<void> => call('begin_sign_in',undefined,undefined);
export const sign_out = ():Promise<void> => call('sign_out',undefined,undefined);
export const export_archive = ():Promise<string> => call('export_archive',undefined,'Preview: no file created');
// The browser preview shows a realistic export; only the core validates and imports one.
const sampleImport:ImportPreview = {exportedAt:'2025-10-19T08:12:00+02:00',appVersion:'0.1.30',from:'2024-11-04',to:'2025-11-04',rows:[
 {kind:'Goals',added:2,replaced:1,kept:0},
 {kind:'Your sessions',added:3,replaced:0,kept:1},
 {kind:'Plan links',added:4,replaced:0,kept:2},
 {kind:'Rest, pauses and limits',added:5,replaced:1,kept:3},
 {kind:'Plan-week prompts',added:6,replaced:0,kept:2},
 {kind:'Events',added:1,replaced:0,kept:0},
 {kind:'Routines',added:2,replaced:0,kept:4},
 {kind:'Weekly plan history',added:9,replaced:0,kept:2},
 {kind:'Sleep plan snapshots',added:14,replaced:0,kept:0},
 {kind:'Plan prompt preference',added:0,replaced:0,kept:1}],skipped:[
 {what:'Garmin data and journal logs',reason:"Not imported: a sync downloads Garmin data for this phone, and journal reads stay this phone's own choice."},
 {what:'Milestones and records',reason:"Not imported: they are awarded again from this phone's Garmin recordings."},
 {what:'Notification settings',reason:'Not imported: this phone keeps its own choices.'}]};
export const choose_import = ():Promise<ImportPreview> => call('choose_import',undefined,sampleImport);
export const apply_import = ():Promise<ImportPreview> => call('apply_import',undefined,sampleImport);
export const request_notification_permission = ():Promise<boolean> => call('request_notification_permission',undefined,true);
export const notification_permission = ():Promise<boolean|null> => call('notification_permission',undefined,new URLSearchParams(location.search).get('fixture')!=='notifications-blocked');
// Desktop re-reads due reminders on resume and after a settings change.
export const check_reminders = ():Promise<void> => call('check_reminders',undefined,undefined);
export const reminders_status = ():Promise<ReminderStatus|null> => call('reminders_status',undefined,null);
export const get_settings = ():Promise<Settings> => call('get_settings',undefined,previewSettings);
export const patch_settings = (patch:SettingsPatch):Promise<void> => {if(!isTauri()){previewSettings={...previewSettings,...copy(patch)};return Promise.resolve()}return invoke<void>('patch_settings',{patch})};
export const onStatusChanged = (callback:()=>void):Promise<()=>void> => isTauri()?listen('status-changed',callback):Promise.resolve(()=>{});
