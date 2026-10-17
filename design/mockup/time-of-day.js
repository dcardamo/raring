/* Today through the day: one leading card that follows the person's own sleep and wake times, with the
   hero (scores, freshness, chips) and the metric cards held still. Runs after index.html's inline script
   and plan-strength.js, and reuses their helpers. The same synthetic day at different clock times:
   woke 06:39, Garmin's bedtime window 22:30–23:06, wind-down lead 30 min, so tonight's plan rises at
   20:30 (two hours before the window), winds down at 22:00 and is past at 23:06. */

Object.assign(ico, {
  battery: svgI('<rect x="7" y="5" width="10" height="16" rx="2.5"/><path d="M10 3h4"/>'),
  sun: svgI('<circle cx="12" cy="12" r="4"/><path d="M12 2.5v2M12 19.5v2M2.5 12h2M19.5 12h2M5.3 5.3l1.4 1.4M17.3 17.3l1.4 1.4M5.3 18.7l1.4-1.4M17.3 6.7l1.4-1.4"/>'),
});

const TOD = {
  date: "Thu 16 Oct", window: ["22:30", "23:06"], windDown: "22:00", need: "7h 50m",
  // Minutes from the evening's midnight for the strip: 20:00 to 00:00.
  strip: {from: 20 * 60, to: 24 * 60, windDown: 22 * 60, start: 22 * 60 + 30, end: 23 * 60 + 6},
};

// The hero never changes shape. Before last night syncs it still shows the previous day, dated.
const todHero = ({headline, fresh = "Watch synced 12 min ago"}) => `
  <header class="top"><div><div class="eyebrow">${TOD.date}</div><h1>Today</h1></div><div class="avatar">D</div></header>
  <div class="fresh">${fresh}<span class="fresh-btn">Refresh</span></div>
  <section class="dials">
    <div class="dial sm">${arc({size: 100, value: 84, color: C.good, stroke: 9})}<div class="dial-c"><b class="n">84</b><span>Sleep</span></div></div>
    <div class="dial lg">${arc({size: 156, value: 82, color: C.good, stroke: 13})}<div class="dial-c"><b class="n">82</b><span>Readiness</span><em style="color:${C.good}">High</em></div></div>
    <div class="dial sm">${arc({size: 100, value: 612, max: 1000, color: C.good, stroke: 9, band: [420, 760], marker: true})}<div class="dial-c"><b class="n">612</b><span>Load</span></div></div>
  </section>
  <div class="headline"><h2>${headline}</h2>
    <div class="chips"><span class="chip">${ico.moon}<b class="n">7:48</b></span><span class="chip">${ico.hrv}<b class="n">58</b>ms</span><span class="chip">${ico.flame}Room to build</span></div></div>`;

// Tonight's plan, folded to one line until it leads. The label matches Sleep and the reminder.
const planRow = () => `<div class="card plan-row tap" data-go="sleep"><span class="pr-i">${ico.moon}</span>
  <div class="pr-m"><b>Tonight <span>· Garmin plan</span></b><span>Wind down <b class="n">${TOD.windDown}</b> · bed <b class="n">${TOD.window.join("–")}</b></span></div>
  <span class="chev">${ico.chev}</span></div>`;

// Where now sits against wind-down and the window: a position, never a countdown.
function nightStrip(now) {
  const {from, to, windDown, start, end} = TOD.strip, P = m => (m - from) / (to - from) * 100;
  const ticks = [20, 21, 22, 23, 24].map(h => `<span style="left:${P(h * 60)}%">${String(h % 24).padStart(2, "0")}</span>`).join("");
  return `<div class="strip"><div class="strip-bar">
      <i class="st-wd" style="left:${P(windDown)}%;width:${P(start) - P(windDown)}%"></i>
      <i class="st-bw" style="left:${P(start)}%;width:${P(end) - P(start)}%"></i>
      <u style="left:${P(now)}%"></u></div><div class="strip-ticks">${ticks}</div>
    <div class="strip-key"><span><i class="st-wd"></i>Wind down</span><span><i class="st-bw"></i>Bedtime window</span><span><i class="st-now"></i>Now</span></div></div>`;
}

const planNumbers = () => `<div class="pn"><div><span>Wind down</span><b class="n">${TOD.windDown}</b></div>
  <div class="c"><span>Bedtime</span><b class="n">${TOD.window.join("–")}</b></div><div class="r"><span>Sleep need</span><b class="n">${TOD.need}</b></div></div>`;

function planLead(state, now) {
  const title = {evening: "Tonight's sleep", winddown: "Wind down", late: "Tonight"}[state];
  const body = {
    evening: `${planNumbers()}${nightStrip(now)}<p class="note">Garmin added 20 min for today's activity and load.</p>`,
    winddown: `<div class="calm"><span>Bedtime window</span><b class="n">${TOD.window.join("–")}</b><small>Sleep need <b class="n">${TOD.need}</b></small></div>${nightStrip(now)}<p class="note">A suggestion, not an alarm.</p>`,
    late: `<div class="calm"><span>Tonight's window was</span><b class="n">${TOD.window.join("–")}</b></div><p class="note">Rest when you're ready. Tomorrow's plan follows your next sleep.</p>`,
  }[state];
  return `<div class="card lead tap" data-go="sleep"><h3>${title} <span class="pill">${ico.moon}Garmin plan</span></h3>${body}
    <div class="foot"><span>Timing history</span>${ico.chev}</div></div>`;
}

// Today's focus. Mornings lead with last night, then the limiting factor, then the effort.
function focusCard({morning = false, effort = true, missing = false, lead = false} = {}) {
  const night = morning ? (missing
    ? `<div class="fr"><span class="fr-i dimmed">${ico.moon}</span><div><span>Last night</span><b>Not synced yet</b></div><small class="dim">Watch synced 23:10</small></div>
       <p class="note">Your sleep and readiness appear after the watch uploads to Garmin Connect.</p>`
    : `<div class="fr"><span class="fr-i" style="color:${C.good}">${ico.moon}</span><div><span>Last night · woke 06:39</span><b><span class="n">7:48</span> asleep</b></div><small>need <b class="n">7:50</b></small></div>`) : "";
  const factor = missing ? "" : `<div class="fr"><span class="fr-i" style="color:${C.fair}">${ico.gauge}</span><div><span>Lowest readiness factor</span><b>Acute load</b></div><b class="n big2" style="color:${C.fair}">58<small>/100</small></b></div>`;
  const seg = effort && !missing ? `<div class="seg">${["Rest", "Easy", "Steady", "Push"].map(e => `<span${e === "Push" ? ' class="on"' : ""}>${e}</span>`).join("")}</div>` : "";
  return `<div class="card${lead ? " lead" : ""} tap"><h3>${morning ? "This morning" : "Today's focus"} <span class="h-note">From Garmin readiness</span></h3>
    ${night}${morning ? factor + seg : seg + factor}
    ${missing ? "" : `<div class="foot"><span>See all readiness factors</span>${ico.chev}</div>`}</div>`;
}

// During the day: the latest synced Body Battery and stress, and what the day has held. Never "live".
const nowCard = ({activity = true} = {}) => `<div class="card lead"><h3>Right now <span class="h-note">As of watch sync 14:05</span></h3>
  <div class="now-grid"><div class="now-t tap"><div class="batt sm"><i style="height:54%;background:${C.fair}"></i></div><div><b class="n" style="color:${C.fair}">54</b><span>Body Battery</span></div></div>
  <div class="now-t tap"><div class="ring-s" style="color:${C.good}">${ico.pulse}</div><div><b class="n">31</b><span>Stress · low</span></div></div></div>
  ${activity ? `<div class="fr ctx"><span class="fr-i">${ico.run}</span><div><span>Done today</span><b>Lunch Run · <span class="n">142</b> load</b></div></div>`
    : `<div class="fr ctx"><span class="fr-i">${ico.clock}</span><div><span>Garmin's recovery time</span><b><span class="n">18</span> h</b></div></div>`}</div>`;

const restCards = () => `
  <div class="row">
    <div class="card"><h3>Body Battery</h3><div class="bb"><div class="batt"><i style="height:54%;background:${C.fair}"></i></div><div><div class="big n">54</div><div class="small"><b>+62</b> charged</div></div></div></div>
    <div class="card"><h3>HRV <span class="pill ok">Balanced</span></h3><div class="big n">58<span class="unit">ms</span></div><div style="margin-top:10px">${spark([51, 49, 53, 55, 52, 56, 58], {w: 138, h: 34, color: C.good, band: [48, 62], min: 42, max: 64})}</div></div>
  </div>
  <div class="card"><h3>Stress <span class="pill ok">Balanced</span></h3><div class="big n">31</div><div class="small">Low · average</div></div>
  <div class="card"><h3>Vitals <span class="pill ok">${ico.check}All normal</span></h3><div class="vitals">${[["RHR", "46", ico.heart], ["HRV", "58", ico.hrv], ["Resp", "14.1", ico.lungs], ["SpO₂", "96", ico.drop], ["Skin", "+0.1°", ico.temp]].map(([n, v, i]) => `<div class="v"><div class="ring">${i}</div><b>${v}</b><span>${n}</span></div>`).join("")}</div></div>`;

function todScreen({time, hero, cards, note = "", tab = "today"}) {
  return `<div class="glow" style="--glow:rgba(61,220,151,.11)"></div>${statusBar().replace("7:42", time)}
  <div class="scroll"><div class="page tod">${todHero(hero)}${note}<div class="tod-cards">${cards}${restCards()}</div></div></div>${tabs(tab)}`;
}

const todWake = () => todScreen({time: "7:10", hero: {headline: "Ready to push"}, cards: focusCard({morning: true, lead: true}) + planRow()});
const todDay = () => todScreen({time: "14:20", hero: {headline: "Ready to push", fresh: "Watch synced 15 min ago"}, cards: nowCard() + focusCard() + planRow()});
const todEvening = () => todScreen({time: "20:45", hero: {headline: "Wind down at 22:00", fresh: "Watch synced 20 min ago"}, cards: planLead("evening", 20 * 60 + 45) + focusCard()});
const todWindDown = () => todScreen({time: "22:10", hero: {headline: "Time to wind down", fresh: "Watch synced 35 min ago"}, cards: planLead("winddown", 22 * 60 + 10) + focusCard({effort: false})});
const todLate = () => todScreen({time: "23:40", hero: {headline: "Rest when you're ready", fresh: "Watch synced 1 h ago"}, cards: planLead("late") + focusCard({effort: false})});
const todMissing = () => todScreen({time: "7:30", hero: {headline: "Waiting for last night's sleep", fresh: "Watch synced 8 h ago"}, cards: focusCard({morning: true, missing: true, lead: true})});
const todOverview = () => todScreen({time: "14:20", hero: {headline: "Ready to push"},
  note: `<p class="phase-note">${ico.clock}Fixed layout: your wake times vary too much to plan the day around.</p>`,
  cards: focusCard() + `<div class="card"><h3>Tonight's sleep <span class="pill">${ico.moon}Raring estimate</span></h3>${planNumbers().replace(TOD.window.join("–"), "23:10").replace("Bedtime", "Asleep by")}<p class="note">Wake reference 07:00 (median of 9 recorded wake times) minus your latest need.</p></div>`});

// The rule on one page: phases follow your own wake time and tonight's plan, not the clock.
function todRules() {
  const rows = [
    ["06:39", "After waking", "Recorded wake (or usual wake, if last night hasn't synced)", "Last night, the limiting factor, then the effort"],
    ["09:39", "During the day", "Three hours after waking", "Body Battery, stress and what the day held, as of the watch's last upload"],
    ["20:30", "Tonight's plan rises", "Two hours before Garmin's bedtime window", "Wind-down, window and need lead"],
    ["22:00", "Wind down", "The plan's wind-down time (lead from Settings)", "A calm card: the window, no effort call"],
    ["23:06", "Late night", "The window has ended", "No countdown, no effort call, no assumption you're asleep"],
  ];
  return shell(`<header class="top"><div><div class="eyebrow">Rules</div><h1>Today through the day</h1></div></header>
  <div class="card"><div class="phases">${rows.map(([t, n, from, what]) => `<div class="ph"><b class="n">${t}</b><i></i><div><strong>${n}</strong><span>${from}</span><small>${what}</small></div></div>`).join("")}</div></div>
  <div class="card"><h3>Always the same</h3><p class="note" style="margin-top:0">The scores, freshness line, chips, vitals and metric cards never move. Watch and vitals notices stay above the leading card. Tabs never switch on their own, and a change waits until your finger lifts.</p></div>
  <div class="card"><h3>Fixed layout instead</h3><p class="note" style="margin-top:0">When it's turned off in Settings (shift work), with fewer than four recorded wake times in two weeks, wake times spread over three hours, a new time zone, or Garmin's jet-lag rating.</p></div>`, "today");
}

SCREENS.push(
  ["tod-rules", "Today through the day", todRules, "phone", "Time of day"],
  ["tod-wake", "07:10 · After waking", todWake, "phone", "Time of day"],
  ["tod-day", "14:20 · During the day", todDay, "phone", "Time of day"],
  ["tod-evening", "20:45 · Tonight's plan rises", todEvening, "phone", "Time of day"],
  ["tod-winddown", "22:10 · Wind down", todWindDown, "phone", "Time of day"],
  ["tod-late", "23:40 · Late night", todLate, "phone", "Time of day"],
  ["tod-missing", "07:30 · Sleep not synced", todMissing, "phone", "Time of day"],
  ["tod-overview", "Irregular schedule", todOverview, "phone", "Time of day"],
  ["tod-evening", "20:45 · Tonight's plan rises", todEvening, "fold", "Time of day"],
  ["tod-day", "14:20 · During the day", todDay, "desk", "Time of day"],
);
