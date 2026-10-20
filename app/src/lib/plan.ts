import type {PlanTargetKind} from './generated/PlanTargetKind';
import type {PlanState} from './generated/PlanState';
import type {IconName} from './components/Icon.svelte';
import {colors} from './charts';
import {weekdays,monthName} from './format';

export const planNames:Record<PlanTargetKind,string>={strength:'Strength',intensity:'Intensity',zone2:'Zone 2',asleepOnTime:'Asleep on time'};
export const planUnits:Record<PlanTargetKind,string>={strength:'sessions',intensity:'min',zone2:'min',asleepOnTime:'nights'};
export const planIcons:Record<PlanTargetKind,IconName>={strength:'lift',intensity:'flame',zone2:'pulse',asleepOnTime:'moon'};
// Ink on track, amber behind pace, green when done.
export const planColor=(state:PlanState)=>state==='done'?colors.good:state==='behind'?colors.fair:colors.ink;
export {weekdays};
/** Weekday indices (0 = Monday) in the order of a week starting on `first`. */
export const weekOrder=(first:number)=>Array.from({length:7},(_,i)=>(first+i)%7);
export const weekRange=(start:string,end:string)=>{
 const day=(d:string)=>new Date(`${d}T12:00:00`);
 const a=day(start),b=day(end);
 if(Number.isNaN(a.getTime())||Number.isNaN(b.getTime()))return '';
 const short=(d:Date,month:boolean)=>`${weekdays[(d.getDay()+6)%7]} ${d.getDate()}${month?' '+monthName(d.getMonth()):''}`;
 return `${short(a,a.getMonth()!==b.getMonth())} – ${short(b,true)}`;
};
/** "▲ 12", "▼ 1" or "same" against the previous week. */
export const delta=(value:number,previous:number|null)=>previous==null?'':value===previous?'same':`${value>previous?'▲':'▼'} ${Math.round(Math.abs(value-previous))}`;
