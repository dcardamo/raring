import {describe,expect,it} from 'vitest';
import {nightStrip,phaseEnded,tonightLine} from './tonight';
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

describe('tonightLine',()=>{
 const full={...plan,wakeTime:'06:30',needMin:480} as SleepPlan;
 it('runs from an hour before wind-down to past the usual wake, ticking even hours',()=>{
  const line=tonightLine(full,new Date('2025-03-12T21:30:00'))!;
  expect(line.ticks.map(t=>t.label)).toEqual(['22','00','02','04','06']);
  expect(line.ticks[0].left).toBeCloseTo(10);
  expect(line.now).toBeCloseTo(5);
  expect(line.wake).toBeCloseTo(95);
  // Eight hours of need ending at 06:30 starts at 22:30.
  expect(line.needFrom).toBeCloseTo(15);
 });
 it('ends after the asleep-by time without a wake time and hides now in the day',()=>{
  const line=tonightLine({...full,wakeTime:null},new Date('2025-03-12T13:00:00'))!;
  expect(line.wake).toBeNull();
  expect(line.needFrom).toBeNull();
  expect(line.now).toBeNull();
  expect(line.ticks.map(t=>t.label)).toEqual(['22','00']);
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
