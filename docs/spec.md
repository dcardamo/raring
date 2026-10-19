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
  limited to fallback normal ranges, display averages and period
  comparisons, week-over-week changes, weekly plan progress, and the advice
  rules below.
- **Fast.** Opens instantly from local data and syncs in the background.
  Switching views draws the last data at once and refreshes it behind the
  scenes; the full-screen loading state is only for a first launch with no
  data.
- **Private.** Data stays on the device. No accounts, analytics, or services
  other than Garmin.
- **Cross-platform.** One Rust core and one Svelte UI for Android, iOS and
  desktop. Native code is limited to what a platform requires, such as
  background scheduling; everything else lives in the shared code.
- **Export.** Settings creates a ZIP offline from the local database: indexed raw
  Garmin JSON responses, CSVs of all per-day response tables (scalar columns
  plus the full nested JSON), every non-credential Raring settings key, and a
  manifest with creation time, app version and row counts. Credential fields
  are excluded recursively, including in CSVs; sign-in tokens are never read.
  Android offers the completed private-cache file through a read-only share
  URI and schedules cleanup before writing, 24 hours after export creation,
  including an interrupted partial file; desktop saves it in Downloads.
  An export leaves the phone only when the user chooses to share it. FIT
  originals are not downloaded as part of this offline export. A user-created
  export may include stored opted-in journal records; it never includes tokens.
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

Bottom tabs: **Today · Plan · Training · Health · Progress**. Sleep opens
from Today. Each root header has search (pages and Garmin metrics by name)
and a profile button: Profile holds the Garmin history on the phone, gear,
Settings and help. Tapping a dial or card opens its detail; Back returns to
the page that opened it. Earlier links still open their current page:
`#checkin` the latest weekly review, `#plan-setup` the weekly goals,
`#explore` Training; an unknown link opens Today.

### Welcome & privacy

Before sign-in, explain that Raring reads a Garmin account, not a watch, and
that Garmin Connect must keep syncing the watch's data. State that the app
sends no data to Raring, has no analytics or telemetry, and keeps downloaded
history on the phone. Exports leave the phone only when the user shares them.
Explain the unofficial connection and watch-dependent metrics before the
sign-in button. Keep the same information and expandable privacy details
available offline in Settings. Notification permission remains
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
  during the day, "Right now" shows the latest synced Body Battery, stress and
  available heart rate (not a live reading), labeled with the watch's upload time,
  with today's workout or Garmin's
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
  with sleep and activity marked, Garmin's time in each category, and the
  daily average as a trend chart. Times Garmin cannot rate (moving, off-wrist) are gaps, never
  zero.
- Vitals as five compact readings with units, Garmin Health Status when
  available, a range-status indicator and a count of recorded and comparable
  readings. Missing or uncalibrated data is neutral, never an all-clear.
- **This week**: the week's nested goal rings with one line per goal and
  the seven day cells (Rules: Rings), opening Plan; without goals it shows
  the days only. Hidden during wind-down and late night.
- A read-only next-session card from Garmin's scheduled workout calendar,
  separate from readiness and effort advice. It is absent during wind-down
  and late night; missing or stale schedules are not empty schedules. Today's
  recorded activities remain separate from planned sessions. Fitness Age has
  its trend line. The weekly check-in card appears here on Mondays.

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
- Readiness trend chart and today's call. Tap the chart or use the date
  slider to inspect a reading. Dates are anchored to the stored Garmin day,
  missing days remain gaps, and daily bars use Garmin's level boundaries (50
  for Moderate, 75 for High).

### Sleep

- Score gauge, and a time-asleep bar with a marker at Garmin's sleep need
  for that night (when the watch reports it), labeled as last night's need.
- Stage timeline (awake, REM, light, deep) and a stage bar with each stage's
  change against your usual.
- Overnight tiles with lines: HRV, heart rate, respiration, SpO2.
- **Recorded naps** from Garmin's nap endpoint: dated local start/end from
  GMT timestamps plus per-end offsets (seconds), `napTimeSec` in seconds,
  and optional Garmin `napFeedback` without interpreting its enum. Display
  both local dates with the clocks, including when a nap crosses midnight;
  label missing start/end without reconstructing them from duration.
  Show duration separately from main overnight sleep; never add it to sleep
  score, asleep minutes or Raring's sleep plan. Garmin's own `napAdjustment`
  for tonight's sleep need is distinct. An empty array is no recorded naps;
  absent, failed and stale reads have separate neutral labels. Other populated
  nap fields and feedback meanings require further verification before display.
- **Consistency**: 14-night chart with your usual bed and wake windows
  shaded, and Garmin's seven-day average bedtime and wake time above it when
  synced. No rhythm score or chronotype of Raring's own.
- **Sleep history**: Garmin's sleep score as a trend chart; time asleep is
  one overlay away.
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
- Latest recorded Garmin running lactate-threshold HR and pace, running threshold
  power, cycling FTP, heat acclimation and acclimated altitude in a compact
  Training card, each with its own recorded date. Missing, null or stale
  thresholds are omitted; an explicit heat 0% is shown. Availability depends
  on the watch and recorded activities. No thresholds are estimated here.
- **Strength · this week**, when a strength session in the last 12 weeks
  has recorded sets: sets, sessions, records and a small body map. Opens
  Strength.
- A read-only next-session card with active Garmin Coach context when known,
  and an upcoming race countdown from Garmin's calendar dates. Empty, stale
  and unavailable schedule/calendar states are distinct. No scheduled item is
  marked done by time or name; the workout list below contains recorded
  activities with zone strips and load.
- **Gear**: a compact expandable card lists Garmin shoes and bikes, including
  retired gear, with Garmin's accumulated distance against its own distance
  threshold when one exists. Distance uses Garmin account distance units when
  known, otherwise the endpoint's metres. A missing distance is not zero;
  a zero or absent threshold is not a suggested lifespan. The card labels
  Raring's last successful gear-list fetch time, not an activity measurement.
  No gear changes, activity assignments or gear notifications.

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

- **Garmin journal**: a private, short dated view of the last seven lifestyle
  log days, opened explicitly in Health and off by default until local consent.
  Live Health anchors those seven dates to the actual local calendar day even
  when its vitals still show the retained sleep day; explicit historical Health
  reads remain anchored to their requested date. Only consented syncs fetch
  the daily log: recheck at dispatch after client serialization, token
  refresh, pacing and before any 401 retry; atomically check consent again
  when caching each reply. Closing or leaving Health hides entries.
  Garmin's `dailyLogsReport` rows may carry `name`, `logStatus`
  (YES/NO) and `details` with `amount` and `subTypeName` (no generic unit).
  A null report or unknown log status is not a recorded zero/no; completion
  tracking statistics do not prove individual entries. Populated report rows
  are known from community examples, not verified on this account. No new
  scoring or correlations. No journal content in notifications or lock-screen
  views. Opting out stops new journal reads and hides stored rows but does not
  erase them; future user-created exports still include previously downloaded
  logs until local data is erased.
- **Vitals first**: Garmin's overnight average heart rate (labeled Sleep HR,
  not resting HR), HRV, respiration, SpO2 and skin-temperature change as
  tappable tiles when Health Status is available. Without it, use the
  existing resting-HR and other daily readings. Show units, the personal range,
  its Garmin or Raring source, and explicit within/above/below-range or
  missing/baseline-building labels.
  Each detail has a trend chart with the current range shaded on daily ranges, actual dates and missing nights left blank. Nightly HRV against the
  baseline is distinct from Garmin's seven-day HRV status. Explain the comparison
  method behind a tap; personal ranges are not medical targets.
- **Stress**: today's average with a 30-day line of daily averages, opening
  the same detail as Today.
- **All-day heart rate**: Garmin's daily resting HR (bpm) and a tappable day
  chart in two-minute elapsed slots, with recorded sleep and activities shaded
  or marked. Include the next morning's sleep as the previous evening's
  interval, clipped to each displayed day. Missing timestamps, null and
  off-wrist readings are gaps, never zero. The latest sample is synced, not
  live. The daily resting value is distinct from Health Status's Sleep HR
  (the overnight average); name both. Today and yesterday refresh with every
  sync under its request budget; older days are fetched only when opened and
  kept, including empty days. Retry failures and 429s follow the shared
  backoff. Older days open from the previous/next day controls on the detail.
- **Weight and body composition**: show the last dated weigh-in, a sparse
  12-week trend and Garmin's BMI/body fat if recorded. Weight follows Garmin's
  measurement setting (kg or lb), not the activity set's unit in isolation.
  Missing measurements and weeks stay blank, never zero. Weight is private to
  Health; it does not trigger notifications. Until Garmin's measurement-system
  setting syncs, the tile waits rather than guessing kg for a pounds account.
- **Fitness Age**: large number, trend line, and a bar placing it against your
  real age.
- **Biggest lever**: a progress ring toward Garmin's target for the driver
  that would help most (for example vigorous minutes), with the age you'd
  reach.
- **Drivers**: VO2max, resting heart rate, BMI or body fat, and vigorous
  minutes as tiles with range bars. Garmin's Fitness Age component owns its
  target and age estimate; a recorded weigh-in can supply a missing BMI or
  body-fat value measured in the last 30 days but never creates a target or
  Fitness Age prediction.
- **Intensity minutes**: this plan week's total as a ring against Garmin's
  weekly goal, with moderate and vigorous minutes; it opens the weekly plan.
- When two or more vitals are outside their personal ranges, an amber
  message suggests taking it easy and checking how you feel, without
  diagnosing an illness. A partially calibrated monitor reports only the
  readings it can compare.

### Trends

Every trend chart (Readiness, the vitals, stress, Body Battery, VO2max,
acute load, Fitness Age and Sleep's history card) is the same interactive chart, drawn from local data.

- Ranges 7D, 30D, 90D, 1Y and All, plus Dates for a chosen training block. Up to 90 days shows each day; longer
  ranges show weekly or monthly averages, and the selected point says how many
  days it holds.
- **Compare**: off, the previous period (same length, faint dashed line on
  the same axis, with both averages and the change), or a second metric with
  its own right-hand axis. Both metrics are selectable; periods without data
  remain empty. Dates selects a block's inclusive start and end; the comparison
  start selects an equally long block, aligned by elapsed day. One comparison
  at a time; All has no previous period.
- Garmin rates days, not averages: only daily readiness bars take Garmin's
  level colors, and the current baseline band shows only on daily ranges.
  Averages, overlays and changes are ink, with no verdict and no new score.

### Plan

The plan week (Garmin's intensity week, see Plan week) at a glance, from the
local goals and sessions (Rules: Local records to Backlog).

- **Week**: dates, a pause or illness note, the nested week ring and one line
  per goal: recorded of target, placed, to review, left or reached; a change
  during the week shows the starting target. Sleep Score is a Garmin reading
  against its target. Missing evidence says so instead of a number.
- **Goals** (`#plan/commitments`): steppers for strength sessions, Zone 2,
  Zone 4–5 and intensity minutes (Garmin's weekly goal until changed), sleep
  timing (asleep by or wind down at a chosen time) and an average Sleep Score.
  Zero turns a goal off. Saves apply as the Local records rule says.
- **Days**: day cells with rings; a day lists your sessions with one state word
  (Done, Needs review, Planned, Not recorded, Awaiting data, Marked missed),
  recordings not in your plan, and Garmin's scheduled workouts by name,
  read-only and not counted.
- **Left this week**: the backlog, or a paused note with nothing to catch up.
- A migrated sleep-timing observation is explained once.

### Weekly check-in

Opened from the Monday notification (or Progress → Last week, `#review/latest`;
an explicit week is `#review/YYYY-MM-DD`): a short stack of swipeable cards for the
week just ended. Readiness, sleep, training and health each get a gauge or
chart with an arrow against the week before. VO2max and Fitness Age changes
are called out. A timing card shows the week's bed and wake times, their
earliest and latest, how many nights Garmin rated aligned, ahead or behind,
and one timing focus. It is descriptive: no adherence score, streaks or
claims that timing changed a score. An intensity card shows Garmin's weekly
intensity minutes as a ring against Garmin's goal, with the moderate and
vigorous split. Once goals are set, a plan card (mockup `checkin`) lists
each strength, intensity, Zone 2 and migrated sleep-timing goal's result for
the plan week that just ended (read from the local goals), a check or amber
dot, the change against the week before, and next week's targets and rest
days with Keep and Change (Change opens the weekly goals and returns here).
It ends with one win and one focus for the coming week, a line each.

## Android status surfaces

A small home-screen widget shows readiness and Today's focus; a medium one
shows readiness, sleep score, Body Battery and the latest measured stress.
Tapping a reading opens its detail; the small widget's focus opens readiness,
sleep or Today as appropriate. Missing readings are blank, never zero.
Both widgets and the optional ongoing notification always show "As of" with
the selected watch's upload date and time, or "As of unavailable".

The notification has a separate low-importance, silent channel, no sound or
vibration, and private lock-screen content. Collapsed: battery, stress and
heart-rate icons and values. Expanded: battery/stress bars, phase-aware focus
and its reason. Latest heart rate uses the current calendar day's last measured
sample from the shared all-day HR parser. It remains blank without samples;
resting/overnight heart rate is not a substitute.
It starts off, is offered once on Today after the first successful sync, and
can be changed in Settings. Permission denial leaves it off. Explicit opt-out
persists even if snapshot publication fails; native rendering reads authoritative
SQLite enablement, never the old file's enabled bit. Cancellation does not wait
for a network sync. Failed opt-in publication still rolls the opt-in back. Android may let
the user dismiss an ongoing notification; the next update restores it.
Widgets do not require enabling the notification. Their declared minimum and
minimum resize sizes are 140×300 dp (small) and 300×180 dp (medium); focus and
upload time can wrap. Android RemoteViews are tested at those actual bounds
with 1.0, 1.3 and 1.5 font scales, including missing readings.

## Planned training (read-only)

The core fetches active Garmin Coach plans, a dated 14-day-ahead workout
schedule (inclusive start and end), and up to 10 upcoming calendar events in
Garmin's 90-day window. Coach tasks wait for a verified display projection.
Each snapshot is checked at most every six hours (including absent responses),
shares the normal request budget and rate-limit backoff, and becomes stale
after 12 hours or when its cached window no longer covers today. An empty
schedule names its **cached start and end dates**, not a rolling promise of
"the next two weeks" after midnight. A new empty response replaces an older
schedule; failed reads keep the last snapshot but never present it as current.

A calendar date is a local Garmin day, including for all-day races; countdowns
use date subtraction, not UTC instants. Today's sleep/readiness day can still
be yesterday after midnight, but live planned cards always use the **actual
local calendar date**. Explicit historical views retain the selected date.
Garmin's upcoming-events endpoint has a limit of 10: fewer than 10 typed
events with no race means no race in the returned 90-day window, but a full
10-event page is **incomplete coverage**. A full page without a race is never
"no upcoming race"; a race in a full page is only the earliest visible in
that page, marked incomplete. No undocumented pagination or extra requests.
An active Coach plan is separate context, not proof that the next scheduled
workout belongs to it. Unknown statuses and fields stay neutral; no completion
or cancellation mapping is assumed before a populated response is verified.
An empty account does not establish watch or plan support. No Garmin writes,
workout builder or daily-suggested-workout inference ship here.

## Notifications

Sent by the phone itself.

| Notification                  | When                                         | Default |
| ----------------------------- | -------------------------------------------- | ------- |
| Morning readiness             | Last night's sleep and readiness have synced; while resting, ill, paused or limited, a factual update with that line and no training call | On      |
| Weekly check-in               | Monday 08:00                                 | On      |
| VO2max or Fitness Age changed | Either direction                             | On      |
| Vitals off                    | Two or more vitals outside normal range      | On      |
| Training status changed       | On sync                                      | Off     |
| Bedtime                       | Wind-down time of tonight's plan (lead 30 min, configurable) | Off     |
| Persistent status (Android)   | Existing sync; silent, no extra Garmin requests | Off     |

## Rules

- **Local records**: goals, your sessions, plan links, circumstances (rest
  days, pauses, limits, dismissals, wind-down reports), plan-week prompt
  answers and local events are versioned JSON rows (`local:*`, each with
  `schema`) in the local settings table, so the export, a verified account
  switch and sign-out cover them like every other setting; nothing is sent to
  Garmin. A row with a newer schema, or one that doesn't parse, is read as far
  as it parses and never rewritten. Each change is one transaction under the
  account barrier, refused if the account changed since it was queued, and
  refreshes the status snapshot offline. Goals take effect by date: the first
  targets from the start of that plan week, later changes from the day they
  are made (keeping the week's original targets), and a manual change drops
  accepted changes for later weeks. Sessions are added and moved only to today
  or later; a passed open session stays until the person lets it go or marks
  it missed, and moving one clears its link. One recording links to at most
  one plan item; unlinking an automatic pair keeps it apart until linked
  again, and an unlinked session no longer competes for that recording. A pause has one open interval until the person resumes; resuming ends
  it the day before (a pause begun the same day leaves none), and a review
  date only prompts. A dismissal lasts its local day (kept 30 days). A wind-down
  reported before noon belongs to the evening before.
- **Plan migration**: the earlier weekly-plan row migrates once, on the first
  local read or change. No saved version (`configured: false`) means no goals;
  the preset preview is never adopted. Each saved version becomes a goal version
  from the same week (no saved intensity follows Garmin's weekly goal; zero is
  off; rest weekdays kept). Saved asleep-on-time nights become a sleep-timing
  observation against each night's saved plan, shown without a verdict; it can
  be kept or turned off, not restarted. Paused weeks become whole-week pauses.
  The old row and saved sleep plans stay, so earlier weeks keep their original
  results. An empty database writes nothing, so a first sign-in can bind. A
  row this build may not rewrite is left alone and the migration stays
  unfinished; the earlier setup's saves update the goals first and change
  nothing if that is refused.
- **Plan matching**: plan items are your sessions. Garmin's scheduled workouts
  are listed read-only by name and date; their sport, duration and targets are
  unverified, so they are neither counted nor matched. A link the person made
  always holds. Otherwise a session and a recording match when they share the
  local date and sport family (Garmin's type key: run, cycle/bike/ride, swim,
  walk, hike, strength; anything else only by the same type key, so HIIT is
  never strength) and each is the other's only candidate; an unlinked pair
  stays apart. Raring asks ("Needs review") only for a passed or current day
  with a same-sport recording it couldn't pair, or, on a passed day, when the
  only open session and the only unpaired recording are different endurance
  sports (run, cycle, swim, hike) and the recording lasted half to twice the
  planned duration. A session let go or marked missed keeps that state. An open
  session on a passed day without a match is Not recorded, or Awaiting data when
  that day's activity list is incomplete; never Missed unless the person says
  so. A wind-down session is done when a wind-down was reported for its date.
- **Goal ledger**: for each goal on in the week (targets in effect today, or
  at the week's end for a past week): recorded, planned and remaining. Goals
  count recordings, never plans; each recording counts once per goal however it
  was matched. Strength counts Garmin strength activities, Zone 2 sums each
  activity's zone-2 seconds and Zone 4–5 its zone-4 and zone-5 seconds (whole
  minutes, rounded down after summing), under the Evidence coverage rule;
  intensity is Garmin's weekly total (Plan progress) against the chosen
  minutes or Garmin's weekly goal; sleep timing counts nights asleep by the
  chosen time (Garmin's recorded sleep start up to 15 minutes after it; a time
  before noon is after midnight), nights with a reported wind-down, or for the
  migrated observation nights asleep on time against their saved plan.
  Planned is the allocation of sessions still Planned (today or later), never
  a duration converted to zone minutes; a session needing review is neither
  planned nor a second count. Remaining is target − recorded − planned, at
  least zero, and unknown while evidence is incomplete or the week has a pause
  through today. Reached means recorded ≥ target (never for the observation).
  A change during the week shows the week's starting target beside the new one.
  A future week records nothing yet. First targets and migrated versions apply
  to the whole plan week they were saved in, even when Garmin's week starts
  before their date. Sleep timing with no recorded night yet is unknown, not
  zero. Average Sleep Score is Garmin's nightly
  score averaged over the week's recorded mornings through today, shown against
  its target as a reading.
- **Rings**: one per goal on, in the order strength, Zone 2, Zone 4–5,
  intensity, sleep timing (the observation and Sleep Score have none). A day's
  ring draws only what was recorded that day: recorded ÷ the larger of what was
  planned that day and what was recorded, so recorded work with nothing planned
  fills it and planned work alone draws nothing. Days still to come, and passed
  days whose activity list is incomplete (marked unknown), and zone rings on a
  day with an activity lacking zones, draw nothing. A
  day's intensity is Garmin's daily moderate plus twice vigorous minutes. Rest,
  illness and pause mark their days. A week ring is recorded ÷ the weekly
  target, at most full.
- **Week review**: each week is measured against the goals in effect at its
  end (Goal ledger). A week counts when its activity list is complete and it
  had at most one paused or ill day; otherwise it is listed with its reasons
  (incomplete list, ill days, paused days; rest days are noted) and ignored.
  An ongoing week is not counted as complete. Old reviews cannot change goals
  for weeks that have already passed. Met means recorded ≥ target; a minute goal is met comfortably at ≥ 110%.
  For next week, for each goal on (not Sleep Score, not intensity following
  Garmin's goal, not the migrated observation), from up to eight counted weeks
  since goals began: two or more ill days in the week just reviewed ease back;
  fewer than two counted weeks keep; the last two counted weeks met and the
  latest comfortably (sessions and nights: the last three met) nudge up by 10%
  rounded to 5 minutes (at most +20) or one session (strength at most 5,
  nights at most 7); two of the last three counted weeks under 70% (sessions
  and nights: under target) ease back by 10% rounded to 5 (at least 5) or one
  (at least one); otherwise keep. Each proposal shows its reason. Only the
  person's acceptance changes goals, as a version from the next week; Keep all
  and ignoring a review change nothing. The decision is kept with the
  proposals it saw.
- **Plan prompt**: for people who have chosen goals before, while the setting
  is on and the week isn't answered: at the start of a week (its first two
  days) after a week with goals, it offers last week's review then planning;
  otherwise only while this week has nothing placed. Never in wind-down or late
  night. Plan now, Not this week and turning it off are answers.
- **Events**: local events from today and Garmin's next race (Planned
  training) count down by calendar date; an event date is not proof of
  preparation or completion.
- **Progress periods**: Week is the plan week, Month the calendar month,
  3 months the anchor month and the two before, Year the anchor month and the
  eleven before; a period never extends past today. Dates are the activity's
  local start date. Totals are known totals: activities, their recorded time,
  recorded distances only (none is not zero), and Garmin's daily moderate plus
  twice vigorous minutes over the days that have them. A day is covered when
  the activity list is complete for it (Evidence coverage); "active on n of m"
  counts covered days, so a known workout on an uncovered day keeps its
  calendar mark without counting. Calendar days are active, rest, ill, paused,
  none, unknown (an uncovered day without workouts, never zero), future or
  before downloaded history. Bars show minutes by sport per day (week and
  month), per plan week (3 months) or per month (year). A current period is
  compared with the same elapsed days from the start of the previous period; a
  completed one with the whole previous period. No comparison when either
  window isn't fully covered or the shorter previous period can't hold the
  days. Changes are signed, never coloured good or bad. Goals over time shows
  the last twelve plan weeks since goals began, each with its own rings and
  whether it counted. Sleep against need counts nights with both Garmin's
  recorded sleep and need, and those an hour or more below it; no running total.
- **Milestones**: in available history: an exercise's heaviest set for 1, 3,
  5, 8 or 10 reps (Records and bests eligibility), the longest recorded
  distance for a known sport family recorded at least three times, and the first
  completed, counted week with every action goal met. Mixed unknown sports are
  not combined into a distance record. A week with a revision or pause cannot
  earn this whole-week award; observations alone never qualify.
  Records use stable identities and canonical units (grams for lifts), so a
  display-unit change is not an improvement. A genuinely better value recorded
  on a later date can be new; backfill is quiet. Lower, earlier, replaced or
  vanished evidence is corrected without another award; withdrawn records stay
  visible with no source link. Historical period views are bounded by their end
  and never write awards. Current views atomically merge evidence with stored
  acknowledgements; only today's new records are marked new. Explicit
  acknowledgement clears that mark. No streaks, no awards for app opens, wear
  or physiological scores. The current record list is not yet a complete
  archive of superseded dated achievements.
- **Adapt**: today's circumstances come from the local records. Rest covers
  the local calendar day and, after midnight, the night until the next waking
  day (late night, or while the night hasn't synced). A pause covers its dates
  and stays open until resumed; illness is a pause with that reason. While
  resting, ill or paused, or with limits set (excluded activities or a movement
  note), Today has no effort call or Garmin call note and its focus shows no
  effort. The control is
  labelled Ill, Paused, Rest day, Limits on or Adapt today, in that order.
- **Today's lead**: live Today only (never a historical read), gates in this
  order, first match leads: a night that hasn't synced waits (no advice);
  evening, wind-down and late night lead with tonight's sleep (no exercise);
  illness or a pause (resume from Adapt); rest; the fixed layout's recorded
  day; a recording today (the latest, skipping walks once goals exist), with
  its strength count against the goal or Garmin's intensity minutes; Garmin's
  scheduled workout today unless dismissed (limits make it "Review needed",
  since its sport and movements are unknown; Garmin's easy or rest call makes
  it "An easier day may fit"); your planned session today (a movement note
  asks for review; an excluded sport is labelled); no readiness: last night's
  sleep against its need; limits: no training suggestion; "Not today" for the
  day's suggestion: nothing else; Garmin's push call: training is an option, to
  plan, dismissable; otherwise an easier or steady day naming the worst
  Garmin-rated readiness factor. Each lead carries a stable id (dismissals key
  on it), its action and the gates it passed, for Why this.
- **Backlog**: only for the current week, and empty while today is paused or
  ill: each goal's remaining amount, as one strength session per item, or one
  item of the remaining minutes or nights. A cardio item suggests the endurance
  sport recorded most in the last eight weeks. Nothing carries over between weeks.

- **Gear snapshots**: `/gear-service/gear/v2/list` is a complete array;
  `distanceUsedMeters` and `maxUsageDistanceMeters` are metres. Check at most
  daily within sync's request budget and backoff. An empty valid list replaces
  the old list, including retired or deleted items; 404, failed or malformed
  fetches retain the last good snapshot. No request on navigation. Zero
  threshold means no distance threshold, not a retirement verdict.
- **Weight history**: Garmin's weigh-ins are grams; display kg or lb to 0.1
  from the account's measurement system. For repeated weigh-ins on a date,
  keep the latest timestamp. The 12-week trend shows the last recorded value
  in each week, without averaging or carrying it across gaps. Latest is read
  at most daily, recent ranges weekly and older years in budgeted ranges;
  empty results count as checked. No weight-based verdicts or notifications.
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
- **Status snapshot**: readiness and sleep follow Today's displayed day;
  Body Battery and the latest measurable stress sample come from the current
  calendar day, never yesterday during the night handover. The daily stress
  average is not a latest sample. An absent value remains null, including
  latest heart rate when the shared all-day HR timeline has no measured sample. "As of" is only the selected
  watch's upload time, never the snapshot write or API fetch time; include its
  date so an old reading cannot look like today's. No separate stale-score
  verdict or expiry threshold is added.
- **Compact focus**: reuse `TodayFocus` and its phase rules. Wake, daytime
  and fixed layout show the suggested Rest/Easy/Steady/Push effort and the
  same lowest Garmin-rated factor as Today. Without readiness, wait
  neutrally; the shared unsynced-night wait has no effort or reason.
  Evening, wind-down and late night use Today's headline and the shared
  sleep plan's reason, with no effort after wind-down. Outside those phases,
  today's circumstances (Rules: Adapt) replace the effort word and reason:
  "Ill · goals paused", "Training paused", "Rest day", "Training suggestions
  limited", or "No training suggestion today" after Not today, opening Today.
  The limits text never reaches a surface. A local change republishes the
  snapshot offline. Native surfaces
  describe the last snapshot, not a live watch reading. Offline widget
  updates, clock changes and sync re-evaluate the phase; Android may defer
  delivery, so transitions are not exact alarms.
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
- **Heart-rate chart**: the compact card averages only present samples in
  16-minute bars for legibility; the detail plots the two-minute readings.
  An empty bar remains a gap, not a zero or an inferred resting value.
  Use Garmin's UTC start and completed-day end as elapsed bounds: a source day
  can span 23 or 25 hours. A partial day reserves a 24-hour axis (and extends
  it if later real samples require it). The axis explicitly says elapsed
  hours from day start, not wall-clock time; source-local travel/DST clocks
  cannot be reconstructed from a single start offset. Sleep and activities
  use their GMT instants, or local-only times only when start/end source
  offsets agree for a completed day. Their intervals clip at day bounds.
  Ungraded heart rate uses neutral ink, not a poor-status color.
- **Trends**: one value per Garmin day, from the recent per-day rows and the
  history rows behind them. Readiness is the morning score; Body Battery the
  day's high; acute load the day's last readiness snapshot; vitals use Health
  Status where Garmin sent it, otherwise the nightly record, read from the
  same fields as the Sleep and Health screens (Sleep HR is Garmin's overnight
  average and never stands in for resting HR). Garmin's
  invalid placeholders are missing (zero remains valid for stress and scores). Up to 90 days shows days; up to
  two years, weeks (Monday to Sunday); longer, calendar months, the last one
  ending on the selected date. 1Y shows 52 Monday-start weeks through that
  date, including the partial last week. Custom blocks use consecutive seven-day
  buckets from the chosen start, or months clipped to the chosen start/end.
  A week or month's value is the mean of its recorded days,
  never counting missing days as zero; one with none is a gap. A period's
  average is the mean of all its recorded days, not of the bucket averages.
  The previous period is the same span just before (the 52 weeks before, for
  a year), or the chosen equal-length block; the change is this average minus
  that one. All starts at the
  metric's first recorded day. Every supplied date (end, block start,
  comparison start) lies between 1990-01-01 and today, and a comparison block
  must end by today; otherwise the chart says which date to change instead of
  drawing an empty range.
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
- **Pace** (earlier weekly plan; kept for its history and the CLI, not shown
  in the app): a target is behind when its share reached (value ÷ target) is
  below the share of the week's training days that have passed: days before
  today that aren't rest days, over all training days. Today hasn't passed,
  so a session later today is never late. Asleep on time uses nights instead:
  nights already counted over the nights in the week that weren't left out.
  Done means value ≥ target. "On track" counts measurable targets that
  aren't behind; the check-in counts targets reached.
- **Completion and history** (earlier weekly plan; the check-in plan card and
  the Week review now read the local goals): each target's completion is value ÷ target,
  capped at 100%; a week's completion is the average over its measurable
  targets. Paused weeks and weeks with no measurable targets have none. Each
  week is measured against the targets in effect that week; weeks before the
  first configured plan are unconfigured gaps in history and check-ins.
  Changing targets later never assigns new targets to those earlier weeks.
  Check-ins distinguish All reached, Partly reached, No targets reached,
  Partial evidence, Not enough evidence and Paused. All reached requires
  every enabled target to be measured and reached.
- **Weekly focus** (the weekly check-in's closing card only; Today's lead
  follows its own rule): the biggest gap among sleep against need, load-focus
  shortage, falling readiness, and intensity minutes against Garmin's weekly
  goal for the plan week that just ended (without them, Fitness Age's
  vigorous-minutes average against its target).
- **Threshold display**: Garmin's lactate-threshold `speed` is seconds per
  meter; convert to seconds per kilometer or mile with the saved Garmin
  measurement system. HR is bpm, running power and cycling FTP are watts,
  heat acclimation is a percentage, and `altitudeAcclimation` is an
  acclimated elevation in meters (feet for statute units), not a percentage.
  Display each reading's own recorded date rather than the fetch date.
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
  tolerance, personal records, all-day heart-rate samples and Garmin daily
  resting HR, and activities (summary, splits, heart-rate zones, charts and
  GPS routes), plus strength sets for strength activities.
  Weight and body composition use `/weight-service/weight/latest?date=` and
  `/weight-service/weight/dateRange?startDate=&endDate=`; raw and per-day
  readings stay local. Latest running threshold HR/pace, running power,
  cycling FTP and heat/altitude acclimation are cached daily, including empty
  and 404 responses, within the shared request budget and backoff; a 404 or
  failed check preserves a previously recorded reading. No historic per-day
  threshold scan is made. The weekly reduced user-settings preferences supply
  the measurement system for units; the full profile is not stored.
- **Strength**: sets come from `/activity-service/activity/{id}/exerciseSets`
  for activities Garmin types `strength_training`: once, then at most hourly
  for 48 hours after the start to pick up early Connect edits. Later edits
  are not fetched by Refresh or reopening a workout; help states this limit.
  A budgeted explicit detail refresh remains a follow-up. Older strength
  activities found by the history download get their sets after it. Once
  there are sets, weight or a recorded threshold/acclimation response, the
  user settings (`/userprofile-service/userprofile/user-settings`) are read
  weekly and only the measurement system, time format, start of week and
  gender (the body-map figure) are stored; the rest of `userData` is discarded.
  Garmin's public exercise catalog (only when sets exist)
  (`connect.garmin.com/web-data/exercises/Exercises.json`, no credentials) is
  read monthly and only each exercise's primary muscles are kept.
- Each sync fetches the last 90 days day by day, then spends its remaining
  request budget downloading older history in range requests, newest first
  and resuming where the last run stopped, back to the account's oldest
  data. A new phone downloads it all again; there is no Raring cloud. Older
  days keep daily values; Settings shows how far back the history goes.
  Trend charts read only this local history. No Health Status range was found
  in the web GraphQL inventory or probes; older nights' vitals come from the
  nightly records, without extra per-day requests. Respiration can differ
  slightly between Health Status and the nightly record.
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
- **Android account boundary**: the existing `SyncGate` owns credential
  suspension, verification, vault save and activation. Like the desktop credential
  gate, it numbers sign-in attempts and invalidates them on begin/sign-out.
  The WebView callback carries the attempt captured at begin through ticket
  exchange. Ticket HTTP runs outside `SyncGate`; a stale completion (including
  an error) is discarded under the gate before touching the active session.
  Android generation changes and profile verification/promotion hold that same
  gate, so the generation cannot change during the profile request. Auth failure and a new
  sign-in suspend presentation immediately, clear notifications/widgets and
  stop reminders without erasing history. Before saving candidate credentials,
  Rust reads `/userprofile-service/socialProfile` fresh with that candidate and
  binds its numeric `profileId` (the user-profile identifier, not the separate
  social `id`, display name or full name). This field's meaning is supported by
  public client source, not a real-account switching experiment. The first
  migrated session is also verified once; ordinary sync and surface refresh
  do not add identity requests. Same-account reauthentication preserves history,
  targets and plan snapshots. A verified different account switches profile and
  clears account-bound rows, settings, cursors and notification history in one
  SQLite transaction, before activation—even if its first data sync fails.
  Failed/ambiguous verification keeps the old history suspended and candidate
  credentials unsaved. Unidentifiable legacy history requires explicit sign-out,
  never an inferred identity from a name. The native snapshot includes its account
  binding; Rust rejects a stale file for another or unverified account. Android
  status/view commands also require a verified active session. One short Rust
  barrier covers local view/plan/settings mutations, snapshot construction and
  account transitions; profile HTTP waits never hold it. Queued settings saves
  reject a changed account. A credential-free native suspension tombstone is
  checked by every Android consumer, including views and bedtime, even if SQLite
  suspension fails. Vault deletion failure is reported, invalidates the retained
  ciphertext's keystore key and cannot trigger automatic reauthentication.
  Explicit fresh verification is required to resume. Desktop retains the shared
  `DesktopCredentials` boundary from `c1e3c9f` for sync, on-demand reads and
  account changes; no second desktop credential gate is added. Account activation/pending markers are excluded from
  exports. Android passes the expected account into archive construction: the
  first read inside the same SQLite transaction checks active identity, then
  verifies the cached profile before any ZIP bytes are created. Every archive
  read uses that pinned transaction, preventing A→B→A account switching from
  exporting B's rows for an A request. Final identity validation and the atomic
  no-replace rename hold the existing short local session barrier together;
  ZIP construction and profile HTTP never hold it. The native share path also
  checks the verified account under `SyncGate`. Historical/CLI exports remain
  an explicit separate, unbound operation.
- **Preference updates**: production UI controls send only changed fields via
  `patch_settings`, not a cached full settings object. Rust validates names and
  types against `Settings`, then merges with current stored values inside the
  existing account/write barrier. Writes to the same control are disabled while
  pending; unrelated controls remain usable. Settings reads are revision-guarded
  and the one-time status offer writes only its marker. A snapshot publication
  failure rolls back only a new persistent-status opt-in, never other changes
  or revocations. Both Health's journal allow button and Settings' opt-out send
  only `journalEnabled`; false stays authoritative even when status snapshot
  publication fails and unrelated older patches complete later. Journal content
  and consent are excluded from the native snapshot and notification/widget
  allowlists. Full-settings replacement is reserved for
  explicit fixtures/CLI, not the UI transport.
- **Native status snapshot**: after every foreground/background sync attempt
  that reaches the core, Rust atomically replaces a private, versioned JSON
  snapshot beside SQLite. It includes nullable readings, tones, `TodayFocus`
  phase and compact focus, the displayed date, write time and watch upload
  time. No endpoint or request is added; a snapshot write failure is reported
  separately and retried offline, never by repeating a Garmin sync. Kotlin reads and draws it using
  RemoteViews; neither rendering path starts the WebView. Settings and
  native lifecycle events can ask Rust to refresh it offline. Sign-out erases
  it and clears the surfaces; unsupported/corrupt versions render blank.
  Keep the existing hourly WorkManager sync and its existing 15-minute
  missing-readiness morning follow-ups, not a new notification poll or
  foreground service. Widget host updates request an offline refresh every
  30 minutes; reboot, timezone and clock changes also refresh locally.
  Emulator accelerated refresh measurements showed no usable battery-drain
  signal, so there is no evidence for increasing Garmin polling. Actual
  battery/radio cost and Doze delivery remain phone checks.
- **View cache**: after the first view renders, the five tabs and Sleep are
  prefetched and cached by route for the current day. A completed sync, a
  changed sync timestamp or a local day rollover invalidates in-flight reads
  and the cache without clearing the visible view. Status is read on start,
  resume and sync, not before every navigation, and is applied independently
  of navigation so a tab switch can't discard a newer sync or sign-out. View
  commands read SQLite off the WebView thread. Progress keeps its selected
  period and historical anchor with the cached page across detail/Back and
  resume/sync refreshes. Selection revisions reject older in-flight results;
  sign-out resets the selection. Its local calendar date remains separate from
  the selected period's end so forward navigation still works. Today carries the end of its
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
  exists. Status notifications and widgets ship on Android only; their
  versioned snapshot is platform-neutral JSON produced entirely in Rust,
  so later iOS widgets can consume the same contract with a native renderer.
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

After v1: iOS, desktop apps and web app. Garmin lifestyle logs are available
read-only in Health with explicit local consent.

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
