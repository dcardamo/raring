# Pepometer v1 spec

Pepometer turns Garmin data into a Whoop-style daily answer: how am I doing,
why, and what should I do today. It shows Garmin's own metrics, explains them
in plain words, and nudges you to improve. No data dumps, no invented scores.

Android first. Web and iOS come after v1.

## Principles

- **Garmin's numbers, explained.** Every metric is Garmin's. Pepometer's own
  math is limited to your normal ranges, week-over-week changes, and the
  advice rules below.
- **Answer first.** Every screen opens with one sentence: the state, the main
  reason, and what to do. Numbers sit underneath; detail is one tap away.
- **Fast.** Opens instantly from local data and syncs in the background.
- **Private.** Data stays on the phone. No accounts, analytics, or services
  other than Garmin.
- **Its own identity.** Own name, palette, icons and copy. Garmin's metric
  names describe Garmin data. Nothing from WHOOP's brand or app.

## Screens

Bottom tabs: **Today · Sleep · Training · Health**.

### Today

- Three dials: **Readiness** (Training Readiness, 0–100), **Sleep** (Sleep
  Score, 0–100) and **Load** (7-day acute load on a gauge marked with Garmin's
  optimal range).
- A coaching line, for example: "Readiness 38, low. You slept 5h 50m of the
  7h 40m you needed and HRV is below your range. Keep today easy and be asleep
  by 22:15."
- Cards: Body Battery now and its overnight charge, HRV status, vitals ("All
  normal" or what is off), today's workouts, Fitness Age with its trend, and
  the weekly check-in on Mondays.

### Readiness

- Score and Garmin's level (Prime/High green, Moderate amber, Low/Poor red),
  with Garmin's feedback rephrased in plain words.
- Garmin's factors (last night's sleep, recovery time, HRV status, acute load,
  sleep history, stress history), worst first. The biggest drag gets one line
  on what it means.
- 30-day trend and today's suggested effort.

### Sleep

- Score and Garmin's rating; time asleep against Garmin's sleep need (when the
  watch reports it); tonight's suggested bedtime.
- Stage timeline, stage totals against your usual, and Garmin's sleep-score
  factors.
- Overnight HRV, respiration, SpO2 and resting heart rate.
- 14-day bed and wake consistency.

### Training

- Acute load gauge with Garmin's optimal band and load-ratio status.
- Training status (Productive, Maintaining, Unproductive, …): what it means
  and what to change.
- Load focus (low aerobic, high aerobic, anaerobic) against Garmin's targets,
  with any shortage called out.
- VO2max trend, race predictions, endurance and hill scores, running
  tolerance.
- Workout list.

### Workout

- One line on what it did: Garmin's aerobic and anaerobic Training Effect,
  primary benefit, training load and recovery time added.
- Time in heart-rate zones, heart-rate chart, route map, laps and splits, and
  pace, power, cadence and running dynamics when present.

### Health

- **Fitness Age** against your real age, with its trend and Garmin's drivers
  (VO2max, resting heart rate, BMI or body fat, vigorous minutes). Each driver
  shows your value and Garmin's target; the biggest lever comes first.
- **Vitals**: last night's resting heart rate, HRV, respiration, SpO2 and
  skin-temperature change, each on a bar showing your normal range. When two
  or more are outside it, a banner says so: "Your body may be fighting
  something or under strain. Take it easy today." 30-day charts.

### Weekly check-in

Generated Monday morning for the week just ended, compared with the week
before: average readiness; sleep score, time asleep against need, and
consistency; total load and training status; HRV and resting heart rate
trends; VO2max and Fitness Age changes. It ends with one win (a personal
record, a best week, an improvement) and one focus for the coming week. Past
check-ins stay in a list.

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
  (Garmin's latest, otherwise the 14-day average).
- **Normal range**: the middle 80% of the last 30 nights, once there are at
  least 14. HRV uses Garmin's own baseline instead.
- **Weekly focus**: the biggest gap among sleep against need, load-focus
  shortage, falling readiness, and vigorous minutes against the Fitness Age
  target.

## Garmin access

- Pepometer uses Garmin Connect's unofficial API, the one Garmin's own apps
  use. Garmin has not sanctioned it and can change it, so Pepometer stays out
  of the app stores. Releases are APKs on GitHub, installable with Obtainium.
- **Sign-in** uses Garmin's own sign-in page inside the app, MFA included.
  The app catches the one-time service ticket Garmin redirects back with and
  exchanges it at `diauth.garmin.com` for access and refresh tokens, as
  [python-garminconnect](https://github.com/cyberjunky/python-garminconnect)
  does. Tokens are encrypted with an Android Keystore key and refresh
  themselves.
- **Data** comes from `connectapi.garmin.com`: daily summary, sleep, HRV,
  training readiness, Body Battery, SpO2, respiration, training status, max
  metrics, Fitness Age, endurance and hill scores, race predictions, running
  tolerance, personal records, and activities (summary, splits, heart-rate
  zones, FIT file).
- The first sync pulls 90 days; older history fills in slowly in the
  background.
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
- **Later**: `crates/server` (axum) runs the same core and serves the same UI
  as a web app on a home server. It would sign in once from a CLI.

## Milestones

0. **Spike (go/no-go gate)**: a bare app on Dan's phone that signs in to
   Garmin, syncs readiness with the app closed, and posts a notification. If
   any of the three fails, stop and report.
1. Core: Garmin client, storage, sync and backfill, plus a dev CLI that runs
   against real data.
2. Today, Readiness and Sleep, then a screenshot review.
3. Training and Workout, then a screenshot review.
4. Health, notifications and the weekly check-in, then a screenshot review.
5. Polish, first APK release, and the repo goes public after a final review.

After v1: web app, iOS, a home-screen widget, and a journal (Garmin's
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

## What Dan provides

- One terminal sign-in on the development machine for a token, so the app
  can be built against real data that never leaves that machine.
- One sign-in on the phone for the spike.
- Four screenshot reviews.
