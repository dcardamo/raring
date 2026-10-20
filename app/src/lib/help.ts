// Offline help pages. Each area ships its pages with the feature.
import type {helpPages} from './navigation';

export type HelpPage = {title:string,icon:'lift'|'check'|'scale'|'clock',intro:string,steps:string[]};
export const GITHUB_QUESTIONS='github.com/dcardamo/raring/issues';

export const help:Record<typeof helpPages[number],HelpPage>={
 'strength-start':{title:'Lift with your watch',icon:'lift',intro:'Raring reads the sets your watch records. It never changes them.',steps:[
  'Start a Strength activity on your watch.',
  'Press lap at the end of each set. The rest runs until you press lap again.',
  'Your watch counts the reps and guesses the exercise. Add the weight on the watch, or later in Garmin Connect.',
  'After Garmin Connect syncs, Raring shows your sets on the workout and in Strength.',
  'Without lap presses the watch records the whole session as one set, with no reps or exercise.',
 ]},
 'strength-sets':{title:'How sets are counted',icon:'check',intro:'Rests are not sets. Raring counts each set once for every main muscle it works.',steps:[
  'Muscles come from Garmin\'s exercise catalog. Sets of an exercise Garmin couldn\'t name count for no muscle.',
  'Raring flags sets that look miscounted: 0 or missing reps, reps at least 1.8 times the middle of the exercise\'s other sets, and exercises Garmin couldn\'t name.',
  'A flag is only a hint. Raring shows Garmin\'s numbers as recorded and never fixes them for you.',
  'Fix a set in Garmin Connect. Raring automatically rechecks sets only during the first 48 hours after the workout starts. After that, tap Refresh from Garmin on the workout: it reads that workout\'s sets once and shows when they were last checked. Opening a workout never checks Garmin.',
  'A record is a set heavier than any earlier set of that exercise with as many reps or more. Flagged sets never count.',
 ]},
 'strength-muscles':{title:'The body map',icon:'scale',intro:'Raring\'s own front and back figures, shaded by sets.',steps:[
  'One blue, faint to strong: more sets, stronger blue. It says how much, not how well.',
  'Gray means no sets for that muscle.',
  'Back includes lats, traps and lower back; arms include biceps, triceps and forearms; core is abs and obliques; hips are abductors and adductors.',
  'The figure follows your Garmin profile.',
 ]},
 'strength-watches':{title:'Does my watch work?',icon:'clock',intro:'Raring reads sets from your Garmin account, not from the watch.',steps:[
  'Forerunner 970: partly tested. Sets, rests, reps and miscount flags read back. A session with named exercises and weights recorded on the watch hasn\'t been checked yet.',
  'Other watches with a Strength activity that records sets may work the same way. Nobody has confirmed them yet.',
  'Activities without a watch, or from other apps, may have no sets.',
  `Tried another watch? Tell us at ${GITHUB_QUESTIONS}.`,
 ]},
};
export const helpSections:{title:string,pages:(typeof helpPages[number])[]}[]=[
 {title:'Strength',pages:['strength-start','strength-sets','strength-muscles','strength-watches']},
];
