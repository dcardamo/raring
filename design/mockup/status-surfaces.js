/* Synthetic snapshots for the Android home-screen widgets; the preview shares one renderer. */
const SURFACE = {
  day: {asOf: '14:05', readiness: 82, sleep: 84, bb: 54, stress: 31, focus: 'Push'},
  evening: {asOf: '20:25', readiness: 82, sleep: 84, bb: 37, stress: 28, focus: 'Wind down at 22:00'},
  missing: {asOf: 'yesterday 23:10', readiness: null, sleep: null, bb: null, stress: null, focus: 'Waiting for watch upload'},
};
const hasMetric = v => v !== null && v !== undefined;
const freshness = s => `<span class="sf-time">As of ${s.asOf}</span>`;
const surfaceBrand = () => `<span class="sf-brand"><img src="../../site/assets/icon.svg" alt="">Raring</span>`;

function surfaceGauge(s, size) {
  return `<div class="sf-gauge">${arc({size, value: s.readiness ?? 0, color: C.good, stroke: 8})}
    <div class="sf-gauge-text"><b class="n">${hasMetric(s.readiness) ? s.readiness : ''}</b><span>Readiness</span></div></div>`;
}
function statusSmall(s) {
  return `<div class="sf-widget sf-small tap" data-go="today"><div class="sf-widget-top">${surfaceBrand()}</div>
    ${surfaceGauge(s, 78)}<div class="sf-small-focus">${s.focus}</div>${freshness(s)}</div>`;
}
function statusMedium(s) {
  const metric = (label, val, route, extra = '') => `<div class="sf-tile tap" data-go="${route}"><span>${label}</span><b class="n">${hasMetric(val) ? val : ''}</b>${hasMetric(val) ? `<small>${extra}</small>` : ''}</div>`;
  return `<div class="sf-widget sf-medium"><div class="sf-widget-top">${surfaceBrand()}${freshness(s)}</div>
    <div class="sf-medium-grid"><div class="sf-readiness tap" data-go="today">${surfaceGauge(s, 76)}<span>${hasMetric(s.readiness) ? 'High' : ''}</span></div>
    ${metric('Sleep', s.sleep, 'sleep', '/100')}${metric('Body Battery', s.bb, 'today', '/100')}${metric('Stress', s.stress, 'today', '/100')}</div></div>`;
}

function surfaceSection(title, s) {
  return `<section class="sf-section"><h2>${title}</h2><div class="sf-label">Home screen · small + medium</div><div class="sf-widget-pair">${statusSmall(s)}${statusMedium(s)}</div></section>`;
}
function statusSurfaces() {
  return `${statusBar()}<div class="scroll"><div class="sf-page">
    <header class="sf-intro"><span class="eyebrow">Android</span><h1>Home-screen widgets</h1><p>Quiet at a glance. Values are from the last watch upload, never live.</p></header>
    ${surfaceSection('During the day · after a run', SURFACE.day)}
    ${surfaceSection('Evening · sleep takes the lead', SURFACE.evening)}
    ${surfaceSection('Missing · no watch upload', SURFACE.missing)}
  </div></div>`;
}
function statusWidgetHome() {
  const icon = c => `<div class="happ"><i style="background:${c}"></i><span></span></div>`;
  return `${statusBar()}<div class="home sf-home"><div class="hclock"><b class="n">14:20</b><span>Thursday 16 October</span></div>
    <div class="sf-home-pair">${statusMedium(SURFACE.day)}${statusSmall(SURFACE.day)}
      <div class="sf-home-apps">${icon('#2A3350')}${icon('#33304A')}<div class="happ"><img src="../../site/assets/icon.svg" width="54" height="54" alt=""><span>Raring</span></div>${icon('#283A44')}</div>
    </div><div class="dock"><div class="dk">${['#2B3350', '#33304A', '#283A44', '#3A2F3A'].map(c => `<i style="background:${c}"></i>`).join('')}</div><div class="hsearch">${ico.search}</div></div>
  </div>`;
}
SCREENS.push(['status-surfaces', 'Widgets · day, evening, missing', statusSurfaces, 'phone', 'Status surfaces']);
