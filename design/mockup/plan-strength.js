/* Weekly plan, strength, builder, help and widget screens, plus the design-language sheet. Runs after
   index.html's inline script and reuses its helpers (C, ico, svgI, arc, spark, smooth, rng, statusBar,
   tabs, zoneBar). All data is synthetic. */

Object.assign(ico, {
  star: svgI('<path d="M12 3.6l2.5 5.2 5.7.8-4.1 4 1 5.6L12 16.5l-5.1 2.7 1-5.6-4.1-4 5.7-.8z"/>'),
  pencil: svgI('<path d="M4 20h4L19 9l-4-4L4 16z"/><path d="M13.5 6.5l4 4"/>'),
  watch: svgI('<rect x="7" y="6.5" width="10" height="11" rx="3"/><path d="M9.2 6.5l.5-3h4.6l.5 3M9.2 17.5l.5 3h4.6l.5-3"/>'),
  pause: svgI('<path d="M9 6.5v11M15 6.5v11"/>'),
  plus: svgI('<path d="M12 5v14M5 12h14"/>'),
  cal: svgI('<rect x="4" y="5" width="16" height="15" rx="3"/><path d="M4 10h16M9 3v4M15 3v4"/>'),
  search: svgI('<circle cx="11" cy="11" r="6.5"/><path d="M16 16l4 4"/>'),
  grip: svgI('<path d="M9 6h.01M15 6h.01M9 12h.01M15 12h.01M9 18h.01M15 18h.01"/>', 'stroke-width="3.2"'),
  swap: svgI('<path d="M7 4L4 7l3 3M4 7h12M17 20l3-3-3-3M20 17H8"/>'),
  trash: svgI('<path d="M5 7h14M10 7V4.5h4V7M7 7l1 13h8l1-13"/>'),
  split: svgI('<path d="M12 3v7M12 10l-5.5 5.5V21M12 10l5.5 5.5V21"/>'),
  lock: svgI('<rect x="5" y="11" width="14" height="9" rx="2.5"/><path d="M8.5 11V8a3.5 3.5 0 0 1 7 0v3"/>'),
  up: svgI('<path d="M12 19V6M6.5 11.5L12 6l5.5 5.5"/>'),
  down: svgI('<path d="M12 5v13M6.5 12.5L12 18l5.5-5.5"/>'),
  same: svgI('<path d="M5 9.5h14M5 14.5h14"/>'),
  learn: svgI('<circle cx="12" cy="12" r="8.5"/><path d="M12 7.5V12l3 2"/>'),
  chev: svgI('<path d="M9 5l7 7-7 7"/>'),
  alert: svgI('<path d="M12 4.5l8.5 15h-17z"/><path d="M12 10.5v3.5M12 16.8h.01"/>'),
  sync: svgI('<path d="M19.5 12a7.5 7.5 0 0 1-13 5.1M4.5 12a7.5 7.5 0 0 1 13-5.1"/><path d="M17.5 3.5v3.4h-3.4M6.5 20.5v-3.4h3.4"/>'),
  undo: svgI('<path d="M9 7L5 11l4 4"/><path d="M5 11h9a5 5 0 0 1 0 10h-2"/>'),
  help: svgI('<circle cx="12" cy="12" r="8.5"/><path d="M9.6 9.5a2.5 2.5 0 0 1 4.8.9c0 1.7-2.4 2.1-2.4 3.6M12 17h.01"/>'),
  book: svgI('<path d="M5 4.5h9.5a3 3 0 0 1 3 3V20H8a3 3 0 0 1-3-3z"/><path d="M5 17a3 3 0 0 1 3-3h9.5"/>'),
  shield: svgI('<path d="M12 3.5l7 2.8v5.4c0 4.3-3 7.4-7 8.8-4-1.4-7-4.5-7-8.8V6.3z"/>'),
  out: svgI('<path d="M14 5h5v5M19 5l-8 8M17 14v4a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V9a2 2 0 0 1 2-2h4"/>'),
  save: svgI('<path d="M12 4.5V15M7.5 10.5L12 15l4.5-4.5"/><path d="M5 15v3a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2v-3"/>'),
  close: svgI('<path d="M6 6l12 12M18 6L6 18"/>'),
  target: svgI('<circle cx="12" cy="12" r="8.5"/><circle cx="12" cy="12" r="4.5"/><circle cx="12" cy="12" r=".6"/>'),
});

const back = (go, label) => `<button class="back" data-go="${go}">${ico.back.replace("<svg ", '<svg width="20" height="20" ')}${label}</button>`;
const helpBtn = go => `<button class="ibtn tap" data-go="${go}" aria-label="Help">${ico.help}</button>`;
// Glow only behind a status, and only faintly.
// Modal flows and screens with a save bar hide the bottom tabs; the desktop rail stays.
const shell = (body, tab, {glow, bar = "", modal = false} = {}) => `${glow ? `<div class="glow" style="--glow:${glow}"></div>` : ""}${statusBar()}
  <div class="scroll${bar ? " has-bar" : ""}"><div class="page">${body}</div></div>${bar}${bar || modal ? tabs(tab).replace('class="tabs"', 'class="tabs off"') : tabs(tab)}`;
const kg = w => w == null ? "—" : String(w);
const stepper = (v, unit = "", cls = "") => `<div class="stp ${cls}"><button>−</button><b class="n">${v}${unit ? `<small>${unit}</small>` : ""}</b><button>+</button></div>`;
const segc = (opts, on, cls = "") => `<div class="segc ${cls}">${opts.map((o, i) => `<span${i === on ? ' class="on"' : ""}>${o}</span>`).join("")}</div>`;
const sw = on => `<span class="sw${on ? " on" : ""}"></span>`;
const hexMix = (a, b, t) => "#" + [1, 3, 5].map(i => Math.round(parseInt(a.slice(i, i + 2), 16) * (1 - t) + parseInt(b.slice(i, i + 2), 16) * t).toString(16).padStart(2, "0")).join("");

/* ---------- Body map ---------- */
// Raring's own figures: a silhouette with muscle groups drawn as separate shapes, shaded by sets in the
// one amount color. Each figure draws its left half and mirrors it.
const BODY = {female: {sh: 13.4, waist: 8.6, hip: 15.4, neck: 3}, male: {sh: 18, waist: 12.2, hip: 12.8, neck: 4}};
const amountColor = (v, max) => v ? hexMix("#1F3557", C.amount, Math.min(v / max, 1)) : "#353C4C";
function bodyMap(sex, sets, {w = 220, max = 12} = {}) {
  const {sh, waist, hip, neck} = BODY[sex], X = d => (50 - d).toFixed(2), m = g => amountColor(sets[g] || 0, max);
  const el = (cx, cy, rx, ry, g, rot = 0) => `<ellipse cx="${X(50 - cx)}" cy="${cy}" rx="${rx}" ry="${ry}" fill="${m(g)}"${rot ? ` transform="rotate(${rot} ${X(50 - cx)} ${cy})"` : ""}/>`;
  const at = d => 50 - d;
  const skin = `<path d="M50.6,25 L46,25 Q45,28.5 ${X(sh - 3)},29.5 Q ${X(sh + 1)},30.5 ${X(sh + 1.6)},36 L ${X(sh - 1.4)},46 Q ${X(waist + 1)},56 ${X(waist)},66 Q ${X(hip + .6)},76 ${X(hip)},86 L ${X(hip - 1)},97 L 50.6,99 Z"/>
    <path d="M ${X(sh - 1)},30.5 Q ${X(sh + 4)},30.5 ${X(sh + 5.2)},40 L ${X(sh + 6.4)},60 L ${X(sh + 9)},86 L ${X(sh + 4.2)},87.5 L ${X(sh + 1.4)},62 L ${X(sh - 1.4)},46 Z"/>
    <ellipse cx="${X(sh + 6.8)}" cy="92.5" rx="3.3" ry="5.2"/>
    <path d="M ${X(hip)},86 Q ${X(hip + 1)},112 ${X(11.4)},138 Q ${X(12.4)},152 ${X(10)},170 L ${X(9)},194 L ${X(5)},196 L ${X(4.6)},170 Q ${X(3)},152 ${X(3.2)},140 Q ${X(1)},120 ${X(.4)},99 Z"/>
    <ellipse cx="${X(7.4)}" cy="198.5" rx="4.2" ry="3"/>`;
  const front = `<path d="M49.4,31 L ${X(sh - 3)},31 Q ${X(sh - 1)},37 ${X(sh - 2)},43.5 Q ${X(9)},48.5 49.4,45.5 Z" fill="${m("Chest")}"/>
    ${el(at(sh + 1.6), 37, 4.9, 6.8, "Shoulders", 18)}
    ${el(at(sh + 3.6), 51, 3.4, 8.6, "Arms", 9)}${el(at(sh + 6.3), 73.5, 3.2, 10.5, "Arms", 9)}
    ${[48, 55.5, 63].map(y => `<rect x="${X(5.4)}" y="${y}" width="4.6" height="6.4" rx="1.8" fill="${m("Core")}"/>`).join("")}<rect x="${X(4.8)}" y="70.6" width="4" height="9.5" rx="1.8" fill="${m("Core")}"/>
    <path d="M ${X(6.6)},49 Q ${X(waist - .6)},55 ${X(waist - .7)},66 Q ${X(waist - .2)},76 ${X(6.8)},80.5 Z" fill="${m("Core")}"/>
    ${el(at(hip - 6.3), 114, 6.1, 20, "Quads", -4)}${el(at(2.3), 108, 2, 9.5, "Quads")}
    ${el(at(8.2), 160, 3.1, 14, "Calves", 3)}`;
  const rear = `<path d="M49.4,23 L46.5,24 L ${X(sh - 2.5)},31 L 49.4,46 Z" fill="${m("Back")}"/>
    ${el(at(sh + 1.6), 37, 4.9, 6.8, "Shoulders", 18)}
    ${el(at(sh + 3.6), 51, 3.6, 9, "Arms", 9)}${el(at(sh + 6.3), 73.5, 3.2, 10.5, "Arms", 9)}
    <path d="M ${X(sh - 2.6)},37 Q ${X(sh - 2)},48 ${X(waist)},60 Q ${X(6)},68 ${X(2.2)},64 L ${X(2.6)},50 Z" fill="${m("Back")}"/>
    <rect x="${X(4.8)}" y="62" width="3.4" height="17" rx="1.7" fill="${m("Back")}"/>
    ${el(at(hip - 6.3), 91.5, 6.8, 7.8, "Glutes")}
    ${el(at(hip - 6.2), 118, 5.9, 16, "Hamstrings", -3)}
    ${el(at(7.9), 156, 4.3, 12.5, "Calves", 3)}`;
  // Drawn narrow and widened here, which keeps the shapes simple to tune.
  const fig = (muscles, dx) => `<g transform="translate(${dx + 50} 0) scale(1.42 1) translate(-50 0)"><g fill="#272D3B"><ellipse cx="50" cy="13.5" rx="5.8" ry="10"/><rect x="${50 - neck}" y="20" width="${neck * 2}" height="8" rx="2"/>${skin}<g transform="matrix(-1 0 0 1 100 0)">${skin}</g></g>
    ${muscles}<g transform="matrix(-1 0 0 1 100 0)">${muscles}</g></g>`;
  return `<svg class="body" width="${w}" height="${(w * 204 / 214).toFixed(0)}" viewBox="0 0 214 204" role="img" aria-label="Muscles worked">${fig(front, 0)}${fig(rear, 114)}</svg>`;
}
// Black or white text, whichever reads better on a fill; one of them always clears 4.5:1.
const lum = h => [1, 3, 5].map(i => parseInt(h.slice(i, i + 2), 16) / 255).map(x => x <= .03928 ? x / 12.92 : ((x + .055) / 1.055) ** 2.4).reduce((a, x, i) => a + x * [.2126, .7152, .0722][i], 0);
const onFill = h => 1.05 / (lum(h) + .05) >= (lum(h) + .05) / .05 ? "#FFFFFF" : "#000000";
const amountKey = (max, label = "sets") => `<div class="akey"><span class="n">0</span><i style="background:linear-gradient(90deg,#353C4C,#1F3557 12%,${C.amount})"></i><span class="n">${max}+ ${label}</span></div>`;

/* ---------- Weekly plan ---------- */
// Week Mon 13 – Sun 19 Oct; today is Thu 16. Rest days Mon and Fri. "Pace" is the share of the week's
// training days (nights, for sleep) that have passed; behind pace turns a goal amber, done turns it green.
const PT = {
  strength: {name: "Strength", unit: "sessions", done: 2, target: 3, pace: .6, icon: ico.lift, note: "Tue, Wed · Upper B on Sat"},
  intensity: {name: "Intensity", unit: "min", done: 124, target: 150, pace: .6, icon: ico.flame, note: "Garmin's weekly goal"},
  z2: {name: "Zone 2", unit: "min", done: 46, target: 90, pace: .6, icon: ico.pulse, note: "44 min to go"},
  sleep: {name: "Asleep on time", unit: "nights", done: 3, target: 5, pace: 4 / 7, icon: ico.moon, note: "Mon, Tue, Thu"},
};
const TG = [PT.strength, PT.intensity, PT.z2, PT.sleep];
const goalState = (t, done = t.done) => done >= t.target ? "met" : done / t.target >= t.pace - .05 ? "on" : "behind";
const stateColor = {met: C.good, on: C.ink, behind: C.fair};
// Each week's bar is the average completion across targets, each capped at 100%, so partial credit shows.
const HIST_W = ["25 Aug", "1 Sep", "8 Sep", "15 Sep", "22 Sep", "29 Sep", "6 Oct", "Now"];
const HIST_V = {strength: [2, 3, 3, null, 2, 3, 3, 2], intensity: [130, 155, 170, null, 120, 150, 162, 124], z2: [60, 75, 90, null, 70, 85, 72, 46], sleep: [3, 4, 5, null, 4, 5, 4, 3]};
const HIST = HIST_W.map((w, i) => [w, HIST_V.strength[i] === null ? null : Math.round(Object.entries(HIST_V).reduce((a, [k, v]) => a + Math.min(v[i] / PT[k].target, 1), 0) / 4 * 100)]);

const goalRow = (t, compact) => {
  const st = goalState(t), pct = Math.min(t.done / t.target, 1) * 100;
  return `<div class="goal ${st}${compact ? " compact" : ""}"><span class="g-i">${t.icon}</span><div class="g-m">
    <div class="g-t"><b>${t.name}</b><span class="n"><b>${t.done}</b>${compact ? "/" : " of "}${t.target}${compact ? "" : ` ${t.unit}`}</span></div>
    <div class="pbar"><i style="width:${pct}%;background:${stateColor[st]}"></i></div>${compact ? "" : `<small>${st === "behind" ? `<i class="dot" style="background:${C.fair}"></i>` : ""}${t.note}</small>`}</div></div>`;
};

function planCard() {
  return `<div class="card tap" data-go="plan"><h3>This week <span class="h-note">3 days left</span></h3>
    <div class="goals">${TG.map(t => goalRow(t, true)).join("")}</div></div>`;
}

function weekBars() {
  return `<div class="wbars">${HIST.map(([w, pct]) => `<div class="wb${pct === null ? " paused" : ""}${w === "Now" ? " now" : ""}">
    <div class="wb-t">${pct === null ? ico.pause : `<i style="height:${pct}%;background:${pct === 100 ? C.good : w === "Now" ? C.ink : "rgba(228,232,240,.32)"}"></i>`}</div>
    <b class="n">${pct === null ? "–" : pct}</b><span>${w.split(" ")[0]}</span></div>`).join("")}</div>
    <div class="lgd"><span>% of targets reached</span><span><i style="background:${C.good}"></i>All reached</span><span>${ico.pause}Paused</span></div>`;
}

function plan() {
  return shell(`${back("today", "Today")}
  <header class="top"><div><div class="eyebrow">Mon 13 – Sun 19 Oct</div><h1>This week</h1></div><div class="hact">${helpBtn("help")}<button class="ibtn tap" data-go="plan-setup" aria-label="Edit targets">${ico.pencil}</button></div></header>
  <div class="cols">
    <div class="col">
      <div class="card"><div class="psum"><div class="n psum-n">3<small>of 4</small></div><div><b>On track</b><span>3 days left · Zone 2 is behind</span></div></div>
        <div class="goals">${TG.map(t => goalRow(t)).join("")}</div></div>
    </div>
    <div class="col">
      <div class="card tap" data-go="builder"><div class="act"><div class="act-ico">${ico.lift}</div>
        <div class="act-main"><b>Upper B</b><span>Next up · Sat 18</span></div><span class="chev">${ico.chev}</span></div></div>
      <div class="card"><h3>Last 8 weeks</h3>${weekBars()}</div>
    </div>
  </div>`, "today");
}

function planSetup() {
  const row = (t, sub, ctl) => `<div class="trow"><span class="g-i">${t.icon}</span><div><b class="t">${t.name === "Asleep on time" ? "Asleep on time" : t.name}</b><span>${sub}</span></div>${ctl}</div>`;
  const targets = `<div class="card tlist">
    ${row(PT.strength, "Sessions a week", stepper(3))}
    ${row(PT.intensity, "Garmin's weekly goal", `<div class="goalv"><b class="n">150</b><span class="pill">${ico.sync}Garmin</span></div>`)}
    ${row(PT.z2, "Minutes in heart-rate zone 2", stepper(90))}
    ${row(PT.sleep, "Nights within 15 min", stepper(5))}
  </div>`;
  return shell(`${back("plan", "This week")}
  <header class="top"><div><div class="eyebrow">Weekly plan</div><h1>Targets</h1></div>${helpBtn("help")}</header>
  <div class="cols wide-l">
    <div class="col">
      <div class="card"><h3>Start from</h3>${segc(["Strength", "Endurance", "Sleep", "Custom"], 0, "plain")}<div class="hintline">Sets all four targets. Change any of them below.</div></div>
      ${targets}
    </div>
    <div class="col">
      <div class="card"><h3>Rest days</h3>
        <div class="wdays">${["M", "T", "W", "T", "F", "S", "S"].map((d, i) => `<span class="${[0, 4].includes(i) ? "rest" : ""}">${d}</span>`).join("")}</div>
        <div class="hintline">Nothing is due on a rest day.</div></div>
      <div class="card"><div class="trow"><span class="g-i">${ico.pause}</span><div><b class="t">Pause this week</b><span>Travel or illness. Left out of history.</span></div>${sw(false)}</div></div>
      <div class="card"><h3>Last week on these targets</h3><div class="goals">${[[PT.strength, 3], [PT.intensity, 162], [PT.z2, 72], [PT.sleep, 4]].map(([t, v]) => goalRow({...t, done: v, pace: 1}, true)).join("")}</div></div>
    </div>
  </div>`, "today");
}

function checkin() {
  const res = [[PT.strength, 3, 3], [PT.intensity, 162, 150], [PT.z2, 72, 85], [PT.sleep, 4, 5]];
  const delta = (a, b) => a === b ? `<span class="dl">same</span>` : `<span class="dl">${a > b ? "▲" : "▼"} ${Math.abs(a - b)}</span>`;
  return shell(`
  <header class="top"><div><div class="eyebrow">Weekly check-in · 6 – 12 Oct</div><h1>Your plan</h1></div><button class="ibtn" aria-label="Close">${ico.close}</button></header>
  <div class="ci-dots">${[0, 1, 2, 3, 4, 5].map(i => `<span class="${i <= 4 ? "on" : ""}"></span>`).join("")}</div>
  <div class="card ci-card">
    <div class="psum"><div class="n psum-n">3<small>of 4</small></div><div><b>Targets met</b><span>Zone 2 came up short</span></div></div>
    <div class="ci-list">${res.map(([t, v, b]) => { const met = v >= t.target; return `<div class="ci-r"><span class="g-i">${t.icon}</span><b>${t.name}</b><span class="n ci-v">${v}<small> of ${t.target}</small></span><span class="ci-s" style="color:${met ? C.good : C.fair}">${met ? ico.check : `<i class="dot" style="background:${C.fair}"></i>`}</span>${delta(v, b)}</div>`; }).join("")}</div>
    <div class="ci-next"><h3>This week</h3><p class="dim">Same targets · rest Mon and Fri</p>
      <div class="ci-btns"><button class="btn primary">Keep</button><button class="btn tap" data-go="plan-setup">Change</button></div></div>
  </div>
  <div class="ci-ctl"><span>‹ Health</span><span>Win and focus ›</span></div>`, "today", {modal: true});
}

/* ---------- Strength workout ---------- */
const LOWER_A = [
  {name: "Barbell Back Squat", mus: "Quads · Glutes", last: "Wed 8 Oct", lastSets: [[97.5, 5], [97.5, 5], [97.5, 5]], pr: "100 × 5",
    sets: [{w: 60, r: 8, warm: true}, {w: 100, r: 5, pr: true}, {w: 100, r: 5}, {w: 100, r: 5}]},
  {name: "Romanian Deadlift", mus: "Hamstrings · Glutes", last: "Wed 8 Oct", lastSets: [[80, 8], [80, 8], [80, 7]],
    sets: [{w: 80, r: 8}, {w: 80, r: 8}, {w: 80, r: 0, flag: true}, {w: 80, r: 8}]},
  {name: "Dumbbell Walking Lunge", mus: "Quads · Glutes", last: "Wed 8 Oct", lastSets: [[16, 12], [16, 12], [16, 12]],
    sets: [{w: 16, r: 12}, {w: 16, r: 24, flag: true}]},
  {name: "Not recognized", mus: "Garmin couldn't name it", unknown: true,
    sets: [{w: 40, r: 12, flag: true}, {w: 40, r: 12, flag: true}, {w: 40, r: 10, flag: true}]},
  {name: "Standing Calf Raise", mus: "Calves", last: "Wed 8 Oct", lastSets: [[40, 15], [40, 15], [40, 15]],
    sets: [{w: 40, r: 15}, {w: 40, r: 15}, {w: 40, r: 14}]},
];
const SESSION = {Quads: 6, Glutes: 9, Hamstrings: 3, Calves: 3};

function setBars(ex) {
  const maxR = Math.max(...ex.sets.map(s => s.r), ...(ex.lastSets || []).map(s => s[1]), 1);
  let j = 0;
  const bars = ex.sets.map(s => {
    const ghost = !s.warm && ex.lastSets ? ex.lastSets[j++] : null, h = r => Math.max(r / maxR * 56, 3);
    return `<div class="sbar${s.flag ? " flag" : ""}${s.warm ? " warm" : ""}"><b class="n">${s.r}</b>
      <div class="sbar-t">${ghost ? `<u style="height:${h(ghost[1])}px"></u>` : ""}<i style="height:${h(s.r)}px">${s.pr ? ico.star : ""}</i></div>
      <span class="n">${s.warm ? "W " : ""}${kg(s.w)}</span></div>`;
  }).join("");
  return `<div class="setbars">${bars}<div class="sbar key"><b>reps</b><div class="sbar-t"></div><span>kg</span></div></div>`;
}

function exCard(ex) {
  const flags = ex.sets.filter(s => s.flag).length;
  const tag = ex.unknown ? `<span class="pill warn">Name it</span>` : flags ? `<span class="pill warn">${flags} to check</span>` : ex.pr ? `<span class="pill ok">${ico.star}${ex.pr}</span>` : "";
  const last = ex.lastSets ? `<div class="lastl">Last · ${ex.last} · <b class="n">${ex.lastSets.map(s => `${kg(s[0])}×${s[1]}`).join("  ")}</b></div>` : "";
  return `<div class="card ex"><div class="ex-h"><div><b>${ex.name}</b><span>${ex.mus}</span></div>${tag}</div>${setBars(ex)}${last}</div>`;
}

function strengthWorkout() {
  const sets = LOWER_A.reduce((a, e) => a + e.sets.length, 0), reps = LOWER_A.reduce((a, e) => a + e.sets.reduce((b, s) => b + s.r, 0), 0);
  return shell(`${back("training", "Training")}
  <header class="top" style="margin-top:2px"><div><div class="eyebrow">Strength · Wed 15 Oct · 18:05</div><h1>Lower A</h1></div>${helpBtn("help-sheet")}</header>
  <div class="cols">
    <div class="col">
      <div class="card"><div class="sw-hero">
        <div><div class="eyebrow">Sets</div><div class="n hero-n">${sets}</div><div class="tags"><span class="pill ok">${ico.star}1 record</span></div></div>
        <div class="grid2"><div><b class="n">48:02</b><span>Time</span></div><div><b class="n">${reps}</b><span>Reps</span></div><div><b class="n">74</b><span>Load</span></div><div><b class="n">118</b><span>Avg HR</span></div></div>
      </div></div>
      <div class="card alertc tap" data-go="fix"><div class="al"><span class="al-i">${ico.alert}</span><div><b>3 things look off</b><span>A 0-rep set, a doubled set, an unnamed exercise</span></div><button class="btn sm">Fix</button></div></div>
      <div class="card"><h3>Worked</h3><div class="bodywrap">${bodyMap("female", SESSION, {w: 240, max: 9})}</div>${amountKey(9)}</div>
    </div>
    <div class="col">${LOWER_A.map(exCard).join("")}</div>
  </div>`, "training");
}

/* ---------- Fix sets ---------- */
function fixSets() {
  const row = (n, w, r, st = "") => `<div class="erow ${st}"><i class="n">${n}</i><span class="ev n">${kg(w)}<small>kg</small></span><span class="ex-x">×</span><span class="ev n">${r}<small>reps</small></span><span class="es">${st.includes("done") ? ico.pencil : ""}</span></div>`;
  const editor = `<div class="editor">
    <div class="ed"><span class="ed-l">Weight <small>2.5 kg steps</small></span>${stepper(40, "kg", "lg")}</div>
    <div class="ed"><span class="ed-l">Reps</span>${stepper(15, "", "lg")}<div class="quick">${[10, 12, 14, 15].map(v => `<span class="${v === 15 ? "on" : ""}">${v}</span>`).join("")}</div></div>
    <div class="ed-a"><button class="btn sm">${ico.trash}Delete</button><button class="btn sm">Copy to next</button><button class="btn sm primary">Done</button></div></div>`;
  return shell(`${back("strength-workout", "Lower A")}
  <header class="top"><div><div class="eyebrow">Fix sets · Wed 15 Oct</div><h1>3 to check</h1></div>${helpBtn("help-sheet")}</header>
  <div class="cols">
    <div class="col">
      <div class="card"><h3>To check</h3>
        <div class="fx done"><span class="fx-i ok">${ico.check}</span><div class="fx-m"><b>Romanian Deadlift · set 3</b><span>0 reps · deleted</span></div><button class="btn sm ghost">${ico.undo}Undo</button></div>
        <div class="fx"><span class="fx-i">${ico.alert}</span><div class="fx-m"><b>Walking Lunge · set 2</b><span>24 reps · two sets joined?</span></div></div>
        <div class="fx-a"><button class="btn sm primary">${ico.split}Split 12 + 12</button><button class="btn sm">Keep 24</button></div>
        <div class="fx"><span class="fx-i">${ico.alert}</span><div class="fx-m"><b>Not recognized · 3 sets</b><span>40 kg · 12, 12, 10 reps</span></div></div>
        <div class="fx-a chips-l"><span class="chipb on">${ico.check}Seated Leg Curl</span><span class="chipb">Lying Leg Curl</span><span class="chipb">Leg Extension</span><span class="chipb">${ico.search}Search</span></div>
      </div>
    </div>
    <div class="col">
      <div class="card"><h3>All sets</h3>
        <div class="eg"><div class="eg-h"><b>Barbell Back Squat</b>${ico.swap}</div>${row("W", 60, 8)}${row(1, 100, 5)}${row(2, 100, 5)}${row(3, 100, 5)}</div>
        <div class="eg"><div class="eg-h"><b>Romanian Deadlift</b>${ico.swap}</div>${row(1, 80, 8)}${row(2, 80, 8)}<div class="erow gone"><i class="n">–</i><span class="ev n">80<small>kg</small></span><span class="ex-x">×</span><span class="ev n">0<small>reps</small></span><span class="es">${ico.trash}</span></div>${row(3, 80, 8)}</div>
        <div class="eg"><div class="eg-h"><b>Seated Leg Curl</b><span class="h-note">was unnamed</span></div>${row(1, 40, 12, "done")}${row(2, 40, 12, "done")}${row(3, 40, 10, "done")}</div>
        <div class="eg"><div class="eg-h"><b>Standing Calf Raise</b>${ico.swap}</div>${row(1, 40, 15)}${row(2, 40, 15)}${row(3, 40, 15, "done open")}${editor}</div>
      </div>
    </div>
  </div>`, "training", {bar: `<div class="savebar"><div class="sb-in"><div><b class="n">5 changes</b></div><button class="btn primary">Save</button></div></div>`});
}

/* ---------- Builder ---------- */
const UPPER_B = [
  {name: "Barbell Bench Press", eq: "Barbell · 2.5 kg steps", sets: 3, reps: "8–12", w: 72.5, rest: "2:00", prog: "auto", s: "up", sug: "72.5 kg × 8", why: "All sets hit 12 last time · +2.5 kg", warm: [[40, 10], [60, 5]], open: true},
  {name: "One-Arm Dumbbell Row", eq: "Dumbbells · 2 kg steps", sets: 3, reps: "10–12", w: 30, rest: "1:30", prog: "auto", s: "same", sug: "30 kg × 12", why: "2 of 3 sets hit 12 · same weight"},
  {name: "Overhead Barbell Press", eq: "Barbell · 2.5 kg steps", sets: 3, reps: "6–8", w: 42.5, rest: "2:00", prog: "auto", s: "down", sug: "42.5 kg × 6", why: "Under 6 reps twice · −2.5 kg"},
  {name: "Lat Pulldown", eq: "Cable stack · 5 kg steps", sets: 3, reps: "10", w: 55, rest: "1:30", prog: "off"},
  {name: "Dumbbell Lateral Raise", eq: "Dumbbells · 2 kg steps", sets: 3, reps: "12–15", w: 10, rest: "1:00", prog: "auto", s: "up", sug: "10 kg × 14", why: "Next dumbbell is +20% · add reps first"},
  {name: "Cable Face Pull", eq: "Cable stack · 5 kg steps", sets: 3, reps: "12–15", w: 20, rest: "1:00", prog: "auto", s: "learn", sug: "20 kg × 12", why: "New · suggests after 2 sessions"},
  {name: "Plank", eq: "Bodyweight · timed", sets: 3, reps: "0:45", w: null, rest: "0:45", prog: "off"},
];
const STEPS = UPPER_B.reduce((a, e) => a + 2 * (e.sets + (e.warm ? e.warm.length : 0)), 0) - 1;
const DAYS = [{d: "M", n: 13, rest: true}, {d: "T", n: 14, done: true}, {d: "W", n: 15, done: true}, {d: "T", n: 16, today: true}, {d: "F", n: 17, rest: true}, {d: "S", n: 18, sel: true}, {d: "S", n: 19}];

function bxCard(e) {
  const sIco = {up: ico.up, same: ico.same, down: ico.down, learn: ico.learn}[e.s];
  const prog = e.prog === "auto"
    ? `<div class="sug ${e.s}"><span class="sg-i">${sIco}</span><div><b class="n">${e.sug}</b><span>${e.why}</span></div></div>`
    : `<div class="sug off"><span class="sg-i">${ico.pencil}</span><div><b class="n">${e.sets} × ${e.reps}${e.w ? ` · ${e.w} kg` : ""}</b><span>Your targets</span></div></div>`;
  const table = e.open ? `<div class="stab"><div class="st-h"><span>Set</span><span>kg</span><span>Reps</span><span>Rest</span></div>
    ${e.warm.map(([w, r]) => `<div class="st-r warm"><span>W</span><b class="n">${w}</b><b class="n">${r}</b><b class="n">1:00</b></div>`).join("")}
    ${[1, 2, 3].map(n => `<div class="st-r"><span class="n">${n}</span><b class="n">${e.w}</b><b class="n">8–12</b><b class="n">${n < 3 ? e.rest : "—"}</b></div>`).join("")}
    <div class="st-add">${ico.plus}Add set</div></div>`
    : `<div class="bsum"><span><b class="n">${e.sets}</b> sets</span><span><b class="n">${e.reps}</b>${e.reps.includes(":") ? "" : " reps"}</span><span><b class="n">${kg(e.w)}</b>${e.w ? " kg" : ""}</span><span>${ico.clock}<b class="n">${e.rest}</b></span></div>`;
  return `<div class="card bx"><div class="bx-h"><span class="grip">${ico.grip}</span><div class="bx-n"><b>${e.name}</b><span>${e.eq}</span></div>${segc(["Off", "Auto"], e.prog === "auto" ? 1 : 0, "mini")}</div>${table}${prog}</div>`;
}

const weekPick = () => `<div class="wpick">${DAYS.map(d => `<span class="${d.rest ? "rest" : ""}${d.sel ? " sel" : ""}${d.today ? " today" : ""}"><small>${d.d}</small><b class="n">${d.n}</b><i>${d.done ? ico.check : d.rest ? ico.pause : ""}</i></span>`).join("")}</div>`;
const brakeCard = () => `<div class="card"><h3>Readiness brake ${sw(true)}</h3>
  <div class="brake"><span class="hold">Rest</span><span class="hold">Easy</span><span>Steady</span><span>Push</span></div>
  <div class="brk-l"><span class="hold">${ico.lock}Hold weights</span><span>${ico.up}Use suggestions</span></div>
  <div class="hintline">Decided by Saturday morning's readiness.</div></div>`;
const stepMeter = (size = 66) => `<div class="dial stepm" style="--s:${size}px">${arc({size, value: STEPS, max: 100, color: STEPS > 100 ? C.poor : STEPS >= 90 ? C.fair : C.ink, stroke: size > 80 ? 10 : 6})}<div class="dial-c"><b class="n">${STEPS}</b><span>of 100</span></div></div>`;

const CATS = [["Bench Press", 39], ["Row", 41], ["Shoulder Press", 32], ["Pull Up", 37], ["Lateral Raise", 33], ["Curl", 43], ["Squat", 98], ["Deadlift", 21], ["Lunge", 82], ["Plank", 70], ["Core", 52], ["Calf Raise", 20]];
const ROWS = [["Barbell Row", "Barbell · Back"], ["One-Arm Dumbbell Row", "Dumbbell · Back", true], ["Seated Cable Row", "Cable · Back"], ["Chest-Supported Dumbbell Row", "Dumbbell · Back"], ["Inverted Row", "Bodyweight · Back"], ["T-Bar Row", "Barbell · Back"]];
const catalogBody = () => `<div class="srch">${ico.search}<span>Search 1,527 exercises</span></div>
  <div class="cats">${CATS.map(([c, n]) => `<span class="${c === "Row" ? "on" : ""}">${c}<small class="n">${n}</small></span>`).join("")}<span>+35 more</span></div>
  <div class="clist">${ROWS.map(([n, d, added]) => `<div class="cl"><div><b>${n}</b><span>${d}</span></div><span class="cl-a${added ? " added" : ""}">${added ? ico.check : ico.plus}</span></div>`).join("")}</div>`;

function builder() {
  const tpl = [["Upper A", "Done Tue 14"], ["Lower A", "Done Wed 15"], ["Upper B", "Sat 18"], ["Lower B", "Not scheduled"]];
  return shell(`${back("training", "Training")}
  <header class="top"><div><div class="eyebrow">Strength workout</div><h1>Upper B</h1></div>
    <div class="hact">${stepMeter()}${helpBtn("help")}<button class="btn primary desk-only-i">Save · Sat 18</button></div></header>
  <div class="cols c3">
    <div class="col desk-only">
      <div class="card"><h3>Workouts</h3>${tpl.map(([n, d]) => `<div class="tpl${n === "Upper B" ? " on" : ""}"><span class="act-ico sm">${ico.lift}</span><div><b>${n}</b><span>${d}</span></div></div>`).join("")}<div class="st-add">${ico.plus}New workout</div></div>
      <div class="card"><h3>Add exercise</h3>${catalogBody()}</div>
    </div>
    <div class="col">
      ${UPPER_B.map(bxCard).join("")}
      <div class="addex tap" data-go="catalog">${ico.plus}Add exercise</div>
    </div>
    <div class="col">
      <div class="card"><h3>Schedule</h3>${weekPick()}${segc(["Once", "Every Sat"], 0)}<div class="hintline">Your watch gets it on its next sync.</div></div>
      ${brakeCard()}
      <div class="card desk-only"><h3>Steps</h3><div class="steps-c">${stepMeter(110)}<div><div class="lf-top"><span>Sets</span><b>23</b></div><div class="lf-top"><span>Rests</span><b>22</b></div><div class="hintline">Watches take about 100 steps.</div></div></div></div>
      <div class="card desk-only"><h3>Equipment</h3><div class="eqs"><div><span>Barbell plates</span><b class="n">1.25<small>kg pair</small></b></div><div><span>Dumbbells</span><b class="n">2<small>kg steps</small></b></div><div><span>Cable stack</span><b class="n">5<small>kg steps</small></b></div></div></div>
    </div>
  </div>`, "training", {bar: `<div class="savebar fold-bar"><div class="sb-in"><div><b class="n">7 exercises · ${STEPS} steps</b><span>Sat 18</span></div><button class="btn primary">Save</button></div></div>`});
}

function catalog() {
  return shell(`${back("builder", "Upper B")}
  <header class="top"><div><div class="eyebrow">1,527 exercises</div><h1>Add exercise</h1></div></header>
  <div class="card"><h3>Recent</h3><div class="chips-l">${["Barbell Back Squat", "Lat Pulldown", "Romanian Deadlift", "Plank"].map(n => `<span class="chipb">${n}</span>`).join("")}</div></div>
  <div class="card">${catalogBody()}</div>
  <div class="hintline center">Only Garmin's exercises, so your watch can name every set.</div>`, "training");
}

/* ---------- Strength history ---------- */
const SQUAT = {w: [85, 87.5, 87.5, 90, 90, 90, 92.5, 95, 95, 97.5, 97.5, 100], r: [5, 5, 5, 5, 5, 4, 5, 5, 5, 5, 5, 5]};
const EXS = [
  ["Barbell Back Squat", "100 × 5 · 5 · 5", SQUAT.w, true],
  ["Barbell Bench Press", "70 × 12 · 12 · 12", [62.5, 62.5, 65, 65, 65, 67.5, 67.5, 67.5, 70, 70, 70, 70]],
  ["Romanian Deadlift", "80 × 8 · 8 · 8", [70, 70, 72.5, 72.5, 75, 75, 75, 77.5, 77.5, 80, 80, 80]],
  ["One-Arm Dumbbell Row", "30 × 12 · 12 · 11", [26, 26, 26, 28, 28, 28, 28, 30, 30, 30, 30, 30]],
  ["Overhead Barbell Press", "45 × 6 · 5 · 5", [40, 40, 42.5, 42.5, 42.5, 45, 45, 45, 45, 45, 45, 45]],
  ["Lat Pulldown", "55 × 10 · 10 · 10", [50, 50, 50, 50, 55, 55, 55, 55, 55, 55, 55, 55]],
  ["Dumbbell Walking Lunge", "16 × 12 · 12 · 12", [12, 12, 14, 14, 14, 14, 16, 16, 16, 16, 16, 16]],
];
const MUSCLES = [["Chest", 3, 8], ["Back", 6, 11], ["Shoulders", 3, 7], ["Arms", 2, 5], ["Quads", 6, 8], ["Hamstrings", 6, 7], ["Glutes", 9, 10], ["Calves", 3, 4], ["Core", 0, 4]];
const WEEK = Object.fromEntries(MUSCLES.map(([m, v]) => [m, v]));

function trendChart(W, H) {
  const {w, r} = SQUAT, lo = 80, hi = 105, pl = 30, pb = 20, X = i => pl + 6 + i * (W - pl - 14) / (w.length - 1), Y = v => 8 + (hi - v) / (hi - lo) * (H - pb - 14);
  const pts = w.map((v, i) => [X(i), Y(v)]);
  let s = `<svg width="${W}" height="${H}" viewBox="0 0 ${W} ${H}" style="display:block;max-width:100%;height:auto"><defs><linearGradient id="trf${W}" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#E4E8F0" stop-opacity=".12"/><stop offset="1" stop-color="#E4E8F0" stop-opacity="0"/></linearGradient></defs>`;
  [80, 90, 100].forEach(v => s += `<line x1="${pl}" x2="${W}" y1="${Y(v)}" y2="${Y(v)}" stroke="rgba(255,255,255,.06)"/><text x="0" y="${Y(v) + 4}" fill="#8E98AC" font-family="Outfit" font-size="11">${v}</text>`);
  const d = smooth(pts);
  s += `<path d="${d}L${X(w.length - 1)},${H - pb}L${X(0)},${H - pb}Z" fill="url(#trf${W})"/><path d="${d}" fill="none" stroke="${C.ink}" stroke-width="2.2" stroke-linecap="round"/>`;
  pts.forEach(([x, y], i) => {
    const lastPt = i === w.length - 1;
    s += lastPt ? `<circle cx="${x}" cy="${y}" r="6.5" fill="${C.good}" stroke="#0B0F1A" stroke-width="3"/>` : `<circle cx="${x}" cy="${y}" r="3" fill="#0B0F1A" stroke="${C.ink}" stroke-width="2"/>`;
    if (r[i] < 5) s += `<text x="${x}" y="${y + 18}" fill="#8E98AC" font-family="Outfit" font-size="10" text-anchor="middle">×${r[i]}</text>`;
  });
  [["Jul", 0], ["Sep", 5], ["Oct", 10]].forEach(([l, i]) => s += `<text x="${X(i)}" y="${H - 3}" fill="#8E98AC" font-family="Inter" font-size="11" text-anchor="middle">${l}</text>`);
  return s + "</svg>";
}

const muscleList = () => `<div class="mus">${MUSCLES.map(([m, v, avg]) => `<div class="mu"><span>${m}</span><div class="mu-b"><i style="width:${v / 14 * 100}%;background:${amountColor(v, 12)}"></i><u style="left:${avg / 14 * 100}%"></u></div><b class="n">${v}</b></div>`).join("")}</div>
  <div class="lgd"><span><i class="tick"></i>4-week average</span></div>`;

function heat() {
  const r = rng(11), weeks = ["4 Aug", "", "", "", "1 Sep", "", "", "", "29 Sep", "", "", "Now"];
  let s = `<div class="heat"><span></span>${weeks.map(w => `<small>${w}</small>`).join("")}`;
  MUSCLES.forEach(([m, v, avg]) => {
    s += `<span>${m}</span>` + weeks.map((_, i) => { const x = i === 11 ? v : i === 5 ? Math.round(avg * .3) : Math.max(0, Math.round(avg + (r() - .5) * 5)); return `<i class="n" style="background:${amountColor(x, 12)};color:${onFill(amountColor(x, 12))}">${x}</i>`; }).join("");
  });
  return s + "</div>";
}

function strengthHistory() {
  const recs = [[1, "—", ""], [3, "105", "Aug"], [5, "100", "Wed", true], [8, "90", "Sep"], [10, "85", "Aug"]];
  const sessions = [["Wed 15 Oct", "100", "5 · 5 · 5", true], ["Wed 8 Oct", "97.5", "5 · 5 · 5"], ["Wed 1 Oct", "97.5", "5 · 5 · 5"], ["Wed 24 Sep", "95", "5 · 5 · 5"]];
  return shell(`${back("training", "Training")}
  <header class="top"><div><div class="eyebrow">Last 12 weeks</div><h1>Strength</h1></div><div class="hact">${helpBtn("help")}<button class="btn sm tap" data-go="builder">${ico.lift}Workouts</button></div></header>
  <div class="cols c3">
    <div class="col desk-only">
      <div class="card"><h3>Exercises</h3><div class="srch">${ico.search}<span>Find an exercise</span></div>
        ${EXS.map(([n, last, tr, on]) => `<div class="exl${on ? " on" : ""}"><div><b>${n}</b><span class="n">${last}</span></div>${spark(tr, {w: 64, h: 26, color: C.ink2, fill: false, sw: 1.6})}</div>`).join("")}</div>
    </div>
    <div class="col">
      <div class="card"><h3>This week ${segc(["Week", "4 weeks"], 0, "mini")}</h3><div class="bodywrap">${bodyMap("female", WEEK)}</div>${amountKey(12)}</div>
      <div class="exchips">${EXS.slice(0, 4).map(([n], i) => `<span class="${i === 0 ? "on" : ""}">${n.replace("Barbell ", "")}</span>`).join("")}</div>
      <div class="card"><h3>Barbell Back Squat <span class="pill ok">${ico.star}New 5-rep best</span></h3>
        <div class="tr-top"><div class="big n">100<span class="unit">kg × 5</span></div><span class="dim">Heaviest set each session</span></div>
        <div class="narrow-only">${trendChart(326, 150)}</div><div class="wide-only">${trendChart(440, 200)}</div></div>
      <div class="card"><h3>Best at each rep count</h3><div class="recs">${recs.map(([n, v, d, nw]) => `<div class="rec${nw ? " new" : ""}"><span>${n} rep${n > 1 ? "s" : ""}</span><b class="n">${v}</b><small>${d}</small></div>`).join("")}</div></div>
      <div class="card"><h3>Sessions</h3>${sessions.map(([d, w, rr, pr]) => `<div class="ses"><span>${d}</span><b class="n">${w}<small>kg</small></b><span class="n rr">${rr}</span><span class="pr">${pr ? ico.star : ""}</span></div>`).join("")}</div>
    </div>
    <div class="col">
      <div class="card"><h3>Sets per muscle <span class="h-note">Mon – Thu</span></h3>${muscleList()}</div>
      <div class="card wide-only-b"><h3>Weekly sets · 12 weeks</h3>${heat()}</div>
    </div>
  </div>`, "training");
}

function strengthCard() {
  return `<div class="card tap" data-go="strength-history"><h3>Strength · this week <span class="pill ok">${ico.star}1 record</span></h3>
    <div class="st-sum"><div><div class="big n">29</div><div class="small" style="color:#B4BCCB">sets · 2 sessions</div></div>${bodyMap("female", WEEK, {w: 150})}</div></div>`;
}

/* ---------- Strength intro ---------- */
function introArt(step) {
  if (step === 1) return `<svg width="220" height="190" viewBox="0 0 220 190"><rect x="80" y="2" width="60" height="34" rx="10" fill="#1A2030"/><rect x="80" y="154" width="60" height="34" rx="10" fill="#1A2030"/>
    <circle cx="110" cy="95" r="70" fill="#121826" stroke="#2A3142" stroke-width="6"/><rect x="178" y="118" width="8" height="22" rx="3" fill="${C.ink}"/><rect x="178" y="58" width="8" height="18" rx="3" fill="#2A3142"/><rect x="34" y="70" width="8" height="18" rx="3" fill="#2A3142"/>
    <circle cx="182" cy="129" r="17" fill="none" stroke="${C.ink}" stroke-opacity=".35" stroke-width="2"/><text x="205" y="160" fill="${C.ink}" font-family="Inter" font-size="12" font-weight="600" text-anchor="middle">Lap</text>
    <text x="110" y="70" fill="#8E98AC" font-family="Inter" font-size="11" font-weight="600" letter-spacing="1.5" text-anchor="middle">SET 2</text><text x="110" y="112" fill="${C.ink}" font-family="Outfit" font-size="44" font-weight="600" text-anchor="middle">10</text><text x="110" y="132" fill="#B4BCCB" font-family="Inter" font-size="12" text-anchor="middle">reps · 60 kg</text></svg>`;
  if (step === 2) return `<svg width="240" height="170" viewBox="0 0 240 170">${[[8, 1], [8, 1], [0, 2], [8, 1]].map(([r, k], i) => `<rect x="${30 + i * 50}" y="${120 - (r ? r * 11 : 4)}" width="34" height="${r ? r * 11 : 4}" rx="7" fill="${k === 2 ? C.fair : "rgba(228,232,240,.75)"}"/><text x="${47 + i * 50}" y="${112 - (r ? r * 11 : 4)}" fill="${k === 2 ? C.fair : C.ink}" font-family="Outfit" font-size="15" font-weight="600" text-anchor="middle">${r}</text>`).join("")}
    <line x1="20" x2="220" y1="122" y2="122" stroke="rgba(255,255,255,.1)"/><circle cx="147" cy="150" r="13" fill="${C.fair}"/><path d="M147 143v8M147 155h.01" stroke="#0B0F1A" stroke-width="2.4" stroke-linecap="round"/></svg>`;
  return `<svg width="250" height="150" viewBox="0 0 250 150"><rect x="14" y="30" width="54" height="94" rx="12" fill="#121826" stroke="#2A3142" stroke-width="3"/><rect x="24" y="48" width="34" height="6" rx="3" fill="${C.ink}"/><rect x="24" y="60" width="26" height="6" rx="3" fill="#2A3142"/><rect x="24" y="72" width="30" height="6" rx="3" fill="#2A3142"/>
    <path d="M78 77h22" stroke="#8E98AC" stroke-width="2" stroke-dasharray="3 4"/><rect x="104" y="44" width="62" height="62" rx="14" fill="#121826" stroke="#2A3142" stroke-width="3"/><text x="135" y="68" fill="#8E98AC" font-family="Inter" font-size="10" font-weight="600" text-anchor="middle">SAT</text><text x="135" y="94" fill="${C.ink}" font-family="Outfit" font-size="26" font-weight="600" text-anchor="middle">18</text>
    <path d="M172 77h22" stroke="#8E98AC" stroke-width="2" stroke-dasharray="3 4"/><circle cx="220" cy="77" r="24" fill="#121826" stroke="#2A3142" stroke-width="4"/><path d="M211 77h18M214 72v10M226 72v10" stroke="${C.ink}" stroke-width="2.4" stroke-linecap="round"/></svg>`;
}
const INTRO = [
  ["Lift with your watch", "Start a Strength activity. Press lap at the end of each set. Your watch counts the reps."],
  ["Fix it in a minute", "Watches miss and double reps. Raring flags the odd sets so you can fix them after your workout."],
  ["Plan workouts here", "Build a workout and pick a day. Your watch gets it on its next sync, weights filled in."],
];
function intro(step) {
  const dots = `<div class="ci-dots">${[1, 2, 3, 4].map(i => `<span class="${i <= step ? "on" : ""}"></span>`).join("")}</div>`;
  if (step === 4) return shell(`
    <header class="top"><div><div class="eyebrow">Strength · 4 of 4</div><h1>Set up</h1></div><button class="ibtn" aria-label="Close">${ico.close}</button></header>${dots}
    <div class="card"><h3>Your watch</h3><div class="trow"><span class="g-i">${ico.watch}</span><div><b class="t">Forerunner 970</b><span>Records sets and gets workouts</span></div><span class="pill ok">${ico.check}Tested</span></div></div>
    <div class="card"><h3>Body map</h3><div class="sexpick"><div class="on">${bodyMap("female", {Quads: 6, Glutes: 8, Back: 5, Chest: 3, Shoulders: 4}, {w: 130})}<span>${ico.check}Female</span></div><div>${bodyMap("male", {Quads: 6, Glutes: 8, Back: 5, Chest: 3, Shoulders: 4}, {w: 130})}<span>Male</span></div></div></div>
    <div class="card"><h3>Weights</h3><div class="eqs"><div><span>Smallest plates</span>${stepper(1.25, "kg", "sm")}</div><div><span>Dumbbell steps</span>${stepper(2, "kg", "sm")}</div></div><div class="hintline">Kilograms, from your Garmin settings.</div></div>
    <div class="card"><div class="trow"><span class="g-i">${ico.up}</span><div><b class="t">Suggest weights</b><span>Turn off per exercise any time</span></div>${sw(true)}</div></div>
    <div class="intro-btns"><button class="btn primary block tap" data-go="strength-history">Done</button></div>`, "training", {modal: true});
  const [t, p] = INTRO[step - 1];
  return shell(`
    <header class="top"><div><div class="eyebrow">Strength · ${step} of 4</div></div><button class="ibtn" aria-label="Close">${ico.close}</button></header>${dots}
    <div class="intro"><div class="intro-art">${introArt(step)}</div><h1>${t}</h1><p>${p}</p></div>
    <div class="intro-btns"><button class="btn primary block tap" data-go="intro-${step + 1}">Next</button><button class="btn ghost block">Skip</button></div>`, "training", {modal: true});
}

/* ---------- Help ---------- */
const HELP = [
  ["Start here", [[ico.book, "How Raring reads Garmin"], [ico.sync, "Keep Garmin Connect syncing"]]],
  ["Your day", [[ico.gauge, "Readiness and today's call"], [ico.moon, "Tonight's asleep-by time"]]],
  ["Weekly plan", [[ico.target, "How targets count"], [ico.pause, "Rest days and paused weeks"]]],
  ["Strength", [[ico.lift, "Getting started", "intro-1"], [ico.watch, "How sets are counted", "help-sheet"], [ico.pencil, "Fixing sets", "fix"], [ico.up, "Suggested weights"], [ico.lock, "Readiness brake"]]],
  ["Watches and data", [[ico.watch, "Does my watch work?", "", true], [ico.shield, "Privacy"], [ico.save, "Backup and export"]]],
];
function help() {
  return shell(`${back("today", "Today")}
  <header class="top"><div><h1>Help</h1></div></header>
  <div class="srch big">${ico.search}<span>Search help</span></div>
  <div class="hgrid">${HELP.map(([h, rows]) => `<div class="card"><h3>${h}</h3>${rows.map(([i, t, go, ext]) => `<div class="hrow${go ? " tap" : ""}"${go ? ` data-go="${go}"` : ""}><span class="g-i">${i}</span><b>${t}</b><span class="chev">${ext ? ico.out : ico.chev}</span></div>`).join("")}</div>`).join("")}</div>
  <div class="hintline center">Still stuck? Ask on GitHub ${ico.out}</div>`, "today");
}
function helpSheet() {
  return `${strengthWorkout()}<div class="scrim"></div><div class="sheet"><span class="grab"></span>
    <div class="sheet-h"><h2>How sets are counted</h2><button class="ibtn" aria-label="Close">${ico.close}</button></div>
    <ol class="steps-l"><li><span class="n">1</span>Your watch counts reps and guesses the exercise.</li><li><span class="n">2</span>Pressing lap ends a set. Rests are separate.</li><li><span class="n">3</span>Raring flags 0-rep sets, doubled sets and unnamed exercises.</li><li><span class="n">4</span>Your fixes update the activity in Garmin.</li></ol>
    <div class="ci-btns"><button class="btn primary">Got it</button><button class="btn tap" data-go="help">More help</button></div></div>`;
}

/* ---------- Home-screen widget ---------- */
function widget() { return statusWidgetHome(); }

/* ---------- Design language ---------- */
function language() {
  const sw2 = (c, n, d) => `<div class="sw2"><i style="background:${c}"></i><b>${n}</b><span>${d}</span></div>`;
  const st = (v, c, l) => `<div class="dial">${arc({size: 86, value: v, color: c, stroke: 8})}<div class="dial-c"><b class="n" style="font-size:24px">${v}</b><em style="color:${c};font-size:10px">${l}</em></div></div>`;
  return shell(`
  <header class="top"><div><div class="eyebrow">Raring</div><h1>Design language</h1></div></header>
  <p class="lede">Color means something or it isn't there. Icons and titles say where you are; color says how it's going.</p>
  <div class="lgrid">
    <div class="card"><h3>Status</h3><p class="lp">How something is going. Only these three.</p>
      <div class="sws">${sw2(C.good, "Good", "In range, done, improving")}${sw2(C.fair, "Fair", "Watch it, behind, check")}${sw2(C.poor, "Poor", "Low, out of range, failed")}${sw2("transparent", "No data", "A gap, never a zero")}</div>
      <div class="lrow">${st(82, C.good, "High")}${st(58, C.fair, "Moderate")}${st(34, C.poor, "Low")}</div></div>
    <div class="card"><h3>Effort</h3><p class="lp">Heart-rate zones only, cool to hot. Red is hard.</p>
      <div class="zramp">${C.z.map((c, i) => `<div><i style="background:${c}"></i><span>Z${i + 1}</span></div>`).join("")}</div>
      ${zoneBar([3.2, 9.75, 18.3, 17.5, 3.5])}</div>
    <div class="card"><h3>Amount</h3><p class="lp">How much, with no judgement. One blue, faint to strong.</p>
      ${amountKey(12)}<div class="lbodies">${bodyMap("female", WEEK, {w: 150})}${bodyMap("male", WEEK, {w: 150})}</div></div>
    <div class="card"><h3>Ink</h3><p class="lp">Everything else: numbers, charts without a verdict, controls. Text is at least 4.5:1 on every surface.</p>
      <div class="wk" style="height:80px">${[["M", 0], ["T", 95], ["W", 320], ["T", 186]].map(([d, v], i) => `<div>${v ? `<b>${v}</b>` : ""}<i style="height:${Math.max(v / 340 * 56, 4)}px;background:${i === 3 ? C.ink : "rgba(228,232,240,.30)"}"></i><span>${d}</span></div>`).join("")}</div>
      <div class="lbtns"><button class="btn primary sm">Primary</button><button class="btn sm">Secondary</button><button class="btn sm ghost">Quiet</button></div></div>
    <div class="card"><h3>Progress</h3><p class="lp">Partial credit. The bar turns amber when behind pace, green when done.</p>
      <div class="goals">${goalRow(PT.intensity, true)}${goalRow(PT.z2, true)}${goalRow({...PT.strength, done: 3}, true)}</div></div>
    <div class="card"><h3>Type</h3><div class="ltype"><b class="n">82</b><div><b>Outfit</b><span>Numbers and headings</span></div></div><div class="ltype"><span class="inter">Aa</span><div><b>Inter</b><span>Text and labels</span></div></div>
      <div class="tags"><span class="pill">Neutral</span><span class="pill ok">Good</span><span class="pill warn">Check</span></div></div>
  </div>`, "today");
}

SCREENS.unshift(["language", "Design language", language, "phone", "Design language"], ["language", "Design language", language, "desk", "Design language"]);
SCREENS.push(
  ["plan", "Weekly plan", plan, "phone", "Weekly plan"],
  ["plan-setup", "Plan targets", planSetup, "phone", "Weekly plan"],
  ["checkin", "Monday check-in", checkin, "phone", "Weekly plan"],
  ["plan", "Weekly plan", plan, "fold", "Weekly plan"],
  ["plan-setup", "Plan targets", planSetup, "desk", "Weekly plan"],
  ["intro-1", "Strength intro 1", () => intro(1), "phone", "Strength"],
  ["intro-2", "Strength intro 2", () => intro(2), "phone", "Strength"],
  ["intro-3", "Strength intro 3", () => intro(3), "phone", "Strength"],
  ["intro-4", "Strength intro 4", () => intro(4), "phone", "Strength"],
  ["strength-workout", "Strength workout", strengthWorkout, "phone", "Strength"],
  ["fix", "Fix sets", fixSets, "phone", "Strength"],
  ["strength-history", "Strength history", strengthHistory, "phone", "Strength"],
  ["strength-workout", "Strength workout", strengthWorkout, "fold", "Strength"],
  ["strength-history", "Strength history", strengthHistory, "desk", "Strength"],
  ["builder", "Workout builder", builder, "phone", "Workout builder"],
  ["catalog", "Add exercise", catalog, "phone", "Workout builder"],
  ["builder", "Workout builder", builder, "desk", "Workout builder"],
  ["help", "Help", help, "phone", "Help"],
  ["help-sheet", "Help on a screen", helpSheet, "phone", "Help"],
  ["help", "Help", help, "desk", "Help"],
  ["widget", "Home-screen widgets", widget, "phone", "Widget"],
);
