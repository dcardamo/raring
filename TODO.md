# Raring TODO

The prioritized backlog: work top to bottom, one item at a time, each shipped
on its own. Every item keeps the spec's principles: visual first, opens
instantly from local data, Garmin's numbers, nothing leaves the phone. New
data goes into an existing screen or its detail before it earns a new screen.

When an item ships, update `docs/spec.md` in the same change, then move the
item to **Done** with its commit. Endpoints are on `connectapi.garmin.com`
unless noted. "Verified" means a read against a real account returned the
data; everything else comes from
[python-garminconnect](https://github.com/cyberjunky/python-garminconnect)
0.3.19 and is unverified.

## Now

### 1. Full history backfill

Garmin keeps the whole account history and serves old dates, so a new phone
or a reinstall downloads everything again. No Raring cloud is needed.

- Verified: activities back to 2011 and daily summaries from 2025 on a real
  account. Sleep and HRV went back only as far as the watch that records
  them, which suggests Garmin has no retention cutoff (unverified for older
  sleep data).
- Use range queries rather than one request per day. Verified:
  `POST /graphql-gateway/graphql` with
  `query{trainingReadinessRangeScalar(startDate:…, endDate:…)}` returned 5½
  months of readiness in one call. Candidates:
  `/usersummary-service/stats/{stress,bodybattery,heartRate,im,respiration,steps,floors}/daily/{start}/{end}`,
  `/hrv-service/hrv/daily/{start}/{end}`,
  `/sleep-service/stats/sleep/daily/{start}/{end}` (returned empty for a
  period without sleep data, so a period with data still needs checking),
  GraphQL `heartRateVariabilityScalar`, `sleepSummariesScalar` (returned
  null; check its arguments), and `enduranceScoreScalar`/`hillScoreScalar`
  with `aggregation`. Activities page through
  `/activitylist-service/activities/search/activities`.
- Old days store daily values only. Per-day detail (sleep stages, stress
  readings, activity charts and routes) downloads when a day or workout is
  opened, and is then kept.
- Runs in the background, newest first, resumable, inside the existing
  request budget, backing off on 429. Find where the account's data starts
  (first activity, first wellness day) and stop there. Show progress in
  Settings ("History back to Mar 2019").
- Spec: replace "first sync requests up to 90 days … not backfilled".
- Check: sync tests on fixtures of range responses; a real-account run
  timed, with the request count and any 429s recorded.

### 2. Watch sync freshness

Raring is only as fresh as Garmin Connect's last upload from the watch.

- Verified: `/device-service/deviceregistration/devices` lists devices and
  `/device-service/deviceservice/user-device/{deviceId}` returns
  `lastUploadTimestamp`.
- Show "Watch synced 2 h ago" quietly in the Today header. When it's stale
  (for example older than 6 h while awake, or no sleep upload by mid-morning),
  show a gentle card: "Open Garmin Connect to sync your watch", which
  deep-links to Connect when installed.
- Use the device's capabilities to explain missing metrics ("Your watch
  doesn't record skin temperature") instead of empty tiles.

### 3. Garmin's Health Status

- Verified: `POST /graphql-gateway/graphql` with
  `query { healthStatusSummary(calendarDate:"YYYY-MM-DD") { calendarDate outliersCount metrics { value baselineUpperLimit baselineLowerLimit status percentage feedbackKey type } } }`
  returns HRV, HR, SPO2, SKIN_TEMP_C/F and RESPIRATION, each with Garmin's
  baseline bounds and status (`IN_RANGE`, …).
- Use Garmin's baseline and status for the Health vitals and the "vitals
  off" rule. Keep Raring's 80% range only where Garmin returns none, and
  label which is which.
- The web app's other GraphQL queries are worth checking for range sync
  (item 1).
- Spec: update the Rules section's normal range and the Health vitals copy.

### 4. Explain the numbers

Users' most common complaint after clutter: scores without a why, and
scores that seem to disagree.

- On Readiness, Body Battery and Training Status details, a "Why" sheet
  that names the inputs behind each, from Garmin's own factors and feedback:
  for example, "Body Battery charged well overnight, but readiness is held
  back by a high acute load." No new scores.
- When two scores point different ways, a one-line reconciliation on Today,
  behind a tap.
- Spec: list any new wording rules under Rules.

## Next

### 5. Persistent status notification (opt-in)

A silent, ongoing notification that always shows the latest values, like
the persistent notification a weather app keeps up to date.

- Content: Body Battery, stress, latest heart rate, and today's focus (Rest,
  Easy, Steady, Push). Collapsed: one line of icons and numbers. Expanded:
  small bars, the focus with its reason, and "as of" time.
- The focus can change through the day: Garmin recalculates readiness after
  an activity syncs, so a post-workout sync updates it.
- Updated by the existing WorkManager job; no foreground service. Freshness
  is bounded by the watch's upload frequency and WorkManager's 15-minute
  minimum, so always show "as of". Measure battery and request cost before
  defaulting to 15 minutes; consider faster polling only while the screen is
  on.
- Low-importance channel, no sound, off by default, toggled in Settings and
  offered once after the first sync.

### 6. Home-screen widgets

- Small: readiness gauge and focus. Medium: readiness, sleep, Body Battery,
  stress. Same data and refresh as item 5; tapping opens the matching
  screen.
- Rendered natively (Glance or RemoteViews) from a small snapshot the Rust
  core writes after each sync, so the widget never starts the web view.

### 7. Long-term trends and comparisons

Depends on item 1. The analysis Garmin's subscription charges for, free.

- Every trend chart gains ranges up to 1 year and All, with weekly or
  monthly averaging on long ranges.
- Overlay any two metrics on one chart (for example sleep time against
  readiness, or load against HRV), and compare two periods (this training
  block against the last).
- Display only: averages and overlays, no new scores. Spec: add the
  averaging to Rules.

### 8. Export

- Settings → Export: a zip of JSON (raw Garmin responses) plus CSVs of the
  daily tables, through the Android share sheet.
- Optional FIT originals for activities:
  `/download-service/files/activity/{id}`.

## Later: more Garmin data

Each is a tile or detail in an existing screen first.

9. **Intensity minutes and goals**: `/wellness-service/wellness/daily/im/{d}`,
   `/usersummary-service/stats/im/weekly/{start}/{end}`,
   `/goal-service/goal/user/effective/{d}`. Weekly ring on Health and in the
   check-in; feeds the weekly focus and the Fitness Age lever.
10. **Weight and body composition**: `/weight-service/weight/dateRange`,
    `/weight-service/weight/latest`. Trend tile and the BMI or body-fat
    Fitness Age driver.
11. **All-day heart rate**:
    `/wellness-service/wellness/dailyHeartRate/{displayName}?date=`. Day chart
    with sleep and activities marked, like the stress detail.
12. **Planned training**: Garmin Coach plans
    (`/trainingplan-service/trainingplan/active-plans`, `/tasks/{id}`),
    scheduled workouts
    (`/workout-service/schedule/workoutScheduleByPeriod`) and races
    (`/calendar-service/events/upcoming`). Today shows the planned session
    beside readiness; a race countdown on Training. Read-only here.
13. **Gear**: `/gear-service/gear/v2/list`. Shoe and bike distance against
    the retirement threshold, with an optional notification.
14. **Thresholds and acclimation**: lactate threshold, cycling FTP and
    heat/altitude acclimation
    (`/biometric-service/biometric/latestLactateThreshold`,
    `/latestFunctionalThresholdPower/CYCLING`,
    `/metrics-service/metrics/heataltitudeacclimation/latest/{d}`). Training
    detail tiles, shown only when the watch records them.
15. **Naps and journal**: `/sleep-service/sleep/naps/{d}`; the lifestyle
    log (`/lifestylelogging-service/dailyLog/{d}`) as the planned journal.
16. **Women's health (opt-in)**: cycle and pregnancy views
    (`/periodichealth-service/menstrualcycle/…`) as context on readiness and
    vitals. Sensitive: off by default, excluded from notifications unless
    chosen.
17. **Strength**: read-only sets, reps and weights per strength activity
    (`/activity-service/activity/{id}/exerciseSets`), then the weekly plan.
    Design notes exist from earlier research.

## Later: changing Garmin settings

Settings that people touch day to day, written back to Garmin so the watch
picks them up on its next sync. Raring isn't a full settings app; watch
faces, screens, sensors and audio prompts stay in Connect.

18. **Write spike first.** On the maintainer's watch, using the CLI: change an
    alarm, the sleep schedule, and a daily goal; sync the watch through
    Connect; confirm the watch changed; read the value back. Record which
    writes reach the watch and how long it takes. Nothing below ships until
    its write passed.
    - Device settings (verified readable, write unverified):
      `GET/PUT /device-service/deviceservice/device-info/settings/{deviceId}`
      holds `alarms` (each with a `changeState`), `dndEnabled`, `sleep`
      and `activityTracking`. PUT a partial object built from a fresh GET.
    - User settings (verified readable):
      `/userprofile-service/userprofile/user-settings` holds `userSleep`
      (sleep and wake time, seconds after midnight) and `userData` (units,
      training days). PUT the full object with only the changed keys.
19. **Write rules** (do these before any write ships): every write starts
    from a fresh read, changes only the fields the user touched, reads the
    value back and shows success or failure; nothing writes in the
    background; deletes ask first; a local log lists recent changes. Spec:
    a Garmin writes section.
20. **Day-to-day settings**, in order:
    - Alarms: list, toggle, add, edit time and days.
    - Sleep schedule: bed and wake times, linked from tonight's sleep plan.
    - Daily goals: steps (`set_steps_goal`), intensity minutes, floors,
      hydration (`PUT /goal-service/goal/user/effective/{type}/{d}`).
    - Do not disturb on or off, if the spike shows it reaches the watch.
21. **Quick logging**: weight (`/weight-service/user-weight`), hydration
    (`/usersummary-service/usersummary/hydration/log`), blood pressure
    (`/bloodpressure-service/bloodpressure`; it can't be edited, only deleted
    and added again).
22. **Activity edits**: rename, change type, notes
    (`PUT /activity-service/activity/{id}`); then strength set correction.

## Not planned

- AI-written insights: they often restate the numbers and would need a
  service that sees the data.
- Social feeds, badges and challenges.
- Nutrition logging: Garmin limits it to its subscription.

## Done

Nothing yet.
