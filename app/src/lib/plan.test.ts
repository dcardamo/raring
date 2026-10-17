import {describe,expect,it} from 'vitest';
import {delta,weekOrder,weekRange} from './plan';

describe('plan helpers',()=>{
 it('orders weekdays from Garmin\'s first day',()=>{
  expect(weekOrder(0)).toEqual([0,1,2,3,4,5,6]);
  expect(weekOrder(6)).toEqual([6,0,1,2,3,4,5]);
 });
 it('labels a week and its change',()=>{
  expect(weekRange('2025-10-13','2025-10-19')).toBe('Mon 13 – Sun 19 Oct');
  expect(weekRange('2025-09-29','2025-10-05')).toBe('Mon 29 Sept – Sun 5 Oct');
  expect(delta(162,150)).toBe('▲ 12');
  expect(delta(4,5)).toBe('▼ 1');
  expect(delta(3,3)).toBe('same');
  expect(delta(3,null)).toBe('');
 });
});
