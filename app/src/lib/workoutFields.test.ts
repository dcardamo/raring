import {describe,it,expect} from 'vitest';
import {fieldValue,groupName,streamScale} from './workoutFields';
const f=(key:string,value:number,unit:string)=>({key,label:key,value,unit});
describe('workout fields',()=>{
 it('reads pace, speed and elevation in the activity unit',()=>{
  expect(fieldValue(f('maxSpeed',251,'s/km'),'km')).toBe('4:11/km');
  expect(fieldValue(f('maxSpeed',251,'s/km'),'mi')).toBe('6:44/mi');
  expect(fieldValue(f('averageMovingSpeed',1280,'s/km'),'m')).toBe('2:08/100 m');
  expect(fieldValue(f('maxSpeed',43.2,'km/h'),'km')).toBe('43.2 km/h');
  expect(fieldValue(f('maxSpeed',32.18688,'km/h'),'mi')).toBe('20 mph');
  expect(fieldValue(f('minElevation',100,'m'),'mi')).toBe('328 ft');
  expect(fieldValue(f('elevationLoss',0,'m'),'km')).toBe('0 m');
 });
 it('reads running form as Garmin does: stride in metres, bounce in centimetres',()=>{
  expect(fieldValue(f('strideLength',113.4,'cm'),'km')).toBe('1.13 m');
  expect(fieldValue(f('strideLength',91.44,'cm'),'mi')).toBe('3 ft');
  expect(fieldValue(f('verticalOscillation',8.4,'cm'),'km')).toBe('8.4 cm');
  expect(fieldValue(f('verticalOscillation',7.62,'cm'),'mi')).toBe('3 in');
  expect(fieldValue(f('verticalRatio',7.44,'%'),'km')).toBe('7.4%');
  expect(fieldValue(f('groundContactTime',238,'ms'),'km')).toBe('238 ms');
  expect(fieldValue(f('avgStepSpeedLoss',15.47,'cm/s'),'km')).toBe('15.5 cm/s');
 });
 it('reads strokes, fluid and power in their own units',()=>{
  expect(fieldValue(f('averageStrokeDistance',1.6,'m'),'m')).toBe('1.6 m');
  expect(fieldValue(f('averageStrokeDistance',1.524,'m'),'yd')).toBe('5 ft');
  expect(streamScale('strokeDistance','yd')?.unit).toBe('ft');
  expect(fieldValue(f('waterEstimated',820,'ml'),'km')).toBe('820 ml');
  expect(fieldValue(f('waterEstimated',591.47,'ml'),'mi')).toBe('20 fl oz');
  expect(fieldValue(f('intensityFactor',0.8,''),'km')).toBe('0.80');
  expect(fieldValue(f('leftBalance',47.69,'%'),'km')).toBe('47.7%');
  expect(fieldValue(f('trainingStressScore',75.8,''),'km')).toBe('75.8');
  expect(fieldValue(f('movingDuration',3101,'s'),'km')).toBe('51:41');
  expect(fieldValue(f('poolLength',25,'yd'),'yd')).toBe('25 yd');
 });
 it('names groups by sport and scales the streams that need it',()=>{
  expect([groupName('form','run'),groupName('form','bike'),groupName('form','walk')]).toEqual(['Running form','Cadence','Steps and cadence']);
  expect([groupName('pace','bike'),groupName('pace','swim'),groupName('strokes','swim'),groupName('strokes','other')]).toEqual(['Speed','Pace','Swim','Strokes']);
  expect(streamScale('strideLength','km')?.conv(113)).toBeCloseTo(1.13);
  expect(streamScale('strideLength','mi')?.unit).toBe('ft');
  expect(streamScale('groundContactTime','mi')).toBeNull();
 });
});
