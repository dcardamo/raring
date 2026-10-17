# Weekly plan and strength mockups: notes

Open `index.html` for the gallery, or `index.html?app#plan` for one screen
full-window: resize it to walk through phone, unfolded (660 px and up) and
desktop (1000 px and up, side rail instead of tabs). Screens: `language`,
`plan`, `plan-setup`, `checkin`, `intro-1` to `intro-4`, `strength-workout`,
`fix`, `strength-history`, `builder`, `catalog`, `help`, `help-sheet`,
`widget`. The compatibility page is `site/compatibility.html`. Screenshots
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
rose strength). The core five screens are recolored to match. The spec's Look
section needs rewriting if you accept it; I haven't touched the spec.

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

## Open questions

Each has a recommendation. *Rules* marks Raring's own math, which needs a
line in the spec's Rules section.

1. **Accept the design language** and rewrite the spec's Look section.
   Recommend yes.
2. **Zone 3 yellow** instead of Garmin's green. Recommend yes, so green keeps
   one meaning; it differs from Garmin's own app.
3. **Plan history as average completion.** Recommend yes, over "targets met",
   which reads as failure for nearly-made weeks. *Rules.*
4. **Pace** decides whether a goal is amber: the share of the week's
   training days (nights, for sleep) that have passed. Recommend yes. *Rules.*
5. **Body map figure** comes from a setting chosen in the intro, defaulting
   from the Garmin profile when it has one. Recommend yes. The figures are
   hand-drawn placeholders and need an illustration pass before shipping.
6. **Sets per muscle** count a set once for each primary muscle in Garmin's
   catalog. If the catalog has no muscle data, Raring keeps a small mapping
   table. *Rules.*
7. **Help content** lives in the app, offline, as short pages, with a link to
   GitHub for questions. Recommend shipping the strength pages with strength,
   and the rest as each area ships.
8. **Week start** follows Garmin's start-of-week setting so intensity minutes
   match Garmin's total. Recommend yes; the mockup uses Monday.
9. **Asleep on time** means within 15 minutes of the suggested time, and
   nights without a plan are left out. *Rules.*
10. **Miscount flags:** 0 reps; reps at least 1.8 times the median of the
    exercise's other sets; an exercise Garmin couldn't name. Never fixed
    automatically. *Rules.*
11. **Readiness brake timing.** Readiness only exists on the morning. Recommend
    scheduling progressed weights and swapping in held weights after the
    morning sync on a Rest or Easy day. This depends on the watch syncing
    before the session, which the Forerunner 970 spike has to confirm.
12. **Progression defaults:** double progression with the smallest equipment
    step; add reps first when the next step is over 10% heavier; drop one
    step after two sessions below range; wait two sessions on a new exercise.
    *Rules.*
13. **Step cap:** warn at 90, offer "split workout" at 100. The spike needs to
    confirm how Garmin counts repeats.
14. **Weekly repeat** ("Every Sat") needs Raring to reschedule each week.
    Recommend shipping "Once" first.
15. **Compatibility page:** publish only when the features ship; name only
    tested models; create a `compatibility` issue label first.
16. **Units** follow Garmin's setting.

Not mocked: empty states, saving offline, a failed save. For a failed save,
recommend keeping edits on the device, fetching the sets again before writing
in case they changed in Connect, and saying plainly what was rejected.
