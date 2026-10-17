export const colors = {good:'#3DDC97',fair:'#FFB547',poor:'#FF6B6B',amount:'#5CA8FF',ink:'#E4E8F0',secondary:'#B4BCCB',deep:'#2F6FE0',light:'#5C9BFF',rem:'#A9C8FF',awake:'#E4E8F0',z:['#7D879C','#5CA8FF','#FFD666','#FF9F43','#FF6B6B']} as const;
export const toneColor=(tone:string)=>tone==='poor'?colors.poor:tone==='fair'?colors.fair:tone==='neutral'?colors.secondary:colors.good;
// Body Battery has no tone in the view contract; only its display color is bucketed.
export const batteryColor=(value:number)=>value>=70?colors.good:value>=30?colors.fair:colors.poor;
export const clamp=(n:number)=>Math.max(0,Math.min(1,Number.isFinite(n)?n:0));
export function line(values:(number|null)[],w:number,h:number,min?:number,max?:number){const present=values.filter((v):v is number=>v!=null && Number.isFinite(v));if(!present.length)return '';const low=min??Math.min(...present),high=max??Math.max(...present),range=high-low||1;let d='';values.forEach((v,i)=>{if(v==null || !Number.isFinite(v))return;d+=`${d && values[i-1]!=null?'L':'M'}${(4+i*(w-8)/Math.max(values.length-1,1)).toFixed(1)},${(h-4-(v-low)/range*(h-8)).toFixed(1)}`});return d}
export const position=(value:number,scale:[number,number])=>clamp((value-scale[0])/(scale[1]-scale[0]||1))*100;
export const stressColors:Record<string,string>={rest:colors.secondary,low:colors.good,medium:colors.fair,high:colors.poor};
export const stressNames:Record<string,string>={rest:'Rest',low:'Low',medium:'Medium',high:'High'};
// Garmin's category bounds, 0–100.
export const stressLevel=(value:number)=>value<26?'rest':value<51?'low':value<76?'medium':'high';
