// Names, units and ring colours for weekly goal kinds. The counting is the core's.
import {colors,goalColors} from './charts';
import type {GoalKind} from './generated/GoalKind';
import type {GoalTargets} from './generated/GoalTargets';
import type {Sport} from './generated/Sport';
import type {ItemState} from './generated/ItemState';
import type {Allocation} from './generated/Allocation';
import type {BacklogItem} from './generated/BacklogItem';
import type {IconName} from './components/Icon.svelte';

export const GOALS: Record<GoalKind, {name: string, unit: string, one: string, color: string, icon: IconName}> = {
 strength: {name: 'Strength', unit: 'sessions', one: 'session', color: goalColors.strength, icon: 'strength'},
 zone2: {name: 'Zone 2', unit: 'min', one: 'min', color: goalColors.zone2, icon: 'pulse'},
 highIntensity: {name: 'Zone 4–5', unit: 'min', one: 'min', color: goalColors.highIntensity, icon: 'flame'},
 intensity: {name: 'Intensity', unit: 'min', one: 'min', color: goalColors.intensity, icon: 'flame'},
 sleepTiming: {name: 'Sleep timing', unit: 'nights', one: 'night', color: goalColors.sleep, icon: 'moon'},
 sleepScore: {name: 'Sleep Score', unit: 'avg', one: 'avg', color: goalColors.sleep, icon: 'moon'},
};
export const unitFor = (kind: GoalKind, n: number) => n === 1 ? GOALS[kind].one : GOALS[kind].unit;
export const amount = (kind: GoalKind, n: number) => `${Math.round(n)} ${unitFor(kind, n)}`;

export const SPORT_NAMES: Record<Sport, string> = {running: 'Run', cycling: 'Ride', walking: 'Walk', hiking: 'Hike', swimming: 'Swim', strength: 'Strength', other: 'Session'};
export const SPORT_COLORS: Record<Sport, string> = {running: colors.good, cycling: colors.amount, walking: '#BCD0A3', hiking: '#CEB184', swimming: goalColors.intensity, strength: goalColors.strength, other: '#B5A9D6'};
export const SPORT_ICONS: Record<Sport, IconName> = {running: 'run', cycling: 'bike', walking: 'walk', hiking: 'hike', swimming: 'swim', strength: 'strength', other: 'other'};
// Garmin Coach cardio has no sport yet: a neutral cardio icon, not the alert-like 'other'.
export const itemIcon = (p: {intention: {sport: Sport}, coach: {kind: string | null} | null}): IconName => p.coach?.kind === 'cardio' ? 'pulse' : SPORT_ICONS[p.intention.sport];

/** One word per state, the same everywhere. */
export const STATE_WORD: Record<ItemState, string> = {done: 'Done', review: 'Needs review', planned: 'Planned', notRecorded: 'Not recorded', awaiting: 'Awaiting data', letGo: 'Let go', missed: 'Marked missed'};

export const noGoals = (): GoalTargets => ({strengthSessions: 0, zone2Min: 0, highIntensityMin: 0, intensityMin: 0, intensityGarminGoal: false, sleepNights: 0, sleepTime: null, sleepKind: 'asleepBy', sleepScore: 0, restDays: []});

const CHUNK: Record<string, [keyof Allocation, number]> = {zone2: ['zone2Min', 45], highIntensity: ['highIntensityMin', 20], intensity: ['intensityMin', 30]};
// A backlog item taken for a day: one session, or a share of the minutes
// left; the whole remainder when less than ten minutes would be left over.
export function backlogDraft(b: BacklogItem, date: string): {date: string, sport: Sport, allocation: Partial<Allocation>} {
  if (b.kind === 'strength') return {date, sport: 'strength', allocation: {strengthSessions: 1}};
  const [field, chunk] = CHUNK[b.kind] ?? ['zone2Min', 30];
  return {date, sport: b.sport ?? 'running', allocation: {[field]: b.amount - chunk < 10 ? b.amount : chunk}};
}
