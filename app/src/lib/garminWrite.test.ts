import {describe,expect,it} from 'vitest';
import {localInput,withOffset,writeSummary,refusal} from './garminWrite';
import type {GarminChange} from './generated/GarminChange';

const change=(over:Partial<GarminChange>):GarminChange=>({at:'2026-10-21T12:00:00Z',resource:'Weight',fields:[],outcome:'applied',message:null,mismatches:[],waitingForWatch:false,...over});

describe('garmin writes',()=>{
 it('sends the picked local time with this device’s offset for that moment',()=>{
  const at=withOffset('2026-10-21T07:58');
  expect(at).toMatch(/^2026-10-21T07:58:00[+-]\d{2}:\d{2}$/);
  // The same instant read back is the time the person picked.
  expect(localInput(new Date(at!))).toBe('2026-10-21T07:58');
  for(const bad of ['','2026-10-21','2026-02-30T07:58','2026-10-21T25:00'])expect(withOffset(bad)).toBeNull();
 });
 it('summarises Garmin’s answer, its message and differences',()=>{
  expect(writeSummary(change({}))).toBe('Saved in Garmin.');
  expect(writeSummary(change({outcome:'duplicate',message:'Garmin already has 81.3 kg at 06:58 on 21 Oct that day'}))).toBe('Not sent: already in Garmin. Garmin already has 81.3 kg at 06:58 on 21 Oct that day.');
  expect(writeSummary(change({outcome:'differs',mismatches:[{field:'weight',label:'Weight',before:'165.0 lb',after:'154.3 lb'}]}))).toBe('Saved, but Garmin reads back something else. Weight: sent 165.0 lb, Garmin has 154.3 lb.');
  expect(writeSummary(change({outcome:'unconfirmed',message:'Couldn’t reach Garmin; Garmin may or may not have saved it'}))).toMatch(/Sent, not confirmed\. .*Check Garmin Connect to see whether it changed\.$/);
  const added={field:'weighIn:1',label:'Weigh-in added',before:null,after:'81.3 kg at 07:58 on 21 Oct'};
  const deleted={field:'weighIn:1',label:'Weigh-in deleted',before:'81.3 kg at 07:58 on 21 Oct',after:null};
  expect(writeSummary(change({outcome:'unconfirmed',fields:[added],message:"Garmin accepted it, but its weigh-ins don't show it yet"}))).toMatch(/Wait a few minutes or check Garmin Connect before adding it again\.$/);
  expect(writeSummary(change({outcome:'unconfirmed',fields:[added],message:'Saved, but Garmin’s read-back failed'}))).toMatch(/won’t be saved twice\.$/);
  expect(writeSummary(change({outcome:'unconfirmed',fields:[deleted],message:'Garmin answered with error 502; Garmin may or may not have saved it'}))).toMatch(/only this weigh-in can be deleted\.$/);
  expect(refusal(new Error('Not connected to Garmin; nothing was sent'))).toBe('Not connected to Garmin; nothing was sent');
  expect(refusal('Error: Account verification required')).toBe('Account verification required');
 });
});
