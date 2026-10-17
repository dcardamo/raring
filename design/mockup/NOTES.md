# Weekly plan and strength mockups: notes

Open `index.html` for the gallery, or `index.html?app#plan` for one screen
full-window: resize it to walk through phone, unfolded (660 px and up) and
desktop (1000 px and up, side rail instead of tabs). Screens:
`plan`, `plan-setup`, `checkin`, `strength-workout`, `fix`,
`strength-history`, `builder`, `catalog`, `widget`. The compatibility page is
`site/compatibility.html`. Screenshots are in `shots/`; regenerate them and
`screens.png` with `node design/mockup/shoot.mjs`. All data is synthetic.

## Choices made

- **Strength is rose** (`#FF6FB5` to `#E0489A`). It's the one free hue between
  sleep violet and warning coral, and it stays clear of training orange. The
  old Training list drew the strength workout in sleep violet; it now uses rose.
- **The plan has no color of its own.** Each target uses its area's color:
  strength rose, intensity minutes orange, Zone 2 the zone-2 blue, asleep-by
  violet. The glow follows the week's state (mint when on pace).
- **Partial credit everywhere.** Each target is a ring reading "N of target",
  and there's no combined plan score. History shows each week's targets as
  partly filled rings, plus a count of targets met.
- **Placement.** A compact plan card sits on Today under Today's call and
  opens the full plan. Training gets a Strength card that opens Strength
  history. In the Monday check-in, the plan is the card before "Win & focus",
  with Keep and Adjust buttons.
- **Strength workouts show sets as bars.** Bar height is reps, the weight is
  underneath, last time's sets are a dashed outline and suspected miscounts
  are amber. Watch-counted and edited sets are marked separately.
- **Corrections put flagged sets first,** each with a one-tap fix, followed by
  every set with large steppers. Nothing reaches Garmin until one explicit
  Save, which sends the whole list (Garmin's `PUT` replaces the array).
- **Records are measured only:** the heaviest weight at 1, 3, 5, 8 and 10
  reps. There's no estimated one-rep max, volume or load score.
- **Builder:** each exercise has Off (your own targets) or Auto, and Auto shows
  the suggestion with a one-line reason. It also has a step meter, scheduling
  to Garmin's calendar and the readiness brake. On desktop it uses three
  columns: workouts and catalog, exercises, then schedule and settings.
- **Widgets:** a medium widget with readiness, the headline and asleep-by; a
  small one with readiness and asleep-by; an evening small widget that leads
  with asleep-by; and a waiting state.
- **Compatibility** groups watches by capability; only the Forerunner 970 is
  named. The page has its own stylesheet and isn't linked from the home page.

## Open questions

Each has a recommendation. Items marked *Rules* need a line in the spec's
Rules section, because they're Raring's own math.

1. **Week start.** Recommend following Garmin's start-of-week setting, so
   intensity minutes match Garmin's weekly total. The mockup uses Monday.
2. **Pace tick** on each ring (where even progress would be today, with rest
   days skipped). Recommend keeping it: it's the only "am I behind" cue.
   *Rules.*
3. **Asleep-by counting.** Recommend counting a night as on time within 15
   minutes, and leaving nights without a plan out of the count. *Rules.*
4. **Intensity target.** Recommend Garmin's intensity minutes and Garmin's
   goal by default, with "Vigorous only" as an option. Use Garmin's weekly
   total; don't recompute it.
5. **Zone 2 source.** Recommend activity time in Garmin's zone 2 only. Check
   whether all-day zone time is available before the spec says either way.
6. **Rest days** only move the pace tick and label Today ("Planned rest");
   they don't lower targets. "Pause this week" leaves the week out of
   history. Recommend as mocked.
7. **Sets per muscle.** Recommend counting a set once for each primary muscle
   in Garmin's catalog and ignoring secondary muscles. If the catalog has no
   muscle data, keep a small mapping table. *Rules.*
8. **Miscount flags.** Recommend three to start: 0 reps; reps at least 1.8
   times the median of the exercise's other sets ("doubled"); and an exercise
   Garmin couldn't name. Flags never fix anything automatically. *Rules.*
9. **Readiness brake timing.** Readiness only exists on the morning itself.
   Recommend scheduling with progressed weights, then swapping in held
   weights after the morning sync on a Rest or Easy day, with a
   notification. This only works if the watch syncs before the session, which
   hasn't been checked on the Forerunner 970. If the spike says no, show the
   brake as advice on Today instead.
10. **Progression defaults.** Recommend double progression with the smallest
    equipment step; add reps first when the next step is over 10% heavier;
    drop one step after two sessions below the range; wait for two sessions
    on a new exercise. *Rules.*
11. **Step cap.** The mockup counts each set and its rest, with warm-ups
    included and the last rest dropped. Recommend warning at 90 steps and
    offering "split workout" at 100. The spike needs to confirm how Garmin
    counts repeats.
12. **Weekly repeat** ("Every Sat"). Garmin schedules one date at a time, so
    Raring would have to reschedule every week. Recommend shipping "Once"
    first.
13. **Exercise suggestions when fixing** an unnamed exercise. Recommend
    taking them from the scheduled workout, then recent exercises.
14. **Widget evening switch.** Recommend that the small widget leads with
    asleep-by three hours before the asleep-by time, while the medium widget
    always shows both. Widgets are native per platform, so iOS would reuse the
    layout with WidgetKit.
15. **Compatibility page.** Recommend publishing it only when the features
    ship, and naming only models someone has tested. It links to
    `issues/new?labels=compatibility`, so a `compatibility` label or issue
    template needs to exist first.
16. **Units.** Recommend following Garmin's unit setting (kg or lb) rather
    than adding a Raring setting.

Not mocked: empty states (no strength history yet), saving offline, and a
failed save. For a failed save, recommend keeping the edits on the device,
fetching the sets again before the `PUT` in case they were edited in
Connect, and saying plainly what Garmin rejected.
