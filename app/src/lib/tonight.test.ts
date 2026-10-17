import {describe,expect,it} from 'vitest';
import {nightStrip,phaseEnded} from './tonight';
import type {SleepPlan} from './generated/SleepPlan';
import type {TodayView} from './generated/TodayView';

const plan={windDownAt:'2025-03-12T22:00:00',bedtimeStartAt:'2025-03-12T22:30:00',bedtimeAt:'2025-03-12T23:10:00'} as SleepPlan;

describe('nightStrip',()=>{
 it('spans whole hours from two hours before the window to an hour after it',()=>{
  const strip=nightStrip(plan,new Date('2025-03-12T21:00:00'))!;
  expect(strip.ticks.map(t=>t.label)).toEqual(['20','21','22','23','00','01']);
  expect(strip.windDown).toBe(40);
  expect(strip.now).toBe(20);
 });
 it('drops the marker outside the axis and works without Garmin\'s window',()=>{
  const strip=nightStrip({...plan,bedtimeStartAt:null},new Date('2025-03-12T15:00:00'))!;
  expect(strip.now).toBeNull();
  expect(strip.start).toBe(strip.end);
 });
});

describe('phaseEnded',()=>{
 const view=(until:string|null)=>({focus:{until}}) as TodayView;
 it('expires a cached Today at its phase boundary',()=>{
  const t=new Date('2025-03-12T20:30:00').getTime();
  expect(phaseEnded(view('2025-03-12T20:30:00'),t-1)).toBe(false);
  expect(phaseEnded(view('2025-03-12T20:30:00'),t)).toBe(true);
  expect(phaseEnded(view(null),t)).toBe(false);
 });
});
