import {describe,it,expect} from 'vitest';
import {rangeStatus,vitalStatus,vitalSummary,limitingFactor} from './signals';
import type {VitalsCard} from './generated/VitalsCard';
import type {VitalReading} from './generated/VitalReading';

const vital = (value:number|null,normal:[number,number]|null):VitalReading => ({
  kind:'restingHr',value,normal,unit:'bpm',scale:[30,90],trend:[],tone:'neutral',status:null,rangeSource:normal?'raring':'none',
});
const card = (...readings:VitalReading[]):VitalsCard => ({readings,offCount:0,baselineReady:true});

describe('personal range presentation',()=>{
 it('does not call missing readings normal, even when a baseline exists',()=>{
  expect(rangeStatus(null,[44,52])).toEqual({label:'Not recorded',tone:'neutral'});
  expect(vitalSummary(card(vital(48,[44,52]),vital(null,[44,52])))).toMatchObject({
   headline:'1 reading in range',detail:'1 of 2 recorded · 1 comparable',tone:'neutral',
  });
 });
 it('keeps uncalibrated readings neutral',()=>{
  expect(rangeStatus(48,null).tone).toBe('neutral');
  expect(vitalSummary(card(vital(48,null))).headline).toBe('Getting to know your usual');
  expect(vitalSummary(card(vital(null,null))).headline).toBe('Waiting for readings');
  expect(vitalSummary(card()).tone).toBe('neutral');
 });
 it('uses inclusive bounds, accepts zero, and labels the direction of outliers',()=>{
  expect(rangeStatus(44,[44,52]).label).toBe('Within range');
  expect(rangeStatus(52,[44,52]).label).toBe('Within range');
  expect(rangeStatus(0,[0,0]).tone).toBe('good');
  expect(rangeStatus(40,[44,52]).label).toBe('Below usual');
  expect(rangeStatus(60,[44,52]).label).toBe('Above usual');
 });
 it('distinguishes a nightly HRV comparison from Garmin’s seven-day status',()=>{
  expect(rangeStatus(40,[48,62],'hrv').label).toBe('Below baseline');
  expect(rangeStatus(70,[48,62],'hrv').label).toBe('Above baseline');
 });
 it('uses Garmin status instead of reclassifying by its displayed bounds',()=>{
  expect(vitalStatus({...vital(60,[44,52]),status:'IN_RANGE',tone:'good',rangeSource:'garmin'}))
    .toEqual({label:'Within Garmin range',tone:'good'});
  expect(vitalStatus({...vital(48,[44,52]),status:'ABOVE_RANGE',tone:'poor',rangeSource:'garmin'}).tone).toBe('poor');
  expect(vitalStatus({...vital(48,[44,52]),status:'ONBOARDING',rangeSource:'none'}).tone).toBe('neutral');
  expect(vitalStatus({...vital(48,[44,52]),status:'NEW_STATUS',rangeSource:'garmin'}).tone).toBe('neutral');
 });
 it('counts only comparable readings without hiding an outlier behind incomplete baselines',()=>{
  expect(vitalSummary(card(vital(60,[44,52]),vital(48,null),vital(null,[44,52])))).toMatchObject({
   headline:'1 outside your ranges',outside:1,tone:'fair',
  });
  expect(vitalSummary(card(vital(48,[44,52]))).headline).toBe('Within your ranges');
 });
});

it('selects the lowest recorded Garmin factor without changing their order',()=>{
 const factors=[
  {key:'sleep',name:'Sleep',percent:80,tone:'good' as const,detail:'Good'},
  {key:'hrv',name:'HRV',percent:null,tone:'neutral' as const,detail:'Pending'},
  {key:'recovery',name:'Recovery',percent:40,tone:'poor' as const,detail:'Low'},
 ];
 expect(limitingFactor(factors)?.key).toBe('recovery');
 expect(factors[0].key).toBe('sleep');
 expect(limitingFactor([factors[1]])).toBeNull();
 expect(limitingFactor([])).toBeNull();
});
