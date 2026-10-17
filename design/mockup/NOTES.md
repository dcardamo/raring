# Weekly plan and strength mockups: notes

Open `index.html` for the gallery, or `index.html?app#plan` for one screen
full-window: resize it to walk through phone, unfolded (660 px and up) and
desktop (1000 px and up, side rail instead of tabs). Screens: `language`,
`plan`, `plan-setup`, `checkin`, `intro-1` to `intro-4`, `strength-workout`,
`fix`, `strength-history`, `builder`, `catalog`, `help`, `help-sheet`,
`widget`, `reconcile`, `reconcile-closed`, `why-readiness`, `why-battery`,
`why-training`. The compatibility page is `site/compatibility.html`. Screenshots
are in `shots/`; regenerate them and `screens.png` with
`node design/mockup/shoot.mjs`. All data is synthetic.

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
- **Strength intro:** four steps the first time someone opens Strength: lift
  with your watch, fix it after, plan workouts, then setup (watch check, body
  map, weight steps, suggested weights).
- **Body map:** Raring's own front and back figures, female and male, shaded
  by sets per muscle in the amount blue. On the workout detail (this
  session), strength history (this week or four weeks), the Training card and
  the intro's setup.

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
- The body map figure defaults from the Garmin profile when it has one; the
  strength intro's choice overrides it.
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

- **Body map art (question 2).** The figures are hand-drawn placeholders;
  defer the illustration pass until strength ships.
- **Readiness brake timing (question 8).** Readiness only exists on the
  morning. Schedule progressed weights and swap in held weights after the
  morning sync on a Rest or Easy day, if the builder's Forerunner 970
  workout-push spike confirms the watch syncs before the session. *Rules:
  add the brake to the spec when the builder ships.*

Not mocked: empty states, saving offline, a failed save. For a failed save,
recommend keeping edits on the device, fetching the sets again before writing
in case they changed in Connect, and saying plainly what was rejected.
