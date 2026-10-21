import {describe,it,expect} from 'vitest';
import {monthDays,shiftMonth} from './calendar';

describe('activity calendar months',()=>{
 it('steps across years from any day of a month',()=>{
  expect(shiftMonth('2025-01-31',-1)).toBe('2024-12-01');
  expect(shiftMonth('2024-12-01',1)).toBe('2025-01-01');
  expect(shiftMonth('2025-10-01',-13)).toBe('2024-09-01');
 });
 it('lists every day, leap February included',()=>{
  expect(monthDays('2024-02-01').at(-1)).toBe('2024-02-29');
  expect(monthDays('2025-02-01')).toHaveLength(28);
  expect(monthDays('2025-10-01')[0]).toBe('2025-10-01');
  expect(monthDays('2025-10-01').at(-1)).toBe('2025-10-31');
 });
});
