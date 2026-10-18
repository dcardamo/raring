# Raring experience prototype — unshipped

Stage 1 of item 26. Independently synthetic data; no account connection, Garmin
writes, analytics or remote assets. Existing `../index.html` and the product app
remain unchanged. This is a journey prototype, not a second implementation of
the Rust core. Source-truth and native behaviour need later implementation tests.

Open `index.html` directly, or serve **only this directory**:

```sh
nix develop -c python3 -m http.server 48726 --bind 0.0.0.0 --directory design/mockup/experience
```

On neptune: `http://neptune:48726/index.html`. The current review server PID is
recorded privately in `.data/experience-preview.pid` at the repository root.
Do not serve the repository root or share private observations.

## Controls and checks

The purple **Scenario** control is test tooling, not product navigation. Choose
one of four profiles or U01–U64, change the day/phase, simulate offline/resume/sync,
and inspect illustrative status surfaces or legacy links. Choices persist in a
prototype-only browser storage key; **Reset choices** resets the synthetic profile.
The clock selector represents existing personal phases, not a new fixed coaching
clock. `?case=U13` loads a case once; `?text=1.3` or `?text=2` scales text.

```sh
nix develop -c node design/mockup/experience/check.mjs --shots
```

Requires `nix develop -c npm ci --prefix app`. The script uses the project's
Playwright and installed Chromium; `CHROMIUM`, `RARING_NODE_MODULES` and
`EXPERIENCE_URL` override paths. Results are written to ignored
`.data/experience-check.json`; synthetic screenshots go to `shots/`.
Cases contain expected start-page copy, not claims that every journey passed
with real people. See the stage-1 checkpoint in `docs/experience-design.md`.

## Short unfamiliar-person walkthrough

The moderator chooses the case **before** showing the screen, dismisses the case
label, and does not explain the menu. Ask what the screen means before hints.
Record time to meaning/action, first destination, wrong turns, configuration,
source confusion and pressure. Keep consented observations under ignored `.data/`
or `.research/`; do not commit participants or personal screenshots.

1. **U01**, new/no-plan: “What matters now? What would you do next?”
2. **U03**, sleep-first: “How was last night? What about tonight? Is there a pattern?”
3. **U19**, mixed: “Find your workout and explain what counted. Where would you
   find strength bests, a rare metric, gear, a comparison and your exported data?”
4. **U11**: “Plan something for tomorrow. Now choose a day of rest.” Return after
   simulated offline/resume. “What changed? What didn’t?”
5. **U30/U31**: “What does this restriction do? Is this routine compatible?”
   Misunderstood compatibility is a blocker, not a partial success.
6. **U52/U53**: “Find your lifestyle logs, decide whether to allow reads, open
   them, leave, return, turn reads off, and explain what an export still contains.”
7. **Interrupted week:** choose Mixed, Mon 16 Nov daytime. Open recorded strength;
   advance to Tue evening and mark wind-down; Wed morning inspect the poor night;
   Wed daytime review intervals and rest. Advance past midnight (rest persists),
   Thu daytime pause training, Fri inspect high readiness and status surfaces,
   Sun explicitly resume, inspect weekly review and week 46's original targets.
   Add an activity exclusion/movement note and an optional supplied routine if
   relevant. “Did the pause create a failure, catch-up task or achievement?”

At least three unfamiliar participants are required, including simple-needs and
detail-oriented people. Browser/AI reviews are not participants. If discovery
fails, reconsider Explore as primary navigation rather than coaching people to
use it. **Stop before stage 2** for the coordinator/Dan decision: proceed with
explicitly named limited evidence, or pause and release exclusivity. No merge,
push, release or automatic next stage is authorised.
