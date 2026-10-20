import {describe,expect,it} from 'vitest';
import {phaseEnded,wakeLabel} from './tonight';
import type {SleepPlan} from './generated/SleepPlan';
import type {TodayView} from './generated/TodayView';


describe('phaseEnded',()=>{
 const view=(until:string|null)=>({focus:{until}}) as TodayView;
 it('expires a cached Today at its phase boundary',()=>{
  const t=new Date('2025-03-12T20:30:00').getTime();
  expect(phaseEnded(view('2025-03-12T20:30:00'),t-1)).toBe(false);
  expect(phaseEnded(view('2025-03-12T20:30:00'),t)).toBe(true);
  expect(phaseEnded(view(null),t)).toBe(false);
 });
});
