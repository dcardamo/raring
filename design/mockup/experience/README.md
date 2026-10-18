# Raring experience prototype v2 — unshipped

Stage 1 of item 26. The prototype uses the shipped Svelte components and view
contract, with independently synthetic data. No credentials, Garmin writes,
analytics, external fonts or map tiles. Production behaviour remains unchanged.

**Preview:** http://neptune:48726/ · first visit opens mixed Wednesday. **Scenario**
selects simple, sleep-first, mixed or sparse; U01–U64; time/day; offline; sync;
background/resume; naps; calendar coverage; status surfaces; legacy links; and
**Compare** (shipped | retired v1 | v2 captures). Tabs: Today · Plan · Training ·
Health · Progress; the header search sheet finds any metric. Explore is retired
and `#explore` opens Training.
Prototype-only choices persist in this browser. Export is disclosure-only.

**Comparison provenance:** coordinator routing records identify `b965aa4` as the
Opus 5.5 checkpoint A. The session switched to GPT-6 Astra at 13:05Z; completion
`adbb43e` is therefore mixed-model. Do not label the whole v2 result Opus-only.
This preview owns port 48726; Dan's independently isolated comparison candidate
owns 48727 and is not inspected or modified by this lane.

## Run and check

From the repo, inside `nix develop`:

```sh
cd app
npm run experience                  # separate Vite entry: /experience.html
npm run check                       # real view-contract type checks
npm run test:experience             # U starts, journeys, layout/privacy checks
npm run test:experience -- --shots   # regenerate synthetic captures here
npm run build:experience            # app/dist-experience/index.html
```

Serve **only** `app/dist-experience`, never the repository or `.data`:

```sh
nix develop -c python3 -m http.server 48726 --bind 0.0.0.0 --directory app/dist-experience
```

The active server's verified PID is in ignored `.data/experience-preview.pid`.
Stop only that verified PID. To check the served build:
`cd app && EXPERIENCE_URL=http://neptune:48726/ npm run test:experience`.

Source: `app/src/experience/`. Shipped screen extraction: `app/src/lib/screens/`.
Provenance, proposed period rules, parity and evidence limits:
[`docs/experience-design.md`](../../../docs/experience-design.md).
The vanilla v1 renderer is retired; git keeps it and Compare keeps its captures.
The original mockup at `../index.html` still describes the shipped app.

## Unfamiliar-person walkthrough

Choose the case before showing the screen, dismiss the case label, and give no
navigation hints. Record first meaning/action time, wrong turns, extra setup,
source confusion and pressure. Keep consented observations in ignored `.data/`
or `.research/`, never committed.

1. **U01:** “What matters now? What would you do next?”
2. **U03:** “How was last night? What about tonight? Is there a pattern?”
3. **U19:** “Find your workout and explain what counted. Find strength bests,
   a rare metric, gear, a comparison and exported data.”
   **U41:** “What is left of this week’s goals? Deal with the session you
   didn’t record. Is the Garmin plan yours?”
4. **U11:** “Plan something tomorrow, then choose a rest day.” Repeat offline
   and after resume. “What changed? What didn’t?”
5. **U30/U31:** “What does the restriction do? Is this routine compatible?”
   Misunderstood compatibility blocks progression.
6. **U52/U53:** “Find lifestyle logs, decide whether to allow reads, open,
   leave, return, turn reads off, and explain what an export contains.”
7. **Interrupted week:** Monday strength → Tuesday wind-down self-report →
   Wednesday short night/conflicting intervals → rest through midnight →
   Thursday pause → Friday high readiness without a training prompt → Sunday
   explicit resume and review. Inspect the original targets and pause interval.

At least three unfamiliar participants, including simple-needs and
detail-oriented people, must complete the usability gate. Browser checks and
AI review are not participants. Android/device, real data, long-history
performance and migrations remain unverified. **Stop before stage 2** for the
coordinator/Dan decision: proceed with named limited evidence, or pause and
release exclusivity. No main merge, push or release is authorised.
