import {describe,it,expect} from 'vitest';
import {rangeStatus,vitalStatus,vitalSummary,limitingFactor,trendBand,vitalTrend,vitalName} from './signals';
import type {TrendMetric} from './generated/TrendMetric';
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
   headline:'1 reading in range',detail:'1 of 2 recorded',tone:'neutral',
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
  // Outside a usual range is attention (amber), as on the charts, not a poor rating.
  expect(rangeStatus(40,[44,52]).tone).toBe('fair');
  expect(rangeStatus(60,[44,52]).tone).toBe('fair');
 });
 it('distinguishes a nightly HRV comparison from Garmin’s seven-day status',()=>{
  expect(rangeStatus(40,[48,62],'hrv').label).toBe('Below baseline');
  expect(rangeStatus(70,[48,62],'hrv').label).toBe('Above baseline');
 });
 it('uses Garmin status instead of reclassifying by its displayed bounds',()=>{
  expect(vitalStatus({...vital(60,[44,52]),status:'IN_RANGE',tone:'good',rangeSource:'garmin'}))
    .toEqual({label:'Within Garmin range',tone:'good'});
  expect(vitalStatus({...vital(48,[44,52]),status:'ABOVE_RANGE',tone:'fair',rangeSource:'garmin'}).tone).toBe('fair');
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

it('selects the factor Garmin rates worst without changing their order',()=>{
 const factors=[
  {key:'sleep',name:'Sleep',percent:80,tone:'good' as const,detail:'Good'},
  {key:'hrv',name:'HRV',percent:null,tone:'neutral' as const,detail:'Pending'},
  {key:'recovery',name:'Recovery',percent:40,tone:'poor' as const,detail:'Low'},
  {key:'acuteLoad',name:'Acute load',percent:20,tone:'neutral' as const,detail:'Garmin rates your Acute load at 20%.'},
  {key:'stressHistory',name:'Stress history',percent:35,tone:'fair' as const,detail:'Moderate'},
 ];
 expect(limitingFactor(factors)?.key,'a Poor rating beats a lower percentage').toBe('recovery');
 expect(factors[0].key).toBe('sleep');
 expect(limitingFactor([factors[1],factors[3]])).toBeNull();
 expect(limitingFactor([])).toBeNull();
});

describe('trend bands come only from the same measure',()=>{
 const reading=(kind:VitalReading['kind'],normal:[number,number],healthStatus:boolean):VitalReading=>({
  kind,value:50,normal,unit:'',scale:[0,100],trend:[],tone:'good',status:healthStatus?'IN_RANGE':null,rangeSource:healthStatus?'garmin':'raring',
 });
 const health=(healthStatus:boolean)=>({
  restingHrRange:[44,50] as [number,number],
  vitals:card(reading('restingHr',[52,62],healthStatus),reading('hrv',[48,63],healthStatus),reading('respiration',[13,15],healthStatus),reading('spo2',[94,98],healthStatus),reading('skinTemp',[-0.4,0.4],healthStatus)),
 });
 const metrics:TrendMetric[]=['readiness','sleepScore','sleepTime','hrv','restingHr','sleepHr','respiration','spo2','skinTemp','bodyBattery','stress','vo2max','fitnessAge','acuteLoad'];
 it.each([
  // metric, with Health Status, without
  ['restingHr',[44,50],[44,50]],
  ['sleepHr',[52,62],null],
  ['hrv',[48,63],[48,63]],
  ['respiration',[13,15],[13,15]],
  ['spo2',[94,98],[94,98]],
  ['skinTemp',[-0.4,0.4],[-0.4,0.4]],
 ] as [TrendMetric,[number,number]|null,[number,number]|null][])('%s',(metric,withStatus,without)=>{
  expect(trendBand(metric,health(true))).toEqual(withStatus);
  expect(trendBand(metric,health(false))).toEqual(without);
 });
 it('never shades a metric without a measured range',()=>{
  for(const m of metrics.filter(m=>!['restingHr','sleepHr','hrv','respiration','spo2','skinTemp'].includes(m))){
   expect(trendBand(m,health(true))).toBeNull();
   expect(trendBand(m,health(false))).toBeNull();
  }
 });
 it('names and routes Health Status HR as Sleep HR, never resting HR',()=>{
  expect(vitalTrend(reading('restingHr',[52,62],true))).toBe('sleepHr');
  expect(vitalName(reading('restingHr',[52,62],true))).toBe('Sleep HR');
  expect(vitalTrend(reading('restingHr',[52,62],false))).toBe('restingHr');
  expect(vitalName(reading('restingHr',[52,62],false))).toBe('Resting HR');
 });
});
