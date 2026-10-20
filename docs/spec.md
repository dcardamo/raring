# Raring v1 spec

Raring turns Garmin data into a clear daily answer: how am I doing,
why, and what should I do today, readable at a glance from dials, charts and
color. It shows Garmin's own metrics and nudges you to improve. No data dumps,
no invented scores.

Android first. iOS and desktop come next, then web.

The visual reference is the approved redesign's captures in
[design/mockup/experience](../design/mockup/experience/) (`shots/`), made with
synthetic data. `design/mockup/index.html` shows the app before the redesign.

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
  The manifest names the archive format's version and the Garmin account that
  made the export (a numeric identifier, never a credential), so another phone
  can validate an import and refuse a file from a different account (Rules:
  Import).
- **Its own identity.** Own name, palette, icons and copy. Garmin's metric
  names describe Garmin data. Nothing copied from other fitness apps.

## Look

- Warm graphite background (`#111317`), opaque charcoal cards (`#1B1E24`)
  and raised controls (`#242830`, selected `#303641`). No decorative status
  glow. Warm-white ink (`#F4F1EA`), supporting text (`#C2C8D1`) and muted
  text (`#A1ABB9`) give readings priority over chrome. Tokens live in
  `app/src/app.css`; the chart palette in `charts.ts` is checked against it.
- Color means something or it isn't used. Icons and titles say where you
  are; color says how it's going:
  - **Status**: mint green good (`#60D3A0`), ochre fair (`#F2B85E`), coral
    poor (`#F17E78`). In text a status colour comes with a word or symbol;
    on charts colour alone carries it, with no colour key. Missing data
    is a gap, never a zero. Amber is attention, such as a reading outside its
    usual range; coral is a poor rating.
  - **Effort**: heart-rate zones run gray, blue, yellow, orange, red. Red
    means hard.
  - **Neutral**: one muted blue (`#76ACEE`), faint to strong, for amounts and
    readings without a verdict, such as sets per muscle, weight, VO₂ max and
    time asleep. A chart's readings take status colours or this blue, never
    ink (Rules: Chart colours).
  - **Ink**: text, controls and reference marks such as your age, goals and
    the period before. Primary buttons use warm-white ink as their fill with
    dark text; selected controls use a raised neutral surface. Time-range pickers keep a light selected
    segment with dark text. The selected bottom tab has an icon backing,
    a stronger label and an accessible current-page state, not color alone.
- Text is at least 4.5:1 against its surface, including large text. Text on a
  colored fill is black or white, whichever reads better.
- Gauges are solid arcs open at the bottom, colored by status. Load gauges
  mark Garmin's optimal range as a neutral band, with a dot for the current
  value.
- A small radius vocabulary: 16 px cards, 10 px inner controls, 6 px small
  marks. Spacing follows 4/8/12/16/24/32 px, with 16 px card insets. Inter for
  root and card headings and body text; Outfit for tabular readings and compact
  domain headings. Root headings are 30 px, card headings 14 px sentence case,
  metadata 11–12 px. The lead remains larger than its evidence.
- **Goal rings**: one ring per weekly goal, nested in a fixed order with a
  fixed colour each (strength pink, Zone 2 blue, Zone 4–5 orange, intensity
  teal, sleep timing pale blue). A ring draws only recorded work, with no
  track for the rest; moon for rest, hatching for ill or paused, dots for
  awaiting data (Rules: Rings). Every ring fits: strokes thin as goals are
  added, and no ring's radius is under 3 px. Each grid of rings has a colour
  key of the goals it draws; nothing is written inside a week ring.
- **Done, planned, not recorded** look the same everywhere: done is filled
  with a check and its recorded stats, planned is outlined with its planned
  time, passed and not recorded is muted.
- **Garmin's items** (scheduled workouts, Garmin Coach) carry a small ◇ and a
  "Garmin · read-only" label, beside your own plan in the same days.
- The effort scale is one continuous bar of joined chevrons; it is never shaped
  like buttons. All four levels share one rail with aligned seams. The
  suggested level is drawn in full colour as the same chevron, taller above and
  below the rail, with its label in bold; the others stay muted on the rail.
  Selection never moves a seam or a label.
- Weight is a neutral metric: the neutral blue, laid out like every other
  history card; its changes are directions, never good or bad colours.
- Every chart has axes, units and labelled values; a usual range is a band,
  a goal a dashed line. Amount rows share a zero origin and scale, with the
  value outside the bar, a labelled tick for a comparison mean or an outlined
  band for Garmin's target. Unknown amounts are not filled; measured zero
  stays zero. Duration rows name the clock format instead of adding percentages
  with an unclear denominator. Period plots use round ticks; count ticks are
  integers. Gaps are drawn as gaps. Shared single-metric charts use
  2 px main traces, quiet 1 px gridlines and green-tinted usual-range bands
  with dashed edges; range bars (vitals, drivers) tint the usual range or
  Garmin's target green under the reading's dot; goal lanes
  keep their labelled goal color. Owned line icons use a 1.75-unit stroke.
- Chart text is at least 11 px for axis ticks and 12 px for other labels on a
  390 px phone, never overlaps other text or a marker, and stays inside the
  screen at 360–1100 px and at larger system text. Collisions are resolved by
  layout: units and value scales sit outside the plot, and tick labels give
  way (thinned in a fixed order) rather than shrinking.
- Reduced motion (the system setting) stops every animation and transition.
- Phone first; an unfolded or desktop window adds columns without changing
  the order.

## Screens

Bottom tabs: **Today · Plan · Training · Health · Progress**. Each root header
has the date, the watch's upload time, **Adapt** (labelled with today's
circumstance), search (pages, cards and Garmin metrics by name, with plain
spellings such as "spo2" and synonyms; a card result opens its page scrolled to
it; "resting heart rate" finds Garmin's daily resting HR, and Sleep HR appears
under its own name only when Health Status supplies it) and a profile button.
Sleep opens from Today and Health. Tapping a dial or card opens its detail;
Back closes an open sheet first, then returns to the page that opened it,
keeping its section, period and scroll. Short "?" sheets explain a page or
card in shipped terms. Earlier links still open their current page:
`#checkin` the latest weekly review, `#plan-setup` the weekly goals,
`#explore` Training, `#metric/weight` the weight detail; a link this version
doesn't know opens Today and says so. A page that opens one past day or
workout names its date or activity in the route (`#sleep/YYYY-MM-DD`,
`#health/YYYY-MM-DD`, `#metric/stress/YYYY-MM-DD`,
`#metric/heartRate/YYYY-MM-DD`, `#workout/ID`); Back returns to the page or
section that opened it, and `sleep`, `sleep/last`, `sleep/tonight`, `health`
and `metric/stress` keep their current meaning.

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

A briefing with an ending, in this order:

- **Last night and readiness**: four compact stats in one row (two by two
  at large text), each opening its detail: **Sleep** (time asleep in a ring
  against Garmin's need, the Sleep Score under it; opens Last night),
  **Readiness** (Garmin's score in a ring in its tone, the level word under
  it), **Sleep HR** and **HRV** (last night's value with ▲ or ▼ and "6
  above usual", "Within usual", "Usual not set yet" or "Garmin calibrating";
  they open their metric detail at 30 days; Rules: Night vitals on Today). A
  tone colour appears only with its words, when Garmin rates a reading low
  or it is outside its usual range in the unwelcome direction. A watch that
  can't record readiness says so. While the displayed night hasn't synced
  all four wait neutrally: no earlier score, readiness, vitals, chips or
  reconciliation shown as this morning's. A missing reading is a dash and
  "Not recorded". From the evening on only Sleep and Readiness stay. A score
  without a duration is labelled as one.
- **The lead**: one headline, at most one line and one action, chosen by the
  gates in Rules: Today's lead; a suggestion can be dismissed with "Not
  today". When Garmin's readiness allows, the **effort** strip (Rest, Easy,
  Steady, Push) marks the suggested level, with one reason line from
  Garmin's inputs under it. It is a display, not buttons. When a session
  leads, the headline is about the day ("A steady day fits"), never the
  session's name.
- **Today's session**: a separate card below the day's context, with the
  session's name, its source (◇ Garmin Coach with its week, ◇ Garmin
  calendar, or Your plan), what it is in plain words and how it compares
  with today's suggested effort (Rules: Today's session). **Keep · Choose
  another** sit inside it (Rules: Today's choice), so they act on the
  session. There is no effort scale
  beside a recorded workout (the day can finish) or from the evening on.
  Evening, wind-down and late night lead with tonight's sleep (the evening
  and wind-down cards carry the same sleep clock, compact, beside the
  wind-down, in-bed and window times) and keep any
  session still planned as a quiet line; beside tonight's plan card the hero
  keeps only the headline, since the card carries the window and opens
  Tonight.
- **Your day**: sleep stages, Body Battery, stress, heart rate and recorded
  activities and naps on one 00–24 axis with a now marker (Rules: Your day).
  Each row's value sits at its right (Body Battery's is the latest level,
  the day's daily-stats value when Garmin sent no intraday levels), with a
  nap's Body Battery change under it; heart rate is drawn against the day's own lowest and highest readings
  and never clipped, with no scale on the plot.
  Each row opens its detail. From the evening on it follows tonight's plan,
  compact: sleep, Body Battery and recordings. A calendar day with nothing
  recorded yet (after midnight, or before the night syncs) hides it. The
  upload time is said once, in the header, never repeated in a card.
- **Recorded today**: one list of everything the calendar day (Your day's)
  recorded, workouts and naps together in the order they started, directly
  under the day's headline while the day leads. Workouts carry Garmin's benefit
  and training effect (one decimal); naps their Body Battery change. Up to four
  rows, then a link to all of today's activities. The headline never repeats a
  workout from this list. Under a "Done today" headline the list goes untitled. Compact after tonight's plan: without the effect
  labels.
- **This week**: the week's rings and one line per goal beside **Left this
  week** with Do it today (not from the evening on: a weekly gap is context,
  not a command to catch up tonight), over the day cells; hidden during
  wind-down and late night, with the weekly check-in link and the
  plan-your-week prompt. Without weekly goals it says so.
- While older history is still downloading, the upload line adds "History
  since" its first month.
- Watch-upload and missing-reading notices sit above last night and
  readiness; Today explains only the readings it shows (readiness and the
  sleep score). The ending
  says "That's today." The plan-your-week prompt appears on root pages when
  due (Rules: Plan prompt). Settings → Today → "Follow my day" keeps a fixed
  order for irregular schedules (Rules: Time of day).

### Readiness

- Large gauge with Garmin's level (Prime/High green, Moderate amber, Low/Poor
  red) and a few-word headline from Garmin's own feedback. A watch that
  can't record readiness says so, with no headline.
- Factor bars worst first, each with Garmin's rating word beside its
  factor level on the 0–100 scale; the value Garmin used (when supplied)
  and rating lead each row. Tapping one explains it in the Why sheet's
  sentence. Factor levels are not shares of the readiness score. A bar's
  color is Garmin's rating; a percentage without a rating keeps its number
  but stays neutral. A factor Garmin didn't rate (it sends NONE, for example
  without valid sleep) is a gap, never 0.
- A Why sheet lists the six factors worst first, each with the value Garmin
  used (sleep score, recovery hours, 7-day HRV, acute load) and Garmin's
  rating, under one line naming what holds readiness back and what helps.
- Readiness trend chart and today's call: Today's effort bar and reason
  line, shown when Today shows them. Tap the chart or use the date
  slider to inspect a reading. Dates are anchored to the stored Garmin day,
  missing days remain gaps, and daily bars use Garmin's level boundaries (50
  for Moderate, 75 for High).

### Sleep

Two tabs, **Last night** and **Tonight** (Rules: Sleep tabs).

Last night:

- Score gauge, and a time-asleep bar with a marker at Garmin's sleep need
  for that night (when the watch reports it), labeled as last night's need.
  The scale fits both amounts; the difference is neutral, not a more-is-better
  verdict. A missing duration has no fill.
- Stage timeline (awake, REM, light, deep) with whole-hour ticks, and in the
  same card aligned duration rows in that order. A tick marks each stage's
  recorded mean. One small subheading says “Ticks: your 30-night average”
  with the recorded-night count (distinct counts are listed when stages have
  different coverage). Each bar keeps only its average value alongside it;
  there is no trailing comparison note. Each stage's accessible label names
  its duration, average, recorded-night count, source and window end (Rules:
  Sleep stage comparison). The mean is descriptive, never a target or rating.
- Overnight tiles with lines: HRV, heart rate, respiration, SpO2, each with
  the night's start and end times and its low and high, opening the metric's
  detail.
- **Recorded naps** from Garmin's nap endpoint: dated local start/end from
  GMT timestamps plus per-end offsets (seconds), `napTimeSec` in seconds,
  and optional Garmin `napFeedback` without interpreting its enum. Display
  both local dates with the clocks, including when a nap crosses midnight;
  label missing start/end without reconstructing them from duration.
  Show duration separately from main overnight sleep; never add it to sleep
  score, asleep minutes or Raring's sleep plan. Garmin's own `napAdjustment`
  for tonight's sleep need is distinct. An empty array is no recorded naps;
  absent ("not checked yet"), failed and stale reads have separate neutral
  labels. Other populated
  nap fields and feedback meanings require further verification before display.
- **Consistency**: 14-night chart with your usual bed and wake windows
  shaded and their clock ranges in the key, each week's Monday marked, a
  "Bedtime spread" pill, and Garmin's seven-day average bedtime and wake time
  above it when synced. No rhythm score or chronotype of Raring's own.
- **Sleep history**: Garmin's sleep score as a trend chart with round
  ticks; time asleep is one overlay away, and other comparisons are in
  Health.
- **Sleep alignment**: Garmin's status (Aligned, Ahead, Behind, Jet lag) and
  last night's sleep against Garmin's optimal sleep window on an overnight
  timeline, with the last seven nights below. Garmin's window is a dashed
  outline, never shaded like your usual windows, keyed "Garmin's
  longer-term window (dashed)". Without a
  rating for last night, show the latest one with its date; say when Garmin
  is still calibrating, and otherwise that alignment is unavailable.

Tonight:

- A 24-hour sleep clock as the hero, midnight at the top: the planned
  sleep from the asleep-by time to the expected wake (asleep-by plus the
  need) as a thick arc, led by a lighter segment for the minutes to fall
  asleep from in bed; the wind-down as a softer arc before in bed; Garmin's
  window as a short emphasised segment just outside the ring; and a now dot.
  Inside, only the need and two lines ("In bed 22:51", "Up 06:56"); hour
  ticks 00, 06, 12 and 18 sit outside. Below it the facts once: the
  wind-down (Raring's, with its lead), Garmin's bedtime window or Raring's
  asleep-by estimate, Garmin's need with when it was read and "so about
  8:05 in bed with 15 min to fall asleep (Raring)", and the usual wake (from
  four recorded wakes in 14 nights, with the count until there are 14,
  otherwise a "reference") when it differs from the expected wake; the
  sleep goal when one is set; and the time-to-fall-asleep setting. "Why
  this plan" lists the evidence:
  who set the bedtime, Garmin's need, baseline and adjustments, the
  wind-down lead and one tip toward steadier timing. A suggestion, never an
  alarm; it never changes the watch's sleep schedule (behaviour, not copy:
  no disclaimer says so).
- **I wound down** (a self-report for tonight's evening) and the wind-down
  reminder switch. With a sleep-timing goal, "Make tonight one of them"
  places tonight's wind-down session.
- This week's sleep goals, and **Recent nights vs tonight's window**: the
  14 nights ending last night as bars from Garmin's recorded sleep start to
  end on a clock axis (evening at the top), against tonight's bedtime band
  (Garmin's window start or in bed, whichever is earlier, to asleep-by) and
  tonight's wake as a dashed line; a missing night is a dotted mark, in the
  key. One line says how many of the last seven recorded nights were below
  that night's own Garmin need. No running debt. Each night opens that
  night (Rules: Opened past days and workouts).

**A past night.** The seven-night bars and Sleep history's selected day open
`#sleep/YYYY-MM-DD`: Last night for that date, with no Tonight tab and no
plan for a night that has passed. A night this phone has never stored is
fetched once when opened, inside the shared budget and backoff; the page
says while it loads, and says when Garmin has no record for it. Today's
night is never shown in its place.

### Training

- **This week**: lanes per goal kind against the goal and your usual week
  (Rules: Week lanes), and recorded minutes by sport. Without goals, one
  cardio lane.
- Three sections, the choice kept across a detail and Back:
  - **Fitness**, around three questions. *Enough, or too much?*: acute load
    as a dot on a compact bar with Garmin's optimal range as a neutral band,
    training status with its Why sheet, load focus as aligned amount rows
    against Garmin's target ranges (printed), and a compact acute-load history
    chart starting at 30 days against the explicitly current optimal range;
    Compare and all history opens the acute-load metric detail. *Is my fitness improving?*: VO2max with its 30-day change and
    history, endurance and hill scores, running tolerance. *What am I ready
    for?*: Garmin's race predictions, thresholds and the next race. Garmin's
    next scheduled session is a read-only card.
  - **Strength**: this week's sessions against the goal and sets, a small
    muscle map over a chosen range with gaps named (Rules: Muscle gaps; the
    range is shared with Strength), and a link to Strength.
  - **Activities**: every stored recording with Garmin's primary benefit,
    training effect and intensity minutes as one text line, filterable by
    sport. The last 31 days open instantly; Show older pages back through
    stored history (filtered by sport before the page limit) without a
    Garmin request. Today's "All today's activities" opens this section
    narrowed to that day; the old `#activities` link opens Training.
- Latest recorded Garmin running lactate-threshold HR and pace, running threshold
  power, cycling FTP, heat acclimation and acclimated altitude in a compact
  Training card, each with its own recorded date. Missing, null or stale
  thresholds are omitted; an explicit heat 0% is shown. Availability depends
  on the watch and recorded activities. No thresholds are estimated here.
- Gear lives in Profile.

### Workout

- Training load, aerobic and anaerobic Training Effect gauges, primary
  benefit and recovery time.
- Stats grid, route map, heart-rate chart colored by zone (bpm axis, each
  zone's floor labelled, the average marked, round minute ticks), time-in-zone
  rows with each zone's bpm range and recorded duration on a shared zero-based
  scale (no percentage column); missing/invalid zone durations say Not recorded,
  while an explicit zero reads 0:00. An incomplete set of zone durations has
  no normalized overview strip. Split bars have the fastest
  highlighted: rides by speed, swims by pace per 100 m or 100 yd, everything
  else by pace; a lap's distance shows only where laps differ. Power, cadence
  and running dynamics when present. Heart rate is drawn once: at the top of
  Along the way when streams exist, otherwise in its own card.
- **Units**: distance, pace, speed and elevation follow Garmin's measurement
  setting (km, /km, km/h, m or mi, /mi, mph, ft); swims read in the pool's
  unit (m or yd) with pace per 100. Garmin's swim fields (pool length,
  lengths, SWOLF, strokes per length, stroke rate, distance per stroke) show
  under Recorded by your watch when sent.
- **Strength activities with sets** replace the effect gauges and stats with
  the sets: a hero (sets, time, reps, load, average heart rate, records), a
  card naming the sets that look off, a body map of this session, and one
  card per exercise in the order done. Each card shows Garmin's name and the
  muscles, a bar per set (reps above, weight below with its change against
  the same set last time, flagged sets amber, record sets starred) over a
  dashed outline of last time's set, and last time's sets as text. An exercise Garmin couldn't name is "Not recognized".
  The heart-rate chart and zones follow. A session timed without lap presses
  says no sets were recorded and links to help. A ? opens a sheet on how sets
  are counted. A card on the workout says when Garmin's sets were last read
  and offers Refresh from Garmin (Rules: Explicit strength refresh). Raring
  never changes the sets; fixing them is a later Garmin write.
- **How it counted**: the workout's share of this week's goals, the plan
  session it completed and any milestone (Rules: How it counted).
- **Along the way**: the stored streams this activity has (pace or speed,
  elevation, cadence, power, breathing rate, stamina, performance condition,
  watch temperature) on heart rate's time axis; **What it did**: Garmin's
  training effect with its band in plain words, intensity minutes, Body
  Battery change and stamina; **Recorded by your watch**: the other summary
  fields it has (Rules: Workout detail).
- An activity the history download listed but never detailed opens with its
  stored summary and fills in its charts, route, splits and zones once when
  opened (Rules: Opened past days and workouts).

### Strength

Opened from Training → Strength: lifts and muscles.

- Body map over a chosen range (7D–Max) with gaps named and sets per muscle
  group listed (Rules: Muscle gaps); shading scales to the range's busiest
  group.
- One exercise at a time (chips on the phone, a list with sparklines on
  desktop): Max weight or Volume each session over a chosen range (Rules:
  Lift volume), the best weight in available history at 1,
  3, 5, 8 and 10 reps, and its sessions (heaviest set, reps of every set,
  record star), each opening its workout. Bodyweight exercises show their
  most reps in one set instead of a weight (Rules: Records and bests). Show-all controls expose every
  exercise session and workout in the supplied history, not just the initial
  six sessions or eight workouts. The strength workout list starts at the
  same first-week boundary as the chart and filters by sport before any
  pagination limit; newer workouts in other sports cannot hide strength
  activities.
- Sets per muscle this week as aligned amount rows, with a tick and value
  for the previous four weeks' weekly average, the strength workouts, and a
  Figure control (Profile, Female, Male) that overrides the profile's figure for every body map (Rules:
  Body-map figure).
- Phone: one column. Unfolded (660 px+): two columns. Desktop (1000 px+):
  exercise list, exercise detail, muscles.

### Body map

Raring's own front and back figures, female or male from the Garmin profile
unless Strength's figure picker overrides it, with muscle groups as separate shapes: chest, back, shoulders, arms, core,
hips, quads, hamstrings, glutes and calves. Each is shaded in the amount blue
by sets, gray when none.

### Help

Short offline pages, opened from Settings, the "?" sheets, the ? on Strength
and the strength workout sheet: lifting with the watch, how sets are counted,
the body map and which watches work. The "?" sheets on pages and cards say in
two or three lines what a number is, where it comes from and how to read it,
and link a Help page only where their area has one. Each area ships its pages with the feature.
Questions go to GitHub; the address is shown as text.

### Health

In this order: long-term health (Fitness Age against your age, opening its
history, and its drivers with the biggest lever), this morning's vitals, then resting heart
rate, overnight HRV, average stress and Body Battery's daily high as compact
range charts with the usual band, weight, intensity minutes and the journal.
Health summarises: each chart keeps its own metric, offers 7D, 30D, 90D and
1Y, and a tap reads a day; one link opens the metric's canonical detail (today
and the full Trends control: comparisons, dates and all history). Your day
stays on Today. A daily metric whose newest reading is older than two days
says so on its chart (Rules: Reading freshness).

- **Garmin journal**: a private, short dated view of the last seven lifestyle
  log days, opened explicitly in Health and off by default until local consent.
  Live Health anchors those seven dates to the actual local calendar day even
  when its vitals still show the retained sleep day; explicit historical Health
  reads remain anchored to their requested date. Only consented syncs fetch
  the daily log: recheck at dispatch after client serialization, token
  refresh, pacing and before any 401 retry; atomically check consent again
  when caching each reply. Closing or leaving Health hides entries.
  Garmin's `dailyLogsReport` rows may carry `name`, `logStatus`
  (YES/NO) and `details` with `amount` and `subTypeName` (no generic unit;
  shown as words, "2 coffee").
  A null report or unknown log status is not a recorded zero/no; completion
  tracking statistics do not prove individual entries. Populated report rows
  are known from community examples, not verified on this account. No new
  scoring or correlations. No journal content in notifications or lock-screen
  views. Opting out stops new journal reads and hides stored rows but does not
  erase them; future user-created exports still include previously downloaded
  logs until local data is erased.
- **Vitals**: Garmin's overnight average heart rate (labeled Sleep HR,
  not resting HR), HRV, respiration, SpO2 and skin-temperature change as
  tappable tiles when Health Status is available. Without it, use the
  existing resting-HR and other daily readings. Show units, the personal range,
  its Garmin or Raring source, and explicit within/above/below-range or
  missing/baseline-building labels.
  Each detail has a trend chart with the current range shaded on daily ranges, actual dates and missing nights left blank. Nightly HRV against the
  baseline is distinct from Garmin's seven-day HRV status. Explain the comparison
  method behind a tap; personal ranges are not medical targets.
- **Stress**: the daily-average range chart, with a link to today's
  three-minute readings (the same detail as Today). The detail shows Garmin's
  duration in each stress level and Active (not rated) as aligned amount rows;
  missing time is not extrapolated to a whole day.
- **All-day heart rate** (opened from the resting heart-rate chart):
  Garmin's daily resting HR (bpm), its history with Raring's usual range
  (Rules: Normal range), the day's minimum and maximum, and a day chart in two-minute elapsed slots, with recorded sleep and activities shaded
  or marked. Include the next morning's sleep as the previous evening's
  interval, clipped to each displayed day. Missing timestamps, null and
  off-wrist readings are gaps, never zero. The latest sample is synced, not
  live. The daily resting value is distinct from Health Status's Sleep HR
  (the overnight average); name both. Today and yesterday refresh with every
  sync under its request budget; older days are fetched only when opened and
  kept, including empty days. Retry failures and 429s follow the shared
  backoff. Older days open from the previous/next day controls on the detail.
- **Weight and body composition**: laid out like Health's other history
  cards, in the neutral blue. The Health card has 7D, 30D, 90D and 1Y, the
  period's dates, trend weight and the period's trend change with its
  direction, the last weigh-in's date/age, the chart with a tap-to-read
  weigh-in (scale and trend), a key and one link to the canonical
  `health/weight` detail (`metric/weight` alias). The detail leads with trend
  weight, then the chart with 7D–Max, the range average/change and a reading
  slider; neutral 1–90-day changes with sparklines (7 days and longer); and
  Garmin's recorded BMI/body fat as labelled values. Scale weigh-ins are dots
  on a faint line, the trend a solid line, axes name the unit.
  Help ("?") explains smoothing. Adding a weight lives on Profile only (Profile and
  Settings). Weight follows Garmin's
  measurement system (kg/lb), not activity-set units in isolation. Missing
  measurements stay gaps, never zero; no notifications or unsolicited weight
  targets. Until the account units sync, the card waits rather than guessing.
- **Fitness Age**: large number labelled as Garmin's estimate with its date,
  years younger or older than your age, its change in years against a year
  ago, and a bar from 20 to 70 (widened in tens when an age falls outside) with
  a YOU mark for the estimate above it and an AGE mark for your age below it:
  the bar is green younger than your age; older is coral when the estimate is
  older than your age and neutral otherwise, and YOU takes its status colour. The bar and a "Fitness Age history" link open
  `metric/fitnessAge`: the estimate, your age and the difference, then the full
  history chart (opening on the latest estimate) with your age now as a dashed
  line and each estimate coloured against it (Rules: Chart colours).
- **Drivers**: VO2max, resting heart rate, BMI or body fat, and vigorous
  minutes as tiles with range bars, Garmin's target in words ("Target ≥ 150
  min/wk") and Met or how far off it is; no trend arrows. The driver that would
  help most (the biggest lever) adds the age you'd reach at its target. The
  drivers come from the estimate the Fitness Age card shows. Garmin's Fitness Age component owns its
  target and age estimate; a recorded weigh-in can supply a missing BMI or
  body-fat value measured in the last 30 days but never creates a target or
  Fitness Age prediction.
- **Intensity minutes**: this plan week's total as a ring against Garmin's
  weekly goal, with moderate and vigorous minutes; it opens the weekly plan.
- When two or more vitals are outside their personal ranges, an amber
  message suggests taking it easy and checking how you feel, without
  diagnosing an illness. A partially calibrated monitor reports only the
  readings it can compare.
- **A past day**: the Progress calendar and a selected trend point open
  `#health/YYYY-MM-DD`, anchored to that date: its charts and weight end
  there; its Fitness Age (Garmin's estimate as of that date, with that
  estimate's drivers), stress and vitals are that day's; intensity minutes
  are its plan week up to that day, named by those days (a passed week opens
  its review, this week the plan); and the journal's seven days end there.
  The stress detail (`#metric/stress/YYYY-MM-DD`) shows that day's timeline;
  the all-day heart-rate detail (`#metric/heartRate/YYYY-MM-DD`) opens that
  day and reads its heart-rate day on demand. Missing per-day detail (Fitness
  Age, sleep, HRV, stress, the daily summary, Health Status) is fetched once
  on opening and kept, inside the shared budget and backoff; the page says
  while it loads, and says when Garmin has nothing for that day. Today's
  readings are never substituted for an older date (Rules: Opened past days
  and workouts).

### Trends

Every trend chart (Readiness, the vitals, stress, Body Battery, VO2max,
acute load, Fitness Age and Sleep's history card) is the same interactive chart, drawn from local data.
Health's cards use its compact form (Health), except Fitness Age, whose card is
its bar and opens the chart. Garmin's sparse estimates (VO2max,
Fitness Age) open on the latest estimate rather than an empty latest day.

- Shared range control: 7D, 14D, 30D, 90D, 1Y and Max (All history), plus Dates for a chosen training block. Up to 90 days shows each day; longer
  ranges show weekly or monthly averages, and the selected point says how many
  days it holds.
- **Compare**: off, the previous period (same length, faint dashed line on
  the same axis, with both averages and the change), or a second metric with
  its own right-hand axis. Both metrics are selectable; periods without data
  remain empty. Dates selects a block's inclusive start and end; the comparison
  start selects an equally long block, aligned by elapsed day. One comparison
  at a time; All has no previous period.
- The period's recorded-day average leads the history chart, labelled as
  Raring's mean of Garmin readings with its dates and recorded-day count.
  Selecting a point changes only the smaller dated readout below the plot,
  never that mean. A previous-period comparison puts both means, counts and
  the neutral signed change above the plot. This also applies to VO2max and
  Fitness Age: their inspected readout starts at the latest available bucket,
  explicitly labelled as a weekly/monthly average where applicable. No
  deduplicated estimate cadence is inferred. Current bands print their bounds and source. Readiness, Sleep
  Score, stress and Body Battery use a fixed 0–100 domain on either axis;
  other metrics keep round, adaptive ticks.
- Colours follow Rules: Chart colours. Garmin's levels colour days and
  weekly or monthly averages alike, which sit on the same scale. The current
  usual band shows, and colours readings in or out of it, only on daily
  ranges; it is today's range, not each past day's, and is labelled so.
  Without it a range metric's readings are the neutral blue. Overlays, the
  previous period and changes stay neutral, with no new score.

### Plan

The plan week (Garmin's intensity week, see Plan week) at a glance, from the
local goals, your sessions and Garmin's planned work (Rules: Local records to
Backlog, Garmin Coach).

- **Week**: dates, a pause or illness note, the nested week ring and one line
  per goal: recorded of target, then placed, recordings to review and left,
  or only reached once it is; a change during the week shows the starting
  target on a second line. Sleep Score is a Garmin reading against its target.
  Missing evidence says so instead of a number. Below it, one place for what
  needs you: each session needing review, naming its day and the recordings
  it may be, opens its sheet; sessions not recorded are one row (how many and
  which days) that opens them with Reschedule and Let it go. This week of a
  Garmin Coach plan is one read-only line with how many are done; its
  sessions are in the days.
- **Goals** (`#plan/commitments`): steppers for strength sessions, Zone 2,
  Zone 4–5 and intensity minutes (Garmin's weekly goal until changed), sleep
  timing (asleep by or wind down at a chosen time) and an average Sleep Score.
  Zero turns a goal off. Saves apply as the Local records rule says.
- **Days**: day cells with rings and a row of marks (an outlined dot per
  session still planned, a muted hollow dot for one not recorded, ◇ for
  Garmin's, ☾ for a Garmin Coach rest day); a day lists your sessions (an
  unnamed one by sport and time) and Garmin Coach sessions with one state word
  (Done, Needs review, Planned, Not recorded, Awaiting data, Marked missed),
  Garmin Coach rest days (said beside your sessions that day), recordings not
  in your plan (or, when a session may be one, the sessions it may be, opening
  the choice), and Garmin's scheduled workouts by name, read-only and not
  counted. Durations are minutes ("52 min"), never clock-shaped. Add a session for today or a later day of the plan week (Plan
  shows one week; Move and Reschedule offer the same days), named on the
  button ("Place on Sun 19 Oct").
- **A session** opens a sheet: its recording when done, with "Not this
  session" to undo a match; "This was my session" for a candidate; Edit
  (time, name, minutes, goal shares), Move or Reschedule, Let it go, Mark as
  missed (passed days only) or Remove; Reopen; its history. Send to Garmin is
  shown disabled and labelled not yet available. Garmin's sessions open a
  read-only sheet (Rules: Garmin Coach).
- **Left this week**: the backlog with Do it today (a draft of one session
  or a share of the minutes: 45 of Zone 2, 20 of Zone 4–5, 30 of intensity,
  or the whole remainder when under ten minutes would be left), or a paused
  note with nothing to catch up.
- **Events**: local races or events with a countdown (removing one asks
  first), and Garmin's next race; an event with an event week (Rules: Event
  week) shows that sport's recorded minutes this week.
- A migrated sleep-timing observation is explained once, with Review timing.

### Progress

Four sections, the choice kept across a detail and Back:

- **Overall**: the period report (week, month, 3 months, year; previous and
  next; known totals as amounts, never clock times ("19 h 54 min", "278 h" from
  100 hours), and one "Compared with" line with each total's signed change only
  when comparable, Rules: Progress periods; sleep against need for the period;
  activity by sport on a chart with axes, minutes per day or hours per week or
  month, days to come left blank and the current day or bucket outlined), the
  ring calendar for month and past week periods (whole plan
  weeks, days outside the period faint; the current week is Plan's day strip;
  without goals, the activity calendar), the weekly goal rings for the last 12
  weeks (passed weeks open their review and this week opens Plan; a week that
  didn't count is faded, hatched when ill or paused and dotted when data was
  missing) with one link to all reviews, and the latest 12 records and
  milestones (dated with the year when it isn't this one) with a link to all
  of them (`#progress/milestones`, by month).
- **Cardio**: Zone 2, Zone 4–5 (while it has a goal) and intensity minutes per
  week for 12 weeks, each week against its own target as a stepped dashed line
  (intensity following Garmin's goal says "Garmin goal"), and Garmin's VO2max
  estimates on the same weeks (Rules: Progress domains). The current week's
  bar is outlined (its tick stays a date; spoken labels say this week).
- **Strength**: strength sessions per week against each week's goal, with
  whole-number ticks, and a link to Training · Strength for bests and lifts
  (shown there once, with its ranges and measures).
- **Long-term**: training hours, resting heart rate, overnight HRV, VO2max and
  Fitness Age by month, each opening its detail; the current month's bar is
  outlined and spoken labels say "so far".

Follow-through and measured change stay apart; nothing is a grade. Domain
charts key the outlined current period, dashed weekly goal and hatched/dotted
coverage marks. Monthly physiological traces are the neutral blue, not
colored as a verdict from their direction. Resting heart rate opens the daily resting-HR
page, never the overnight Sleep HR detail.

### Profile and Settings

- **Profile** (the header's profile button): what Garmin knows (age, latest
  weigh-in with its date, units, the watch's last upload, history on the
  phone), limitations and injuries (opening Adapt's limits), gear, Settings
  and Help (Rules: Profile). Add a weight is disabled: it would write to
  Garmin.
- **Adapt** (every root header): Rest today, I'm ill, Pause training,
  Lighter week, Limits and My routines; Resume shows the goals first (Rules:
  Adapt, My routines).
- **Settings**: notifications, the wind-down lead, the time to fall asleep
  (also on Sleep · Tonight beside the plan), persistent status, Follow
  my day (and that Raring doesn't learn from how you use the app), the
  plan-your-week prompt, journal consent, recent Garmin changes once there are
  any (Rules: Recent Garmin changes), export, a validated import of another
  export's settings (Rules: Import), sign-out, help and how Raring works.
  History on the phone is on Profile. Export and Import each lead with one
  line, the rest behind "What's included"; an import preview (with years)
  replaces the intro and puts Import and Cancel first. Sign-out ("Sign out and
  erase this phone's data") confirms in place, names what it erases (Garmin
  history, journal logs, goals, plans, limits and routines; nothing in Garmin)
  and offers Export first.

### Weekly check-in

Opened from the Monday notification (or a passed week in Progress's weekly
goal rings; `#review/latest`, an explicit week is `#review/YYYY-MM-DD`): one
page for the week just ended, the decision first. Once goals are set, the
goals card lists each goal's result for the plan week (read from the local
goals: met, partly, not met or not counted, and why: rest, illness, a pause,
missing data) and, for the week that can still decide (Rules: Week review),
next week's proposed goals with Accept and Keep all; Edit goals opens the
weekly goals and returns here. Then the week's readings in one grid:
readiness, Sleep Score, acute load and Fitness Age, each with its average,
the change against the week before (under 1 is steady) and its seven days
labelled. Only readiness and Sleep Score carry status colours; load and
Fitness Age are neutral, with no gauge or invented scale. An intensity card
shows Garmin's weekly intensity minutes as a ring against Garmin's goal, with
the moderate and vigorous split. A timing card shows the week's bed and wake
times, their earliest and latest, how many nights Garmin rated aligned, ahead
or behind, and one timing focus. It is descriptive: no adherence score,
streaks or claims that timing changed a score. VO2max and Fitness Age changes
of 1 or more are called out. It ends with one win (a change of 1 or more the
grid shows) and one focus naming Garmin's metric and its values (Rules:
Weekly focus), a line each.

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
While resting, ill, paused or limited, the focus says so and carries no
training call, from the same local records as Today, republished offline on
each local change. Widgets, the notification and the lock screen never show
limits, injury notes, routines, journal entries or weight.
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

**Adaptive Garmin Coach.** An adaptive plan's sessions are not in the workout
schedule. When the stored plan list has a `FBT_ADAPTIVE` plan that is
scheduled or active, the core reads
`GET /trainingplan-service/trainingplan/fbt-adaptive/{trainingPlanId}`
(positive numeric id) on the same six-hour check, budget and backoff. Its
`taskList` holds the current week only, one task per day with
`calendarDate`, `weekId`, `workoutOrder` and `taskWorkout` (`workoutName`,
`workoutPhrase`, `workoutDescription`, `estimatedDurationInSecs`, `restDay`,
`trainingEffectLabel`, `sportType.sportTypeKey`,
`adaptiveCoachingWorkoutStatus`; `workoutId` is null, so no steps); the plan
carries `durationInWeeks`. Seen phrases: `BASE`, `LACTATE_THRESHOLD`,
`POOREST_SLEEP_RECOVERY`, `FORCED_REST` and strength codes ending
`_GENERIC_GYM` or `_GENERIC_BODYWEIGHT`; cardio descriptions are a target
heart rate like "129bpm", strength and rest have none. A response for another plan, without the adaptive
category, or with an empty or undated task list counts as a data failure and
keeps the last good snapshot. Sessions show only while the plan list still
names that plan.

## Notifications

Settings reads whether Android allows Raring's notifications each time it
opens. When Android blocks them, one line at the top of Notifications says so
and the switches are greyed; it offers Allow until a request is declined. A
declined request is remembered on the phone: the line then points to Android
Settings and Raring doesn't ask again. Granting it later through the
persistent-status switch clears the note. Each switch says when it fires.

Sent by the phone itself.

| Notification                  | When                                         | Default |
| ----------------------------- | -------------------------------------------- | ------- |
| Morning readiness             | Last night's sleep and readiness have synced; while resting, ill, paused or limited, a factual update with that line and no training call | On      |
| Weekly review                 | Monday 08:00                                 | On      |
| VO2max or Fitness Age changed | Either direction                             | On      |
| Vitals outside your ranges    | Two or more vitals outside normal range      | On      |
| Training status changed       | On sync                                      | Off     |
| Wind-down reminder            | Wind-down time of tonight's plan (lead 30 min before bedtime, configurable) | Off     |
| Persistent status (Android)   | Existing sync; silent, no extra Garmin requests | Off     |

The morning readiness notification opens Today, the weekly check-in the
latest review (`#review/latest`), the bedtime reminder Sleep · Tonight
(`#sleep/tonight`) and widget readings their details.

On desktop the app shows the same notifications itself while it runs, with
the same wording, settings, rest/illness/limits suppression and night
identity. A desktop build records a notification only after the Tauri
notification plugin accepted it, and on Linux only when the session has a
notification server (the plugin discards that server's error); a refusal or
failure leaves it unmarked and Settings says why. Settings also states the
next pending wind-down while signed in.

## Rules

- **Local records**: goals, your sessions, plan links, circumstances (rest
  days, pauses, limits, dismissals, wind-down reports), plan-week prompt
  answers, local events and your routines are versioned JSON rows (`local:*`, each with
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
- **Garmin writes**: every change to Garmin follows one path in the core,
  and nothing ships a write until it has passed on a real watch (TODO item
  20). A write runs only from the person's explicit action, never from sync
  or another background job; a delete also needs an explicit confirmation.
  It starts from a fresh read of the resource and applies a typed change to
  the fields the person touched. Each change carries the value the person
  saw: if Garmin's value differs, or a list item it names is gone, nothing is
  sent ("changed in Garmin since you looked"). Other values, including ones
  changed in Connect meanwhile, come from the fresh read; list items are
  found by their ids, never their position. Only the touched top-level keys
  (and the resource's id) are sent, so a stale whole object is never
  written. The write uses the same session, pacing, one token refresh on 401,
  and 429 backoff (`backoff_until`) as sync; during a backoff nothing is
  sent. After Garmin accepts it, the resource is read back and every sent
  value compared, with string lists as sets and numbers by value. The
  outcome is one of: saved; saved but read back differently (with what
  differs); sent but unconfirmed (the read-back failed, or the server or
  connection failed after sending); not sent because it changed in Garmin;
  refused, with Garmin's message (any 4xx but 401/429, at most 200
  characters, never an HTML page); not saved (rate limit, not found,
  network); or sign in again. Invalid changes are refused before any
  request. Writes are serialized, so two saves never interleave.
  The first resource is a watch's alarms
  (`/device-service/deviceservice/device-info/settings/{deviceId}`: time in
  minutes after midnight, `alarmMode` ON/OFF, `alarmDays`, removal), proven
  against a local mock only; nothing in the app sends it yet.
- **Recent Garmin changes**: every attempt that passes validation is logged
  in a versioned settings row (`garmin:changes`, `schema` 1), newest first,
  at most 50, with its time, what was written, each field's old and new value
  as the person chose them, the outcome, Garmin's message and read-back
  differences. A row with a newer schema or one that doesn't parse is shown
  as far as it parses and never rewritten. Like other settings it is in the
  export and erased by sign-out and a verified account switch, but never
  imported (Rules: Import); an attempt that outlives its account isn't
  logged. A saved change to a watch is
  "waiting for watch sync" until that watch's stored
  `lastDownloadTimestamp` (from sync's device detail) is later than the one
  read right after the write, or than the write time if that read failed.
  Settings lists the log; it never sends anything.
- **Import**: a Raring export's settings can be restored on another phone by
  choosing the ZIP with the system file picker (Android) or a file dialog
  (desktop); it is read from the phone and nothing is sent anywhere. Import
  needs a verified signed-in account and an export that names its account; a
  file from another account, an unknown archive format or a newer one, a row
  written by a newer version of Raring, an unreadable record shape or a
  credential field is refused whole, before anything is written. A newer
  archive marker, a newer `schema`, an unknown `local:*` row, an unreadable
  weekly plan or saved night, and a credential key at any depth are refusals,
  not skips. Only settings are imported: the weekly plan row, prospective
  sleep-plan snapshots, the versioned local records and the plan prompt
  preference. Garmin responses, journal logs, milestones and recent Garmin
  changes are not imported: a sync downloads this phone's Garmin data and
  journal reads stay this phone's own choice, milestones are awarded again
  from its recordings, and the change log is account history of what the
  other phone sent (skipped whatever its schema; the preview names it when
  the file has one).
  A validated preview names the counts per record kind (added, replaced,
  kept), the covered date range and what is skipped and why, before the person
  confirms. Applying is one transaction under the account/write barrier, and
  the offline status snapshot follows. Existing local records are never
  silently overwritten: goals merge by effective date and a same-day copy is
  replaced only by a later save; sessions, plan links, review decisions and
  routine reports take the later timestamp, and a later link re-pairs a pair
  an earlier unlink separated; rest days, wind-down reports, dismissals,
  prompt answers, events, checklist items, plan weeks and sleep-plan snapshots
  only fill what this phone lacks (a saved night is prospective evidence and
  is never replaced); a pause with the same id takes a closed interval over an
  open one; limits fill only an empty set; the kept session for a day takes
  the later day; the plan prompt preference fills a phone that never saved
  settings. A row this build may not rewrite is skipped and named. A second
  import of the same file changes nothing.
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
- **Garmin Coach**: an adaptive plan's tasks in the plan week are read-only
  planned work beside your sessions, with Garmin Coach attribution; a rest-day
  task is a Garmin rest day, not a session. `strength_training` plans one
  strength session; `other` is a cardio session that plans no Zone 2, Zone 4–5
  or intensity minutes (none are inferred from its label or duration); an
  unfamiliar sport is shown and never counted or matched. A Coach strength
  session matches a strength recording, and a Coach cardio session any one
  run, ride, swim, walk or hike, on its day under the Plan matching rule
  (each the other's only candidate, otherwise Raring asks; links and unlinks
  as for your sessions). Garmin's own status is shown as Garmin's ("Not
  complete" is the only value seen) and never marks a session done. Coach
  sessions can't be edited, moved or let go; Keep and Choose accept today's.
  Only Aerobic base, Lactate threshold and Recovery effect labels are shown.
  With limits set, Garmin's workouts are marked "Review needed · contents
  unknown to Raring" (Garmin sends no movements, and no sport for scheduled
  workouts or Coach cardio); a Coach strength session with strength excluded
  says it conflicts. Raring never calls a Garmin workout compatible or safe.
  A scheduled workout opens a read-only detail: name, day, unread completion
  flag, not counted, and Not today on its day.
  A snapshot older than 12 hours names the plan as possibly out of date and
  plans nothing, like the schedule.
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
  planned nor a second count, and to review counts the recordings in question
  (one recording that may be two sessions is one). Remaining is target − recorded − planned, at
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
  person's acceptance changes goals, as a version from the next week, or from
  the day it is made once that week has begun (the week keeps its starting
  targets, as for any later change); Keep all and ignoring a review change
  nothing. Only the latest completed week's review offers the choice, until it
  is decided; older and ongoing weeks show their outcomes, and a decided week
  the proposals it saw and what was accepted.
- **Workout detail**: from the stored activity only. Streams beyond heart
  rate (`directSpeed` in km/h, `directElevation`, `directDoubleCadence` or
  `directBikeCadence`, `directPower`, `directRespirationRate`,
  `directAvailableStamina`, `directPerformanceCondition`,
  `directAirTemperature`) share the samples'
  elapsed-time axis (at most ~300 points) and appear only when recorded; a
  missing sample is a gap. Runs, walks and swims show speed as pace, drawn
  faster-up. Summary fields appear
  only when Garmin sent them (moving time, moderate and vigorous intensity
  minutes — shown as moderate + 2 × vigorous, Garmin's figures — Body Battery
  change, stamina at start and end, descent, normalized power, intensity
  factor, best 20-minute power, FTP, stride length, breathing rate, recovery
  heart rate). Training effect keeps Garmin's value and label and adds its
  band (below 1 no benefit, 1 minor, 2 maintaining, 3 improving, 4 highly
  improving, 5 overreaching) with one fixed plain line per band and kind. A
  VO₂ max link appears only when the stored estimate for the activity's date
  differs from the one before it. A stream is used only when its
  descriptor's unit key is the expected one (mps, meter, stepsPerMinute, rpm,
  watt, breathsPerMinute, dimensionless, celcius); its values are already in
  that unit and the descriptor's factor is never applied. The watch's
  temperature (`directAirTemperature` and the summary's average, lowest and
  highest) is labelled as the watch's own sensor, never the weather.
- **How it counted**: a live workout read names, for its plan week's goals
  on, its share (a strength recording is that week's Nth strength session of
  the target; Zone 2 and Zone 4–5 add its own zone minutes), the week's
  recorded total, the plan item it completed, and any milestone awarded for
  it. With no weekly goal on and nothing else to say, the card is left out
  rather than implying a goal rejected the workout. Garmin's estimates (VO₂ max, Fitness Age) are never credited to one
  workout.
- **Muscle gaps**: sets per muscle group over a range ending today (7D–1Y,
  or Max from the first stored session), each named active set counting once
  for each of its primary muscle groups (the Sets per muscle rule). A group
  with at most round(days ÷ 14) sets in the range is a gap, fewest first.
- **Lift volume**: per session and exercise, the heaviest counted set and
  volume = Σ weight × reps over counted sets (named, weighted, with reps, not
  flagged as doubled); bodyweight, unweighted, rep-less and doubled sets are
  left out of both and their number is shown. No estimated one-rep max. Lift
  charts share one Max weight / Volume choice and the 7D–Max range picker,
  each range ending today; points are sessions, not averages, placed on a
  date axis from the range's first day (the first session for Max) to
  today, so breaks show as gaps. An exercise with no weighted set charts the
  most reps in one counted bodyweight set each session instead, with no
  Volume choice.
- **Week lanes**: live Training only. One lane per cardio goal on (Zone 2,
  Zone 4–5, intensity), or recorded cardio minutes (every sport but strength
  and other) when none is, plus strength sessions when that goal is on or a
  strength activity was recorded in the last 8 weeks. Each lane is cumulative
  from the plan week's first day through today: Zone 2 and Zone 4–5 from each
  activity's zone minutes, intensity from Garmin's daily moderate + 2 ×
  vigorous totals, strength as sessions. A day outside the activity list's
  coverage (recorded lanes), with an activity lacking zones, or without
  Garmin's daily totals (intensity), makes the lane unknown from then on. The
  usual week uses up to the 8 previous plan weeks that the week review counts
  (complete, full activity coverage, at most one ill or paused day) and whose
  lane is fully known; with 3 or more, the band is each weekday's 25th–75th
  percentile (linear interpolation) and the median; fewer, no band.
- **Event week**: in the current plan week, the soonest event with a sport
  (a local event; Garmin's race has no verified sport and only counts down)
  that has your sessions of that sport not let go tags them in their days
  ("For Harbour 10K") and shows that sport's recorded minutes this week
  through today on the event's line. Tagging changes nothing in the ledger: a
  session counts toward each weekly goal once.
- **Plan prompt**: for people who have chosen goals before, while the setting
  is on and the week isn't answered: at the start of a week (its first two
  days) after a week with goals, it offers last week's review then planning;
  otherwise only while this week has nothing placed. Never in wind-down or late
  night, on any page. Plan now, Not this week and turning it off are answers.
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
- **Milestone archive**: `progress/milestones` lists every milestone in
  available history, newest first and grouped by month, from the same awards
  as Progress (read once more awards nothing; corrections keep their award).
  Progress shows the latest 12 and links to it.
- **Progress domains**: from local data only. Week and month periods carry
  ring cells for the whole plan weeks covering them (the Rings rule). Cardio
  and Strength show the 12 plan weeks ending this week from covered history:
  recorded Zone 2 and Zone 4–5 minutes (unknown when any activity lacks zones
  or the activity list is incomplete), Garmin's weekly intensity minutes
  (Plan progress), strength activities, and Garmin's latest VO₂ max estimate
  dated in the week; a week with two or more ill or paused days, or an
  incomplete list, carries that mark. Each week carries its own targets (Goal
  ledger: in effect at its end, Garmin's weekly goal for intensity following
  it), none before goals began or while a goal was off. Long-term
  shows 12 calendar months: recorded minutes (none before history or with an
  uncovered day), the month's average of Garmin's daily resting heart rate
  and overnight HRV, and the latest VO₂ max and Fitness Age estimates dated in
  it. Nothing is interpolated.
- **Milestones**: in available history: an exercise's heaviest set for 1, 3,
  5, 8 or 10 reps (Records and bests eligibility), an exercise's most reps
  in one bodyweight set, the longest recorded
  distance for a known sport family recorded at least three times, and the first
  completed, counted week with every action goal met. Mixed unknown sports are
  not combined into a distance record. A week with a revision or pause cannot
  earn this whole-week award; observations alone never qualify. A set that is
  the heaviest for several rep counts is shown as one row with its weight and
  reps and the counts it holds ("12 kg × 10 · heaviest for 1–10 reps").
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
  Lighter week (busy or travelling, no reason asked) halves this week's
  strength, Zone 2, Zone 4–5 and chosen intensity targets from today (counts
  to at least 1, minutes to 5-minute steps; Garmin's intensity goal and sleep
  goals unchanged) and restores the current targets from next week unless a
  later week already has its own; the week keeps its original targets on
  record. Goals first set today are changed in Goals instead.
  Resuming first shows the week's goals as they are; the person resumes them
  unchanged or resumes and opens the goals. Nothing is added to catch up.
- **My routines**: a checklist the person supplies (exercise name, optional
  how; up to 30 items) in a `local:routines` row, edited in Adapt. Each item
  can be self-reported done for the local day (one report per item and day,
  kept 90 days; undo removes it). Raring adds no exercises, load or
  progression, and a self-report is never a Garmin activity, intensity
  minutes, record, ring or goal count. Names never appear in notifications,
  widgets or the status snapshot.
- **Today's lead**: live Today only (never a historical read), gates in this
  order, first match leads: a night that hasn't synced waits (no advice,
  even while the previous day's readiness stays on screen);
  evening, wind-down and late night lead with tonight's sleep (no exercise;
  a session still planned today stays as a quiet "still in your plan" line);
  illness or a pause (resume from Adapt); rest; the fixed layout's recorded
  day; a recording today (the latest, skipping walks once goals exist),
  headed "Done today" above the Recorded today list (which lists it with
  the rest), with its strength count against
  the goal or Garmin's intensity minutes; Garmin's
  scheduled workout today unless dismissed (limits make it "Review needed",
  since its sport and movements are unknown); your planned session today,
  Garmin Coach's included (a movement note asks for review; an excluded
  sport is labelled). A leading session's headline names the day from
  Garmin's call (push "A harder day fits", steady "A steady day fits", easy
  or rest "An easier day may fit", none "Planned for today") and the session
  itself goes to Today's session; no readiness: last night's
  sleep against its need; limits: no training suggestion; "Not today" for the
  day's suggestion: nothing else; Garmin's push call: training is an option, to
  plan, dismissable; otherwise an easier or steady day naming the worst
  Garmin-rated readiness factor. Each lead carries a stable id (dismissals key
  on it), its action and the gates it passed (kept in the view for tests and
  diagnostics; Today shows one reason line from Garmin's inputs instead).
- **Today's session**: described only from verified fields. A Garmin Coach
  session: strength (`strength_training`) says "gym" or "bodyweight" when
  `workoutPhrase` starts `STRENGTH_` and ends `_GENERIC_GYM` or
  `_GENERIC_BODYWEIGHT`; cardio (`other`) adds the target heart rate when
  `workoutDescription` is exactly "<n>bpm" (30–230); both add Garmin's
  estimated duration ("about 45 min"). Garmin's effect label is said in plain
  words with Garmin's label beside it: Recovery "Recovery: very easy",
  Aerobic base "Easy aerobic base: steady, conversational effort", Lactate
  threshold "Threshold: hard, sustained effort". `POOREST_SLEEP_RECOVERY`
  adds "Coach eased this after a short night". The source names Garmin's
  `weekId` and `durationInWeeks` ("Garmin Coach · week 1 of 4"). The fit
  line compares the label's intensity (recovery below aerobic base below
  threshold, on the effort scale's rest, easy and steady) with today's
  suggested effort and says only "Harder than today's suggested effort",
  never a prescription; it is absent when the session fits, without a label
  or without a call. Unknown phrases, sports and labels leave only the name
  and duration. A Garmin calendar workout shows its name only; your session
  its name, time, sport and duration.
- **Today's choice**: Keep records one of today's sessions (yours, open and
  dated today, or Garmin's scheduled workout today by name) as the day's
  choice; it leads over the day's other sessions in the same gate and lasts
  its local day. Choose another lists today's Garmin schedule and your sessions,
  your open sessions later this week (moved to today and kept), what's left in
  the backlog, a new session and Rest today. A backlog item suggests the
  endurance sport recorded most in eight weeks that the limits allow,
  otherwise running or walking; one the limits exclude is labelled
  "Excluded by your limits" in Choose and on This week. Nothing is sent to Garmin.
- **Profile**: from local data only, no request: Garmin's age (sent with
  Fitness Age), measurement setting, the latest weigh-in with its date (flagged
  after 7 days), the primary watch's last upload, history on the phone, gear,
  and your limitations (edited in Adapt's limits). The watch's model name is
  not shown until its device-list field is verified. Adding a weight is a
  Garmin write and stays disabled.
- **Sleep tabs**: Sleep has Last night and Tonight (`sleep/last`,
  `sleep/tonight`; switching replaces the page, never stacks). A plain `sleep`
  link opens Tonight when Today's phase is evening, wind-down or late night,
  otherwise Last night; the bedtime reminder and Today's tonight line open
  Tonight. Anything showing last night (Today's last-night tile and moon
  chip, Your day's sleep row, naps, the widget's Sleep tile, search's "Last
  night's sleep") opens Last night. Tonight shows the saved sleep plan
  (wind-down, Garmin's window or the asleep-by estimate, usual wake, Garmin's
  need, with when Raring read it, its weekday unless today), the week's sleep
  goals, a self-reported "I wound down" for the evening before the night the
  plan describes (open from two hours before that night's window; without a
  plan, a report before noon is the evening before), and the seven nights
  ending last night against each night's own Garmin need: how many recorded
  nights were below it, never a running debt. A missing night is labelled,
  not zero.
- **Opened past days and workouts**: only an explicit open of a page whose
  route names its own date or activity reads Garmin for that detail;
  navigation prefetch, cached tabs and ordinary sync never do. One open reads
  only the parts its page shows and only rows this phone has never
  stored—absent, not Garmin's empty answer—in a fixed page order, inside a
  small budget and the shared per-endpoint retry wait and 429 backoff, and
  keeps every answer, including empty ones. Today's data is never substituted for an older date; a part
  Garmin has nothing for says so. The route keeps the date, Back returns to
  the page or section that opened it, and the plain `sleep`, `sleep/last`,
  `sleep/tonight`, `health`, `metric/stress` and workout links keep their
  current meaning. The all-day heart-rate read keeps its own one-day rule.
- **Your day**: live Today only, the local calendar day on one 00–24 axis:
  last night's recorded stages at their clock times, clipped at midnight;
  Garmin's intraday Body Battery and stress in three-minute slots and heart
  rate in two-minute slots, all three left to their details on a day that
  isn't 24 elapsed hours (a clock change), since their slots count elapsed
  minutes; today's activities and naps at their start times (a nap starting
  before midnight belongs to the day before). Gaps stay gaps and nothing is
  drawn after the watch's last upload or now (an upload before midnight draws
  nothing; with no upload time, now). A nap's Body Battery change is the first
  measured level at or after its end minus the last at or before its start,
  when both exist and the nap ended by the upload; levels Garmin marks as
  modelled don't count. Keep refuses a sport your limits exclude.
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
- **Stored weight history**: Garmin's weigh-ins are grams; display kg or lb
  from the account's measurement system, never guessing before it is known.
  For repeated weigh-ins on a date, keep the latest timestamp; merge the latest
  endpoint only when its valid reading is newer than that stored day. The
  legacy 12-week scale projection keeps the last reading per week for existing
  consumers. Latest is fetched at most daily, recent ranges weekly and older
  years in budgeted ranges; empty results count as checked. Viewing weight
  never requests Garmin data or writes settings. No weight notifications.
- **Trend weight**: Raring's exponentially smoothed trend starts at the first
  stored weigh-in. A subsequent weight w, g days later, moves trend t by
  `(1 − 0.9^g) × (w − t)`. Missing days retain the prior trend only between
  measurements; no series is projected past the last weigh-in. Computation
  uses unrounded kg, with conversion only for display. A change over N days is
  the final trend minus its value N days earlier, unknown before the trend
  existed. Show 1, 3, 7, 14, 30 and 90 days, ending at the last weigh-in; from
  7 days also show `change × 7 / N` per week. Absolute change strictly below
  `max(0.2 kg, 0.05 kg × N / 7)` is Steady, otherwise Losing/Gaining. These
  words are neutral directions, not goals, verdicts or physiological claims.
  Weight uses the neutral blue, never good/bad direction colours.
- **Weight chart ranges**: 7D, 14D, 30D, 90D, 1Y (365 days), Max (all stored
  readings), ending at the last weigh-in. Average is the mean of daily trend
  values in the shown window, not of plot buckets; change uses the same N-day
  rule (Max compares the first and final trend). Up to 90 days plots daily,
  longer ranges weekly, over 400 days fortnightly: each bucket takes its last
  weigh-in and that date's trend, or the held trend at bucket end if no weigh-in.
  Scale-line breaks reflect gaps over two days in underlying measurements,
  not gaps created by downsampling. Scale points remain actual weigh-ins;
  monotone display interpolation must not overshoot them. Charts name the
  dates and units; body fat/BMI retain their recorded meaning.
- **Reading freshness**: age is calendar days from the source reading date
  to the view's as-of day, never API fetch time. Unknown/future source dates
  have unknown age. A weight reading is flagged only after seven days; daily
  metric freshness uses two days: a trend chart ending today flags its
  metric's newest reading (within 400 days) when it is older. Charts ending
  in the past, and sparse estimates (VO₂ max, Fitness Age), are not flagged.
  No reminders to weigh in.
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
- **Night vitals on Today**: last night's Sleep HR is Health Status's HR
  against its baseline when Garmin sends one, otherwise the sleep record's
  overnight average (`avgHeartRate`) against the Normal range fallback of
  the last 30 recorded nights' overnight averages; never resting HR. HRV is
  the vitals card's nightly HRV against Garmin's baseline. Inside the range
  it is "Within usual"; outside, the difference from the range's middle in
  the metric's own units, rounded ("6 above usual"); an onboarding Health
  Status is "Garmin calibrating"; no range is "Usual not set yet". Higher
  Sleep HR and lower HRV outside the range are fair-toned; everything else is
  neutral. Shown for the displayed night in the morning and day phases only,
  not while that night hasn't synced.
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
- **Time to fall asleep and in bed**: Garmin's window and need stay
  Garmin's; the asleep-by time (the window's end, or Raring's estimate) is
  when to be asleep. Raring adds the time to fall asleep from Settings
  (default 15 min, 0–60 in 5-minute steps): in bed by is the asleep-by time
  minus it, and time in bed tonight is the need plus it ("7:50 asleep, so
  about 8:05 in bed"), from in bed to the expected wake (asleep-by plus the
  need). In bed by and time in bed are labelled Raring's.
- **Wind-down**: the earlier of the window's start and the in-bed time
  (without a window, the in-bed time) minus the lead from Settings. The reminder fires once per night, keyed by the morning
  the night ends, between the wind-down and asleep-by times; it is skipped
  when no recorded sleep (a duration, start or end) has synced for the
  current day. Times are local wall-clock times, so the platform reschedules
  when the timezone changes. A desktop build has no WorkManager: while it
  runs it reads the cached plan on its own wall clock, wakes at the pending
  wind-down, checks again on resume, after a sync and after a notification
  setting changes, and drops a reminder whose asleep-by time passed while the
  machine was asleep.
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
  shared sleep plan, never fixed clock hours. The usual wake is the recorded
  wake times' once there are at least four in 14 nights; until then it is
  the wake time of Garmin's sleep schedule for the displayed day (that
  weekday's window, otherwise the daily one, otherwise `userSleep`), which
  counts as credible and is labelled "Garmin sleep schedule" on Today.
  Garmin's untouched defaults (`defaultSleepTime` or `defaultWakeTime`) are
  no schedule. After waking lasts three hours,
  or until the plan rises if that comes first; the day runs until two hours
  before Garmin's window starts (Raring's asleep-by time without one); the
  evening until the plan's wind-down time; wind-down until the window ends;
  late night until the next expected wake, so crossing midnight keeps
  tonight's plan. From that wake (or, without a credible usual wake, from the
  calendar rollover) until the handover, whatever the layout or fallback,
  Today waits neutrally for the night: no effort call, no previous plan and
  no verdict. After the handover the calendar day shows with its missing
  night stated. Advice order: after waking, this morning then the one-line
  plan; during the day, today's focus, the one-line plan; from
  the evening on, the plan, then Your day and recorded today (compact), then
  today's focus. Before the evening Your day stays under the lead. Two or
  more outside-range vitals go first in every phase. The fixed layout (today's focus, then the
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
- **Compact focus**: the live Today's lead (Rules: Today's lead) decides,
  in its gate order. Tonight's sleep uses the lead's headline and the shared
  sleep plan's reason, opening Sleep. A night that hasn't synced (including
  the phase's neutral wait, which keeps the previous day's readiness on
  screen) says "Waiting for last night's sleep" with no effort or reason.
  Illness, a pause, rest, limits, a review and Not today use a short private
  line instead of the lead's text: "Ill · goals paused", "Training paused",
  "Rest day", "Training suggestions limited", or "No training suggestion
  today", opening Today. A recorded workout says "Activity recorded today",
  opening Today. Wherever else Today shows Garmin's effort call, the
  suggested Rest/Easy/Steady/Push effort and Today's reason line from
  Garmin's inputs; without readiness, "Waiting for readiness". Session, workout and
  activity names and the limits text never reach a surface. A local change republishes the
  snapshot offline. Native surfaces
  describe the last snapshot, not a live watch reading. Offline widget
  updates, clock changes and sync re-evaluate the phase; Android may defer
  delivery, so transitions are not exact alarms.
- **Timing review**: the week's earliest and latest bed and wake times, the
  wake-time spread (latest minus earliest) against the week before, and
  counts of Garmin's alignment ratings. The focus follows the majority rating
  when at least three nights are rated, otherwise whether wake times varied
  by an hour or more, and names the week's median wake time.
- **Sleep stage comparison**: for the selected night D, each stage's mean
  is the sum of its finite, non-negative Garmin duration seconds in stored
  detailed sleep rows dated D−29 through D inclusive, divided by the number
  of those recorded values, then converted to minutes. Each stage has its
  own denominator: missing, null and invalid values do not count; an explicit
  zero does. No earlier or future night fills a gap. With no values the mean
  and delta are unknown; an absent/invalid selected stage has no amount row.
  One recorded night is labelled as one, not a usual range. The core exposes
  that mean and its count, and the selected duration minus the mean; the UI
  never reconstructs a comparator from the delta. This is a descriptive
  comparison, not a score, target or favorable direction.
- **Normal range and vitals off**: use Garmin Health Status's per-day
  baseline bounds and status first. A reported outside-range status is
  fair-toned (amber, as on the charts) and counts toward the two-vitals
  warning and notification; `ONBOARDING`, unknown
  statuses and missing values are neutral, even if provisional bounds exist.
  When Garmin has no Health Status metric, use the middle 80% of the last 30
  recorded nights (at least 14) and count readings outside that fallback
  range too; HRV retains Garmin's existing HRV baseline. Never compare Garmin's overnight
  average HR to resting-HR history: the resting-HR chart's band is always this
  fallback computed from Garmin's daily resting HR over the last 30 days, even
  when Health Status supplies Sleep HR, and Sleep HR's baseline shades only
  Sleep HR. Health Status's nightly HRV status is
  separate from Garmin's seven-day HRV status. Personal ranges are not
  medical targets.
- **Stress bars**: the Today card averages each 15 minutes of Garmin's
  readings; the detail shows every reading. Colors follow Garmin's categories,
  without a colour key; the selected reading names its level.
- **Heart-rate chart**: the compact card averages only present samples in
  16-minute bars for legibility; the detail plots the two-minute readings.
  An empty bar remains a gap, not a zero or an inferred resting value.
  Use Garmin's UTC start and completed-day end as elapsed bounds: a source day
  can span 23 or 25 hours. A partial day reserves a 24-hour axis (and extends
  it if later real samples require it). The axis explicitly says elapsed
  hours from day start ("6 h", readout "23 h 58 m into the day"), never
  clock-style times; a 23- or 25-hour day says how long it is. Source-local
  travel/DST clocks cannot be reconstructed from a single start offset. Sleep and activities
  use their GMT instants, or local-only times only when start/end source
  offsets agree for a completed day. Their intervals clip at day bounds.
  Ungraded heart rate uses neutral ink, not a poor-status color.
- **Trends**: one value per Garmin day, from the recent per-day rows and the
  history rows behind them. Readiness is the morning score; Body Battery the
  day's high; acute load the day's Training Status (the value Training
  shows), else the day's last readiness snapshot; vitals use Health
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
- **Chart colours**: every metric chart is in one of four categories, shown by
  colour alone with no colour key; a line changes colour exactly where it
  crosses a threshold. The usual band still prints its bounds and source, and
  the age line its age.
  - Garmin's levels: readiness (Low/Poor under 50, Moderate 50–74,
    High/Prime 75+), Sleep Score (Poor under 60, Fair 60–79, Good 80+),
    stress (rest and low 0–50 good, medium 51–75 fair, high 76+ poor) and
    Body Battery's color.
  - Usual ranges: HRV, resting HR, Sleep HR, respiration, SpO₂ and skin
    temperature are good inside the shown range (bounds included) and fair
    outside it either way: attention, not a rating. Acute load against
    Garmin's optimal range is good inside it, fair below and poor above, as
    on Training's gauge.
  - Your age: a Fitness Age estimate below your age now is good, equal fair,
    above poor.
  - Neutral: weight, VO₂ max, time asleep, Progress's monthly averages, sleep's
    overnight curves and lift history, and range metrics without a shown
    band, in the neutral blue.
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
  extend coverage. Workout zone durations likewise preserve missing/invalid
  values as unknown, never filling an omitted numbered zone with zero.
  Strength also requires a known Garmin type for every
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
  vigorous-minutes average against its target), said as one line with
  Garmin's metric and values ("Garmin's load focus: low aerobic was furthest
  below its range").
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
- **Body-map figure**: every body map uses Strength's saved figure choice:
  Profile follows Garmin's profile gender; Female or Male overrides it. The
  choice is a setting, exported with the rest, and every body map (workout,
  Strength, Training) follows it. Unknown or missing profile gender falls back
  to Female, as before.
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
- **Explicit strength refresh**: only a person tapping Refresh from Garmin on
  a stored strength activity's detail reads its sets again. It spends one
  request on `/activity-service/activity/{id}/exerciseSets`, obeys the shared
  rate-limit backoff and per-endpoint retry wait, and keeps the previous sets
  when the read fails. Navigation, opening the workout and ordinary syncs
  never trigger it. The detail shows when Garmin's sets were last read;
  before any read it says not checked yet. A refresh of a workout whose
  Garmin sets were removed leaves no sets and says so.
- **Records and bests**: a set is a record when an earlier session has the
  exercise and no earlier set (or earlier set in this session) was at least
  as heavy for at least as many reps. A best at N reps is the heaviest set
  with at least N reps. A session's top set is its heaviest set, then most
  reps. Only named, weighted sets that aren't flagged for reps count.
  Bodyweight sets (Garmin weight 0 or null) have no weight to compare, so
  they earn no weighted record or best; instead each named exercise's best
  is the most reps in one counted bodyweight set in available history
  (same exclusions and earliest-on-a-tie rule), shown labelled reps. The
  weighted and reps bests are separate awards on the same stable identity.
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
  are fetched only by an explicit Refresh from Garmin on that workout (Rules:
  Explicit strength refresh); opening a workout never reads Garmin. Older
  strength activities found by the history download get their sets after it. Once
  there are sets, weight or a recorded threshold/acclimation response, the
  user settings (`/userprofile-service/userprofile/user-settings`) are read
  weekly; they are also read daily while fewer than four wake times are
  recorded in 14 nights (Rules: Time of day), and once (at most daily) for a
  stored row without the sleep schedule. Only the measurement system, time
  format, start of week, gender (the body-map figure) and the sleep schedule
  (`userSleep` times and default flags, `userSleepWindows` start, end and
  frequency) are stored; the rest is discarded.
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
  slightly between Health Status and the nightly record. An explicitly opened
  past night may read that day's sleep and HRV; an opened Health day its
  stress, daily summary and Health Status; an opened older workout its
  activity, splits, heart-rate zones and chart details. Both are one budgeted pass for rows this phone has
  never stored, share the retry wait and 429 backoff, and are never
  prefetched (Rules: Opened past days and workouts).
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
  prefetched and cached by route for the current day, from local data only.
  Pages built from a tab's view share its cache: Sleep's Last night and
  Tonight, Readiness and Today's metric pages, Gear and Training's metric
  pages, Health's metric pages and the weekly goals. A cached page opens at
  once and is read again behind it. A page that isn't cached is read before
  it replaces the shown page, for up to 200 ms, so a quick read opens it
  whole; a slower one opens with "Preparing this view…" until its read
  arrives, and a later tap supersedes the wait.
  A dated past day, dated metric or old workout is never prefetched; it is
  read when opened, and its one explicit detail fetch follows that open.
  A local change (goals, sessions, Adapt, routines) clears the cache and
  rereads the visible page and the header's Adapt state. Training's and
  Progress's chosen sections are kept for the session; Progress's domain
  views and the milestone archive are read when first shown. A completed
  sync or a changed sync timestamp invalidates in-flight reads and prefetches
  the tabs again, but keeps cached views: until a page is read again it opens
  with what it had, so a sync never empties one. A local day rollover
  invalidates in-flight reads and clears the cache without clearing the
  visible view. Status is read on start,
  resume and sync, not before every navigation, and is applied independently
  of navigation so a tab switch can't discard a newer sync or sign-out. View
  commands read SQLite off the WebView thread. They parse stored nights
  without their per-sample arrays, except the one night whose overnight
  charts Sleep draws. Progress keeps its selected
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
  posting and recording the reminder. Desktop has no alarm service: while the
  app runs, a single check delivers due notifications through the Tauri
  notification plugin, waking at the pending wind-down and at least every five
  minutes, again on resume, after a sync and after a notification setting
  changes. Only accepted deliveries are recorded, so one that could not be
  shown stays pending and is retried.
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
- Chart legibility: a browser check measures the text inside every chart on
  every page at 360, 390, 720 and 1100 px and at 1, 1.3 and 2× text, and fails
  on overlaps, text on a marker, undersized labels, gauge numbers wider than
  their ring, and (Today, Sleep, Readiness) text leaving its card. It includes
  period summaries and previous-period comparisons, the shared amount rows
  and every Training and Progress section.
- Android: an emulator install-and-render smoke test. The spike proves
  sign-in and background sync on a real phone.
- A test fails if fixtures contain email addresses, Garmin IDs or tokens. Real
  data and tokens are gitignored.

## Manual steps

- One terminal sign-in on the development machine for a token, so the app
  can be built against real data that never leaves that machine.
- One sign-in on the phone for the spike.
- Four screenshot reviews.
