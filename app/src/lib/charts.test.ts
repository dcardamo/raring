import {describe,expect,it} from 'vitest';
import {batteryColor,colors,toneColor} from './charts';

describe('color roles',()=>{
 it('uses status colors for Garmin tones, not screen colors',()=>{
  expect(['good','fair','poor','neutral'].map(toneColor)).toEqual([colors.good,colors.fair,colors.poor,colors.secondary]);
 });
 it('keeps green out of the effort ramp',()=>{
  expect(colors.z).toEqual(['#7D879C','#5CA8FF','#FFD666','#FF9F43','#FF6B6B']);
 });
 it('colors the reported Body Battery level',()=>{
  expect([0,29,30,69,70,100].map(batteryColor)).toEqual([colors.poor,colors.poor,colors.fair,colors.fair,colors.good,colors.good]);
 });
});
