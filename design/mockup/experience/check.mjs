// Browser checks for the experience prototype: U-case states, counted journeys,
// layout budgets and accessibility heuristics. Synthetic data only.
//   node design/mockup/experience/check.mjs [--shots]
// Needs Playwright from app/node_modules (RARING_NODE_MODULES overrides) and Chromium
// (CHROMIUM overrides). A prototype pass is not usability, device or real-data evidence.
import {createRequire} from 'node:module';
import {mkdir, writeFile} from 'node:fs/promises';
import {dirname, resolve} from 'node:path';
import {fileURLToPath, pathToFileURL} from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const modules = process.env.RARING_NODE_MODULES ?? resolve(here, '../../../app/node_modules');
const {chromium} = createRequire(resolve(modules, 'noop.js'))('playwright');
const base = process.env.EXPERIENCE_URL ?? pathToFileURL(resolve(here, 'index.html')).href;
const shots = process.argv.includes('--shots');
const outDir = resolve(here, 'shots');
const failures = [], notes = [], journeys = [], layout = [];
const fail = (what, detail) => failures.push(`${what}: ${detail}`);

const browser = await chromium.launch({executablePath: process.env.CHROMIUM ?? '/run/current-system/sw/bin/chromium', args: ['--no-sandbox']});
async function fresh(viewport = {width: 390, height: 844}, opts = {}) {
  const context = await browser.newContext({viewport, deviceScaleFactor: 1, reducedMotion: opts.reducedMotion ?? 'no-preference'});
  // Block anything that isn't the prototype itself: no remote fonts, analytics or data.
  await context.route('**/*', route => route.request().url().startsWith(base.replace(/index\.html.*$/, '')) ? route.continue() : (fail('network', route.request().url()), route.abort()));
  const page = await context.newPage();
  page.on('pageerror', e => fail('pageerror', e.message));
  return {context, page};
}
const open = async (page, query, hash = '') => { await page.goto(`${base}?${query}${hash}`); await page.waitForTimeout(60); };
const route = page => page.evaluate(() => location.hash.slice(1));
const text = page => page.evaluate(() => document.querySelector('main').textContent);
const settle = page => page.waitForTimeout(90);

// Every deliberate tap is counted; typing is not (design section 15).
function counter(page, name) {
  const path = [];
  const tap = async (label, scope = 'main, .sheet') => {
    const loc = typeof label === 'string'
      ? page.locator(`${scope}`).getByRole('button', {name: label, exact: true}).first()
      : page.locator(`${scope}`).getByRole('button', {name: label}).first();
    await loc.click();
    path.push(typeof label === 'string' ? label : String(label));
    await settle(page);
  };
  return {tap, path, done: (expectRoute, budget, extra = '') => { if (typeof budget === 'number' && path.length > budget) fail(name, `${path.length} taps exceeds ${budget}`); journeys.push({name, taps: path.length, budget, path: path.join(' → '), route: expectRoute, extra}); }};
}

// ---------- 1. U-case states ----------
{
  const {context, page} = await fresh();
  await open(page, '');
  const cases = await page.evaluate(() => window.RaringCases);
  for (const c of cases) {
    await open(page, `case=${c.id}`);
    const t = await text(page);
    for (const e of c.expect) if (!t.includes(e)) fail(c.id, `missing “${e}” on #${await route(page)}`);
    for (const f of c.forbid) if (t.toLowerCase().includes(f.toLowerCase())) fail(c.id, `forbidden “${f}” on #${await route(page)}`);
  }
  notes.push(`${cases.length} U-case states loaded`);
  await context.close();
}

// ---------- 2. Journeys ----------
async function journey(name, caseId, fn, viewport) {
  const {context, page} = await fresh(viewport);
  await open(page, `case=${caseId}`);
  await page.locator('.case-note button').click().catch(() => {});
  try { await fn(page, counter(page, name)); } catch (e) { fail(name, e.message.split('\n')[0]); }
  await context.close();
}
const expectRoute = async (page, want, label) => { const r = await route(page); if (r !== want) throw new Error(`${label}: expected #${want}, got #${r}`); };
const expectText = async (page, want, label) => { const t = await text(page); if (!t.includes(want)) throw new Error(`${label}: missing “${want}”`); };
const expectNoText = async (page, want, label) => { const t = await text(page); if (t.includes(want)) throw new Error(`${label}: unexpected “${want}”`); };

await journey('U03 Sleep-only: last night from Today', 'U03', async (page, j) => {
  await j.tap(/^Sleep 5h 45/);
  await expectRoute(page, 'sleep', 'U03'); await expectText(page, 'Night ending Wed 18 Nov', 'U03');
  j.done('sleep', 1);
  await page.locator('.back').click(); await settle(page); await expectRoute(page, 'today', 'U03 Back');
});
await journey('U04 Tonight from Today', 'U03', async (page, j) => {
  await j.tap(/^Tonight: wind down/);
  await expectRoute(page, 'sleep/tonight', 'U04'); await expectText(page, 'Garmin plan', 'U04');
  const inView = await page.evaluate(() => { const r = document.getElementById('sec-tonight').getBoundingClientRect(); return r.top >= 0 && r.top < innerHeight / 2; });
  if (!inView) throw new Error('U04: Tonight section not brought into view');
  j.done('sleep/tonight', 1, 'opens the Tonight section, not the top of Sleep');
});
await journey('U05 Recorded workout from its preview', 'U05', async (page, j) => {
  await j.tap(/^Walk · 12:30/);
  await expectRoute(page, 'workout/a-1204', 'U05'); await expectText(page, 'No target needed', 'U05');
  j.done('workout/a-1204', 1);
});
await journey('U11 Add a local intention', 'U11', async (page, j) => {
  await j.tap('Plan a session'); await j.tap('Strength'); await j.tap('Add to plan');
  await expectRoute(page, 'today', 'U11');
  const s = await page.evaluate(() => window.RaringApp.state.intentions);
  if (s.length !== 1 || s[0].date !== '2026-11-20' || s[0].time) throw new Error(`U11: unexpected intention ${JSON.stringify(s)}`);
  await expectText(page, 'Strength today', 'U11 lead now shows the intention');
  j.done('today', 'one sheet', 'activity + day (today preselected), time optional');
});
await journey('U28 Rest today, offline, resume', 'U28', async (page, j) => {
  await j.tap(/^Adapt today/); await j.tap('Rest today');
  await expectText(page, 'Rest day', 'U28 lead');
  j.done('today', 2);
  await page.evaluate(() => window.RaringApp.set({offline: true}));
  await page.reload(); await settle(page);
  await expectText(page, 'Rest day', 'U28 after reload offline');
  await expectText(page, 'Offline · saved data', 'U28 offline label');
  await page.getByRole('button', {name: 'Plan', exact: true}).click(); await settle(page);
  await expectText(page, 'Rest today: no exercise prompts today or tonight.', 'U28 Plan reflects rest');
  await page.evaluate(() => { window.RaringApp.set({permission: 'granted'}); window.RaringApp.external('surfaces'); }); await settle(page);
  await expectText(page, 'Rest day · no training call', 'U28 notification suppressed');
  await page.evaluate(() => window.RaringApp.set({clock: 'late'})); await settle(page);
  await page.evaluate(() => window.RaringApp.external('today')); await settle(page);
  await expectText(page, 'Rest today', 'U28 rest carries through the night');
  await page.evaluate(() => window.RaringApp.set({day: '2026-11-21', clock: 'morning'})); await settle(page);
  const label = await page.locator('.adapt').textContent();
  if (label.trim() !== 'Adapt today') throw new Error(`U28: rest should end in the next waking day, header says ${label}`);
});
await journey('U29 Pause then explicit resume', 'U12', async (page, j) => {
  await page.evaluate(() => window.RaringApp.set({rest: null, day: '2026-11-19'})); await settle(page);
  await j.tap(/^Adapt today/); await j.tap(/^Pause training/); await j.tap('Pause from today');
  j.done('adapt/pause', 3, 'longer-lived circumstance: one extra deliberate step');
  await page.evaluate(() => window.RaringApp.set({day: '2026-11-20'}));
  await page.evaluate(() => window.RaringApp.external('today')); await settle(page);
  await expectText(page, 'Training is paused', 'U29 high readiness stays paused');
  await expectNoText(page, 'Plan a session', 'U29');
  await page.evaluate(() => window.RaringApp.external('plan')); await settle(page);
  await expectText(page, 'Adjusted · paused since Thu', 'U26 mid-week pause label');
  await page.evaluate(() => window.RaringApp.set({day: '2026-11-22'})); await settle(page);
  const r = counter(page, 'U29 Resume');
  await r.tap(/^Paused/); await r.tap(/^Pause training/); await r.tap('Resume training…'); await r.tap('Resume as they are');
  r.done('adapt/pause', 4, 'shows commitments for review; no catch-up');
  await page.evaluate(() => window.RaringApp.external('plan')); await settle(page);
  await expectText(page, 'Adjusted · paused Thu–Sun', 'U26 pause interval retained after resume');
});
await journey('U30 Exclude an activity and add a movement note', 'U19', async (page, j) => {
  await page.evaluate(() => window.RaringApp.set({day: '2026-11-19'})); await settle(page);
  await page.evaluate(() => window.RaringApp.external('today')); await settle(page);
  await j.tap(/^Adapt today/); await j.tap(/^Avoid activities or movements/); await j.tap('Run');
  await page.locator('textarea').fill('No deep knee bends'); await page.locator('textarea').blur(); await settle(page);
  j.done('adapt/limits', 3, 'plus typing the note');
  await page.evaluate(() => window.RaringApp.external('today')); await settle(page);
  await expectText(page, 'A Garmin session conflicts with your limits', 'U30 Today');
  await page.evaluate(() => window.RaringApp.external('plan')); await settle(page);
  await expectText(page, 'review needed for your movement note', 'U31 Plan');
  await page.evaluate(() => window.RaringApp.external('surfaces')); await settle(page);
  const s = await text(page);
  if (s.includes('knee')) throw new Error('U30: movement note leaked into a status surface');
});
await journey('U22 Link a recorded activity to an intention', 'U15', async (page, j) => {
  await page.evaluate(() => window.RaringApp.external('plan')); await settle(page);
  await j.tap(/Strength · 07:00/); await j.tap('This was my session');
  await expectText(page, 'Linked by you', 'U22');
  j.done('intention/i0', 2, 'from Plan');
});
await journey('U22b Intention left unlinked is not missed', 'U15', async (page) => {
  await page.evaluate(() => window.RaringApp.set({day: '2026-11-17'}));
  await page.evaluate(() => window.RaringApp.external('intention/i0')); await settle(page);
  await expectText(page, 'Not linked. That’s not a missed workout', 'U22b');
});
for (const [root, label] of [['today', 'Today'], ['plan', 'Plan'], ['progress', 'Progress']]) {
  for (const area of ['Sleep', 'Readiness', 'Training', 'Strength', 'Activities', 'Health']) {
    await journey(`Discover ${area} from ${label}`, 'U19', async (page, j) => {
      await page.evaluate(r => window.RaringApp.external(r), root); await settle(page);
      await j.tap(/^Explore$/); await j.tap(new RegExp(`^${area}`));
      await expectRoute(page, area.toLowerCase(), `${area} from ${label}`);
      j.done(area.toLowerCase(), 2);
      await page.locator('.back').click(); await settle(page); await expectRoute(page, 'explore', `${area} Back`);
      await page.locator('.back').click(); await settle(page); await expectRoute(page, root, `${area} Back to ${label}`);
    });
  }
}
await journey('U50 Rare metric by search', 'U19', async (page, j) => {
  await j.tap(/^Explore$/);
  await page.getByRole('searchbox').fill('hill'); await settle(page);
  await j.tap(/^Hill score/);
  await expectRoute(page, 'metric/hill', 'U50 search'); j.done('metric/hill', 2, 'plus typing “hill”');
});
await journey('U50 Rare metric by browsing', 'U19', async (page, j) => {
  await j.tap(/^Explore$/); await j.tap(/^Training/); await j.tap(/^Hill score/);
  await expectRoute(page, 'metric/hill', 'U50 browse'); j.done('metric/hill', 3);
});
await journey('U50 Gear', 'U19', async (page, j) => {
  await j.tap(/^Explore$/); await j.tap(/^Gear Shoes and bikes/);
  await expectRoute(page, 'gear', 'U50 gear'); j.done('gear', 2);
});
await journey('U50 Comparison', 'U19', async (page, j) => {
  await j.tap(/^Explore$/); await j.tap(/^Trends & comparisons/); await j.tap(/^VO₂ max/);
  await page.locator('main select').selectOption('prev'); j.path.push('Compare: Previous period'); await settle(page);
  await expectText(page, 'dashed: previous period', 'U50 compare'); j.done('metric/vo2max', 4);
});
await journey('Back keeps filter and scroll', 'U19', async (page, j) => {
  await j.tap(/^Explore$/); await j.tap(/^Activities/); await j.tap('Strength');
  await page.evaluate(() => window.scrollTo(0, 160)); await settle(page);
  const before = await page.evaluate(() => scrollY);
  await j.tap(/^Strength · Mon 16 Nov/);
  await page.goBack(); await settle(page);
  const pressed = await page.locator('.chip[aria-pressed=true]').first().textContent();
  const after = await page.evaluate(() => scrollY);
  if (pressed.trim() !== 'Strength') throw new Error(`filter lost on browser Back: ${pressed}`);
  if (Math.abs(after - before) > 2) throw new Error(`scroll not restored: ${before} → ${after}`);
  if (after < 50) throw new Error('scroll test did not scroll');
  j.done('activities', 'n/a', `browser Back restored filter and scroll ${after}px`);
}, {width: 390, height: 520});
await journey('U51 Past day of an intraday metric', 'U51', async (page) => {
  await page.locator('main input[type=range]').evaluate(el => { el.value = '10'; el.dispatchEvent(new Event('change', {bubbles: true})); }); await settle(page);
  await expectText(page, 'only downloaded for recent days', 'U51');
});
await journey('U53 Journal consent, open, leave, background, opt out', 'U52', async (page, j) => {
  await j.tap('Allow journal reads…'); await j.tap('Allow journal reads'); await j.tap('Open journal');
  await expectText(page, 'Caffeine', 'U53 open'); j.done('health/journal', 3, 'consent sheet then explicit open');
  await page.locator('.back').click(); await settle(page);
  await expectNoText(page, 'Caffeine', 'U53 hidden after immediate Back to Health');
  await page.locator('.back').click(); await settle(page); await expectRoute(page, 'explore', 'U53 left Health');
  await page.evaluate(() => window.RaringApp.navigate('health/journal')); await settle(page);
  await expectNoText(page, 'Caffeine', 'U53 hidden after leaving');
  await page.getByRole('button', {name: 'Open journal'}).click(); await settle(page);
  await page.evaluate(() => { Object.defineProperty(document, 'hidden', {configurable: true, value: true}); document.dispatchEvent(new Event('visibilitychange')); }); await settle(page);
  await expectNoText(page, 'Caffeine', 'U53 hidden after background');
  await page.evaluate(() => window.RaringApp.external('today')); await settle(page);
  await expectNoText(page, 'Caffeine', 'U52 Today never shows journal');
  await page.evaluate(() => window.RaringApp.external('settings')); await settle(page);
  await page.locator('#sec-journal input[type=checkbox]').uncheck(); await settle(page);
  await expectText(page, 'downloaded logs', 'U53 opt-out disclosure in Settings');
  await page.evaluate(() => window.RaringApp.external('health/journal')); await settle(page);
  await expectText(page, '4 downloaded days remain on this phone', 'U53 retention after opt-out');
  await expectNoText(page, 'Caffeine', 'U53 entries hidden after opt-out');
});
await journey('U56 Dismiss survives sync and resume; new notice is distinct', 'U56', async (page, j) => {
  await j.tap('Not today');
  await expectText(page, 'Nothing else needs you today', 'U56');
  j.done('today', 1);
  await page.evaluate(() => window.RaringApp.sync()); await settle(page);
  await page.reload(); await settle(page);
  await expectText(page, 'Nothing else needs you today', 'U56 after sync + reload');
  await page.evaluate(() => window.RaringApp.sync('exception')); await settle(page);
  await expectText(page, 'Two overnight readings are outside your usual ranges', 'U56 new exception');
  await expectText(page, 'Nothing else needs you today', 'U56 still dismissed');
});
await journey('U64 Settings, export disclosure, sign out', 'U64', async (page, j) => {
  await j.tap('Settings'); await expectText(page, 'circumstances and notes, routine checklist and self-reports', 'U64 export');
  await j.tap('Sign out and erase…');
  const sheet = await page.locator('.sheet').textContent();
  if (!sheet.includes('Rest, pauses, limits and notes')) throw new Error('U64: erase disclosure misses circumstances');
  j.done('settings', 2, 'export disclosure visible on Settings');
});
await journey('U58 Old links and Back', 'U58', async (page) => {
  const map = await page.evaluate(() => Object.entries(window.RaringApp.LEGACY).map(([from]) => [from, window.RaringApp.canonical(from).route, window.RaringApp.defaultTrail(window.RaringApp.canonical(from).route)]));
  for (const [from, to, trail] of map) {
    await page.evaluate(h => { location.hash = h; }, from); await page.waitForTimeout(150);
    const r = await route(page);
    if (r !== to) { fail('U58', `#${from} opened #${r}, expected #${to}`); continue; }
    for (let i = trail.length - 2; i >= 0; i--) {
      await page.goBack(); await page.waitForTimeout(120);
      const b = await route(page);
      if (b !== trail[i]) { fail('U58', `#${from}: Back reached #${b}, expected #${trail[i]}`); break; }
    }
  }
  await page.evaluate(() => { location.hash = 'not-a-page'; }); await page.waitForTimeout(150);
  await expectText(page, 'today', 'U58 invalid');
  const note = await page.locator('.case-note').textContent();
  if (!note.includes('isn’t a Raring page')) fail('U58', 'invalid link opened Today silently');
  notes.push(`${map.length} legacy links mapped with Back paths`);
});
await journey('U58 Status surface tap', 'U01', async (page, j) => {
  await page.evaluate(() => { window.RaringApp.set({permission: 'granted'}); window.RaringApp.external('surfaces'); }); await settle(page);
  await j.tap(/^Morning readiness/); await expectRoute(page, 'readiness', 'U58 surface');
  await page.goBack(); await settle(page); await expectRoute(page, 'today', 'U58 surface Back');
  j.done('readiness', 1, 'cold link; Back → Today');
});
await journey('Interrupted week walkthrough', 'U15', async (page) => {
  const A = (fn, arg) => page.evaluate(([f, a]) => window.RaringApp[f](a), [fn, arg]);
  const at = async (day, clock, where = 'today') => { await page.evaluate(([d, c]) => window.RaringApp.set({day: d, clock: c}), [day, clock]); await A('external', where); await settle(page); };
  await at('2026-11-16', 'day'); await expectText(page, 'Strength recorded', 'Mon recognition');
  await at('2026-11-17', 'evening', 'sleep/tonight');
  await page.getByRole('button', {name: 'I wound down (optional)'}).click(); await settle(page);
  await at('2026-11-18', 'morning', 'sleep'); await expectText(page, 'You marked wind-down done', 'Wed preparation separate'); await expectText(page, '5h 10', 'Wed imperfect night');
  await at('2026-11-18', 'day'); await expectText(page, 'An easier day may fit', 'Wed conflict');
  await page.getByRole('button', {name: 'Review session'}).click(); await settle(page);
  await page.getByRole('button', {name: 'Rest today'}).click(); await settle(page);
  await at('2026-11-18', 'day'); await expectText(page, 'Intervals 6 × 3 min stays on your Garmin calendar, unchanged.', 'Wed rest keeps schedule as context');
  await at('2026-11-18', 'late'); if ((await page.locator('.adapt').textContent()).trim() !== 'Rest today') fail('week', 'rest did not carry past midnight');
  await at('2026-11-19', 'day'); if ((await page.locator('.adapt').textContent()).trim() !== 'Adapt today') fail('week', 'rest did not end on Thursday');
  await A('external', 'adapt/pause'); await settle(page); await page.getByRole('button', {name: 'Pause from today'}).click(); await settle(page);
  await at('2026-11-20', 'day'); await expectText(page, 'Training is paused', 'Fri pause with readiness 84');
  await at('2026-11-20', 'day', 'progress'); await expectText(page, 'You kept going before the pause', 'Fri Progress');
  await at('2026-11-22', 'day', 'adapt/pause'); await page.getByRole('button', {name: 'Resume training…'}).click(); await settle(page);
  await page.getByRole('button', {name: 'Resume as they are'}).click(); await settle(page);
  await at('2026-11-22', 'day', 'review/2026-W47'); await expectText(page, 'Adjusted · paused Thu–Sun', 'Sun review keeps the interval'); await expectNoText(page, 'Reached', 'no retroactive award');
  await at('2026-11-22', 'day', 'review/2026-W46'); await expectText(page, 'targets saved then: 3 strength · 150 min', 'old version honest');
  notes.push('interrupted week walked Mon→Sun');
});

// Reviewer regressions: actual transitions, not start-page copy checks.
await journey('Morning rest owns actual calendar day', 'U10', async (page, j) => {
  await j.tap(/^Adapt today/); await j.tap('Rest today');
  const day = await page.evaluate(() => window.RaringApp.state.rest.day);
  if (day !== '2026-11-18') throw new Error(`rest saved for ${day}, not the actual day`);
  await page.evaluate(() => window.RaringApp.set({clock: 'morning'}));
  await expectText(page, 'Rest day', 'rest survives synced-night handover');
});
await journey('Movement review suppresses every training prompt', 'U31', async (page) => {
  await page.evaluate(() => { window.RaringApp.set({day: '2026-11-21', permission: 'granted'}); window.RaringApp.external('today'); }); await settle(page);
  await expectText(page, 'Review needed before this session', 'unknown leg routine');
  await page.evaluate(() => window.RaringApp.external('surfaces')); await settle(page);
  await expectText(page, 'Training suggestions limited', 'limited surfaces');
  await expectNoText(page, 'Training is an option', 'no high-readiness override');
  await page.evaluate(() => window.RaringApp.set({permission: 'declined'})); await settle(page);
  if (await page.locator('.surface').filter({hasText: 'Morning readiness'}).count()) throw new Error('notification shown after permission denied');
});
await journey('Historical revisions do not time travel', 'U26', async (page) => {
  await page.evaluate(() => window.RaringApp.set({day: '2026-11-16'})); await settle(page);
  await expectNoText(page, 'Reached · adjusted Thu', 'Monday before revision');
  await page.evaluate(() => window.RaringApp.external('workout/a-1107')); await settle(page);
  await expectText(page, 'Counted as a strength session', 'old timer evidence');
  await page.evaluate(() => window.RaringApp.external('workout/a-1102')); await settle(page);
  await expectText(page, '1 of 3', 'older workout uses week 46 targets');
});
await journey('Permission denial does not repeat prompt', 'U57', async (page, j) => {
  await j.tap('Remind me to wind down');
  if (await page.locator('.sheet').count()) throw new Error('permission requested again');
  await page.evaluate(() => window.RaringApp.external('surfaces')); await settle(page);
  await expectText(page, 'Persistent status off', 'status defaults off');
});
await journey('Metric dates and comparison survive caller Back', 'U51', async (page, j) => {
  await j.tap('Dates');
  await page.getByLabel('From', {exact:true}).fill('2026-10-25');
  await page.getByLabel('To', {exact:true}).fill('2026-11-10');
  await page.getByRole('combobox', {name:'Compare', exact:true}).selectOption('sleepTime'); await settle(page);
  await page.evaluate(() => window.RaringApp.navigate('help')); await settle(page);
  await page.goBack(); await settle(page);
  await expectText(page, '2026-10-25–2026-11-10', 'custom date retention');
  await expectText(page, 'Second axis: Time asleep', 'overlay retention');
});
await journey('Intention edit, timing migration and routine self-report', 'U22', async (page, j) => {
  await j.tap('Edit'); await j.tap('Tue 17 Nov'); await j.tap('Save changes');
  await expectText(page, 'Changed to Strength', 'edit history');
  await page.evaluate(() => window.RaringApp.external('plan')); await settle(page);
  await j.tap('Add a wind-down intention'); await j.tap('Add wind-down intention');
  const st = await page.evaluate(() => window.RaringApp.state);
  if (!st.winddownIntention || Object.values(st.winddown).some(Boolean)) throw new Error('migration inferred wind-down completion');
  await page.evaluate(() => window.RaringApp.external('adapt/routines')); await settle(page);
  await page.getByLabel('Exercise name').fill('Wall sit 3 × 30 s'); await j.tap('Add to checklist'); await j.tap('Mark done today');
  await expectText(page, 'Done today (self-reported)', 'routine separate from Garmin');
});

await journey('Unknown timing never reuses old-night training advice', 'U38', async (page) => {
  await page.evaluate(() => window.RaringApp.set({clock:'late'})); await settle(page);
  await expectText(page, 'Your recorded context', 'neutral fixed presentation after midnight');
  await expectNoText(page, 'Training is an option', 'no old-night training advice');
  await page.evaluate(() => window.RaringApp.external('surfaces')); await settle(page);
  await expectText(page, 'Recorded context only', 'same fixed presentation on surfaces');
});
await journey('First commitment does not adopt old targets', 'U01', async (page) => {
  await page.evaluate(() => window.RaringApp.external('plan/commitments')); await settle(page);
  await page.getByRole('button',{name:'2',exact:true}).click(); await settle(page);
  const st = await page.evaluate(() => window.RaringApp.state.ownCommitments);
  if (st.im !== 0) throw new Error('intensity commitment silently adopted');
  await page.evaluate(() => window.RaringApp.external('progress')); await settle(page);
  await expectNoText(page, 'targets then: 3 strength', 'no pre-setup achievements');
});
await journey('Sleep above need has honest sign', 'U03', async (page) => {
  await page.evaluate(() => window.RaringApp.set({day:'2026-11-20'})); await settle(page);
  await expectText(page, '0h 03 above', 'per-night difference');
  await expectNoText(page, '-1h', 'no negative duration formatter');
});

await journey('Movement review covers a local Run intention', 'U01', async (page, j) => {
  await page.evaluate(() => window.RaringApp.set({limits:{exclude:[],note:'No deep knee bends',routines:{}}}));
  await page.getByRole('button',{name:'Plan',exact:true}).click(); await settle(page);
  await j.tap('Add to my plan'); await j.tap('Run');
  if (!(await page.locator('.sheet').textContent()).includes('Review needed')) throw new Error('movement note missing in non-strength planning');
  await j.tap('Add to plan');
  await page.getByRole('button',{name:'Today',exact:true}).click(); await settle(page);
  await expectText(page, 'Review your planned session', 'movement review before readiness');
  await j.tap('Review session'); await expectText(page, 'No deep knee bends', 'note visible on the local intention');
});
await journey('Fixed layout preserves tonight and self-report identity', 'U37', async (page) => {
  const before = await page.evaluate(() => window.RaringRules.derive(window.RaringApp.state).tonight.nightOf);
  await page.evaluate(() => window.RaringApp.set({follow:false}));
  await page.evaluate(() => window.RaringApp.external('sleep/tonight')); await settle(page);
  await expectText(page, 'Wed night → Thu morning', 'presentation independent night');
  await page.getByRole('button',{name:'I wound down (optional)',exact:true}).click(); await settle(page);
  const marks = await page.evaluate(() => window.RaringApp.state.winddown);
  if (!marks[before] || marks['2026-11-19']) throw new Error('preparation assigned to a different night');
});
await journey('Time-only edit retains activity and original day', 'U19', async (page, j) => {
  await page.evaluate(() => window.RaringApp.external('intention/i0')); await settle(page);
  await j.tap('Edit'); await page.locator('.sheet input[type=time]').fill('08:30'); await page.locator('.sheet input[type=time]').blur(); await settle(page);
  await j.tap('Save changes');
  const i = await page.evaluate(() => window.RaringApp.state.intentions.find(x=>x.id==='i0'));
  if (i.sport !== 'strength' || i.date !== '2026-11-16' || i.time !== '08:30') throw new Error(`time edit corrupted draft: ${JSON.stringify(i)}`);
});
await journey('Changing date/type clears a confirmed link', 'U22', async (page, j) => {
  await j.tap('This was my session'); await j.tap('Edit'); await j.tap('Run'); await j.tap('Tue 17 Nov'); await j.tap('Save changes');
  const i = await page.evaluate(() => window.RaringApp.state.intentions.find(x=>x.id==='i0'));
  if (i.link) throw new Error('incompatible activity link survived');
  await expectText(page, 'old activity link cleared', 'association history retained');
  await expectNoText(page, 'Linked by you:', 'no wrong-day completion display');
});

// ---------- 3. Layout budgets ----------
const VIEWS = [
  ['phone-390', {width: 390, height: 844}, 1], ['phone-360', {width: 360, height: 780}, 1],
  ['phone-390-text1.3', {width: 390, height: 844}, 1.3], ['phone-390-text2', {width: 390, height: 844}, 2],
  ['unfolded-720', {width: 720, height: 900}, 1], ['desktop-1280', {width: 1280, height: 820}, 1],
];
const TODAY_CASES = ['U01', 'U03', 'U08', 'U10', 'U13', 'U23', 'U24', 'U29', 'U30', 'U37', 'U43'];
for (const [name, viewport, scale] of VIEWS) {
  const {context, page} = await fresh(viewport);
  for (const id of TODAY_CASES) {
    await open(page, `case=${id}&text=${scale}`);
    await page.locator('.case-note button').click().catch(() => {}); await settle(page);
    const m = await page.evaluate(() => {
      const r = el => el ? el.getBoundingClientRect() : null;
      const tab = document.querySelector('.tabs').getBoundingClientRect();
      const tabTop = tab.top > innerHeight / 2 ? tab.top : innerHeight;
      const lead = r(document.querySelector('.lead h2')), act = r(document.querySelector('.lead .primary')), notice = r(document.querySelector('main .notice'));
      const facts = r(document.querySelector('.facts'));
      const wide = [...document.querySelectorAll('main *')].filter(el => { const b = el.getBoundingClientRect(); return b.width && (b.right > innerWidth + 1 || b.left < -1); }).map(el => el.className || el.tagName).slice(0, 3);
      const clipped = [...document.querySelectorAll('.fact, .fact *, .tab, .hbtn, .adapt, .lib, .chip, .tag, .primary, .secondary, .row .rl, .lead h2')].filter(el => el.offsetParent && el.scrollWidth > el.clientWidth + 1).map(el => (el.className || el.tagName) + ':' + el.textContent.trim().slice(0, 20)).slice(0, 3);
      const fonts = [...document.querySelectorAll('main *')].filter(el => el.childNodes.length && [...el.childNodes].some(n => n.nodeType === 3 && n.textContent.trim())).map(el => parseFloat(getComputedStyle(el).fontSize));
      return {height: document.documentElement.scrollHeight, vh: innerHeight, tabTop, leadBottom: lead?.bottom, actionBottom: act?.bottom, noticeBottom: notice?.bottom, leadBeforeFacts: lead && facts ? lead.top < facts.top : null,
        overflowX: document.documentElement.scrollWidth - innerWidth, wide, clipped, minFont: Math.min(...fonts)};
    });
    const aboveFold = m.actionBottom <= m.tabTop && (m.noticeBottom == null || m.noticeBottom <= m.tabTop);
    const viewports = +(m.height / m.vh).toFixed(2);
    layout.push({view: name, id, aboveFold, viewports, minFont: m.minFont, overflowX: m.overflowX});
    if (m.overflowX > 1 || m.wide.length) fail(`${name} ${id}`, `horizontal overflow ${m.overflowX}px ${m.wide.join(',')}`);
    if (m.clipped.length) fail(`${name} ${id}`, `text wider than its box: ${m.clipped.join(' | ')}`);
    if (scale === 1 && name.startsWith('phone') && !aboveFold) fail(`${name} ${id}`, `lead action or notice below the fold (${Math.round(m.actionBottom)} > ${Math.round(m.tabTop)})`);
    if (name === 'phone-390' && viewports > 2) fail(`${name} ${id}`, `Today is ${viewports} viewports tall`);
    if (m.minFont < 12 * scale - .01) fail(`${name} ${id}`, `text below ${12 * scale}px (${m.minFont}px)`);
    if (m.leadBeforeFacts === false) fail(`${name} ${id}`, 'lead is not before context in reading order');
  }
  await context.close();
}

// Detail layouts must also reflow at 360 px and large text, not only Today.
for (const scale of [1, 1.3, 2]) {
  const {context, page} = await fresh({width:360,height:780});
  await open(page, `case=U19&text=${scale}`);
  for (const r of ['plan', 'progress', 'explore', 'sleep', 'training', 'training/performance', 'strength', 'activities', 'health', 'settings', 'adapt', 'metric/hrv', 'workout/a-1203/evidence']) {
    await page.evaluate(r => window.RaringApp.external(r), r); await settle(page);
    const extra = await page.evaluate(() => document.documentElement.scrollWidth - innerWidth);
    if (extra > 1) fail(`detail ${r} text${scale}`, `horizontal overflow ${extra}px`);
  }
  await context.close();
}

// ---------- 4. Accessibility heuristics ----------
{
  const {context, page} = await fresh({width: 390, height: 844}, {reducedMotion: 'reduce'});
  const pages = ['today', 'plan', 'progress', 'explore', 'sleep', 'training', 'strength', 'activities', 'health', 'settings', 'adapt', 'workout/a-1201', 'metric/hrv', 'review/2026-W46', 'surfaces'];
  await open(page, 'case=U24');
  const anim = await page.evaluate(() => getComputedStyle(document.querySelector('.celebrate')).animationName);
  if (anim !== 'none') fail('reduced motion', `celebration animates (${anim})`);
  const celebrateText = await page.locator('.lead h2').textContent();
  if (!celebrateText.includes('Strength target reached')) fail('reduced motion', 'information lost without animation');
  for (const p of pages) {
    await page.evaluate(r => window.RaringApp.external(r), p); await settle(page);
    const unnamed = await page.evaluate(() => [...document.querySelectorAll('button, a, input, select, textarea')].filter(el => {
      const name = (el.getAttribute('aria-label') || el.textContent || el.labels?.[0]?.textContent || el.getAttribute('placeholder') || '').trim();
      return !name && el.offsetParent !== null;
    }).map(el => el.outerHTML.slice(0, 80)));
    if (unnamed.length) fail(`a11y ${p}`, `unnamed controls: ${unnamed.join(' | ')}`);
    const small = await page.evaluate(() => [...document.querySelectorAll('main button, .tabs button')].filter(el => el.offsetParent && !el.classList.contains('inline') && el.getBoundingClientRect().height < 40).map(el => el.textContent.trim().slice(0, 30)));
    if (small.length) fail(`a11y ${p}`, `targets under 40px tall: ${small.slice(0, 4).join(' | ')}`);
  }
  await open(page, 'case=U13&text=1');
  await page.locator('.case-note button').click(); await settle(page);
  await page.evaluate(() => document.activeElement.blur());
  const order = [];
  for (let i = 0; i < 6; i++) { await page.keyboard.press('Tab'); order.push(await page.evaluate(() => (document.activeElement.getAttribute('aria-label') || document.activeElement.textContent).trim().slice(0, 24))); }
  notes.push(`keyboard order on Today: ${order.join(' → ')}`);
  if (order.indexOf('Explore') < 0 || order.indexOf('Adapt today') < 0) fail('keyboard', `Explore/Adapt not early in tab order: ${order.join(' → ')}`);
  await page.getByRole('button', {name: /^Adapt today/}).focus(); await page.keyboard.press('Enter'); await settle(page);
  const focused = await page.evaluate(() => document.activeElement.closest('.sheet') != null);
  if (!focused) fail('keyboard', 'focus did not move into the Adapt sheet');
  await page.keyboard.press('Escape'); await settle(page);
  if (await page.locator('.sheet').count()) fail('keyboard', 'Escape did not close the sheet');
  await context.close();
}

// ---------- 5. Screenshots ----------
if (shots) {
  await mkdir(outDir, {recursive: true});
  const list = [
    ['phone', 'U01', ''], ['phone', 'U03', ''], ['phone', 'U10', ''], ['phone', 'U13', ''], ['phone', 'U24', ''], ['phone', 'U29', ''], ['phone', 'U30', ''], ['phone', 'U37', ''], ['phone', 'U08', ''],
    ['phone', 'U19', 'full'], ['phone', 'U26', 'full'], ['phone', 'U27', 'full'], ['phone', 'U50', 'full'], ['phone', 'U42', 'full'], ['phone', 'U15', 'full'], ['phone', 'U31', 'full'], ['phone', 'U53', ''], ['phone', 'U53', 'journal-open'], ['phone', 'U53', 'opt-out'], ['phone', 'U44', ''],
    ['phone', 'U13', 'sheet:adapt'], ['phone', 'U11', 'sheet:add'], ['phone', 'U13', 'sheet:why'], ['phone', 'U28', 'surfaces'],
    ['phone-text1.3', 'U13', ''], ['phone-text2', 'U13', 'full'], ['phone-360', 'U30', ''], ['unfolded', 'U13', ''], ['desktop', 'U13', ''], ['desktop', 'U19', 'plan'],
  ];
  const sizes = {phone: [390, 844, 1], 'phone-text1.3': [390, 844, 1.3], 'phone-text2': [390, 844, 2], 'phone-360': [360, 780, 1], unfolded: [720, 900, 1], desktop: [1280, 820, 1]};
  const {context, page} = await fresh();
  for (const [size, id, mode] of list) {
    const [w, hgt, scale] = sizes[size];
    await page.setViewportSize({width: w, height: hgt});
    await open(page, `case=${id}&text=${scale}`);
    await page.locator('.case-note button').click().catch(() => {}); await settle(page);
    // Clicking the case banner scrolls upward; restore the target section before capture.
    await page.evaluate(() => window.RaringApp.external(location.hash)); await settle(page);
    if (mode === 'journal-open') { await page.getByRole('button', {name:'Open journal',exact:true}).click(); await settle(page); }
    if (mode === 'opt-out') { await page.evaluate(() => { const s = window.RaringApp.state; window.RaringApp.set({journal: {...s.journal, consent:false, open:false}}); }); await settle(page); }
    if (mode.startsWith('sheet:')) { await page.evaluate(n => window.RaringApp.openSheet(n), mode.slice(6)); await settle(page); }
    if (mode === 'surfaces') { await page.evaluate(() => { window.RaringApp.set({rest: {day: '2026-11-20'}, permission: 'granted', channels: {morning: true, weekly: true, status: true}}); window.RaringApp.external('surfaces'); }); await settle(page); }
    if (mode === 'plan') { await page.evaluate(() => window.RaringApp.external('plan')); await settle(page); }
    if (mode === 'full' || mode === 'surfaces' || mode === 'plan') { const full = await page.evaluate(() => document.documentElement.scrollHeight); if (full > hgt) await page.setViewportSize({width: w, height: full}); }
    const r = (await route(page)).replace(/\//g, '-');
    await page.screenshot({path: resolve(outDir, `${size}-${id}-${mode ? mode.replace(':', '-') + '-' : ''}${r}.png`)});
  }
  await context.close();
}

await browser.close();
const fold = layout.filter(l => l.view === 'phone-390');
const summary = {failures, notes, journeys, layout: {phone390: fold, other: layout.filter(l => l.view !== 'phone-390')}};
await mkdir(resolve(here, '../../../.data'), {recursive: true}).catch(() => {});
await writeFile(resolve(here, '../../../.data/experience-check.json'), JSON.stringify(summary, null, 2)).catch(() => {});
console.log('Journeys (deliberate taps):');
for (const j of journeys) console.log(`  ${j.name}: ${j.taps} [budget ${j.budget}] ${j.path}${j.extra ? ` (${j.extra})` : ''}`);
console.log('Today at 390×844: ' + fold.map(l => `${l.id} ${l.viewports}vp${l.aboveFold ? '' : ' BELOW-FOLD'}`).join(', '));
const large = layout.filter(l => l.view.includes('text'));
console.log('Large text (lead action above fold?): ' + large.map(l => `${l.view.replace('phone-390-', '')}/${l.id}:${l.aboveFold ? 'yes' : 'no'}`).join(' '));
for (const n of notes) console.log(`Note: ${n}`);
if (failures.length) { console.error(`\n${failures.length} failure(s):\n` + failures.map(f => `  ${f}`).join('\n')); process.exit(1); }
console.log('\nAll prototype checks passed. This is not usability, device or real-data evidence.');
