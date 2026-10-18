// Local prototype goals and allocations; no production storage or Garmin writes.
import {between, addDays, dayLabel} from './dates';
import {versionFor, type Ctx, type GoalTargets, type Intention, type Allocation} from './rules';
import type {Activity} from './world';
export type GoalKey = 'strength' | 'zone2' | 'im' | 'sleep';
export const GOAL_INFO = {strength:{name:'Strength',unit:'sessions',color:'#FF7DB8'},zone2:{name:'Zone 2',unit:'min',color:'#5CA8FF'},im:{name:'Intensity',unit:'min',color:'#36C9C6'},sleep:{name:'Sleep timing',unit:'nights',color:'#A9C8FF'}} as const;
export const emptyGoals = (): GoalTargets => ({strength:0,zone2:0,im:0,sleep:0,sleepTime:'22:15',sleepKind:'winddown'});
export function goalsFor(ctx: Ctx, date = ctx.calendarDate): GoalTargets {
 const accepted=versionFor({state:ctx.state,calendarDate:date});
 const base=accepted.revisions.at(-1) ?? accepted.base;
 const local=(ctx.state.goalVersions??[]).filter(v=>v.from<=date).at(-1);
 return local ? local.targets : {...emptyGoals(),strength:base?.strength??0,im:base?.im??0};
}
export const isOpen = (i:Intention) => !i.status || i.status==='open';
export const allocationFor = (i:Intention):Allocation => i.allocation ?? (i.sport==='strength' && i.kind!=='sleep' ? {strength:1} : {});
export function candidates(ctx:Ctx, intention:Intention):Activity[] {
 if(intention.kind==='sleep')return [];
 return ctx.world.activities.filter(a=>a.date===intention.date && a.sport===intention.sport);
}
export function intentionState(ctx:Ctx,i:Intention):'recorded'|'released'|'missed'|'planned'|'not-recorded'|'awaiting'|'sleep-observation' {
 if(i.status==='released'||i.status==='missed')return i.status;
 if(i.kind==='sleep' && i.date<ctx.calendarDate)return 'sleep-observation';
 if(i.link && candidates(ctx,i).some(a=>a.id===i.link))return 'recorded';
 if(i.date>=ctx.calendarDate)return 'planned';
 return ctx.world.coverageIncompleteFrom && i.date>=ctx.world.coverageIncompleteFrom || ctx.world.coverageGaps.includes(weekIdFor(i.date)) ? 'awaiting' : 'not-recorded';
}
import {weekId as weekIdFor} from './dates';
// One factual contribution per recording. Matching an intention never adds a
// second workout, and planned duration is never substituted for zone evidence.
export function recordedFor(activities:Activity[],key:GoalKey):number {
 const unique=[...new Map(activities.map(a=>[a.id,a])).values()];
 if(key==='strength')return unique.filter(a=>a.sport==='strength').length;
 if(key==='zone2')return Math.floor(unique.reduce((n,a)=>n+(a.zones[1]??0),0)/60);
 if(key==='im')return unique.reduce((n,a)=>n+a.im,0);
 return 0;
}
export function allocations(ctx:Ctx) {
 const goals=goalsFor(ctx), inWeek=ctx.world.activities.filter(a=>between(a.date,ctx.weekFrom,ctx.calendarDate));
 const dayList=Array.from({length:Math.max(1,Math.min(7,Math.round((Date.parse(ctx.calendarDate)-Date.parse(ctx.weekFrom))/86400000)+1))},(_,i)=>addDays(ctx.weekFrom,i));
 const complete=!dayList.some(d=>ctx.world.coverageGaps.includes(weekIdFor(d)) || !!ctx.world.coverageIncompleteFrom && d>=ctx.world.coverageIncompleteFrom);
 const pending=ctx.state.intentions.filter(i=>isOpen(i)&&!i.link&&between(i.date,ctx.calendarDate,ctx.weekTo));
 return (Object.keys(GOAL_INFO) as GoalKey[]).filter(key=>goals[key]>0).map(key=>{
  const relevant=pending.filter(i=>(allocationFor(i)[key]??0)>0);
  const ambiguous=key!=='sleep' && relevant.some(i=>candidates(ctx,i).length>0);
  const planned=relevant.reduce((n,i)=>n+(allocationFor(i)[key]??0),0);
  const recorded=key==='sleep'?null:recordedFor(inWeek,key);
  const paused=!!ctx.paused||!!ctx.plan?.midPause||!!ctx.plan?.wholePause;
  const remaining=key==='sleep'?Math.max(0,goals.sleep-ctx.state.intentions.filter(i=>isOpen(i)&&i.kind==='sleep'&&between(i.date,ctx.weekFrom,ctx.weekTo)).length):!complete||ambiguous||paused?null:Math.max(0,goals[key]-(recorded??0)-planned);
  return {key,...GOAL_INFO[key],target:goals[key],recorded,planned,remaining,complete,ambiguous,paused};
 });
}
export function movedIntention(i:Intention,date:string,when:string):Intention {
 return {...i,date,link:null,status:'open',history:[...i.history,[when,`Moved from ${dayLabel(i.date)} to ${dayLabel(date)}${i.garminId?' · Garmin unchanged':''}`]]};
}
export function dayEvidence(ctx:Ctx,date:string) {
 const goals=goalsFor(ctx,date), acts=ctx.world.activities.filter(a=>a.date===date);
 const plans=ctx.state.intentions.filter(i=>isOpen(i)&&i.date===date);
 const future=date>ctx.calendarDate;
 const unknown=!future&&(date>ctx.upload.date||!!ctx.world.coverageIncompleteFrom&&date>=ctx.world.coverageIncompleteFrom||ctx.world.coverageGaps.includes(weekIdFor(date)));
 const night=ctx.nightOf(date);
 const cardioKey:GoalKey=goals.zone2>0?'zone2':'im';
 const allocated=(key:GoalKey)=>plans.reduce((n,i)=>n+(allocationFor(i)[key]??0),0);
 const cardio=goals.zone2>0?recordedFor(acts,'zone2'):acts.filter(a=>a.sport!=='strength'&&a.sport!=='yoga').reduce((n,a)=>n+a.min,0);
 return {date,night,acts,plans,future,unknown,sleepPending:date>ctx.displayedDay,
  cardio,cardioLabel:goals.zone2>0?'Zone 2 minutes':'Cardio minutes',cardioTarget:goals.zone2>0?allocated(cardioKey):plans.filter(i=>i.sport!=='strength'&&i.kind!=='sleep').reduce((n,i)=>n+(i.durationMin??0),0),
  strength:recordedFor(acts,'strength'),strengthTarget:allocated('strength'),
  vitals:future||!night?'unknown':ctx.state.vitalsException&&date===ctx.displayedDay?'unusual':'usual',
  rest:ctx.state.rest?.day===date,
  paused:[...ctx.state.pauses,...(ctx.state.pause?[ctx.state.pause]:[])].some(p=>date>=p.since&&(!p.until||date<=p.until))};
}
