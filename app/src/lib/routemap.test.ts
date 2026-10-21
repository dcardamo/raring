import {describe,expect,it} from 'vitest';
import {fit,mercator} from './routemap';
import sample from './sample/workout.json';

describe('route card fit',()=>{
 it('places the route exactly where the core framed its street map',()=>{
  const frame=fit(sample.route.map(p=>mercator(p as [number,number])!))!;
  const core=sample.map.frame;
  expect(Math.abs(frame.x0-core.x0)*core.scale).toBeLessThan(1e-6);
  expect(Math.abs(frame.y0-core.y0)*core.scale).toBeLessThan(1e-6);
  expect(Math.abs(frame.scale/core.scale-1)).toBeLessThan(1e-12);
 });
 it('zooms a standing start in only so far and skips points off the map',()=>{
  expect(fit([[0.5,0.5],[0.5,0.5]])?.scale).toBe(256*65536);
  expect(fit([[0.5,0.5]])).toBeNull();
  expect(mercator([89,0])).toBeNull();
  expect(mercator([Number.NaN,0])).toBeNull();
 });
});
