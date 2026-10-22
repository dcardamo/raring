import {describe,expect,it} from 'vitest';
import {sleepEvening,recentNights,clockLabel} from './clock';
import type {SleepPlan} from './generated/SleepPlan';

const plan={needReadAt:null,night:'2025-03-13',bedtime:'23:10',bedtimeStart:'22:30',bedtimeSource:'garmin',inBed:'22:55',inBedAt:'2025-03-12T22:55:00',fallAsleepMin:15,inBedMin:485,
 wakeAt:'2025-03-13T07:00:00',windDown:'22:00',windDownLeadMin:30,windDownAt:'2025-03-12T22:00:00',bedtimeStartAt:'2025-03-12T22:30:00',bedtimeAt:'2025-03-12T23:10:00',
 wakeTime:'07:00',needMin:470,source:'tonight',baselineMin:null,adjustments:[],reason:null,tip:null,wakeNights:14,stale:false} satisfies SleepPlan;

describe('sleepEvening',()=>{
 it('puts the evening on one axis, half an hour either side of its decisions',()=>{
  // Half an hour before now to half an hour past the asleep-by time, both rounded out
  // to the half hour: 21:00 to 00:00.
  const e=sleepEvening(plan,new Date('2025-03-12T21:30:00'))!;
  expect([e.from,e.to]).toEqual([21,0]);
  expect(e.windDown!.left).toBeCloseTo(60/180*100);
  expect(e.windDown!.width).toBeCloseTo(55/180*100);
  // 22:55 in bed to 23:10 asleep is the quarter of an hour Raring allows for falling asleep.
  expect(e.fallAsleep!.left).toBeCloseTo(115/180*100);
  expect(e.fallAsleep!.width).toBeCloseTo(15/180*100);
  // Garmin's window 22:30–23:10 is bracketed under the bed edge, not drawn as sleep.
  expect(e.window!.left).toBeCloseTo(90/180*100);
  expect(e.window!.width).toBeCloseTo(40/180*100);
  expect(e.now).toBeCloseTo(30/180*100);
  expect(e.ticks.map(t=>[clockLabel(t.h),Math.round(t.at)]))
   .toEqual([['21',0],['22',33],['23',67],['00',100]]);
 });
 it('keeps the axis still once the evening has begun, so now moves along it',()=>{
  // From wind-down on the axis starts half an hour before it, whatever the clock says.
  const e=sleepEvening(plan,new Date('2025-03-12T22:20:00'))!;
  expect([e.from,e.to]).toEqual([21.5,0]);
  expect(e.windDown!.left).toBeCloseTo(30/150*100);
  expect(e.now).toBeCloseTo(50/150*100);
 });
 it('has no falling-asleep span at zero minutes and no window without Garmin’s',()=>{
  const e=sleepEvening({...plan,inBed:'23:10',inBedAt:'2025-03-12T23:10:00',fallAsleepMin:0,bedtimeStart:null,bedtimeStartAt:null,bedtimeSource:'wakeHistory'},new Date('2025-03-12T21:30:00'))!;
  expect(e.fallAsleep).toBeNull();
  expect(e.window).toBeNull();
 });
 it('draws nothing without an asleep-by time or with it before in bed',()=>{
  expect(sleepEvening({...plan,bedtimeAt:''},new Date())).toBeNull();
  expect(sleepEvening({...plan,inBedAt:'2025-03-13T00:30:00'},new Date())).toBeNull();
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
