import type {TrendMetric} from './generated/TrendMetric';
import type {TrendRange} from './generated/TrendRange';
/** Optional caller-owned chart state; absent in the production shell. */
export type ChartState = {range:TrendRange,offset:number,primary:TrendMetric|null,customStart:string,customEnd:string,comparisonStart:string,compare:'off'|'previous'|'metric',overlayMetric:TrendMetric|null};
export type ChartStateContext = (key:string) => {initial:ChartState|undefined,save:(state:ChartState)=>void};
