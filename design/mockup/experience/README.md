# Raring redesign — the approved design record

The captures here are the design Dan approved on 2026-10-19 and that shipped
as item 26: Today · Plan · Training · Health · Progress, Profile from the
header, Adapt on every root page and short "?" help. They came from the
stage-1 prototype, which ran on synthetic data and has since been removed;
git keeps its source (`app/src/experience/` before the cutover commit).

- `shots/`: synthetic captures by checkpoint (P1–P5) and journey (U-cases),
  at phone, unfolded and desktop widths.
- `fonts/`: the Inter and Outfit files the captures use, with their licences.

`docs/spec.md` describes what ships; `docs/experience-design.md` holds the
decisions, the parity ledger and the U01–U64 comparison of the app against
this design. The original mockup at `../index.html` shows the app before the
redesign.

## Unfamiliar-person walkthrough

Still to do: at least three unfamiliar participants, including simple-needs
and detail-oriented people, on the shipped app (a debug build with the
synthetic demo, or their own account with consent). Give no navigation
hints. Record first meaning and action time, wrong turns, extra setup, source
confusion and pressure. Keep consented observations in ignored `.data/` or
`.research/`, never committed.

1. "What matters now? What would you do next?" (Today)
2. "How was last night? What about tonight? Is there a pattern?" (Sleep)
3. "Find your workout and explain how it counted. Find strength bests, a rare
   metric, gear, a comparison and exported data."
4. "What is left of this week's goals? Deal with a session you didn't record.
   Is the Garmin Coach session yours?"
5. "Plan something tomorrow, then choose a rest day." Repeat offline and after
   resume. "What changed? What didn't?"
6. "What do your limits do? Is this Garmin workout compatible?" Reading it as
   checked for compatibility is a failure.
7. "Find lifestyle logs, decide whether to allow reads, open, leave, return,
   turn reads off, and explain what an export contains."

Browser checks and AI review are not participants.
