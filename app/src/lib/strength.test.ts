import {describe,expect,it} from 'vitest';
import {flagSummary,lift,weight,repsLabel} from './strength';
import {amountColor,textOn,figureArt} from './bodymap';
import type {StrengthSession} from './generated/StrengthSession';

const session=(zero:number,doubled:number,unnamedExercises:number):StrengthSession=>({unit:'kg',figure:'female',sets:10,reps:80,unknownRepSets:0,missingRepSets:0,restS:0,records:0,setsRecorded:true,zeroRepSets:zero,doubledSets:doubled,unnamedSets:unnamedExercises*2,muscles:[],
 exercises:Array.from({length:unnamedExercises},()=>({key:null,name:null,muscles:[],sets:[],lastDate:null,lastSets:[],record:null}))});

describe('strength', () => {
 it('summarizes flags as things to check', () => {
  expect(flagSummary(session(0,0,0))).toBeNull();
  expect(flagSummary(session(1,1,1))).toEqual({title:'3 things look off',detail:'A 0-rep set, a doubled set and an unnamed exercise'});
  // Unnamed sets of one exercise are one thing to check.
  expect(flagSummary(session(0,0,1))).toEqual({title:'1 thing looks off',detail:'An unnamed exercise'});
  expect(flagSummary(session(2,0,2))).toEqual({title:'4 things look off',detail:'2 0-rep sets and 2 unnamed exercises'});
 });
 it('distinguishes missing, partial and zero rep totals', () => {
  const base=session(0,0,0);
  expect(repsLabel({...base,reps:null,unknownRepSets:2})).toEqual({value:'—',label:'Reps unknown'});
  expect(repsLabel({...base,reps:14,unknownRepSets:1})).toEqual({value:'14',label:'Reps · 1 set uncounted'});
  expect(repsLabel({...base,reps:0})).toEqual({value:'0',label:'Reps'});
  expect(flagSummary({...base,missingRepSets:1})?.detail).toBe('A set without a rep count');
 });
 it('formats weights and lifts', () => {
  expect(weight(97.5)).toBe('97.5');
  expect(weight(null)).toBe('—');
  expect(lift({weight:null,reps:12})).toBe('—×12');
 });
 it('shades the body map in one blue with readable labels', () => {
  expect(amountColor(0,12)).toBe('#353C4C');
  expect(amountColor(12,12)).toBe('#5ca8ff');
  expect(amountColor(40,12)).toBe('#5ca8ff');
  expect(textOn('#5ca8ff')).toBe('#000000');
  expect(textOn('#1f3557')).toBe('#FFFFFF');
  for(const figure of ['female','male'] as const){
   const art=figureArt(figure);
   const muscles=new Set([...art.front,...art.back].map(s=>s.muscle));
   expect([...muscles].sort()).toEqual(['arms','back','calves','chest','core','glutes','hamstrings','hips','quads','shoulders']);
   expect(art.outline).not.toMatch(/NaN/);
  }
 });
});
