/* Weekly plan, strength, builder and widget screens. Runs after index.html's inline script and
   reuses its helpers (C, ico, svgI, arc, spark, smooth, rng, statusBar, tabs, zoneBar). All data is synthetic. */

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
  upload: svgI('<path d="M12 15V4.5M7.5 9L12 4.5 16.5 9"/><path d="M5 15v3a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2v-3"/>'),
  chev: svgI('<path d="M9 5l7 7-7 7"/>'),
  alert: svgI('<path d="M12 4.5l8.5 15h-17z"/><path d="M12 10.5v3.5M12 16.8h.01"/>'),
  sync: svgI('<path d="M19.5 12a7.5 7.5 0 0 1-13 5.1M4.5 12a7.5 7.5 0 0 1 13-5.1"/><path d="M17.5 3.5v3.4h-3.4M6.5 20.5v-3.4h3.4"/>'),
  undo: svgI('<path d="M9 7L5 11l4 4"/><path d="M5 11h9a5 5 0 0 1 0 10h-2"/>'),
});

const hexA = (h, a) => `rgba(${parseInt(h.slice(1, 3), 16)},${parseInt(h.slice(3, 5), 16)},${parseInt(h.slice(5, 7), 16)},${a})`;
const back = (go, label) => `<button class="back" data-go="${go}">${ico.back.replace("<svg ", '<svg width="20" height="20" ')}${label}</button>`;
const shell = (glow, body, tab, extra = "") => `<div class="glow" style="--glow:${glow}"></div>${statusBar()}
  <div class="scroll${extra ? " has-bar" : ""}"><div class="page">${body}</div></div>${extra}${extra ? tabs(tab).replace('class="tabs"', 'class="tabs off"') : tabs(tab)}`;
const kg = w => w == null ? "—" : String(w);
const stepper = (v, unit = "", cls = "") => `<div class="stp ${cls}"><button>−</button><b class="n">${v}${unit ? `<small>${unit}</small>` : ""}</b><button>+</button></div>`;
const segc = (opts, on, cls = "") => `<div class="segc ${cls}">${opts.map((o, i) => `<span${i === on ? ' class="on"' : ""}>${o}</span>`).join("")}</div>`;
const sw = (on, color = C.mint) => `<span class="sw${on ? " on" : ""}" style="--c:${color}"></span>`;

// An arc with a tick where the target would be if progress were even through the week.
function tarc(o, pace) {
  const s = arc(o);
  if (pace == null) return s;
  const {size, stroke = 10, sweep = 270} = o, r = (size - stroke) / 2 - 3, c = size / 2, a = (90 + (360 - sweep) / 2 + sweep * pace) * Math.PI / 180;
  const r0 = r - stroke / 2 - 3, r1 = r + stroke / 2 + 3;
  return s.replace(/<\/svg>$/, `<line x1="${c + r0 * Math.cos(a)}" y1="${c + r0 * Math.sin(a)}" x2="${c + r1 * Math.cos(a)}" y2="${c + r1 * Math.sin(a)}" stroke="#EEF1F7" stroke-width="2.2" stroke-linecap="round" opacity=".8"/></svg>`);
}

/* ---------- Weekly plan data ---------- */
// Week Mon 13 – Sun 19 Oct; today is Thu 16. Rest days Mon and Fri, so three of five training days
// have passed, and four of seven nights.
const PT = {
  strength: {name: "Strength", long: "Strength sessions", unit: "sessions", done: 2, target: 3, color: C.rose, color2: C.rose2, icon: ico.lift, pace: .6},
  intensity: {name: "Intensity", long: "Intensity minutes", unit: "min", done: 124, target: 150, color: C.flame, color2: "#FF5E3A", icon: ico.flame, pace: .6},
  z2: {name: "Zone 2", long: "Zone 2 minutes", unit: "min", done: 46, target: 90, color: C.sky, color2: "#3D8BFF", icon: ico.pulse, pace: .6},
  sleep: {name: "Asleep by", long: "Nights asleep on time", unit: "nights", done: 3, target: 5, color: C.violet, color2: "#6E5BFF", icon: ico.moon, pace: 4 / 7},
};
const TG = [PT.strength, PT.intensity, PT.z2, PT.sleep];
const DAYS = [
  {d: "M", n: 13, rest: true, sleep: "ok"},
  {d: "T", n: 14, str: "done", im: 18, z2: 6, sleep: "ok"},
  {d: "W", n: 15, str: "done", im: 46, z2: 30, sleep: "late"},
  {d: "T", n: 16, today: true, im: 60, z2: 10, sleep: "ok"},
  {d: "F", n: 17, rest: true},
  {d: "S", n: 18, str: "planned"},
  {d: "S", n: 19},
];
const HIST = {weeks: ["25 Aug", "1 Sep", "8 Sep", "15 Sep", "22 Sep", "29 Sep", "6 Oct", "13 Oct"],
  strength: [2, 3, 3, null, 2, 3, 3, 2], intensity: [130, 155, 170, null, 120, 150, 162, 124], z2: [60, 75, 90, null, 70, 85, 72, 46], sleep: [3, 4, 5, null, 4, 5, 4, 3]};

const ring = (t, size, stroke, {pace, done = t.done, target = t.target} = {}) => {
  const met = done >= target;
  return `<div class="dial tg-ring" style="--s:${size}px">${tarc({size, value: Math.min(done, target), max: target, color: t.color, color2: t.color2, stroke}, met ? null : pace)}
    <div class="dial-c"><b class="n">${done}</b><span>${met && size >= 100 ? `<i style="color:${t.color}">${ico.check}</i>` : ""}of ${target}</span></div></div>`;
};

function planCard() {
  return `<div class="card tap" data-go="plan"><h3>This week <span class="pill ok">3 of 4 on pace</span></h3>
    <div class="tg4">${TG.map(t => `<div class="tg">${ring(t, 74, 7, {pace: t.pace})}<em>${t.name}</em></div>`).join("")}</div>
    <div class="sub plan-next"><span style="color:${C.rose}">${ico.lift}</span>Upper B · Sat<span class="dim">· Fri rest</span></div></div>`;
}

function dayStrip() {
  return `<div class="days">${DAYS.map(d => `<div class="day${d.rest ? " rest" : ""}${d.today ? " today" : ""}">
    <b>${d.d}</b><span class="n dn">${d.n}</span>
    <i class="tok ${d.str || (d.rest ? "restt" : "none")}">${d.str ? ico.lift : d.rest ? ico.pause : ""}</i>
    <div class="mb">${d.im != null ? `<i style="height:${Math.max(d.im / 60 * 100, 8)}%;background:${C.flame}"></i><i style="height:${Math.max(d.z2 / 30 * 100, 8)}%;background:${C.sky}"></i>` : ""}</div>
    <i class="zz ${d.sleep || "none"}">${d.sleep === "ok" ? ico.check : d.sleep === "late" ? "+30" : ""}</i></div>`).join("")}</div>
  <div class="lgd"><span><i style="background:${C.rose}"></i>Strength</span><span><i style="background:${C.flame}"></i>Intensity</span><span><i style="background:${C.sky}"></i>Zone 2</span><span><i style="background:${C.violet}"></i>Asleep by</span><span><i class="hatch"></i>Rest</span></div>`;
}

const pie = (f, color, now) => `<i class="pie${f >= 1 ? " full" : ""}${now ? " now" : ""}" style="--c:${color};--f:${Math.min(f, 1)}">${f >= 1 ? ico.check : ""}</i>`;
function histGrid() {
  const W = HIST.weeks, keys = ["strength", "intensity", "z2", "sleep"], last = W.length - 1;
  let s = `<div class="hist"><span></span>${W.map((w, i) => {
    const [d, m] = w.split(" "), showM = i === 0 || W[i - 1].split(" ")[1] !== m;
    return `<span class="hw${i === last ? " now" : ""}"><b class="n">${i === last ? "Now" : d}</b><small>${i === last ? "&nbsp;" : showM ? m : "&nbsp;"}</small></span>`;
  }).join("")}`;
  keys.forEach(k => {
    const t = PT[k];
    s += `<span class="hk" style="color:${t.color}">${t.icon}</span>` + HIST[k].map((v, i) => v == null ? `<span class="hp">${ico.pause}</span>` : pie(v / t.target, t.color, i === last)).join("");
  });
  s += `<span class="hk met">Met</span>` + W.map((_, i) => HIST.strength[i] == null ? `<span class="hm">Paused</span>`
    : `<span class="hm n">${keys.filter(k => HIST[k][i] >= PT[k].target).length}<small>/4</small></span>`).join("");
  return s + "</div>";
}

/* ---------- Weekly plan ---------- */
function plan() {
  return shell("rgba(61,220,151,.20)", `${back("today", "Today")}
  <header class="top"><div><div class="eyebrow">Mon 13 – Sun 19 Oct</div><h1>This week</h1></div><button class="ibtn tap" data-go="plan-setup" aria-label="Edit targets">${ico.pencil}</button></header>
  <div class="cols">
    <div class="col">
      <section class="plan-hero">${TG.map(t => `<div class="tg">${ring(t, 140, 11, {pace: t.pace})}<em>${t.long}</em></div>`).join("")}</section>
      <div class="headline"><h2>Nearly on pace</h2>
        <div class="chips"><span class="chip">${ico.clock}<b class="n">3</b> days left</span><span class="chip" style="color:${C.sky}">${ico.pulse}<b class="n">44</b><span style="color:#A3ACBF">min Zone 2 to go</span></span></div>
        <div class="tickkey"><i></i>Even pace for Thursday</div></div>
    </div>
    <div class="col">
      <div class="card"><h3>Day by day</h3>${dayStrip()}</div>
      <div class="card tap" data-go="builder"><h3>Next up</h3>
        <div class="act"><div class="act-ico" style="background:rgba(255,111,181,.15);color:${C.rose}">${ico.lift}</div>
          <div class="act-main"><b>Upper B</b><span>Sat 18 · 7 exercises · in Garmin's calendar</span></div><span class="chev">${ico.chev}</span></div>
        <div class="sub" style="color:#A3ACBF"><span style="color:#A3ACBF">${ico.pause}</span>Fri 17 · planned rest</div></div>
    </div>
  </div>
  <div class="card"><h3>Week by week <span class="pill">8 weeks</span></h3>${histGrid()}</div>`, "today");
}

function planSetup() {
  const head = (t, sub, on = true) => `<div class="trow"><div class="ticon" style="background:${hexA(t.color, .15)};color:${t.color}">${t.icon}</div><div><b class="t">${t.long}</b><span>${sub}</span></div>${sw(on, t.color)}</div>`;
  const zstrip = `<div class="zstrip">${C.z.map((c, i) => `<i class="${i === 1 ? "on" : ""}" style="background:${c}">${i === 1 ? "Z2" : ""}</i>`).join("")}</div><div class="hintline"><b class="n">128–145</b> bpm · Garmin's heart-rate zones</div>`;
  const targets = `
    <div class="card">${head(PT.strength, "Any Garmin Strength activity")}<div class="tset">${stepper(3)}<span class="u">sessions a week</span></div></div>
    <div class="card">${head(PT.intensity, "Vigorous minutes count double")}<div class="tset"><div class="goalv"><b class="n">150</b><span>min a week</span></div><span class="pill">${ico.sync}Garmin's goal</span></div>
      ${segc(["Intensity minutes", "Vigorous only"], 0)}</div>
    <div class="card">${head(PT.z2, "Zone 2 alone, from workouts")}<div class="tset">${stepper(90)}<span class="u">min a week</span></div>${zstrip}</div>
    <div class="card">${head(PT.sleep, "Within 15 min of the asleep-by time")}<div class="tset">${stepper(5)}<span class="u">of 7 nights</span></div></div>`;
  const last = [[PT.strength, 3], [PT.intensity, 162], [PT.z2, 72], [PT.sleep, 4]];
  return shell("rgba(61,220,151,.16)", `${back("plan", "This week")}
  <header class="top"><div><div class="eyebrow">Weekly plan · Mon – Sun</div><h1>Targets</h1></div></header>
  <div class="cols wide-l">
    <div class="col tgrid">${targets}</div>
    <div class="col">
      <div class="card"><h3>Rest days</h3>
        <div class="wdays">${["M", "T", "W", "T", "F", "S", "S"].map((d, i) => `<span class="${[0, 4].includes(i) ? "rest" : ""}">${[0, 4].includes(i) ? ico.pause : ""}${d}</span>`).join("")}</div>
        <div class="hintline">Nothing is due on a rest day</div></div>
      <div class="card"><div class="trow"><div class="ticon" style="background:rgba(255,255,255,.07);color:#A3ACBF">${ico.pause}</div><div><b class="t">Pause this week</b><span>Travel or illness · left out of history</span></div>${sw(false)}</div></div>
      <div class="card"><h3>Last week on these</h3><div class="tg4">${last.map(([t, v]) => `<div class="tg">${ring(t, 70, 6, {done: v})}<em>${t.name}</em></div>`).join("")}</div></div>
      <div class="hintline center">${ico.check}Saved on this device · in backup and export</div>
    </div>
  </div>`, "today");
}

function checkin() {
  const res = [[PT.strength, 3, 3], [PT.intensity, 162, 150], [PT.z2, 72, 85], [PT.sleep, 4, 5]];
  const delta = (a, b, u) => a === b ? `<span class="dl">= same</span>` : `<span class="dl ${a > b ? "up" : ""}">${a > b ? "▲" : "▼"} ${Math.abs(a - b)}${u}</span>`;
  return shell("rgba(61,220,151,.22)", `
  <header class="top"><div><div class="eyebrow">Weekly check-in · 6 – 12 Oct</div><h1>Your plan</h1></div><button class="ibtn" aria-label="Close">${svgI('<path d="M6 6l12 12M18 6L6 18"/>')}</button></header>
  <div class="ci-dots">${[0, 1, 2, 3, 4, 5].map(i => `<span class="${i <= 4 ? "on" : ""}"></span>`).join("")}</div>
  <div class="card ci-card">
    <div class="ci-head"><div class="n ci-big">3<small>of 4</small></div><div><h2>Targets met</h2><span class="dim">Zone 2 came up short</span></div></div>
    <div class="plan-hero sm">${res.map(([t, v, b]) => `<div class="tg">${ring(t, 118, 10, {done: v})}<em>${t.name}</em>${delta(v, b, t.unit === "min" ? " min" : "")}</div>`).join("")}</div>
    <div class="ci-next"><h3>This week</h3>
      <div class="ci-tgts">${TG.map(t => `<span style="color:${t.color}">${t.icon}<b class="n">${t.target}</b></span>`).join("")}<span class="dim">${ico.pause}Mon · Fri</span></div>
      <div class="ci-btns"><button class="btn mint">Keep targets</button><button class="btn tap" data-go="plan-setup">Adjust</button></div></div>
  </div>
  <div class="ci-ctl"><span>‹ Health</span><span style="color:${C.mint}">Win & focus ›</span></div>`, "today");
}

/* ---------- Strength workout ---------- */
const LOWER_A = [
  {name: "Barbell Back Squat", mus: "Quads · Glutes", last: "Wed 8 Oct", lastSets: [[97.5, 5], [97.5, 5], [97.5, 5]], pr: "100 × 5",
    sets: [{w: 60, r: 8, warm: true}, {w: 100, r: 5, pr: true}, {w: 100, r: 5}, {w: 100, r: 5}]},
  {name: "Romanian Deadlift", mus: "Hamstrings · Glutes", last: "Wed 8 Oct", lastSets: [[80, 8], [80, 8], [80, 7]],
    sets: [{w: 80, r: 8}, {w: 80, r: 8}, {w: 80, r: 0, flag: "0 reps"}, {w: 80, r: 8}]},
  {name: "Dumbbell Walking Lunge", mus: "Quads · Glutes", last: "Wed 8 Oct", lastSets: [[16, 12], [16, 12], [16, 12]],
    sets: [{w: 16, r: 12}, {w: 16, r: 24, flag: "Doubled?"}]},
  {name: "Not recognized", mus: "Garmin couldn't name it", unknown: true,
    sets: [{w: 40, r: 12, flag: "?"}, {w: 40, r: 12, flag: "?"}, {w: 40, r: 10, flag: "?"}]},
  {name: "Standing Calf Raise", mus: "Calves", last: "Wed 8 Oct", lastSets: [[40, 15], [40, 15], [40, 15]],
    sets: [{w: 40, r: 15}, {w: 40, r: 15}, {w: 40, r: 14}]},
];

function setBars(ex) {
  const maxR = Math.max(...ex.sets.map(s => s.r), ...(ex.lastSets || []).map(s => s[1]), 1);
  let j = 0;
  const bars = ex.sets.map(s => {
    const ghost = !s.warm && ex.lastSets ? ex.lastSets[j++] : null, h = r => Math.max(r / maxR * 62, 3);
    return `<div class="sbar${s.flag ? " flag" : ""}${s.warm ? " warm" : ""}"><b class="n">${s.r}</b>
      <div class="sbar-t">${ghost ? `<u style="height:${h(ghost[1])}px"></u>` : ""}<i style="height:${h(s.r)}px">${s.pr ? ico.star : ""}</i></div>
      <span class="n">${s.warm ? "W " : ""}${kg(s.w)}</span></div>`;
  }).join("");
  return `<div class="setbars">${bars}<div class="sbar key"><b>reps</b><div class="sbar-t"></div><span>kg</span></div></div>`;
}

function exCard(ex) {
  const flags = ex.sets.filter(s => s.flag).length;
  const tag = ex.unknown ? `<span class="pill warn">Pick exercise</span>` : flags ? `<span class="pill warn">${ico.alert}${flags} to check</span>` : ex.pr ? `<span class="pill rose">${ico.star}${ex.pr}</span>` : "";
  const last = ex.lastSets ? `<div class="lastl">${ico.clock}Last · ${ex.last} · <b class="n">${ex.lastSets.map(s => `${kg(s[0])}×${s[1]}`).join("  ")}</b></div>` : "";
  return `<div class="card ex"><div class="ex-h"><div><b>${ex.name}</b><span>${ex.mus}</span></div>${tag}</div>${setBars(ex)}${last}</div>`;
}

function strengthWorkout() {
  const sets = LOWER_A.reduce((a, e) => a + e.sets.length, 0), reps = LOWER_A.reduce((a, e) => a + e.sets.reduce((b, s) => b + s.r, 0), 0);
  return shell("rgba(255,111,181,.18)", `${back("training", "Training")}
  <header class="top" style="margin-top:2px"><div><div class="eyebrow">Strength · Wed 15 Oct · 18:05</div><h1>Lower A</h1></div><div class="act-ico" style="background:rgba(255,111,181,.15);color:${C.rose}">${ico.lift}</div></header>
  <div class="cols">
    <div class="col">
      <div class="card"><div class="sw-hero">
        <div><div class="eyebrow">Sets</div><div class="n hero-n" style="color:${C.rose}">${sets}</div>
          <div class="tags"><span class="pill rose">${ico.star}1 record</span><span class="pill">${ico.watch}Counted by watch</span></div></div>
        <div class="grid2"><div><b class="n">48:02</b><span>Time</span></div><div><b class="n">${reps}</b><span>Reps</span></div><div><b class="n" style="color:${C.flame}">74</b><span>Load</span></div><div><b class="n">118</b><span>Avg HR</span></div></div>
      </div></div>
      <div class="card alertc tap" data-go="fix"><div class="al"><span class="al-i">${ico.alert}</span><div><b>3 things look off</b><span>A 0-rep set, a doubled set, an unnamed exercise</span></div><button class="btn rose sm">Fix</button></div></div>
      <div class="card"><h3>Heart rate</h3>${zoneBar([14, 22, 9, 3, 0.1])}<div class="tags"><span class="pill ok">Aerobic 1.9 · Maintaining</span><span class="pill violet">Anaerobic 1.4</span></div></div>
    </div>
    <div class="col">${LOWER_A.map(exCard).join("")}</div>
  </div>`, "training");
}

/* ---------- Fix sets ---------- */
function fixSets() {
  const row = (n, w, r, st = "", extra = "") => `<div class="erow ${st}"><i class="n">${n}</i><span class="ev n">${kg(w)}<small>kg</small></span><span class="ex-x">×</span><span class="ev n">${r}<small>reps</small></span><span class="es">${st === "done" ? ico.pencil : st === "flag" ? ico.alert : ""}</span>${extra}</div>`;
  const editor = `<div class="editor">
    <div class="ed"><span class="ed-l">Weight <small>Barbell · 2.5 kg steps</small></span>${stepper(40, "kg", "lg")}</div>
    <div class="ed"><span class="ed-l">Reps</span>${stepper(15, "", "lg")}<div class="quick">${[10, 12, 14, 15].map(v => `<span class="${v === 15 ? "on" : ""}">${v}</span>`).join("")}</div></div>
    <div class="ed-a"><button class="btn sm">${ico.trash}Delete</button><button class="btn sm">Copy to next</button><button class="btn sm mint">Done</button></div></div>`;
  const body = `${back("strength-workout", "Lower A")}
  <header class="top"><div><div class="eyebrow">Fix sets · Wed 15 Oct</div><h1>3 to check</h1></div><span class="pill">${ico.upload}Writes to Garmin</span></header>
  <div class="cols">
    <div class="col">
      <div class="card"><h3>To check</h3>
        <div class="fx done"><span class="fx-i ok">${ico.check}</span><div class="fx-m"><b>Romanian Deadlift · set 3</b><span>0 reps · deleted</span></div><button class="btn sm ghost">${ico.undo}Undo</button></div>
        <div class="fx"><span class="fx-i">${ico.alert}</span><div class="fx-m"><b>Walking Lunge · set 2</b><span>24 reps · two sets joined?</span></div></div>
        <div class="fx-a"><button class="btn sm rose">${ico.split}Split 12 + 12</button><button class="btn sm">Keep 24</button></div>
        <div class="fx"><span class="fx-i">${ico.alert}</span><div class="fx-m"><b>Not recognized · 3 sets</b><span>40 kg · 12, 12, 10 reps</span></div></div>
        <div class="fx-a chips-l"><span class="chipb on">${ico.check}Seated Leg Curl</span><span class="chipb">Lying Leg Curl</span><span class="chipb">Leg Extension</span><span class="chipb">${ico.search}Search</span></div>
      </div>
    </div>
    <div class="col">
      <div class="card"><h3>All sets <span class="pill">${ico.watch}Watch · ${ico.pencil}Edited</span></h3>
        <div class="eg"><div class="eg-h"><b>Barbell Back Squat</b>${ico.swap}</div>${row("W", 60, 8)}${row(1, 100, 5)}${row(2, 100, 5)}${row(3, 100, 5)}</div>
        <div class="eg"><div class="eg-h"><b>Romanian Deadlift</b>${ico.swap}</div>${row(1, 80, 8)}${row(2, 80, 8)}<div class="erow gone"><i class="n">–</i><span class="ev n">80<small>kg</small></span><span class="ex-x">×</span><span class="ev n">0<small>reps</small></span><span class="es">${ico.trash}</span></div>${row(3, 80, 8)}</div>
        <div class="eg"><div class="eg-h"><b>Seated Leg Curl</b><span class="pill rose" style="margin-left:auto">Was unnamed</span></div>${row(1, 40, 12, "done")}${row(2, 40, 12, "done")}${row(3, 40, 10, "done")}</div>
        <div class="eg"><div class="eg-h"><b>Standing Calf Raise</b>${ico.swap}</div>${row(1, 40, 15)}${row(2, 40, 15)}${row(3, 40, 15, "done open")}${editor}</div>
      </div>
    </div>
  </div>`;
  const bar = `<div class="savebar"><div class="sb-in"><div><b class="n">5 changes</b><span>Nothing changes in Garmin until you save</span></div><button class="btn rose">${ico.upload}Save to Garmin</button></div></div>`;
  return shell("rgba(255,111,181,.14)", body, "training", bar);
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

function bxCard(e, i) {
  const sIco = {up: ico.up, same: ico.same, down: ico.down, learn: ico.learn}[e.s];
  const prog = e.prog === "auto"
    ? `<div class="sug ${e.s}"><span class="sg-i">${sIco}</span><div><b class="n">${e.sug}</b><span>${e.why}</span></div></div>`
    : `<div class="sug off"><span class="sg-i">${ico.pencil}</span><div><b class="n">${e.sets} × ${e.reps}${e.w ? ` · ${e.w} kg` : ""}</b><span>Your targets</span></div></div>`;
  const table = e.open ? `<div class="stab"><div class="st-h"><span>Set</span><span>kg</span><span>Reps</span><span>Rest</span></div>
    ${e.warm.map(([w, r]) => `<div class="st-r warm"><span>W</span><b class="n">${w}</b><b class="n">${r}</b><b class="n">1:00</b></div>`).join("")}
    ${[1, 2, 3].map(n => `<div class="st-r"><span class="n">${n}</span><b class="n">${e.w}</b><b class="n">8–12</b><b class="n">${n < 3 ? e.rest : "—"}</b></div>`).join("")}
    <div class="st-add">${ico.plus}Add set</div></div>`
    : `<div class="bsum"><span><b class="n">${e.sets}</b> sets</span><span><b class="n">${e.reps}</b>${e.reps.includes(":") ? "" : " reps"}</span><span><b class="n">${kg(e.w)}</b>${e.w ? " kg" : ""}</span><span>${ico.clock}<b class="n">${e.rest}</b></span></div>`;
  return `<div class="card bx${e.open ? " open" : ""}"><div class="bx-h"><span class="grip">${ico.grip}</span><div class="bx-n"><b>${e.name}</b><span>${e.eq}</span></div>${segc(["Off", "Auto"], e.prog === "auto" ? 1 : 0, "mini")}</div>${table}${prog}</div>`;
}

const weekPick = () => `<div class="wpick">${DAYS.map((d, i) => `<span class="${d.rest ? "rest" : ""}${i === 5 ? " sel" : ""}${d.str === "done" ? " did" : ""}${d.today ? " today" : ""}"><small>${d.d}</small><b class="n">${d.n}</b>${d.str === "done" ? `<i>${ico.check}</i>` : d.rest ? `<i>${ico.pause}</i>` : i === 5 ? `<i>${ico.lift}</i>` : "<i></i>"}</span>`).join("")}</div>`;

const brakeCard = () => `<div class="card"><h3>Readiness brake ${sw(true, C.rose)}</h3>
  <div class="brake"><span class="hold">Rest</span><span class="hold">Easy</span><span class="go">Steady</span><span class="go">Push</span></div>
  <div class="brk-l"><span class="hold">${ico.lock}Hold weights</span><span class="go">${ico.up}Use suggestions</span></div>
  <div class="hintline">Set from Saturday morning's readiness</div></div>`;

const stepMeter = (size = 66) => `<div class="dial stepm" style="--s:${size}px">${arc({size, value: STEPS, max: 100, color: C.rose, color2: C.rose2, stroke: size > 80 ? 10 : 6})}<div class="dial-c"><b class="n">${STEPS}</b><span>of 100</span></div></div>`;

const CATS = [["Bench Press", 39], ["Row", 41], ["Shoulder Press", 32], ["Pull Up", 37], ["Lateral Raise", 33], ["Curl", 43], ["Squat", 98], ["Deadlift", 21], ["Lunge", 82], ["Plank", 70], ["Core", 52], ["Calf Raise", 20]];
const ROWS = [["Barbell Row", "Barbell · Back"], ["One-Arm Dumbbell Row", "Dumbbell · Back", true], ["Seated Cable Row", "Cable · Back"], ["Chest-Supported Dumbbell Row", "Dumbbell · Back"], ["Inverted Row", "Bodyweight · Back"], ["T-Bar Row", "Barbell · Back"]];
const catalogBody = () => `<div class="srch">${ico.search}<span>Search 1,527 exercises</span></div>
  <div class="cats">${CATS.map(([c, n]) => `<span class="${c === "Row" ? "on" : ""}">${c}<small class="n">${n}</small></span>`).join("")}<span>+35 more</span></div>
  <div class="clist">${ROWS.map(([n, d, added]) => `<div class="cl"><div><b>${n}</b><span>${d}</span></div><span class="cl-a${added ? " added" : ""}">${added ? ico.check : ico.plus}</span></div>`).join("")}</div>
  <div class="hintline">Garmin's catalog only, so the watch can name every set</div>`;

function builder() {
  const tpl = [["Upper A", "Tue 14", true], ["Lower A", "Wed 15", true], ["Upper B", "Sat 18"], ["Lower B", "Not scheduled"]];
  return shell("rgba(255,111,181,.16)", `${back("training", "Training")}
  <header class="top"><div><div class="eyebrow">Workout · stored in Garmin</div><h1>Upper B</h1></div>
    <div class="hact">${stepMeter()}<button class="btn rose desk-only-i">${ico.upload}Save · Sat 18</button></div></header>
  <div class="cols c3">
    <div class="col desk-only">
      <div class="card"><h3>Workouts</h3>${tpl.map(([n, d, done]) => `<div class="tpl${n === "Upper B" ? " on" : ""}"><span class="act-ico sm" style="background:rgba(255,111,181,.13);color:${C.rose}">${ico.lift}</span><div><b>${n}</b><span>${done ? `${ico.check}${d}` : d}</span></div></div>`).join("")}<div class="st-add">${ico.plus}New workout</div></div>
      <div class="card"><h3>Add exercise</h3>${catalogBody()}</div>
    </div>
    <div class="col">
      ${UPPER_B.map(bxCard).join("")}
      <div class="addex tap" data-go="catalog">${ico.plus}Add exercise</div>
    </div>
    <div class="col">
      <div class="card"><h3>Schedule <span class="pill">${ico.cal}Garmin calendar</span></h3>${weekPick()}${segc(["Once", "Every Sat"], 0)}<div class="hintline">The watch picks it up on its next sync</div></div>
      ${brakeCard()}
      <div class="card desk-only"><h3>Steps</h3><div class="steps-c">${stepMeter(110)}<div><div class="lf-top"><span>Sets</span><b>23</b></div><div class="lf-top"><span>Rests</span><b>22</b></div><div class="hintline">Garmin's limit is about 100 steps</div></div></div></div>
      <div class="card desk-only"><h3>Equipment</h3><div class="eqs"><div><span>Barbell plates</span><b class="n">1.25<small>kg pair</small></b></div><div><span>Dumbbells</span><b class="n">2<small>kg steps</small></b></div><div><span>Cable stack</span><b class="n">5<small>kg steps</small></b></div></div></div>
    </div>
  </div>`, "training", `<div class="savebar fold-bar"><div class="sb-in"><div><b class="n">7 exercises · ${STEPS} steps</b><span>Saves the workout and puts it on Sat 18</span></div><button class="btn rose">${ico.upload}Save to Garmin</button></div></div>`);
}

function catalog() {
  return shell("rgba(255,111,181,.12)", `${back("builder", "Upper B")}
  <header class="top"><div><div class="eyebrow">Garmin catalog</div><h1>Add exercise</h1></div></header>
  <div class="card"><div class="recent"><h3>Recent</h3><div class="chips-l">${["Barbell Back Squat", "Lat Pulldown", "Romanian Deadlift", "Plank"].map(n => `<span class="chipb">${n}</span>`).join("")}</div></div></div>
  <div class="card">${catalogBody()}</div>`, "training");
}

/* ---------- Strength history ---------- */
const SQUAT = {w: [85, 87.5, 87.5, 90, 90, 90, 92.5, 95, 95, 97.5, 97.5, 100], r: [5, 5, 5, 5, 5, 4, 5, 5, 5, 5, 5, 5]};
const EXS = [
  ["Barbell Back Squat", "100 × 5 · 5 · 5", "Wed", SQUAT.w, true],
  ["Barbell Bench Press", "70 × 12 · 12 · 12", "Tue", [62.5, 62.5, 65, 65, 65, 67.5, 67.5, 67.5, 70, 70, 70, 70]],
  ["Romanian Deadlift", "80 × 8 · 8 · 8", "Wed", [70, 70, 72.5, 72.5, 75, 75, 75, 77.5, 77.5, 80, 80, 80]],
  ["One-Arm Dumbbell Row", "30 × 12 · 12 · 11", "Tue", [26, 26, 26, 28, 28, 28, 28, 30, 30, 30, 30, 30]],
  ["Overhead Barbell Press", "45 × 6 · 5 · 5", "Tue", [40, 40, 42.5, 42.5, 42.5, 45, 45, 45, 45, 45, 45, 45]],
  ["Lat Pulldown", "55 × 10 · 10 · 10", "Tue", [50, 50, 50, 50, 55, 55, 55, 55, 55, 55, 55, 55]],
  ["Dumbbell Walking Lunge", "16 × 12 · 12 · 12", "Wed", [12, 12, 14, 14, 14, 14, 16, 16, 16, 16, 16, 16]],
];
const MUSCLES = [["Chest", 3, 8], ["Back", 6, 11], ["Shoulders", 3, 7], ["Arms", 2, 5], ["Quads", 6, 8], ["Hamstrings", 6, 7], ["Glutes", 9, 10], ["Calves", 3, 4], ["Core", 0, 4]];

function trendChart(W, H) {
  const {w, r} = SQUAT, lo = 80, hi = 105, pl = 30, pb = 20, X = i => pl + 6 + i * (W - pl - 14) / (w.length - 1), Y = v => 8 + (hi - v) / (hi - lo) * (H - pb - 14);
  const pts = w.map((v, i) => [X(i), Y(v)]);
  let s = `<svg width="${W}" height="${H}" viewBox="0 0 ${W} ${H}" style="display:block;max-width:100%;height:auto"><defs><linearGradient id="trf${W}" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="${C.rose}" stop-opacity=".28"/><stop offset="1" stop-color="${C.rose}" stop-opacity="0"/></linearGradient></defs>`;
  [80, 90, 100].forEach(v => s += `<line x1="${pl}" x2="${W}" y1="${Y(v)}" y2="${Y(v)}" stroke="rgba(255,255,255,.06)"/><text x="0" y="${Y(v) + 4}" fill="#69728A" font-family="Outfit" font-size="11">${v}</text>`);
  const d = smooth(pts);
  s += `<path d="${d}L${X(w.length - 1)},${H - pb}L${X(0)},${H - pb}Z" fill="url(#trf${W})"/><path d="${d}" fill="none" stroke="${C.rose}" stroke-width="2.4" stroke-linecap="round"/>`;
  let best = 0;
  pts.forEach(([x, y], i) => {
    const pr = r[i] >= 5 && w[i] > best && i > 0; if (r[i] >= 5) best = Math.max(best, w[i]);
    const lastPt = i === w.length - 1;
    s += lastPt ? `<circle cx="${x}" cy="${y}" r="7" fill="${C.rose}" stroke="#0B0F1A" stroke-width="3"/>`
      : `<circle cx="${x}" cy="${y}" r="${pr ? 4.2 : 3}" fill="${pr ? C.rose : "#0B0F1A"}" stroke="${C.rose}" stroke-width="2"/>`;
    if (r[i] < 5) s += `<text x="${x}" y="${y + 18}" fill="#69728A" font-family="Outfit" font-size="10" text-anchor="middle">×${r[i]}</text>`;
  });
  [["Jul", 0], ["Sep", 5], ["Oct", 10]].forEach(([l, i]) => s += `<text x="${X(i)}" y="${H - 3}" fill="#69728A" font-family="Inter" font-size="11" text-anchor="middle">${l}</text>`);
  return s + "</svg>";
}

const muscleBars = (rows = MUSCLES) => `<div class="mus">${rows.map(([m, v, avg]) => `<div class="mu"><span>${m}</span><div class="mu-b"><i style="width:${v / 14 * 100}%"></i><u style="left:${avg / 14 * 100}%"></u></div><b class="n">${v}</b></div>`).join("")}</div>
  <div class="lgd"><span><i style="background:${C.rose}"></i>This week so far</span><span><i class="tick"></i>4-week average</span></div>`;

function heat() {
  const r = rng(11), weeks = ["4 Aug", "", "", "", "1 Sep", "", "", "", "29 Sep", "", "", "Now"];
  let s = `<div class="heat"><span></span>${weeks.map(w => `<small>${w}</small>`).join("")}`;
  MUSCLES.forEach(([m, v, avg]) => {
    s += `<span>${m}</span>` + weeks.map((_, i) => { const x = i === 11 ? v : i === 5 ? Math.round(avg * .3) : Math.max(0, Math.round(avg + (r() - .5) * 5)); return `<i class="n" style="background:${hexA(C.rose, .08 + x / 14 * .8)}">${x}</i>`; }).join("");
  });
  return s + "</div>";
}

function strengthHistory() {
  const recs = [[1, "—", ""], [3, "105", "Aug"], [5, "100", "Wed", true], [8, "90", "Sep"], [10, "85", "Aug"]];
  const sessions = [["Wed 15 Oct", "100", "5 · 5 · 5", true], ["Wed 8 Oct", "97.5", "5 · 5 · 5"], ["Wed 1 Oct", "97.5", "5 · 5 · 5"], ["Wed 24 Sep", "95", "5 · 5 · 5"]];
  return shell("rgba(255,111,181,.16)", `${back("training", "Training")}
  <header class="top"><div><div class="eyebrow">Last 12 weeks</div><h1>Strength</h1></div><button class="btn rose sm tap" data-go="builder">${ico.lift}Workouts</button></header>
  <div class="cols c3">
    <div class="col desk-only">
      <div class="card"><h3>Exercises</h3><div class="srch">${ico.search}<span>Find an exercise</span></div>
        ${EXS.map(([n, last, d, tr, on]) => `<div class="exl${on ? " on" : ""}"><div><b>${n}</b><span class="n">${last}</span></div>${spark(tr, {w: 64, h: 26, color: C.rose, fill: false, sw: 1.8})}</div>`).join("")}</div>
    </div>
    <div class="col">
      <div class="exchips">${EXS.slice(0, 4).map(([n], i) => `<span class="${i === 0 ? "on" : ""}">${n.replace("Barbell ", "")}</span>`).join("")}</div>
      <div class="card"><h3>Barbell Back Squat <span class="pill rose">${ico.star}New 5-rep best</span></h3>
        <div class="tr-top"><div class="big n">100<span class="unit">kg × 5</span></div><span class="dim">Heaviest set each session</span></div>
        <div class="narrow-only">${trendChart(326, 150)}</div><div class="wide-only">${trendChart(440, 200)}</div></div>
      <div class="card"><h3>Best at each rep count</h3><div class="recs">${recs.map(([n, v, d, nw]) => `<div class="rec${nw ? " new" : ""}"><span>${n} rep${n > 1 ? "s" : ""}</span><b class="n">${v}</b><small>${nw ? ico.star : ""}${d}</small></div>`).join("")}</div></div>
      <div class="card"><h3>Sessions</h3>${sessions.map(([d, w, rr, pr]) => `<div class="ses"><span>${d}</span><b class="n">${w}<small>kg</small></b><span class="n rr">${rr}</span><span class="pr">${pr ? ico.star : ""}</span></div>`).join("")}</div>
    </div>
    <div class="col">
      <div class="card"><h3>Sets per muscle <span class="pill">Mon – Thu</span></h3>${muscleBars()}</div>
      <div class="card wide-only-b"><h3>Weekly sets · 12 weeks</h3>${heat()}</div>
    </div>
  </div>`, "training");
}

function strengthCard() {
  return `<div class="card tap" data-go="strength-history"><h3>Strength · this week <span class="pill rose">${ico.star}1 record</span></h3>
    <div class="st-sum"><div><div class="big n" style="color:${C.rose}">29</div><div class="small" style="color:#A3ACBF">sets · 2 sessions</div></div>
    <div class="mus mini">${MUSCLES.slice(4, 7).concat([MUSCLES[1]]).map(([m, v, avg]) => `<div class="mu"><span>${m}</span><div class="mu-b"><i style="width:${v / 14 * 100}%"></i><u style="left:${avg / 14 * 100}%"></u></div><b class="n">${v}</b></div>`).join("")}</div></div></div>`;
}

/* ---------- Home-screen widget ---------- */
function widget() {
  const icon = (c, k) => `<div class="happ"><i style="background:${c}">${k || ""}</i><span>${k ? "Raring" : ""}</span></div>`;
  const raring = `<img src="../../site/assets/icon.svg" width="54" height="54" alt="">`;
  return `${statusBar()}<div class="home">
    <div class="hclock"><b class="n">7:42</b><span>Thursday 16 October</span></div>
    <div class="wg">
      <div class="wd med"><div class="dial">${arc({size: 118, value: 82, color: C.mint, color2: "#2BB6D9", stroke: 11})}<div class="dial-c"><b class="n" style="font-size:38px">82</b><em style="color:${C.mint}">High</em></div></div>
        <div class="wd-r"><b>Ready to push</b><div class="wd-sl"><span class="moon">${ico.moon}</span><div><span>Asleep by</span><b class="n">22:50</b></div></div><small>Updated 06:58</small></div></div>
      <div class="wd sm"><div class="dial">${arc({size: 96, value: 82, color: C.mint, color2: "#2BB6D9", stroke: 9})}<div class="dial-c"><b class="n" style="font-size:30px">82</b><em style="color:${C.mint}">High</em></div></div><div class="wd-b"><span style="color:#B9AEFF">${ico.moon}</span><b class="n">22:50</b></div></div>
      <div class="wd sm eve"><div class="wd-moon">${ico.moon}</div><b class="n">22:50</b><span>Asleep by</span><div class="wd-b"><i style="background:${C.mint}"></i><b class="n">82</b><small>today</small></div></div>
      <div class="wd sm wait"><div class="dial">${arc({size: 96, value: 0, color: "transparent", stroke: 9})}<div class="dial-c"><b class="n" style="font-size:26px;color:#69728A">—</b></div></div><div class="wd-b"><span style="color:#69728A">${ico.sync}</span><small>Waiting for Garmin</small></div></div>
      <div class="apps4">${icon("#2A3350")}${icon("#33304A")}<div class="happ">${raring}<span>Raring</span></div>${icon("#283A44")}</div>
    </div>
    <div class="dock"><div class="dk">${["#2B3350", "#33304A", "#283A44", "#3A2F3A"].map(c => `<i style="background:${c}"></i>`).join("")}</div><div class="hsearch">${ico.search}</div></div>
  </div>`;
}

SCREENS.push(
  ["plan", "Weekly plan", plan, "phone", "Weekly plan"],
  ["plan", "Weekly plan", plan, "fold", "Weekly plan"],
  ["plan-setup", "Plan targets", planSetup, "phone", "Weekly plan"],
  ["checkin", "Monday check-in", checkin, "phone", "Weekly plan"],
  ["plan-setup", "Plan targets", planSetup, "desk", "Weekly plan"],
  ["strength-workout", "Strength workout", strengthWorkout, "phone", "Strength"],
  ["fix", "Fix sets", fixSets, "phone", "Strength"],
  ["strength-history", "Strength history", strengthHistory, "phone", "Strength"],
  ["strength-workout", "Strength workout", strengthWorkout, "fold", "Strength"],
  ["strength-history", "Strength history", strengthHistory, "desk", "Strength"],
  ["builder", "Workout builder", builder, "phone", "Workout builder"],
  ["catalog", "Add exercise", catalog, "phone", "Workout builder"],
  ["builder", "Workout builder", builder, "desk", "Workout builder"],
  ["widget", "Home-screen widgets", widget, "phone", "Widget"],
);
