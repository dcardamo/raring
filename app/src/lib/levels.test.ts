import {describe,it,expect} from 'vitest';
import {chartScale,chartColor,gradientStops} from './levels';
import {colors} from './charts';

describe('chart colour categories',()=>{
 it('colours Garmin levels at Garmin’s thresholds',()=>{
  const readiness=chartScale('readiness');
  expect([49,50,74,75].map(readiness.tone)).toEqual(['poor','fair','fair','good']);
  const sleep=chartScale('sleepScore');
  expect([59,60,79,80].map(sleep.tone)).toEqual(['poor','fair','fair','good']);
  // Rest and low stress are good; medium fair; high poor.
  const stress=chartScale('stress');
  expect([0,25,50,51,75,76,100].map(stress.tone)).toEqual(['good','good','good','fair','fair','poor','poor']);
  const battery=chartScale('bodyBattery');
  expect([29,30,69,70].map(battery.tone)).toEqual(['poor','fair','fair','good']);
  expect(battery.key.map(k=>k.tone)).toEqual(['poor','fair','good']);
 });
 it('colours a usual range’s readings inside good and outside fair, bounds included',()=>{
  const hrv=chartScale('hrv',{band:[48,62]});
  expect([47,48,55,62,63].map(hrv.tone)).toEqual(['fair','good','good','good','fair']);
  expect(hrv.key).toEqual([{tone:'fair',label:'Outside it'}]);
 });
 it('leaves a range metric neutral without its band, as on weekly and monthly averages',()=>{
  for(const m of ['hrv','restingHr','sleepHr','respiration','spo2','skinTemp','acuteLoad'] as const){
   expect(chartScale(m).edges).toEqual([]);
   expect(chartScale(m).tone(50)).toBe('neutral');
  }
 });
 it('rates acute load against Garmin’s optimal range: below fair, above poor',()=>{
  const load=chartScale('acuteLoad',{band:[300,600]});
  expect([299,300,600,601].map(load.tone)).toEqual(['fair','good','good','poor']);
 });
 it('compares Fitness Age with your age, and is neutral without it',()=>{
  const age=chartScale('fitnessAge',{age:46});
  expect([45,46,47].map(age.tone)).toEqual(['good','fair','poor']);
  expect(age.key.map(k=>k.tone)).toEqual(['good','fair','poor']);
  expect(chartScale('fitnessAge').tone(40)).toBe('neutral');
 });
 it('draws readings without a verdict in the neutral blue, never ink',()=>{
  for(const m of ['vo2max','sleepTime',null] as const)expect(chartScale(m).tone(50)).toBe('neutral');
  expect(chartColor('neutral')).toBe(colors.neutral);
  expect(chartColor('neutral')).not.toBe(colors.ink);
  expect(chartColor('fair')).toBe(colors.fair);
 });
 it('switches gradient colours exactly at each edge inside the plotted domain',()=>{
  // Domain 40–70, band 48–62: top fair, good between 62 and 48, fair below.
  expect(gradientStops(chartScale('hrv',{band:[48,62]}),[40,70])).toEqual([
   {offset:0,color:colors.fair},
   {offset:8/30,color:colors.fair},{offset:8/30,color:colors.good},
   {offset:22/30,color:colors.good},{offset:22/30,color:colors.fair},
   {offset:1,color:colors.fair},
  ]);
  // A fixed 0–100 domain keeps both stress edges.
  expect(gradientStops(chartScale('stress'),[0,100]).map(s=>s.offset)).toEqual([0,.24,.24,.49,.49,1]);
  // Edges outside the domain leave one colour; neutral scales draw no gradient.
  expect(gradientStops(chartScale('hrv',{band:[48,62]}),[50,60])).toEqual([{offset:0,color:colors.good},{offset:1,color:colors.good}]);
  expect(gradientStops(chartScale('vo2max'),[40,60])).toEqual([]);
 });
 it('switches at an edge that lies on either bound instead of blending across the plot',()=>{
  // Stress plotted 0–51: everything below 51 is good, so the colour after the top stop is good.
  const top=gradientStops(chartScale('stress'),[0,51]);
  expect(top.filter(s=>s.offset===0).at(-1)!.color).toBe(colors.good);
  expect(top.at(-1)).toEqual({offset:1,color:colors.good});
  // A band ending on the bottom bound keeps the plot above it good up to that bound.
  const bottom=gradientStops(chartScale('hrv',{band:[48,62]}),[48,70]);
  expect(bottom.filter(s=>s.offset<1).at(-1)).toEqual({offset:8/22,color:colors.good});
  expect(bottom.filter(s=>s.offset===1)[0].color).toBe(colors.good);
 });
});
