import type {DistanceUnit} from './generated/DistanceUnit';
import {elapsed} from './format';
// One formatter for activity distance, pace, speed and elevation. Values arrive
// in kilometres, seconds per km, km/h and metres; the activity's unit decides
// how they read (miles, feet and mph for statute; per 100 m or yd for swims).
const KM_PER_MI=1.609344,KM_PER_YD=0.0009144,FT_PER_M=3.28084;
const trim=(v:number,digits:number)=>v.toFixed(digits).replace(/\.?0+$/,'');
export const statute=(u:DistanceUnit)=>u==='mi'||u==='yd';
export const swimUnit=(u:DistanceUnit)=>u==='m'||u==='yd';
export const distance=(km:number|null|undefined,u:DistanceUnit)=>{
 if(km==null)return '—';
 if(u==='m')return `${Math.round(km*1000)} m`;
 if(u==='yd')return `${Math.round(km/KM_PER_YD)} yd`;
 return u==='mi'?`${trim(km/KM_PER_MI,2)} mi`:`${trim(km,2)} km`;
};
export const paceUnit=(u:DistanceUnit)=>({km:'/km',mi:'/mi',m:'/100 m',yd:'/100 yd'})[u];
export const paceSeconds=(secondsPerKm:number,u:DistanceUnit)=>({km:1,mi:KM_PER_MI,m:.1,yd:KM_PER_YD*100})[u]*secondsPerKm;
export const pace=(secondsPerKm:number|null|undefined,u:DistanceUnit)=>secondsPerKm==null?'—':elapsed(paceSeconds(secondsPerKm,u));
export const speedUnit=(u:DistanceUnit)=>statute(u)?'mph':'km/h';
export const speed=(kmh:number,u:DistanceUnit)=>statute(u)?kmh/KM_PER_MI:kmh;
export const elevationUnit=(u:DistanceUnit)=>statute(u)?'ft':'m';
export const elevation=(m:number,u:DistanceUnit)=>statute(u)?m*FT_PER_M:m;
