import type {TodayView} from './generated/TodayView';

// Plan and phase times are local wall-clock ISO strings, which Date reads as local time.
const local=(iso:string|null)=>iso?new Date(iso).getTime():NaN;

// The usual wake is credible from four recorded wakes in 14 nights (spec Rules: Time of day).
export const wakeLabel=(nights:number)=>nights>=14?'usual':nights>=4?`usual · ${nights} nights`:'reference';

// A cached Today is stale once its phase has ended; reading it again re-evaluates the phase.
export function phaseEnded(view:TodayView, now=Date.now()):boolean {
 const until=local(view.focus.until);
 return Number.isFinite(until) && until<=now;
}
