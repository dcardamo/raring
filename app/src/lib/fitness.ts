import {colors} from './charts';

export const fitnessAgeColor=(age:number,actual:number|null)=>actual==null?colors.ink:age<actual?colors.good:age===actual?colors.fair:colors.poor;
export const driverScale=(key:string):[number,number]=>({vo2max:[20,70],restingHr:[35,85],bmi:[15,35],bodyFat:[5,40],vigorousMinutes:[0,200],vigorousDays:[0,7]} as Record<string,[number,number]>)[key]??[0,100];
export const driverRange=(key:string,target:number|null):[number,number]|null=>target==null?null:key==='bmi'?[18.5,target]:['restingHr','bodyFat'].includes(key)?[driverScale(key)[0],target]:[target,driverScale(key)[1]];
