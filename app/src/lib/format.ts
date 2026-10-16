export const duration=(minutes:number|null|undefined)=>{if(minutes==null)return '—';const m=Math.round(minutes);return `${Math.floor(m/60)}:${String(m%60).padStart(2,'0')}`};
export const elapsed=(seconds:number|null|undefined)=>{if(seconds==null)return '—';const s=Math.round(seconds);return s>=3600?`${Math.floor(s/3600)}:${String(Math.floor(s%3600/60)).padStart(2,'0')}:${String(s%60).padStart(2,'0')}`:`${Math.floor(s/60)}:${String(s%60).padStart(2,'0')}`};
export const pace=(seconds:number|null|undefined)=>seconds==null?'—':`${elapsed(seconds)}/km`;
export const clock=(iso:string|null|undefined)=>iso?iso.slice(11,16):'—';
export const dateLabel=(date:string)=>new Date(`${date}T12:00:00`).toLocaleDateString('en-GB',{weekday:'short',day:'numeric',month:'short'});
export const signed=(value:number|null|undefined,unit='')=>value==null?'—':`${value>0?'+':value<0?'−':''}${num(Math.abs(value),1)}${unit}`;
export const num=(value:number|null|undefined,precision=0)=>value==null?'—':value.toFixed(precision).replace(/\.0+$/,'');
