# A focused, adaptive Raring experience

## Status and scope

This is the target product and interaction design for TODO item 26. It is
not a description of features already shipped. `spec.md` remains the contract
for the current implementation; implementation must update it as this design
ships. All example people, restrictions, messages and achievements below are
illustrative, not a record of anyone's health or Garmin account.

This is **one initiative, including the usability work**, not a feature
redesign followed by a separate UX cleanup. Information architecture,
progressive disclosure, motivation, adaptation and discoverability must be
designed and evaluated together. A feature is not done because its data renders.

Start only after the already in-flight **Raring 9+10: Status notification and
widgets** workspace is complete, integrated and checked. Its coordinator must
confirm completion and that no other Raring implementation lane remains active;
an idle agent or finished coding without integration is not sufficient. Then
start item 26 without opening another backlog item between them.

Implementation is exclusive and sequential. Once item 26 begins, do not run
other feature work, independent redesign branches or parallel implementation
lanes. Its stages are checkpoints in one workstream, not separate projects to
implement concurrently. Read-only reviews and checks can support that workstream.
An urgent unrelated fix requires an explicit decision to pause this initiative.
Do not silently expand it to include the rest of the backlog. Time-sensitive
verification, such as a real daylight-saving transition, may continue as checks
and feed this initiative; the freeze is not a reason to miss that evidence.

Use one integration branch for the coordinated cutover. Every push to `main`
creates a release, so intermediate shell/route changes must not be merged there
as unfinished stages. Keep the current app releasable on `main` until the
integrated experience passes the release gate. Commit sequential checkpoints
on that branch; this is not permission for parallel implementation branches.
If a smaller independently releasable slice is proposed, agree its complete user
journeys and migration boundary before merging it. No automatic push or release
is authorised by this document.

The baseline includes **shipped naps and the consented Garmin journal**, as well
as thresholds, gear, planned training, strength, long-range trends and the weekly
plan. These are not new endpoint projects and must not disappear during redesign.
The start gate also brings in items 9 and 10's notification/widgets: preserve
them and adapt their shared context, suppression and privacy rules rather than
leaving an older motivational experience running alongside the new one.

### Contents

1. [Product promise](#1-product-promise)
2. [Start simply and grow](#2-the-experience-grows-with-the-person)
3. [Foundations, ambitions and commitments](#3-meaning-without-an-elaborate-goal-system)
4. [Information architecture](#4-information-architecture)
5. [Today](#5-today-a-briefing-with-an-ending)
6. [Timing and precedence](#6-timing-precedence-and-stability)
7. [Plan and migration](#7-plan-intentions-without-homework)
8. [Rest, restrictions and rehabilitation](#8-adapt-to-the-person-not-just-their-readings)
9. [Training and contribution](#9-training-preserve-different-kinds-of-progress)
10. [Sleep and naps](#10-sleep-and-recovery-including-naps)
11. [Health and journal privacy](#11-health-and-sensitive-information)
12. [Progress and motivation](#12-progress-and-person-serving-motivation)
13. [Bounded personalisation](#13-personalisation-with-little-configuration)
14. [Preservation and implementation boundaries](#14-implementation-boundaries-and-preservation-map)
15. [Usability criteria and U01–U64 journeys](#15-usability-evaluation-journeys-not-screenshot-approval)
16. [Sequential delivery and completion](#16-sequential-delivery-and-completion)

## 1. Product promise

**Know what matters today. See what your effort is building.**

Raring helps a person understand their situation, make a useful choice, recognise
what they have done, and prepare for what comes next. Garmin supplies recorded
measurements and estimates. The person supplies intentions and circumstances.
Raring connects them with small, explicit, local rules.

The product loop is:

**Understand now → choose or follow a plan → recognise progress → adapt.**

A successful interaction may take ten seconds and end with the person closing
the app until tomorrow. More visits, longer sessions, uninterrupted watch wear
and more exercise are not product success measures.

### Principles

1. **Several things can matter; only one needs to lead right now.** Sleep,
   physical capability, longer-term health and an event can coexist. Do not
   require a primary goal or combine everything into a new wellbeing score.
2. **Value before configuration.** After sign-in and the first usable sync,
   Today must help without goal setup, dashboard editing or a tutorial tour.
3. **Readiness is not suitability.** Injury restrictions, illness, a chosen
   rest day and the person's plans outrank an attractive readiness number.
4. **Actions and outcomes are different.** A completed session is evidence of
   participation. It is not proof of improved fitness or a lower Fitness Age.
5. **Adapt the message, not the map.** Stable destinations and controls support
   changing content. Never make a feature disappear because it was used less.
6. **Depth is invited, not imposed.** A useful summary opens its explanation,
   then the underlying data. Advanced controls do not occupy the overview.
7. **Recovery is legitimate.** No catch-up debt, lost status, compulsory streak
   or pressure to turn a high readiness reading into a workout.
8. **Unknown is not failure.** Partial history, late uploads and unsupported
   readings limit claims, not the person's achievements.
9. **Quiet by default.** One leading message, bounded supporting content and
   notifications only with permission. No motivational feed to keep scrolling.
10. **Private and explainable.** No cloud coach, advertising profile or remote
    behavioural analytics. Every suggestion can explain the facts behind it.

### Non-goals

- A new strain, biological age, muscle-recovery or overall performance score.
- Medical assessment, injury diagnosis, exercise clearance or treatment plans.
- Automatically generated race programmes, rehabilitation prescriptions or
  workout progression. The separately gated workout builder remains separate.
- Automatically changing Garmin calendars, workouts, settings or watch alarms.
- Social feeds, competitive leaderboards, engagement rewards or wearable-use
  streaks. No reward for opening the app or producing complete data.
- Making all Garmin endpoints available on overview screens.
- A dashboard editor or a lengthy preference questionnaire.

## 2. The experience grows with the person

These are usage situations, not account tiers or modes the person must select.
People can move between them on any visit.

| Situation | Immediate value | What must not be required |
| --- | --- | --- |
| Just looking | Understand today, last night's sleep and one useful next step | A goal, plan, tutorial, notification permission or full history download |
| Following a routine | See recorded activities, tonight's preparation and a weekly pattern | Manual workout logging or a personal dashboard |
| Making a plan | Add an intention or a small weekly commitment | Configuring every target, sport and rest day |
| Working toward something | Connect an event or aspiration with commitments and evidence | Choosing one exclusive goal or accepting a generated programme |
| Adapting to life | Rest, pause or exclude unsuitable activities | Rebuilding the plan or disclosing a diagnosis |
| Investigating | Find metrics, exercise details and comparisons predictably | Training the personalisation system or finding a transient card |

### First use

- Explain Garmin access and local privacy, sign in, and show usable cached data
  as soon as it arrives. Background history download is not an onboarding gate.
- No mandatory goal selection. Do not silently adopt preset targets as chosen
  commitments. An existing Garmin weekly goal may be shown, labelled as Garmin's.
- Explain only a consequential missing prerequisite: for example, the watch
  needs to upload to Garmin Connect. Do not display a wall of empty features.
- Keep Plan, Progress and Explore usable before a plan exists. Empty states
  offer one relevant action, not an advertisement for everything the app can do.
- Ask for notification permission when a person requests a reminder, not as a
  prerequisite to seeing their data. Do not repeat a declined permission prompt.
- Introduce a capability at its point of use. Examples: explain set tracking
  when a strength session has no sets; offer a commitment from a relevant weekly
  summary; explain comparisons when opening a trend.

### Configuration budget

Most people should never open Settings to make the core experience useful.

- A lightweight intention can be added in one short sheet: choose an activity
  and day, with time optional. No goal wizard first.
- A commitment asks for only its quantity and period; rest days and refinements
  are optional, and the existing Garmin goal is not duplicated unnecessarily.
- Circumstance controls ask what to change, not why the person is injured or ill.
- More consequential configuration is justified only at the point it is needed:
  restrictions, journal consent, reminders and local data export.
- Settings retains an escape hatch for fixed time-of-day presentation, quiet
  celebrations, personalisation reset and notification/privacy controls. These
  are controls, not prerequisites.

## 3. Meaning without an elaborate goal system

Use four distinct concepts in the experience. Foundations are fixed product
framing, not another kind of user-configured or stored goal. Only ambitions,
commitments and intentions require corresponding local records:

| Concept | Meaning | Example |
| --- | --- | --- |
| Foundation | An enduring area of wellbeing, not a compulsory target | Sleep and recovery; activity and physical capability; longer-term health |
| Ambition | Optional direction, possibly with a date | Prepare for an event; maintain strength; improve Garmin Fitness Age |
| Commitment | Something the person has chosen to do | Two strength sessions in a week |
| Intention | A dated local plan, not proof of completion | A strength session tomorrow evening |

Measurements and recorded activities are evidence, not additional goals.

### Foundations

Sleep is always relevant, even without a sleep target. Recovery and activity
remain relevant without an event. Garmin Fitness Age can be available as a
longer-term indicator without a setup step; it is not a grade of the person or
a required number to reduce. Missing Fitness Age does not leave the experience
without a long-term purpose.

A person may choose several ambitions. Do not ask them to rank their life or
pick a single permanent focus. The leading message is selected for this moment,
not by hiding other ambitions. An activity can be relevant to several ambitions,
but its underlying evidence is counted only once per applicable measure.

### Optional ambitions

- An event can come from verified Garmin calendar data or a clearly labelled
  local event entry. A local date is not a Garmin race registration or a plan.
- A race countdown is context, not proof of adequate preparation. Do not infer
  taper periods, required distances or achievable finish times.
- A Fitness Age ambition uses Garmin's estimate and reported drivers. Do not
  imply that strength contributes to that formula without Garmin evidence, or
  that better sleep will produce a particular age change.
- Strength and cardio do not need to be explicit ambitions for their activities
  to be recognised. People can simply do them and see what is recorded.
- Suggested commitments require acceptance. Observing a habit never silently
  turns it into an obligation.

### Progress versus outcome

Show these separately:

- **Follow-through:** recorded activities and chosen commitments met.
- **Measured change:** Garmin trends, recorded performance and exercise bests.

"You kept your routine; the estimate has not changed yet" is a valid message.
Never invent an improvement to justify participation. Do not show a percentage
such as "70% race-ready" or "80% healthy".

## 4. Information architecture

### Working navigation design

Prototype **three stable task destinations: Today, Plan and Progress**, with a
persistent, text-labelled **Explore** entry in each root header. Explore opens
the full library; it is not inside Settings, an overflow menu or a hidden gesture.
Settings is a separate conventional control.

This is a testable navigation proposal, not a requirement to preserve a number
of tabs. Three task destinations describe the daily loop; Explore is the stable
escape route to any data. If the prototype's library entry fails the discovery
criteria in section 15, promote Explore into primary navigation before building
the shell. Do not preserve three destinations at the cost of hidden features.

On larger screens the same destinations can use a rail. Reading order and
content hierarchy remain the same; extra width does not justify displaying
all the details at once.

| Destination | Question | Owns |
| --- | --- | --- |
| Today | What matters now, and what counted today? | Current context, one next decision, recent accomplishments, tonight's preparation |
| Plan | What have I chosen, and what is coming? | Coexisting ambitions, commitments, schedule, local intentions and adaptations |
| Progress | What is adding up or changing? | Weekly review, participation history, meaningful changes and milestones |
| Explore | What do these readings mean? | Stable access to Sleep, Readiness, Training, Strength, Activities and Health details |

### Destination rules

- Keep direct Today links to Sleep and current-state details. A person who only
  wants last night's sleep must not learn a planning architecture first.
- Explore's first view includes plainly labelled Sleep, Readiness, Training,
  Strength, Activities and Health entries. No category is removed because it
  has not been visited. Sparse destinations explain their limitations locally.
- Give each detail one canonical route, with several useful entrances. A workout
  opened from Today, Plan or Progress is the same workout detail. A metric's
  full trend/comparison view likewise has one canonical detail route; Progress
  shows a preview linking there, not a second independent analysis screen.
- A relevant card opens the relevant section: a timing observation opens sleep
  timing, not the top of a long Sleep page. Preserve the target section on Back.
- Back returns to the actual caller, preserving date, filter and scroll. Android
  Back, browser history, external links and notification links must agree.
- Retain old deep links with aliases where routes change. Never redirect a valid
  old workout or metric link silently to Today.
- Stable labels and locations are more important than reducing the visible
  navigation by one button. Icons supplement text; they do not replace it.
- A compact local search can help find named metrics and exercises in Explore,
  but browsing must work without knowing the technical metric name.

### Disclosure, not concealment

Every surface has three depths:

1. **Meaning:** a short statement, one useful visual and a clear next action.
2. **Explanation:** two or three supporting facts, source, dates and limits.
3. **Evidence:** complete measurements, activities, charts and comparisons.

A collapsed wall of twenty accordion headings is still a wall. Move genuine
analysis into focused destinations rather than merely folding all the cards.
No necessary explanation should depend on reading documentation outside the app.

## 5. Today: a briefing with an ending

### Content hierarchy

At ordinary phone text size, the first viewport should contain enough to answer
"what matters now?" and act. Do not force a full-screen decorative gauge before
the answer.

1. **Context header:** date, quiet watch-upload freshness, Explore and Settings.
   Keep a text-labelled "Adapt today" action in a stable position even when no
   adaptation is active; it changes to its state, such as "Rest today". Plan has
   the same entry. Use a second header row if necessary rather than squeezing
   date, freshness, Explore, Settings and adaptation into one narrow line.
2. **Leading card:** one opportunity, chosen intention, accomplishment or sleep
   step. A short headline, at most two short explanatory lines, one primary
   action and an optional "Why this?" link.
3. **Compact context:** relevant readings and direct links, not three dominant
   gauges. Readiness, energy and sleep are possible facts, not mandatory panels.
4. **What counted today:** at most two preview rows, followed by "Today's
   activities" when there is more. Recorded naps can be a quiet dated event,
   never a sleep-completion badge.
5. **One supporting look-ahead or progress line:** for example tonight's
   wind-down or a relevant weekly commitment. No full plan dashboard.

The ordinary default page should have a natural stopping point within roughly
two phone viewports. The overview, leading action and essential exception must
not require that full scroll. Larger text may require more space; never shrink
text or hide the exception to hit a height target.

An important data or health notice may replace the lead or take a compact
priority position. Multiple notices are grouped with a clear count and an open
action, not silently suppressed. A low score does not make the entire screen red.

### Example leading states

| Situation | Message intent | Action |
| --- | --- | --- |
| Fresh high readiness, no intention | Training is an option if it fits the person and their week | Plan a session |
| High readiness on a chosen rest day | Acknowledge readiness without undoing rest | View plan, not "push" |
| Scheduled session | Show what the person already planned; qualify with current evidence | Review session |
| Completed workout | Recognise what was recorded and its direct contribution | Open workout/progress |
| Low readiness | Explain that an easier day may fit; keep the scheduled session separate | Review today |
| Evening | Make room for tonight's sleep | See tonight's plan |
| Wind-down or late night | Quiet support, no exercise deficit or countdown | Optional sleep detail |
| New or delayed data | Explain what is not yet known without reusing old advice as current | Relevant sync/help action |

These messages are rule-based templates with evidence, not generated coaching
paragraphs. The actual wording must be tested for clarity and pressure.

### What counted today

For a recorded activity, the app may show:

1. **Direct contribution:** counted toward an accepted commitment, or a neutral
   session count if there is no target.
2. **Garmin assessment:** the recorded training effect or benefit, attributed
   to Garmin without strengthening its claim.
3. **Measured progress:** an eligible exercise record or a separately evidenced
   longer-term change. Never infer an age change from a session.

Do not show all three by default. Choose the most useful supported statement;
the workout detail contains the rest. A workout may be worth acknowledging
without any improvement, record or target. A small load is not a poor workout.

Late sync credits the recorded day. Backfill must not unleash a queue of old
celebrations on Today. Reopening the app does not replay a celebration already
seen. Corrections update evidence without scolding the person.

### A day can finish

When the person's chosen action is complete, recognition can remain the lead.
Do not immediately create another task. When sleep preparation becomes relevant,
it can lead without erasing the day's activity. During wind-down, progress is a
quiet summary rather than an invitation to open charts or protect a streak.

## 6. Timing, precedence and stability

### Hard eligibility before relevance

Do not use one opaque priority score to trade safety, privacy and commitments
against each other. Apply gates first, then choose among eligible messages.

1. **Privacy:** consent and explicit-open requirements determine what can appear.
   Sensitive journal content never becomes an adaptive message.
2. **Evidence:** source, date, freshness and coverage must support the claim.
   Missing readiness does not invalidate a known recorded workout, but it blocks
   a readiness-based effort suggestion.
3. **Circumstances:** illness pause, restrictions and explicit rest constrain
   exercise prompts. A high score cannot override them.
4. **Existing decisions:** respect scheduled sessions, accepted local intentions,
   dismissals and stated availability before proposing new activity.
5. **Garmin guidance:** use its readiness/effort rules and load constraints; do
   not manufacture a daily exertion budget or medical clearance.
6. **Time and feasibility:** use the relevant day phase and any declared slot.
   A weekly gap is context, not a command to catch up tonight.
7. **Relevance:** choose the supported message most useful now, with a stable
   tie-break and one leading message.

The existing personal wake/sleep phase rules, missing-night handover, timezone
and daylight-saving protections remain the starting point. Do not substitute
arbitrary fixed morning/evening hours. Without credible timing, use a neutral
fixed presentation, explain the limitation and avoid guessing the person's day.

### Dates are not interchangeable

- Sleep/readiness can retain the last night until the existing day handover.
- Live intentions, activities, scheduled events and the journal use their
  appropriate actual local calendar dates, labelled separately when needed.
- A prospective plan for tonight remains tied to the night across midnight.
- Historical views never create new intentions, advice snapshots or adherence.
- A nap with endpoints on different dates displays both dates.

Never merge these into a convenient global "today" key during the redesign.

### User decisions persist

- "Not today" dismisses the same suggestion for that local day. A sync or a
  navigation round trip does not resurrect it. A different genuine exception
  can still appear, with a different reason.
- A selected workout time remains an intention, not inferred availability for
  every future day. No calendar access is required.
- "Rest today" covers that day's exercise prompts and the following overnight
  period; exercise prompting resumes only in the next waking-day context.
- Longer pauses and restrictions do not expire because readiness improves.
  An optional review date prompts review, not automatic resumption.
- No moving controls under a finger, switching routes, changing a selected
  historical date or rearranging an open detail after background sync.
- Cache the last usable view; update at an idle boundary. Preserve explanations
  for currently displayed advice and let the person see why it changed.

### Notifications

Do not introduce a new motivational notification stream. Reuse the same eligible
focus and suppression rules for existing opt-in reminders. Dismissed, paused or
restricted advice must not reappear through notifications. Never expose injury
notes, rehabilitation names, journal contents or weight on the lock screen.
A reminder is not a reason to reopen the app to claim credit.

For existing channels: morning readiness must not contain a training call during
rest, illness or a conflicting restriction; a requested factual readiness update
can remain neutral. Metric-change notifications are opt-in factual updates, not
new activity demands. Weekly check-in links open the canonical Progress review,
including aliases for old links. Bedtime reminders retain existing consent and
night identity, and do not become sleep-score or adherence warnings. Existing
non-diagnostic vitals notices remain separate from motivational messages.

## 7. Plan: intentions without homework

The first view answers "what have I decided?", not "what should I configure?".

- A compact week strip shows scheduled items, local intentions, recorded
  activities and rest distinctly. Do not communicate their differences by
  colour alone.
- Show accepted commitments separately from appointments. A weekly target is
  not seven tasks automatically placed on a calendar.
- Ambitions, including events, coexist in a short section with their dates or
  directions. The absence of an event removes the countdown, not the plan.
- Expose one straightforward "Add to my plan" action. Activity type and day
  are enough for a local intention; time and association with an ambition are
  optional. Editing or removing it does not write to Garmin.
- Existing Garmin Coach/scheduled content remains read-only and explicitly
  attributed. Unknown completion/cancellation fields stay unknown.
- Never mark a Garmin scheduled workout complete by matching an activity's name
  or time. For a local intention, offer same-day, same-type recorded activities
  alongside it and a one-tap "This was my session" association. This is an
  explicit user-confirmed link, not proof of Garmin schedule completion.
- Recorded activities always earn their factual contribution without that link;
  clearing local intentions is optional, not a workout-logging chore. Unresolved
  intentions become "not linked" in history, never assumed missed workouts.
  Local self-reports, user-confirmed links and Garmin completion are distinct.
- Only user-accepted commitments receive a completion state. Do not populate
  silent default strength, Zone 2 or sleep targets.
- Keep the existing plan-week alignment with Garmin's intensity-week data. Label
  all periods, including different windows used by Fitness Age drivers.
- No assumed access to the phone calendar, inferred free time or automatically
  generated race or rehabilitation schedule.

### Changing expectations

A person can pause, reduce or revise future commitments without a failure screen.
Show the change honestly as an adjusted plan, not a retroactive perfect week.
Preserve prior target versions, evidence and dated achievements. Do not assign
new targets to weeks before setup or rewrite old weeks after a later change.

Current-week revisions retain original/revised expectations, effective dates and
pause intervals. A lower target already satisfied by recorded work displays
"Reached · adjusted" from the revision onward, without a retroactive completion
celebration. The revision itself cannot create a whole-week achievement; later
recorded actions can earn recognition against the revised commitments normally.
Independent exercise records remain eligible. Already-earned dated achievements
remain attached to their original evidence and target version. Do not introduce
"perfect week" badges that make an honest adjustment a loss. This extends the
current per-week versioning rather than relying on its overwrite behaviour.

A whole-week pause is "Paused". A mid-week pause is "Adjusted · paused since …":
keep the factual activity totals and original targets for reference, suppress
behind/on-track pace verdicts, and do not prorate targets or award completion
merely for pausing. Actual actions can still be recognised. The person may
explicitly revise targets; returning creates no catch-up sessions.
Resume is explicit, with the existing commitments shown for review, not silently
increased or reduced. An event remains visible but cannot override restrictions.

Migration treats saved plan versions as accepted commitments, and
`configured: false` as no accepted commitments regardless of the preview preset.
Existing whole-week pauses and historical results retain their semantics. The
current migration week is labelled adjusted where semantics change, not silently
re-scored; sleep timing has the specific migration below.

## 8. Adapt to the person, not just their readings

"Adapt today" must be visible without finding Settings or a recommendation.
Simple day-level choices are fast; longer-lived constraints are progressive.
These are composable circumstances, not mutually exclusive diagnoses.

| Circumstance | Behaviour |
| --- | --- |
| Rest today | Suppress new exercise prompts; retain chosen schedule as context and sleep help |
| Illness/recovery break | Pause training expectations until the person resumes; do not diagnose from sensors |
| Training with limitations | Enforce activity exclusions; require review for movement restrictions with unverified workout contents |
| Rehabilitation routine | Offer only the routine the person or their clinician supplied; record voluntary self-reported completion |
| Busy/travelling | Offer a smaller or paused plan without inventing a health reason |

If several apply, the more restrictive exercise gate wins. Rest and illness do
not hide ordinary access to recorded data or make the user unable to edit plans.

### Restrictions

- Ask what to avoid, not for a medical history. Ship structured activity-type
  exclusions and a visible movement/exercise restriction note; no diagnosis is
  needed. Avoid building a clinical movement ontology or exercise recommender.
- A movement restriction triggers "review needed" for exercise suggestions and
  routines whose contents cannot be verified, including generic strength plans.
  The first version cannot claim compatibility. The person can explicitly select
  an existing routine they intend to use; that is their choice, not clearance.
- Structured exercise-level matching is gated on verified workout-step contents
  or the separately planned builder. Until then, recording "avoid squats" does
  not mean the app has parsed and screened every Garmin workout.
- Free text remains visible context, not something the system pretends to
  understand. Exact activity exclusions can be enforced; unknown movement
  contents cannot bypass the review-needed state.
- Do not infer a permitted substitute. The person explicitly chooses activities
  or existing routines they intend to continue. Permission is not clearance.
- Do not auto-resume on a date, improved readiness, a new workout or silence.
- A conflicting Garmin item stays visible with a conflict explanation; Raring
  does not silently modify it or nag the person to perform it.
- Changes take effect across Today, Plan, reminders and cached suggestions.

### Rehabilitation, deliberately small

The first version is a local named checklist supplied by the person: exercise
names, optional instructions and optional dates/reminders. No diagnosis-based
exercise catalogue, prescribed load, automatic progression or recovery forecast.
Completion is labelled self-reported. It does not fabricate a Garmin workout,
strength record, intensity minutes or physiological benefit. Keep this distinct
from the later Garmin workout builder and write support.

## 9. Training: preserve different kinds of progress

### Cardio and strength

Training detail gives both clear entrances. Different evidence does not require
identical cards or equal chart counts.

- Cardio: Garmin load/status, load focus, VO2 max, recorded sport performance,
  running/cycling-specific metrics where supplied.
- Strength: recorded sessions, exercises, sets, bests, records and muscle coverage.
  An activity without sets still counts as participation, not strength gains.
- Mixed activities retain Garmin's identity. Never silently classify HIIT as
  strength or assume a bodyweight set produced a weighted record.
- Muscle maps show recorded work, not muscle freshness, injury status or safety.
- Race predictions, thresholds, hill score, running tolerance and gear belong in
  the relevant detail. Their existence does not make every user a runner.

### Acute load

Garmin acute load is a weighted measure of recent exercise loads, derived from
its estimates of exercise demand. Strength activities can contribute through
that physiology, but it does not measure local tissue tolerance, muscle recovery
or strength progression.

Present it as context: "How does recent recorded load compare with Garmin's
range for your history?" Not "How much exercise is left today?"

- No load-to-readiness ring to close.
- No target derived by subtracting today's load from a Garmin band.
- No pressure to restore load during illness or injury.
- A brief explanation links to the full range, history and Garmin status inputs.

### Counting contributions

Keep existing evidence coverage and arithmetic contracts. Intensity minutes use
Garmin's moderate plus twice-vigorous weighting; zone minutes are different.
Fitness Age vigorous averages have their own longer window. Name each, and do
not imply that their totals are interchangeable. Missing activity lists/types,
zones or set details do not become zero or a failed commitment.

## 10. Sleep and recovery, including naps

Sleep is a foundation, not a performance competition. The story is:

**Notice a pattern → make room for sleep → prepare calmly → review over time.**

### Sleep destination

Give immediate access to last night and tonight; history is an explicit next
step rather than a long chart inserted between them.

- Last night: recorded duration against that night's recorded need, score and
  a short supported observation. Stages and overnight measurements are deeper.
- Tonight: Garmin's window/need or the labelled existing Raring fallback, with
  wind-down. Preserve the distinction from Garmin's long-term alignment window.
- Patterns: duration, timing, alignment and need across a stated period, with
  gaps. A few short nights can be described without a new rhythm score.
- Preparation: an optional local wind-down intention/reminder, not a promise
  to fall asleep at an exact time or an automatic watch-setting change.

Do not universally prescribe earlier bedtimes. Use the existing timing guidance,
credible wake information and user constraints. No countdown or failure verdict
once the sleep window has passed.

### Sleep shortfall and debt

Garmin's sleep need already accounts for recent sleep history and may include
nap/activity adjustments. A distinct numeric sleep-debt metric requires a
verified Garmin field and meaning; it is not assumed available or a dependency
for shipping this redesign.

Before that, show per-night differences or a clearly labelled descriptive
pattern only when both that night's sleep and need are recorded. Any new summary
formula must be specified and tested in `spec.md`. Never call a running sum a
physiological debt, double-count adjusted need, backfill missing needs from
current values or display hours the person must "repay" tonight.

### Timing adherence versus preparation

Existing asleep-on-time history is an observed comparison against a saved,
prospective plan. Preserve its evidence rules and historical values, but do not
use it as an obedience badge or a nightly pass/fail demand.

Migrate an explicitly saved asleep-on-time target into a **sleep timing
observation** for the current/future experience: retain its chosen frequency as
a reference and display recorded nights neutrally, without behind/on-track
verdicts or inclusion in new action-completion celebrations. Historical weekly
results remain as originally evaluated; the transition week is marked adjusted.
On the first relevant Plan visit, explain this change once and offer, but do not
automatically create, a separate wind-down intention. Defaults that were never
saved do not become observations. Do not silently substitute a self-report habit
for a previously measured target.

Optional wind-down completion is a separate self-report. It must not be inferred
from sleep onset, device wear, a reminder firing or an app open. Neither grants
points for achieving a high sleep or readiness score. The person can review
patterns without adopting any sleep commitment.

### Naps are already shipped

- Retain separately recorded naps with source, duration and dated endpoints;
  missing endpoints remain missing, including across midnight.
- Empty, unknown/unavailable and stale nap reads remain different states.
- A nap may appear quietly in today's recorded events and in Sleep. Do not
  encourage naps to earn achievements or close an overnight sleep target.
- Never add nap minutes to main-sleep duration, its score, timing adherence or
  Raring's own bedtime calculation. Garmin's reported nap adjustment to tonight's
  need remains the authoritative separate input.
- New Garmin need can update tonight's current guidance. It does not rewrite
  the immutable prospective plan already saved for adherence.

## 11. Health and sensitive information

Health detail owns vitals, all-day heart rate, stress, weight/body composition,
Fitness Age and the journal. Their summaries can link from elsewhere without
replicating full dashboards.

- Show Fitness Age as Garmin's estimate with its date, trend coverage and named
  drivers. Its attainable estimate is not a promise or an overall wellbeing
  score. Avoid unrequested weight-loss goals or weight notifications.
- Explain differences between metrics rather than merging them. Sleep HR and
  daily resting HR remain distinct. Current energy, readiness and training
  status answer different questions and have different time horizons.
- Missing or calibrating vitals are neutral, not reassurance that all is well.
  Preserve existing non-diagnostic notices for multiple out-of-range readings.

### Journal privacy survives new navigation

The Garmin journal is off by default and requires consent before any read.
Consent does not make its entries eligible for Today, Progress, personalisation,
search snippets, milestones or notifications. Opening entries remains a second,
explicit action; leaving the journal context or backgrounding hides them.

Preserve dispatch-time consent checks, retry/in-flight protection and the actual
local-day journal window. A null report is unavailable, not an empty journal.
Populated journal fields remain subject to verification.

Turning consent off stops new reads and hides entries; it does not erase already
downloaded logs. Existing export/erase disclosures must remain clear: downloaded
logs are included in later user-created exports until local data is erased.

## 12. Progress and person-serving motivation

### The Progress overview

Lead with one meaningful supported accomplishment or change. Below it, separate
follow-through from measured outcomes; do not average them into a grade.

- Weekly review: one win, one observation, one choice for the coming week.
- Consistency: show repeated participation across a labelled period, including
  paused/unknown weeks and changing target versions.
- Outcomes: a compact trend relevant to recorded activity or an ambition.
- Milestones: dated events with evidence, not a badge wall.
- Other areas remain discoverable through stable named links, even when they
  cannot support a current summary.

No plan does not mean no Progress. Recorded workouts, sleep patterns and Garmin
changes can be useful without a configured commitment. One reading does not
justify a trend; a flat trend does not imply that an effort was wasted.

### Weekly progress board

Use separate tracks for accepted commitments: session markers for counts,
amount bars for minutes, and clearly labelled preparation actions if chosen.
Planned, recorded, self-reported, paused and unknown states are distinguishable
without colour. No single combined body score; extra cardio cannot compensate
for a missing strength session or sleep.

Completing a target can trigger one brief, optional celebration. It stays done;
there is no automatic escalation or extra reward for excessive volume. Quiet
presentation and reduced motion must retain the same information.

### Milestones

Examples include a first completed plan week, repeated participation across
measured weeks, an eligible exercise best, or a new recorded Garmin estimate.
An event date arriving is not proof the event was completed.

- Attribute measurements and distinguish "best in available history" from
  lifetime records. Retain existing strength eligibility/miscount rules.
- Prefer cumulative and rolling-window consistency to fragile daily streaks.
  Denominators and missing periods must be clear, not cherry-picked silently.
- Never reward app opens, watch wear, complete data capture or a physiological
  score as if it were a moral achievement.
- Previous achievements are not lost because the person rests or takes a break.
  Source corrections can correct a factual record; explain them neutrally.
- Pausing/revising a plan cannot itself create a retroactive completion award;
  adaptation is not a loss of status and later genuine actions still count.
- Use recorded dates and stable evidence identities to prevent duplicate
  celebrations, including after backfill, restart, timezone change or sync.
- Keep celebrations out of wind-down and late night. Review them later without
  making the person reopen the app to claim anything.

## 13. Personalisation with little configuration

### Learn relevance, not obligations

Use a small set of inspectable local rules, not a black-box recommendation model.
No server or external personalisation service is introduced.

| Input | May influence | Must not imply |
| --- | --- | --- |
| Recorded sports/exercises | Relevant detail previews and planning choices | Goals, available time, safety or permission to prescribe |
| Credible sleep/wake pattern | Existing day phases and sleep preparation timing | Work schedule, a diagnosis or a compulsory bedtime |
| Current Garmin readings and freshness | Evidence-backed current context | A new physiological score or medical clearance |
| Accepted intentions/commitments | Leading plan context and measured contribution | Automatic future commitments |
| Explicit rest/restrictions/dismissals | Hard prompt exclusions and durable user choices | A diagnosis or automatic recovery date |
| Repeated voluntary use of a non-sensitive detail | A bounded convenient shortcut | That the metric is objectively more important or deserves a health nudge |

Current activity and one visit are weak evidence. Do not flip the interface
because of a single unusual workout or remove strength because injury reduced
recent sessions. Absence of recorded activity is not proof of inactivity.

### Bounded adaptation

- Navigation, core domain links and control positions are stable.
- The leading message and bounded supporting slots can change with context.
- Initially use only watch context and explicit choices for the lead. Usage
  adaptation may promote a detail shortcut after repeated use across separate
  days; it never determines exercise advice or changes the global navigation.
- Implement usage adaptation only if it improves the discovery scenarios over
  the stable baseline. Do not build a learning system merely because it is possible.
- If included, store only small local counters/last-used dates for eligible
  detail routes, not a clickstream, search text or viewed health values. Cap
  counts, retain no more than 30 days, and provide a reset/disable control.
- Resetting learned relevance must not erase explicit restrictions, consent,
  plans or recorded data. Explain these as separate controls.
- "Why this?" names the actual reason. "Less of this" affects optional prompts,
  not the availability of the feature or consequential data warnings.
- Sensitive journal contents, weight concerns and injury notes are not inputs
  to learned promotion. Do not infer illness or injury from watch patterns.

The predictable baseline must remain useful with all learning disabled. There
is no dashboard arrangement the user must maintain.

## 14. Implementation boundaries and preservation map

### Preserve evidence; change presentation

| Existing capability | Target access | Important contract |
| --- | --- | --- |
| Readiness and Why sheets | Today context; Explore → Readiness | Garmin factors, unrated gaps, freshness |
| Body Battery/stress | Today when relevant; canonical details | Synced rather than live; gaps and distinct history measures |
| Sleep/stages/alignment/timing | Today → Sleep; Explore → Sleep | Night dates, window sources, prospective plans |
| Naps | Sleep; quiet dated Today event | Separate from main sleep and Garmin need adjustment |
| Training load/focus/status | Training detail and relevant explanations | No daily budget or muscular recovery interpretation |
| VO2 max, predictions, endurance, hill, tolerance, thresholds | Relevant training/metric detail | Units, dates, stale/missing states, watch availability |
| Activities, routes, splits, zones | Today's activities; Explore → Activities | Recorded source, filters, coverage and canonical workout links |
| Strength sets, muscles, records and help | Explore → Strength; workout/progress links | Timer-only credit, Garmin guesses, missing/flagged sets |
| Gear | Training → Gear | Garmin distances and thresholds; no inferred replacement deadline |
| Weekly plan/setup/history | Plan and Progress | Optional targets, versions, pauses and evidence coverage |
| Garmin schedule, Coach and race | Plan; relevant Today line | Read-only, dated bounds, partial/stale coverage |
| Fitness Age and drivers | Progress; Health detail | Garmin estimate, driver windows, no invented causal credit |
| Vitals and all-day HR | Health; relevant Today context | Non-diagnostic ranges, Sleep HR versus resting HR, elapsed DST axis |
| Weight/body composition | Health only unless explicitly opened | Units, gaps, no weight alerts or implicit target |
| Garmin journal | Explicit Health journal entry | Consent, explicit opening, retention/export disclosures |
| Trends/comparisons | Progress and metric details | Local history, actual bounds, gaps and source differences |
| Settings, help, sync status, export, erase | Stable root access and contextual help | Local-first, offline export, no tokens, consent and destructive confirmation |
| Notifications | Existing opt-in channels, including completed item 9 | Shared context gates, privacy and deduplication |
| Home-screen widgets from completed item 10 | Existing widget placements and canonical tap destinations | Preserve installation/update behaviour; share freshness, adaptations and privacy rules |

### Existing gaps are not shipped features

- General past-day Sleep/Stress detail fetching is not built. Keep existing
  local trends useful; do not promise a complete historic detail from a scalar
  chart. The all-day heart-rate detail already has its own historical path.
- A selectable past weekly-review list must be implemented if exposed here;
  it is not safe to assume that prose in the old spec means a working route.
- Populated Garmin Coach, schedule and race semantics remain partly unverified.
  Do not build automatic completion or programme-specific advice on them.
- Fitness Age history and some performance histories are sparse. No extrapolation
  to create progress stories. A precise numeric sleep-debt feed is unverified.
- Full watch-recorded strength-set compatibility and later Connect edits retain
  the existing testing/refresh limitations. This redesign is not their fix.

### Core and UI responsibilities

Extend the existing Rust view contract, not a parallel UI-only coaching layer.
A displayed message needs a stable identity, kind, eligible period, source dates,
supporting evidence, limitations, destination/action and dismissal semantics.
Use typed structures with explicit unknown states, not a generic rules engine
or an arbitrary ranking score. Every new derived rule belongs in `spec.md`'s
Rules section before it ships: eligibility/precedence, revised commitment and
pause states, milestone eligibility, rolling-window consistency, corrections and
any descriptive sleep summary. Define periods, denominators, unknown states and
versioning explicitly, and test each rule. A motivating headline does not exempt
its underlying calculation from the project's limits on Raring's own math.

Keep local ambitions, intentions, circumstances, routine self-reports and
acknowledgements distinct from Garmin readings. Version the local schema and
export these settings/records with clear privacy disclosure; exclude credentials.
Sign-out/erase and migration tests must account for every new local record.

Relevant integration points:

- `crates/core/src/view.rs`: authoritative view types; regenerate TypeScript.
- `crates/core/src/screens.rs` and `screens/`: existing screen projections and explanations.
- `crates/core/src/plan.rs`: targets, evidence coverage and prospective sleep plans.
- `crates/core/src/naps_journal.rs`: preserve nap and journal source/privacy semantics.
- Existing phase, sleep-plan, trend, sync and notification code: reuse its source
  decisions, request budgets, token serialization and cadence.
- `app/src/App.svelte`, `lib/navigation.ts`, `lib/api.ts` and components: new shell,
  focused screen composition, canonical deep links and cache invalidation.
- Native Android notification/reminder entry points: same adaptation and privacy
  rules as the foreground, not an independent recommendation path.

Core state changes must invalidate cached advice immediately. Network refresh
is not required to apply rest, restrictions or dismissals. New navigation must
not change the request budget or fetch all detail eagerly. Keep the no-network
historical chart path and the current shared on-demand request gates.

### Deliberate changes to the current contract

Implementation must replace, rather than leave contradictory versions of:

- The fixed four-domain bottom-tab requirement and route/prefetch assumptions.
- The three dominant Today gauges and full weekly-plan/Fitness Age cards there.
- The visual-first interpretation that makes a large gauge the default hero.
  Retain concise visual communication, but let meaning and action determine the
  visual; do not replace a stats dump with paragraphs of coaching.
- Long unstructured domain-card stacks and full comparison controls on overviews.
- Default Strength-preset previews being mistaken for accepted commitments.
  The current spec does not require a primary goal, and this design does not add one.
- The old TODO exclusion of all private achievements: person-serving milestones
  are in scope; competitive/engagement gamification remains out.
- The weekly-focus rule that always leads with the biggest deficit. Context,
  recognition and user choices now determine which supported message leads.
- Current-week target revisions, pause semantics and sleep-target migration as
  explicitly defined above, without rewriting historical results.
- The unimplemented past-check-in list promise: provide the canonical review
  history in this initiative rather than continuing to describe an absent route.

Stage 1 establishes the new visual reference in `design/mockup/`. Mark or separate
superseded screens and update the spec's reference links at cutover so future
work is not directed back to the old dashboard. Committed prototypes use only
synthetic data.

Existing source, privacy, sleep-day, strength evidence and Garmin write constraints
remain in force unless a separately reviewed change explicitly replaces them.

## 15. Usability evaluation: journeys, not screenshot approval

### Common acceptance criteria

These are proposed release targets, not results already measured.

- **First value:** after usable data is available, a new person can explain what
  Today is saying within ten seconds, without setup or a tour. Authentication
  and Garmin network delay are measured separately, not hidden in that claim.
- **Immediate purpose:** at ordinary text size on a 390 × 844 phone, the leading
  meaning and action are visible without scrolling. At 360 px and large text,
  reading order remains clear; no clipping or forced small type.
- **Directness:** last night's sleep is one action from Today; an existing
  recorded workout is one action from its preview; named core destinations are
  no more than two actions from any root via Explore. A metric's evidence is
  one further action from its meaningful summary.
- **Planning:** adding a simple local intention requires one short sheet and
  no goal setup. Rest today is reachable and confirmable in at most two actions.
  Persistent restrictions require more detail, not repeated daily configuration.
- **Discovery:** a person can find Sleep, Strength, workouts, Health/vitals,
  trends, adaptations, Plan and Settings/export without being told which menu
  to use. Search is optional. Failures change the navigation; they are not solved
  with a tutorial. Check the inactive Adapt today control as well as its active state.
- **Restraint:** the no-plan/non-training profile sees no race, strength setup
  or target-deficit cards unless relevant or deliberately opened.
- **Predictability:** the same task is findable on the next visit, with learning
  disabled, after a different activity and when data is missing.
- **Comprehension:** people can distinguish recorded, planned, self-reported,
  unavailable and Garmin-estimated information without a verbal explanation.
- **Control:** dismissing advice, resting and pausing produce no repeated prompt
  on resume, sync or a notification, including while offline.
- **Depth:** common tasks succeed without charts; curious users can reach the
  full charts and comparisons without losing their place.
- **Pressure:** nobody needs to protect a streak, repay an exercise deficit,
  earn rest or claim a reward by reopening the app.

An action means a deliberate tap/click, not each character typed. Count actual
paths, wrong turns and backtracking, not just the shortest theoretical route.

### Journey catalogue

Each row specifies a starting state and the task to attempt. Test the default
route first, with no moderator hints. These are also acceptance-case identifiers
for prototype reviews, UI checks and core-rule tests.

| ID | Starting state and task | Expected path/result; failure to watch for |
| --- | --- | --- |
| U01 | New user, data synced, no goals: "What matters now?" | Today gives a useful explanation; no setup wall or default target verdict |
| U02 | Initial sync partial: "Can I use this yet?" | Available data is useful; missing/old data labelled, no wait for full history |
| U03 | Sleep-only user: "How was last night?" | Today → Sleep in one action; no training or event setup |
| U04 | Sleep-only user: "What about tonight?" | Tonight's window/need and source are easy to find; detailed stages not in the way |
| U05 | Casual mixed activity, no commitments: "Did my workout show up?" | Today preview opens the right activity; neutral recognition without invented goal credit |
| U06 | Returning user after weeks away | Useful current context; no missed-day backlog, broken streak or onboarding replay |
| U07 | Unsupported readiness on this watch | Other value remains; capability-aware explanation, no fictitious effort verdict |
| U08 | Stale watch upload, network available | Understand freshness and open Connect guidance; API fetch time not substituted |
| U09 | Offline with cached data | Browse details and change local intentions/rest; no blocking network spinner |
| U10 | Morning before last night syncs | Neutral waiting state, no old-night effort presented as today's; dated activities remain usable |
| U11 | High readiness, nothing planned: "Could I train?" | Plan a session without a goal wizard; no required amount of load |
| U12 | High readiness on a rest day | Rest respected; no "wasted potential" or changed plan |
| U13 | Low readiness, hard Garmin session scheduled | Review the conflict; no silent cancellation or automatic substitute |
| U14 | High readiness, no declared availability | Optional planning action; no invented free slot or pressure to act now |
| U15 | Completed workout: "How did that count?" | Direct commitment/recorded benefit explanation; no promised age change |
| U16 | Timer-only strength session | Counts as a session where coverage permits; no sets/records fabricated |
| U17 | Detailed strength session: "Am I progressing?" | Exercise bests and prior sets reachable; training load not used as strength quality |
| U18 | HIIT and cycling, no running | Relevant choices and history; race predictions not the overview's centre |
| U19 | Cardio plus strength plus an event | Separate contributions coexist; no exclusive-goal question or combined score |
| U20 | Named event but no verified Garmin plan | Countdown/local event is honest context; no generated race-readiness verdict |
| U21 | Full/partial/stale Garmin calendar | Coverage and dates visible in detail; no false "no upcoming events" |
| U22 | Local intention and similar Garmin activity | Recorded work counts without manual logging; optional one-tap user-confirmed link, no auto-match or missed-workout verdict |
| U23 | Two recorded activities and a late upload | Correct recorded dates and bounded previews; no repeated or historic celebration flood |
| U24 | Weekly target completed early | Completion remains satisfying; no automatic higher target or more-is-better prompt |
| U25 | Incomplete activity-list coverage | Progress awaiting evidence, not zero or behind; known workouts still visible |
| U26 | Current-week target lowered or paused | Reached-adjusted/partial-pause states clear, no proration; previous evidence preserved, edits do not award completion or erase later progress |
| U27 | Targets change across weeks | Review uses each week's version; pre-setup weeks not assigned achievements |
| U28 | User wants one day of rest | Find Adapt today, activate, leave and resume offline; same choice persists |
| U29 | Illness pause plus high readiness | No exercise encouragement; resume requires person, not score or date |
| U30 | Injury excludes a sport and movements | Activity exclusion enforced; movement restriction visible with review-needed state, no claimed screening or safe alternative |
| U31 | Strength routine with unknown movements | "Review needed", not "compatible"; unknown does not bypass restrictions |
| U32 | Conflicting Garmin session during restriction | Conflict shown without changing Garmin; stable access to original item |
| U33 | User-supplied rehabilitation checklist | Fast optional completion, self-reported label; no fabricated Garmin activity/physiological credit |
| U34 | Busy week or travel without illness | Adjust commitments without medical questions, guilt or catch-up demands |
| U35 | Evening after an inactive day | Sleep preparation leads; no late exercise deficit or countdown |
| U36 | Evening workout explicitly planned | Intention remains accessible alongside sleep context; no silent deletion or nag |
| U37 | After bedtime / after midnight | Calm wording and correct night; no reset to morning training advice |
| U38 | Shift work or uncertain sleep timing | Credible fallback and easy fixed presentation; no assumed standard bedtime |
| U39 | Daylight-saving change or travel | Existing phase/timezone protections survive; reminder and night identities correct |
| U40 | Several short recorded nights | Clear period and recorded need; no invented debt amount or repayment demand |
| U41 | Existing asleep-on-time target, with some missing saved plans | Migration explained once, old results retained, timing observed neutrally; no fabricated adherence or automatic wind-down habit |
| U42 | Self-reported wind-down, poor sleep outcome | Preparation and sleep result remain separate; no punishment for outcome |
| U43 | Recorded nap on the same day | Find nap in Sleep and quiet Today event; overnight totals unchanged |
| U44 | Nap crosses midnight or lacks an endpoint | Dated endpoints/missing labels correct; duration not used to invent a clock |
| U45 | Nap empty, unavailable or stale | States distinct in detail; no overview card demanding the person nap |
| U46 | Garmin updates tonight's need after activity/nap | Current guidance updates; saved adherence plan is not rewritten |
| U47 | Fitness Age exists but no declared ambition | Available in Progress/Health; not compulsory or portrayed as whole-person grade |
| U48 | Fitness Age flat or sparse despite regular activity | Recognise participation separately; no invented trend or causal credit |
| U49 | User asks about acute load | Understand purpose and limits in one explanation; no daily budget to fill |
| U50 | User wants Health/vitals, a rare metric, gear or a comparison | Stable Explore path and canonical detail, labelled source/units; no need to train the interface |
| U51 | Past-day chart with no detailed historic record | Honest gap/limit, not a current-day substitution or fictitious fetch promise |
| U52 | Journal off: "Where are my lifestyle logs?" | Find explicit consent path; no prefetch, snippets or unwanted prompt on Today |
| U53 | Journal on; open, background, opt out, export | Explicit opening/privacy reset, consent stop and retention/export disclosure all hold |
| U54 | Learning enabled then disabled/reset | Stable core routes and useful baseline; restrictions and consent unaffected |
| U55 | Repeatedly open a metric, then change interests | At most bounded shortcut adaptation; no navigation shuffle or unwanted health advice |
| U56 | Dismiss a suggestion; later sync/resume | Same suggestion stays dismissed; legitimate new exception distinguishable |
| U57 | Notification permission declined | Full app value remains; no repeated permission prompts or lost credit |
| U58 | Notification or old deep link into a detail | Correct destination and Back path; no obsolete section or onboarding loop |
| U59 | Large type, screen reader, reduced motion | Meaning, statuses, labels and actions remain usable; no colour-only progress |
| U60 | Phone, unfolded screen and desktop | Same reading hierarchy; no masonry rearrangement of the leading message |
| U61 | Long history, many activities, cold start | Local overview stays responsive; detail/trend work is lazy and does not block it |
| U62 | Wearable not worn for a day | Gaps, not failure; achievements remain and no wear streak is lost |
| U63 | Backfill/source correction invalidates a claimed best | Correct evidence once, neutrally; no duplicate award or knowingly false record |
| U64 | Find Settings/export, then sign out after local plans/restrictions exist | Discover controls without hints; new records covered by disclosure/export/erase, credentials excluded |

### How to evaluate

1. Build a linked prototype of whole journeys, not isolated attractive screens.
   Include no-plan, sleep-only, mixed training, event, adaptation and sparse-data
   profiles. Exercise a complete week with an interruption and an imperfect night.
2. Run moderated task walkthroughs with at least three people unfamiliar with
   the design, including a simple-needs user and a detail-oriented user. Ask
   them to explain what they think the screen means before supplying any hints.
   This is a usability gate, not a statistically representative health study.
3. Record time to meaning/action, first chosen destination, wrong turns, extra
   configuration, unwanted content, source confusion and perceived pressure.
   Keep observations local and consented; do not add analytics to production.
4. A critical task requiring coaching, a misunderstood restriction, unsafe
   implication, hidden core destination or confusing planned/recorded state is
   a blocker. Fix the design and rerun affected journeys; do not average those
   failures away with successful easy tasks.
5. Repeat on the implemented app with browser checks and an Android device.
   A maintainer walkthrough is useful but not a substitute for unfamiliar-user
   discovery. If participants or device evidence are unavailable, mark that gate
   unverified rather than declaring UX validated.

If participants, a device or another gate prerequisite are unavailable, stop at
that gate and ask the maintainer for an explicit decision: proceed with a named,
limited-evidence prototype/release decision, or pause the initiative and release
the exclusivity hold on other work. Record the missing evidence, affected
journeys, chosen navigation and accepted risk. Do not choose a navigation by
assumption, waive the gate silently or freeze the backlog indefinitely. An
exception does not turn unverified usability into a passed result.

### Technical and visual checks

- Turn scenario IDs into deterministic core/UI cases where applicable. Cover
  timing transitions, data gaps, explicit choices, restriction precedence,
  versioned expectations, corrected evidence and notification suppression.
- Run the project's Rust, clippy, Svelte/TypeScript, unit, UI and build checks.
  Regenerate view types through the existing workflow, never by hand.
- Test cold/warm navigation, long history, slow sync, no connectivity, app resume,
  Android Back, source-date changes and accessibility on real layouts.
- Capture honest viewports at 360/390 px plus unfolded/desktop sizes, and large
  text/reduced motion. Screenshots support, not replace, task walkthroughs.
- Use a consented private copy of real data under `.data/` for realism; do not
  commit personal readings, screenshots, account IDs or health restrictions.
  Committed fixtures and screenshots use independently synthetic values.
- Compare prototype and implementation against the same content budgets. New
  feature cards must displace lower-priority overview content or live in detail;
  they may not simply extend the page.
- Measure first usable render and detail response on the test device. Preserve
  immediate cached rendering and the existing local trend performance; record
  actual timings and regressions rather than claiming speed from screenshots.

## 16. Sequential delivery and completion

### Stage 1 — Prove the experience before restructuring the app

Produce linked phone prototypes and a preservation map for all shipped features.
Run the simple-to-deep and interrupted-week journeys, including naps and journal
privacy. Validate the navigation hypothesis and content budgets. Record the
chosen structure and unresolved usability failures in this document. Do not
start several competing screen implementations.

### Stage 2 — Establish the shell and shared context

Implement stable destinations, canonical detail routes, Back/deep-link behaviour
and the bounded overview structure. Preserve current evidence and privacy flows.
Introduce only the local state needed for subsequent stages with explicit schema
migration, export and erase semantics. Keep old data/targets and links usable.

### Stage 3 — Make Today and adaptations coherent

Implement leading-message eligibility, explicit rest/restrictions/dismissals,
recorded accomplishments and sleep-phase behaviour together. Extend the same
rules to existing reminders. No high-readiness training prompt ships before
restriction, freshness and rest suppression work.

### Stage 4 — Connect Plan and Progress

Reuse weekly evidence/targets; add lightweight local intentions, coexisting
ambitions, honest revisions, rehabilitation self-report and dated reviews.
Introduce the smallest milestone set with corrections/deduplication. Keep
unverified Garmin programme semantics and numeric sleep debt out of the claims.
Usage-based shortcut adaptation is optional and only earns inclusion by passing
the stable-baseline comparison; it is not required for the product to be useful.

### Stage 5 — Finish the journeys, not just the screens

Run the full applicable catalogue, unfamiliar-user discovery, real-data private
review, device checks and existing regressions. Fix hierarchy and wording as
well as functional failures. Update `spec.md` to the shipped behaviour and keep
verified Garmin facts and genuine follow-ups in TODO. No parallel feature lane
resumes simply because one screen looks finished.

### Definition of done

- A person with no goals gets immediate value without configuration.
- A person with several ambitions is not forced to choose one and can see
  distinct contributions without an invented composite score.
- Rest, illness, restrictions and rehabilitation alter expectations honestly
  everywhere, including offline and notification paths.
- Today is timely and bounded; Plan and Progress have distinct jobs; full data
  remains reliably discoverable without a tutorial or dashboard editor.
- Sleep and naps support wellbeing without debt pressure or new sleep scoring.
- All shipped capabilities have tested destinations; sensitive journal and export
  semantics survive the migration.
- User-serving progress is satisfying without engagement mechanics or streak loss.
- Critical journey and source-truth tests pass. Usability/device gates are either
  verified or explicitly held for a decision, never silently waived.
- The spec describes what shipped, the TODO item records the shipping commits,
  and remaining endpoint/physical-device limitations are documented without
  claiming them fixed by the redesign.

## Garmin references

These explain metric boundaries, not new endpoint verification:

- [Garmin: exercise load and EPOC](https://www.garmin.com/en-US/blog/fitness/what-is-epoc-defining-excess-post-exercise-oxygen-consumption/)
- [Garmin: factors used by Sleep Coach](https://www.garmin.com/en-US/blog/fitness/five-factors-garmin-sleep-coach-uses-to-find-your-sleep-needs/)

Implementation-specific verified payload facts remain in `TODO.md` and source
tests. Public explanations do not establish that a particular account supplies
a field or that the app has already implemented it.
