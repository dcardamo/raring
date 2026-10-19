// Names, units and ring colours for weekly goal kinds. The counting is the core's.
import type {GoalKind} from './generated/GoalKind';
import type {GoalTargets} from './generated/GoalTargets';
import type {Sport} from './generated/Sport';
import type {ItemState} from './generated/ItemState';
import type {IconName} from './components/Icon.svelte';

export const GOALS: Record<GoalKind, {name: string, unit: string, one: string, color: string, icon: IconName}> = {
 strength: {name: 'Strength', unit: 'sessions', one: 'session', color: '#FF7DB8', icon: 'strength'},
 zone2: {name: 'Zone 2', unit: 'min', one: 'min', color: '#5CA8FF', icon: 'pulse'},
 highIntensity: {name: 'Zone 4–5', unit: 'min', one: 'min', color: '#FF8A5C', icon: 'flame'},
 intensity: {name: 'Intensity', unit: 'min', one: 'min', color: '#36C9C6', icon: 'flame'},
 sleepTiming: {name: 'Sleep timing', unit: 'nights', one: 'night', color: '#A9C8FF', icon: 'moon'},
 sleepScore: {name: 'Sleep Score', unit: 'avg', one: 'avg', color: '#A9C8FF', icon: 'moon'},
};
export const unitFor = (kind: GoalKind, n: number) => n === 1 ? GOALS[kind].one : GOALS[kind].unit;
export const amount = (kind: GoalKind, n: number) => `${Math.round(n)} ${unitFor(kind, n)}`;

export const SPORT_NAMES: Record<Sport, string> = {running: 'Run', cycling: 'Ride', walking: 'Walk', hiking: 'Hike', swimming: 'Swim', strength: 'Strength', other: 'Session'};
export const SPORT_COLORS: Record<Sport, string> = {running: '#3DDC97', cycling: '#5CA8FF', walking: '#9FD36B', hiking: '#C9A35C', swimming: '#36C9C6', strength: '#FF7DB8', other: '#A48BFF'};
export const SPORT_ICONS: Record<Sport, IconName> = {running: 'run', cycling: 'bike', walking: 'walk', hiking: 'hike', swimming: 'swim', strength: 'strength', other: 'other'};

/** One word per state, the same everywhere. */
export const STATE_WORD: Record<ItemState, string> = {done: 'Done', review: 'Needs review', planned: 'Planned', notRecorded: 'Not recorded', awaiting: 'Awaiting data', letGo: 'Let go', missed: 'Marked missed'};

export const noGoals = (): GoalTargets => ({strengthSessions: 0, zone2Min: 0, highIntensityMin: 0, intensityMin: 0, intensityGarminGoal: false, sleepNights: 0, sleepTime: null, sleepKind: 'asleepBy', sleepScore: 0, restDays: []});
