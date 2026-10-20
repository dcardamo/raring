import type {SleepPlan} from './generated/SleepPlan';
import type {NightWindow} from './generated/NightWindow';

// Plan times are local wall-clock ISO strings, which Date reads as local time.
const local=(iso:string|null|undefined)=>iso?new Date(iso).getTime():NaN;
const hours=(t:number)=>{const d=new Date(t);return d.getHours()+d.getMinutes()/60};

/** Degrees clockwise from midnight at the top of a 24-hour ring. */
export const clockAngle=(t:number)=>hours(t)/24*360;

export type Arc={from:number,to:number};
export type SleepClock={windDown:Arc|null,fallAsleep:Arc|null,sleep:Arc,window:Arc|null,now:number};

/** Tonight's plan on a 24-hour ring: wind-down, the minutes to fall asleep,
 * the planned sleep to the expected wake, Garmin's window and now. Arcs run
 * clockwise; `to` may pass 360 when an arc crosses midnight. */
export function sleepClock(plan:SleepPlan, now:Date):SleepClock|null{
 const windDown=local(plan.windDownAt),inBed=local(plan.inBedAt||plan.bedtimeAt),asleep=local(plan.bedtimeAt),wake=local(plan.wakeAt);
 if(![windDown,inBed,asleep,wake].every(Number.isFinite)||wake<=asleep)return null;
 const arc=(a:number,b:number):Arc=>{const from=clockAngle(a);let to=from+(b-a)/3600_000*15;if(to-from>=360)to=from+359.9;return {from,to}};
 const start=local(plan.bedtimeStartAt);
 return {
  windDown:inBed>windDown?arc(windDown,inBed):null,
  fallAsleep:asleep>inBed?arc(inBed,asleep):null,
  sleep:arc(asleep,wake),
  window:Number.isFinite(start)&&asleep>start?arc(start,asleep):null,
  now:clockAngle(now.getTime()),
 };
}

/** An SVG arc path on a ring of radius r around (c,c), angles in degrees from the top. */
export function arcPath(c:number,r:number,a:Arc):string{
 const p=(deg:number)=>{const rad=(deg-90)*Math.PI/180;return `${(c+r*Math.cos(rad)).toFixed(2)} ${(c+r*Math.sin(rad)).toFixed(2)}`};
 const large=a.to-a.from>180?1:0;
 return `M${p(a.from)}A${r} ${r} 0 ${large} 1 ${p(a.to)}`;
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
