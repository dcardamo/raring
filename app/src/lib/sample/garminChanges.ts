// Synthetic change-log entries for the `garmin-changes` preview fixture: one per kind of result.
import type {GarminChange} from '../generated/GarminChange';
const time={field:'alarm:7.alarmTime',label:'Alarm time',before:'06:30',after:'07:00'};
export const sampleGarminChanges=[
 {at:'2025-10-16T07:40:00Z',resource:'Watch alarms',fields:[time],outcome:'applied',message:null,mismatches:[],waitingForWatch:true},
 {at:'2025-10-16T07:20:00Z',resource:'Watch alarms',fields:[time],outcome:'differs',message:null,mismatches:[{...time,before:'07:00',after:'06:30'}],waitingForWatch:false},
 {at:'2025-10-15T21:05:00Z',resource:'Watch alarms',fields:[{field:'alarm:8.alarmDays',label:'Alarm days',before:'Once',after:'Monday, Friday'}],outcome:'rejected',message:'Invalid alarm days',mismatches:[],waitingForWatch:false},
 {at:'2025-10-15T21:00:00Z',resource:'Watch alarms',fields:[time],outcome:'conflict',message:'Changed in Garmin since you looked: alarm time is now 06:40',mismatches:[],waitingForWatch:false},
 {at:'2025-10-14T18:00:00Z',resource:'Watch alarms',fields:[{field:'alarm:9',label:'Alarm removed',before:'05:45',after:null}],outcome:'failed',message:'Garmin is limiting requests; try again later',mismatches:[],waitingForWatch:false},
] satisfies GarminChange[];
