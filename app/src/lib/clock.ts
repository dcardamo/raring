import type {SleepPlan} from './generated/SleepPlan';
import type {NightWindow} from './generated/NightWindow';

// Plan times are local wall-clock ISO strings, which Date reads as local time.
const local=(iso:string|null|undefined)=>iso?new Date(iso).getTime():NaN;
const hours=(t:number)=>{const d=new Date(t);return d.getHours()+d.getMinutes()/60};

export type Span={left:number,width:number};
export type SleepEvening={from:number,to:number,windDown:Span|null,fallAsleep:Span|null,window:Span|null,now:number,ticks:{h:number,at:number}[]};

const HALF=30*60_000;
/** Tonight's evening as one left-to-right axis, in percentages of its width:
 * from half an hour before now or wind-down, whichever is earlier, to half an
 * hour past the asleep-by time. The night's length is text, not scale, so the
 * evening keeps the room its decisions need; a position, never a countdown. */
export function sleepEvening(plan:SleepPlan, now:Date):SleepEvening|null{
 const windDown=local(plan.windDownAt),inBed=local(plan.inBedAt||plan.bedtimeAt),asleep=local(plan.bedtimeAt),nowAt=now.getTime();
 if(![windDown,inBed,asleep,nowAt].every(Number.isFinite)||asleep<inBed)return null;
 const start=Math.floor((Math.min(nowAt,windDown)-HALF)/HALF)*HALF,end=Math.ceil((asleep+HALF)/HALF)*HALF;
 if(end-start<HALF)return null;
 const at=(t:number)=>Math.max(0,Math.min(100,(t-start)/(end-start)*100));
 const span=(a:number,b:number):Span|null=>b>a?{left:at(a),width:at(b)-at(a)}:null;
 const win=local(plan.bedtimeStartAt),ticks:{h:number,at:number}[]=[];
 for(let t=Math.ceil(start/3_600_000)*3_600_000;t<=end;t+=3_600_000)ticks.push({h:new Date(t).getHours(),at:at(t)});
 return {
  from:hours(start),to:hours(end),
  windDown:span(windDown,inBed),
  fallAsleep:span(inBed,asleep),
  window:Number.isFinite(win)?span(win,asleep):null,
  now:at(nowAt),
  ticks,
 };
}

/** Evening-relative hours: after noon stays, before noon is the next morning (+24). */
const evening=(h:number)=>h<12?h+24:h;
const day=(date:string,back:number)=>new Date(Date.parse(date+'T12:00:00Z')-back*864e5).toISOString().slice(0,10);

export type RecentNight={date:string,bed:number|null,wake:number|null};
export type RecentNights={nights:RecentNight[],top:number,bottom:number,band:[number,number]|null,wake:number|null,ticks:number[]};

/** The 14 nights ending `date` on a clock axis from the evening (top) to the
 * morning, against tonight's planned bedtime band and wake. Missing nights stay
 * in the list with no times. */
export function recentNights(date:string, windows:NightWindow[], plan:SleepPlan|null):RecentNights{
 const byDate=new Map(windows.map(w=>[w.date,w]));
 const nights=Array.from({length:14},(_,i)=>{const d=day(date,13-i),w=byDate.get(d);return {date:d,bed:w?evening(w.bedHour):null,wake:w?evening(w.wakeHour):null}});
 const planned=(iso:string|null|undefined)=>{const t=local(iso);return Number.isFinite(t)?evening(hours(t)):null};
 const bandFrom=plan?Math.min(...[plan.bedtimeStartAt,plan.inBedAt].map(planned).filter((v):v is number=>v!=null)):Infinity;
 const bandTo=plan?planned(plan.bedtimeAt):null;
 const band:[number,number]|null=Number.isFinite(bandFrom)&&bandTo!=null&&bandTo>=bandFrom?[bandFrom,bandTo]:null;
 const wake=plan?planned(plan.wakeAt):null;
 const values=[...nights.flatMap(n=>n.bed!=null&&n.wake!=null&&n.wake>n.bed?[n.bed,n.wake]:[]),...(band??[]),...(wake!=null?[wake]:[])];
 const top=values.length?Math.floor(Math.min(...values)/2)*2:20,bottom=values.length?Math.ceil(Math.max(...values)/2)*2:32;
 const ticks=Array.from({length:(bottom-top)/2+1},(_,i)=>top+i*2);
 return {nights,top,bottom:Math.max(bottom,top+2),band,wake,ticks};
}

export const clockLabel=(h:number)=>String(((Math.round(h)%24)+24)%24).padStart(2,'0');
