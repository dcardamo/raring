import type {PlanWeek} from '../generated/PlanWeek';

// Synthetic UI evidence states, also exercised against Rust in plan_evidence.rs.
export function missingPlanEvidence(week:PlanWeek,all:boolean):PlanWeek {
 return {...week,headline:all?'Not enough evidence':'Partial evidence',
  detail:all?'4 targets awaiting data':'3 targets awaiting data',
  total:all?0:1,onTrack:all?0:1,reached:all?0:1,completion:all?null:1,
  items:week.items.map(item=>item.kind==='intensity'&&!all?
   {...item,value:item.target,completion:1,state:'done',note:"Garmin's weekly goal · Reached",unavailable:null}:
   {...item,state:'onTrack',unavailable:({strength:'Workout list incomplete',zone2:'Heart-rate zones incomplete',intensity:'Intensity summaries incomplete',asleepOnTime:'Needs a saved sleep plan and recorded night'})[item.kind]})};
}
