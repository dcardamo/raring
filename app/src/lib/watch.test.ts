import {describe,expect,it} from 'vitest';
import {watchAge,watchLabel,watchNeedsSync} from './watch';
import sampleToday from './sample/today.json';
import type {TodayView} from './generated/TodayView';
const now=new Date('2025-10-16T14:00:00Z');
const view={...sampleToday,watchLastUpload:'2025-10-16T12:00:00Z',watchSleepUploadStale:false} as unknown as TodayView;
describe('watch upload freshness',()=>{
 it('distinguishes Garmin upload from app refresh, rejecting future or absent uploads',()=>{
  expect(watchLabel(view.watchLastUpload,now)).toBe('Watch synced 2 h ago');
  expect(watchLabel(null,now)).toBeNull();
  expect(watchAge('2025-10-17T14:00:00Z',now)).toBeNull();
 });
 it('asks gently when old while awake or sleep upload is missing',()=>{
  const awake=new Date(2025,9,16,14);
  const old=new Date(awake.getTime()-6*3600_000).toISOString();
  expect(watchNeedsSync({...view,watchLastUpload:old},awake)).toBe(true);
  expect(watchNeedsSync({...view,watchLastUpload:old},new Date(2025,9,16,3))).toBe(false);
  expect(watchNeedsSync({...view,watchLastUpload:null,watchSleepUploadStale:true},awake)).toBe(true);
  expect(watchNeedsSync({...view,watchLastUpload:null,watchSleepUploadStale:false},awake)).toBe(false);
 });
});
