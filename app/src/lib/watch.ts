import type {TodayView} from './generated/TodayView';

// Garmin's upload time, not Raring's last API fetch. Invalid/future times are unknown.
export function watchAge(upload:string|null, now:Date):number|null {
 const time=upload?Date.parse(upload):NaN;
 const minutes=(now.getTime()-time)/60000;
 return Number.isFinite(minutes) && minutes>=0?minutes:null;
}
export function watchLabel(upload:string|null, now:Date):string|null {
 const age=watchAge(upload,now);
 if(age===null)return null;
 if(age<60)return `Watch synced ${Math.max(1,Math.floor(age))} min ago`;
 if(age<1440)return `Watch synced ${Math.floor(age/60)} h ago`;
 return `Watch synced ${Math.floor(age/1440)} d ago`;
}
export function watchNeedsSync(view:TodayView, now:Date):boolean {
 const age=watchAge(view.watchLastUpload,now);
 // Avoid a nag during the user's likely sleep period; missing sleep after
 // the usual wake time is handled separately by the core.
 return view.watchSleepUploadStale || (age!==null && age>=360 && now.getHours()>=7 && now.getHours()<22);
}
