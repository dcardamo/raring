/* Long-term trends: ranges up to a year and All, weekly or monthly averages, a second metric overlaid,
   and a period against the one before. Runs after index.html and plan-strength.js and reuses their
   helpers (C, ico, svgI, rng, statusBar, tabs, shell, back, segc). All data is synthetic. */

Object.assign(ico, {
  left: svgI('<path d="M15 5l-7 7 7 7"/>'),
  right: svgI('<path d="M9 5l7 7-7 7"/>'),
  caret: svgI('<path d="M7 10l5 5 5-5"/>'),
});

const RANGES = ["7D", "30D", "90D", "1Y", "All", "Dates"];
const rangeBar = on => `<div class="ranges" role="group">${RANGES.map(r => `<span${r === on ? ' class="on"' : ""}>${r}</span>`).join("")}</div>`;
const readinessLevel = v => v >= 75 ? C.good : v >= 50 ? C.fair : C.poor;
const hm = m => `${Math.floor(m / 60)}:${String(Math.round(m % 60)).padStart(2, "0")}`;

// A seeded walk around a base with a slow seasonal swing; null marks a missing reading.
function series(n, {seed, base, swing, noise, gaps = [], round = 0}) {
  const r = rng(seed), f = 10 ** round;
  return Array.from({length: n}, (_, i) => gaps.some(([a, b]) => i >= a && i <= b) ? null
    : Math.round((base + Math.sin(i / n * Math.PI * 2.4) * swing + (r() - .5) * noise) * f) / f);
}

/* One chart for every trend. The primary series is ink (bars or a line); only daily readiness bars take
   Garmin's level colors. An overlaid metric gets the secondary ink and its own right-hand axis; the
   previous period is a faint dashed line on the same axis. Missing buckets are gaps with a dot on the
   baseline, never zero. */
function trendSvg({w, h = 172, values, bars = false, levels = false, overlay, previous, scale, scale2, sel, fmt = v => v, fmt2 = v => v, band}) {
  const n = values.length, L = 34, R = overlay ? w - 38 : w - 6, T = 10, B = h - 22;
  const X = i => L + (i + .5) * (R - L) / n, Y = (v, [a, b]) => B - (v - a) / (b - a) * (B - T);
  const path = (vals, sc) => vals.map((v, i) => v == null ? "" : `${i && vals[i - 1] != null ? "L" : "M"}${X(i).toFixed(1)},${Y(v, sc).toFixed(1)}`).join("");
  const ticks = [scale[1], (scale[0] + scale[1]) / 2, scale[0]];
  let s = `<svg class="tsvg" width="${w}" height="${h}" viewBox="0 0 ${w} ${h}">`;
  if (band) s += `<rect x="${L}" y="${Y(band[1], scale)}" width="${R - L}" height="${Y(band[0], scale) - Y(band[1], scale)}" fill="#fff" opacity=".1"/>`;
  ticks.forEach((t, i) => {
    s += `<line x1="${L}" x2="${R}" y1="${Y(t, scale)}" y2="${Y(t, scale)}" stroke="#fff" opacity=".07"/><text x="0" y="${Y(t, scale) + 3}" class="tk">${fmt(t)}</text>`;
    if (overlay) { const t2 = [scale2[1], (scale2[0] + scale2[1]) / 2, scale2[0]][i]; s += `<text x="${w}" y="${Y(t, scale) + 3}" class="tk" text-anchor="end">${fmt2(t2)}</text>`; }
  });
  if (sel != null) s += `<line x1="${X(sel)}" x2="${X(sel)}" y1="${T - 4}" y2="${B + 4}" stroke="#fff" opacity=".35" stroke-dasharray="2 3"/>`;
  const bw = Math.max(1.6, Math.min(16, (R - L) / n * .62));
  if (previous) s += `<path d="${path(previous, scale)}" fill="none" stroke="${C.ink}" stroke-opacity=".42" stroke-width="1.6" stroke-dasharray="4 4" stroke-linecap="round"/>`;
  values.forEach((v, i) => {
    if (v == null) { s += `<circle cx="${X(i)}" cy="${B + 6}" r="1.4" fill="#fff" opacity=".3"/>`; return; }
    if (bars) s += `<rect x="${X(i) - bw / 2}" y="${Y(v, scale)}" width="${bw}" height="${Math.max(2, B - Y(v, scale))}" rx="${Math.min(2, bw / 2)}" fill="${levels ? readinessLevel(v) : C.ink}" opacity="${i === sel ? 1 : overlay ? .4 : levels ? .6 : .38}"/>`;
  });
  if (!bars) {
    s += `<path d="${path(values, scale)}" fill="none" stroke="${C.ink}" stroke-width="2" stroke-linejoin="round" stroke-linecap="round"/>`;
    if (n <= 40) values.forEach((v, i) => { if (v != null && i !== sel) s += `<circle cx="${X(i)}" cy="${Y(v, scale)}" r="2.2" fill="${C.ink}"/>`; });
  }
  if (overlay) s += `<path d="${path(overlay, scale2)}" fill="none" stroke="${C.ink2}" stroke-width="1.6" stroke-linejoin="round" stroke-linecap="round" stroke-dasharray="1 0"/>`;
  if (sel != null && values[sel] != null && !bars) s += `<circle cx="${X(sel)}" cy="${Y(values[sel], scale)}" r="4.5" fill="${C.ink}" stroke="#0B0F1A" stroke-width="2"/>`;
  if (sel != null && overlay?.[sel] != null) s += `<circle cx="${X(sel)}" cy="${Y(overlay[sel], scale2)}" r="4" fill="#0B0F1A" stroke="${C.ink2}" stroke-width="2"/>`;
  return s + "</svg>";
}

const slider = (pos, max) => `<div class="tctl"><button>${ico.left}</button><div class="track"><i style="left:${pos / max * 100}%"></i></div><button>${ico.right}</button></div>`;
const axis = labels => `<div class="taxis">${labels.map(l => `<span>${l}</span>`).join("")}</div>`;
// All has no period before it, so Previous is off there.
const compareBar = (on, extra = "", all = false) => {
  const seg = segc(["Off", "Previous", "Metric"], on, "plain cmp-seg");
  return `<div class="cmp"><span class="cmp-l">Compare</span>${all ? seg.replace("<span>Previous", '<span class="dis">Previous') : seg}${extra}</div>`;
};
// Narrow and wide renderings of the same chart; container queries pick one.
const both = (o, wide = 800) => `<div class="narrow-only">${trendSvg({...o, w: 326})}</div><div class="wide-only">${trendSvg({...o, w: wide})}</div>`;

/* ---------- Readiness, one year of weekly averages ---------- */
const YEAR = series(52, {seed: 11, base: 70, swing: 7, noise: 9, gaps: [[30, 31]]});
function trendYear() {
  const sel = 49;
  return shell(`${back("today", "Today")}
  <header class="top"><div><div class="eyebrow">Thu 16 Oct</div><h1>Readiness</h1></div></header>
  <div class="tgrid">
  <div class="card tcard"><h3>Readiness history ${rangeBar("1Y")}</h3>
    <div class="tread"><div><b class="n">${YEAR[sel]}</b><span>weekly average</span></div><span>Week of 22 Sep<small>6 of 7 days recorded</small></span></div>
    ${both({values: YEAR, bars: true, scale: [0, 100], sel}, 800)}
    ${axis(["Oct 2025", "Apr 2026", "Oct 2026"])}${slider(sel, 51)}
    <div class="lgd"><span><i style="background:${C.ink}"></i>Weekly average of recorded days</span><span class="cnt">50 / 52 weeks</span></div>
    ${compareBar(0)}</div>
  <div class="card"><h3>What's driving it</h3>${[["Acute load", 58, C.fair], ["Stress hist.", 73, C.good], ["HRV", 78, C.good]].map(([n, v, c]) => `<div class="f"><span>${n}<b>${v}</b></span><div class="bar"><i style="width:${v}%;background:${c}"></i></div></div>`).join('<div style="height:12px"></div>')}</div>
  <div class="card"><h3>About averages</h3><p class="lp">Long ranges average the days Garmin recorded. Garmin rates single days, not averages, so weekly bars stay ink. Missing days are left out, never counted as zero.</p></div>
  </div>`, "today");
}

/* ---------- Readiness with time asleep overlaid, 90 days ---------- */
const R90 = series(90, {seed: 5, base: 72, swing: 6, noise: 22, gaps: [[41, 41], [63, 64]]});
const S90 = R90.map((v, i) => v == null ? (i === 64 ? null : 430) : Math.round(380 + v * 1.1 + (rng(i)() - .5) * 50));
function trendOverlay() {
  const sel = 84;
  return shell(`${back("today", "Today")}
  <header class="top"><div><div class="eyebrow">Thu 16 Oct</div><h1>Readiness</h1></div></header>
  <div class="tgrid">
  <div class="card tcard"><h3>Readiness history ${rangeBar("90D")}</h3>
    <div class="pick"><span>Primary metric · Readiness</span>${ico.caret}</div>
    <div class="tread two"><div><b class="n" style="color:${readinessLevel(R90[sel])}">${R90[sel]}</b><span>readiness</span></div><div class="ov"><b class="n">${hm(S90[sel])}</b><span>time asleep</span></div><span>Sat 11 Oct</span></div>
    ${both({values: R90, bars: true, levels: true, scale: [0, 100], overlay: S90, scale2: [300, 540], sel, fmt2: hm}, 800)}
    ${axis(["19 Jul", "2 Sep", "16 Oct"])}${slider(sel, 89)}
    <div class="lgd"><span><i style="background:${C.poor}"></i>Low / Poor</span><span><i style="background:${C.fair}"></i>Moderate</span><span><i style="background:${C.good}"></i>High / Prime</span><span><i class="ln"></i>Time asleep, right axis</span></div>
    ${compareBar(2, `<div class="pick"><span>Time asleep</span>${ico.caret}</div>`)}</div>
  <div class="card"><h3>Today's call</h3><div class="seg"><span>Rest</span><span>Easy</span><span>Steady</span><span class="on">Push</span></div></div>
  <div class="card"><h3>Two metrics</h3><p class="lp">Each line keeps its own scale and Garmin's own numbers. Lining them up shows when they move together; it doesn't say one causes the other.</p></div>
  </div>`, "today");
}

/* ---------- HRV, 30 nights against the 30 before ---------- */
const H30 = series(30, {seed: 3, base: 58, swing: 3, noise: 9, gaps: [[12, 12]]});
const HP = series(30, {seed: 9, base: 54, swing: 2, noise: 9, gaps: [[20, 21]]});
function trendCompare() {
  const sel = 29;
  return shell(`${back("health", "Health")}
  <header class="top"><div><div class="eyebrow">Garmin metric</div><h1>HRV</h1></div></header>
  <div class="tgrid">
  <div class="card tcard"><h3>Nightly history ${rangeBar("Dates")}</h3>
    <div class="pick"><span>Primary metric · HRV</span>${ico.caret}</div>
    <div class="t-blocks"><label>Block start<input type="date" value="2025-09-17"></label><label>Block end<input type="date" value="2025-10-16"></label></div>
    <div class="tread two"><div><b class="n">${H30[sel]}</b><span>ms</span></div><div class="ov"><b class="n dim">${HP[sel]}</b><span>30 nights before</span></div><span>Thu 16 Oct<small>Latest night</small></span></div>
    ${both({values: H30, previous: HP, scale: [36, 76], band: [48, 62], sel}, 800)}
    ${axis(["17 Sep", "1 Oct", "16 Oct"])}${slider(sel, 29)}
    <div class="lgd"><span><i class="ln solid"></i>Last 30 nights</span><span><i class="ln dash"></i>30 nights before</span><span><i style="background:rgba(255,255,255,.13)"></i>Current baseline</span></div>
    ${compareBar(1)}
    <div class="t-blocks"><label>Comparison block start<input type="date" value="2025-08-18"></label><span>Same length · aligned by elapsed day</span></div>
    <div class="pavg"><div><span>Average</span><b class="n">58<small>ms</small></b><em>29 nights</em></div><div><span>30 nights before</span><b class="n dim">54<small>ms</small></b><em>28 nights</em></div><div><span>Change</span><b class="n">+4<small>ms</small></b></div></div></div>
  <div class="card"><h3>Baseline</h3><div class="rb" style="margin:6px 0 10px"><u style="left:30%;width:35%"></u><i style="left:55%;background:${C.good}"></i></div><p class="lp">48–62 ms · Garmin</p></div>
  <div class="card"><h3>About this metric</h3><p class="lp">Your nightly average against Garmin's baseline. Averages and changes are descriptive: no new score, and the change carries no verdict.</p></div>
  </div>`, "health");
}

/* ---------- Stress, everything since 2014 in monthly averages ---------- */
const ALL = series(150, {seed: 21, base: 33, swing: 5, noise: 8, gaps: [[0, 0], [14, 52], [97, 99]]});
function trendAll() {
  const sel = 146;
  return shell(`${back("health", "Health")}
  <header class="top"><div><div class="eyebrow">Garmin metric</div><h1>Stress</h1></div></header>
  <div class="tgrid">
  <div class="card tcard"><h3>Daily average ${rangeBar("All")}</h3>
    <div class="tread"><div><b class="n">${ALL[sel]}</b><span>monthly average</span></div><span>July 2026<small>29 of 31 days recorded</small></span></div>
    ${both({values: ALL, bars: true, scale: [0, 60], sel}, 800)}
    ${axis(["May 2014", "2020", "Oct 2026"])}${slider(sel, 149)}
    <div class="lgd"><span><i style="background:${C.ink}"></i>Monthly average of recorded days</span><span class="cnt">108 / 150 months</span></div>
    ${compareBar(0, "", true)}
    <p class="lp tnote">Gaps are months your watches didn't rate stress.</p></div>
  </div>`, "health");
}

SCREENS.push(
  ["trend-year", "Readiness · 1 year", trendYear, "phone", "Trends"],
  ["trend-overlay", "Overlay a metric", trendOverlay, "phone", "Trends"],
  ["trend-compare", "Previous period", trendCompare, "phone", "Trends"],
  ["trend-all", "All history", trendAll, "phone", "Trends"],
  ["trend-overlay", "Overlay a metric", trendOverlay, "fold", "Trends"],
  ["trend-compare", "Previous period", trendCompare, "desk", "Trends"],
);
