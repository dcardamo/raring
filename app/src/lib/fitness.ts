import {colors} from './charts';
import {num} from './format';
import type {Driver} from './generated/Driver';

export const fitnessAgeColor=(age:number,actual:number|null)=>actual==null?colors.ink:age<actual?colors.good:age===actual?colors.fair:colors.poor;
export const driverScale=(key:string):[number,number]=>({vo2max:[20,70],restingHr:[35,85],bmi:[15,35],bodyFat:[5,40],vigorousMinutes:[0,200],vigorousDays:[0,7]} as Record<string,[number,number]>)[key]??[0,100];
export const driverRange=(key:string,target:number|null):[number,number]|null=>target==null?null:key==='bmi'?[18.5,target]:lowerIsBetter(key)?[driverScale(key)[0],target]:[target,driverScale(key)[1]];
export const lowerIsBetter=(key:string)=>['restingHr','bmi','bodyFat'].includes(key);
const digits=(key:string)=>key==='bmi'||key==='vigorousDays'?1:0;
const amount=(d:Driver,v:number)=>`${num(v,digits(d.key))}${d.unit?' '+d.unit:''}`;
/** Garmin's target in words: "Target ≥ 150 min", "Target ≤ 54 bpm". */
export const driverTarget=(d:Driver)=>d.target==null?'':`Target ${lowerIsBetter(d.key)?'≤':'≥'} ${amount(d,d.target)}`;
/** How far an unmet driver is from Garmin's target, without trend arrows. */
export const driverGap=(d:Driver)=>d.value==null||d.target==null?'':lowerIsBetter(d.key)?`${amount(d,Math.max(0,d.value-d.target))} over`:`${amount(d,Math.max(0,d.target-d.value))} to go`;
