// Independently synthetic data for the experience prototype. No value here comes
// from a real Garmin account; names, readings and restrictions are illustrative.
(function (root) {
  const DAYS = ['2026-11-16', '2026-11-17', '2026-11-18', '2026-11-19', '2026-11-20', '2026-11-21', '2026-11-22'];

  // Nights are filed under the morning they end, as Garmin does.
  const nights = {
    '2026-11-10': {asleep: 452, need: 470, score: 79, bed: '22:58', wake: '06:46'},
    '2026-11-11': {asleep: 431, need: 465, score: 74, bed: '23:20', wake: '06:52'},
    '2026-11-12': {asleep: 470, need: 470, score: 82, bed: '22:49', wake: '06:44'},
    '2026-11-13': null,
    '2026-11-14': {asleep: 495, need: 475, score: 85, bed: '23:05', wake: '07:40'},
    '2026-11-15': {asleep: 468, need: 470, score: 80, bed: '23:12', wake: '07:30'},
    '2026-11-16': {asleep: 446, need: 470, score: 77, bed: '22:56', wake: '06:45'},
    '2026-11-17': {asleep: 425, need: 480, score: 72, bed: '23:18', wake: '06:48'},
    '2026-11-18': {asleep: 310, need: 490, score: 48, bed: '00:52', wake: '06:40', note: 'Woke often between 02:00 and 04:00'},
    '2026-11-19': {asleep: 402, need: 495, score: 63, bed: '23:31', wake: '06:58'},
    '2026-11-20': {asleep: 488, need: 485, score: 84, bed: '22:40', wake: '07:15'},
    '2026-11-21': {asleep: 470, need: 475, score: 81, bed: '22:55', wake: '07:35'},
    '2026-11-22': {asleep: 455, need: 470, score: 78, bed: '23:02', wake: '07:20'},
  };

  const readiness = {
    '2026-11-16': {value: 72, level: 'Moderate', limit: 'Sleep history'},
    '2026-11-17': {value: 66, level: 'Moderate', limit: 'Recovery time'},
    '2026-11-18': {value: 38, level: 'Low', limit: 'Sleep score'},
    '2026-11-19': {value: 52, level: 'Moderate', limit: 'Sleep history'},
    '2026-11-20': {value: 84, level: 'High', limit: 'Acute load'},
    '2026-11-21': {value: 81, level: 'High', limit: 'Acute load'},
    '2026-11-22': {value: 77, level: 'High', limit: 'Sleep history'},
  };

  const bodyBattery = {'2026-11-16': 71, '2026-11-17': 64, '2026-11-18': 32, '2026-11-19': 48, '2026-11-20': 79, '2026-11-21': 76, '2026-11-22': 70};

  const activities = [
    {id: 'a-1102', date: '2026-11-09', start: '07:02', sport: 'strength', name: 'Strength', min: 44, im: 22, load: 61, benefit: 'Base', aerobic: 2.1, sets: 'full'},
    {id: 'a-1104', date: '2026-11-10', start: '18:05', sport: 'running', name: 'Run', min: 32, km: 5.6, im: 38, load: 74, benefit: 'Tempo', aerobic: 3.0},
    {id: 'a-1107', date: '2026-11-12', start: '07:10', sport: 'strength', name: 'Strength', min: 28, im: 8, load: 30, benefit: 'Recovery', aerobic: 1.3, timerOnly: true},
    {id: 'a-1110', date: '2026-11-14', start: '09:30', sport: 'hiit', name: 'HIIT', min: 30, im: 46, load: 88, benefit: 'Anaerobic', aerobic: 2.6},
    {id: 'a-1111', date: '2026-11-15', start: '10:00', sport: 'cycling', name: 'Ride', min: 70, km: 31.2, im: 64, load: 112, benefit: 'Base', aerobic: 2.9},
    {id: 'a-1112', date: '2026-11-15', start: '16:20', sport: 'walking', name: 'Walk', min: 36, km: 3.1, im: 0, load: 9, benefit: 'Recovery', aerobic: 0.8, lateUpload: '2026-11-16'},
    {id: 'a-1201', date: '2026-11-16', start: '07:05', sport: 'strength', name: 'Strength', min: 42, im: 18, load: 58, benefit: 'Base', aerobic: 2.0, sets: 'full', record: 'Dumbbell bench press 22 kg × 8'},
    {id: 'a-1202', date: '2026-11-16', start: '17:40', sport: 'cycling', name: 'Ride', min: 25, km: 9.4, im: 20, load: 31, benefit: 'Base', aerobic: 1.8},
    {id: 'a-1203', date: '2026-11-17', start: '18:10', sport: 'running', name: 'Run', min: 35, km: 6.1, im: 40, load: 79, benefit: 'Tempo', aerobic: 3.1},
    {id: 'a-1204', date: '2026-11-18', start: '12:30', sport: 'walking', name: 'Walk', min: 24, km: 2.2, im: 0, load: 6, benefit: 'Recovery', aerobic: 0.6},
    {id: 'a-1205', date: '2026-11-22', start: '10:15', sport: 'walking', name: 'Walk', min: 40, km: 3.6, im: 10, load: 14, benefit: 'Recovery', aerobic: 1.0},
  ];
  // Only used by the "target completed early" case; it shares the normal evidence path.
  const extraActivity = {id: 'a-1206', date: '2026-11-18', start: '07:00', sport: 'strength', name: 'Strength', min: 38, im: 16, load: 52, benefit: 'Base', aerobic: 1.9, sets: 'full'};

  const sets = {
    'a-1201': [
      {exercise: 'Dumbbell bench press', muscles: 'Chest, arms', sets: [[10, 20], [8, 22, 'record'], [8, 22]], last: '20 kg × 10, 10, 8'},
      {exercise: 'One-arm row', muscles: 'Back, arms', sets: [[10, 24], [10, 24], [9, 24]], last: '22 kg × 10, 10, 10'},
      {exercise: 'Not recognised', muscles: 'Garmin could not name this exercise', sets: [[12, null], [12, null]], last: 'No earlier sets'},
      {exercise: 'Plank', muscles: 'Core', sets: [[1, null, 'flag']], last: '2 holds', flagged: 'One set looks off: 1 rep recorded for a timed hold'},
    ],
    'a-1102': [
      {exercise: 'Dumbbell bench press', muscles: 'Chest, arms', sets: [[10, 20], [10, 20], [8, 20]], last: '20 kg × 10, 9'},
      {exercise: 'Goblet squat', muscles: 'Quads, glutes', sets: [[10, 20], [10, 20]], last: '18 kg × 10, 10'},
    ],
    'a-1206': [
      {exercise: 'Goblet squat', muscles: 'Quads, glutes', sets: [[10, 20], [10, 22], [8, 22]], last: '20 kg × 10, 10'},
    ],
  };

  const exercises = [
    {id: 'dumbbell-bench-press', name: 'Dumbbell bench press', best: {1: null, 3: null, 5: '22 kg', 8: '22 kg', 10: '20 kg'}, sessions: ['a-1201', 'a-1102']},
    {id: 'one-arm-row', name: 'One-arm row', best: {1: null, 3: null, 5: null, 8: '24 kg', 10: '24 kg'}, sessions: ['a-1201']},
    {id: 'goblet-squat', name: 'Goblet squat', best: {1: null, 3: null, 5: null, 8: '22 kg', 10: '20 kg'}, sessions: ['a-1102']},
    {id: 'plank', name: 'Plank', best: {}, sessions: ['a-1201']},
  ];

  const schedule = [
    {id: 'g-int', date: '2026-11-18', name: 'Intervals 6 × 3 min', sport: 'running', source: 'garmin'},
    {id: 'g-tempo', date: '2026-11-19', name: 'Tempo run 40 min', sport: 'running', source: 'garmin'},
    {id: 'g-legs', date: '2026-11-21', name: 'Leg day A', sport: 'strength', source: 'garmin', routine: true},
  ];

  const metrics = {
    readiness: {name: 'Training Readiness', domain: 'readiness', unit: '', base: 66, spread: 18, source: 'Garmin'},
    sleepScore: {name: 'Sleep score', domain: 'sleep', unit: '', base: 76, spread: 10, source: 'Garmin'},
    sleepTime: {name: 'Time asleep', domain: 'sleep', unit: 'min', base: 440, spread: 60, source: 'Garmin overnight duration'},
    sleepHr: {name: 'Sleep HR', domain: 'health', unit: 'bpm', base: 52, spread: 4, source: 'Garmin overnight average, not daily resting HR'},
    bodyBattery: {name: 'Body Battery', domain: 'health', unit: '', base: 62, spread: 20, source: 'Garmin daily highest (history); current value is separate'},
    stress: {name: 'Stress', domain: 'health', unit: '', base: 31, spread: 9, source: 'Garmin daily average'},
    heartRate: {name: 'All-day heart rate', domain: 'health', unit: 'bpm', base: 54, spread: 3, source: 'Garmin resting HR (daily)'},
    restingHr: {name: 'Resting heart rate', domain: 'health', unit: 'bpm', base: 54, spread: 3, source: 'Garmin daily resting value, not Sleep HR'},
    hrv: {name: 'HRV', domain: 'health', unit: 'ms', base: 48, spread: 7, source: 'Garmin nightly average'},
    respiration: {name: 'Respiration', domain: 'health', unit: 'brpm', base: 14.2, spread: 0.8, source: 'Garmin overnight'},
    spo2: {name: 'Pulse Ox', domain: 'health', unit: '%', base: 95, spread: 1.5, source: 'Garmin overnight'},
    skinTemp: {name: 'Skin temperature change', domain: 'health', unit: '°C', base: 0, spread: 0.4, source: 'Garmin overnight change'},
    vo2max: {name: 'VO₂ max', domain: 'training', unit: '', base: 47, spread: 1, source: 'Garmin estimate', sparse: true},
    endurance: {name: 'Endurance score', domain: 'training', unit: '', base: 5870, spread: 120, source: 'Garmin'},
    hill: {name: 'Hill score', domain: 'training', unit: '', base: 61, spread: 3, source: 'Garmin', sparse: true},
    runningTolerance: {name: 'Running tolerance', domain: 'training', unit: 'km', base: 31, spread: 5, source: 'Garmin weekly'},
    acuteLoad: {name: 'Acute load', domain: 'training', unit: '', base: 410, spread: 80, source: 'Garmin 7-day weighted load'},
    fitnessAge: {name: 'Fitness Age', domain: 'health', unit: 'years', base: 41, spread: 0.3, source: 'Garmin estimate', sparse: true},
    weight: {name: 'Weight', domain: 'health', unit: 'kg', base: 74.2, spread: 0.6, source: 'Garmin weigh-ins', sparse: true},
  };

  const vitals = [
    {id: 'sleepHr', label: 'Sleep HR', value: '52', unit: 'bpm', state: 'Within range'},
    {id: 'hrv', label: 'HRV', value: '46', unit: 'ms', state: 'Within range'},
    {id: 'respiration', label: 'Respiration', value: '14.1', unit: 'brpm', state: 'Within range'},
    {id: 'spo2', label: 'Pulse Ox', value: '95', unit: '%', state: 'Within range'},
    {id: 'skinTemp', label: 'Skin temp', value: '+0.1', unit: '°C', state: 'Baseline building'},
  ];

  const naps = {
    same: [{start: ['2026-11-18', '13:10'], end: ['2026-11-18', '13:36'], min: 26}],
    cross: [{start: ['2026-11-17', '23:40'], end: ['2026-11-18', '00:20'], min: 40, note: 'A nap filed by Garmin across midnight'},
      {start: ['2026-11-18', '15:05'], end: null, min: 22}],
  };

  const journal = [
    {date: '2026-11-18', state: 'available', entries: [['Caffeine', '2 cups'], ['Alcohol', 'No'], ['Late meal', 'Status unknown']]},
    {date: '2026-11-17', state: 'empty', entries: []},
    {date: '2026-11-16', state: 'available', entries: [['Caffeine', '1 cup'], ['Alcohol', 'Yes']]},
    {date: '2026-11-15', state: 'unavailable', entries: []},
  ];

  const gear = [
    {name: 'Road shoes', used: '512 km', limit: '700 km', state: 'Garmin threshold'},
    {name: 'Commuter bike', used: '1 840 km', limit: null, state: 'No distance threshold'},
    {name: 'Old trail shoes', used: '804 km', limit: '800 km', state: 'Retired in Garmin'},
  ];

  // Accepted commitment versions; a version applies from its week onward.
  const commitmentVersions = [
    {from: '2026-W41', strength: 2, im: 150, note: 'First saved plan'},
    {from: '2026-W45', strength: 3, im: 150, note: 'Raised strength'},
    {from: '2026-W47', strength: 2, im: 150, note: 'Lowered strength before the week began'},
  ];
  const weeks = [
    {id: '2026-W40', range: '28 Sep–4 Oct', state: 'none', label: 'No plan yet', detail: '3 recorded activities'},
    {id: '2026-W41', range: '5–11 Oct', state: 'reached', label: 'Reached', detail: 'Strength 2 of 2 · 168 of 150 min', version: '2 strength · 150 min'},
    {id: '2026-W42', range: '12–18 Oct', state: 'partial', label: '1 of 2 reached', detail: 'Strength 1 of 2 · 172 of 150 min', version: '2 strength · 150 min'},
    {id: '2026-W43', range: '19–25 Oct', state: 'paused', label: 'Paused', detail: 'Whole week paused · 2 activities recorded', version: '2 strength · 150 min'},
    {id: '2026-W44', range: '26 Oct–1 Nov', state: 'unknown', label: 'Awaiting evidence', detail: 'Activity list incomplete · 2 known workouts', version: '2 strength · 150 min'},
    {id: '2026-W45', range: '2–8 Nov', state: 'reached', label: 'Reached', detail: 'Strength 3 of 3 · 151 of 150 min', version: '3 strength · 150 min'},
    {id: '2026-W46', range: '9–15 Nov', state: 'partial', label: '1 of 2 reached', detail: 'Strength 2 of 3 · 178 of 150 min', version: '3 strength · 150 min'},
  ];

  const milestones = [
    {date: '2026-11-16', text: 'Dumbbell bench press 22 kg × 8', kind: 'Best in available history (since Sep 2025)', route: 'workout/a-1201'},
    {date: '2026-11-09', text: 'Goblet squat 20 kg × 10', kind: 'Best in available history · corrected once: an earlier 24 kg set was edited in Garmin Connect to 14 kg, so it is no longer a best', route: 'workout/a-1102', corrected: true},
    {date: '2026-11-03', text: 'VO₂ max 47 recorded', kind: 'Garmin estimate · first at this value in available history', route: 'metric/vo2max'},
    {date: '2026-10-11', text: 'First plan week with every commitment met', kind: 'Follow-through · 2 of 2 strength, 168 of 150 min', route: 'review/2026-W41'},
  ];

  const helpPages = {
    'strength-start': {title: 'Lifting with the watch', body: 'Start a strength activity on the watch and press lap between sets so Garmin records each set. A session timed without lap presses still counts as a session; it has no sets to show.'},
    'strength-sets': {title: 'How sets are counted', body: 'Garmin guesses each exercise and counts reps from wrist movement. Raring shows what Garmin recorded and flags sets that look off; it never changes them.'},
    'strength-muscles': {title: 'The body map', body: 'The map shades muscles by recorded sets. It is not muscle freshness, injury status or safety.'},
    'strength-watches': {title: 'Which watches work', body: 'Set recording depends on the watch. Raring shows what was uploaded to Garmin Connect.'},
    'sync': {title: 'How syncing works', body: 'Your watch uploads to Garmin Connect; Raring then reads Garmin Connect. If the watch has not uploaded, open Garmin Connect on the phone to sync it. Raring never shows its own fetch time as the watch upload time.'},
    'adapt': {title: 'Adapt today and circumstances', body: 'Rest today covers today and the following night. Pauses and limits last until you resume them; a review date only reminds you. Raring does not diagnose, clear exercise or suggest substitutes.'},
    'acute-load': {title: 'What acute load is', body: 'Garmin weights your recent exercise loads into one number and compares it with a range based on your own history. Strength can contribute through physiology, but acute load does not measure muscle recovery, tissue tolerance or strength progress. It is context, not a budget to fill today.'},
  };

  root.RaringData = {DAYS, nights, readiness, bodyBattery, activities, extraActivity, sets, exercises, schedule, metrics, vitals, naps, journal, gear, commitmentVersions, weeks, milestones, helpPages};
})(typeof window !== 'undefined' ? window : globalThis);
