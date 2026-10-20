import type {SleepPlan} from './generated/SleepPlan';
import type {TodayView} from './generated/TodayView';

// Plan and phase times are local wall-clock ISO strings, which Date reads as local time.
const local=(iso:string|null)=>iso?new Date(iso).getTime():NaN;
const HOUR=3600_000;

// The usual wake is credible from four recorded wakes in 14 nights (spec Rules: Time of day).
export const wakeLabel=(nights:number)=>nights>=14?'usual':nights>=4?`usual · ${nights} nights`:'reference';

// Where now sits against wind-down and the window: positions in percent on a whole-hour axis.
export function nightStrip(plan:SleepPlan, now:Date){
 const windDown=local(plan.windDownAt),start=local(plan.bedtimeStartAt??plan.bedtimeAt),end=local(plan.bedtimeAt);
 if(![windDown,start,end].every(Number.isFinite))return null;
 const from=Math.floor(Math.min(windDown,start-2*HOUR)/HOUR)*HOUR,to=Math.ceil((end+HOUR)/HOUR)*HOUR;
 const at=(t:number)=>(t-from)/(to-from)*100;
 const ticks=Array.from({length:(to-from)/HOUR+1},(_,i)=>{const t=from+i*HOUR;return {left:at(t),label:String(new Date(t).getHours()).padStart(2,'0')}});
 const n=now.getTime();
 return {windDown:at(windDown),start:at(start),end:at(end),now:n>=from && n<=to?at(n):null,ticks};
}

// Tonight from an hour before wind-down to the usual wake: the need is a span ending at
// that wake (the same subtraction as the asleep-by estimate). Ticks every two hours
// on even clock hours keep a ten-hour axis readable on a phone.
export function tonightLine(plan:SleepPlan, now:Date){
 const windDown=local(plan.windDownAt),start=local(plan.bedtimeStartAt??plan.bedtimeAt),end=local(plan.bedtimeAt);
 if(![windDown,start,end].every(Number.isFinite))return null;
 let wake=NaN;
 if(plan.wakeTime){
  const [h,m]=plan.wakeTime.split(':').map(Number),d=new Date(end);
  d.setHours(h,m,0,0);
  if(d.getTime()<=end)d.setDate(d.getDate()+1);
  wake=d.getTime();
 }
 const known=Number.isFinite(wake);
 const from=Math.floor((Math.min(windDown,start)-HOUR)/HOUR)*HOUR,to=Math.ceil(((known?wake:end)+HOUR/2)/HOUR)*HOUR;
 const at=(t:number)=>(t-from)/(to-from)*100;
 const ticks=[];
 for(let t=from;t<=to;t+=HOUR){const h=new Date(t).getHours();if(h%2===0)ticks.push({left:at(t),label:String(h).padStart(2,'0')})}
 const n=now.getTime();
 return {windDown:at(windDown),start:at(start),end:at(end),wake:known?at(wake):null,needFrom:known?at(Math.max(from,wake-plan.needMin*60_000)):null,now:n>=from && n<=to?at(n):null,ticks};
}

// A cached Today is stale once its phase has ended; reading it again re-evaluates the phase.
export function phaseEnded(view:TodayView, now=Date.now()):boolean {
 const until=local(view.focus.until);
 return Number.isFinite(until) && until<=now;
}
