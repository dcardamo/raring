// Shared text and time handling for writes to Garmin (Rules: Garmin writes).
import type {GarminChange} from './generated/GarminChange';
import type {WriteOutcome} from './generated/WriteOutcome';
import type {WeightUnit} from './generated/WeightUnit';

export const RESULT:Record<WriteOutcome,string>={applied:'Saved in Garmin',differs:'Saved, but Garmin reads back something else',unconfirmed:'Sent, not confirmed',conflict:'Not sent',rejected:'Garmin refused it',failed:'Not saved',needsSignIn:'Not sent: sign in again',duplicate:'Not sent: already in Garmin'};

// The core's accepted range, 20–300 kg, in the person's unit.
export const WEIGHT_BOUNDS:Record<WeightUnit,[number,number]>={kg:[20,300],lb:[44.1,661.3]};

const pad=(n:number)=>String(n).padStart(2,'0');
/** A local time as a datetime-local input holds it, `YYYY-MM-DDTHH:MM`. */
export const localInput=(d:Date)=>`${d.getFullYear()}-${pad(d.getMonth()+1)}-${pad(d.getDate())}T${pad(d.getHours())}:${pad(d.getMinutes())}`;
/** That local time with this device's UTC offset for it (RFC 3339), or null. */
export function withOffset(local:string):string|null{
 if(!/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}$/.test(local))return null;
 const d=new Date(local);
 if(Number.isNaN(d.getTime())||localInput(d)!==local)return null;
 const offset=-d.getTimezoneOffset(),sign=offset<0?'-':'+',abs=Math.abs(offset);
 return `${local}:00${sign}${pad(Math.floor(abs/60))}:${pad(abs%60)}`;
}

// Garmin accepted an add but its list doesn't show it yet: the list may lag, or the weigh-in may be
// filed under another date, so a retry could add a second one.
const NOT_LISTED='weigh-ins don’t show it yet';
function unconfirmedAdvice(change:GarminChange):string{
 const field=change.fields[0]?.field??'';
 if(!field.startsWith('weighIn:'))return 'Check Garmin Connect to see whether it changed';
 if(change.fields[0].after===null)return 'Try again: Raring reads Garmin first, so only this weigh-in can be deleted';
 if((change.message??'').replace(/'/g,'’').includes(NOT_LISTED))return 'Wait a few minutes or check Garmin Connect before adding it again';
 return 'Try again: Raring checks Garmin first, so it won’t be saved twice';
}

/** One line for a write's result: Garmin's answer, its message and what differs. */
export function writeSummary(change:GarminChange):string{
 const parts=[RESULT[change.outcome]];
 if(change.message)parts.push(change.message);
 for(const m of change.mismatches)parts.push(m.before===null&&m.after===null?`${m.label} came back different`:`${m.label}: sent ${m.before??'none'}, Garmin has ${m.after??'none'}`);
 if(change.outcome==='unconfirmed')parts.push(unconfirmedAdvice(change));
 return parts.join('. ')+'.';
}
/** A refusal before anything was sent, as the shell reports it. */
export const refusal=(e:unknown)=>String(e instanceof Error?e.message:e).replace(/^Error:\s*/,'')||'Nothing was sent to Garmin';
