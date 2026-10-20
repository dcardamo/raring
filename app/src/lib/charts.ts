// Semantic palette, mirrored in app.css and checked by design-tokens.test.ts.
export const colors = {good:'#60D3A0',fair:'#F2B85E',poor:'#F17E78',amount:'#76ACEE',neutral:'#76ACEE',ink:'#F4F1EA',secondary:'#C2C8D1',deep:'#6795D1',light:'#83AFE8',rem:'#BDCEF0',awake:'#F4F1EA',z:['#A1ABB9','#76ACEE','#EFC96C','#F09B63','#F17E78']} as const;
export const goalColors = {strength:'#EC91BF',zone2:colors.amount,highIntensity:'#F09B63',intensity:'#58C7BD',sleep:'#AEC7ED'} as const;
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
