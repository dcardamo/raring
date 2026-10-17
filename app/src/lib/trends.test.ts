import {describe, expect, it} from 'vitest';
import {bucketFor, localPoints, monday, period, pointLabel, spans, trendValue, changeValue} from './trends';
import {sampleTrend} from './sample/trend';

describe('trend periods', () => {
  it('matches the core: days to 90, weeks to two years, months beyond', () => {
    expect([bucketFor(90), bucketFor(91), bucketFor(731), bucketFor(732)]).toEqual(['day', 'week', 'week', 'month']);
  });
  it('builds a year from 52 Monday weeks ending today', () => {
    const {from, bucket} = period('year', '2026-10-15', null);
    expect(monday('2026-10-15')).toBe('2026-10-12');
    expect(spans(from, '2026-10-15', bucket)).toHaveLength(52);
  });
  it('clips the last month to today', () => {
    expect(spans('2025-01-01', '2025-02-10', 'month')).toEqual([['2025-01-01', '2025-01-31'], ['2025-02-01', '2025-02-10']]);
  });
  it('labels weeks and months', () => {
    expect(pointLabel({start: '2026-09-21', end: '2026-09-27', value: 1, days: 7}, 'week')).toBe('Week of 21 Sept 2026');
    expect(pointLabel({start: '2026-07-01', end: '2026-07-31', value: 1, days: 31}, 'month')).toBe('July 2026');
  });
  it('keeps missing local days as gaps', () => {
    expect(localPoints([5, null, 7], '2026-01-03', 3).map(p => [p.start, p.value, p.days]))
      .toEqual([['2026-01-01', 5, 1], ['2026-01-02', null, 0], ['2026-01-03', 7, 1]]);
  });
  it('shows fractional average changes even for whole-number metrics',()=>{
    expect(changeValue(.4,'°C',1)).toBe('+0.4');
    expect(changeValue(-.3,'br/min',1)).toBe('−0.3');
    expect(changeValue(.4,'ms',0)).toBe('+0.4');
    expect(changeValue(-.04,'',0)).toBe('−<0.1');
    expect(changeValue(.2,'min',0)).toBe('+<1 min');
    expect(changeValue(0,'ms',0)).toBe('0');
  });
  it('rejects cleared or reversed custom dates in the preview',()=>{
    const q={metric:'hrv' as const,range:'custom' as const,start:'',end:'2025-10-16',overlay:null,previous:false,comparisonStart:null};
    expect(()=>sampleTrend(q)).toThrow('Invalid trend date');
    expect(()=>sampleTrend({...q,start:null})).toThrow('Choose a block start');
    expect(()=>sampleTrend({...q,start:'2025-10-17'})).toThrow('no later than today');
    expect(()=>sampleTrend({...q,start:'2025-10-01',end:''})).toThrow('Invalid trend date');
  });
  it('keeps every preview date between the first day and today, like the core',()=>{
    const q={metric:'hrv' as const,range:'days30' as const,start:null,end:null,overlay:null,previous:true,comparisonStart:null};
    expect(()=>sampleTrend({...q,end:'2025-10-17'})).toThrow('no later than today');
    expect(()=>sampleTrend({...q,comparisonStart:'2025-10-17'})).toThrow('no later than today');
    expect(()=>sampleTrend({...q,comparisonStart:'2025-10-01'})).toThrow('runs past today');
    expect(()=>sampleTrend({...q,range:'custom',start:'1989-12-31'})).toThrow('from 1990-01-01');
    expect(sampleTrend({...q,comparisonStart:'2025-09-01'}).previous?.points[0].start).toBe('2025-09-01');
  });
  it('reads minutes as hours', () => {
    expect(trendValue(453, 'min')).toBe('7:33');
    expect(trendValue(14.25, 'br/min', 1)).toBe('14.3');
  });
  it('offers no period before All in the preview data', () => {
    expect(sampleTrend({metric: 'stress', range: 'all', end: null, overlay: null, previous: true, start:null, comparisonStart:null}).previous).toBeNull();
  });
});
