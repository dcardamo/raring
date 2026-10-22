/* Tonight's plan as a chart: the shipped ring, and three other ways to draw the same night, so they can
   be judged side by side. Loaded after index.html's inline script and time-of-day.js, and reuses their
   helpers.

   One synthetic night in every frame: now 21:40, wind down 22:40, Garmin's window 23:10–23:30, in bed
   23:20, 15 min to fall asleep, so asleep 23:35, need 7h 00m, up 06:35. */

const TN = {
  now: 21 * 60 + 40, windDown: 22 * 60 + 40, win: [23 * 60 + 10, 23 * 60 + 30],
  inBed: 23 * 60 + 20, asleep: 23 * 60 + 35, wake: 6 * 60 + 35, need: "7h 00m",
};
// Minutes after the evening's midnight, so 06:35 sorts after 23:35.
const tn24 = m => m < 12 * 60 ? m + 24 * 60 : m;

// A clock time as an angle, 0 at the top, clockwise.
const tnAng = (m, from, to, sweep = 360) => (tn24(m) - tn24(from)) / (tn24(to) - tn24(from)) * sweep;
// An arc between two clock times, clockwise from a to b, on a ring of radius r.
function tnArc(r, a, b, {stroke = 8, color = "#5C9BFF", opacity = 1, sweep = 360, cap = "round", from = 0, to = 24 * 60} = {}) {
  const A = m => tnAng(m, from, to, sweep), a1 = A(a);
  let a2 = A(b);
  if (a2 <= a1) a2 += sweep;
  const p = d => [50 + r * Math.sin(d * Math.PI / 180), 50 - r * Math.cos(d * Math.PI / 180)];
  const [x1, y1] = p(a1), [x2, y2] = p(a2), large = a2 - a1 > 180 ? 1 : 0;
  return `<path d="M${x1.toFixed(2)},${y1.toFixed(2)} A${r},${r} 0 ${large} 1 ${x2.toFixed(2)},${y2.toFixed(2)}" fill="none"
    stroke="${color}" stroke-opacity="${opacity}" stroke-width="${stroke}" stroke-linecap="${cap}"/>`;
}
const TNY = {from: 21 * 60, to: 7 * 60 + 30};
const tnp = m => (tn24(m) - tn24(TNY.from)) / (tn24(TNY.to) - tn24(TNY.from)) * 100;
const tnSeg = (a, b) => `left:${tnp(a).toFixed(2)}%;width:${(tnp(b) - tnp(a)).toFixed(2)}%`;

/* ---------- B: the evening, zoomed ---------- */
// The axis is the evening, not the night: an hour before wind-down (or now, if that is earlier) to half an
// hour past Garmin's window. That is where the decisions are, and a 40-minute wind-down gets 60px instead
// of 10. The night's length is one fact above the axis, not the scale of it. A position, never a countdown.
const EVE_TO = 23 * 60 + 50;
function evening(now, clock, live = false) {
  const from = Math.floor((Math.min(now, TN.windDown) - 30) / 30) * 30;
  const p = m => (tn24(m) - tn24(from)) / (tn24(EVE_TO) - tn24(from)) * 100;
  const seg = (a, b) => `left:${p(a).toFixed(2)}%;width:${(p(b) - p(a)).toFixed(2)}%`;
  const ticks = [];
  for (let h = Math.ceil(from / 60); h * 60 <= EVE_TO; h++) ticks.push(h % 24);
  return `<div class="tn-strip">
    <div class="tn-labels"><span style="left:${p(TN.windDown).toFixed(2)}%">${live ? "Wind down now" : `Wind down <b class="n">22:40</b>`}</span></div>
    <div class="tn-track">
      <i class="wd" style="${seg(TN.windDown, TN.inBed)}"></i>
      <i class="fa" style="${seg(TN.inBed, TN.asleep)}"></i>
      <u class="now" style="left:${p(now).toFixed(2)}%">${live ? "" : `<b>Now ${clock}</b>`}</u>
    </div>
    <div class="tn-ticks">${ticks.map(h => `<span style="left:${p(h * 60).toFixed(2)}%">${String(h).padStart(2, "0")}</span>`).join("")}</div>
    <div class="tn-win"><i style="${seg(TN.win[0], TN.win[1])}"></i><span style="right:${(100 - p(TN.win[0])).toFixed(2)}%">Garmin <b class="n">23:10–23:30</b></span></div>
  </div>`;
}

/* ---------- C: the ring, scoped to the night ---------- */
// Midnight still at the top, but the circle is the night only: every segment is as wide as its minutes
// and the day is the gap at the bottom. 22:10 to 07:05 over 300°, so the seam falls in the gap.
const RING = {from: 22 * 60 + 10, to: 7 * 60 + 5};
function nightRing() {
  const opt = {sweep: 300, from: RING.from, to: RING.to}, A = m => tnAng(m, RING.from, RING.to, 300) - tnAng(24 * 60, RING.from, RING.to, 300);
  const at = m => ((A(m) % 360) + 360) % 360, R = 34;
  const hours = [22, 24, 26, 28, 30].map(h => {
    const a = at(h % 24 * 60) * Math.PI / 180;
    return `<span class="rt" style="left:${(50 + (R + 12) * Math.sin(a)).toFixed(1)}%;top:${(50 - (R + 12) * Math.cos(a)).toFixed(1)}%">${String(h % 24).padStart(2, "0")}</span>`;
  }).join("");
  const arc = (a, b, o) => {
    let a1 = at(a), a2 = at(b);
    if (a2 <= a1) a2 += 300;
    const p = d => [50 + o.r * Math.sin(d * Math.PI / 180), 50 - o.r * Math.cos(d * Math.PI / 180)];
    const [x1, y1] = p(a1), [x2, y2] = p(a2);
    return `<path d="M${x1.toFixed(2)},${y1.toFixed(2)} A${o.r},${o.r} 0 ${a2 - a1 > 180 ? 1 : 0} 1 ${x2.toFixed(2)},${y2.toFixed(2)}" fill="none"
      stroke="${o.color}" stroke-opacity="${o.opacity ?? 1}" stroke-width="${o.stroke}" stroke-linecap="round"/>`;
  };
  const nowA = at(TN.now) * Math.PI / 180;
  return `<div class="tn-ring">
    <svg viewBox="0 0 100 100" aria-hidden="true">
      ${arc(RING.from, RING.to, {r: R, stroke: 8, color: "rgba(255,255,255,.07)"})}
      ${arc(TN.windDown, TN.inBed, {r: R, stroke: 5, color: C.fair, opacity: .85})}
      ${arc(TN.inBed, TN.asleep, {r: R, stroke: 8, color: C.stage.light, opacity: .45})}
      ${arc(TN.asleep, TN.wake, {r: R, stroke: 8, color: C.stage.light})}
      ${arc(TN.win[0], TN.win[1], {r: R + 6.5, stroke: 2.5, color: C.ink})}
      <circle class="nowdot" cx="${(50 + R * Math.sin(nowA)).toFixed(2)}" cy="${(50 - R * Math.cos(nowA)).toFixed(2)}" r="3.6"/>
    </svg>${hours}
    <div class="tn-centre"><b class="n">23:20</b><span>In bed by</span><small>asleep 23:35</small></div>
  </div>`;
}

/* ---------- The cards ---------- */
const tnPill = `<span class="pill">${ico.moon}Garmin plan</span>`;
const tnFoot = `<div class="foot"><span>Tonight’s plan</span>${ico.chev}</div>`;
const tnNote = `Raring’s wind-down is 30 min before Garmin’s window. Garmin took 20 min off for a falling HRV trend.`;

// Shipped: the 24-hour ring, midnight at the top, a third of the circle spent on the day either side.
function cardShipped() {
  const A = m => tn24(m) / 1440 * 360, R = 34;
  const hours = [0, 6, 12, 18].map(h => `<span class="tk t${h}">${String(h).padStart(2, "0")}</span>`).join("");
  const arc = (a, b, o) => {
    const p = d => [50 + o.r * Math.sin(d * Math.PI / 180), 50 - o.r * Math.cos(d * Math.PI / 180)];
    const a1 = A(a), a2 = A(b) > A(a) ? A(b) : A(b) + 360, [x1, y1] = p(a1), [x2, y2] = p(a2);
    return `<path d="M${x1.toFixed(2)},${y1.toFixed(2)} A${o.r},${o.r} 0 ${a2 - a1 > 180 ? 1 : 0} 1 ${x2.toFixed(2)},${y2.toFixed(2)}" fill="none"
      stroke="${o.color}" stroke-opacity="${o.opacity ?? 1}" stroke-width="${o.stroke}" stroke-linecap="round"/>`;
  };
  const nowA = A(TN.now) * Math.PI / 180;
  return `<div class="card lead"><h3>Tonight’s sleep ${tnPill}</h3>
    <div class="tn-row"><div class="tn-shipped">
      <svg viewBox="0 0 100 100" aria-hidden="true">
        <circle cx="50" cy="50" r="${R}" fill="none" stroke="rgba(255,255,255,.07)" stroke-width="8"/>
        ${arc(TN.windDown, TN.inBed, {r: R, stroke: 5, color: C.fair, opacity: .85})}
        ${arc(TN.inBed, TN.asleep, {r: R, stroke: 8, color: C.stage.light, opacity: .45})}
        ${arc(TN.asleep, TN.wake, {r: R, stroke: 8, color: C.stage.light})}
        ${arc(TN.win[0], TN.win[1], {r: R + 6.5, stroke: 2.5, color: C.ink})}
        <circle class="nowdot" cx="${(50 + R * Math.sin(nowA)).toFixed(2)}" cy="${(50 - R * Math.cos(nowA)).toFixed(2)}" r="3.6"/>
      </svg>${hours}
      <div class="tn-centre"><b class="n">7:00</b></div>
    </div>
    <dl class="tn-times">
      <div><dt>Wind down</dt><dd class="n">22:40</dd></div>
      <div><dt>In bed by</dt><dd class="n">23:20</dd></div>
      <div><dt>Garmin window</dt><dd class="n">23:10–23:30</dd></div>
    </dl></div>
    <p class="note">Garmin took off 20 min for decreasing.</p>${tnFoot}</div>`;
}

// A: the three times, and the reasons in words. Nothing to misread, nothing to read either.
function cardPlain() {
  return `<div class="card lead"><h3>Tonight’s sleep ${tnPill}</h3>
    <div class="pn"><div><span>Wind down</span><b class="n">22:40</b></div>
      <div class="c"><span>In bed by</span><b class="n">23:20</b></div>
      <div class="r"><span>Up</span><b class="n">06:35</b></div></div>
    <p class="note">Garmin’s window 23:10–23:30 · need ${TN.need} asleep. ${tnNote}</p>
    ${tnFoot}</div>`;
}

// B: the evening zoomed. The night's length is said once, above the axis.
function cardEvening(now = TN.now, clock = "21:40", live = false) {
  const head = live
    ? `<div><span class="tn-k">Bedtime window</span><b class="n tn-big">23:10–23:30</b></div>
       <span class="tn-sub"><span>in bed 23:20 · asleep 23:35</span><span>need <b class="n">${TN.need}</b> asleep</span></span>`
    : `<div><span class="tn-k">In bed by</span><b class="n tn-big">23:20</b></div>
       <span class="tn-sub"><span>asleep 23:35 · up 06:35</span><span>need <b class="n">${TN.need}</b> asleep</span></span>`;
  return `<div class="card lead"><h3>Tonight’s sleep ${tnPill}</h3>
    <div class="tn-top">${head}</div>
    ${evening(now, clock, live)}
    <p class="note">${tnNote}</p>${tnFoot}</div>`;
}

// C: the ring, scoped to the night.
function cardRing() {
  return `<div class="card lead"><h3>Tonight’s sleep ${tnPill}</h3>
    ${nightRing()}
    <dl class="tn-times three">
      <div><dt>Wind down</dt><dd class="n">22:40</dd></div>
      <div><dt>Garmin window</dt><dd class="n">23:10–23:30</dd></div>
      <div><dt>Up</dt><dd class="n">06:35</dd></div>
    </dl>
    <p class="note">${tnNote}</p>${tnFoot}</div>`;
}

// The same rows compact on Today: the card it replaces is the ring plus three facts.
function todayWith(card) {
  return todHero({headline: "Make room for sleep"}) + card;
}

/* ---------- Screens ---------- */
SCREENS.push(
  ["tn-now", "Before · 24-hour ring", () => shell(todayWith(cardShipped()), "today", {glow: "rgba(61,220,151,.12)"}), "phone", "Tonight’s chart"],
  ["tn-plain", "A · the times, no chart", () => shell(todayWith(cardPlain()), "today", {glow: "rgba(61,220,151,.12)"}), "phone", "Tonight’s chart"],
  ["tn-evening", "Shipped · the evening, zoomed", () => shell(todayWith(cardEvening()), "today", {glow: "rgba(61,220,151,.12)"}), "phone", "Tonight’s chart"],
  ["tn-ring", "C · the ring, scoped to the night", () => shell(todayWith(cardRing()), "today", {glow: "rgba(61,220,151,.12)"}), "phone", "Tonight’s chart"],
  ["tn-sleep", "B · on Sleep · Tonight", tnHooked, "phone", "Tonight’s chart"],
  ["tn-wind", "B · at wind-down, 22:50", () => shell(todayWith(cardEvening(22 * 60 + 50, "22:50", true)), "today", {glow: "rgba(61,220,151,.12)"}), "phone", "Tonight’s chart"],
  ["tn-rules", "Tonight’s chart · the options", tnRules, "phone", "Tonight’s chart"],
);

// B shown where it would live: the hero of Sleep · Tonight.
function tnHooked() {
  return shell(`<header class="top"><div><div class="eyebrow">Thu 16 Oct</div><h1>Sleep</h1></div></header>
    <div class="row"><button class="pill" style="border:0">Last night</button><button class="pill ok" style="border:0">Tonight</button></div>
    ${cardEvening()}
    <div class="card"><h3>Recent nights vs tonight’s window</h3>
      <p class="note" style="margin-top:0">The last 14 nights from bedtime to wake, on the same axis tonight’s plan uses.</p></div>`,
    "sleep");
}

function tnRules() {
  const rows = [
    ["0", "Before · 24 hours", "A third of the circle is day either side. Wind-down, in bed and Garmin’s window land 10–17° apart, so only the list under it can separate them. The centre shows the need, which is the same every night."],
    ["A", "No chart", "Three times and a sentence. Nothing to misread, and no sense of how long the night is or where now sits."],
    ["B", "The evening, zoomed · shipped", "The axis is the evening, from an hour before wind-down to half an hour past the window, so a 40-minute wind-down gets 60px instead of 10. Wind-down, the minutes to fall asleep and Garmin’s window are all on it; the night’s length is text above. A position, never a countdown."],
    ["C", "The ring, scoped to the night", "Midnight still at the top, but the circle is the night only: every segment is as wide as its minutes and the day is the gap. Keeps Raring’s clock, at the cost of the 24-hour dial and of room for the times."],
  ];
  return shell(`<header class="top"><div><div class="eyebrow">Sleep · Tonight</div><h1>One night, four charts</h1></div></header>
    <div class="card"><div class="phases plain">${rows.map(([k, n, why]) => `<div class="ph"><b class="n">${k}</b><i></i><div><strong>${n}</strong><small>${why}</small></div></div>`).join("")}</div></div>
    <div class="card"><h3>Same in every frame</h3><p class="note" style="margin-top:0">Now 21:40. Wind down 22:40, Garmin’s window 23:10–23:30, in bed 23:20, 15 min to fall asleep, asleep 23:35, need 7h 00m, up 06:35.</p></div>
    <div class="card"><h3>What each one is asked to answer</h3><p class="note" style="margin-top:0">When do I stop? When am I in bed? Is that soon? How long is the night? Why tonight and not another? The shipped ring answers the last two. B answers all six.</p></div>`,
    "today");
}
