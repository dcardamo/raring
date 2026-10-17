import type {Muscle} from './generated/Muscle';
import type {StrengthSession} from './generated/StrengthSession';
import type {SetFlag} from './generated/SetFlag';
import type {Lift} from './generated/Lift';
import type {WeightUnit} from './generated/WeightUnit';

export const muscleNames:Record<Muscle,string>={chest:'Chest',back:'Back',shoulders:'Shoulders',arms:'Arms',core:'Core',hips:'Hips',quads:'Quads',hamstrings:'Hamstrings',glutes:'Glutes',calves:'Calves'};
export const weight=(w:number|null)=>w==null?'—':String(Math.round(w*10)/10);
export const lift=(l:Lift)=>`${weight(l.weight)}×${l.reps??'—'}`;
export const unitLabel=(unit:WeightUnit)=>unit==='lb'?'lb':'kg';

const flagWords:Record<SetFlag,[string,string]>={zeroReps:['a 0-rep set','0-rep sets'],missingReps:['a set without a rep count','sets without rep counts'],doubled:['a doubled set','doubled sets'],unnamed:['an unnamed exercise','unnamed sets']};
/** "3 things look off" and "A 0-rep set, a doubled set, an unnamed exercise". */
export function flagSummary(s:StrengthSession):{title:string,detail:string}|null{
 const counts:[SetFlag,number][]=[['zeroReps',s.zeroRepSets],['missingReps',s.missingRepSets],['doubled',s.doubledSets],['unnamed',s.unnamedSets]];
 // Unnamed sets of one exercise are one thing to check.
 const things=counts.map(([flag,n])=>[flag,flag==='unnamed'?s.exercises.filter(e=>e.key==null).length:n] as [SetFlag,number]).filter(([,n])=>n>0);
 const total=things.reduce((a,[,n])=>a+n,0);
 if(!total)return null;
 const parts=things.map(([flag,n])=>n===1?flagWords[flag][0]:`${n} ${flag==='unnamed'?'unnamed exercises':flagWords[flag][1]}`);
 const detail=parts.length>1?`${parts.slice(0,-1).join(', ')} and ${parts.at(-1)}`:parts[0];
 return {title:total===1?'1 thing looks off':`${total} things look off`,detail:detail[0].toUpperCase()+detail.slice(1)};
}

/** The hero's reps: a gap when no set has a count, and the uncounted sets beside a partial sum. */
export function repsLabel(s:StrengthSession):{value:string,label:string}{
 if(s.reps==null)return {value:'—',label:'Reps unknown'};
 if(!s.unknownRepSets)return {value:String(s.reps),label:'Reps'};
 return {value:String(s.reps),label:`Reps · ${s.unknownRepSets} set${s.unknownRepSets>1?'s':''} uncounted`};
}
