import {describe,it,expect} from 'vitest';
import {normalize,searchHits,searchIndex} from './search';
import {knownRoute} from './navigation';

const find=(q:string,sleepHr:boolean|null=true)=>searchHits(q,searchIndex(sleepHr)).map(e=>`${e.name} → ${e.route}${e.anchor?'#'+e.anchor:''}`);

describe('search',()=>{
 it('normalises subscripts, accents and case',()=>{
  expect(normalize('SpO₂')).toBe('spo2');
  expect(normalize('VO₂ max')).toBe('vo2 max');
  expect(normalize('Last night’s sleep')).toBe('last nights sleep');
 });
 it('sends resting heart rate to Garmin’s daily resting HR, and Sleep HR to the overnight vital',()=>{
  expect(find('resting')).toEqual(['Resting heart rate (daily) → metric/heartRate']);
  expect(find('sleep hr')).toEqual(['Sleep HR → metric/restingHr']);
  // Without Health Status, the vital is Garmin's resting HR under its own name.
  expect(find('resting',false)).toEqual(['Resting heart rate → metric/restingHr']);
  expect(find('sleep hr',false)).toEqual([]);
  expect(find('sleep hr',null)).toEqual([]);
 });
 it.each([
  ['spo2','SpO₂ → metric/spo2'],['blood oxygen','SpO₂ → metric/spo2'],['vo2','VO₂ max → metric/vo2max'],
  ['fitness age','Fitness Age → health#fitness-age'],['intensity','Intensity minutes → health#intensity'],
  ['journal','Garmin journal → health#journal'],['body fat','Body fat → health/weight'],['bmi','BMI → health/weight'],
  ['health','Health → health'],['import','Import settings → settings#import'],['export','Export your data → settings#export'],
  ['zone 2','Zone 2 → plan'],['workouts','Activities → training#training-sections'],['temperature','Skin temperature → metric/skinTemp'],['all-day heart','Heart rate → metric/heartRate'],
 ])('%s finds %s',(q,hit)=>{
  expect(find(q)).toContain(hit);
 });
 it('every entry opens a page this app has',()=>{
  for(const sleepHr of [true,false,null])for(const e of searchIndex(sleepHr))expect(knownRoute(e.route),e.name).toBe(e.route);
 });
});
