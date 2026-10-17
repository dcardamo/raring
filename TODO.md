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

### Time-of-day follow-ups

From item 4, in priority order.

- Live with it on the maintainer's phone for a few days: the morning,
  daytime, evening, wind-down and late-night cards at the real times, the
  switch after resuming from the background, and a night that syncs late.
- Check the first daylight-saving night (2026-11-01 for the maintainer): the
  late-night card should hold until the usual wake on the new wall clock,
  and the one-hour offset change must not switch to the fixed layout.
- Check travel on a real trip: more than an hour of offset change should
  show the fixed layout until a night is recorded in the new zone.
- Without enough recorded nights, Garmin's own sleep schedule (`userSleep`
  in `/userprofile-service/userprofile/user-settings`, verified readable)
  could anchor the phases instead of the fixed layout.
- On desktop widths the masonry columns can place the leading card beside a
  notice rather than first in reading order; check it once desktop ships.
- "Timing history" opens Sleep at its top; scroll it to Consistency.
- Item 9's notification focus can reuse `TodayFocus` rather than another
  phase rule.

### Sleep rhythm follow-ups

From item 3, in priority order.

- Compare a real night with Garmin Connect and the watch (see the item 3
  entry under Done) before release.
- Check the first daylight-saving night (2026-11-01 for the maintainer):
  whether Garmin's window and bedtime minutes stay wall-clock times, and
  that the Android reminder still fires at the wind-down time.
- Check the reminder on a real phone overnight. A ten-minute alarm window can
  be deferred in Doze; if reminders arrive late, offer exact alarms behind
  Android's alarms-and-reminders permission.
- Sleep alignment on other watches and while Garmin calibrates is unverified;
  Raring treats status codes containing CALIBRAT, NOT_ENOUGH, INSUFFICIENT or
  LEARNING as calibrating. Record real codes when seen.
- Desktop has no bedtime reminder: its sync marks notifications as sent
  without showing them. Deliver them (or stop marking them) before calling
  reminders cross-platform.
- Garmin's average bedtime before midnight: confirm whether
  `averageLocalSleepStartTime` comes back negative or wrapped (Raring accepts
  both).

### Trends follow-ups

From item 11, in priority order.

- Check the trend charts on a real phone with the full history: each range
  should draw without a visible wait, and the overlay picker should be easy
  to use one-handed.
- Opening an old day or workout should fetch its per-day detail (sleep
  stages, stress readings, charts, route) on demand and keep it. Not built:
  it needs a way to open a past day on Sleep and Stress and an online fetch
  path for views, larger than item 11.
- Weekly trend buckets run Monday to Sunday. Consider aligning them with
  Garmin's reported intensity-week boundary, which the weekly plan already
  uses; a display preference need not match that data window.
- Fitness Age history is limited to downloaded per-day estimates; endurance,
  hill score and race predictions have no long-history rows. The web GraphQL lists
  `enduranceScoreScalar` and `hillScoreScalar` (start, end, weekly
  aggregation); unverified.

## Next

### 7a. Weekly plan follow-ups

- Check on a watch whose account starts the week on Sunday whether its own
  intensity-minutes widget resets on Monday like Garmin's weekly rows.
- The mockup's "Next up" strength workout card on the plan screen waits for
  planned training (item 15) or the workout builder (item 22).
- Offline help pages "How targets count" and "Rest days and paused weeks"
  when the Help screen ships.
- The shared `dateLabel` reads "Mon, 5 Oct" on Android's WebView (Chrome
  124) but "Mon 5 Oct" in desktop browsers; build it by hand as `plan.ts`
  does for week ranges.

### 8a. Strength follow-ups

- **Explicit detail refresh**: add a budgeted way to refresh older strength
  sets through the existing worker and serialized token access. Automatic
  rechecks stop 48 hours after a workout starts; Refresh and reopening an
  older workout do not retrieve later Connect edits. Coordinate any shared
  on-demand refresh with the integration lane rather than adding another path.
- **Confirm a full Forerunner 970 session.** Read-back is only partly
  verified on the watch: no watch-recorded session with named, weighted sets
  (or one run from a structured workout) has been read yet. Record one with
  a lap press after each set and weights entered, compare Raring's workout
  screen with Connect, then decide whether the compatibility page can move
  from partial to Tested.
- **Strength intro and figure choice** (`intro-1` to `intro-4` in
  `design/mockup/`): the four first-run steps and the female/male figure
  override. The figure follows the Garmin profile until then.
- **Exercise display names**: Raring title-cases Garmin's keys
  ("One Arm Dumbbell Row"). Look for Garmin's own localized names before
  adding a table.
- **Bodyweight exercises**: bests and records need a weight, so bodyweight
  sets (Garmin's weight 0 or null) get none. Consider most reps instead.
- The weekly plan's strength target (item 7) counts Garmin strength
  activities, including ones timed without sets. `StrengthWeek.strengthActivities`
  is distinct from `sessionsWithSets`; missing lap presses must not reduce
  credit toward the weekly activity goal.

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

### Export follow-up: import local settings on a new phone

The ZIP includes every safe settings row, including the weekly plan's targets
and paused weeks (`plan`) and prospective sleep-plan snapshots
(`sleep_plan:YYYY-MM-DD`). Offer a validated import of these settings on a new
phone; no import is provided yet.

### Export follow-up: optional activity FIT originals

`/download-service/files/activity/{id}` is an unverified candidate. Each
activity would cost one request; first count the activities and show that
request count as an explicit choice. Download within the existing request
budget and 429 backoff, resume after interruption, then add the files to a
separate export. The offline JSON/CSV export does not make network requests.

## Later: more Garmin data

Each is a tile or detail in an existing screen first.

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

- **13. Weight and body composition** (commit "Show weight and body composition on Health").
  Health shows the latest weigh-in, a 12-week sparse trend and recorded BMI/body
  fat when supplied. Fitness Age continues to use Garmin's targets; a weight
  reading can fill an absent BMI/body-fat driver without inventing a target or
  age estimate. Garmin's measurement system sets kg or lb. No weight alerts.
  Latest is checked daily; the recent 90-day range is checked weekly, including
  empty ranges; older years use budgeted range-history chunks. Verified
  read-only endpoints (no 429 or auth failure):
  - `GET /weight-service/weight/dateRange?startDate=YYYY-MM-DD&endDate=YYYY-MM-DD`
    returns `{startDate,endDate,dateWeightList,totalAverage}`. Each entry has
    `calendarDate`, `date` and `timestampGMT` (epoch milliseconds), `weight`
    in **grams**, `samplePk`, `sourceType`, optional `weightDelta`, `bmi`,
    `bodyFat`, `bodyWater`, `boneMass`, `muscleMass`, `visceralFat`,
    `metabolicAge`, `physiqueRating`; body-composition fields can be null.
    Multiple entries for a day are possible. An empty range returns an empty
    `dateWeightList` and an object `totalAverage` with null measurements.
  - `GET /weight-service/weight/latest?date=YYYY-MM-DD` returns a single
    weigh-in with `date`/`timestampGMT` in milliseconds, `weight` in grams,
    nullable BMI and body-composition fields, `sourceType`, `version`,
    `weightDelta` and nullable `caloricIntake`. Omitting `date` returned HTTP
    400; latest may predate the requested date. A body-composition reading
    with non-null fields was not available to verify on this account.
  - Add Weight to item 11's long-range chart metric picker using the stored
    daily weigh-ins; retain gaps rather than filling missing weeks.
  - Confirm `/latest` on an account with no weigh-ins and the non-null body
    composition values from a compatible scale; neither was available in the
    read-only account verification.
- **12. Export** (commit "Export Garmin history from local data"). Settings
  builds a ZIP in shared Rust code containing all saved raw Garmin JSON,
  per-day CSV tables, non-credential keys from the local settings table and
  a dated, versioned manifest. Android shares a private-cache FileProvider
  URI (a filename-scoped cleanup is registered before writing, 24 hours from
  export creation, including crashed partials); desktop saves in Downloads. Welcome, Settings
  and site privacy copy explain that exports leave the phone only when shared.
  On a read-only copy of the backfill lane's full-history database: 3,836
  raw responses and 2,571 daily rows exported offline in 12 seconds to a
  9.1 MB ZIP, with 40,192 KiB peak RSS (`time -v`, separate process); no
  Garmin requests or 429s. Integrated emulator testing opened the actual
  Android share sheet and verified the ZIP includes stored plan versions and
  prospectively captured sleep plans. Android publication uses an atomic
  no-replace rename because its sandbox forbids hard links. Desktop Downloads
  and sharing on a physical phone remain unverified. FIT originals remain a
  separate, explicitly budgeted follow-up above.

- **11. Long-term trends and comparisons** (commit "Add long-term trends and
  comparisons"). Every trend chart gains 7D, 30D, 90D, 1Y and All, with
  weekly averages up to two years and monthly beyond, selectable primary
  and overlay metrics, and comparison with the previous period or chosen
  equal-length training blocks. Sleep, acute load and Fitness Age gain history
  cards and Body Battery a daily-high chart. The Rust core averages from local rows
  (`trend` command, `raring trend` in the CLI); no new requests. On the
  maintainer's full-history database (37 MB) a debug CLI read took at most
  31 ms for All (14 metrics checked, 0–31 ms). Emulator-5580 passed
  custom-block and overlay smoke checks with that database; four All-history
  IPC reads including an HRV overlay took 14–38 ms. No new requests are made
  by charts. Read-only checks against the real account, three GraphQL
  requests, no 429s:
  - No Health Status range was found in the web app's published GraphQL
    operation inventory or three live probes (not proof none exists): `healthStatusSummaries`,
    `healthStatusSummaryRange`, `healthStatusRange`, `healthStatusScalar`,
    `healthStatusRangeScalar`, `healthStatusSummariesScalar`,
    `healthStatusSummaryRangeScalar`, `healthStatusHistory`, `healthStatus`,
    `healthStatusTrend` and `healthStatusTrendScalar` are undefined fields,
    and `healthStatusSummary` accepts only `calendarDate` (`startDate`,
    `endDate` and `days` are unknown arguments). Introspection is disabled.
  - Health Status is preferred; older dates fall back to nightly records.
    Sleep HR (`avgHeartRate`) stays distinct from resting HR. The two sources
    can report different respiration averages; do not describe them as
    interchangeable measurements.
  - History coverage is metric- and account-dependent. Skin temperature may
    exist only in detailed sleep rows; Fitness Age has no historical range
    backfill. Summaries marked `includesWellnessData: false` do not establish
    resting HR, Body Battery or stress coverage. Sparse VO2 max readings
    remain sparse rather than being filled into unmeasured days.
  - Acute load history comes from readiness history's `acuteLoad`; the
    corresponding Training Status field is `dailyTrainingLoadAcute`.
    Detailed account comparisons remain in the private research notes.
- **8. Read-only strength view** (commit "Show strength sets, history and the
  body map"). Strength activities show Garmin's sets per exercise with
  last time, records and miscount flags; Training has a this-week card;
  Strength history has phone, unfolded and desktop layouts with the body map,
  bests, sessions, sets per muscle and a 12-week heatmap. Raring's own body
  map figures replace the mockup placeholders. Offline help pages and a help
  sheet ship with it. `site/compatibility.html` covers set read-back only,
  with partial evidence on the Forerunner 970. It is linked from the website's
  footer; reports use GitHub's `compatibility` label. Verified read-only on a
  real account (no 429 or auth failures):
  - `/activity-service/activity/{id}/exerciseSets` returns
    `{activityId, exerciseSets}`; `exerciseSets` is `null` for activities
    without sets (such as ones from non-Garmin sources). Each set has
    `setType` `ACTIVE` or `REST`, `startTime` (UTC, no offset), `duration` in
    seconds, `repetitionCount` (may be null), `weight` in grams (0 or null
    when none), `messageIndex`, `wktStepIndex` (null without a structured
    workout) and about 20 velocity/power fields, null on the Forerunner 970.
    `exercises` ranks Garmin's guesses by `probability`, each
    `{category, name}`; `name` may be null with a known category, and an
    unrecognized set has category `UNKNOWN`.
  - Testing limits: the Forerunner 970 reads covered sessions timed without
    lap presses (one `ACTIVE` set, no reps, `UNKNOWN`), 0-rep unrecognized
    sets, `REST` sets and a category-only guess without a weight. Complete
    named, weighted sets were only seen from a source other than the watch.
    Doubled sets were not seen in real data; they are covered by synthetic
    tests.
  - The activity list rejects `activityType=strength_training` with HTTP 400;
    filter on `activityType.typeKey` locally.
  - Garmin's exercise catalog
    (`connect.garmin.com/web-data/exercises/Exercises.json`, public, no
    token) lists categories and exercises with primary and secondary
    muscles; some categories carry muscles of their own. Raring keeps only
    primary muscles.
  - `/userprofile-service/userprofile/user-settings` gives
    `measurementSystem`, `timeFormat`, `firstDayOfWeek.dayName` and `gender`;
    only those safe preferences are stored, never the full `userData`.
  Scrubbed, synthetic-valued fixtures are `crates/core/tests/fixtures/
  exercise_sets*.json` and `exercise_catalog.json`.
  Review fixes separate timer-only workouts from sessions-with-sets aggregates,
  preserve partial/missing rep totals, and expose complete history lists.
  Strength activity queries share the chart's first-week boundary and filter
  by sport without the general list's cap. Synthetic history links resolve
  ID-matched workout details; browser tests check the rendered date and sets.
  Screenshot investigation on synthetic data: viewport/document/body/main
  widths all match at 390, 720 and 1280 px. The old 535 px phone image came
  from capture code disabling shell clipping, exposing the decorative glow
  to x=535. Plain viewport captures in `app/src/scripts/strength-screens.mjs`
  preserve normal clipping and write measurements to `widths.json`.
- **7. Weekly plan, intensity minutes and goals** (commit "Add the weekly plan
  and intensity minutes"). A weekly plan on Today, its own screen and
  setup (presets, four targets, rest days, pause, last week on these
  targets), an intensity-minutes ring on Health and in the check-in, and a
  plan card in the check-in with week-over-week changes. Targets live in the
  core's settings store, versioned by week. Pace, asleep-on-time, Zone 2,
  completion and history rules are in the spec. Asleep on time uses item 3's
  shared sleep plan (`night`, `bedtimeStartAt`, `bedtimeAt`), captured
  prospectively on live sync/open and never reconstructed from backfill.
  Snapshot settings are immutable and included in the planned export. Intensity
  minutes feed the weekly focus (Garmin's weekly goal first, then Fitness
  Age's vigorous average). The week follows Garmin's intensity window, not
  the start-of-week setting (evidence below). Sync adds one request a day
  (weekly intensity rows); the user-settings and goals endpoints are not
  called. Verification used 37 read-only requests (seven single reads and
  one budget-30 sync), with no 429 or auth failures; the plan's intensity
  total matched Garmin's weekly row. Covered by `crates/core/tests/weekly_plan.rs`
  (pace, rest days, partial credit, capped history, paused weeks, per-week
  targets, Garmin week start, midnight windows, daily sync of the rows) and
  `plan_evidence.rs` (incomplete/verified-zero coverage, timer-only strength,
  authoritative weekly intensity, immutable prospective sleep plans, no
  pre-plan achievement history, honest summary states). The UI regression
  script covers partial/unavailable states at phone, unfolded and desktop
  widths, including known-only completion that must not read as all reached.
  Existing databases need a new complete recent activity-list scan before
  strength/Zone 2 can be judged; missing activity types or zone durations
  remain unavailable. No additional Garmin requests are introduced by these
  evidence fixes. Follow-up verification: 196 Rust tests, clippy, Svelte/TS
  checks, 39 UI unit tests, browser regressions/build and the secret scan
  pass. Partial/unavailable screenshots are retained at 390/700/1100 px in
  `.data/screens/weekly-evidence-*`; emulator-5574 verified unknown activity
  coverage, absent historical sleep evidence, pre-plan gaps and target saves,
  then was shut down. No additional account calls for the corrections.
  - Verified read-only on a real account (2026-10-17):
    - `/usersummary-service/stats/im/weekly/{start}/{end}` returns an array of
      `{calendarDate, moderateValue, vigorousValue, weeklyGoal}` (minutes),
      one row per week, dated by its first day. The rows were dated Mondays
      even with a Sunday start-of-week setting (user settings
      `firstDayOfWeek`, device `startOfWeek`).
    - `/wellness-service/wellness/daily/im/{d}` returns the day's
      `moderateMinutes`/`vigorousMinutes` and the week so far
      (`weeklyModerate`, `weeklyVigorous`, `weeklyTotal` = moderate + 2 ×
      vigorous, `weekGoal`). Sunday carried the week that began the Monday
      before; Monday reset to zero. The daily summary's
      `moderateIntensityMinutes`, `vigorousIntensityMinutes` and
      `intensityMinutesGoal` hold the same daily values, and their sums matched
      the weekly rows for two full weeks, so Raring reads days from summaries.
    - `/goal-service/goal/user/effective/{d}` returns
      `[{goalType, goalValue}]`, including `INTENSITY_MINUTES`, equal to the
      summary's `intensityMinutesGoal`; Raring doesn't need the request.
    - So the plan week follows Garmin's intensity window, not the start-of-week
      setting: totals match Garmin's weekly total. Whether the watch's own
      intensity widget also resets on Monday is unconfirmed.
- **4. Today that fits the time of day** (commit "Lead Today with the card
  that fits the time of day"). The hero (gauges, headline, chips and item
  6's reconciliation line) and the metric cards stay put; one advice card
  leads, chosen in Rust (`phase.rs`, `TodayView.focus`) from the displayed
  night's wake and the shared sleep plan: this morning, right now, tonight's
  plan two hours before the window, wind-down and late night. No effort call
  or countdown after wind-down; an unsynced night is neutral. A Settings
  switch turns it off; few or scattered wake times, travel or a plan that
  doesn't fit fall back to the fixed layout with a one-line reason.
  Boundaries, fallback and advice order are under Rules in the spec. A
  cached Today past its phase is read again, and the visible one is re-read
  at the boundary, deferred while touching. `raring show today --at` and
  `raring demo` give clock-controlled views; the mockup is `tod-*` in
  `design/mockup/`. No new endpoints. A real-account sync (60 requests, no
  429s or auth failures) gave a Garmin plan that ran through every phase. A
  jet-lag rating counts toward the travel fallback only on a recorded
  night. Review fixes: the neutral wait for an unsynced night holds in every
  fallback; the displayed day follows the usual wake (day sleepers keep their
  night through an afternoon wake) instead of a noon cutoff; wake times are
  read on a clock circle (see Usual wake and Displayed day under Rules).
  Not yet checked on a phone (see follow-ups).
- **6. Explain the numbers** (commit "Explain readiness, Body Battery and
  training status"). Why sheets on the Readiness, Body Battery and Training
  details, and a one-line reconciliation on Today when readiness and another
  score point different ways; rules and wording in the spec. Built from
  stored responses, no new requests. Verified on a real account:
  - Readiness: each `*FactorFeedback` is POOR, MODERATE, GOOD, VERY_GOOD or
    NONE; NONE comes with a 0% and means unrated (now a gap). The snapshot
    carries the values behind the factors: `sleepScore`, `recoveryTime`
    (minutes), `hrvWeeklyAverage`, `acuteLoad`. `feedbackShort` codes seen:
    TAKE_IT_EASY, LISTEN_TO_YOUR_BODY, WELL_RECOVERED, EXCELLENT_RECOVERY,
    LET_YOUR_BODY_RECOVER, TIME_TO_RECHARGE, ENERGIZING_NAP,
    GOOD_SLEEP_LAST_NIGHT.
  - Daily summary: `bodyBatteryAtWakeTime`, `bodyBatteryDuringSleep` and
    `bodyBatteryActivityEventList` (SLEEP, ACTIVITY, NAP, RECOVERY events
    with `bodyBatteryImpact`, a GMT start plus `timezoneOffset` in ms,
    duration, `activityName` and `shortFeedback`); the Body Battery report
    has the same events without names. `bodyBatteryDynamicFeedbackEvent.
    feedbackShortType` reads `{DAY|SLEEP_PREPARATION|SLEEP_TIME_PASSED}_
    {STRESSFUL|BALANCED|RECOVERING|NOT_STRESS_DATA}_AND_{INACTIVE|ACTIVE}
    [_AND_EXERCISE|_AND_INTENSIVE_EXERCISE][_AND_BB_LOW]`.
  - Training status: `acuteTrainingLoadDTO.acwrStatus` (LOW, OPTIMAL, HIGH),
    `trainingStatusFeedbackPhrase` (PRODUCTIVE_6) beside a numeric
    `trainingStatus`, and the load balance's `trainingBalanceFeedbackPhrase`
    (AEROBIC_LOW_SHORTAGE, AEROBIC_HIGH_SHORTAGE). STRAINED days coincided
    with HRV status LOW.
  - Follow-ups, in priority order: compare the sheets' wording with Garmin
    Connect on a phone; readiness `feedbackLong` (MOD_HRV_LOW,
    POOR_RT_HIGH_SS_POOR) seems to encode Garmin's main reason but needs a
    mapping checked against Connect before use; training status
    `fitnessTrend` (the VO2max trend input) was always 0 with sport NONE,
    so the sheet shows the 30-day VO2max change instead; the end-of-day
    Body Battery feedback event can carry the previous evening's event, so
    only the live one is used; sheets for past days wait for item 11.
- **Instant view switching** (commit "Switch cached views immediately").
  Tab views are prefetched after the first render and cached by route for the
  current day; navigation displays cached data before a background refresh,
  without a status IPC. Successful syncs, changed last-sync timestamps and
  local day rollover invalidate in-flight reads and cached tabs while keeping
  the visible view until fresh data arrives. Scroll position is restored on
  the cached render; touch cancellation and backgrounding flush deferred
  updates. Status refreshes on start, resume and sync, independently of
  navigation so a concurrent tab switch cannot discard a newer sync or
  sign-out result. Rust view and status commands read SQLite off the WebView
  thread. On emulator-5572
  with a 37 MB full-history database (debug APK, 10 warm tab activations):
  tab activation-to-first-paint was 87–521 ms before, with the full-screen
  loading state on every switch; after cache-invalidation fixes it was
  8–53 ms, with no full-screen loading state.
  Direct Today IPC still took ~480–500 ms on Android debug, but animation
  frames continued (~29–30 frames instead of 0–1); no persistent connection
  was needed. No Garmin endpoints or account requests were involved.
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
  Health Status costs one extra request on an ordinary sync, two during the
  morning two-day refresh, and up to 30 for the first 30-night download on a
  new install; older days are skipped so history range requests retain budget.
  Other web GraphQL range queries remain a possible future optimization.
- **2. Watch sync freshness** (commit "Show watch upload freshness and missing
  capabilities"). Verified read-only on a real account:
  `/device-service/deviceregistration/devices` returned an array of five
  registrations; exactly one had `primaryActivityTrackerIndicator: true`, with
  explicit `sleepSkinTemperatureCapable`, `hrvStatusCapable`,
  `sleepScoreCapable` and `trainingReadinessCapable` booleans.
  `/device-service/deviceservice/user-device/{deviceId}` returned an object
  with numeric epoch-millisecond `lastUploadTimestamp` and
  `lastDownloadTimestamp` (some registrations had null upload), plus `deviceId`,
  `userId`, `deviceStatus`, `applicationId` and `applicationVersionId`.
  Scrubbed, synthetic-valued response-shape fixtures are in
  `crates/core/tests/fixtures/`. Today distinguishes watch upload from Raring's
  own fetch, offers an Android Connect launch with plain-guidance fallback, and
  explains unsupported missing metrics only for explicit false capabilities.
  A 404 device list is checked again after an hour, not on every morning sync;
  without a primary tracker, ambiguous registrations produce no watch or
  capability claim. An overdue sleep upload appears only in the upload card.
  Capture used 30 requests (profile, 19 daily/activity kinds, four activity
  details, list and five details); a budget-2 live sync stored both device
  responses and Today showed a watch upload time. Across 32 reads there were
  no 429 or auth failures. Phone/watch Connect launch with the real installed
  app remains to verify.
- **3. Sleep rhythm and bedtime coaching** (commit "Add sleep rhythm and
  bedtime coaching"). Sleep shows Garmin's Sleep Alignment against its
  optimal window with the last seven nights, and Garmin's seven-day average
  bed and wake times on Consistency. The shared plan (Today, Sleep and the
  reminder) gains a wind-down time, Garmin's reasons for the need, a
  steady-timing tip and evidence behind a tap. The weekly check-in gains a
  timing review with one focus. The off-by-default bedtime reminder fires at
  a configurable lead (30 min) from an Android alarm, once per night,
  rescheduled on sync, settings, timezone, reboot and app-update events. Verified on the
  maintainer's Forerunner 970 from 18 stored nights (2026-09-29 to
  2026-10-16) plus two direct reads and a 20-request sync on 2026-10-17, no
  429s:
  - `dailySleepDTO.sleepAlignment` came back every night: `status`
    (`ALIGNED`, `AHEAD`, `BEHIND`, and `JET_LAG` with a null window on a night
    with no recorded sleep), `optimalSleepWindowStartMins`/`EndMins`/
    `MidpointMins` in minutes from local midnight of the morning the night
    ends (negative is the evening before; the window is
    `sleepNeedBaselineMins` long), `lastSleepMidpointMins` (the recorded
    sleep's midpoint, to the minute), `deviceId` and
    `isPreferredActivityTracker`. The window moved 10 min a night during the
    first week, then held. No calibrating status, feedback text or
    supported-device flag appeared.
  - `nextSleepNeed` is filed under the next day's `calendarDate`; its
    `recommendedBedtimeStartMins`/`EndMins` count from the evening's
    midnight. It carries `baseline`, `actual`, `feedback` (`INCREASED`,
    `NO_CHANGE_NO_ADJUSTMENTS`) and adjustment codes: `trainingFeedback`
    (`HIGH_ACWR`, `DAILY_ACTIVITY_AND_TODAYS_LOAD`), `sleepHistoryAdjustment`
    (`INCREASING_LOW_QUALITY`), `hrvAdjustment`, `napAdjustment`
    (`NO_CHANGE`). `displayedForTheDay` was false on a morning read, true on
    older ones; the hourly sync refetches today's sleep, so the plan follows
    any update.
  - `/sleep-service/stats/sleep/daily/{start}/{end}` returns
    `overallStats.averageLocalSleepStartTime`/`EndTime` (seconds from local
    midnight; the bedtime mean is signed across midnight) and per-night
    `individualStats[].values` including `sleepAlignmentStatus` and
    `sleepAlignmentOswStart`/`End`; nights without sleep are left out.
    Raring stores seven days ending today as `sleep_stats`, refetched until
    it includes last night.
  - No Sleep Consistency score or status appears in either response, so the
    descriptive chart stays.
  - The emulator scheduled the alarm, remapped it after a timezone change,
    and posted the reminder once. Not yet checked on a phone, on other
    watches or across a daylight-saving night (see follow-ups).

- **1. Full history backfill** (commit "Download the whole Garmin history
  in the background"). After the per-day sync of the last 90 days, each run
  spends its remaining budget on range requests, resuming from cursors
  (`history:<series>:lo/hi`). Verified on a real account: about 260 requests
  covered activities back to 2011 and daily summaries back to 2014, with no
  429s at the client's 300 ms pacing; a normal sync afterwards made no
  history requests. On the emulator, against a synthetic stand-in server,
  the Android background job signed in, downloaded the whole history over
  its first run and one hourly run, Settings went from "Downloading" to
  "All of your Garmin history", and a later run made no history requests.
  Still to check on a phone. Open: the first background run (budget 1400)
  takes about 7 minutes at real pacing, close to WorkManager's 10-minute
  stop; the emulator, slower per request, ran 22 minutes in the foreground
  without being stopped. Consider shorter chained runs if phones stop it.
  Range limits and shapes, all verified:
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
