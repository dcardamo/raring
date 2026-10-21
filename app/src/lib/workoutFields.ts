import type {DistanceUnit} from './generated/DistanceUnit';
import type {Feel} from './generated/Feel';
import type {FieldGroup} from './generated/FieldGroup';
import type {Icon} from './generated/Icon';
import type {Stat} from './generated/Stat';
import type {StreamKind} from './generated/StreamKind';
import {elapsed,num} from './format';
import {elevation,elevationUnit,pace,paceUnit,speed,speedUnit,statute} from './units';
// A workout's Garmin fields in the activity's units. Values arrive as the
// core sends them (metric, pace in s/km, speed in km/h); statute accounts
// read ft, in, mph and fl oz, and swims keep the pool's own unit.
const CM_PER_FT=30.48,CM_PER_IN=2.54,ML_PER_FL_OZ=29.5735,FT_PER_M=3.28084;
export const FEELS:Feel[]=['veryWeak','weak','normal','strong','veryStrong'];
// Garmin's own answers on the watch.
export const FEEL_WORD:Record<Feel,string>={veryWeak:'Very weak',weak:'Weak',normal:'Normal',strong:'Strong',veryStrong:'Very strong'};
export const groupName=(group:FieldGroup,icon:Icon)=>({
 time:'Time',pace:icon==='bike'?'Speed':'Pace',form:icon==='run'?'Running form':icon==='bike'?'Cadence':'Steps and cadence',
 power:'Power',strokes:icon==='swim'?'Swim':'Strokes',elevation:'Elevation',heartRate:'Heart rate',breathing:'Breathing rate',
 stress:'Stress',temperature:'Watch temperature',energy:'Calories and fluid',
} satisfies Record<FieldGroup,string>)[group];
const ELEVATION=new Set(['elevationLoss','minElevation','maxElevation']);
export const fieldValue=(s:Stat,u:DistanceUnit)=>{
 const st=statute(u),v=s.value;
 if(s.unit==='s')return elapsed(v);
 if(s.unit==='s/km')return `${pace(v,u)}${paceUnit(u)}`;
 if(s.unit==='km/h')return `${num(speed(v,u),1)} ${speedUnit(u)}`;
 if(ELEVATION.has(s.key))return `${num(elevation(v,u))} ${elevationUnit(u)}`;
 if(s.key==='averageStrokeDistance')return st?`${num(v*FT_PER_M,1)} ft`:`${num(v,1)} m`;
 if(s.key==='strideLength')return st?`${num(v/CM_PER_FT,2)} ft`:`${num(v/100,2)} m`;
 if(s.key==='verticalOscillation')return st?`${num(v/CM_PER_IN,1)} in`:`${num(v,1)} cm`;
 if(s.unit==='ml')return st?`${num(v/ML_PER_FL_OZ)} fl oz`:`${num(v)} ml`;
 if(s.key==='differenceBodyBattery')return `${v>0?'+':''}${num(v)}`;
 if(s.key==='intensityFactor')return num(v,2);
 const digits=['%','cm/s','brpm','°C'].includes(s.unit)||s.key==='trainingStressScore'?1:0;
 return `${num(v,digits)}${s.unit==='%'?'%':s.unit?' '+s.unit:''}`;
};
// Along the way's streams that need a unit change or decimals; the rest plot as sent.
export const streamScale=(kind:StreamKind,u:DistanceUnit):{conv:(v:number)=>number,unit:string,digits:number}|null=>{
 const st=statute(u);
 switch(kind){
  case 'strideLength':return st?{conv:v=>v/CM_PER_FT,unit:'ft',digits:1}:{conv:v=>v/100,unit:'m',digits:2};
  case 'verticalOscillation':return st?{conv:v=>v/CM_PER_IN,unit:'in',digits:1}:{conv:v=>v,unit:'cm',digits:1};
  case 'verticalRatio':return {conv:v=>v,unit:'%',digits:1};
  case 'strokeDistance':return st?{conv:v=>v*FT_PER_M,unit:'ft',digits:1}:{conv:v=>v,unit:'m',digits:1};
  default:return null;
 }
};
