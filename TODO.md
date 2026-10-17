# Raring TODO

The prioritized backlog: work top to bottom, one item at a time, each shipped
on its own. Every item keeps the spec's principles: visual first, opens
instantly from local data, Garmin's numbers, nothing leaves the phone. New
data goes into an existing screen or its detail before it earns a new screen.

When an item ships, update `docs/spec.md` in the same change, then move the
item to **Done** with its commit. Endpoints are on `connectapi.garmin.com`
unless noted. "Verified" means a read against a real account returned the
data. Unverified endpoint candidates come from
[python-garminconnect](https://github.com/cyberjunky/python-garminconnect)
0.3.19 unless another source is linked; public client code is not proof of
availability on every account or watch.

## Now

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

### 3. Sleep rhythm and bedtime coaching

Extend Sleep and the shared Tonight's sleep card, not a new screen. The
14-night consistency chart, Garmin sleep need and recommended bedtime window
(with a labeled wake-history fallback) already exist.

- Keep three ideas separate: Garmin's **Sleep Alignment** (sleep relative to
  circadian rhythm), **Sleep Consistency** (including Garmin's seven-day
  average bedtime), and **Sleep Coach** (how much sleep is needed). A regular
  schedule is not necessarily an aligned one; tonight's recommendation is
  not the same as the longer-term optimal sleep window.
  [Garmin's feature descriptions][garmin-sleep-rhythm].
- Data gate: inspect the existing
  `/wellness-service/wellness/dailySleepData/{displayName}?date=…&nonSleepBufferMinutes=60`
  response first. Raring already reads `dailySleepDTO.nextSleepNeed`,
  including `recommendedBedtimeStartMins/EndMins`.
  [Public client code][garmin-alignment-fields] also reads
  `dailySleepDTO.sleepAlignment.optimalSleepWindowStartMins/EndMins`;
  these alignment fields are **unverified on our account**. Check their
  units, date/timezone meaning, Garmin feedback/status and calibration
  states against Connect and the watch; record supported-device limitations
  and scrubbed fixtures before promising alignment coaching.
- Show actual sleep against Garmin's optimal window on a simple overnight
  timeline, with recent bed/wake history below. Keep Garmin's target distinct
  from Raring's existing shaded *usual* windows. Use Garmin's consistency
  summary if supplied; otherwise retain the descriptive chart, not a new
  rhythm score or inferred chronotype.
- Turn the plan into one useful next step: a wind-down time, the bedtime
  window, sleep need and a short reason, with evidence behind a tap. Prefer
  Garmin's feedback; explain whether a time is Garmin's recommendation or
  Raring's fallback. Coach toward steadier bed/wake timing, not simply an
  earlier bedtime. Add a timing review and one focus to the existing weekly
  check-in; no new adherence score, failure badges, streak pressure,
  sleep-stage promises or claims that timing caused a score change.
- When alignment is unavailable, keep the existing plan/history usable.
  Distinguish unsupported, still calibrating, missing and stale data where
  Garmin supplies enough information; otherwise say unavailable. Do not
  label ordinary averages as a circadian recommendation.
- Extend the existing off-by-default bedtime reminder, not a second alert:
  configurable wind-down lead, initially 30 minutes before the window
  starts (or the fallback asleep-by time). Schedule from the cached plan,
  suppress stale plans, deduplicate per sleep episode, and reschedule/cancel
  when the plan or timezone changes. Coaching does not write Garmin's sleep
  schedule.
- Spec: update Sleep, Tonight's sleep, weekly focus and Bedtime notifications;
  document advice precedence and source labels. Check synthetic fixtures
  for alignment present/absent/calibrating, conflicting target and tonight
  windows, irregular schedules, stale plans, midnight and DST; compare a
  supported real-account result with Garmin before release.

### 4. Today that fits the time of day

Change emphasis, not navigation. Put one timely action first while keeping
Today · Sleep · Training · Health and metric access predictable.

- Keep stable metric shortcuts alongside a changing highlight. Bedtime
  guidance stays collapsed earlier in the day, rises about an hour before
  its window, and opens timing history. Keep longer-term rhythm separate
  from tonight's recommendation. Sources are in the maintainer's private
  research notes; use no other app's branding, assets, copy or scoring.
- **After waking:** last night's sleep and readiness, the main limiting
  factor, then today's effort suggestion. Missing sleep is neutral: show
  freshness and the known missing-data reason, not a bad-night verdict.
- **During the day:** current Body Battery/stress and activity or recovery
  context, using the latest synced readiness. Do not imply live watch data
  or invent a planned workout before planned-training support exists.
- **Approaching sleep:** promote the shared bedtime plan from item 3,
  emphasizing wind-down and bedtime timing rather than yesterday's sleep
  score. Prototype promotion two hours before the window starts, followed
  by a calm wind-down state. Once the window has passed, avoid a red
  countdown or pressure to exercise; do not assume the person is asleep.
- Anchor phases to the person's main sleep/wake pattern and upcoming plan,
  not universal morning/evening clock cutoffs. Allow automatic emphasis to
  be disabled for shift work or irregular schedules; without a credible
  schedule retain the stable overview. Re-evaluate locally on open/resume,
  time boundaries and sync, including offline; crossing midnight must not
  discard a still-relevant sleep plan.
- Adapt one leading area, not a constantly reshuffling dashboard. Keep
  compact score access, freshness and vitals notices visible; Sleep still
  holds the full history. Never switch tabs or move content under someone's
  finger. Use the same plan and source labels across Today, Sleep and
  reminders; no separate competing advice engine.
- Spec: update Today's hierarchy and define phase boundaries, fallback and
  advice priority under Rules. Check clock-controlled phone screenshots
  for wake/day/wind-down/late-night using the same data, plus missing/stale
  data, post-workout updates, shift work, travel, DST and overnight dates.
  Confirm fast offline rendering and unchanged access to all metrics.

[garmin-sleep-rhythm]: https://www.garmin.com/en-US/blog/fitness/the-difference-between-garmin-venu-3-and-venu-4/
[garmin-alignment-fields]: https://github.com/cyberjunky/ha-garmin/blob/main/src/ha_garmin/client.py

### 6. Explain the numbers

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

### 7. Weekly plan, intensity minutes and goals

Add a weekly plan to Today and the check-in, backed by Garmin's intensity
minutes and goals: `/wellness-service/wellness/daily/im/{d}`,
`/usersummary-service/stats/im/weekly/{start}/{end}` and
`/goal-service/goal/user/effective/{d}`. Show the intensity-minutes ring on
Health and in the check-in; it feeds the weekly focus and Fitness Age lever.

- Targets: strength sessions, vigorous/intensity minutes (Garmin's goal by
  default), Zone 2 minutes and asleep-on-time nights. Show partial credit,
  planned rest days, the weekly check-in and week-over-week history. See
  `design/mockup/` screens `plan`, `plan-setup` and `checkin`.
- Decisions in `design/mockup/NOTES.md`: start the week on Garmin's setting;
  follow Garmin's units. Pace uses elapsed training days (nights for sleep);
  asleep-on-time means within 15 minutes of the suggested time, excluding
  nights without a plan. History averages target completion capped at 100%
  per target. Add Raring's pace, timing and completion math to the spec's
  Rules section when this ships.

### 8. Read-only strength view

- First read real strength activities from the maintainer's Forerunner 970
  and check the quality of `/activity-service/activity/{id}/exerciseSets`.
  Do not call the endpoint verified until a real-account read succeeds.
- Show recorded sets, reps and weights per activity, last time, PRs, weekly
  sets per muscle, a body map and miscount flags. See `design/mockup/` screens
  `strength-workout` and `strength-history`.
- Decisions in `design/mockup/NOTES.md`: count a set once for each primary
  muscle in Garmin's catalog (use a small mapping if absent); flag zero
  reps, reps at least 1.8 times the exercise's other-set median, or an
  unnamed exercise, without auto-correction. Body map art gets an
  illustration pass when strength ships; ship its offline help pages then.
  Publish compatibility only for tested models when features ship, after
  creating a `compatibility` issue label. Add Raring's set-count and flag
  math to the spec's Rules section when this ships.

### 9. Persistent status notification (opt-in)

A silent, ongoing notification that always shows the latest values, like
the persistent notification a weather app keeps up to date.

- Content: Body Battery, stress, latest heart rate, and the day-aware focus
  from item 4 (effort during the day, bedtime timing near sleep). Collapsed:
  one line of icons and numbers. Expanded: small bars, the focus with its
  reason, and "as of" time.
- The focus can change through the day: Garmin recalculates readiness after
  an activity syncs, so a post-workout sync updates it.
- Updated by the existing WorkManager job; no foreground service. Freshness
  is bounded by the watch's upload frequency and WorkManager's 15-minute
  minimum, so always show "as of". Measure battery and request cost before
  defaulting to 15 minutes; consider faster polling only while the screen is
  on.
- Low-importance channel, no sound, off by default, toggled in Settings and
  offered once after the first sync.

### 10. Home-screen widgets

- Small: readiness gauge and focus. Medium: readiness, sleep, Body Battery,
  stress. Same data and refresh as item 9; tapping opens the matching
  screen.
- Rendered natively (Glance or RemoteViews) from a small snapshot the Rust
  core writes after each sync, so the widget never starts the web view.

### 11. Long-term trends and comparisons

Depends on item 1. The analysis Garmin's subscription charges for, free.

- Every trend chart gains ranges up to 1 year and All, with weekly or
  monthly averaging on long ranges.
- Overlay any two metrics on one chart (for example sleep time against
  readiness, or load against HRV), and compare two periods (this training
  block against the last).
- Display only: averages and overlays, no new scores. Spec: add the
  averaging to Rules.
- Older days live in the history rows listed under Done (item 1), next to
  the per-day rows of the last 90 days; each chart reads both. Check the web
  app's other GraphQL queries for a Health Status range read before adding
  per-day requests to long-range trends.
- Opening an old day or workout should fetch its per-day detail (sleep
  stages, stress readings, charts, route) on demand and keep it. Not built.

### 12. Export

- Settings → Export: a zip of JSON (raw Garmin responses) plus CSVs of the
  daily tables, through the Android share sheet.
- Optional FIT originals for activities:
  `/download-service/files/activity/{id}`.

## Later: more Garmin data

Each is a tile or detail in an existing screen first.

13. **Weight and body composition**: `/weight-service/weight/dateRange`,
    `/weight-service/weight/latest`. Trend tile and the BMI or body-fat
    Fitness Age driver.
14. **All-day heart rate**:
    `/wellness-service/wellness/dailyHeartRate/{displayName}?date=`. Day chart
    with sleep and activities marked, like the stress detail.
15. **Planned training**: Garmin Coach plans
    (`/trainingplan-service/trainingplan/active-plans`, `/tasks/{id}`),
    scheduled workouts
    (`/workout-service/schedule/workoutScheduleByPeriod`) and races
    (`/calendar-service/events/upcoming`). Today shows the planned session
    beside readiness; a race countdown on Training. Read-only here.
16. **Gear**: `/gear-service/gear/v2/list`. Shoe and bike distance against
    the retirement threshold, with an optional notification.
17. **Thresholds and acclimation**: lactate threshold, cycling FTP and
    heat/altitude acclimation
    (`/biometric-service/biometric/latestLactateThreshold`,
    `/latestFunctionalThresholdPower/CYCLING`,
    `/metrics-service/metrics/heataltitudeacclimation/latest/{d}`). Training
    detail tiles, shown only when the watch records them.
18. **Naps and journal**: `/sleep-service/sleep/naps/{d}`; the lifestyle
    log (`/lifestylelogging-service/dailyLog/{d}`) as the planned journal.
19. **Women's health (opt-in)**: cycle and pregnancy views
    (`/periodichealth-service/menstrualcycle/…`) as context on readiness and
    vitals. Sensitive: off by default, excluded from notifications unless
    chosen.

## Later: changing Garmin settings

Settings that people touch day to day, written back to Garmin so the watch
picks them up on its next sync. Raring isn't a full settings app; watch
faces, screens, sensors and audio prompts stay in Connect.

20. **Write spike first.** On the maintainer's watch, using the CLI: change an
    alarm, the sleep schedule, and a daily goal; sync the watch through
    Connect; confirm the watch changed; read the value back. Record which
    writes reach the watch and how long it takes. Nothing below ships until
    its write passed.
    - Results on a Forerunner 970 (2026-10-17): an alarm time edit (full
      settings object PUT, 200) and the floors goal (200) both showed on the
      watch after one ordinary Connect sync. The sleep schedule PUT (204)
      reads back correctly; whether the watch picks it up is unconfirmed.
      The alarm's `changeState` stayed `UNCHANGED` and the write still
      reached the watch. `lastDownloadTimestamp` on
      `/device-service/deviceservice/user-device/{id}` moves when the watch
      syncs, so the app can say "waiting for watch sync" until it does.
      Re-sending the whole user settings object reorders
      `availableTrainingDays` but keeps its values.
    - Device settings (alarms verified writable):
      `GET/PUT /device-service/deviceservice/device-info/settings/{deviceId}`
      holds `alarms` (each with a `changeState`), `dndEnabled`, `sleep`
      and `activityTracking`. PUT a partial object built from a fresh GET.
    - User settings (verified readable):
      `/userprofile-service/userprofile/user-settings` holds `userSleep`
      (sleep and wake time, seconds after midnight) and `userData` (units,
      training days). PUT the full object with only the changed keys.
21. **Write rules** (do these before any write ships): every write starts
    from a fresh read, changes only the fields the user touched, reads the
    value back and shows success or failure; nothing writes in the
    background; deletes ask first; a local log lists recent changes. Spec:
    a Garmin writes section.
22. **Strength workout builder**: after the write rules, start with a
    workout-push spike on the Forerunner 970: create and schedule a strength
    workout, confirm it reaches the watch, do it and read the sets back.
    Check whether morning readiness sync arrives before a session (gates the
    readiness brake) and how Garmin counts repeats against the ~100-step cap.
    Then build workouts with progression: double progression with the smallest
    equipment step, reps first if the next step is over 10% heavier, one step
    down after two sessions below range and two sessions' wait on a new
    exercise. Schedule progressed weights, swapping in held weights after
    morning sync on Rest or Easy days only if the spike supports it. Warn at
    90 steps, offer a split at 100, subject to the spike's repeat count;
    ship one-time scheduling first, not weekly rescheduling. See
    `design/mockup/NOTES.md` and the `builder` mockup; add Raring's
    progression math and readiness brake to the spec's Rules section when
    this ships. Ship builder help and compatibility for tested models then.
23. **Day-to-day settings**, in order:
    - Alarms: list, toggle, add, edit time and days.
    - Sleep schedule: bed and wake times, linked from tonight's sleep plan.
    - Daily goals: steps (`set_steps_goal`), intensity minutes, floors,
      hydration (`PUT /goal-service/goal/user/effective/{type}/{d}`).
    - Do not disturb on or off, if the spike shows it reaches the watch.
24. **Quick logging**: weight (`/weight-service/user-weight`), hydration
    (`/usersummary-service/usersummary/hydration/log`), blood pressure
    (`/bloodpressure-service/bloodpressure`; it can't be edited, only deleted
    and added again).
25. **Activity edits and strength set correction**: rename, change type,
    notes (`PUT /activity-service/activity/{id}`), then correct recorded
    strength sets. Follow the write rules in item 21; for a failed save,
    retain local edits, fetch the sets again before retrying in case they
    changed in Connect, and explain what was rejected (see
    `design/mockup/NOTES.md` and the `fix` mockup).

## Not planned

- AI-written insights: they often restate the numbers and would need a
  service that sees the data.
- Social feeds, badges and challenges.
- Nutrition logging: Garmin limits it to its subscription.

## Done

- **5. Garmin's Health Status** (commit "Use Garmin Health Status for vitals").
  `POST /graphql-gateway/graphql` with `healthStatusSummary(calendarDate)`
  returns six metric entries: HRV, HR, SPO2, SKIN_TEMP_C/F and RESPIRATION,
  with `value`, baseline limits, `status`, `percentage` and `feedbackKey`.
  Seven real read-only dates: a recent day returned all six `IN_RANGE`, an
  early-watch day returned all six `ONBOARDING` with provisional numeric
  limits, and an older day returned a null summary. The GraphQL `HR` value
  matched sleep's `dailySleepDTO.avgHeartRate`, **not** resting heart rate;
  SpO2 matched the overnight value. Raring uses the Celsius entry and Garmin's
  status for comparable readings, including the two-vitals rule; unknown or
  onboarding statuses remain neutral. The existing 30-night range is labeled
  as Raring's fallback where Health Status has no metric. Synthetic fixtures
  cover in-range, outliers, onboarding and missing data. Real outside-range
  statuses and visual agreement with Connect/watch still need a phone check.
  Other web GraphQL range queries remain a possible future optimization.

- **1. Full history backfill** (commit "Download the whole Garmin history
  in the background"). After the per-day sync of the last 90 days, each run
  spends its remaining budget on range requests, resuming from cursors
  (`history:<series>:lo/hi`). Verified on a real account: about 260 requests
  covered activities back to 2011 and daily summaries back to 2014, with no
  429s at the client's 300 ms pacing; a normal sync afterwards made no
  history requests. Checked from the CLI only; the Android background
  job running it is still to check on the emulator and a phone. Range limits and shapes, all verified:
  - `/usersummary-service/v2/dailysummaries?startDate&endDate`, at most a
    year (`P1Y`); a nested summary, not the per-day shape. Stored as
    `daily_summary`.
  - `/metrics-service/metrics/trainingreadiness/{start}/{end}` (366 days) and
    `/metrics-service/metrics/maxmet/daily/{start}/{end}` (2 years worked)
    return exactly the per-day responses; they fill `readiness` and
    `max_metrics` rows without overwriting existing ones.
  - `/hrv-service/hrv/daily/{start}/{end}` (366 days): items equal the
    per-day `hrvSummary`, stored as `hrv_summary`.
  - `/sleep-service/sleep/dailySleepSummariesByDate?startDate&endDate`
    (28 days): items equal the per-day `dailySleepDTO`, stored as
    `sleep_summary`.
  - `/activitylist-service/activities/search/activities` takes
    `startDate`/`endDate` and up to 1000 per page; items stored as `listed`.
  - The `/usersummary-service/stats/*/daily` and
    `/sleep-service/stats/sleep/daily` ranges stop at 28 days, and the
    Body Battery reports reject long ranges.
