import {describe,it,expect} from 'vitest';
import {duration,elapsed,pace,clock,dateLabel,signed,num} from './format';
describe('Garmin formatting',()=>{
 it('renders durations and times',()=>{expect(duration(468)).toBe('7:48');expect(duration(null)).toBe('—');expect(elapsed(3134)).toBe('52:14');expect(elapsed(5050)).toBe('1:24:10');expect(clock('2025-10-16T06:12:00')).toBe('06:12')});
 it('renders paces and deltas',()=>{expect(pace(307)).toBe('5:07/km');expect(signed(-4,'m')).toBe('−4m');expect(signed(1)).toBe('+1');expect(signed(0)).toBe('0');expect(signed(1.714285)).toBe('+1.7');expect(num(null)).toBe('—')});
 it('renders a local calendar date without UTC rollover',()=>{expect(dateLabel('2025-10-16')).toBe('Thu 16 Oct')});
});
