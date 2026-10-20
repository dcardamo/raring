/// <reference types="node" />
import {readFileSync} from 'node:fs';
import {describe,expect,it} from 'vitest';
import {colors,goalColors} from './charts';
import {GOALS} from './goals';

const css=readFileSync(new URL('../app.css',import.meta.url),'utf8');
const token=(name:string)=>{
 const value=css.match(new RegExp(`--${name}:([^;]+);`))?.[1];
 if(!value)throw new Error(`Missing design token: ${name}`);
 return value.trim();
};
const luminance=(hex:string)=>[1,3,5].map(i=>parseInt(hex.slice(i,i+2),16)/255)
 .map(c=>c<=.04045?c/12.92:((c+.055)/1.055)**2.4)
 .reduce((sum,c,i)=>sum+c*[.2126,.7152,.0722][i],0);
const contrast=(a:string,b:string)=>{
 const [hi,lo]=[luminance(a),luminance(b)].sort((x,y)=>y-x);
 return (hi+.05)/(lo+.05);
};

describe('Raring visual tokens',()=>{
 it('keeps CSS surfaces and chart/goal palettes in agreement',()=>{
  for(const name of ['good','fair','poor','amount','ink','deep','light','rem','awake'] as const)expect(colors[name]).toBe(token(name));
  expect(colors.secondary).toBe(token('text2'));
  for(const [kind,name] of [['strength','strength'],['zone2','zone2'],['highIntensity','high'],['intensity','intensity'],['sleep','sleep']] as const){
   expect(goalColors[kind]).toBe(token(`goal-${name}`));
  }
  expect(GOALS.sleepScore.color).toBe(GOALS.sleepTiming.color);
 });
 it('keeps every text and semantic color above 4.5:1 on every solid surface',()=>{
  const inks=[...['text','text2','text3','ink','good','fair','poor','amount'].map(token),...Object.values(goalColors)];
  for(const surface of ['bg','card','surface2','surface3'])for(const ink of inks){
   expect(contrast(ink,token(surface)),`${ink} on ${surface}`).toBeGreaterThanOrEqual(4.5);
  }
  expect(contrast(token('bg'),token('ink'))).toBeGreaterThanOrEqual(4.5);
 });
 it('keeps chart text minimums explicit',()=>{
  expect(parseFloat(token('type-axis'))).toBeGreaterThanOrEqual(11);
  expect(parseFloat(token('type-label'))).toBeGreaterThanOrEqual(12);
 });
});
