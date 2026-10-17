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
  limited to fallback normal ranges, week-over-week changes, weekly plan
  progress, and the advice rules below.
- **Fast.** Opens instantly from local data and syncs in the background.
  Switching views draws the last data at once and refreshes it behind the
  scenes; the full-screen loading state is only for a first launch with no
  data.
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

- The quiet header line reports the selected watch's last upload to Garmin
  Connect, not Raring's last API fetch. A stale upload gets a gentle card
  inviting a Connect sync; without Connect installed, show plain guidance.
  Explain unsupported missing readings only when the selected watch explicitly
  says it cannot record them.
- Three gauges: **Readiness** (Training Readiness, 0–100) large in the
  middle, **Sleep** (Sleep Score) on the left, **Load** (7-day acute load
  against Garmin's optimal range) on the right.
- A few-word headline ("Ready to push", "Take it easy") and three reason
  chips: sleep time, HRV and load.
- When readiness and another score point different ways, a small line under
  the chips ("Why Body Battery and readiness differ"); tapping it shows one
  sentence naming Garmin's factors. It's a self-contained piece Today's
  layout can move.
- **Through the day**: the header, gauges, headline, chips and the
  reconciliation line stay in place, and the metric cards below keep their
  order. One advice card leads, chosen by where the person is in their own
  day (see Time of day under Rules): after waking, "This morning" shows last
  night's sleep against its need, the limiting factor and then the effort;
  during the day, "Right now" shows the latest synced Body Battery and stress,
  labeled with the watch's upload time, with today's workout or Garmin's
  recovery time; from two hours before tonight's window the sleep plan leads
  with a strip placing now against wind-down and the window, then a calm
  wind-down card and, after the window, a late-night card. Earlier in the day
  the plan is one line. Wind-down and late night show no effort call, no
  countdown and no verdict. A night that hasn't synced says so neutrally,
  with the watch's upload time or the known reason. The headline follows the
  phase ("Wind down at 22:00", "Time to wind down", "Rest when you're
  ready"). Watch and missing-reading notices stay above the leading card;
  two outside-range vitals move vitals to the top. Settings → Today → "Follow
  my day" turns this off (shift work); then, and without a credible
  schedule, Today keeps the fixed order below and says why in one line.
  Changes never switch tabs and wait until a finger lifts.
- **Today's focus**: Rest, Easy, Steady or Push, with the suggestion lit up,
  followed by the readiness factor Garmin rates worst (nothing when Garmin
  rated none). Tap to see all six factors and their feedback.
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
  over Garmin's baseline band. The Body Battery detail has a Why sheet: the
  overnight charge and level at wake, today's stress, and each workout, nap
  or rest period with Garmin's impact, plus Garmin's feedback on the day.
- **Stress**: Garmin's average so far today, its category and day word, and
  the day in 15-minute bars. Tap for the full day of three-minute readings
  with sleep and activity marked, Garmin's time in each category, and 7-/30-day
  daily averages. Times Garmin cannot rate (moving, off-wrist) are gaps, never
  zero.
- Vitals as five compact readings with units, Garmin Health Status when
  available, a range-status indicator and a count of recorded and comparable
  readings. Missing or uncalibrated data is neutral, never an all-clear.
- **This week**: the weekly plan's targets as compact bars with "N/target"
  and the days left, opening the plan. Before targets are set, a small card
  invites setting them up.
- Today's workouts with a heart-rate zone strip, and Fitness Age with its
  trend line. The weekly check-in card appears here on Mondays.

### Readiness

- Large gauge with Garmin's level (Prime/High green, Moderate amber, Low/Poor
  red) and a few-word headline from Garmin's own feedback.
- Factor bars worst first; tapping one explains it in a sentence. A bar's
  color is Garmin's rating; a percentage without a rating keeps its number
  but stays neutral. A factor Garmin didn't rate (it sends NONE, for example
  without valid sleep) is a gap, never 0.
- A Why sheet lists the six factors worst first, each with the value Garmin
  used (sleep score, recovery hours, 7-day HRV, acute load) and Garmin's
  rating, under one line naming what holds readiness back and what helps.
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
- A Why sheet behind the status: acute load with Garmin's load-ratio status,
  load focus, HRV status and VO2max, with a plain line on what the status
  means.
- Daily load bars for the week.
- Load focus bars (low aerobic, high aerobic, anaerobic) against Garmin's
  target ranges, with a shortage flag.
- VO2max with its trend, race predictions, endurance and hill scores, running
  tolerance.
- **Strength · this week**, when a strength session in the last 12 weeks
  has recorded sets: sets, sessions, records and a small body map. Opens
  Strength.
- Workout list with zone strips and load.

### Workout

- Training load, aerobic and anaerobic Training Effect gauges, primary
  benefit and recovery time.
- Stats grid, route map, heart-rate chart colored by zone, time-in-zone bars,
  and split bars with the fastest highlighted. Power, cadence and running
  dynamics when present.
- **Strength activities with sets** replace the effect gauges and stats with
  the sets: a hero (sets, time, reps, load, average heart rate, records), a
  card naming the sets that look off, a body map of this session, and one
  card per exercise in the order done. Each card shows Garmin's name and the
  muscles, a bar per set (reps above, weight below, flagged sets amber,
  record sets starred) over a dashed outline of last time's set, and last
  time's sets as text. An exercise Garmin couldn't name is "Not recognized".
  The heart-rate chart and zones follow. A session timed without lap presses
  says no sets were recorded and links to help. A ? opens a sheet on how sets
  are counted. Raring never changes the sets; fixing them is a later Garmin
  write.

### Strength

Opened from Training's strength card: the last 12 weeks.

- Body map for this week, or the four weeks ending this week.
- One exercise at a time (chips on the phone, a list with sparklines on
  desktop): the heaviest set each session as a chart, the best weight at 1,
  3, 5, 8 and 10 reps, and its sessions (heaviest set, reps of every set,
  record star), each opening its workout. Show-all controls expose every
  exercise session and workout in the supplied history, not just the initial
  six sessions or eight workouts. The strength workout list starts at the
  same first-week boundary as the chart and filters by sport before any
  pagination limit; newer workouts in other sports cannot hide strength
  activities.
- Sets per muscle this week with a tick at the previous four weeks' weekly
  average; from 660 px a 12-week heatmap of weekly sets per muscle; the
  strength workouts.
- Phone: one column. Unfolded (660 px+): two columns. Desktop (1000 px+):
  exercise list, exercise detail, muscles.

### Body map

Raring's own front and back figures, female or male from the Garmin profile,
with muscle groups as separate shapes: chest, back, shoulders, arms, core,
hips, quads, hamstrings, glutes and calves. Each is shaded in the amount blue
by sets, gray when none.

### Help

Short offline pages, opened from Settings, the ? on Strength and the
strength workout sheet: lifting with the watch, how sets are counted, the
body map and which watches work. Each area ships its pages with the feature.
Questions go to GitHub; the address is shown as text.

### Health

- **Vitals first**: Garmin's overnight average heart rate (labeled Sleep HR,
  not resting HR), HRV, respiration, SpO2 and skin-temperature change as
  tappable tiles when Health Status is available. Without it, use the
  existing resting-HR and other daily readings. Show units, the personal range,
  its Garmin or Raring source, and explicit within/above/below-range or
  missing/baseline-building labels.
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
- **Intensity minutes**: this plan week's total as a ring against Garmin's
  weekly goal, with moderate and vigorous minutes; it opens the weekly plan.
- When two or more vitals are outside their personal ranges, an amber
  message suggests taking it easy and checking how you feel, without
  diagnosing an illness. A partially calibrated monitor reports only the
  readings it can compare.

### Weekly plan

Opened from Today's plan card. Phone, unfolded (two columns from 660 px) and
desktop layouts follow `design/mockup/` screens `plan` and `plan-setup`.

- **This week**: the week's dates, how many targets are on track ("3 of 4"),
  the days left and which targets are behind. One line per target: an icon,
  "N of target" in Garmin's units, a bar (ink on track, amber behind pace,
  green when done) and a few words: the days that counted or what is left.
  Partial credit shows for measured targets; incomplete evidence says why
  instead of showing a failure. The summary's denominator includes all enabled
  targets, while only measured targets count as reached or on track.
- **Last 8 weeks**: one bar per week, the share of targets reached; a week
  reaching every target is green, a paused week shows a pause mark, and a
  week without a plan or measurable evidence is a gap, never zero. An asterisk
  marks completion across known targets only; even 100% there isn't green
  "All reached" while another target is unknown.
- **Targets** (setup): start from a preset (Strength, Endurance, Sleep,
  Custom) that sets all four targets, then adjust any of them: strength
  sessions a week, intensity minutes (Garmin's weekly goal unless changed),
  Zone 2 minutes and nights asleep on time; zero turns a target off. Pick
  rest days, pause this week (travel or illness, left out of history), and
  see last week measured against the current targets. Changes save at once
  and apply from the current week; earlier weeks keep the targets they had.
  "Last week on these targets" is explicitly a retrospective setup comparison,
  not an achievement record for a week before setup.
- Targets are a local setting in the core's own database, so every platform
  gets them; nothing is sent to Garmin or anywhere else.

### Weekly check-in

Opened from the Monday notification: a short stack of swipeable cards for the
week just ended. Readiness, sleep, training and health each get a gauge or
chart with an arrow against the week before. VO2max and Fitness Age changes
are called out. A timing card shows the week's bed and wake times, their
earliest and latest, how many nights Garmin rated aligned, ahead or behind,
and one timing focus. It is descriptive: no adherence score, streaks or
claims that timing changed a score. An intensity card shows Garmin's weekly
intensity minutes as a ring against Garmin's goal, with the moderate and
vigorous split. Once targets are set, a plan card (mockup `checkin`) lists
each target's result for the plan week that just ended, a check or amber dot,
the change against the week before, and next week's targets and rest days
with Keep and Change. It ends with one win and one focus for the
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

- **Watch freshness**: prefer the unique primary activity tracker. Without one,
  select a sole registration unless it is explicitly an accessory, or select
  the only watch-like registration if all others are explicitly accessories;
  an unknown second device is ambiguous. The selected watch's
  `lastUploadTimestamp` is its upload time (UTC milliseconds); if there is no
  unambiguous watch or valid timestamp, say the upload time is unavailable,
  never substitute Raring's fetch time. Display elapsed minutes
  under an hour, whole hours under a day, then whole days. At six hours without
  an upload between 07:00 and 22:00 local time, suggest opening Garmin Connect;
  never nag overnight. Also suggest it after 10:00 local time when today's
  sleep is absent, yesterday's recorded wake time has passed, and the last
  upload predates that expected wake. Missing sleep alone does not imply a
  stale watch. Refresh the device list at most hourly, including after a 404,
  and the selected device detail on each budgeted sync. Explicit `false` watch
  capabilities may explain missing sleep scores, training readiness, HRV status
  and skin temperature; absent/unknown capabilities do not mean unsupported.
  An overdue sleep upload belongs only in the upload card, not the unsupported
  metrics explanation.
- **Suggested effort**: Garmin's readiness feedback, rephrased. The level sets
  push, steady or easy; a high load ratio caps it at steady.
- **Biggest drag**: the readiness factor Garmin rates worst, its percentage
  breaking ties. Unrated factors never count; with none rated there is no
  drag.
- **Bedtime**: Garmin's recommended bedtime window for tonight when it sends
  one; the asleep-by time is its end. Otherwise the next usual wake (see
  Usual wake) minus tonight's sleep need (Garmin's next-night adjusted need when
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
- **Usual wake**: recorded wake times of the last 14 nights read as clock
  times on a 24-hour circle: each is unwrapped to within 12 hours of their
  circular mean, then the median and the middle half's spread are taken, so
  a steady 23:55/00:05 pattern is one cluster. The median is placed between
  18:00 the evening before and 18:00 on the morning's date. It is credible
  with at least four wake times and a spread of three hours or less. The
  next expected wake after a displayed day is the first usual wake more than
  12 hours after that day's recorded wake (or its own usual wake), since
  Garmin files a night under the day it ends. The sleep plan, Today's phases,
  the weekly timing review and the displayed day share this rule.
- **Displayed day**: the calendar day once its night (a recorded sleep or
  readiness) has synced. Until then the latest night of the previous two
  days stays shown until its handover: three hours after the credible next
  expected wake, otherwise noon of the next calendar day. Stored date keys
  are never changed and no future dates are fetched.
- **Time of day**: phases are local wall-clock times anchored to the
  displayed night's recorded wake (otherwise its usual wake) and tonight's
  shared sleep plan, never fixed clock hours. After waking lasts three hours,
  or until the plan rises if that comes first; the day runs until two hours
  before Garmin's window starts (Raring's asleep-by time without one); the
  evening until the plan's wind-down time; wind-down until the window ends;
  late night until the next expected wake, so crossing midnight keeps
  tonight's plan. From that wake (or, without a credible usual wake, from the
  calendar rollover) until the handover, whatever the layout or fallback,
  Today waits neutrally for the night: no effort call, no previous plan and
  no verdict. After the handover the calendar day shows with its missing
  night stated. Advice order: after waking, this morning then the one-line
  plan; during the day, right now, today's focus, the one-line plan; from
  the evening on, the plan then today's focus. Two or more outside-range
  vitals go first in every phase. The fixed layout (today's focus, then the
  full plan) applies when the setting is off, with fewer than four recorded
  wake times in 14 nights, a usual-wake spread over three hours, Garmin's
  jet-lag rating for a recorded night, the phone's UTC offset more than an
  hour from the latest recorded night's (a daylight-saving change keeps the
  schedule), no plan, or a plan that doesn't fit the wake time. The phase
  is re-evaluated from local data on open, resume, every view refresh, sync
  and at each phase boundary, offline included.
- **Timing review**: the week's earliest and latest bed and wake times, the
  wake-time spread (latest minus earliest) against the week before, and
  counts of Garmin's alignment ratings. The focus follows the majority rating
  when at least three nights are rated, otherwise whether wake times varied
  by an hour or more, and names the week's median wake time.
- **Normal range and vitals off**: use Garmin Health Status's per-day
  baseline bounds and status first. A reported outside-range status counts
  toward the two-vitals warning and notification; `ONBOARDING`, unknown
  statuses and missing values are neutral, even if provisional bounds exist.
  When Garmin has no Health Status metric, use the middle 80% of the last 30
  recorded nights (at least 14) and count readings outside that fallback
  range too; HRV retains Garmin's existing HRV baseline. Never compare Garmin's overnight
  average HR to resting-HR history. Health Status's nightly HRV status is
  separate from Garmin's seven-day HRV status. Personal ranges are not
  medical targets.
- **Stress bars**: the Today card averages each 15 minutes of Garmin's
  readings; the detail shows every reading. Colors follow Garmin's categories.
- **Body Battery color**: good at 70 and above, fair from 30, poor below 30.
  Garmin rates no level, so only the display color is bucketed.
- **Plan week**: Garmin's intensity-minutes week, taken from the dates of its
  weekly intensity rows (Monday until they sync). Garmin's intensity week
  can differ from the account's start-of-week setting; following
  Garmin's window keeps the plan's intensity total equal to Garmin's weekly
  total and lines up with the Monday check-in.
- **Why sheets**: only Garmin's inputs, ratings and feedback, worst first.
  Readiness is *held back by* the factors Garmin rates Poor (Moderate ones
  when none is Poor and readiness isn't High or Prime) and *helped by* the
  ones it rates Good or better, at most two each, ordered by Garmin's
  percentage. Body Battery's line names the overnight charge, the level at
  wake, the workouts that drained it and Garmin's stress word for the day.
  Training status sorts its inputs Poor, Fair, Good, then unrated, and names
  up to two that aren't Good. An input Garmin sent no rating for stays
  unrated; Raring never rates it from its own ranges.
- **Scores that differ**: readiness and another score point different ways
  when one is good and the other poor. Readiness, Sleep Score, HRV status
  and training status use their usual colors; Body Battery uses its level at
  wake, since readiness is a morning reading and the battery drains through
  the day. Fair or missing scores never count. Check sleep, Body Battery, HRV
  status, then training status, and show only the first. The line names the
  other score, then the readiness factors as in the Why sheet: "held back
  by" when readiness is poor, "held up by" when it's good. Without a factor
  Garmin rated that way, it just says readiness is low or high.
- **Wording**: name Garmin's inputs and ratings and stop there: "held back
  by" and "helping", never "because" or a cause Garmin didn't give. Numbers
  are Garmin's; the only arithmetic is hours from Garmin's recovery minutes
  and the existing 30-day VO2max change. No new scores, no medical claims.
- **Plan progress**: intensity minutes are Garmin's weekly total for the
  plan week: the daily summaries' moderate minutes plus twice the vigorous
  ones (Garmin's own weighting), against the target or Garmin's weekly goal.
  Strength counts activities whose Garmin type is strength training,
  including timer-only sessions without recorded sets: missing set tracking
  doesn't mean no workout happened. Zone 2
  minutes sum each activity's seconds in heart-rate zone 2 from Garmin's
  `hrTimeInZones`: Garmin's own zones for that activity, from zone 2's
  `zoneLowBoundary` up to zone 3's, as set on the watch when it was recorded;
  Raring never computes zones from heart-rate samples. Whole minutes,
  rounded down.
- **Evidence coverage**: strength and Zone 2 require a successfully fetched,
  complete activity-list span covering the week through today (or all seven
  days of a past week). A full pagination page proves only dates newer than
  its oldest entry, since that date may continue on the next page; a short
  page proves the list ended. Cached retry pages and malformed lists do not
  extend coverage. Strength also requires a known Garmin type for every
  indexed activity; missing details aren't a zero workout count. Zone 2
  requires an explicit zone-2 duration for every indexed activity, or an
  explicitly empty zone array (verified zero). Missing/null/malformed zones
  remain unknown. No set or lap evidence is required to count strength.
  Intensity needs numeric moderate and vigorous values for every elapsed day
  through today, or Garmin's authoritative weekly row; future days aren't
  required. Daily values take priority when complete, because they refresh
  more frequently than weekly rows. Incomplete targets stay unavailable, not
  Behind, and are excluded from completion. A fully listed empty week and
  explicit zero summaries are measured zeros, not gaps. Existing databases
  need a new activity-list scan before activity coverage can be trusted.
- **Saved sleep plans**: on live sync and opening current Today or Sleep,
  capture the first non-stale shared plan prospectively in the local settings
  store (`sleep_plan:YYYY-MM-DD`): the night, dated bounds, source, an
  offset-bearing capture timestamp, and local capture time. Capture must precede the window's
  start (or asleep-by estimate) by more than 15 minutes and no recorded sleep
  for that night may already exist. The first capture is immutable; later
  advice changes, outcome uploads and backfill never rewrite it. Historical
  chart reads never capture plans. Include these settings records in export.
- **Asleep on time**: compare Garmin's recorded sleep start only to that
  night's saved plan: inside Garmin's bedtime window widened by 15 minutes
  either side, or within 15 minutes of the saved Raring asleep-by estimate.
  The capture must precede the actual sleep start as well. Without a saved
  plan or recorded sleep, exclude the night rather than count a miss. This
  intentionally provides no retroactive adherence for nights before capture
  existed, even if their sleep data and recommendations can be backfilled.
- **Pace**: a target is behind when its share reached (value ÷ target) is
  below the share of the week's training days that have passed: days before
  today that aren't rest days, over all training days. Today hasn't passed,
  so a session later today is never late. Asleep on time uses nights instead:
  nights already counted over the nights in the week that weren't left out.
  Done means value ≥ target. "On track" counts measurable targets that
  aren't behind; the check-in counts targets reached.
- **Completion and history**: each target's completion is value ÷ target,
  capped at 100%; a week's completion is the average over its measurable
  targets. Paused weeks and weeks with no measurable targets have none. Each
  week is measured against the targets in effect that week; weeks before the
  first configured plan are unconfigured gaps in history and check-ins.
  Changing targets later never assigns new targets to those earlier weeks.
  Check-ins distinguish All reached, Partly reached, No targets reached,
  Partial evidence, Not enough evidence and Paused. All reached requires
  every enabled target to be measured and reached.
- **Weekly focus**: the biggest gap among sleep against need, load-focus
  shortage, falling readiness, and intensity minutes against Garmin's weekly
  goal for the plan week that just ended (without them, Fitness Age's
  vigorous-minutes average against its target).
- **Strength sets**: Garmin's `ACTIVE` sets are sets; `REST` sets are rest
  time. A set's exercise is Garmin's most probable guess; an `UNKNOWN`
  category is unnamed. Weights are Garmin's grams shown in kilograms or
  pounds by Garmin's measurement setting, to 0.1.
- **Sets per muscle**: each named set counts once for each primary-muscle
  group of its exercise in Garmin's catalog: the exercise's own entry, else
  its category's, else Raring's small category mapping. Garmin's muscles
  group as lats, traps and lower back → back; biceps, triceps and forearm →
  arms; abs and obliques → core; hips, abductors and adductors → hips; the
  rest one to one. Two muscles in one group count once. Unnamed sets count
  for no muscle. Weeks start on Garmin's start-of-week day. The tick is the
  average of the four weeks before this one; the four-week body map sums the
  four weeks ending with this one.
- **Miscount flags**: an active set is flagged for 0 or no reps; for reps at
  least 1.8 times the median of the same exercise's other sets with reps in
  that session (none without another set); or for an exercise Garmin
  couldn't name. Flags never change Garmin's numbers. The card counts each
  flagged set, except an unnamed exercise counts once.
- **Records and bests**: a set is a record when an earlier session has the
  exercise and no earlier set (or earlier set in this session) was at least
  as heavy for at least as many reps. A best at N reps is the heaviest set
  with at least N reps. A session's top set is its heaviest set, then most
  reps. Only named, weighted sets that aren't flagged for reps count.
- **Sessions with sets**: require at least one active set with a named
  exercise or positive reps. Timer-only sessions add nothing to set, muscle
  or record aggregates, even in a week containing full set workouts. The
  strength card labels its tally "sessions with sets". All Garmin strength
  activities still appear in Training and count toward the weekly plan's
  session goal. Strength's activity list keeps workouts without set details,
  labeled "No sets recorded".
- **Rep totals**: sum known active-set rep counts; if any are missing, label
  how many sets are uncounted. If all counts are missing, show a gap, never
  zero. A recorded zero remains zero; missing counts have their own flag.

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
  sleep stats (Garmin's average bed and wake times), HRV, training readiness,
  Body Battery, stress, SpO2, respiration, Health Status (GraphQL baseline and
  status), weekly intensity minutes (dates of Garmin's intensity weeks, read
  daily), training status, max metrics, Fitness Age, endurance and hill
  scores, race predictions, running
  tolerance, personal records, and activities (summary, splits, heart-rate
  zones, charts and GPS routes), plus strength sets for strength activities.
- **Strength**: sets come from `/activity-service/activity/{id}/exerciseSets`
  for activities Garmin types `strength_training`: once, then at most hourly
  for 48 hours after the start to pick up early Connect edits. Later edits
  are not fetched by Refresh or reopening a workout; help states this limit.
  A budgeted explicit detail refresh remains a follow-up. Older strength
  activities found by the history download get their sets after it. Once
  there are sets, the user settings
  (`/userprofile-service/userprofile/user-settings`) are read weekly and only
  the measurement system, time format, start of week and gender (the body-map
  figure) are stored; the rest of `userData` is discarded. Garmin's public exercise catalog
  (`connect.garmin.com/web-data/exercises/Exercises.json`, no credentials) is
  read monthly and only each exercise's primary muscles are kept.
- Each sync fetches the last 90 days day by day, then spends its remaining
  request budget downloading older history in range requests, newest first
  and resuming where the last run stopped, back to the account's oldest
  data. A new phone downloads it all again; there is no Raring cloud. Older
  days keep daily values; Settings shows how far back the history goes.
- Garmin Connect stays installed on the phone because it uploads the watch's
  data. Device registration lists watches and their capabilities; device
  service details give the watch's last upload. Opening Connect is an Android
  package launch only; on other platforms or without Connect installed Raring
  gives plain guidance.
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
- **View cache**: after the first view renders, the four tabs are
  prefetched and cached by route for the current day. A completed sync, a
  changed sync timestamp or a local day rollover invalidates in-flight reads
  and the cache without clearing the visible view. Status is read on start,
  resume and sync, not before every navigation, and is applied independently
  of navigation so a tab switch can't discard a newer sync or sign-out. View
  commands read SQLite off the WebView thread. Today carries the end of its
  time-of-day phase: a cached Today past it is read again instead of shown,
  and the visible Today is read again when it passes.
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
