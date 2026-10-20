import {describe,it,expect} from 'vitest';
import {metricAxis,plotAxis} from './plot';

describe('shared plot geometry',()=>{
 it('gives amounts a zero baseline and round ticks without changing the values',()=>{
  const input=[90,120,null,165];
  expect(plotAxis(input,{zero:true})).toEqual({scale:[0,200],ticks:[0,50,100,150,200]});
  expect(input).toEqual([90,120,null,165]);
 });
 it('includes comparator bounds, handles empty/flat data and preserves signed domains',()=>{
  for(const input of [[],[null],[0],[5,5],[450,610,60,150,800],[-.3,.4]]){
   const {scale,ticks}=plotAxis(input);
   expect(scale.every(Number.isFinite)).toBe(true);
   expect(scale[1]).toBeGreaterThan(scale[0]);
   for(const v of input)if(v!=null){expect(v).toBeGreaterThanOrEqual(scale[0]);expect(v).toBeLessThanOrEqual(scale[1])}
   expect(ticks[0]).toBe(scale[0]);expect(ticks.at(-1)).toBe(scale[1]);
  }
 });
 it('keeps count axes integer and duration ticks on readable minute boundaries',()=>{
  expect(plotAxis([0,1,2],{zero:true,integer:true}).ticks.every(Number.isInteger)).toBe(true);
  expect(plotAxis([2,8],{zero:true,integer:true}).ticks.every(Number.isInteger)).toBe(true);
  expect(plotAxis([20,252,256],{zero:true,unit:'min'}).ticks.every(t=>t%60===0)).toBe(true);
 });
 it('honors explicit plot bounds and physical caps',()=>{
  expect(plotAxis([98,99],{min:90,max:100,cap:100}).scale).toEqual([90,100]);
  expect(plotAxis([null,Infinity,NaN],{zero:true}).scale.every(Number.isFinite)).toBe(true);
 });
 it('uses the same fixed score domain for either axis, but not for physiology',()=>{
  for(const m of ['readiness','sleepScore','stress','bodyBattery'] as const){
   expect(metricAxis(m,[23,31],null).scale).toEqual([0,100]);
   expect(metricAxis(m,[0,null,100],null).scale).toEqual([0,100]);
  }
  expect(metricAxis('skinTemp',[-.3,.4],null).scale[0]).toBeLessThan(0);
  expect(metricAxis('hrv',[48,58],[45,62]).scale).not.toEqual([0,100]);
 });
});
