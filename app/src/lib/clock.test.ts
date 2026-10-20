import {describe,expect,it} from 'vitest';
import {sleepClock,arcPath,recentNights,clockLabel} from './clock';
import type {SleepPlan} from './generated/SleepPlan';

const plan={needReadAt:null,night:'2025-03-13',bedtime:'23:10',bedtimeStart:'22:30',bedtimeSource:'garmin',inBed:'22:55',inBedAt:'2025-03-12T22:55:00',fallAsleepMin:15,inBedMin:485,
 wakeAt:'2025-03-13T07:00:00',windDown:'22:00',windDownLeadMin:30,windDownAt:'2025-03-12T22:00:00',bedtimeStartAt:'2025-03-12T22:30:00',bedtimeAt:'2025-03-12T23:10:00',
 wakeTime:'07:00',needMin:470,source:'tonight',baselineMin:null,adjustments:[],reason:null,tip:null,wakeNights:14,stale:false} satisfies SleepPlan;

describe('sleepClock',()=>{
 it('places wind-down, falling asleep, sleep and Garmin’s window on a 24-hour ring',()=>{
  const c=sleepClock(plan,new Date('2025-03-12T21:30:00'))!;
  expect(c.windDown).toEqual({from:330,to:343.75});
  expect(c.fallAsleep).toEqual({from:343.75,to:347.5});
  // Asleep 23:10 to 07:00 crosses midnight: the arc runs past 360.
  expect(c.sleep.from).toBe(347.5);
  expect(c.sleep.to).toBeCloseTo(465);
  expect(c.window).toEqual({from:337.5,to:347.5});
  expect(c.now).toBe(322.5);
 });
 it('has no falling-asleep arc at zero minutes and no window without Garmin’s',()=>{
  const c=sleepClock({...plan,inBed:'23:10',inBedAt:'2025-03-12T23:10:00',fallAsleepMin:0,bedtimeStart:null,bedtimeStartAt:null,bedtimeSource:'wakeHistory'},new Date('2025-03-12T12:00:00'))!;
  expect(c.fallAsleep).toBeNull();
  expect(c.window).toBeNull();
  expect(c.windDown).toEqual({from:330,to:347.5});
 });
 it('draws nothing without an expected wake',()=>{
  expect(sleepClock({...plan,wakeAt:''},new Date())).toBeNull();
 });
 it('draws arcs clockwise from the top',()=>{
  expect(arcPath(50,40,{from:0,to:90})).toBe('M50.00 10.00A40 40 0 0 1 90.00 50.00');
  expect(arcPath(50,40,{from:0,to:270})).toContain(' 0 1 1 ');
 });
});

describe('recentNights',()=>{
 it('keeps 14 nights, marks missing ones and frames tonight’s band and wake',()=>{
  const r=recentNights('2025-03-12',[{date:'2025-03-12',bedHour:23.5,wakeHour:6.75},{date:'2025-03-10',bedHour:0.5,wakeHour:7.25}],plan);
  expect(r.nights).toHaveLength(14);
  expect(r.nights[13]).toEqual({date:'2025-03-12',bed:23.5,wake:30.75});
  expect(r.nights[11]).toEqual({date:'2025-03-10',bed:24.5,wake:31.25});
  expect(r.nights[12]).toEqual({date:'2025-03-11',bed:null,wake:null});
  expect(r.band).toEqual([22.5,23+10/60]);
  expect(r.wake).toBe(31);
  expect([r.top,r.bottom]).toEqual([22,32]);
  expect(r.ticks.map(clockLabel)).toEqual(['22','00','02','04','06','08']);
 });
 it('keeps a sensible axis without nights or a plan',()=>{
  const r=recentNights('2025-03-12',[],null);
  expect(r.nights.every(n=>n.bed==null)).toBe(true);
  expect([r.top,r.bottom,r.band,r.wake]).toEqual([20,32,null,null]);
 });
});
