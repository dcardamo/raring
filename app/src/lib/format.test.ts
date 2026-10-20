import {describe,it,expect} from 'vitest';
import {duration,elapsed,pace,clock,dateLabel,dateLabelIn,dayMonthYear,hoursMinutes,monthLabel,shortMonth,signed,signedTime,num,readAt} from './format';
describe('Garmin formatting',()=>{
 it('dates a read time unless it was today',()=>{
  const now=new Date('2025-10-16T21:00:00');
  expect(readAt(new Date('2025-10-16T09:05:00').toISOString(),now)).toBe('09:05');
  expect(readAt(new Date('2025-10-12T09:05:00').toISOString(),now)).toBe('Sun 09:05');
  expect(readAt('nonsense',now)).toBe('—');
 });
 it('renders durations and times',()=>{expect(duration(468)).toBe('7:48');expect(duration(null)).toBe('—');expect(elapsed(3134)).toBe('52:14');expect(elapsed(5050)).toBe('1:24:10');expect(clock('2025-10-16T06:12:00')).toBe('06:12')});
 it('renders paces and deltas',()=>{expect(pace(307)).toBe('5:07/km');expect(signed(-4,'m')).toBe('−4m');expect(signed(1)).toBe('+1');expect(signed(0)).toBe('');expect(signed(0.8,'m')).toBe('');expect(signed(1.714285)).toBe('+1.7');expect(num(null)).toBe('—')});
 it('renders a local calendar date without UTC rollover',()=>{expect(dateLabel('2025-10-16')).toBe('Thu 16 Oct');expect(dateLabel('')).toBe('Date unavailable')});
 it('builds date labels by hand so every WebView matches',()=>{
  // Android's en-GB WebView used to insert a comma after the weekday.
  expect(dateLabel('2025-09-05')).toBe('Fri 5 Sept');
  expect(dateLabel('2025-12-01')).toBe('Mon 1 Dec');
  expect(dateLabel('2025-10-16T07:10:00')).toBe('Thu 16 Oct');
  expect(dateLabel('not-a-date')).toBe('Date unavailable');
  expect(shortMonth('2025-09-05')).toBe('Sept');
  expect(monthLabel('2025-10-01')).toBe('Oct 2025');
  expect(monthLabel('2025-09-01')).toBe('Sept 2025');
  expect(monthLabel('')).toBe('Date unavailable');
 });
 it('keeps Garmin wall times but converts offset timestamps to device time',()=>{
  expect(clock('2025-10-16 06:12:00')).toBe('06:12');
  const instant='2025-10-16T20:12:00Z';
  expect(clock(instant)).toBe(new Date(instant).toLocaleTimeString('en-GB',{hour:'2-digit',minute:'2-digit'}));
  expect(clock('invalid')).toBe('—');expect(clock(null)).toBe('—');
 });
 it('writes amounts of time and dates whose year matters',()=>{
  expect(hoursMinutes(1194)).toBe('19 h 54 min');expect(hoursMinutes(16701)).toBe('278 h');expect(hoursMinutes(45)).toBe('45 min');expect(hoursMinutes(null)).toBe('—');
  expect(signedTime(185)).toBe('+3 h 05');expect(signedTime(-44)).toBe('−44 min');expect(signedTime(0)).toBe('');
  expect(dayMonthYear('2025-10-23')).toBe('23 Oct 2025');
  expect(dateLabelIn('2025-10-23','2026-10-22')).toBe('Thu 23 Oct 2025');expect(dateLabelIn('2026-10-19','2026-10-22')).toBe('Mon 19 Oct');
 });
});
