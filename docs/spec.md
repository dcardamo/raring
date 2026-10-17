# Raring v1 spec

Raring turns Garmin data into a clear daily answer: how am I doing,
why, and what should I do today, readable at a glance from dials, charts and
color. It shows Garmin's own metrics and nudges you to improve. No data dumps,
no invented scores.

Android first. iOS and desktop come next, then web.

The visual reference is [design/mockup](../design/mockup/): open
`index.html` in a browser (narrow windows behave like the phone app), or see
`screens.png`. It uses sample data.

## Principles

- **Visual first.** Graphics carry the meaning: dials, gauges, bars, charts
  and color. Text is a headline of a few words, labels and numbers. Anything
  longer sits behind a tap.
- **Garmin's numbers.** Every metric is Garmin's. Raring's own math is
  limited to your normal ranges, week-over-week changes, and the advice rules
  below.
- **Fast.** Opens instantly from local data and syncs in the background.
- **Private.** Data stays on the device. No accounts, analytics, or services
  other than Garmin.
- **Cross-platform.** One Rust core and one Svelte UI for Android, iOS and
  desktop. Native code is limited to what a platform requires, such as
  background scheduling; everything else lives in the shared code.
- **Its own identity.** Own name, palette, icons and copy. Garmin's metric
  names describe Garmin data. Nothing copied from other fitness apps.

## Look

- Dark ink background, with a faint glow behind a screen's main gauge in that
  gauge's status color.
- Color means something or it isn't used. Icons and titles say where you
  are; color says how it's going:
  - **Status**: green good, amber fair, red poor. Missing data is a gap,
    never a zero.
  - **Effort**: heart-rate zones run gray, blue, yellow, orange, red. Red
    means hard.
  - **Amount**: one blue, faint to strong, for counts without a judgement,
    such as sets per muscle.
  - **Ink**: everything else, including charts without a verdict, selected
    controls and primary buttons (white on ink).
- Text is at least 4.5:1 against its surface (3:1 for large text). Text on a
  colored fill is black or white, whichever reads better.
- Gauges are solid arcs open at the bottom, colored by status. Load gauges
  mark Garmin's optimal range as a neutral band, with a dot for the current
  value.
- Rounded cards, Outfit for numbers and headings, Inter for text.

## Screens

Bottom tabs: **Today · Sleep · Training · Health**. Tapping a dial or card
opens its detail.

### Welcome & privacy

Before sign-in, explain that Raring reads a Garmin account, not a watch, and
that Garmin Connect must keep syncing the watch's data. State that the app
sends no data to Raring, has no analytics or telemetry, and keeps downloaded
history on the phone. Explain the unofficial connection and watch-dependent
metrics before the sign-in button. Keep the same information and expandable
privacy details available offline in Settings. Notification permission remains
optional after sign-in.

### Today

- Three gauges: **Readiness** (Training Readiness, 0–100) large in the
  middle, **Sleep** (Sleep Score) on the left, **Load** (7-day acute load
  against Garmin's optimal range) on the right.
- A few-word headline ("Ready to push", "Take it easy") and three reason
  chips: sleep time, HRV and load.
- **Today's focus**: Rest, Easy, Steady or Push, with the suggestion lit up,
  followed by the lowest recorded Garmin readiness factor. Tap to see all six
  factors and their feedback.
- **Tonight's sleep**: one next step: the wind-down time, Garmin's bedtime
  window and sleep need for tonight, and one line on why the need is what it
  is, from Garmin's feedback. Without Garmin's window it shows Raring's
  asleep-by estimate instead, labeled as Raring's, with the usual wake time
  and the number of recorded wake times; a historical-average need is
  explicitly labeled. The same plan appears on Sleep and drives the bedtime
  reminder. Hide the plan if the need is unavailable, or if both the window
  and wake history are; when no recorded sleep has synced for the current
  day, say the estimate uses older nights.
- Body Battery as a battery with its overnight charge; HRV with a 7-day line
  over Garmin's baseline band.
- **Stress**: Garmin's average so far today, its category and day word, and
  the day in 15-minute bars. Tap for the full day of three-minute readings
  with sleep and activity marked, Garmin's time in each category, and 7-/30-day
  daily averages. Times Garmin cannot rate (moving, off-wrist) are gaps, never
  zero.
- Vitals as five compact readings with units, a range-status indicator and
  a count of recorded and comparable readings. Missing or uncalibrated data
  is neutral, never an all-clear.
- Today's workouts with a heart-rate zone strip, and Fitness Age with its
  trend line. The weekly check-in card appears here on Mondays.

### Readiness

- Large gauge with Garmin's level (Prime/High green, Moderate amber, Low/Poor
  red) and a few-word headline from Garmin's own feedback.
- Factor bars worst first; tapping one explains it in a sentence.
- Interactive 7-/30-day trend chart and today's call. Tap the chart or use
  the date slider to inspect a reading. Dates are anchored to the stored
  Garmin day, missing days remain gaps, and readiness uses Garmin's level
  boundaries (50 for Moderate, 75 for High).

### Sleep

- Score gauge, and a time-asleep bar with a marker at Garmin's sleep need
  for that night (when the watch reports it), labeled as last night's need.
- Stage timeline (awake, REM, light, deep) and a stage bar with each stage's
  change against your usual.
- Overnight tiles with lines: HRV, heart rate, respiration, SpO2.
- **Consistency**: 14-night chart with your usual bed and wake windows
  shaded, and Garmin's seven-day average bedtime and wake time above it when
  synced. No rhythm score or chronotype of Raring's own.
- **Sleep alignment**: Garmin's status (Aligned, Ahead, Behind, Jet lag) and
  last night's sleep against Garmin's optimal sleep window on an overnight
  timeline, with the last seven nights below. Garmin's window is a dashed
  outline, never shaded like your usual windows, and is labeled as the
  longer-term rhythm, separate from tonight's bedtime window. Without a
  rating for last night, show the latest one with its date; say when Garmin
  is still calibrating, and otherwise that alignment is unavailable.
- Tonight's sleep plan, shared with Today. "Why this plan" lists the
  evidence: who set the bedtime, Garmin's need, baseline and adjustments, the
  wind-down lead, and one tip toward steadier timing. It is a suggestion, not
  an alarm, and never changes the watch's sleep schedule.

### Training

- Large semicircle acute-load gauge with Garmin's optimal band, training
  status chip (Productive, Maintaining, …), load ratio and recovery time.
- Daily load bars for the week.
- Load focus bars (low aerobic, high aerobic, anaerobic) against Garmin's
  target ranges, with a shortage flag.
- VO2max with its trend, race predictions, endurance and hill scores, running
  tolerance.
- Workout list with zone strips and load.

### Workout

- Training load, aerobic and anaerobic Training Effect gauges, primary
  benefit and recovery time.
- Stats grid, route map, heart-rate chart colored by zone, time-in-zone bars,
  and split bars with the fastest highlighted. Power, cadence and running
  dynamics when present.

### Health

- **Vitals first**: resting heart rate, HRV, respiration, SpO2 and
  skin-temperature change as tappable tiles. Show units, the personal range
  and explicit within/above/below-range or missing/baseline-building labels.
  Each detail has interactive 7-/30-night history with the current range
  shaded, actual dates and missing nights left blank. Nightly HRV against the
  baseline is distinct from Garmin's seven-day HRV status. Explain the comparison
  method behind a tap; personal ranges are not medical targets.
- **Stress**: today's average with a 30-day line of daily averages, opening
  the same detail as Today.
- **Fitness Age**: large number, trend line, and a bar placing it against your
  real age.
- **Biggest lever**: a progress ring toward Garmin's target for the driver
  that would help most (for example vigorous minutes), with the age you'd
  reach.
- **Drivers**: VO2max, resting heart rate, BMI or body fat, and vigorous
  minutes as tiles with range bars.
- When two or more vitals are outside their personal ranges, an amber
  message suggests taking it easy and checking how you feel, without
  diagnosing an illness. A partially calibrated monitor reports only the
  readings it can compare.

### Weekly check-in

Opened from the Monday notification: a short stack of swipeable cards for the
week just ended. Readiness, sleep, training and health each get a gauge or
chart with an arrow against the week before. VO2max and Fitness Age changes
are called out. A timing card shows the week's bed and wake times, their
earliest and latest, how many nights Garmin rated aligned, ahead or behind,
and one timing focus. It is descriptive: no adherence score, streaks or
claims that timing changed a score. It ends with one win and one focus for the
coming week, a line each. Past check-ins stay in a list.

## Notifications

Sent by the phone itself.

| Notification                  | When                                         | Default |
| ----------------------------- | -------------------------------------------- | ------- |
| Morning readiness             | Last night's sleep and readiness have synced | On      |
| Weekly check-in               | Monday 08:00                                 | On      |
| VO2max or Fitness Age changed | Either direction                             | On      |
| Vitals off                    | Two or more vitals outside normal range      | On      |
| Training status changed       | On sync                                      | Off     |
| Bedtime                       | Wind-down time of tonight's plan (lead 30 min, configurable) | Off     |

## Rules

- **Suggested effort**: Garmin's readiness feedback, rephrased. The level sets
  push, steady or easy; a high load ratio caps it at steady.
- **Biggest drag**: the readiness factor Garmin rates worst.
- **Bedtime**: Garmin's recommended bedtime window for tonight when it sends
  one; the asleep-by time is its end. Otherwise usual wake time (14-day
  median) minus tonight's sleep need (Garmin's next-night adjusted need when
  present, otherwise its latest adjusted need or the 14-day average). Label
  the source and wake-history count. Garmin's optimal sleep window (Sleep
  Alignment) is never used as tonight's bedtime.
- **Advice precedence and labels**: bedtime from Garmin's window ("Garmin
  plan"), else Raring's estimate ("Raring estimate"). The reason line comes
  from Garmin's need, baseline and adjustment codes. The timing tip uses
  Garmin's alignment status for last night when present (behind: keep a steady
  wake time rather than forcing one early night), otherwise the usual wake
  time; it never just says to go to bed earlier.
- **Wind-down**: the window's start (or Raring's asleep-by time) minus the
  lead from Settings. The reminder fires once per night, keyed by the morning
  the night ends, between the wind-down and asleep-by times; it is skipped
  when no recorded sleep (a duration, start or end) has synced for the
  current day. Times are local wall-clock times, so the platform reschedules
  when the timezone changes.
- **Timing review**: the week's earliest and latest bed and wake times, the
  wake-time spread (latest minus earliest) against the week before, and
  counts of Garmin's alignment ratings. The focus follows the majority rating
  when at least three nights are rated, otherwise whether wake times varied
  by an hour or more, and names the week's median wake time.
- **Normal range**: the middle 80% of the last 30 nights, once there are at
  least 14. HRV uses Garmin's own baseline instead.
- **Stress bars**: the Today card averages each 15 minutes of Garmin's
  readings; the detail shows every reading. Colors follow Garmin's categories.
- **Body Battery color**: good at 70 and above, fair from 30, poor below 30.
  Garmin rates no level, so only the display color is bucketed.
- **Weekly focus**: the biggest gap among sleep against need, load-focus
  shortage, falling readiness, and vigorous minutes against the Fitness Age
  target.

## Garmin access

- Raring uses Garmin Connect's unofficial API, the one Garmin's own apps
  use. Garmin has not sanctioned it and can change it, so Raring stays out
  of the app stores. Releases are APKs on GitHub, installable with Obtainium.
- **Sign-in** uses Garmin's own sign-in page inside the app, MFA included.
  The app catches the one-time service ticket Garmin redirects back with and
  exchanges it at `diauth.garmin.com` for access and refresh tokens, as
  [python-garminconnect](https://github.com/cyberjunky/python-garminconnect)
  does. Tokens are encrypted with an Android Keystore key and refresh
  themselves.
- **Data** comes from `connectapi.garmin.com`: daily summary, sleep
  (including Garmin's sleep need, bedtime window and Sleep Alignment), seven-day
  sleep stats (Garmin's average bed and wake times), HRV,
  training readiness, Body Battery, stress, SpO2, respiration, training status, max
  metrics, Fitness Age, endurance and hill scores, race predictions, running
  tolerance, personal records, and activities (summary, splits, heart-rate
  zones, charts and GPS routes).
- Each sync fetches the last 90 days day by day, then spends its remaining
  request budget downloading older history in range requests, newest first
  and resuming where the last run stopped, back to the account's oldest
  data. A new phone downloads it all again; there is no Raring cloud. Older
  days keep daily values; Settings shows how far back the history goes.
- Garmin Connect stays installed on the phone because it uploads the watch's
  data.
- python-garminconnect is the reference when Garmin changes sign-in or
  endpoints.

## Architecture

- `crates/core` (Rust): Garmin client, SQLite storage (raw responses plus
  typed tables), sync, rules and notification decisions.
- `app/` (Tauri 2, Svelte and TypeScript): the UI, sign-in and settings. The
  UI reaches the core through one small typed API, so a web build can swap in
  HTTP later.
- **Background sync**: a small Kotlin plugin schedules WorkManager jobs,
  hourly, and every 15 minutes in the morning until readiness arrives. A job
  calls the Rust core directly over JNI, without starting the UI, then posts
  any notifications.
- **Bedtime reminder**: the core returns tonight's wind-down time as a local
  wall-clock time; Android sets an alarm with a ten-minute window for it after
  every sync, on app start, on settings changes, after a reboot or update and
  when the timezone or clock changes, all from the cached plan without the
  network, and asks the core again when it fires. One lock covers checking,
  posting and recording the reminder. Android only for now: the desktop app
  has no notification delivery, so it shows the plan but sends no reminder.
- **Platform code**: keep each platform's native layer a thin shim over the
  core. Background scheduling is inherently native (WorkManager on Android,
  background refresh on iOS, syncing while the app runs on desktop); secure
  token storage and other logic belong in Rust where a cross-platform crate
  exists.
- **Public website**: `site/` is a static introduction, installation guide and
  privacy page, hosted at raring.app with GitHub Pages. It has no tracking,
  sign-in or access to Garmin data; it is not the future web version of the app.
- **Later**: `crates/server` (axum) runs the same core and serves the same UI
  as a web app on a home server. It would sign in once from a CLI.

## Milestones

0. **Spike (go/no-go gate)**: a bare app on a real phone that signs in to
   Garmin, syncs readiness with the app closed, and posts a notification. If
   any of the three fails, stop and report.
1. Core: Garmin client, storage, sync and backfill, plus a dev CLI that runs
   against real data.
2. Today, Readiness and Sleep, then a screenshot review.
3. Training and Workout, then a screenshot review.
4. Health, notifications and the weekly check-in, then a screenshot review.
5. Polish, first APK release, and the repo goes public after a final review.

After v1: iOS, desktop apps, web app, a home-screen widget, and a journal (Garmin's
lifestyle logging is readable through the same API).

## Testing

- Core: unit tests on recorded, scrubbed Garmin responses, and table tests for
  every rule.
- UI: Playwright screenshots of each screen on fixture data. These are the
  review screenshots.
- Android: an emulator install-and-render smoke test. The spike proves
  sign-in and background sync on a real phone.
- A test fails if fixtures contain email addresses, Garmin IDs or tokens. Real
  data and tokens are gitignored.

## Manual steps

- One terminal sign-in on the development machine for a token, so the app
  can be built against real data that never leaves that machine.
- One sign-in on the phone for the spike.
- Four screenshot reviews.
