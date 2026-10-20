import {describe,it,expect} from 'vitest';
import {addDays,historyAxis,historyPoints,historyScale,readinessTone} from './history';

describe('dated metric history',()=>{
 it('ends on the Garmin data date rather than the phone date',()=>{
  const days=historyPoints([10,20,30],'2025-03-10',7);
  expect(days[0]).toEqual({date:'2025-03-04',value:null});
  expect(days[4]).toEqual({date:'2025-03-08',value:10});
  expect(days[6]).toEqual({date:'2025-03-10',value:30});
 });
 it('keeps gaps, zero and leap days',()=>{
  expect(historyPoints([1,null,0],'2024-03-01',3)).toEqual([
   {date:'2024-02-28',value:1},{date:'2024-02-29',value:null},{date:'2024-03-01',value:0},
  ]);
 });
 it('takes the last days without bridging missing data',()=>{
  expect(historyPoints([1,2,null,4],'2025-01-02',3)).toEqual([
   {date:'2024-12-31',value:2},{date:'2025-01-01',value:null},{date:'2025-01-02',value:4},
  ]);
  expect(historyPoints([NaN,Infinity],'2025-01-02',2).every(p=>p.value===null)).toBe(true);
 });
 it('scales to include every outlier and the entire personal range',()=>{
  const [low,high]=historyScale([0,null,200],[40,60]);
  expect(low).toBeLessThan(0);expect(high).toBeGreaterThan(200);
  const flat=historyScale([5,5],null);
  expect(flat[0]).toBeLessThan(5);expect(flat[1]).toBeGreaterThan(5);
  expect(historyScale([],null)).toEqual([0,100]);
  expect(historyScale([null],[48,62])).toEqual([45.9,64.1]);
 });
 it('ticks at round values that cover every reading',()=>{
  expect(historyAxis([66,75,83],null)).toEqual({scale:[60,90],ticks:[60,70,80,90]});
  expect(historyAxis([],null)).toEqual({scale:[0,100],ticks:[0,25,50,75,100]});
  expect(historyAxis([14.1,14.4],null).ticks).toEqual([14,14.2,14.4,14.6]);
  // Minutes tick at whole hours or their halves, never at 7:23.
  const asleep=historyAxis([412,468,505],null,'min');
  expect(asleep.ticks.every(t=>t%30===0)).toBe(true);
  expect(asleep.scale[0]).toBeLessThanOrEqual(412);expect(asleep.scale[1]).toBeGreaterThanOrEqual(505);
  for(const values of [[0,200],[5,5],[48.2,61.7],[-0.3,0.4]]){const {scale,ticks}=historyAxis(values,null);
   expect(ticks.length).toBeGreaterThanOrEqual(2);expect(ticks.length).toBeLessThanOrEqual(5);
   expect(ticks[0]).toBe(scale[0]);expect(ticks.at(-1)).toBe(scale[1]);
   expect(Math.min(...values)).toBeGreaterThan(scale[0]);expect(Math.max(...values)).toBeLessThan(scale[1]);}
 });
 it('uses Garmin readiness boundaries rather than a sleep or competing score scale',()=>{
  expect([24,25,49,50,69,70,74,75,94,95,100].map(readinessTone))
   .toEqual(['poor','poor','poor','fair','fair','fair','fair','good','good','good','good']);
 });
});

describe('addDays', () => {
  it('crosses months and daylight-saving changes by calendar day', () => {
    expect(addDays('2025-10-27', 6)).toBe('2025-11-02');
    expect(addDays('2025-03-08', 1)).toBe('2025-03-09');
  });
});
