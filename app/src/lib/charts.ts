export const colors = {mint:'#3DDC97',amber:'#FFB547',coral:'#FF6B6B',violet:'#9B8CFF',flame:'#FF8A3D',sky:'#5CC8FF',deep:'#5B4BE0',rem:'#B79CFF',light:'#7C93FF',awake:'#FF9B8F',z:['#7D879C','#5CC8FF','#3DDC97','#FFB547','#FF6B6B']} as const;
export const toneColor=(tone:string)=>tone==='poor'?colors.coral:tone==='fair'?colors.amber:tone==='neutral'?'#A3ACBF':colors.mint;
export const clamp=(n:number)=>Math.max(0,Math.min(1,Number.isFinite(n)?n:0));
export function line(values:(number|null)[],w:number,h:number,min?:number,max?:number){const present=values.filter((v):v is number=>v!=null && Number.isFinite(v));if(!present.length)return '';const low=min??Math.min(...present),high=max??Math.max(...present),range=high-low||1;let d='';values.forEach((v,i)=>{if(v==null || !Number.isFinite(v))return;d+=`${d && values[i-1]!=null?'L':'M'}${(4+i*(w-8)/Math.max(values.length-1,1)).toFixed(1)},${(h-4-(v-low)/range*(h-8)).toFixed(1)}`});return d}
export const position=(value:number,scale:[number,number])=>clamp((value-scale[0])/(scale[1]-scale[0]||1))*100;
export const stressColors:Record<string,string>={rest:colors.sky,low:colors.mint,medium:colors.amber,high:colors.coral};
export const stressNames:Record<string,string>={rest:'Rest',low:'Low',medium:'Medium',high:'High'};
// Garmin's category bounds, 0–100.
export const stressLevel=(value:number)=>value<26?'rest':value<51?'low':value<76?'medium':'high';
