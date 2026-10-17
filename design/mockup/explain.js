/* "Why" sheets and the score reconciliation on Today (TODO item 6). Runs after index.html's inline
   script and plan-strength.js; reuses their helpers (C, ico, arc, today, training). All data is synthetic. */

Object.assign(ico, {
  split2: svgI('<path d="M4 12h6l4-5h6M14 17h6M10 12l4 5"/>'),
  info: svgI('<circle cx="12" cy="12" r="8.5"/><path d="M12 11v5M12 8h.01"/>'),
  down2: svgI('<path d="M7 10l5 5 5-5"/>'),
});

const tone = t => t === "good" ? C.good : t === "fair" ? C.fair : t === "poor" ? C.poor : C.ink2;
const whyButton = label => `<button class="why-btn">${ico.info}${label}</button>`;

// One input behind a score: Garmin's name, the value it used, Garmin's rating, and what the input is.
const input = ([name, value, rating, t, detail]) => `<div class="why-row">
  <div class="why-top"><b>${name}</b>${value ? `<span class="n">${value}</span>` : ""}<span class="pill" style="color:${tone(t)}">${rating ?? "Not rated"}</span></div>
  <p>${detail}</p></div>`;

function whySheet(under, title, feedback, summary, rows, source) {
  return `${under}<div class="scrim"></div><div class="sheet why-sheet"><span class="grab"></span>
    <div class="sheet-h"><h2>${title}</h2><button class="ibtn" aria-label="Close">${ico.close}</button></div>
    ${feedback ? `<div class="why-feedback">Garmin · ${feedback}</div>` : ""}
    <p class="why-summary">${summary}</p>
    <div class="why-list">${rows.map(input).join("")}</div>
    <p class="why-source">${source}</p></div>`;
}

function whyReadiness() {
  return whySheet(today(), "Why readiness is High", "Well recovered",
    "Recovery time and sleep are helping readiness; acute load is holding it back a little.",
    [["Acute load", "612", "Moderate", "fair", "Your last 7 days of training load against what you're used to."],
      ["Stress history", null, "Good", "good", "Your stress over the last three days."],
      ["HRV", "55 ms", "Good", "good", "Your 7-day HRV average against your baseline."],
      ["Sleep history", null, "Good", "good", "Your sleep over the last three nights against your need."],
      ["Sleep", "84", "Good", "good", "Last night's sleep score."],
      ["Recovery time", "18 h", "Very good", "good", "Hours until Garmin expects you to be recovered from recent workouts."]],
    "From Garmin's readiness factors, worst first. Raring adds no score.");
}

function whyTraining() {
  return whySheet(training(), "Why Productive", null,
    "Your load is moving your fitness forward. Acute load is in your optimal range; load focus is short on anaerobic.",
    [["Acute load", "612 · ratio 1.1", "Optimal", "good", "Your last 7 days of load against your 4-week load."],
      ["Load focus", null, "Anaerobic shortage", "fair", "How the last 4 weeks split across low aerobic, high aerobic and anaerobic."],
      ["HRV status", "55 ms", "Balanced", "good", "Your 7-day HRV average against your baseline."],
      ["VO₂ max", "52", "+1 in 30 days", "neutral", "Garmin's fitness estimate; its trend separates productive from maintaining."]],
    "From Garmin's training status inputs. Raring adds no score.");
}

function whyBattery() {
  const under = `<div class="glow" style="--glow:rgba(255,181,71,.10)"></div>${statusBar()}<div class="scroll">
    ${back("today", "Today")}<header class="top"><div><div class="eyebrow">Garmin metric</div><h1>Body Battery</h1></div></header>
    <section class="detail-dial"><div class="dial">${arc({size: 210, value: 46, color: C.fair, stroke: 15})}<div class="dial-c"><b class="n" style="font-size:60px">46</b><span>Latest synced</span></div></div></section>
    <div style="text-align:center;margin-top:8px">${whyButton("Why 46?")}</div></div>`;
  return whySheet(under, "Why Body Battery is 46", "Stressful and inactive so far",
    "Charged 34 overnight to 71 at wake. Since then, two workouts and a stressful day.",
    [["Overnight", "+34", "71 at wake", "good", "Charge while you slept, 22:40–06:30."],
      ["Stress", "38", "Stressful", "fair", "Stress drains Body Battery while you're awake and still."],
      ["Morning Run", "−9", "Improving tempo", "neutral", "06:12 · 52 min."],
      ["Strength", "−4", "Maintaining", "neutral", "17:12 · 48 min."],
      ["Restful period", "+2", null, "neutral", "13:05 · 20 min."]],
    "From Garmin's Body Battery events and feedback.");
}

// Today with readiness and Body Battery pointing different ways: a one-line reason behind a tap.
function reconcile() {
  const row = `<button class="recon open" aria-expanded="true"><span class="recon-dots"><i style="background:${C.good}"></i><i style="background:${C.poor}"></i></span>Why Body Battery and readiness differ${ico.down2}</button>
    <p class="recon-line">Body Battery charged to 86 overnight, but readiness is held back by recovery time and acute load.</p>`;
  return today()
    .replace(`--glow:rgba(61,220,151,.13)`, `--glow:rgba(255,107,107,.11)`)
    .replace(arc({size: 156, value: 82, color: C.good, stroke: 13}), arc({size: 156, value: 34, color: C.poor, stroke: 13}))
    .replace(`<b class="n">82</b><span>Readiness</span><em style="color:${C.good}">High</em>`, `<b class="n">34</b><span>Readiness</span><em style="color:${C.poor}">Low</em>`)
    .replace(`<h2>Ready to push</h2>`, `<h2>Take it easy</h2>`)
    // Garmin's factors and the call agree with the low score: recovery time and acute load hold it back.
    .replace(`<span>Recovery<b>92</b></span><div class="bar"><i style="width:92%;background:${C.good}">`, `<span>Recovery<b>22</b></span><div class="bar"><i style="width:22%;background:${C.poor}">`)
    .replace(`<span>Acute load<b>58</b></span><div class="bar"><i style="width:58%;background:${C.fair}">`, `<span>Acute load<b>31</b></span><div class="bar"><i style="width:31%;background:${C.poor}">`)
    .replace(`<span class="pill ok">5 of 6 good</span>`, `<span class="pill ok">4 of 6 good</span>`)
    .replace(`<span>Easy</span><span>Steady</span><span class="on">Push</span>`, `<span class="on">Easy</span><span>Steady</span><span>Push</span>`)
    .replace(`<span class="chip">${ico.flame}Room to build</span></div>`, `<span class="chip">${ico.flame}On track</span></div>${row}`);
}

function reconcileClosed() {
  return reconcile().replace(`class="recon open" aria-expanded="true"`, `class="recon" aria-expanded="false"`).replace(/<p class="recon-line">.*?<\/p>/, "");
}

SCREENS.push(
  ["reconcile", "Scores that differ, opened", reconcile, "phone", "Explain the numbers"],
  ["reconcile-closed", "Scores that differ", reconcileClosed, "phone", "Explain the numbers"],
  ["why-readiness", "Why readiness", whyReadiness, "phone", "Explain the numbers"],
  ["why-battery", "Why Body Battery", whyBattery, "phone", "Explain the numbers"],
  ["why-training", "Why training status", whyTraining, "phone", "Explain the numbers"],
  ["why-readiness", "Why readiness", whyReadiness, "desk", "Explain the numbers"],
);
