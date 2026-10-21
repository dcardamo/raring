# Weekly plan and strength mockups: notes

Open `index.html` for the gallery, or `index.html?app#plan` for one screen
full-window: resize it to walk through phone, unfolded (660 px and up) and
desktop (1000 px and up, side rail instead of tabs). Screens: `language`,
`plan`, `plan-setup`, `checkin`, `strength-workout`,
`fix`, `strength-history`, `builder`, `catalog`, `help`, `help-sheet`,
`widget`, `status-surfaces`, `reconcile`, `reconcile-closed`, `why-readiness`, `why-battery`,
`why-training`. The compatibility page is `site/compatibility.html`. Screenshots
are in `shots/`; regenerate them and `screens.png` with
`node design/mockup/shoot.mjs`. All data is synthetic.

## Status surfaces (items 9 and 10)

`status-surfaces` shows the same synthetic snapshot on small/medium home-screen
widgets. The time is the watch upload, not the time the widget was drawn. The
separate `widget` screen puts the widgets on a home screen for scale. An
earlier silent, ongoing status notification (Body Battery, stress and latest
heart rate) was removed: readings arrive only when the watch uploads to
Connect, so an always-visible readout of fast-changing values was often wrong.

Both widgets sit on the app's card surface (`#1B1E24`) with its ink and
supporting-text colours, no border and no brand header: the gauge is the mark.
The gauge matches the app's (270° arc open at the bottom, Garmin's rating
colour, a white marker at the reading) with the number inside and "Readiness"
in the opening. The small widget adds the focus line below; the medium widget
lists sleep, Body Battery and stress beside the gauge, one tappable row each.
All widgets show **As of**, even when some values are unavailable. A missing
reading is a muted dash with an empty track, never a blank or zero. Daytime
focus can update after an activity sync. In the evening it becomes wind-down
and Garmin's window rather than an effort instruction; the readiness number
still belongs to today's snapshot. The native gauge opens Readiness, each row
its detail, and the rest of the small widget follows the focus.

The minimum-size renders are `shots/widget-min-{small,medium}-{1.0,1.3,1.5}.png`
and `widget-min-{small,medium}-missing.png`: actual RemoteViews inflated at the
declared 150×190 dp / 280×150 dp bounds on emulator-5560, with text-size
assertions confirming each font scale. Focus and upload time wrap rather than
disappear; tests check every nonempty line, child bounds and tap regions, plus
missing-data cases. These are not launcher captures and do not claim coverage
of other launchers or physical phones.

The existing hourly network schedule stays unchanged. `scripts/measure-status.mjs`
measures accelerated offline refreshes only (seed a synthetic demo, select
an isolated emulator): one cold refresh took 401 ms; warm
refreshes 62–174 ms including adb/file checks, with no WebView. The emulator
reported 0 mAh for batches of one, four and twenty: not a meaningful estimate
of physical phone battery or radio cost, and no justification for more polling.

## Design language

Color means something or it isn't there. Icons and titles say where you are;
color says how it's going. The `language` screen shows it.

- **Status:** green good, amber fair, red poor. Only for how something is
  going: scores by level, ranges, flags, goals behind or done. Missing data is
  a gap, never a zero.
- **Effort:** heart-rate zones, cool to hot: gray, blue, yellow, orange, red.
  Red means hard. Zone 3 moves from green to yellow so green only ever means
  good.
- **Amount:** one blue, faint to strong, for "how much" with no judgement:
  the body map, sets per muscle and the weekly heatmap.
- **Ink:** everything else. Charts without a verdict, numbers, selected
  controls and primary buttons are white on the dark ink, not a hue.
- **Contrast:** all text is at least 4.5:1 against the surface it sits on
  (3:1 for large text). The grays are `#B4BCCB` for secondary and `#8E98AC`
  for tertiary text; the old `#69728A` measured under 3:1 on chips and tiles.
  Text on a colored fill is black or white, whichever reads better.
  `node design/mockup/contrast.mjs` checks every screen.
- **Glow** stays only behind Today, Sleep, Training and Health heroes,
  faintly, in the status color.

This replaces the spec's "one color per area" (violet sleep, orange training,
rose strength); the spec's Look section now describes it.

## Changes from the first round

- **Weekly plan** is one list: each target is a line with "N of target" and a
  bar that's ink on track, amber when behind pace and green when done. The
  rings, pace ticks and day strip are gone. History is one bar per week:
  the average completion across targets, each capped at 100%, so partial
  credit shows. Today shows the same four bars, compact.
- **Plan targets** start from a preset (Strength, Endurance, Sleep, Custom)
  and list all four targets in one card.
- **Garmin wording:** "Writes to Garmin" and "Save to Garmin" are gone; the
  fix screen says "5 changes" and "Save". Help explains once that edits update
  the activity in Garmin.
- **Help:** a Help screen (from the avatar on Today, and a ? on plan,
  strength and builder screens) and a bottom sheet for in-context help.
- **Help at the point of use:** there is no mandatory first-run walkthrough.
  Strength's ? and its empty states explain lifting with the watch, how sets
  are counted and fixing them, as the help pages and sheets do everywhere
  else. The optional figure picker (Profile, Female, Male) sits on Strength's
  body-map card, so the choice stays discoverable without an intro.
- **Body map:** Raring's own front and back figures, female and male, shaded
  by sets per muscle in the amount blue. On the workout detail (this
  session), strength history (this week or four weeks) and the Training card.
  Strength's figure picker uses the same figures.

## Explain the numbers

- **Why sheets** reuse the help sheet: a bottom sheet on phones and unfolded
  screens, a centred dialog on desktop. A "Why High?" button under the
  readiness headline, "Why 46?" under the Body Battery gauge and "Why
  Productive?" under Training's chips open it. Rows: Garmin's name, the
  value it used, Garmin's rating as a status pill, and one line on what the
  input is. Unrated inputs have no pill.
- **Scores that differ** is a dashed pill under Today's chips, with a dot in
  each score's status color; tapping it unfolds one sentence in place. It is
  one small component so Today's hierarchy can move it.

## Decided

- The design language above, including yellow for zone 3 so green keeps one
  meaning.
- Plan history shows each week's average completion across targets, each
  capped at 100%. It goes into the spec's Rules section with the weekly plan.
- The body map figure follows the Garmin profile; Strength's Profile, Female
  or Male picker overrides it and is saved with the other settings.
- **Pace** decides whether a goal is amber: the share of the week's
  training days (nights, for sleep) that have passed. Yes. *Rules: add to
  the spec when the weekly plan ships.*
- **Sets per muscle** count a set once for each primary muscle in Garmin's
  catalog. If the catalog has no muscle data, Raring keeps a small mapping
  table. *Rules: add to the spec when strength ships.*
- **Help content** lives in the app, offline, as short pages, with a link to
  GitHub for questions. Ship the strength pages with strength, and the rest
  as each area ships.
- **Week start** follows Garmin's start-of-week setting so intensity minutes
  match Garmin's total; the mockup uses Monday.
- **Asleep on time** means within 15 minutes of the suggested time, and
  nights without a plan are left out. *Rules: add to the spec when the
  weekly plan ships.*
- **Miscount flags:** 0 reps; reps at least 1.8 times the median of the
  exercise's other sets; an exercise Garmin couldn't name. Never fixed
  automatically. *Rules: add to the spec when strength ships.*
- **Progression defaults:** double progression with the smallest equipment
  step; add reps first when the next step is over 10% heavier; drop one
  step after two sessions below range; wait two sessions on a new exercise.
  *Rules: add to the spec when the builder ships.*
- **Step cap:** warn at 90, offer "split workout" at 100. The spike needs to
  confirm how Garmin counts repeats.
- **Weekly repeat** ("Every Sat") needs Raring to reschedule each week.
  Ship "Once" first.
- **Compatibility page:** publish only when the features ship; name only
  tested models; create a `compatibility` issue label first.
- **Units** follow Garmin's setting.

## Open gates

- **Body map art (question 2).** Done with strength: the app draws its own
  female and male front and back figures (`app/src/lib/bodymap.ts`). The
  mockup's figures remain placeholders.
- **Readiness brake timing (question 8).** Readiness only exists on the
  morning. Schedule progressed weights and swap in held weights after the
  morning sync on a Rest or Easy day, if the builder's Forerunner 970
  workout-push spike confirms the watch syncs before the session. *Rules:
  add the brake to the spec when the builder ships.*

## Today through the day

Screens `tod-rules`, `tod-wake`, `tod-day`, `tod-evening`, `tod-winddown`,
`tod-late`, `tod-missing` and `tod-overview` (`time-of-day.js`) show one
synthetic day at different clock times. Only the leading card changes; the
hero (scores, freshness line, headline, chips) and the metric cards stay
where they are.

- **After waking:** "This morning": last night (asleep, need, wake time),
  the lowest readiness factor, then the effort. Before last night syncs,
  the card says so neutrally with the watch's last upload, without a
  verdict or yesterday's effort.
- **During the day:** Your day stays under the lead; its rows carry the
  latest synced Body Battery and stress, and the header carries the upload
  time. (`time-of-day.js` still draws the retired "Right now" card and the
  chips, which repeated the hero and Your day.)
- **Tonight's plan rises** two hours before Garmin's window: wind-down,
  window and need, with a strip placing now against them. A position, never
  a countdown. Earlier in the day the plan is one line.
- **Wind down** and **late night** are calm, ink-only cards without the
  effort segments; late night says the window has passed and nothing more.
- **Fixed layout** (today's order) when adaptation is off in Settings or the
  schedule isn't credible; an automatic fallback names its reason in one
  line.

Not mocked: empty states, saving offline, a failed save. For a failed save,
recommend keeping edits on the device, fetching the sets again before writing
in case they changed in Connect, and saying plainly what was rejected.

# Long-term trends: notes

Screens `trend-year`, `trend-overlay`, `trend-compare` and `trend-all`
(`trends.js`, `trends.css`). One chart serves every trend: Readiness, the
vitals, stress, Body Battery, sleep, acute load and VO₂ max.

## Decided

- **Ranges:** 7D, 30D, 90D, 1Y and All, plus Dates for a custom training block. Up to 90 days shows one point per
  day; 1Y shows weekly averages; All shows weekly averages for up to two
  years of history and monthly averages beyond. Weeks start on Monday until
  Garmin's start-of-week setting syncs (item 7).
- **Averages** are the mean of the days Garmin recorded in the week or
  month. Missing days are left out, never zero; a week or month with no
  recorded day is a gap. The selected point says how many days it holds.
  *Rules: in the spec.*
- **Color:** Garmin rates single days, not averages. Daily readiness bars
  keep Garmin's level colors; weekly and monthly readiness bars are ink.
  Other metrics stay ink lines at every range (a bar from an axis that
  doesn't start at zero would exaggerate). The current baseline band shows
  only on daily ranges, since past baselines differed.
- **Overlay:** both the primary and second metric can be selected. The second
  uses secondary ink with its own right-hand axis; bars dim under it. Picked
  from every supported trend metric; empty periods remain empty. No correlation figure: lining two lines up implies no cause.
- **Previous period:** the same length immediately before, as a faint
  dashed line on the same axis, with both periods' averages and the change
  in ink (no verdict color). A comparison start date can instead select any
  equally long block, aligned by elapsed day. Not offered on All.
- One comparison at a time: off, previous period, or a metric.
- **Layout:** the mockup spans the chart card across unfolded and desktop
  screens. The app keeps the card in its masonry column and draws the chart
  at the card's width, so cards don't leave holes around it.
