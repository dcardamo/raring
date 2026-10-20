export const duration=(minutes:number|null|undefined)=>{if(minutes==null)return '—';const m=Math.round(minutes);return `${Math.floor(m/60)}:${String(m%60).padStart(2,'0')}`};
export const elapsed=(seconds:number|null|undefined)=>{if(seconds==null)return '—';const s=Math.round(seconds);return s>=3600?`${Math.floor(s/3600)}:${String(Math.floor(s%3600/60)).padStart(2,'0')}:${String(s%60).padStart(2,'0')}`:`${Math.floor(s/60)}:${String(s%60).padStart(2,'0')}`};
export const pace=(seconds:number|null|undefined)=>seconds==null?'—':`${elapsed(seconds)}/km`;
// Garmin local timestamps are wall-clock times; sync timestamps include an offset.
export const clock=(iso:string|null|undefined)=>{
 if(!iso)return '—';
 if(/(?:Z|[+-]\d{2}:?\d{2})$/i.test(iso)){
  const date=new Date(iso);
  return Number.isNaN(date.getTime())?'—':date.toLocaleTimeString('en-GB',{hour:'2-digit',minute:'2-digit'});
 }
 return /^\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}/.test(iso)?iso.slice(11,16):'—';
};
export const weekdays=['Mon','Tue','Wed','Thu','Fri','Sat','Sun'];
// Built by hand: Android's WebView puts a comma after the weekday in en-GB
// dates. en-GB's short month is "Sept", not "Sep".
const months=['Jan','Feb','Mar','Apr','May','Jun','Jul','Aug','Sept','Oct','Nov','Dec'];
const calendarDay=(date:string)=>{
 const day=new Date(`${date.slice(0,10)}T12:00:00`);
 return Number.isNaN(day.getTime())?null:day;
};
export const monthName=(month:number)=>months[month];
export const shortMonth=(date:string)=>{
 const day=calendarDay(date);
 return day?months[day.getMonth()]:'';
};
export const dateLabel=(date:string)=>{
 const day=calendarDay(date);
 return day?`${weekdays[(day.getDay()+6)%7]} ${day.getDate()} ${months[day.getMonth()]}`:'Date unavailable';
};
// An offset timestamp as the device's local date and clock, so both halves agree.
export const localDateTime=(iso:string|null|undefined)=>{
 if(!iso||!/(?:Z|[+-]\d{2}:?\d{2})$/i.test(iso))return iso?`${dateLabel(iso)} · ${clock(iso)}`:'—';
 const at=new Date(iso);
 if(Number.isNaN(at.getTime()))return '—';
 return `${weekdays[(at.getDay()+6)%7]} ${at.getDate()} ${months[at.getMonth()]} · ${clock(iso)}`;
};
// One narrow weekday letter for each of the seven days from a week's first date.
export const weekLetters=(start:string)=>{
 const first=calendarDay(start);
 return Array.from({length:7},(_,i)=>first?'MTWTFSS'[(first.getDay()+6+i)%7]:'');
};
export const monthLabel=(date:string)=>{
 const day=calendarDay(date);
 return day?`${months[day.getMonth()]} ${day.getFullYear()}`:'Date unavailable';
};
export const signed=(value:number|null|undefined,unit='')=>value==null?'—':Math.abs(value)<1?'':`${value>0?'+':'−'}${num(Math.abs(value),1)}${unit}`;
export const num=(value:number|null|undefined,precision=0)=>value==null?'—':value.toFixed(precision).replace(/\.0+$/,'');

// Minutes from local midnight as a 24-hour clock time.
export const minuteClock=(m:number)=>`${String(Math.floor(m/60)%24).padStart(2,'0')}:${String(Math.floor(m%60)).padStart(2,'0')}`;
