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

- Dark ink background with a soft glow tinted by the screen's main state, for
  example green when readiness is high.
- One color per area: readiness mint to cyan (amber and coral at lower
  levels), sleep violet, load and training orange. Heart-rate zones run gray,
  blue, green, amber, red.
- Gauges are arcs open at the bottom, with a soft halo. Load gauges mark
  Garmin's optimal range as a band, with a dot for the current value.
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
- **Tonight's sleep**: suggested asleep-by time, Garmin sleep need and usual
  wake time. The same plan appears on Sleep; a historical-average need is
  explicitly labeled, along with the number of recorded wake times. Hide the plan if need or wake history is unavailable.
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
  (when the watch reports it).
- Stage timeline (awake, REM, light, deep) and a stage bar with each stage's
  change against your usual.
- Overnight tiles with lines: HRV, heart rate, respiration, SpO2.
- 14-night consistency chart with your bed and wake windows shaded.
- Tonight's sleep plan, shared with Today: asleep-by time, sleep need and
  usual wake time. It is a suggestion, not an alarm.

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
are called out. It ends with one win and one focus for the coming week, a line
each. Past check-ins stay in a list.

## Notifications

Sent by the phone itself.

| Notification                  | When                                         | Default |
| ----------------------------- | -------------------------------------------- | ------- |
| Morning readiness             | Last night's sleep and readiness have synced | On      |
| Weekly check-in               | Monday 08:00                                 | On      |
| VO2max or Fitness Age changed | Either direction                             | On      |
| Vitals off                    | Two or more vitals outside normal range      | On      |
| Training status changed       | On sync                                      | Off     |
| Bedtime                       | 30 minutes before the suggested bedtime      | Off     |

## Rules

- **Suggested effort**: Garmin's readiness feedback, rephrased. The level sets
  push, steady or easy; a high load ratio caps it at steady.
- **Biggest drag**: the readiness factor Garmin rates worst.
- **Bedtime**: usual wake time (14-day median) minus tonight's sleep need
  (Garmin's next-night adjusted need when present, otherwise its latest
  adjusted need or the 14-day average). Label the source and wake-history count.
- **Normal range**: the middle 80% of the last 30 nights, once there are at
  least 14. HRV uses Garmin's own baseline instead.
- **Stress bars**: the Today card averages each 15 minutes of Garmin's
  readings; the detail shows every reading. Colors follow Garmin's categories.
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
- **Data** comes from `connectapi.garmin.com`: daily summary, sleep, HRV,
  training readiness, Body Battery, stress, SpO2, respiration, training status, max
  metrics, Fitness Age, endurance and hill scores, race predictions, running
  tolerance, personal records, and activities (summary, splits, heart-rate
  zones, charts and GPS routes).
- The first sync requests up to 90 days of daily history. Saved history grows
  as new days sync; older history beyond the initial window is not backfilled.
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
