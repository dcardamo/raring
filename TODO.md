# Raring TODO

The prioritized backlog: work top to bottom, one item at a time, each shipped
on its own. Every item keeps the spec's principles: visual first, opens
instantly from local data, Garmin's numbers, nothing leaves the phone but
Garmin reads and street maps' area downloads. New data goes into an existing
screen or its detail before it earns a new screen.

When an item ships, update `docs/spec.md` in the same change, then move the
item to **Done** with its commit. Endpoints are on `connectapi.garmin.com`
unless noted. "Verified" means a read against a real account returned the
data. Unverified endpoint candidates come from
[python-garminconnect](https://github.com/cyberjunky/python-garminconnect)
0.3.19 unless another source is linked; public client code is not proof of
availability on every account or watch.

## Now

### Adaptive layout device follow-ups

- Dan approved Today in the Fold's actual WebView. Check the other tabs,
  largest Android system text, TalkBack and sheets with the keyboard open
  on the Fold. Browser text scaling does not replace those device checks.
- Strength's existing resize-observer delivery warnings still appear during
  rapid browser resizing. Geometry and state checks pass; investigate the
  remaining chart observer cycle separately if it affects device behavior.

### 26a. Experience follow-ups

In priority order; the redesign shipped (Done, item 26) without them.

1. **Unfamiliar participants:** at least three, including simple-needs and
   detail-oriented people, on the shipped app, with the walkthrough in
   [design/mockup/experience](design/mockup/experience/README.md). Record
   first meaning, discovery, comprehension and pressure; consented notes stay
   in ignored `.data/`. Dan's approval is a maintainer decision, not this gate.
2. **Garmin Coach status and week boundary:** after a completed and a skipped
   Coach session, one serialized read-only `fbt-adaptive` read to see which
   `adaptiveCoachingWorkoutStatus` values Garmin sends, and whether next
   week's tasks appear at the week boundary (see 15a). Until then Garmin's
   status is shown as Garmin's and never completes a session.
3. **Real-data journeys:** walk the U01–U64 journeys on a real account through
   `.data/garmin-run.sh` (read-only), notes and screenshots only in `.data/`,
   including migration of an earlier weekly plan and a long history on the
   phone (cold start and Progress timings).
4. **Phone checks:** TalkBack, large system text, Doze/background sync, and
   tapping each notification (morning, weekly check-in, bedtime → Tonight).
5. **Populated Garmin schedule:** `workoutScheduleByPeriod` entries stay
   read-only by name and date until a populated response verifies their
   sport, duration and completion fields.

### Data presentation prototype follow-ups

- Dan approved the prototype for coordinator merge after consolidating
  Sleep's repeated stage-average copy. The tidy-up is done below; this lane
  does not merge, push or release.
- Check the new amount rows and comparison controls with TalkBack and the
  largest system text on a phone. Browser geometry checks are not a physical
  WebView or screen-reader check.
- Historical baseline bands need each day's actual Garmin bounds/status in
  the trend contract. Daily points are now coloured against the current
  range (Rules: Chart colours), which is labelled as current; with per-day
  bounds each past day could be rated against its own range, so a 90D chart's
  early days don't borrow today's. No account requests in this lane.
- Decide whether Trends should suppress previous-period changes for unequal
  coverage. Current comparisons show both recorded-day counts, without a
  significance or favorable-direction claim; no new coverage threshold.
- Regenerate coherent browser fixtures from the core: the existing handcrafted
  strength counts, latest load/VO2 and overnight HR disagree across pages.
  Identical before/after fixtures expose those differences rather than hiding
  them in presentation logic.

### Visual language follow-ups

- Maintainer review of the refined warm-graphite prototype. Dan approved
  keeping neutral surfaces, stronger data accents, the original light-selected
  range picker and a repair of the chevron bar. Settle the direction before a
  broader data/layout pass; this is not release approval.
- Check the new surfaces and contrast on the maintainer's phone with TalkBack
  and the largest system text. Browser chart checks cover 360–1100 px and
  1/1.3/2× text, not physical WebView text zoom.
- Review the refined tokens on main's new Today/Sleep components: the four
  morning stats, separate session card, 24-hour clock and recent-nights chart.
  They are integrated in the prototype; their layout and data remain main's.
- The Android window background (`MainActivity.kt`) and the WebView
  `theme-color` (`app/index.html`) are still the earlier navy `#0B0F1A`, not
  the app's `#111317`; launch and system-bar edges can show the old colour.

### Chart colour follow-ups

From the chart colour pass, in priority order.

- Live with it on the maintainer's phone: days outside the usual ranges
  (amber), a stressful week, a low Body Battery, the Fitness Age bar with
  Garmin's real estimate cadence and the weight card with real weigh-ins;
  TalkBack on the age bar and the weight chart.
- Fitness Age history compares every estimate with your age now. Stored
  `fitness_age` rows carry Garmin's `chronologicalAge`; the trend could carry
  it so an older estimate is compared with the age then.
- Remaining ink data marks: all-day heart rate (detail and Your day), sleep
  consistency bars, load focus bars and the training lanes' line. Decide for
  each: neutral blue, a zone colour, or ink as a reference mark.
- A workout's Along the way keeps a colour per stream (pace teal, elevation
  tan, cadence purple, power ochre, performance condition ink) from before
  the chart colour pass; its running form and stroke streams are the
  neutral blue. Decide whether the older streams follow.
- The Fitness Age detail offers 7D–30D, which Garmin's sparse estimates
  usually leave empty (as on VO₂ max); consider 90D, 1Y and Max only.

### Stress by day follow-ups

From the stress calendar and partly worn stress days, in priority order.

- Live with it on the maintainer's phone: whether the calendar's colours and
  outlined days read without a key (the "?" explains them), whether the
  hollow days on Health's stress card read as "the watch was off", and
  whether the 30D and 90D averages without them match how the weeks felt.
  Check the month at the largest Android font and with TalkBack.
- Garmin's sleep score rates overnight stress against a 0–15 range, so Fair
  nights can be common and the night bars mostly amber. If the amber reads as
  noise on the phone, draw only Poor nights rather than adding a day/night
  toggle.
- The workout strip above the stress detail's plot is small. If it doesn't
  tie a rise after a hard session to the workout at a glance, try a light
  band like Asleep's.
- Tapping an old calendar day whose per-day stress was never stored reads it
  from Garmin once (Rules: Opened past days and workouts); check on a phone
  that months of history-only days don't make that feel slow.
- Confirm what Garmin's stress word follows before the app explains it: it
  isn't a cut of the daily average (a lower-average day can get the more
  stressful word), and what it does follow is undocumented.
- The stress timeline, and the workouts on it, keep a fixed 24-hour axis
  from the day's start, so on a 25-hour clock-change day the last hour drops
  off and later times read an hour out. Use the heart-rate chart's elapsed
  day length (Rules: Heart-rate chart) for the stress samples, workouts and
  labels together, with spring and fall tests.

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
- Garmin's weekday sleep windows (`userSleepWindows`, `MONDAY`…`SUNDAY`
  beside `DAILY`): whether a weekday names the night's bedtime day or its
  wake day is unverified; Raring reads the displayed day's. Check against a
  schedule that differs by weekday, and whether the windows carry Garmin's
  defaults when `userSleep` does.

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
- Check desktop reminders in a real session: Linux with a notification
  daemon and with Do Not Disturb, plus macOS and Windows. A missing Linux
  server leaves the reminder pending, but Do Not Disturb still accepts and
  records one that nobody sees.
- A desktop notification opens no screen when clicked; Android's tap targets
  (Today, the review, Sleep · Tonight) have no desktop twin yet.
- Garmin's average bedtime before midnight: confirm whether
  `averageLocalSleepStartTime` comes back negative or wrapped (Raring accepts
  both).

### Trends follow-ups

From item 11, in priority order.

- Check the trend charts on a real phone with the full history: each range
  should draw without a visible wait, and the overlay picker should be easy
  to use one-handed.
- Verify the new opened-day and opened-workout reads on a real account. The
  read-only `raring day-detail YYYY-MM-DD` and `raring workout-detail ID` run
  the same core path as the app. Record how far back `dailySleepData`,
  `dailyStress`, the daily summary, Health Status and an activity's
  splits/zones/charts answer; Health Status may be the shortest.
- An opened old day without a Health Status row shows an empty vitals card and
  offers a Vitals read. If Garmin has no Health Status that far back, fall
  back to the stored nightly records before offering a fetch (the emulator
  demo shows this empty card).
- Naps for an opened older day stay unfetched (naps sync only the last two
  days), so the nap card reads unavailable. Decide whether the open should
  read `sleep-service/sleep/naps/{date}` too.
- An opened older strength workout's sets still arrive from history sync's
  `strength_without_sets` queue, not from the open; confirm that happens
  before a person opens the workout, or leave sets to Refresh from Garmin.
- Android's `fetchDetail` with real credentials is unverified on a device: the
  demo has no Garmin tokens and refuses before JNI, so only compilation and
  the credential-less guard were exercised.
- Weekly trend buckets run Monday to Sunday. Consider aligning them with
  Garmin's reported intensity-week boundary, which the weekly plan already
  uses; a display preference need not match that data window.
- Fitness Age history is limited to downloaded per-day estimates, though a
  past date's estimate is readable (Fitness Age window follow-ups); endurance,
  hill score and race predictions have no long-history rows. The web GraphQL lists
  `enduranceScoreScalar` and `hillScoreScalar` (start, end, weekly
  aggregation); unverified.

### Health and profile review follow-ups

From the Health/Profile UX review (`.data/ux-review/health-profile.md`), in
priority order; the bugs and most majors shipped (Done).

- **VO₂ max and Fitness Age as dated estimates** (review M5): plot one point
  per Garmin estimate with a faint step line, no weekly or monthly averaging
  and no missing-day marks, and date the VO₂ max hero ("52 · Garmin estimate ·
  Sun 12 Oct"). First confirm with a read-only probe whether stored
  `max_metrics` and `fitness_age` rows repeat the estimate daily or appear
  only on the day Garmin made it; the charts already open on the latest
  estimate.
- **Endurance, hill score and running tolerance details** (review M12): each
  is a bare number. Carry the estimate's date and Garmin's level in
  `TrainingView` and show them on the detail, or stop the Training tiles from
  opening a detail until there is history (see the Trends follow-up on
  `enduranceScoreScalar`). Coordinate with the Training lane.
- **Trend chart axes** (review m1, m2): round y ticks with the unit on the top
  label, and decide whether the missing-day dots under bar charts stay (line
  charts keep them under a dashed join across each gap). Chart legibility lane.
- **Body Battery's latest time** (review m5): the detail gauge says "Latest
  synced" without a time; `BodyBatteryCard` has no reading time yet.
- **Vital details repeat the hero value** in the chart readout (review m15).
- **Notification state on a phone**: on emulator-5560 (in-place debug
  install, signer `0d59d1bf…7a63`, synthetic demo) a revoked
  `POST_NOTIFICATIONS` showed the blocked line with Allow and greyed the six
  switches, and re-granting cleared it on the next Settings open. Still to
  check on a phone: the app's notifications switch turned off in Android
  Settings while the permission stays granted (`areNotificationsEnabled`).
- **Sign-in error wording**: map the native sign-in failures (no network,
  cancelled, Garmin refused) to one plain line each once their messages are
  known; Raring now only drops the "Error:" prefix.

### Chart legibility follow-ups

From the chart legibility pass, in priority order.

- Check Sleep · Tonight, Your day and the Sleep charts on the maintainer's
  phone at the largest Android font size. The browser check emulates text
  zoom by multiplying every font size; WebView's own text zoom is unverified.
- The check compares text with other text and with `data-mark` markers, not
  with bars, points or bands; mark more shapes as charts change.

### Fitness Age window follow-ups

From the Fitness Age window on Today, Plan and Health, in priority order.

- Live with it on the maintainer's phone for a week: the card after This
  week on a vigorous day, under Recorded today the day Garmin's estimate
  drops, the push-morning lever line, and Plan's line; TalkBack on the bars
  and the large-text layout of the card.
- Garmin's estimate is read once a day at the first sync, so a Garmin update
  later that day reaches Today's drop card the next morning. Decide whether a
  vigorous day should read `fitness_age` again (one request, in budget).
- Vigorous days: `vigorousDaysAvg` is not "days with 10 or more vigorous
  minutes": the 15 Oct 2026 estimate said 0.1 with two such days in its
  window, while 16 Oct said 0.3 over the same days. Find Garmin's rule before
  counting days anywhere.
- Fitness Age history: a past date's `fitnessage/{date}` returns that day's
  estimate (see the Done entry), so a budgeted backfill could fill the
  history chart and Progress's Long-term months beyond the downloaded days.
  Opened past Health days now store theirs; one read per month (its last
  day) would give every month a point, at one request per month of history.

### Today session and morning stats follow-ups

From the session card and morning stats, in priority order.

- Check the stats row and session card on the maintainer's phone with a
  Coach strength day and a Coach cardio day, at the largest Android font.
- Coach `workoutPhrase` codes beyond those seen (`BASE`,
  `LACTATE_THRESHOLD`, `POOREST_SLEEP_RECOVERY`, `FORCED_REST`, strength
  codes ending `_GENERIC_GYM`/`_GENERIC_BODYWEIGHT`) fall back to the name
  and duration; record new codes and their meaning when seen. Whether a
  completed Coach task keeps its phrase is unverified.
- Evaluate the lowest overnight heart rate (a minimum of Garmin's two-minute
  sleep samples) as a Today stat; it would be new Raring math sensitive to
  artifacts, so it needs a rule and a real-night comparison first.
- On an account without Health Status, compare Today's Sleep HR range (the
  sleep record's 30-night fallback) with Garmin Connect's sleep heart rate.
- Time to fall asleep: confirm on a phone that changing it moves tonight's
  reminder and Today's wind-down without a sync.

### Today and Sleep review follow-ups

From the Today and Sleep review's remainder, in priority order.

- Health doesn't say when the watch can't record skin temperature or HRV
  status; Today now explains only readiness and the sleep score. Show the
  other missing-metric reasons on Health's vitals.
- Widget labels are 10 sp, under the 11 minimum (review m13; the
  notification now uses Android's notification text sizes). 11 sp fails `StatusWidgetLayoutTest` at the provider minimum size
  with large fonts ("Readiness" 108 > 100 px on emulator-5560); the 10 sp
  layouts pass at font scales 1.0/1.3/1.5. Reaching 11 sp needs a different
  minimum-size layout (fewer labels or a larger minimum), proven by that test.

### Instant page opens follow-ups

From the instant page opens, in priority order.

- Check on the maintainer's phone (release build): tap through the tabs,
  Readiness, metrics, a workout, Strength, Profile and a weekly review, also
  right after a sync and after a local change, and note any "Preparing this
  view…" and how long it shows. The timings above are desktop only.
- If a page still waits on the phone, the next costs are rows read more than
  once within one view (Today reads its 14 recent nights several times) and
  30-day vitals and stress histories parsed whole. A per-read row memo, or
  SQLite `json_extract` as the trend charts use, would remove them.

### Activities calendar follow-ups

From the activities calendar, in priority order.

- Check on the maintainer's phone with the full history (back to 2011): jump
  years from the month's name, tap busy days (a count after one icon), and
  check TalkBack reads each day's recordings and the largest system text at
  360 px. Browser checks covered 320–1100 px at normal text only.
- If finding an event by its date isn't enough, consider a name search over
  stored recordings ("parkrun", "marathon") in the same section, from local
  data. Not started; a maintainer decision.

### Workout fields follow-ups

From the self-evaluation and workout fields pass, in priority order. Field
names and units were read from a real account's stored detail and list
payloads and Garmin's Forerunner 970 manual; nothing was compared with
Connect's own screens.

- Compare a real Forerunner 970 run with an HRM 600 against Connect: feel
  and perceived effort (a stored feel of 0 reads Very weak), running form
  and step speed loss. The detail's own names for step speed loss are
  unknown; Raring reads the list's `avgStepSpeedLoss` and
  `avgStepSpeedLossPercent`, which only reach the workout while its list
  row is stored.
- Impact load (`impactLoad` on a run's detail) isn't shown: its unit is
  unverified. Metres, as running tolerance sends, gave more impact than the
  run's distance on a run with long walking stretches, which Garmin says
  count half. Compare with Connect, or with that day's change in running
  tolerance's `acuteImpactLoad`.
- A self-evaluation added or changed in Connect later isn't seen: the
  activity detail is read once. Consider re-reading it for 48 hours after
  the activity, as strength sets are, or with an explicit refresh.
- Time in power zones: the list carries `powerTimeInZone_1`–`_5` on runs and
  `_1`–`_7` on rides; the zones' watt bounds need Garmin's power zones
  (`/activity-service/activity/{id}/powerTimeInZones` is unverified).
- Ground contact time balance: the list's `avgGroundContactBalance`
  doesn't say which foot it is; add it once a real run shows the side in
  Connect.
- Garmin's `totalWork` matched average power × time in kilocalories on real
  runs and rides; Connect's unit for Work is unverified, so it isn't shown.
- Lap rows (`lapDTOs`) carry per-lap cadence, power and running form; the
  splits show pace and heart rate only. Garmin's intervals (`typedsplits`)
  and weather (`/activity-service/activity/{id}/weather`) are unverified.
- A ride's power curve (`maxAvgPower_1` … `_18000` in the list), cycling
  dynamics (torque effectiveness, pedal smoothness), mountain-bike grit and
  flow, rucking pack weight (`beginPackWeight`, grams) and Garmin's stress
  change (`differenceStress`, sign unverified) aren't shown.
- Self-evaluation in the Activities list: only details carry it, so older
  list-only recordings would show it only once opened.

## Next

### 7a. Weekly plan follow-ups

- Planning next week ahead: Plan shows one week, so Add, Move and Reschedule
  offer only this plan week's remaining days. Week navigation (‹ ›) could open
  next week once Plan has a dated route.
- Check on a watch whose account starts the week on Sunday whether its own
  intensity-minutes widget resets on Monday like Garmin's weekly rows.
- The mockup's "Next up" strength workout card on the plan screen waits for
  planned training (item 15) or the workout builder (item 22).
- Offline help pages "How targets count" and "Rest days and paused weeks"
  when the Help screen ships.

### Plan and Progress review follow-ups

From the UX review (`.data/ux-review/plan-progress.md`), in priority order;
the rest shipped (Done: Plan and Progress review fixes).

- **PP-B13** (rest): Progress's calendar now opens the tapped day, but Today's
  week cells still open Plan at today. Plan has no dated route to select a
  day or another week; add one, then open the tapped day.
- **PP-N5**: sessions during an illness or pause still read Planned and count
  as placed; show them "On hold", muted and not placed (a Goal ledger rule
  change).
- **PP-N12**: goal colours repeat sport colours on Progress (Zone 2 and Ride
  blue, intensity and Swim teal, strength pink); separate the palettes in the
  spec's Look first.
- The review of the week in progress (`#review/<this week>`) still labels
  every goal Not counted; say "so far" instead, or route it to Plan.
- Progress's activity chart has uneven y ticks (143, 72); round tick steps
  belong with the chart legibility work.

### 8a. Strength follow-ups

- Check an explicit Refresh from Garmin against a Connect edit made more than
  48 hours after a real workout started: the button, the last-checked time and
  a removed set need real data, and only synthetic coverage exists so far.
- **Confirm a full Forerunner 970 session.** Read-back is only partly
  verified on the watch: no watch-recorded session with named, weighted sets
  (or one run from a structured workout) has been read yet. Record one with
  a lap press after each set and weights entered, compare Raring's workout
  screen with Connect, then decide whether the compatibility page can move
  from partial to Tested.
- **Exercise display names**: Raring title-cases Garmin's keys
  ("One Arm Dumbbell Row"). Look for Garmin's own localized names before
  adding a table.
- The weekly plan's strength target (item 7) counts Garmin strength
  activities, including ones timed without sets. `StrengthWeek.strengthActivities`
  is distinct from `sessionsWithSets`; missing lap presses must not reduce
  credit toward the weekly activity goal.

### Training and workout review follow-ups

From the read-only UX review of Training and workouts, in priority order.

- **Real swim and walk**: a real metric pool swim's list row has
  `poolLength` in centimetres with `unitOfPoolLength.factor` 100, plus
  `averageSwolf`, `avgStrokes`, `strokes` and the stroke rate; the activity
  detail's swim fields (`activeLengths`, `averageStrokes`,
  `averageStrokeDistance`) and a yard pool's factor are still synthetic.
  Compare a real pool swim (and an open-water swim) with Connect. Stroke
  rate and distance per stroke streams (`directStrokeCadence`,
  `directStrokeDistance`) are verified on paddleboarding only. Add a swim and
  a walk to `raring demo`.
- **Endurance and hill levels**: show Garmin's classification word and the
  reading's date beside Endurance, Hill score and Run tolerance once their
  fields are verified (the tiles now carry a chevron only).
- **Units elsewhere**: running tolerance, Progress distance totals and gear
  still read km, and the watch temperature reads °C; follow Garmin's
  measurement setting there too.
- **Acute-load band per day**: the 30-day chart and metric detail use today's
  optimal range; draw Garmin's stored range for each day where it exists
  (each day's Training Status carries `minTrainingLoadChronic` and
  `maxTrainingLoadChronic`).
- **Fixtures from the core**: the preview's training, strength and workout
  fixtures are still hand-made and disagree in places (lane vs board
  strength counts, squat bests, the sets check time before the workout);
  generate them from `raring demo` as `raring show` does.

### Thresholds follow-ups

- Compare the Training card's HR, pace, running power, FTP and acclimation
  with Garmin Connect and a real watch after a sync. Verify Garmin's speed
  seconds-per-meter and acclimated-altitude meters against the displayed
  units, including a measurement-system change.
- On another compatible watch/account, check missing/404 threshold responses,
  null calibration and truly stale power readings; these cases have only
  synthetic coverage so far.
### 14a. All-day heart rate follow-ups

- Compare one day's resting value, activity markers and off-wrist gaps with
  Garmin Connect and the watch on a real phone; the current device review is
  emulator-only.
- Compare the elapsed-time chart with a real DST change or travel day. Synthetic
  23-/25-hour transitions retain their readings and GMT-based overlays, but a
  physical watch's source-zone behavior is still unverified.

### 9–10a. Status surface follow-ups

Items 9 and 10 are shipped, and item 26 kept both surfaces with its shared
rest, illness, limits and privacy rules.

- On a real phone, add both widgets, enable the silent notification and
  compare readings/upload time after a Connect sync. Check metric taps,
  permission denial, Settings off/on, reboot and overnight Doze.
- Check upgrading an existing installation and reconnecting the same Garmin
  account: cached history and local targets should survive identity
  verification. Different-account isolation has been tested with synthetic
  accounts, not by switching real accounts.
- Measure phone battery/radio cost before considering faster polling. Keep
  the existing hourly/morning sync; emulator rendering measurements and
  reported 0 mAh do not establish phone battery cost.
- Compare the notification's latest HR sample with Today's available data;
  absent readings stay blank. Rendering itself makes no Garmin requests.
- Check other launchers and lock-screen privacy choices. Android can defer
  updates and allows users to dismiss ongoing notifications. Minimum-size
  RemoteViews passed emulator tests at font scales 1.0, 1.3 and 1.5.
- Check the restyled notification on a phone (Android 16, wallpaper
  colours): it was checked on an Android 15 emulator in light, dark and
  1.5× text only. Android before 12 keeps the header in the collapsed view,
  so "As of" moves there; that path is untested.
- The per-kind alert channels replace the old "Raring" channel: a phone that
  had changed that channel's sound or importance starts from the defaults.

### Import follow-ups

- Check the desktop chooser and preview on a real desktop (Linux, macOS or
  Windows). The dialog is built and unit-tested around the core, but this
  lane's host is headless, so only the Android picker was checked end to end.
- Exports made before the manifest named the archive format and its account
  are refused, so a backup taken by v0.1.30 or earlier cannot seed a new
  phone; decide whether a migration path (reading the cached profile from the
  raw entries) is worth it.

### Export follow-up: optional activity FIT originals

`/download-service/files/activity/{id}` is an unverified candidate. Each
activity would cost one request; first count the activities and show that
request count as an explicit choice. Download within the existing request
budget and 429 backoff, resume after interruption, then add the files to a
separate export. The offline JSON/CSV export does not make network requests.

### 15a. Planned training follow-ups

- On an account with a scheduled Coach plan and an upcoming race, compare
  Raring's next-session name and Garmin calendar day with Garmin Connect,
  and confirm the race title and all-day countdown on a real phone. Current
  empty-account reads cannot verify populated payload fields or Coach task IDs.
- **Verified 2026-10-19 (read-only, one account with an adaptive Garmin
  Coach plan; structure only):** `active-plans` lists the plan with
  `trainingPlanCategory` `FBT_ADAPTIVE`, `trainingStatus.statusKey`,
  `startDate`/`endDate` local timestamps and a null `taskList`. Adaptive
  sessions are **not** in `workoutScheduleByPeriod` (empty) or
  `tasks/{id}` (empty). They are in
  `GET /trainingplan-service/trainingplan/fbt-adaptive/{trainingPlanId}`:
  `taskList` holds the current week only (one task per day, `weekId` of the
  week), each with `calendarDate`, `dayOfWeekId`, `workoutOrder` and
  `taskWorkout` (`workoutName`, `estimatedDurationInSecs`, `restDay`,
  `trainingEffectLabel` — seen AEROBIC_BASE, LACTATE_THRESHOLD, RECOVERY,
  UNKNOWN, INVALID — `sportType.sportTypeKey` — seen `other` for cardio,
  `strength_training`, null on rest days — `adaptiveCoachingWorkoutStatus`,
  only NOT_COMPLETE seen, `workoutId` null). The month calendar
  (`/calendar-service/year/{y}/month/{m0}`, month 0-based) also lists them as
  `fbtAdaptiveWorkout` items; Raring doesn't read it. Raring now reads
  `fbt-adaptive` (see spec, Planned training and Rules: Garmin Coach).
- Verify with a completed and a skipped Coach session which
  `adaptiveCoachingWorkoutStatus` values Garmin sends, and whether next
  week's tasks appear at the week boundary. Until then Garmin's status is
  shown as Garmin's and never treated as completion.
- Coach tasks (`GET /trainingplan-service/trainingplan/tasks/{planId}?localeKey=en`)
  returned empty for an adaptive plan and stay unused.
- Check whether Garmin's scheduled-workout list exposes explicit completion
  or cancellation fields on populated accounts before assigning either meaning.
  Do not infer completion by matching activity names or times.
- A cached 14-day-ahead schedule covers one fewer future day after midnight.
  The empty card now names the cached dates; consider a date-roll refresh
  (still budgeted and paced) if the uncovered last day matters in practice.
- If a populated calendar uses a different race discriminator, add only that
  verified shape. A full 10-event page is partial coverage; look for documented
  pagination/filtering before considering another request.

### Street maps follow-ups

In priority order.

- **Phone timing**: measure a first open (download and draw) and a cached
  open on a real phone in a release build. Measured so far only on the
  software-rendered emulator in a debug build: about 1.5 s from opening the
  page to the drawn map, 18 ms for a drawn card.
- **A second source**: OpenFreeMap's TileJSON URL is built in. If it stops
  serving, VersaTiles (`tiles.versatiles.org`, keyless, no CDN in front)
  serves the Shortbread schema, which needs its own class mapping;
  `tile.openstreetmap.org` and `vector.openstreetmap.org` forbid offline
  and prefetch use. Protomaps' planet builds discourage hotlinking.
- **Old areas**: a stored area keeps the week's map it was downloaded from.
  Consider one fresh download for an area over a year old when a card there
  is opened.
- **Website and README pictures**: the published screens predate the street
  map; republish them from synthetic data (`npm run screenshots`, then
  `publish-screens.mjs`).
- **Routes across ±180°**: like the plain route before them, both fits take
  longitudes as they are, so a route crossing the antimeridian (Fiji,
  Chukotka) fits nearly the whole world. Unwrap its longitudes in the core and
  `routemap.ts`, and wrap tile x when downloading and drawing.
- Labels and pan/zoom were left out on purpose (2026-10-20); the stored tiles
  could feed them later.

## Later: more Garmin data

Each is a tile or detail in an existing screen first.

18a. **Journal populated-shape follow-up**: when a consenting account has
     entries, compare Garmin's actual fields/units with the community example,
     then expand only verified display values. Never create an entry for QA.
19. Women's health: parked (see Parked).

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
    - More writes on the maintainer's account (2026-10-19), each read back
      with only the written fields changed; watch pickup unconfirmed.
      `PUT /biometric-service/heartRateZones` with the whole per-sport array
      (`maxHeartRateUsed`, `zone1Floor`–`zone5Floor`; `changeState` reads
      back `UNCHANGED`) returned 204. User settings `firstDayOfWeek`
      (Monday is `dayId` 3, `sortOrder` 3) and `preferredLongTrainingDays`
      returned 204. Device `startOfWeek` (full object PUT) returned 200.
      `availableTrainingDays` with one rest day returned 204; two rest days
      in a row are rejected (400 "Cannot set consecutive rest days").
21a. **Write rules follow-ups** (before the first write ships):
    - With Dan's approval, verify on the maintainer's watch that the device
      settings PUT carrying only `deviceId` and `alarms` (built from a fresh
      GET) leaves every other device setting unchanged; the verified alarm
      write in item 20 sent the whole object. If Garmin clears omitted keys,
      send the whole fresh object for this resource (`Resource::keep`).
      Alarm removal through the same PUT is unverified.
    - The first write command must hold the Android `SyncGate` / desktop
      credential gate around the write, as sync does, and save refreshed
      tokens; the core takes a caller-owned `Client`.
    - "Waiting for watch sync" ends only after a normal sync stores a newer
      device detail; the log doesn't fetch it on its own.
    - The website's privacy page and the README say the app only reads from
      Garmin and never sends goals or plans there. Rewrite both with the
      first shipped write.
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

## Done

- **One brand mark everywhere** (commit "Replace the app icon with one gauge
  mark everywhere"). The launcher icon (and so Obtainium's), the notification
  small icon, the in-app brand mark, favicon and raring.app now share one
  mark: the app's readiness gauge with ink progress, a graphite track and a
  green marker. `scripts/brand-icon.mjs` writes every vector copy;
  `cargo tauri icon src-tauri/icons/icon.json` the raster sizes. Android's
  adaptive icon now has a graphite background, a safe-zone foreground and a
  monochrome layer for themed icons (it was a dark tile baked onto white, so
  the circle mask cropped the gauge). The dark-theme notification tint follows
  the marker green. Verified on an emulator: launcher, status bar, shade and
  welcome screen; app checks, unit and UI tests and build pass.

- **App-wide adaptive layout** (commits "Group Plan and Progress sections for
  responsive grid", "Make sheet layouts wrap and scroll at narrow widths"
  and "Keep whole sections readable across folded and wide layouts").
  Replaced masonry and the 660px loose-child grid with intact sections and
  text-sized column thresholds. Today keeps its header and sync separate,
  metrics full-width and recordings with their lead. Plan and Progress group
  headings with their content; Training, Health, Sleep and details follow
  document-order rows. Navigation has its own shell row, named icons when
  large labels cannot fit, and no covered final content. Sheets wrap and
  scroll; strength selectors follow the named page container so only one
  is visible. Workout/weekly chart gutters scale with text.
  Verified: cargo workspace tests and clippy; Svelte/TypeScript, 137 unit
  tests, build and UI regression; Today states at 344–1100px; 20 page/section
  states at 344–1280px and twelve sheet/form states at 344/390/840px with
  1×/2× text and live resizing; chart and card-spill checks on all 42 pages
  at 360/390/720/1100px and 1×/1.3×/2× text. Synthetic screenshots are in
  `.data/screens/today-layout-*.png` and `.data/screens/adaptive-*.png`.

- **Self-evaluation and Garmin's workout fields in groups** (commit "Show
  how a workout felt and Garmin's workout fields in groups"). A workout shows
  the watch's self-evaluation as How it felt (Garmin's feel answer of five and
  perceived effort out of 10), and the summary fields Garmin sends that
  Raring left out: running form (max cadence, ground contact time, vertical
  oscillation and ratio, step speed loss), moving, best and grade-adjusted
  pace, elapsed time, max power, Training Stress Score, each leg's power,
  max cadence, strokes and stroke rate for paddling, lowest and highest
  elevation, breathing range, yoga's stress, sweat loss, resting calories,
  fluid and calories consumed, and lowest available stamina. Recorded by
  your watch groups them under headings, and Along the way adds the running
  form and paddle stroke streams. List-only recordings read the activity
  list's names for the same fields. Two bugs found on real data: rides
  recorded as `road_biking` or `virtual_ride` read as other sports (pace, not
  speed), and a pool's length read 100 times too long.

- **Dated stress pages name their day** (commit "Name the day on dated
  stress pages and read the calendar again with it"). An opened past day's
  stress detail said "average so far today"; it now says "average for Sat 26
  Sept", as the calendar opens such days often. When an opened day's detail
  arrives from Garmin, the calendar reads its month again, so the day's cell
  matches the page.

- **The night before on the stress calendar** (commit "Mark the night before
  on the stress calendar"). Each calendar day carries Garmin's rating of the
  night that ended that morning, from its sleep score (`sleepScores.stress.
  qualifierKey` and `avgSleepStress`; history `sleep_summary` rows carry both
  at the top level), as a bar on top in Garmin's colours and in the day's
  label. Combined in one cell rather than a separate calendar or a toggle:
  Garmin's day already counts the night's hours after midnight, and reading
  both together sets a stressful day with a poor night apart from one with a
  calm night. A day the watch was barely worn can still have a rated night.

- **Stress by day** (commit "Show stress by day on a month calendar"), from
  the maintainer asking whether daily stress would help on a calendar, with
  exercise stress set apart. The stress detail has a month of Garmin's word
  for each day (green Calm or Balanced, amber Stressful, red Very Stressful),
  partly worn days outlined and uncoloured, a dot on days with a recorded
  workout, the shown day ringed and named, and earlier months from local
  data (`stress_month`; CLI `raring stress-month [YYYY-MM]`). A tapped day
  opens its dated detail, and days chosen there replace each other, so Back
  returns to the opener. The detail's timeline marks recorded workouts above
  the plot. An awake-only word is neutral on the stress pill too; it used to
  take the average's colour, so "Very Stressful Awake" could be green. List
  pages' space-separated GMT start times now place activities on the
  heart-rate and stress timelines. No exercise filter: Garmin doesn't rate
  stress while you move, and readings right after a workout are marked
  rather than removed, which would be new Raring math.

- **Partly worn stress days** (commit "Leave partly worn stress days out of
  stress averages"), from the maintainer asking for stress by day. A stress
  day Garmin rated only in part (word `UNKNOWN`, or an awake-only word) stays
  out of the stress trend's values, averages, counts and comparisons; daily
  charts draw Garmin's average for it as a hollow point labelled "partly
  worn · not averaged", and it still counts for freshness. Verified on a real
  account's stored rows: the daily summary's `stressQualifier` is `UNKNOWN`
  on days the watch was off for long stretches while `avgStressLevel` is
  still set, sometimes from one or two hours of readings, so those days used
  to plot as ordinary ones. `*_AWAKE` words (`BALANCED_AWAKE`) appeared only
  on days without a recorded night; history rows carry the word in lowercase
  (`daily_summary` `stress.qualifier`, `very_stressful_awake`). The All
  range starts at the first reading, partly worn or not (commit "Start the
  stress history at its first reading, partly worn or not").

- **Joined chart points** (commit "Join chart points with a line, dashed
  across gaps"), from the maintainer: Long-term's VO₂ max and Fitness Age
  were bare dots, hard to read as a trend. Progress's line and dot charts
  and the daily trend charts (Health's cards, the metric details) now join
  neighbouring readings, with a faint dashed line across a gap; nothing is
  plotted for a gap, and the trend charts still mark missing days under the
  axis. On the trend charts Garmin's estimates keep the one line of "Join
  Garmin's VO₂ max and Fitness Age estimates in one line". A monthly average with no neighbour was invisible on a line
  chart and now has a dot. Fitness Age still has at most one point a month,
  only where estimates are stored (see the Fitness Age window follow-up on
  its history).

- **Dates for Progress's Cardio, Strength and Long-term** (commit "Choose
  the weeks and months on Progress's Cardio, Strength and Long-term"), from
  the maintainer: only Overall had a period picker. Cardio and Strength now
  choose 12 weeks, 6 months or a year of plan weeks and Long-term a year, 2
  or 5 years of months, each with previous and next like Overall (previous
  stops where the core's dates start, 1990) and kept across a detail and
  Back. An empty window says so and a chart without readings keeps its
  baseline instead of a 0–1 scale (Overall's activity chart drew one when
  the list was incomplete). On a copy of the full-history database (desktop
  release build) the default read took 21 ms, a year of weeks 20 ms and
  five years of months 146 ms; phone timings are unverified.

- **A past Health day's own Fitness Age, weight and week** (commits "Show a
  past Health day's own Fitness Age, weight and week" and "Past days: name
  intensity's real days, date weigh-ins by the day, read stress alone"),
  from the maintainer opening a day from Progress's calendar: Fitness Age
  was blank and "What sets it" gone on any day before the first synced
  estimate, the weight card showed today's weigh-ins, intensity minutes
  looked like the whole week's with this week's Plan behind them, and Body
  Battery's link said "today". Opening a past Health day now also reads that
  day's Fitness Age once (one more request in the day's budget; Garmin
  returns the estimate as of the date), the drivers come from the estimate
  the card shows, and a day Garmin has none for says so; the dated stress
  page never reads Fitness Age, and a night and a Health day of the same
  date are separate opens. Weight ends on the day and counts the weigh-in's
  age from it, intensity minutes name the days they cover (Garmin's
  whole-week row says the whole week) and a passed week opens its review,
  and the journal says its seven days end there. Checked in a browser
  preview on a copy of the full-history database; the phone's real fetch is
  unverified.

- **Fitness Age on Today and Plan** (commits "Show Fitness Age's vigorous
  window on Today, Plan and Health" and "Read Fitness Age's lever from the
  window's estimate and keep its card under Recorded today", the second from
  a review), from the maintainer wanting Today and
  Plan to show progress toward Health's Fitness Age levers, positively.
  The workout-driven lever is vigorous minutes, so Raring shows Garmin's
  vigorous driver live from the daily summaries, never the age moving per
  workout. Today: a "+24 vigorous min" chip on recorded workouts; a compact
  card after This week on a day with vigorous minutes (six-week average
  before and with today, six seven-day bars with today's minutes green,
  Garmin's age at its target), moving under Recorded today the day Garmin's
  estimate drops a whole year; on Garmin's push call before any vigorous
  minutes, one line names the lever. Plan: one read-only line with this
  week's vigorous minutes against Garmin's target (the maintainer chose a
  line over a goal). Health: the same bars under the drivers. How it
  counted: the workout's vigorous minutes, its week's total and the last day
  they count. Before/after mockups (synthetic) are in
  `.data/mockups/fitness-age/`. The demo seed's vigorous target is now 75.
  - Verified read-only on a real account (2026-10-20, five requests beside the
    stored daily rows): `/fitnessage-service/fitnessage/{date}` returns the estimate
    as of a past date, with `lastUpdated` its day or an earlier one (asking
    for 6 Oct 2026 returned 5 Oct's: Garmin doesn't estimate every day).
    `components.vigorousMinutesAvg.value` equalled the daily summaries'
    `vigorousIntensityMinutes` over the 42 days ending on the estimate's day,
    that day included, ÷ 6, for 3, 5, 8, 12, 15 and 16 Oct 2026; six Monday
    weeks (12 Oct: 4.0 against Garmin's 12.8) and the 42 days before the
    estimate's day (3 Oct: 24.7 against 18.3) did not. `targetValue` was 75
    and `numOfWeeksForIm` 6; a met driver (`rhr`) carries no target, and
    `bmi`'s `potentialAge` was older than the estimate itself, so only the
    lever's age is shown. Per-activity vigorous minutes can sum below the
    day's (all-day minutes count), so the window reads the daily summaries.

- **Phone feedback on Today and Training** (commits "Today: score ring for
  Sleep, green in-range vitals and no closing line", "Join Garmin's VO₂ max
  and Fitness Age estimates in one line", "Name the Sleep Score with
  Garmin's word" and "Read Garmin's optimal acute-load range and show it on
  Today after a workout"), from the maintainer's phone. Today's Sleep stat
  is Garmin's Sleep Score in a ring in its level's colour, like Readiness,
  with Garmin's word and the time asleep under it ("Good · 7:33"), instead
  of time asleep in a full ring against the need; every rating word in the
  row takes its tone colour. Sleep HR and HRV are green within the usual
  range and amber outside it in either direction (core `NightReading.tone`),
  as on Health's vitals and the charts. "That's today." is gone. VO₂ max and
  Fitness Age histories join Garmin's estimates in one line, dotted where it
  crosses days without one, with no gap marks; before, a line appeared only
  where two estimates fell on consecutive days. VO₂ max beside the status at
  the top of Training was tried and reverted: it looked wrong to the
  maintainer, pushed the status's Why below it and took VO₂ max out of "Is
  my fitness improving?". Garmin sets no daily load target, so under Done
  today acute load is a dot on Garmin's optimal range, as on Training.
  Verified on the maintainer's account: Training Status sends that range as
  `acuteTrainingLoadDTO.minTrainingLoadChronic` and `maxTrainingLoadChronic`
  (its `acwrStatus` agreed on every day checked), which Raring hadn't read,
  so Training had shown no range; the older weekly load tunnel
  (`loadTunnelMin`, `loadTunnelMax`, `weeklyTrainingLoad`) is null. Health
  Status's `HR` equals the sleep record's `avgHeartRate` and sits above
  Garmin's daily resting HR, so the stat stays "Sleep HR". Garmin's `maxmet`
  daily rows exist only on days it updates VO₂ max. The Sleep Score's word
  is `sleepScores.overall.qualifierKey`; Garmin's EXCELLENT (from 90) is
  unverified on this account.

- **Street maps under workout routes** (commits "Draw workout routes over
  street maps from OpenFreeMap" and "Keep street-map downloads to one area at
  a time on the source's host"), from exploring private, free and
  accountless maps. On by default, off in Settings → Street maps (which also
  erases them). The core downloads OpenStreetMap vector tiles (OpenMapTiles
  schema) from OpenFreeMap by area, the 8×8 block of tiles at the card's
  zoom (64 tiles, usually 20–40 km across), so the server sees an area,
  never the route; it decodes them itself and draws each card once into SVG
  path data (green, water, waterways, paths, minor roads, roads, major
  roads), kept per workout. Only a route card without a drawn map asks;
  other workouts in a stored area draw with no request. The WebView's CSP is
  unchanged. Verified 2026-10-20: OpenFreeMap needs no key or account and
  allows app offline downloads of at most 3,000 tiles or 50 MB with at most
  4 parallel requests and an identifying User-Agent; its privacy policy
  keeps no IPs in access logs, but Cloudflare sits in front and sees each
  request. One area's z13 tiles measured 5.4 MB in central London, 3.7 MB
  in Manhattan and 0.9 MB around Boulder. A drawn card is about 25 KB of
  path data for central Ottawa. On the emulator (debug build), the demo
  workout's map appeared about 1.5 s after opening, and a drawn card
  returned in 18 ms. MapLibre GL JS (about 1.6 MB minified) wasn't needed.

- **Activities calendar** (commit "Find activities on a month calendar"),
  from the maintainer wanting to find specific events. Training → Activities
  has a List / Calendar switch. The calendar reads one stored month from the
  core (`activity_month`; `raring show activities --date D`), never Garmin:
  weeks start on Garmin's week start, each day shows its recordings' sport
  icons in start order (two, or one and a count) and opens that day's list,
  and the month's recordings are listed under it. ‹ › step a month; the
  month's name opens a year of months, bounded by the oldest stored recording
  and this month. The sport filter applies to both views, and the view, month
  and day hold across a workout and Back. On a copy of the full-history
  database (1,170 recordings since 2011), the busiest month (66 recordings)
  took a median 19 ms per desktop release CLI run (no LTO), opening the
  database and printing included; Training took 62 ms the same way.

- **Notifications restyled** (commit "Restyle notifications for Android's
  shade and say what changed"). The status notification draws no card of
  its own: it uses Android's notification text styles, so it follows light
  and dark, and the header says "As of …" instead of the post time.
  Collapsed: Body Battery, stress and heart rate with icons, then "As of".
  Expanded: the focus as the title, its reason (hidden when empty), then a
  labelled row each with rounded bars (none for heart rate or a missing
  reading). The small icon is the app's gauge mark, tinted mint. Alerts
  use one channel per kind (Morning readiness, Weekly check-in, Health
  changes, Wind-down reminder, Garmin account) with a category. Copy:
  training status said Garmin's raw code ("Now PRODUCTIVE_6.") and fired
  when only its numeric suffix changed; it now reads "Training status:
  Productive" / "Was Maintaining." and compares the named status. VO₂ max
  and Fitness Age say the new whole value and the direction ("Down from
  45."). Vitals name the readings and their usual ranges ("SpO₂ 93% (usual
  95–99) · Skin temp +0.8°C (usual −0.4 to +0.4)") under "2 vitals outside
  your ranges".

- **Instant page opens** (commits "Read stored nights without their
  samples" and "Open pages whole and keep cached views through a sync"),
  from the maintainer seeing "Preparing this view…" after taps. View reads
  parsed whole stored nights (about 150 KB each, almost all per-sample
  arrays) up to 30 at a time and several times over: Today parsed 33 MB of
  JSON, the weekly review 88 MB. Nights are now read without their samples
  except the one night whose overnight charts Sleep draws; Your day and the
  weekly review no longer build whole Sleep pages, and an activity's type
  reads the list pages only when its detail lacks one. On a copy of the
  full-history database (37 MB, desktop release build, median read):
  Today 104 → 21 ms, Sleep 95 → 25, Training 49 → 19, Health 31 → 12,
  Plan 10 → 2, Progress 26 → 8, weekly review 280 → 11, a workout 11 → 3.
  Every view's JSON was identical before and after (491 views at five clock
  times, on two databases). In the app, Readiness, Gear, weekly goals and the
  metric pages open from their tab's prefetched view; a sync keeps cached
  views (each opens at once and is read again) instead of emptying them; and
  a page that isn't cached is read for up to 200 ms before it replaces the
  shown one, so a quick read opens it whole. No Garmin requests changed.

- **Chart colours, Fitness Age bar and weight layout** (commits "Colour charts
  by range, restore the Fitness Age bar and align weight", "Tint range bars,
  keep Health's weight range and say when Fitness Age matches" and "Drop the
  colour keys from charts"), from the maintainer's phone. Charts are no
  longer ink on ink: each metric is in one category, shown by colour alone
  (no colour key on any chart, the stress timeline's included). Garmin's
  levels (readiness, Sleep Score, stress, Body Battery) colour days and
  averages; usual ranges are a
  green-tinted band, readings inside green and outside amber; acute load is
  amber below and coral above Garmin's optimal range; Fitness Age is green
  under your age and coral over it; weight, VO₂ max, time asleep, monthly
  averages and range metrics without a shown band are one neutral blue. A
  line changes colour where it crosses a threshold; range bars (vitals,
  drivers) tint their range green under the dot. Health vitals outside
  their range are amber (core tone fair, still counted for the warning and
  notification), matching the charts. Health's Fitness Age card is the
  YOU/AGE bar again, green younger and coral older; it and a link open the
  new `metric/fitnessAge` page with the full history and your age as a
  dashed line. Weight uses Health's history-card layout (ranges on the card,
  period, trend weight and change, chart, reading, key, one link) and the
  detail leads with trend weight, then chart, changes and body composition.
  Intensity minutes' ring uses the intensity goal colour until met.

- **Sleep stage comparison copy** (commit "Say the sleep stage comparison once").
  One subheading explains the 30-night average and recorded-night coverage;
  each stage keeps its small average value and full accessible label. Different
  stage counts remain distinct rather than implying equal coverage. The repeated
  explanations and trailing note are gone; no data or core rule changed.

- **Data presentation prototype** (commit "Make metric comparisons and chart evidence explicit"). Shared period summaries separate existing core means and changes from the
  inspected day; current range bounds, source dates and recorded counts stay
  visible. History uses fixed 0–100 domains for bounded Garmin metrics,
  including overlays. Shared amount rows align stage/zone/stress/load-focus
  and muscle comparisons, preserve zero/unknown and remove ambiguous
  percentage columns. The zone-duration contract now preserves omitted, null
  and invalid durations as unknown (tested in Rust), rather than zero-filling;
  partial zone summaries no longer draw a misleading normalized strip. Sleep-stage comparisons now expose their core mean and
  per-stage sample count under the tested 30-calendar-day rule. Training's
  load preview uses the common history chart; Progress uses round ticks,
  neutral physiological traces and a coverage/goal key, with the resting-HR
  link corrected. The arbitrary Fitness Age ruler is gone; Readiness factors
  lead with Garmin's input and rating, not an unexplained percent. Today,
  Plan, Tonight, weight rules and the five tabs remain. Private research and
  comparison captures stay ignored; no new endpoints or account requests.
  Dan approved coordinator merge subject to the stage-copy tidy-up above;
  this lane remains branch-only, not released.

- **Visual language on the current Today/Sleep layouts** (commit "Apply the
  refined language to the current Today and Sleep components"). Main's four
  morning stats, session card, Sleep HR detail, sleep clock and recent-nights
  chart are integrated into the prototype. Their new facts and rules remain
  main's; cards, readings and plots use the refined tokens. TonightLine stays
  removed. Approval of this prototype still does not authorize release.

- **Visual language refinement** (commit "Strengthen data accents and repair
  the effort bar"). Data colors regain saturation while surfaces stay neutral;
  time-range pickers restore their light selection and original radii. The
  effort bar uses one shared SVG rail with matching chevron seams, a raised
  selected cap and separately aligned text, rather than overlapping clipped
  boxes of different heights. Scores, labels and actions are unchanged.

- **Warm-graphite visual language prototype** (commit "Prototype Raring's
  warm graphite visual language"). Opaque neutral surfaces, warmer ink,
  softer semantic colors, shared card/control radii, Inter structural headings,
  tabular Outfit readings, quieter owned icons and consistent chart strokes.
  Root navigation gains a neutral selected-icon backing; card headings become
  sentence case, gaps between cards grow to 16 px, and the lead gets more
  breathing room. Data, page order, chart domains, goal rules and actions are
  unchanged. A token parity/contrast test prevents the CSS and chart palettes
  from drifting. Private research and synthetic before/after captures stay
  ignored. Review and phone checks remain above; no release in this lane.

- **Tonight as a sleep clock** (commit "Draw tonight's plan as a 24-hour
  sleep clock"), from the maintainer's phone (v0.1.41): the horizontal
  Tonight timeline is replaced by a 24-hour ring (midnight at the top) with
  the planned sleep from asleep-by to the expected wake, the minutes to fall
  asleep lighter before it, the wind-down softer, Garmin's window as a short
  outer segment and a now dot; inside only the need, "In bed" and "Up".
  The facts sit below once, with the time-to-fall-asleep setting. "Last 7
  nights against need" becomes "Recent nights vs tonight's window": 14
  nights of recorded sleep on a clock axis against tonight's bedtime band
  and wake, missing nights marked, plus one below-need line. Today's
  evening and wind-down cards use the same clock, compact.

- **Today's session, morning stats and time to fall asleep** (commit
  "Separate today's session from the day and add the morning stats"), from
  the maintainer's phone (v0.1.41). Today's hero is about the day (the
  headline from Garmin's call, the effort strip with its reason under it);
  today's session is its own card below with its source (◇ Garmin Coach ·
  week N of M, ◇ Garmin calendar, Your plan), what it is from verified Coach
  fields (`workoutPhrase` gym/bodyweight, `workoutDescription` "<n>bpm",
  Garmin's effect label in plain words, "Coach eased this after a short
  night") and a fit line against the suggested effort, with Keep · Choose
  another inside it. The `coach_week` projection now keeps `workoutPhrase`,
  the parsed target, `weekId` and `durationInWeeks` from the stored payload
  (no new request). The last-night/readiness card is four compact stats:
  Sleep, Readiness, Sleep HR (Health Status HR, else the sleep record's
  overnight average against its own 30-night range, never resting HR) and
  HRV against Garmin's baseline, each opening its detail (a new
  `metric/sleepHr` page). Settings and the core gain "Time to fall asleep"
  (default 15, 0–60): in bed by = asleep-by − it, wind-down leads the earlier
  of Garmin's window and in bed, and the reminder, Today's wind-down line
  and the notification say "In bed by". Garmin's window and need are
  unchanged.

- **Today and Sleep minors** (commit "Drop disclaimers and key Tonight's
  missing nights"), from the Today and Sleep review (m1–m6, plus a
  2× overflow in Sleep's hero). Guardrail sentences are gone from the plan
  cards, Why this plan, the 7-night chart, the nap rows, Adapt's limits and
  routines, the readiness Why sheet and the status offer; alignment keys
  "Garmin’s longer-term window (dashed)". Today's plan card footer reads
  "Tonight’s plan" (it opens Tonight). The wake is "usual" from four
  recorded wakes ("usual · 9 nights"), not only at 14. The 7-night chart
  draws a missing night as a dotted gap, keys it and the darker below-need
  bars, and hides when nothing is recorded. The wind-down reminder is a
  switch row. Widget and notification labels stay 10 sp (m13 is open, see
  the follow-up). Sleep's hero and This week's "Left this week" rows wrap at
  large text instead of running off the screen. Not changed: m19's "84" over
  "Score" is line-box overlap only (glyphs clear at 2×); m21 (tab labels at
  2×) is outside this lane.

- **Sleep · Last night reads without a legend hunt** (commit "Make Sleep's
  overnight tiles, stages and history readable"), from the Today and Sleep
  review (M6, M11, M15, m1 naps, m8, m9, m16, m18). Overnight tiles open
  their metric (a past night only heart rate's dated page) with the night's
  clock ends and low/high; stage changes are neutral and sit in the Stages
  card, whose hypnogram ticks whole hours; Sleep history is Sleep score with
  an "Overlay time asleep" toggle, round ticks (`historyAxis`) and "More in
  Health"; naps say "not checked yet" or "Couldn’t read naps" and drop the
  disclaimer. The preview's sample trend now runs to last night. The stage
  legend, Garmin averages and Consistency key wrap at 2× text (they ran out
  of their cards at 720 px on main too, unnoticed by the check).

- **Readiness tells one effort story** (commit "Show Readiness's call on
  Today's effort bar and explain each factor"), from the Today and Sleep
  review (M9, M10, m15). Today's call is Today's chevron bar
  (`EffortBar.svelte`) with Today's reason line, under the same gate; each
  factor prints Garmin's rating word and a tap gives the Why sheet's
  sentence with the value Garmin used. An unsupported watch says so without
  a "waiting" headline. The demo's readiness now carries `sleepScore`,
  `hrvWeeklyAverage` and `acuteLoad`, so its Why sheet shows the values; the
  null values the review saw were the demo's, and the High-level summary
  still names only helpers (spec Rules: Why sheets).

- **Today says each fact once; Choose and This week explain limits**
  (commit "Cut Today's repeats and say why limits block a backlog item"),
  from the Today and Sleep review (B7, M1, M2, M4, M5, m7, m10–m12, m14,
  m17, m20). The Right now card (`TodayCard::Now`) and the chips row are
  gone; Your day's Body Battery value falls back to the day's daily-stats
  level (`bodyBatteryNow`) so nothing is lost. The upload time is only in
  the header. A recording leads as "Done today" above the activity, not its
  name twice. Notices sit above last night and readiness and list only
  readiness and sleep-score reasons. Your day hides when the calendar day
  has nothing recorded. The backlog suggests the most-recorded endurance
  sport the limits allow (else running or walking); an excluded item reads
  "Excluded by your limits" in Choose and on This week, whose Do it today
  is gone from the evening. Choose marks Garmin's session ◇ "Garmin ·
  read-only", "Review needed" under limits. The "?" and event chips have
  44 px targets.

- **Plan and Progress review fixes** (from the UX review in
  `.data/ux-review/plan-progress.md`). Commit "Offer review decisions only
  where they can apply": only the latest completed, undecided week's review
  offers Keep all and Accept (`WeekReview.decidable`); a change accepted after
  next week began applies from that day as a revision, keeping the week's
  starting target (`appliesFrom`, "From today, Thu 22 Oct"); kept proposals
  read "Kept (proposed 3)". Add a session, Move and Reschedule offer only this
  plan week's days, with the day on the button and an optional name.
  Commit "Make the goal rings fit and say why a week didn't count": every
  ring fits at any size (four- and five-goal weeks drew nothing inside 36 px),
  marks sit over faded rings in the weekly strip (`GoalWeek.current`, `mark`),
  this week's ring opens Plan, ring grids have a colour key, the week ring's
  centre is clear, day cells mark planned and not-recorded sessions
  (`DayCell.planned`) with the weekday above the date, the calendar falls back
  to activity days without goals and hides for the current week, and the
  Last week / Earlier weeks rows merge into the strip's "All weekly reviews".
  Commit "Lead the weekly review with the decision": the goals card comes
  first; readiness, Sleep Score, acute load and Fitness Age sit in one grid
  with averages, the change against the week before and seven labelled days
  instead of seven swipe cards; load and Fitness Age lose their invented
  gauges and score colours; the win needs a change of 1 or more (it said
  "Fitness age improved" beside "Steady"); the focus names Garmin's metric;
  "Keep the momentum" is gone.
  Commit "Make Progress readable and judge each week by its own goals":
  Cardio and Strength draw each week's own target as a stepped line
  (`DomainWeek.*Goal`; intensity following Garmin's goal now has its line),
  sleep against need reads the need Sleep reads (Garmin's seconds field too)
  and sits in the period report, Progress and reviews take the caller's clock
  so a stale Garmin Coach snapshot plans nothing on both Plan and Progress,
  one milestone row per set with its reps, dates carry the year when it isn't
  this one, totals read "19 h 54 min" under one "Compared with" line, the
  activity bars are an axis chart with blank days to come, the current week
  and month are outlined, counts get whole-number ticks and measured zeros a
  label; the causal copy, Latest readings tiles and the Strength bests and
  lift charts (Training · Strength has them) are cut.
  Commit "Gather what needs you on Plan": the week card keeps the rings, goal
  lines (once reached, only "reached"; "was 2" on a second line; "to review"
  counts recordings) and one Needs you place: review rows name the day and
  recording and open the session, not-recorded sessions are one row with
  Reschedule and Let it go; the event and Garmin Coach groups shrink to the
  event's line in Events and one Coach line, so Days is back above the fold.
  A recording that may be a session says which; unnamed sessions read "Run
  18:00"; Coach cardio has a neutral icon; a Coach rest day is marked and said
  beside your session; durations read "52 min"; removing an event asks
  first; Do it today leaves no crumbs; inputs fit at large text; explainer
  copy is cut.
  Merged with the chart legibility work: the stepped goal lines, outlined
  current bars and whole-number ticks keep its minimums (current bars keep
  date ticks; spoken labels say this week or so far), and
  `chart-legibility.mjs` now also measures Progress's Cardio, Strength and
  Long-term sections.

- **Training and workout review: major items** (commit "Make workouts and
  Training readable per sport and unit"). Heart rate is one chart (bpm axis,
  labelled zone floors, average, round minutes) at the top of Along the way
  or in its own card; pace is drawn faster-up; distance, pace, speed and
  elevation follow Garmin's measurement setting, swims read in the pool's
  unit with pace per 100 and Garmin's swim fields; rides split by speed;
  splits print their scale and only differing lap distances; zones show bpm
  ranges; load focus prints Garmin's target as a solid band; the acute bar is
  a toned dot on a neutral band with a round scale end; Training's load
  history is a compact 30-day chart opening a new acute-load metric detail;
  set bars show the weight change against last time; muscles are listed
  once on Strength with shading scaled to the range; the text-only "Lifts and
  exercises" card, the TE pills in Activities and the disclaimer lines are
  gone; race predictions say Half marathon and Marathon. Smaller fixes: the
  lane legend is a one-line key, the strength lane steps, Refresh from Garmin
  is a text button, the next session carries ◇ Garmin schedule · read-only,
  the Why sheet says "1.1× your 4-week load", Strength's ? opens the strength
  help sheet, and dropped leading spaces are restored. Synthetic pool-swim and
  walk fixtures were added to the preview, and Training's preview list now
  matches Strength's sessions in date order.

- **Training and workout review: bugs** (commit "Fix Training and workout
  review bugs"). From the read-only UX review of Training and workouts: the
  in-range headline says "In your optimal range"; week-lane letters follow
  the plan week's first day; "Sets checked" uses one local date and clock;
  Back keeps the chosen exercise, lift range, measure and one muscle range
  shared by Training and Strength; lift charts sit on a date axis (range
  start to today, year on the first month and January, dots thinned past 40
  sessions) and bodyweight exercises chart most reps instead of dead weight
  controls; Activities pages back through stored history by sport without a
  Garmin request, Today's link opens it narrowed to today and `#activities`
  aliases Training; unknown lanes are never 0; How it counted is left out
  without goals; the acute-load trend prefers Training Status so the chart
  and card agree; the Training eyebrow is the date and the 7-day fallback is
  titled as such.

- **Settings, Profile and Help review fixes** (commit "Make Settings say what
  it erases and what Android allows"). Settings reads Android's notification
  state each time it opens (new `notification_permission` command: Android's
  notifications switch and the runtime permission) and replaces the old
  Permission card with one blocked line that greys the switches; each switch
  says when it fires, with clearer names. Sign-out is "Sign out and erase this
  phone's data" with an in-place confirm naming goals, plans, limits and
  routines and offering Export first. Import previews carry years and the
  export's Raring version, Export and Import lead with one line, an empty
  Garmin change log and the duplicate history card are gone, "?" sheets link
  Help only where the area has pages, the heart-rate axis reads elapsed hours
  ("6 h"), gear shows the app's date format, and sign-in errors drop the
  "Error:" prefix.

- **Health review fixes: resting-HR band, search and a summary Health**
  (commit "Give Health its own resting-HR band and summary charts"). The
  resting-HR chart's band is Raring's usual range over Garmin's daily resting
  HR (`HealthView.restingHrRange`), never Health Status's Sleep HR baseline;
  a table test covers every trend metric's band with and without Health
  Status. Search normalises "spo2"/"vo2", adds synonyms and Health, Fitness
  Age, intensity, journal, body composition, import/export and tab entries,
  opens cards scrolled into view, and sends "resting heart rate" to Garmin's
  daily resting HR while Sleep HR appears under its own name. Health's charts
  are compact (own metric, 7D–1Y, tap to read, one link to the canonical
  detail); the heart-rate detail gained the resting-HR history. Fitness Age's
  history moved into its hero (sparkline and separate card removed, latest
  estimate selected), the biggest lever folded into its driver, drivers print
  Garmin's targets in words, the weight card summarises 30 days and opens the
  full detail, Add weight lives on Profile only, signed zeros read 0.0, range
  chips wrap at large text, and journal subtypes read as words.

- **Chart and label legibility** (commit "Make every chart's text legible at
  phone widths and large text"). Sleep · Tonight's dial is a timeline from an
  hour before wind-down to the usual wake (wind-down, Garmin's window or the
  asleep-by estimate, now, and the need ending at the usual wake), with the
  times only in the facts, which wrap at large text. Your day keeps scales
  and the nap change off the plot and draws heart rate against the day's own
  range (no clipping, review B9). The 7-night chart's values sit under each
  day, its scale is whole even hours with gridlines (B5). Consistency labels
  its pill, colours and keys the usual windows and marks Mondays (M14). The
  Card header wraps, the readiness tile stacks and gauges grow with the text
  (M7, M8). Every chart's ticks are at least 11 px and labels 12 px at 390 px,
  units sit outside the plots, and tick labels thin rather than collide
  (`declutter.ts`). `chart-legibility.mjs` (in `npm run test:ui`) measures
  31 pages at 360/390/720/1100 px and 1/1.3/2× text. The seven-night chart
  is a labelled group of per-night buttons, not an image, so each night is
  reachable by assistive tech. UI only; no core change.

- **Garmin's sleep schedule anchors Today's phases** (commit "Anchor Today on
  Garmin's sleep schedule until nights are recorded"). With fewer than four
  recorded wakes in 14 nights, the displayed day's scheduled wake (weekday
  window, else `DAILY`, else `userSleep`) replaces the fixed layout and Today
  names "Garmin sleep schedule"; Garmin's defaults never anchor. The weekly
  settings read now keeps the schedule, is also due daily while it's needed
  and once for a row stored before it was kept. Verified read-only on one
  account (shape only): `userSleep` is top level with `sleepTime`,
  `wakeTime` (integer seconds after local midnight, 0–86400),
  `defaultSleepTime`, `defaultWakeTime` (booleans); `userSleepWindows` is 8
  objects with `startSleepTimeSecondsFromMidnight`,
  `endSleepTimeSecondsFromMidnight` and `sleepWindowFrequency` (`DAILY`,
  `MONDAY`…`SUNDAY`). The Sleep header on Tonight names the evening and
  window instead of last night's times. Women's health is parked.

- **Today and Tonight at the day boundary** (commit "Keep the unsynced
  morning and Tonight's report on the right night"), from the Today and
  Sleep review (B2–B4, B6, B8, M13) and Plan review PP-B4. While last night
  hasn't synced the hero shows no earlier score, readiness, chips or
  reconciliation; Recorded today is the calendar day's, so yesterday's run
  is gone after midnight; last-night taps (tile, moon chip, Your day, naps,
  search, the widget's Sleep tile) open Last night; "I wound down" belongs to
  the night Tonight shows and opens two hours before its window; the need's
  read time carries its weekday unless today; beside tonight's plan card the
  hero keeps only its headline; the plan prompt is gated by Today's phase in
  the core (`plan_board`), so opening Plan at night hides it too.

- **Status surfaces follow Today's lead** (commit "Choose the status focus
  with Today's lead"). The persistent notification and widgets take their
  focus from the live Today's lead gates (`status::snapshot` runs
  `lead::apply`), keeping the private circumstance lines; the evening now
  reads "Make room for sleep" as Today does, a recorded workout reads
  "Activity recorded today", and the reason is Today's own line from
  Garmin's inputs instead of "biggest drag" (review M12). Today's waiting
  gate also covers the phase's neutral wait, so yesterday's readiness no
  longer suggests training between the usual wake and the handover (B1). No
  effort scale beside a recorded workout or from the evening on (M3).

- **Tonight's plan above Your day** (commit "Lead the night with tonight's
  plan, above Your day"). v0.1.33 computed wind-down correctly but always drew
  Your day with the hero, above every leading card, so at 23:21 the Wind down
  card sat under Your day and recorded today, beside Monday's check-in link.
  The core's card order now places Your day (`TodayCard::Day`): with the hero
  before the evening, after tonight's plan (compact) from then on. Wind-down
  and late night also hide the check-in link. Core and UI regressions cover a
  23:10–23:30 window with a 22:40 wind-down at 21:30, 22:50, 23:21 and 01:30,
  with and without a run.

- **Desktop reminders** (commit "Show desktop reminders while the app runs").
  A desktop sync used to mark every notification as sent without showing it.
  The desktop build now shows the same notifications Android posts through
  the Tauri notification plugin, with the same wording, settings,
  rest/illness/limits suppression and night identity. Only an accepted
  delivery is recorded; a refusal or failure leaves the reminder unmarked and
  Settings says why, with the next pending wind-down. Desktop has no
  WorkManager: while the app runs it re-reads the cached plan on its own wall
  clock, wakes at the pending wind-down and at least every five minutes, and
  checks again on resume, after a sync and after a notification setting
  changes, so a reminder whose window passed while the machine slept is
  dropped rather than fired late. On Linux the check refuses to mark
  anything when the session has no notification server, because the plugin
  discards that error; verified under Xvfb with no server: no notify call, no
  recorded reminder and "no desktop notification service" kept for Settings.
  Rust tests cover per-notification marking, wake targets and a night already
  sent; a real notification session remains to check (see follow-ups).

- **Opening an old day or workout** (commits "Fetch a past day or an older
  workout's detail only when it is opened" and "Open a past day or an older
  workout from the date that names it"). Past-night
  (`#sleep/YYYY-MM-DD`), past-day (`#health/YYYY-MM-DD`) and dated stress and
  heart-rate routes show that date on Sleep's Last night and on Health/Stress,
  opened from Progress's calendar, a selected trend point and the seven-night
  chart; Back returns to the page or section that opened each, and the plain
  `sleep`, `sleep/last`, `sleep/tonight`, `health` and `metric/stress` links
  keep their meaning. A route whose day or workout has never been read fetches
  only the missing rows its page shows once (a night's sleep and HRV; a
  Health day's stress, summary and Health Status; an old activity's
  summary/splits/zones/charts) through
  the existing on-demand shape: Android's `SyncGate` and token vault or
  desktop's `with_desktop_tokens`, a small budget, the shared per-endpoint
  retry wait and 429 backoff, never from navigation prefetch, and every
  answer kept, including empty ones. The page says while it loads and when
  Garmin has nothing for that day; today's data is never shown for a past
  date. Older workouts listed but not detailed by the history download now
  fill their route and streams when opened, and nothing else fetches them.
  The CLI's `day-detail` and `workout-detail` run the same read-only core
  path. Core mock-client tests cover fetch order, partial and empty answers,
  unchanged repeats, today/future/unknown keys and 429; browser tests cover
  the calendar, trend-point and seven-night entries, caller-aware Back, the
  dated stress and heart-rate pages, and the one-fetch loading and unavailable
  states. Emulator-5560 (shared; signer-verified in-place debug install,
  SHA-256 `0d59d1bf…7a63`) showed dated Sleep/Health/stress pages from a
  calendar day and a seven-night bar; the synthetic demo has no Garmin
  credentials, so `fetchDetail` refused it locally without a request or a
  sign-out. The credentialed JNI read is unverified on a device.

- **21. Write rules** (commit "Add the Garmin write rules and a log of
  recent changes"). One core write path (`crates/core/src/writes.rs`): only
  from an explicit action (only `writes` can create the proof
  `Client::put_json` needs, so sync can't write), deletes need a confirm
  flag, a fresh read, a typed patch checked against the value the person saw
  (a change in Connect to the same field is a conflict, other changes are
  kept), only the touched top-level keys sent, the shared session, pacing,
  401 refresh and 429 backoff, then a read-back of everything sent. Outcomes:
  applied, read back differently, unconfirmed, conflict, rejected with
  Garmin's message, failed, needs sign-in. A versioned, bounded (50) local
  log, newest first, with "waiting for watch sync" from the watch's
  `lastDownloadTimestamp`; exported, erased by sign-out and account switch,
  and skipped by settings import (named in its preview) as account history.
  Settings lists it (read-only). Proven with watch alarms against a local
  mock only: concurrent change, read-back mismatch, 400 with message,
  401/refresh, 429 and a confirmed delete. No Garmin request was made; no
  app feature writes yet.

- **Import local settings on a new phone** (commit "Import a validated Raring
  export's settings on a new phone"). Settings reads a Raring ZIP chosen with
  the Android system file picker or a desktop file dialog, validates the
  archive format version, the account in the manifest, each row's `schema`,
  each record's shape and the absence of credentials, then shows counts per
  record kind (added, replaced, kept), the covered date range and what is
  skipped and why before the person confirms. Applying is one transaction
  under the account/write barrier; goals merge by effective date and later
  save, sessions, links and decisions take the later timestamp, and rest days,
  pause closures, limits, prompts, events, routines, plan weeks and saved
  nights only fill what the phone lacks. Garmin responses, journal logs and
  milestones are not imported; the UI says so. Re-importing a file changes
  nothing. The manifest now names the archive format version and the account
  that made the export; files from another account are refused whole. Checked
  on the emulator with the synthetic demo account: the picker listed the
  synthetic ZIP, the preview reported two rows, the import wrote them, the
  cache copy was removed, and the demo state was restored afterwards.

- **8a strength introduction and figure choice** (commit "Replace the strength
  intro with help at the point of use"). The mandatory-seeming `intro-1` to
  `intro-4` walkthrough is gone from the mockup, its gallery entries, notes and
  screenshots; Strength's ? and empty states carry the help instead, as item
  26's progressive disclosure already did. The body-map figure is now a
  visible Profile / Female / Male control on Strength's muscles card, saved as
  the `bodyFigure` setting (Profile by default, following Garmin's profile
  gender) and applied to every body map; core tests cover the override and the
  three-way setting, and the browser UI test covers the picker. Settings export
  includes it with the other preferences.

- **8a explicit strength refresh** (commit "Refresh a strength workout's sets
  only when asked"). A workout's detail now says when Garmin's sets were last
  read and offers **Refresh from Garmin**: one explicit
  `/activity-service/activity/{id}/exerciseSets` request, through the same
  serialized token path as the on-demand heart-rate read (Android's existing
  `SyncGate` and token vault, desktop's `with_desktop_tokens`) and the shared
  request budget, retry wait and 429 backoff. It never runs from navigation or
  an ordinary sync, keeps the previous sets when the read fails, and reports
  Garmin's rate-limit message. Core tests use the local mock client only: a
  strength activity spends one request and updates the detail, a non-strength
  activity spends none, an active backoff blocks it. The Kotlin plugin and JNI
  bridge mirror the existing heart-rate on-demand command. Unverified on a
  real account and device.

- **8a bodyweight bests** (commit "Give bodyweight sets a most-reps best").
  Bodyweight sets (Garmin weight 0 or null) earn no weighted record or best;
  each named exercise's best is now its most reps in one counted set in
  available history, with the same exclusions, earliest-on-a-tie rule and
  correction/no-repeat-award behaviour as weighted bests. Strength's exercise
  detail shows it as "Most reps" (or "Most reps in a set" when the exercise
  has no weighted sets) and Progress records it as "… most reps in a set"
  with a `reps` milestone kind. Core tests cover flags, unnamed and weighted
  sets, ties and a Garmin edit; the browser preview and UI test include a
  bodyweight exercise.

- **7a shared date labels** (commit "Build shared date labels by hand so
  Android and desktop match"). `format.ts` builds the weekday, day and month
  from a calendar date instead of `toLocaleDateString`, including en-GB's
  "Sept", and `plan.ts` reuses it, so the "Mon, 5 Oct" WebView difference is
  gone everywhere the app labels a date. Unit tests cover the labels, the
  short month and invalid dates.

- **26. A focused, adaptive Raring experience** (commits "Rebuild Raring
  around the weekly plan: Today, Plan, Training, Health, Progress", "Time
  three years of synthetic history and fix the activity lookup that made
  Progress take seconds", and the cutover "Remove the redesign prototype
  now that the app ships it"). Five tabs (Today · Plan · Training · Health ·
  Progress), Profile and Adapt on every root page, short "?" help; local goals,
  sessions, links, circumstances, routines, reviews, events and milestones with
  migration from the earlier weekly plan; matching, ledger, rings, backlog and
  week review with proposals; Today's lead with Keep · Choose another and Your
  day; Sleep's Last night and Tonight; week lanes, lift volume, muscle gaps,
  workout streams and plain-language training effect; Health's long-term
  order and freshness; Progress by domain with a ring calendar and milestone
  archive; adaptive Garmin Coach sessions as read-only planned work. The design
  was approved on 2026-10-19 from a synthetic prototype; every U01–U64 journey
  was compared with it (39 match, 19 gaps fixed, 5 differ by approved
  decisions, 1 needs device evidence; table in the design). Released as
  v0.1.28, with the Progress fix in v0.1.29; **Dan installed the release on
  his phone with his real data and approved it**. The prototype was then
  removed; its captures stay in `design/mockup/experience/`. Verified
  read-only: the adaptive Coach shape (15a) and these stored activity facts.
  **Verified stored activity `details` facts (2026-10-19, read-only from the
  maintainer's store; structure only).** Each metric descriptor carries
  `unit {key, factor}`. Unit keys seen: `directSpeed`,
  `directGradeAdjustedSpeed`, `directVerticalSpeed` mps; `directElevation`
  meter; `directDoubleCadence`/`directRunCadence`/`directFractionalCadence`
  stepsPerMinute; `directBikeCadence` rpm; `directPower` watt;
  `directRespirationRate` breathsPerMinute; `directAvailableStamina`,
  `directPotentialStamina`, `directPerformanceCondition`, `directBodyBattery`
  dimensionless; `directStrideLength`, `directVerticalOscillation` centimeter;
  `directGroundContactTime` ms; `directAirTemperature` "celcius" (sic); sums
  second/meter. The **factor is not a scale to apply**: stream values are
  already in the base unit (distance, duration and elevation sums agree with
  the summary fields). `directAirTemperature` appears only on some activities,
  with `summaryDTO` average/min/max temperature on the same ones; it is the
  watch's own sensor. The core uses a stream only when its descriptor's unit
  key is the expected one and never applies the factor.
  Open evidence is tracked in 26a.

- **18. Naps and journal, read-only** (commit "Show Garmin naps and consented journal").
  Sleep shows Garmin's separately recorded naps without changing main-sleep
  totals; Health shows a seven-day lifestyle log only after local consent and
  an explicit open action. GET `/sleep-service/sleep/naps/{d}` returned an
  array (both empty and populated responses were seen): populated rows have
  `napStartTimestampGMT`/`napEndTimestampGMT` (UTC wall strings),
  `napStartTimeOffset`/`napEndTimeOffset` (seconds), `napTimeSec` (seconds),
  `napFeedback` and `napSource`. GET
  `/lifestylelogging-service/dailyLog/{d}` returned `calendarDate`,
  `completionStats` and null `dailyLogsReport` in checked responses; null is
  unavailable, not an empty journal. Community examples include report rows
  with `name`, `logStatus` YES/NO and optional `details` with `amount` and
  `subTypeName`; populated rows on this account are still unverified. Reads
  use the shared budget, pacing and 429 backoff: recent naps and, after
  consent only, seven journal days at six-hour/twenty-four-hour cadences,
  with cached empty and unavailable responses. Consent is checked before
  each journal request, after client mutex/refresh/pacing and before a 401
  retry, and atomically with caching each reply across database connections;
  opt-out discards in-flight results without
  clearing previously downloaded logs. Live Health journals follow the actual
  local day across the sleep-day handover, while historical views keep the
  requested date. Nap cards show dated endpoints across midnight and label
  missing endpoints. Exports still include saved logs after opt-out until
  local data is erased. Synthetic delayed-response, date-boundary and browser
  consent/retention cases cover these constraints. No Garmin writes.
- **17. Thresholds and acclimation** (commit "Show Garmin thresholds in Training").
  Training shows only recorded Garmin running lactate-threshold HR and pace,
  running threshold power, cycling FTP, heat percentage and acclimated
  altitude, each dated separately. Four latest-only reads are checked daily
  within the sync budget, including empty and 404 results; a 404 or failed
  check retains an earlier valid reading, with bounded retry. Any recorded
  threshold/acclimation response enables the existing weekly reduced units
  fetch; the exercise catalog still requires strength sets. An endpoint's
  failure or retry window does not block the other metrics; global auth,
  rate-limit and budget stops still apply. No historic day expansion,
  threshold estimates or writes.
  Verified with four read-only requests on one account (no 429/auth errors):
  - `GET /biometric-service/biometric/latestLactateThreshold` (no arguments)
    returns an array of separately dated `speed`, `hearRate` (Garmin's
    spelling; `heartRate` also accepted) and possibly `heartRateCycling` rows.
    `calendarDate` can be a local ISO datetime; `speed` is seconds per meter.
  - `GET /biometric-service/biometric/powerToWeight/latest/{YYYY-MM-DD}?sport=Running`
    returns an array with `calendarDate`, `sport`, nullable
    `functionalThresholdPower` (watts) and `isStale`.
  - `GET /biometric-service/biometric/latestFunctionalThresholdPower/CYCLING`
    (no arguments) returns an object or array with `calendarDate`, `sport`,
    nullable `functionalThresholdPower` (watts) and `isStale`.
  - `GET /metrics-service/metrics/heataltitudeacclimation/latest/{YYYY-MM-DD}`
    returns an object with nullable `heatAcclimationPercentage`,
    `heatAcclimationDate`, `altitudeAcclimation` (meters),
    `altitudeAcclimationDate`, `acclimationPercentage`, `currentAltitude`
    and trend/previous fields. An explicit 0% is a recorded value.
  Other watches, absent endpoint responses, null calibration on a live watch,
  and Garmin Connect/watch display agreement remain unverified.

- **16. Read-only gear** (commit "Show Garmin gear distance on Training").
  Training has an expandable gear card showing shoes and bikes, including
  retired items, Garmin's accumulated distance and a Garmin retirement
  threshold only when supplied. It labels the last successful list fetch;
  missing distance and zero/missing thresholds stay neutral. No gear writes,
  activity assignment or notifications. The complete list is checked at most
  daily within the existing sync budget and 429 backoff, including empty/404
  answers; failures preserve the last good list. Verified by three read-only
  account requests (no 429 or auth failure): `GET /gear-service/gear/v2/list`
  returns an array without parameters; `userProfilePk` also returned the same
  list, as did the community client's `/gear-service/gear/filterGear` with
  `userProfilePk`. Each v2 row has `uuid`, `gearType`, `status`, `name`, `brand`,
  `usageType`, `distanceUsedMeters`, `maxUsageDistanceMeters`, `firstUseDate`,
  `numActivitiesLinked`, `durationUsedSeconds`, `daysUsed`, `processing`,
  `createdDate` and `userProfilePk`. Distance and threshold are metres;
  the community client's gear creation converts km to metres. Retired state,
  shoes, empty lists, absent distance and absent thresholds were not established
  by these reads; synthetic tests cover them.
  - Follow-up: compare the card's units, status and threshold to Garmin
    Connect on a phone, including a retired item and an account without gear.
  - Optional threshold notification: if added later, opt-in consent, Garmin's
    own positive threshold and per-gear/per-threshold deduplication are
    required; do not change the existing notification cadence.

- **15. Planned training, read-only** (commit "Show planned training and upcoming races").
  Today and Training show Garmin's next scheduled session and Coach context;
  Training shows the next race date/countdown. Today hides the session card
  during wind-down and late night. Verified read-only against one account:
  `GET /trainingplan-service/trainingplan/active-plans?startDate=YYYY-MM-DD`
  returned `searchFilter` and `trainingPlanList` (array);
  `GET /workout-service/schedule/workoutScheduleByPeriod?startDate=YYYY-MM-DD&endDate=YYYY-MM-DD`
  and `GET /calendar-service/events/upcoming?numDaysForward=90&limit=10`
  returned arrays. The community client specifies
  `GET /trainingplan-service/trainingplan/tasks/{planId}?localeKey=en`;
  tasks were not verified because no valid plan ID was available and are now
  deferred until their display value is verified. Populated workout/race
  fields, completion/cancellation flags and device coverage remain unverified.
  Schedule/race days are local calendar dates, countdowns are whole days;
  no guessed UTC times or device-support conclusions. Live planned cards use
  the actual calendar day even when Today shows yesterday's sleep; explicit
  historical views retain their selected date. Cached schedule bounds appear
  on empty/stale cards, and a full 10-event response is incomplete race
  coverage. Reads use six-hour cadence (including 404/empty), shared
  budget/backoff and 12-hour freshness. Synthetic empty, changed, stale,
  all-day and midnight cases plus emulator UI passed.
- **14. All-day heart rate** (commit "Show all-day heart rate and daily resting HR").
  Health and Today's Right now link to the day chart. It marks recorded sleep
  and activities, preserving off-wrist and absent samples as gaps. The chart
  and daily resting HR use Garmin's daily endpoint; Sleep HR remains the
  separate overnight Health Status mean. Today and yesterday refresh on each
  sync within the budget; older days load once when opened and stay cached,
  including absent answers. On-demand reads use the shared retry and 429
  backoff. The chart includes the following morning's recorded sleep for the
  preceding evening, clips both sleep intervals and activities to the displayed
  day, and labels its 23-/25-hour axis as elapsed time rather than an invented
  wall clock. On desktop, one credential gate serializes sync, detail fetch,
  sign-in and sign-out around rotating-token writes. Verified with three
  read-only real-account reads and a budgeted sync, without 429 or auth failure:
  - `GET /wellness-service/wellness/dailyHeartRate/{displayName}?date=YYYY-MM-DD`
    returns an object with `calendarDate`, `startTimestampGMT` and
    `startTimestampLocal` (local midnight), `endTimestampGMT` and
    `endTimestampLocal`, `heartRateValueDescriptors` indexing `timestamp`
    and `heartrate`, `heartRateValues` pairs of Unix milliseconds and beats
    per minute, `restingHeartRate`, `lastSevenDaysAvgRestingHeartRate`,
    `minHeartRate` and `maxHeartRate` (bpm). A profile identifier may also
    appear and is never copied into fixtures.
  - Samples are normally two minutes apart; null readings and omitted
    timestamps occur in gaps, sometimes much longer than a slot. Neither
    is zero. Today's GMT end may precede the end of the day even while
    `endTimestampLocal` names the next local midnight; don't infer freshness
    from the local end field.

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

- **9–10. Persistent status and home-screen widgets** (commit "Finish shared
  status notification and widgets"). Visual-first mockups in `design/mockup/`
  (`status-surfaces`, `widget`). One private version-1 Rust JSON snapshot
  feeds the silent, off-by-default notification and small/medium RemoteViews
  widgets. Shared `TodayFocus`, lowest-factor rule and watch upload time;
  latest stress is a sample, not the daily average; unknown values stay blank.
  Settings offers the notification once after first sync and handles permission
  denial. Foreground and background sync share the writer; widget/lifecycle
  refreshes are offline, and no rendering path starts the WebView.
  Existing hourly/morning sync remains unchanged. No new Garmin endpoints;
  real read-only validation matched snapshot readiness, sleep, phase and
  selected-watch upload to Today without rate-limit/auth failures. HR uses the
  shared calendar-day timeline's last measured sample, without notification-only
  requests. Device/cadence limitations are under 9–10a above. Account-bound
  snapshots require fresh identity verification before vault promotion; same-account
  reauthentication preserves local targets, failed verification suspends rather
  than destroys history, and a verified switch erases account-bound data before
  exposing the new session. Explicit opt-out survives cache publication failure
  and cancels independently of an in-flight sync. Widget minima and large-font
  tests cover real RemoteViews bounds rather than showcase screenshot sizes.
  A short Rust barrier prevents view/plan/settings writes and snapshots from
  crossing an account transition; no network wait holds it. Native tests inject
  SQLite suspension and vault deletion failures, verify fail-closed status,
  bedtime and app authorization, and preserve targets for later same-account
  reauthentication. A platform suspension tombstone prevents retained credentials
  from being automatically promoted after a failed clear. Queued mutations
  validate their expected identity inside the barrier before writing; inactive
  or changed sessions receive no account history metadata from app status.
  An actual emulator IPC fault probe confirmed hidden status and denied views
  after failed SQLite suspension and after restarting with that fault still
  present; lifecycle scheduling reports the failure rather than crashing.
  Exports omit activation/pending markers, bind their SQLite snapshot to the
  expected account and check identity plus publication under one barrier.
  Older ticket exchanges cannot replace a newer sign-in. UI settings are
  validated field patches: unrelated delayed saves cannot reinstate status or
  journal consent, and journal opt-out survives a failed native snapshot write.
  Journal values are excluded from all status-surface allowlists. Final checks:
  352 Rust tests, 57 UI unit tests, browser regression/build, Clippy, secret scan,
  Android APK, 14 native unit tests and 14 emulator instrumentation tests passed.
  Physical-phone, launcher and Doze checks remain open above.

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
