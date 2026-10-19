import { invoke, isTauri } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
// data.ts is generated alongside these JSON fixtures and checks each value with `satisfies`.
import sampleToday from './sample/today.json';
import sampleSleep from './sample/sleep.json';
import sampleTraining from './sample/training.json';
import sampleWorkout from './sample/workout.json';
import sampleStrengthWorkout from './sample/strengthWorkout.json';
import sampleStrengthWorkoutHistory from './sample/strengthWorkoutHistory.json';
import sampleStrength from './sample/strength.json';
import sampleHealth from './sample/health.json';
import type {HeartRateDay} from './generated/HeartRateDay';
import sampleCheckin from './sample/checkin.json';
import sampleStatus from './sample/status.json';
import sampleBoard from './sample/board.json';
import sampleReview from './sample/review.json';
import sampleReviews from './sample/reviews.json';
import type { AppStatus } from './generated/AppStatus';
import type { TodayView } from './generated/TodayView';
import type { SleepView } from './generated/SleepView';
import type { TrainingView } from './generated/TrainingView';
import type { WorkoutView } from './generated/WorkoutView';
import type { StrengthView } from './generated/StrengthView';
import type { HealthView } from './generated/HealthView';
import type { CheckinView } from './generated/CheckinView';
import type { Settings } from './generated/Settings';
import type { TrendQuery } from './generated/TrendQuery';
import type { TrendView } from './generated/TrendView';
import { sampleTrend } from './sample/trend';
import type { LocalAction } from './generated/LocalAction';
import type { PlanBoard } from './generated/PlanBoard';
import type { WeekReview } from './generated/WeekReview';
import type { ReviewRow } from './generated/ReviewRow';

export type { Settings } from './generated/Settings';
export type SettingsPatch = Partial<Settings>;
export const supportsStatusSurfaces = (isTauri() && /Android/.test(navigator.userAgent)) || new URLSearchParams(location.search).get('fixture')==='status-offer';
export const defaultSettings = {notifyMorningReadiness:true,notifyWeeklyCheckin:true,notifyMetricChanges:true,notifyVitals:true,notifyTrainingStatus:false,notifyBedtime:false,notifyStatus:false,statusOfferSeen:false,bedtimeLeadMin:30,adaptiveToday:true,journalEnabled:false,planPrompt:true} satisfies Settings;
let previewSettings:Settings = {...defaultSettings};
const copy = <T>(value:T):T => structuredClone(value);
const call = <T>(name:string,args:Record<string,unknown>|undefined,fallback:T):Promise<T> => isTauri()?invoke<T>(name,args):Promise.resolve(copy(fallback));
export const app_status = ():Promise<AppStatus> => call('app_status',undefined,sampleStatus as AppStatus);
export const today = (date?:string):Promise<TodayView> => call('today',{date},sampleToday as unknown as TodayView);
export const sleep = (date?:string):Promise<SleepView> => call('sleep',{date},sampleSleep as unknown as SleepView);
export const training = (date?:string):Promise<TrainingView> => call('training',{date},sampleTraining as unknown as TrainingView);
export const workout = (id:string):Promise<WorkoutView> => {
 if(isTauri())return invoke<WorkoutView>('workout',{id});
 const history=sampleStrengthWorkoutHistory as unknown as Record<string,WorkoutView>;
 const listed=sampleTraining.workouts.find(a=>a.id===id);
 // Other listed preview activities keep their own summary, never a run's charts.
 const summaryOnly=listed?{summary:listed,recoveryHours:null,stats:[],route:[],heartRate:[],zoneFloors:[],splits:[],strength:null}:null;
 const value=id===sampleStrengthWorkout.summary.id?sampleStrengthWorkout:id===sampleWorkout.summary.id?sampleWorkout:Object.hasOwn(history,id)?history[id]:summaryOnly;
 return value?Promise.resolve(copy(value as unknown as WorkoutView)):Promise.reject(new Error('No preview workout for this ID'));
};
export const strength = (date?:string):Promise<StrengthView|null> => call('strength',{date},sampleStrength as unknown as StrengthView);
export const health = (date?:string):Promise<HealthView> => isTauri()?invoke<HealthView>('health',{date}):Promise.resolve({...copy(sampleHealth as unknown as HealthView),journalEnabled:previewSettings.journalEnabled});
export const heart_rate_day = (date:string):Promise<HeartRateDay> => isTauri()?invoke<HeartRateDay>('heart_rate_day',{date}):Promise.resolve({...((sampleHealth as unknown as HealthView).heartRateDay),date});
export const checkin = (weekStart?:string):Promise<CheckinView> => call('checkin',{weekStart},sampleCheckin as unknown as CheckinView);
export const trend = (query:TrendQuery):Promise<TrendView> => isTauri()?invoke<TrendView>('trend',{query}):Promise.resolve().then(()=>sampleTrend(query));
// The preview keeps local changes in memory; only the Rust core applies the plan rules.
let previewBoard:PlanBoard = copy(sampleBoard as unknown as PlanBoard);
export const plan_board = (date?:string):Promise<PlanBoard> => call('plan_board',{date},previewBoard);
export const week_review = (date:string):Promise<WeekReview> => call('week_review',{date},sampleReview as unknown as WeekReview);
export const reviews = ():Promise<ReviewRow[]> => call('reviews',undefined,sampleReviews as unknown as ReviewRow[]);
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
export const sync_now = ():Promise<void> => call('sync_now',undefined,undefined);
export const open_garmin_connect = ():Promise<boolean> => call('open_garmin_connect',undefined,false);
export const begin_sign_in = ():Promise<void> => call('begin_sign_in',undefined,undefined);
export const sign_out = ():Promise<void> => call('sign_out',undefined,undefined);
export const export_archive = ():Promise<string> => call('export_archive',undefined,'Preview: no file created');
export const request_notification_permission = ():Promise<boolean> => call('request_notification_permission',undefined,true);
export const get_settings = ():Promise<Settings> => call('get_settings',undefined,previewSettings);
export const patch_settings = (patch:SettingsPatch):Promise<void> => {if(!isTauri()){previewSettings={...previewSettings,...copy(patch)};return Promise.resolve()}return invoke<void>('patch_settings',{patch})};
export const onStatusChanged = (callback:()=>void):Promise<()=>void> => isTauri()?listen('status-changed',callback):Promise.resolve(()=>{});
