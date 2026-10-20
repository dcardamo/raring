import {describe,it,expect} from 'vitest';
import {distance,pace,paceUnit,speed,speedUnit,elevation,elevationUnit} from './units';
describe('activity units',()=>{
 it('reads distance in the activity unit, trimmed the same everywhere',()=>{
  expect(distance(10.2,'km')).toBe('10.2 km');
  expect(distance(10,'km')).toBe('10 km');
  expect(distance(16.09344,'mi')).toBe('10 mi');
  expect(distance(1.5,'m')).toBe('1500 m');
  expect(distance(1.3716,'yd')).toBe('1500 yd');
  expect(distance(null,'km')).toBe('—');
 });
 it('paces runs per km or mile and swims per 100 m or yards',()=>{
  expect(pace(300,'km')+paceUnit('km')).toBe('5:00/km');
  expect(pace(300,'mi')+paceUnit('mi')).toBe('8:03/mi');
  expect(pace(1280,'m')+paceUnit('m')).toBe('2:08/100 m');
  expect(pace(1280,'yd')+paceUnit('yd')).toBe('1:57/100 yd');
 });
 it('converts speed and elevation for statute users',()=>{
  expect(speed(32.18688,'mi')).toBeCloseTo(20);expect(speedUnit('mi')).toBe('mph');expect(speedUnit('km')).toBe('km/h');
  expect(elevation(100,'mi')).toBeCloseTo(328.084);expect(elevationUnit('yd')).toBe('ft');expect(elevation(100,'km')).toBe(100);
 });
});
