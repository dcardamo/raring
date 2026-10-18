import type {Icon} from '../../lib/generated/Icon';

export type Sport = 'strength' | 'running' | 'cycling' | 'walking' | 'hiit' | 'swimming' | 'yoga' | 'hiking' | 'other';

// Activity colours stay apart from heart-rate zone and tone colours, and always
// travel with a glyph so no state or sport is told by colour alone.
export const SPORTS: Record<Sport, {name: string, plural: string, icon: Icon, color: string}> = {
  running: {name: 'Run', plural: 'Running', icon: 'run', color: '#36C9C6'},
  cycling: {name: 'Ride', plural: 'Cycling', icon: 'bike', color: '#A48BFF'},
  strength: {name: 'Strength', plural: 'Strength', icon: 'strength', color: '#FF7DB8'},
  walking: {name: 'Walk', plural: 'Walking', icon: 'walk', color: '#D9C089'},
  hiking: {name: 'Hike', plural: 'Hiking', icon: 'hike', color: '#B9A26E'},
  hiit: {name: 'HIIT', plural: 'HIIT', icon: 'flame', color: '#F0A0FF'},
  swimming: {name: 'Swim', plural: 'Swimming', icon: 'swim', color: '#6FA8FF'},
  yoga: {name: 'Yoga', plural: 'Yoga', icon: 'other', color: '#B5DE7C'},
  other: {name: 'Other', plural: 'Other', icon: 'other', color: '#9AA3B5'},
};
export const sportName = (s: Sport) => SPORTS[s].name;
export const GPS: Sport[] = ['running', 'cycling', 'walking', 'hiking'];
export const PLAN_SPORTS: Sport[] = ['strength', 'running', 'cycling', 'walking', 'swimming', 'yoga', 'other'];
export const EXCLUDABLE: Sport[] = ['running', 'cycling', 'strength', 'hiit', 'swimming'];
