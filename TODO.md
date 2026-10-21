# Raring TODO

The open backlog. **Build next** is the work to pick up, in priority order.
The sections after it hold what can't be built yet: checks on a phone or
desktop, reads against a real Garmin account, and decisions for the
maintainer. When one of those settles a question, move the resulting work into
Build next. Shipped items move to [docs/done.md](docs/done.md) (newest first)
with their commit and verified endpoint facts.

Every item keeps the spec's principles: visual first, opens instantly from
local data, Garmin's numbers, nothing leaves the phone but Garmin reads and
writes and street maps' area downloads. New data goes into an existing
screen or its detail before it earns a new screen.

When an item ships, update `docs/spec.md` in the same change. Endpoints are
on `connectapi.garmin.com` unless noted. "Verified" means a read against a
real account returned the data. Unverified endpoint candidates come from
[python-garminconnect](https://github.com/cyberjunky/python-garminconnect)
0.3.19 unless another source is linked; public client code is not proof of
availability on every account or watch.

## Build next

In progress on branch `feat-five-followups` (worktree
`~/.paseo/worktrees/1c8zm467/feat-five-followups`): core-generated fixtures with a swim and
a walk (8), desktop notification clicks (in 11) and naps when an old day is
opened (approved). Move each to `docs/done.md` when it merges.

### 1. Garmin writes, starting with weight

The write rules (item 21) shipped in `crates/core/src/writes.rs`, but nothing
in the app writes yet. They cover editing an existing resource. A weigh-in is
created rather than edited, so the first write also extends the rules to
create and delete. Weight goes first because the Add a weight button
already exists on Profile (disabled) and Connect shows a weigh-in without
waiting for a watch sync.

1. **Weight write spike** (approved 2026-10-21). On the maintainer's account,
   from the CLI and one request at a time: add one weigh-in with
   `POST /weight-service/user-weight` (unverified), read it back through
   `weight/dateRange`, check it in Garmin Connect, then delete it with the
   client's delete-weigh-in endpoint (unverified). Record the request body,
   units, timestamp and time zone handling, whether the reply carries the new
   `samplePk`, and how the delete addresses the entry.
2. **Create and delete in the write rules.** Never retry a create
   automatically: a sent but unconfirmed create could otherwise save twice.
   Before sending, read the day's weigh-ins; if the same value is already
   there, say so instead of sending. A delete still needs the person's
   confirmation. Keep the change log and the existing outcomes, and add the
   rules to the spec's Rules: Garmin writes.
3. **24a. Add a weight** on Profile: the weight only (Garmin's measurement
   system sets kg or lb), time defaulting to now. Hold Android's `SyncGate` or
   the desktop credential gate around the write as sync does, and save
   refreshed tokens. Store the read-back weigh-in so Health shows it without
   a sync. In the same change, rewrite the website's privacy page and the
   README, which say Raring only reads from Garmin, and update the spec's
   Profile entry.
4. **Delete a weigh-in** from the weight detail, with confirmation.
5. **24b. Hydration** (`/usersummary-service/usersummary/hydration/log`),
   then **24c. blood pressure** (`/bloodpressure-service/bloodpressure`; it
   can't be edited, only deleted and added again). Both unverified.

Body fat, BMI and other composition fields are out of scope for the first
weight write.

### 2. Stress timeline on clock-change days (before 1 Nov)

The stress timeline and its workouts keep a fixed 24-hour axis from the
day's start, so on a 25-hour day the last hour drops off and later times read
an hour out. Use the heart-rate chart's elapsed day length (Rules: Heart-rate
chart) for the stress samples, workouts and labels together, with spring and
fall tests.

### 3. Plan opens a chosen day and week

Plan has no dated route. Add one, then:
- Today's week cells open the tapped day instead of Plan at today (PP-B13);
  Progress's calendar already does.
- Week navigation (‹ ›) opens next week, so Add, Move and Reschedule can offer
  its days (7a); today they offer only this plan week's remaining days.

### 4. Plan and Progress fixes

- **PP-N5**: sessions during an illness or pause still read Planned and count
  as placed; show them "On hold", muted and not placed (a Goal ledger rule
  change).
- The review of the week in progress (`#review/<this week>`) labels every
  goal Not counted; say "so far" instead, or route it to Plan.

### 5. Health and Today details

- **Missing-metric reasons on Health**: Health doesn't say when the watch
  can't record skin temperature or HRV status; Today explains only readiness
  and the sleep score.
- **Body Battery's latest time** (review m5): the detail gauge says "Latest
  synced" without a time; `BodyBatteryCard` has no reading time yet.
- **Vital details repeat the hero value** in the chart readout (review m15).
- **Sleep HR detail link**: `metric/sleepHr` lost its only link when Today's
  stat became Resting HR; Health Status accounts still see Sleep HR on
  Health's vitals. Link it from Sleep's overnight heart rate or fold it into
  that vital.
- **Old days without Health Status**: an opened old day without a Health
  Status row shows an empty vitals card and offers a Vitals read. Fall back to
  the stored nightly records before offering a fetch (the emulator demo shows
  this empty card).

### 6. Watch temperature follows Garmin's measurement setting

Distances follow it everywhere (see docs/done.md); the watch temperature
still reads °C.

### 7. Chart axes

Round y ticks with the unit on the top label (review m1, m2), including
Progress's activity chart (143, 72). Decide with it whether the missing-day
dots under bar charts stay; line charts keep them under a dashed join across
each gap.

### 8. Fixtures from the core

The browser preview's training, strength and workout fixtures are hand-made
and disagree (lane vs board strength counts, squat bests, the sets check time
before the workout, latest load/VO₂ and overnight HR across pages). Generate
them from `raring demo` as `raring show` does, and add a swim and a walk to
`raring demo`.

### 9. Garmin's estimates as dated points

- **VO₂ max and Fitness Age** (review M5): plot one point per Garmin estimate
  with a faint step line, no weekly or monthly averaging and no missing-day
  marks, and date the VO₂ max hero (Fitness Age already keeps daily points in
  every range) ("52 · Garmin estimate · Sun 12 Oct").
  First confirm with a read-only probe whether stored `max_metrics` and
  `fitness_age` rows repeat the estimate daily or appear only on the day
  Garmin made it.
- **Endurance, hill score and running tolerance** (review M12): each is a bare
  number. Carry the estimate's date and Garmin's level (classification word)
  in `TrainingView` and show them on the detail once the fields are verified,
  or stop the tiles from opening a detail until there is history.
- **Fitness Age history backfill**: a past date's `fitnessage/{date}` returns
  that day's estimate, so one budgeted read per month (its last day) would
  give the history chart and Progress's Long-term months a point each.

### 10. Per-day ranges on charts

Daily points are coloured against the current range (Rules: Chart colours).
Carry each day's own Garmin bounds and status in the trend contract so a past
day is rated against its own range, and draw the acute-load band per day from
each day's Training Status `minTrainingLoadChronic`/`maxTrainingLoadChronic`.

### 11. Smaller items

- **Sign-in error wording**: map the native sign-in failures (no network,
  cancelled, Garmin refused) to one plain line each once their messages are
  known; Raring now only drops the "Error:" prefix.
- **Desktop notification click**: a desktop notification opens no screen;
  Android's tap targets (Today, the review, Sleep · Tonight) have no desktop
  twin.
- **Widget labels at 11 sp** (review m13): 11 sp fails
  `StatusWidgetLayoutTest` at the provider minimum size with large fonts
  ("Readiness" 108 > 100 px on emulator-5560); the 10 sp layouts pass at font
  scales 1.0/1.3/1.5. Needs a different minimum-size layout (fewer labels or a
  larger minimum), proven by that test.
- **Help pages**: "How targets count" and "Rest days and paused weeks".
- **Website and README pictures** predate the street map; republish them from
  synthetic data (`npm run screenshots`, then `publish-screens.mjs`).
- **Routes across ±180°**: both route fits take longitudes as they are, so a
  route crossing the antimeridian fits nearly the whole world. Unwrap its
  longitudes in the core and `routemap.ts`, and wrap tile x when downloading
  and drawing.
- **Chart legibility check**: it compares text with other text and with
  `data-mark` markers, not bars, points or bands; mark more shapes as charts
  change.

## Check on a phone or desktop

Grouped by area. The maintainer's phone and watch (Forerunner 970) unless
noted. Browser checks at 320–1100 px don't replace WebView text zoom,
TalkBack or Doze.

### Before or on 1 Nov (daylight-saving night)

- The late-night card holds until the usual wake on the new wall clock, and
  the one-hour offset change doesn't switch to the fixed layout.
- Garmin's window and bedtime minutes stay wall-clock times, and the Android
  reminder still fires at the wind-down time.
- The all-day heart-rate chart's elapsed time on the change day (synthetic
  23-/25-hour cases pass; a physical watch's source-zone behaviour is
  unverified).

### Accessibility and large text

- On the Fold: the tabs other than Today (approved), largest system text,
  TalkBack and sheets with the keyboard open.
- TalkBack and largest text on: the amount rows and comparison controls, the
  warm-graphite surfaces and contrast, Sleep · Tonight, Your day and the Sleep
  charts, the Fitness Age bar and weight chart, the stress month, the
  activities calendar at 360 px, the stats row and session card.

### Living with it for a few days

- Time of day: the morning, daytime, evening, wind-down and late-night cards
  at real times, the switch after resuming, and a night that syncs late.
- Colours: days outside the usual ranges, a stressful week, a low Body
  Battery, Fitness Age with Garmin's real cadence, weight with real weigh-ins.
  Whether the stress calendar's colours and outlined days read without a key,
  hollow days read as "the watch was off", and the 30D/90D stress averages
  match how the weeks felt. Then decide the amber stress nights and the
  workout strip (Decide).
- Fitness Age window for a week: the card after This week on a vigorous day,
  under Recorded today when the estimate drops, the push-morning lever line
  and Plan's line.
- The stats row and session card on a Coach strength day and a Coach cardio
  day. Whether Garmin's daily resting HR is in the summary when the night
  first syncs (until then the stat says "Not recorded").
- Changing the time to fall asleep moves tonight's reminder and Today's
  wind-down without a sync.
- Strength's resize-observer delivery warnings appear during rapid browser
  resizing; investigate the chart observer cycle only if it affects the
  device.

### Speed with the full history (release build)

- Tap through the tabs, Readiness, metrics, a workout, Strength, Profile and a
  weekly review, also right after a sync and a local change; note any
  "Preparing this view…" and how long it shows. If pages wait, the next costs
  are rows read more than once per view (Today reads its 14 recent nights
  several times) and 30-day vitals and stress histories parsed whole; a
  per-read row memo or SQLite `json_extract` would remove them.
- Trend charts: each range draws without a visible wait; the overlay picker
  works one-handed.
- Opening an old calendar day whose stress was never stored (one Garmin read)
  doesn't feel slow across months of history.
- The activities calendar back to 2011: year jumps, busy days, TalkBack.
- Street maps: a first open (download and draw) and a cached open. The
  emulator's debug build took about 1.5 s to a drawn map, 18 ms for a card.
- History backfill: the first background run (budget 1400) takes about
  7 minutes at real pacing, close to WorkManager's 10-minute stop. If phones
  stop it, use shorter chained runs.

### Notifications, widgets and background sync

- The bedtime reminder overnight: a ten-minute alarm window can be deferred in
  Doze; if reminders arrive late, offer exact alarms behind Android's
  alarms-and-reminders permission.
- Tapping each notification: morning, weekly check-in, bedtime → Tonight.
- Both widgets and the silent notification after a Connect sync: readings,
  upload time and the latest HR sample against Today; metric taps, permission
  denial, Settings off/on, reboot and overnight Doze; other launchers and
  lock-screen privacy.
- The restyled notification on Android 16 with wallpaper colours (checked on
  an Android 15 emulator only). Android before 12 moves "As of" to the
  collapsed header; untested.
- The per-kind alert channels replaced the old "Raring" channel; a phone that
  changed that channel's sound or importance starts from the defaults.
- The app's notifications switch off in Android Settings while the permission
  stays granted (`areNotificationsEnabled`).
- Battery and radio cost, before considering faster polling.
- Upgrading an installation and reconnecting the same Garmin account keeps
  cached history and local targets.
- Android `fetchDetail` (opened past days and workouts) with real
  credentials; only the credential-less guard has run.

### Desktop

- Reminders in a real session: Linux with a notification daemon and with Do
  Not Disturb (which accepts and records a reminder nobody sees), macOS and
  Windows.
- The import chooser and preview on Linux, macOS or Windows.
- Export to Downloads, and sharing an export from a physical phone.

### Compare with Garmin Connect and the watch

- A real night: Sleep's alignment, window and plan.
- Today's watch sync freshness and the Connect launch with the real app.
- Health Status: real outside-range statuses and visual agreement.
- One day's resting HR, activity markers and off-wrist gaps.
- The Why sheets' wording.
- Training's thresholds card (HR, pace, running power, FTP, acclimation),
  including Garmin's speed seconds-per-metre, acclimated-altitude metres and a
  measurement-system change.
- The gear card's units, status and threshold, with a retired item and an
  account without gear.
- A Forerunner 970 run with an HRM 600: feel and perceived effort (a stored
  feel of 0 reads Very weak), running form and step speed loss (Raring reads
  the list's `avgStepSpeedLoss`/`avgStepSpeedLossPercent`).
- A real pool swim and an open-water swim. A metric pool swim's list row has
  `poolLength` in centimetres with `unitOfPoolLength.factor` 100 plus
  `averageSwolf`, `avgStrokes`, `strokes` and stroke rate; the detail's swim
  fields and a yard pool's factor are still synthetic; stroke streams are
  verified on paddleboarding only.
- **A full strength session**: record one with a lap press after each set and
  weights entered, compare the workout screen with Connect, then decide
  whether the compatibility page moves from partial to Tested.
- Refresh from Garmin against a Connect edit made more than 48 hours after a
  workout started, including a removed set.
- Travel on a real trip: more than an hour of offset change shows the fixed
  layout until a night is recorded in the new zone.

### With people

- **Unfamiliar participants (26a)**: at least three, including simple-needs
  and detail-oriented people, on the shipped app with the walkthrough in
  [design/mockup/experience](design/mockup/experience/README.md). Record first
  meaning, discovery, comprehension and pressure; consented notes stay in
  ignored `.data/`. The maintainer's approval doesn't replace this.

## Probe a real account

Read-only unless noted, serialized, within the budget, notes in `.data/`.

- **Garmin Coach status and week boundary**: after a completed and a skipped
  Coach session, one `fbt-adaptive` read to see which
  `adaptiveCoachingWorkoutStatus` values Garmin sends and whether next week's
  tasks appear at the boundary. Until then Garmin's status is shown as
  Garmin's and never completes a session.
- **Real-data journeys (26a)**: walk U01–U64 through `.data/garmin-run.sh`,
  including migration of an earlier weekly plan and a long history on the
  phone (cold start and Progress timings).
- **Opened past days and workouts**: `raring day-detail YYYY-MM-DD` and
  `raring workout-detail ID` run the app's path. Record how far back
  `dailySleepData`, `dailyStress`, the daily summary, Health Status and an
  activity's splits/zones/charts answer.
- **Populated schedule and races**: on an account with a scheduled plan and a
  race, compare the next session's name and day and the race countdown with
  Connect. `workoutScheduleByPeriod` entries stay read-only by name and date
  until a populated response verifies sport, duration and completion or
  cancellation fields; never infer completion from names or times. A
  different race discriminator gets only its verified shape; a full 10-event
  page is partial coverage.
- **Estimates' history**: `enduranceScoreScalar` and `hillScoreScalar` (web
  GraphQL: start, end, weekly aggregation) and the fields behind the
  endurance and hill levels.
- **Workout fields**: impact load's unit (`impactLoad`; metres gave more
  impact than distance on a run with long walks, which Garmin counts half;
  compare with Connect or running tolerance's `acuteImpactLoad`), power zone
  bounds (`/activity-service/activity/{id}/powerTimeInZones`), which foot
  `avgGroundContactBalance` is, Connect's unit for `totalWork` (it matched
  average power × time in kilocalories), intervals (`typedsplits`) and
  weather (`/activity-service/activity/{id}/weather`).
- **Garmin's rules behind its words**: what the stress word follows (not a cut
  of the daily average); `vigorousDaysAvg` (not "days with 10+ vigorous
  minutes": 0.1 on 15 Oct and 0.3 on 16 Oct over the same days); readiness
  `feedbackLong` codes (MOD_HRV_LOW, POOR_RT_HIGH_SS_POOR) against Connect.
- **Exercise display names**: Garmin's own localized names before adding a
  table; Raring title-cases keys ("One Arm Dumbbell Row").
- **Sleep details**: weekday sleep windows (`userSleepWindows`): whether a
  weekday names the bedtime day or the wake day, and whether they carry
  defaults when `userSleep` does; whether `averageLocalSleepStartTime` before
  midnight is negative or wrapped (both accepted).
- **Opportunistic, when seen**: Coach `workoutPhrase` codes beyond `BASE`,
  `LACTATE_THRESHOLD`, `POOREST_SLEEP_RECOVERY`, `FORCED_REST` and strength
  `_GENERIC_GYM`/`_GENERIC_BODYWEIGHT`, and whether a completed task keeps
  its phrase; sleep alignment codes on other watches and while calibrating
  (CALIBRAT, NOT_ENOUGH, INSUFFICIENT, LEARNING are treated as calibrating);
  journal entries on a consenting account (18a; never create one for QA);
  body composition from a compatible scale and `/latest` with no weigh-ins.
- **Another account or watch**: missing/404 thresholds, null calibration and
  stale power; Sleep HR's 30-night fallback without Health Status; whether a
  Sunday-start watch's intensity widget resets on Monday.

## Decide

- **Trends with unequal coverage**: suppress previous-period changes, or keep
  showing both recorded-day counts (current) without a significance claim.
- **Weekly trend buckets**: Monday–Sunday now; align with Garmin's
  intensity week, which the weekly plan uses?
- **Palettes** (PP-N12): goal colours repeat sport colours on Progress (Zone 2
  and Ride blue, intensity and Swim teal, strength pink); separate them in the
  spec's Look first.
- **Remaining ink data marks**: all-day heart rate, sleep consistency bars,
  load focus bars and the training lanes' line: neutral blue, a zone colour,
  or ink as a reference? And whether a workout's older stream colours (pace
  teal, elevation tan, cadence purple, power ochre) follow the neutral blue.
- **Fitness Age ranges**: drop 7D–30D, which sparse estimates leave empty, for
  90D, 1Y and Max.
- **After living with stress**: draw only Poor nights if amber reads as noise;
  a light band for the workout strip if it doesn't tie a rise to a session.
- **Re-reads**: read `fitness_age` again on a vigorous day (one request); re-read
  a workout's detail for 48 hours so a later self-evaluation is seen (only
  details carry it, so list-only recordings show it once opened).
- **Older strength sets**: confirm history sync's `strength_without_sets` queue
  fills them before a person opens the workout, or leave them to Refresh from
  Garmin.
- **Schedule after midnight**: the cached 14-day schedule covers one fewer
  day; add a budgeted date-roll refresh if the last day matters.
- **Street map areas**: one fresh download for an area over a year old when a
  card there is opened.
- **Activities search**: a name search over stored recordings ("parkrun") if
  finding events by date isn't enough.
- **Old exports**: exports from v0.1.30 or earlier don't name their account and
  are refused; is a migration path (the cached profile in the raw entries)
  worth it?

## Later

### Garmin writes after weight

Settings people touch day to day, written back so the watch picks them up on
its next sync. Raring isn't a full settings app; watch faces, screens,
sensors and audio prompts stay in Connect.

- **23. Day-to-day settings**, in order: alarms (list, toggle, add, edit time
  and days); sleep schedule (bed and wake times, linked from tonight's plan);
  daily goals: steps, intensity minutes, floors, hydration
  (`PUT /goal-service/goal/user/effective/{type}/{d}`); do not disturb, if it
  reaches the watch. Before alarms ship, with the maintainer's approval,
  verify that a device settings PUT carrying only `deviceId` and `alarms`
  (from a fresh GET) leaves other device settings unchanged; if Garmin clears
  omitted keys, send the whole fresh object (`Resource::keep`). Alarm removal
  through the same PUT is unverified. "Waiting for watch sync" ends only after
  a normal sync stores a newer device detail.
- **22. Strength workout builder**: start with a workout-push spike on the
  Forerunner 970: create and schedule a strength workout, confirm it reaches
  the watch, do it and read the sets back. Check whether morning readiness
  sync arrives before a session (gates the readiness brake) and how Garmin
  counts repeats against the ~100-step cap. Then build workouts with
  progression: double progression with the smallest equipment step, reps
  first if the next step is over 10% heavier, one step down after two
  sessions below range and two sessions' wait on a new exercise. Schedule
  progressed weights, swapping in held weights after morning sync on Rest or
  Easy days only if the spike supports it. Warn at 90 steps, offer a split at
  100, subject to the spike's repeat count; ship one-time scheduling first.
  See `design/mockup/NOTES.md` and the `builder` mockup; add the progression
  math and readiness brake to the spec's Rules when this ships, with builder
  help and compatibility for tested models. Plan's "Next up" strength card
  (mockup) waits for this. Keep: missing lap presses must not reduce weekly
  strength credit (`StrengthWeek.strengthActivities` is distinct from
  `sessionsWithSets`).
- **25. Activity edits and strength set correction**: rename, change type,
  notes (`PUT /activity-service/activity/{id}`), then correct recorded sets.
  For a failed save, retain local edits, fetch the sets again before retrying
  in case they changed in Connect, and explain what was rejected (see
  `design/mockup/NOTES.md` and the `fix` mockup).

**Item 20, the write spike, results on a Forerunner 970:**
- 2026-10-17: an alarm time edit (full settings object PUT, 200) and the
  floors goal (200) showed on the watch after one ordinary Connect sync. The
  sleep schedule PUT (204) reads back correctly; watch pickup unconfirmed. The
  alarm's `changeState` stayed `UNCHANGED` and the write still reached the
  watch. `lastDownloadTimestamp` on
  `/device-service/deviceservice/user-device/{id}` moves when the watch syncs.
  Re-sending the whole user settings object reorders `availableTrainingDays`
  but keeps its values.
- Device settings (alarms verified writable):
  `GET/PUT /device-service/deviceservice/device-info/settings/{deviceId}`
  holds `alarms` (each with a `changeState`), `dndEnabled`, `sleep` and
  `activityTracking`.
- User settings (verified readable): `/userprofile-service/userprofile/user-settings`
  holds `userSleep` (sleep and wake time, seconds after midnight) and
  `userData` (units, training days). PUT the full object with only the
  changed keys.
- 2026-10-19, each read back with only the written fields changed; watch
  pickup unconfirmed: `PUT /biometric-service/heartRateZones` with the whole
  per-sport array (`maxHeartRateUsed`, `zone1Floor`–`zone5Floor`;
  `changeState` reads back `UNCHANGED`) returned 204. User settings
  `firstDayOfWeek` (Monday is `dayId` 3, `sortOrder` 3) and
  `preferredLongTrainingDays` returned 204. Device `startOfWeek` (full object
  PUT) returned 200. `availableTrainingDays` with one rest day returned 204;
  two rest days in a row are rejected (400 "Cannot set consecutive rest
  days").

### More Garmin data

- **Activity FIT originals in the export**: `/download-service/files/activity/{id}`
  is an unverified candidate at one request per activity. Count the
  activities and show the request count as an explicit choice; download within
  the budget and 429 backoff, resume after interruption, and put the files in
  a separate export. The offline JSON/CSV export makes no requests.
- **Ride and workout extras**: lap rows (`lapDTOs`: per-lap cadence, power and
  running form), a ride's power curve (`maxAvgPower_1` … `_18000`), cycling
  dynamics, mountain-bike grit and flow, rucking pack weight
  (`beginPackWeight`, grams) and Garmin's stress change (`differenceStress`,
  sign unverified).
- **Gear threshold notification**: opt-in consent, Garmin's own positive
  threshold and per-gear/per-threshold deduplication; don't change the
  existing notification cadence.
- **Street map labels and pan/zoom**, left out on purpose (2026-10-20); the
  stored tiles could feed them.
- **A second map source**, if OpenFreeMap stops serving: VersaTiles
  (`tiles.versatiles.org`, keyless, no CDN) serves the Shortbread schema,
  which needs its own class mapping. `tile.openstreetmap.org` and
  `vector.openstreetmap.org` forbid offline and prefetch use; Protomaps'
  planet builds discourage hotlinking.

## Parked

- **Women's health (opt-in)**, formerly item 19: cycle and pregnancy views
  (`/periodichealth-service/menstrualcycle/…`) as context on readiness and
  vitals; sensitive, off by default, excluded from notifications unless
  chosen. Not started until someone using Raring with cycle data can verify
  it.

## Not planned

- AI-written insights: they often restate the numbers and would need a
  service that sees the data.
- Social feeds, competitive leaderboards/challenges, engagement rewards and
  app-open or wearable-use streaks. Private, evidence-backed milestones and
  optional quiet celebrations shipped with item 26; no new wellbeing score.
- Nutrition logging: Garmin limits it to its subscription.
