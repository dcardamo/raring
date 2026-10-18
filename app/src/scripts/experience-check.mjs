// Browser checks for the unshipped experience prototype: U-case states, counted
// journeys, privacy, legacy links, layout budgets and accessibility heuristics.
//   npm run test:experience [-- --shots]
// EXPERIENCE_URL checks an already served build instead of a dev server.
// A prototype pass is not usability, device or real-data evidence.
import {mkdir, writeFile} from 'node:fs/promises';
import {resolve} from 'node:path';
import {createServer} from 'vite';
import {chromium} from 'playwright';

const shotsOnly = process.argv.includes('--shots-only');
const shots = shotsOnly || process.argv.includes('--shots');
const outDir = resolve('../design/mockup/experience/shots');
const compareDir = resolve('experience-public/compare');
const failures = [], notes = [], journeys = [], layout = [];
const fail = (what, detail) => failures.push(`${what}: ${detail}`);
let server = null, base = process.env.EXPERIENCE_URL;
if (!base) {
 server = await createServer({configFile: resolve('vite.experience.config.ts'), server: {host: '127.0.0.1', port: 0, open: false}, logLevel: 'error'});
 await server.listen();
 base = `http://127.0.0.1:${server.httpServer.address().port}/experience.html`;
}
const origin = new URL(base).origin;
const browser = await chromium.launch({headless: true, executablePath: process.env.CHROMIUM ?? '/run/current-system/sw/bin/chromium', args: ['--no-sandbox']});

async function fresh(viewport = {width: 390, height: 844}, opts = {}) {
 const context = await browser.newContext({viewport, deviceScaleFactor: opts.dpr ?? 1, reducedMotion: opts.reducedMotion ?? 'no-preference', hasTouch: !!opts.touch, isMobile: !!opts.touch});
 // Nothing but the prototype's own origin: no remote fonts, tiles, analytics or data.
 await context.route('**/*', route => { const u = route.request().url(); if (u.startsWith(origin) || u.startsWith('data:')) return route.continue(); fail('network', u); return route.abort(); });
 const page = await context.newPage();
 page.setDefaultTimeout(7000);
 page.on('pageerror', e => fail('pageerror', e.message));
 return {context, page};
}
const settle = page => page.waitForTimeout(120);
const open = async (page, query, hash = '') => { await page.goto(`${base}?${query}${hash}`); await page.waitForSelector('main h1'); await settle(page); };
const route = page => page.evaluate(() => location.hash.slice(1));
const text = page => page.evaluate(() => document.querySelector('main').textContent);
const ext = (page, r) => page.evaluate(r => window.RaringApp.external(r), r).then(() => settle(page));
const set = (page, p) => page.evaluate(p => window.RaringApp.set(p), p).then(() => settle(page));

// Every deliberate tap is counted; typing is not (design section 15).
function counter(page, name) {
 const path = [];
 const tap = async (label, scope = 'main, .sheet, .tabs') => {
  const loc = page.locator(scope).getByRole('button', {name: label, exact: typeof label === 'string'}).first();
  await loc.click();
  path.push(String(label));
  await settle(page);
 };
 return {tap, path, done: (want, budget, extra = '') => { if (typeof budget === 'number' && path.length > budget) fail(name, `${path.length} taps exceeds ${budget}`); journeys.push({name, taps: path.length, budget, path: path.join(' → '), route: want, extra}); }};
}
async function journey(name, caseId, fn, viewport, opts) {
 const {context, page} = await fresh(viewport, opts);
 await open(page, `case=${caseId}`);
 await page.locator('.case-note button').click().catch(() => {});
 try { await fn(page, counter(page, name)); console.log('PASS', name); } catch (e) { fail(name, e.message.split('\n')[0]); console.log('FAIL', name, e.message.split('\n')[0]); }
 await context.close();
}
const expectRoute = async (page, want, label) => { const r = await route(page); if (r !== want) throw new Error(`${label}: expected #${want}, got #${r}`); };
const expectText = async (page, want, label) => { const t = await text(page); if (!t.includes(want)) throw new Error(`${label}: missing “${want}”`); };
const expectNoText = async (page, want, label) => { const t = await text(page); if (t.includes(want)) throw new Error(`${label}: unexpected “${want}”`); };

if (!shotsOnly) {
// ---------- 1. U-case states ----------
{
 const {context, page} = await fresh();
 await open(page, '');
 const cases = await page.evaluate(() => window.RaringCases);
 for (const c of cases) {
  await open(page, `case=${c.id}`);
  await page.locator('.case-note button').click().catch(() => {});
  const t = await text(page);
  for (const e of c.expect) if (!t.includes(e)) fail(c.id, `missing “${e}” on #${await route(page)}`);
  for (const f of c.forbid) if (t.toLowerCase().includes(f.toLowerCase())) fail(c.id, `forbidden “${f}” on #${await route(page)}`);
 }
 notes.push(`${cases.length} U-case states loaded`);
 await context.close();
}

// ---------- 2. Journeys ----------
await journey('U03 Sleep-only: last night from Today', 'U03', async (page, j) => {
 await j.tap('Last night’s sleep');
 await expectRoute(page, 'sleep', 'U03'); await expectText(page, 'Night ending Wed 18 Nov', 'U03');
 j.done('sleep', 1);
 await page.locator('.back').click(); await settle(page); await expectRoute(page, 'today', 'U03 Back');
});
await journey('U04 Tonight from Today', 'U03', async (page, j) => {
 await j.tap(/^Tonight · wind down/);
 await expectRoute(page, 'sleep/tonight', 'U04'); await expectText(page, 'Garmin plan', 'U04');
 await page.waitForTimeout(300);
 const inView = await page.evaluate(() => { const r = document.getElementById('sec-tonight').getBoundingClientRect(); return r.top >= -2 && r.top < innerHeight / 2; });
 if (!inView) throw new Error('U04: Tonight section not brought into view');
 j.done('sleep/tonight', 1, 'opens the Tonight section, not the top of Sleep');
});
await journey('U05 Recorded walk: picture and numbers on Today, then its workout', 'U05', async (page, j) => {
 await expectText(page, '2.2', 'U05 distance inline'); await expectText(page, '24:00', 'U05 time inline');
 if (!await page.locator('.hero .atile .routehero').count()) throw new Error('U05: walk has no route picture on Today');
 await page.locator('.hero .atile').click(); j.path.push('Walk card'); await settle(page);
 await expectRoute(page, 'workout/a-1204', 'U05');
 if (!await page.locator('.wo-pic .routehero').count()) throw new Error('U05: workout detail lost its map');
 await expectText(page, 'Time in zones', 'U05 zones'); await expectText(page, 'Splits', 'U05 splits'); await expectText(page, 'Heart rate', 'U05 HR chart');
 j.done('workout/a-1204', 1);
});
await journey('U11 Add a local intention', 'U11', async (page, j) => {
 await j.tap('Choose a session'); await j.tap(/^Walk/, '.sheet'); await j.tap('Place in my week');
 await expectRoute(page, 'today', 'U11');
 const s = await page.evaluate(() => window.RaringApp.state.intentions);
 if (s.length !== 1 || s[0].date !== '2026-11-20' || s[0].time) throw new Error(`U11: unexpected intention ${JSON.stringify(s)}`);
 if ((await page.locator('.hero .sname').innerText()).trim() !== 'Walk') throw new Error('U11: Today does not show the chosen session');
 j.done('today', 3, 'sport, then today preselected; time optional');
});
await journey('U28 Rest today, offline, resume', 'U28', async (page, j) => {
 await j.tap(/^Adapt today/); await j.tap('Rest today');
 await expectText(page, 'Rest day', 'U28 lead');
 j.done('today', 2);
 await set(page, {offline: true});
 await page.reload(); await page.waitForSelector('main h1'); await settle(page);
 await expectText(page, 'Rest day', 'U28 after reload offline');
 await expectText(page, 'Offline · saved data', 'U28 offline label');
 await page.locator('.tabs').getByRole('button', {name: 'Plan', exact: true}).click(); await settle(page);
 await expectText(page, 'Rest day · through tonight', 'U28 Plan reflects rest');
 await set(page, {permission: 'granted'}); await ext(page, 'surfaces');
 await expectText(page, 'Rest day · no training call', 'U28 notification suppressed');
 await set(page, {clock: 'late'}); await ext(page, 'today');
 if ((await page.locator('.adapt').innerText()).trim() !== 'Rest day') throw new Error('U28: rest did not carry through the night');
 await set(page, {day: '2026-11-21', clock: 'morning'});
 if ((await page.locator('.adapt').innerText()).trim() !== 'Adapt today') throw new Error('U28: rest should end in the next waking day');
});
await journey('U29 Pause then explicit resume', 'U12', async (page, j) => {
 await set(page, {rest: null, day: '2026-11-19'});
 await j.tap(/^Adapt today/); await j.tap(/^Pause training/); await j.tap('Pause from today');
 j.done('adapt/pause', 3, 'longer-lived circumstance: one extra deliberate step');
 await set(page, {day: '2026-11-20'}); await ext(page, 'today');
 await expectText(page, 'Training is paused', 'U29 high readiness stays paused');
 await expectNoText(page, 'Plan a session', 'U29');
 await ext(page, 'plan');
 await expectText(page, 'Training paused since Thu', 'U26 mid-week pause label');
 await set(page, {day: '2026-11-22'}); await ext(page, 'today');
 const r = counter(page, 'U29 Resume');
 await r.tap(/^Paused/); await r.tap(/^Pause training/); await r.tap('Resume training…'); await r.tap('Resume as they are');
 r.done('adapt/pause', 4, 'shows commitments for review; no catch-up');
 await ext(page, 'plan');
 await expectText(page, 'Paused Thu–Sun', 'U26 pause interval retained after resume');
});
await journey('U30 Exclude an activity and add a movement note', 'U19', async (page, j) => {
 await set(page, {day: '2026-11-19'}); await ext(page, 'today');
 await j.tap(/^Adapt today/); await j.tap(/^Avoid activities or movements/); await j.tap('Run');
 await page.locator('textarea').fill('No deep knee bends'); await page.locator('textarea').blur(); await settle(page);
 j.done('adapt/limits', 3, 'plus typing the note');
 await ext(page, 'today');
 await expectText(page, 'Conflicts with your limits', 'U30 Today');
 await ext(page, 'plan');
 await expectText(page, 'Conflicts with your limits', 'U30 Plan agenda');
 await set(page, {day: '2026-11-21'}); await ext(page, 'plan');
 await expectText(page, 'Review needed', 'U31 Plan');
 await set(page, {permission: 'granted'}); await ext(page, 'surfaces');
 if ((await text(page)).includes('knee')) throw new Error('U30: movement note leaked into a status surface');
});
await journey('U22 Link a recorded activity to an intention', 'U15', async (page, j) => {
 await ext(page, 'plan');
 await page.locator('.day-cells .dc').first().click(); j.path.push('Mon'); await settle(page);
 await j.tap(/^Your plan.*Strength.*07:00/); await j.tap('This was my session');
 await expectText(page, 'Linked by you', 'U22');
 j.done('intention/i0', 3, 'from Plan, choosing Monday first');
});
await journey('U22b Intention left unlinked is not missed', 'U15', async (page) => {
 await set(page, {day: '2026-11-17'}); await ext(page, 'intention/i0');
 await expectText(page, 'Already counts', 'U22b'); await expectNoText(page, 'missed', 'U22b');
});
// Five focused tabs replace Explore: every area is one tab, or a tab and one tap.
const tabTo = (page, j, name) => page.locator('.tabs').getByRole('button', {name, exact: true}).click().then(() => { j.path.push(`${name} tab`); return settle(page); });
const AREAS = [
 ['Sleep', 'sleep', 'health', async (page, j) => { await tabTo(page, j, 'Health'); await j.tap(/^Last night/); }],
 ['Readiness', 'readiness', 'health', async (page, j) => { await tabTo(page, j, 'Health'); await j.tap(/^Readiness/); }],
 ['Training', 'training', null, async (page, j) => { await tabTo(page, j, 'Training'); }],
 ['Strength', 'training', null, async (page, j) => { await tabTo(page, j, 'Training'); await page.getByRole('tab', {name: 'Strength', exact: true}).click(); j.path.push('Strength'); await settle(page); }],
 ['Activities', 'training', null, async (page, j) => { await tabTo(page, j, 'Training'); await page.getByRole('tab', {name: 'Activities', exact: true}).click(); j.path.push('Activities'); await settle(page); }],
 ['Health', 'health', null, async (page, j) => { await tabTo(page, j, 'Health'); }],
];
for (const [root, label] of [['today', 'Today'], ['plan', 'Plan'], ['progress', 'Progress']]) {
 for (const [area, want, back, go] of AREAS) {
  await journey(`Discover ${area} from ${label}`, 'U19', async (page, j) => {
   await ext(page, root);
   await go(page, j);
   await expectRoute(page, want, `${area} from ${label}`);
   if (area === 'Strength' && !await page.locator('main').getByText('Heaviest set each session', {exact: false}).count()) throw new Error('Strength section lacks the shipped history');
   if (area === 'Activities' && !await page.locator('main .atile').count()) throw new Error('Activities section is empty');
   j.done(want, 2);
   if (back) { await page.locator('.back').click(); await settle(page); await expectRoute(page, back, `${area} Back`); }
  });
 }
}
await journey('Explore is retired; old links land in Training', 'U19', async (page) => {
 await ext(page, 'explore'); await expectRoute(page, 'training', 'explore alias');
 if (await page.locator('.tabs').getByRole('button', {name: 'Explore'}).count()) throw new Error('Explore tab still present');
 const tabs = await page.locator('.tabs button').allInnerTexts();
 if (tabs.map(t => t.trim()).join(',') !== 'Today,Plan,Training,Health,Progress') throw new Error(`tabs: ${tabs.join(',')}`);
});
await journey('U50 Rare metric by search', 'U19', async (page, j) => {
 await ext(page, 'progress');
 await j.tap('Search metrics');
 await page.getByRole('searchbox').fill('hill'); await settle(page);
 await j.tap(/^Hill score/, '.sheet');
 await expectRoute(page, 'metric/hill', 'U50 search'); j.done('metric/hill', 2, 'header search sheet, plus typing “hill”');
 await page.locator('.back').click(); await settle(page); await expectRoute(page, 'progress', 'U50 search Back returns to caller');
});
await journey('U50 Rare metric by browsing', 'U19', async (page, j) => {
 await tabTo(page, j, 'Training'); await j.tap(/Hill score/);
 await expectRoute(page, 'metric/hill', 'U50 browse'); j.done('metric/hill', 2);
});
await journey('U50 Gear', 'U19', async (page, j) => {
 await tabTo(page, j, 'Training'); await j.tap(/^Gear/);
 await expectRoute(page, 'gear', 'U50 gear'); j.done('gear', 2);
});
await journey('U50 Comparison', 'U19', async (page, j) => {
 await tabTo(page, j, 'Progress'); await j.tap('All trends'); await j.tap(/^VO₂ max/);
 await expectRoute(page, 'metric/vo2max', 'U50 compare'); await expectText(page, 'VO₂ max', 'U50 metric');
 j.done('metric/vo2max', 3, 'shipped metric detail with ranges and previous period');
});
await journey('Back keeps filter and scroll', 'U19', async (page, j) => {
 await tabTo(page, j, 'Training'); await page.getByRole('tab', {name: 'Activities', exact: true}).click(); j.path.push('Activities'); await settle(page);
 await page.locator('.filt').getByRole('button', {name: 'Strength'}).click(); j.path.push('Strength'); await settle(page);
 await page.locator('.scroll').evaluate(el => { el.scrollTop = 500; }); await settle(page);
 const before = await page.locator('.scroll').evaluate(el => el.scrollTop);
 await page.locator('.atile').nth(1).click(); await settle(page);
 await page.goBack(); await page.waitForTimeout(400);
 const pressed = await page.locator('.filt .chipb[aria-pressed=true]').first().innerText();
 const after = await page.locator('.scroll').evaluate(el => el.scrollTop);
 if (pressed.trim() !== 'Strength') throw new Error(`filter lost on browser Back: ${pressed}`);
 if (Math.abs(after - before) > 2) throw new Error(`scroll not restored: ${before} → ${after}`);
 j.done('training', 'n/a', `browser Back restored filter and scroll ${after}px`);
}, {width: 390, height: 640});
await journey('Not recorded → Reschedule moves only the local intention', 'U41', async (page, j) => {
 const card = page.locator('.unfinished').filter({hasText: 'Not recorded'}).first();
 await card.getByRole('button', {name: 'Reschedule'}).click(); j.path.push('Reschedule'); await settle(page);
 await page.getByLabel('New date').fill('2026-11-21'); await j.tap('Move my session');
 const st = await page.evaluate(() => window.RaringApp.state);
 const moved = st.intentions.find(i => i.date === '2026-11-21' && i.sport === 'strength');
 if (!moved || moved.status && moved.status !== 'open' || !moved.history.at(-1)[1].includes('Wed 18 Nov')) throw new Error(`reschedule lost its original date: ${JSON.stringify(moved)}`);
 if (st.intentions.some(i => i.status === 'missed')) throw new Error('reschedule marked something missed');
 await expectNoText(page, 'issed', 'no missed verdict'); await expectNoText(page, 'Not recorded', 'nothing left unfinished');
 j.done('plan', 2, 'plus choosing the date');
 await ext(page, `intention/${moved.id}`); await expectText(page, 'Moved from Wed 18 Nov', 'history keeps the original date');
});
await journey('Possible match needs review, then counts once', 'U41', async (page, j) => {
 const amounts = () => page.locator('.goal').first().locator('.amounts b').allInnerTexts();
 const before = await amounts();
 await expectText(page, 'Needs review', 'review flagged');
 await page.locator('.unfinished.uf-review').getByRole('button', {name: 'This was my session'}).click(); j.path.push('This was my session'); await settle(page);
 const after = await amounts();
 if (before[0] !== after[0] || before[1] !== after[1]) throw new Error(`linking changed recorded/planned: ${before} → ${after}`);
 await expectNoText(page, 'Needs review', 'review resolved');
 j.done('plan', 1, `strength recorded ${after[0]}, planned ${after[1]} before and after`);
});
await journey('Garmin Coach is read-only and can count toward a local goal', 'U41', async (page, j) => {
 await j.tap(/^Garmin Coach/); await expectRoute(page, 'plan/garmin', 'Coach detail');
 await expectText(page, 'Completion unknown', 'Coach completion'); await expectText(page, 'Garmin Connect', 'managed elsewhere');
 const before = await page.evaluate(() => window.RaringApp.state.intentions.length);
 await page.getByRole('button', {name: 'Count toward my goals'}).first().click(); j.path.push('Count toward my goals'); await settle(page);
 await j.tap('Place in my week');
 const st = await page.evaluate(() => window.RaringApp.state.intentions);
 if (st.length !== before + 1 || !st.at(-1).garminId) throw new Error('Garmin session not counted as a local allocation');
 await expectText(page, 'Counts toward your goals', 'local overlay shown');
 j.done('plan/garmin', 3);
});
// ---------- P3a: the weekly loop, rings and Today ----------
await journey('Plan your week: review last week, choose goals, place, done', 'U15', async (page, j) => {
 await set(page, {day: '2026-11-16', clock: 'day', intentions: []}); await ext(page, 'training');
 if (!await page.locator('.pp').count()) throw new Error('prompt missing on Training at the start of the week');
 await j.tap('Start');
 if (!(await page.locator('.sheet').textContent()).includes('9–15 Nov')) throw new Error('review does not show last week');
 const sheet = await page.locator('.sheet').innerText();
 if (!/Met|Partly met|Not recorded/.test(sheet)) throw new Error('review lacks outcomes');
 if (!(await page.locator('.sheet').textContent()).includes('Suggested for this week')) throw new Error('no proposals');
 if (await page.locator('.sheet .prop input:checked').count()) throw new Error('a proposal was pre-accepted');
 await j.tap('Next: goals'); await page.getByRole('button', {name: 'More Zone 2'}).click(); await page.getByRole('button', {name: 'More Zone 2'}).click(); j.path.push('+ Zone 2 ×2');
 await j.tap('Next: place them');
 await page.locator('.sheet .bl').first().getByRole('button', {name: 'Wed'}).click(); j.path.push('Wed'); await page.waitForTimeout(150);
 await j.tap('Next'); await j.tap('Done');
 const st = await page.evaluate(() => window.RaringApp.state);
 if (st.planWeeks?.['2026-W47'] !== 'done') throw new Error('week not marked planned');
 if (!st.intentions.some(i => i.date === '2026-11-18')) throw new Error('nothing placed on Wednesday');
 if (st.goalVersions?.at(-1)?.targets.zone2 !== 20) throw new Error(`zone2 goal ${st.goalVersions?.at(-1)?.targets.zone2}`);
 if (await page.locator('.pp').count()) throw new Error('prompt still shown after planning');
 j.done('training', 9, 'review, goals by steppers, one placement');
});
await journey('Plan prompt: not now, stop reminders, never at wind-down', 'U15', async (page) => {
 await set(page, {day: '2026-11-16', clock: 'winddown'}); await ext(page, 'today');
 if (await page.locator('.pp').count()) throw new Error('prompt during wind-down');
 await set(page, {clock: 'day'}); await ext(page, 'today');
 await page.locator('.pp').getByRole('button', {name: 'Not now'}).click(); await settle(page);
 if (await page.locator('.pp').count()) throw new Error('Not now did not hide it');
 await set(page, {day: '2026-11-23', planWeeks: {}}); await ext(page, 'progress');
 await page.locator('.pp').getByRole('button', {name: 'Stop weekly reminders'}).click(); await settle(page);
 if (await page.evaluate(() => window.RaringApp.state.planReminder) !== false) throw new Error('reminder not turned off');
 await ext(page, 'settings'); await expectText(page, 'Weekly planning reminder', 'setting in Settings');
});
await journey('Backlog: do it today from Today', 'U13', async (page, j) => {
 await j.tap('Do it today');
 const st = await page.evaluate(() => window.RaringApp.state.intentions.filter(i => i.date === '2026-11-18' && i.allocation?.im));
 if (st.length !== 1 || st[0].allocation.im > 30) throw new Error(`backlog item not taken for today: ${JSON.stringify(st)}`);
 j.done('today', 1);
});
await journey('Move a planned session to a future day', 'U13', async (page, j) => {
 await ext(page, 'plan'); await j.tap(/^Move Strength/); await j.tap('Fri 20', '.sheet');
 const i = await page.evaluate(() => window.RaringApp.state.intentions.find(x => x.id === 'i1'));
 if (i.date !== '2026-11-20' || !i.history.at(-1)[1].includes('Wed 18 Nov')) throw new Error(`move failed ${JSON.stringify(i)}`);
 j.done('plan', 2);
});
await journey('Ill: one tap pauses rings and backlog, no catch-up', 'U13', async (page, j) => {
 await ext(page, 'progress'); await j.tap(/^Adapt today/); await j.tap('I’m ill');
 if ((await page.locator('.adapt').innerText()).trim() !== 'Ill') throw new Error('header does not show Ill');
 await ext(page, 'plan'); await expectText(page, 'Backlog paused', 'backlog paused'); await expectText(page, 'no catch-up', 'no catch-up');
 const wed = await page.locator('.day-cells .dc').nth(2).getAttribute('aria-label');
 if (!wed.includes('ill')) throw new Error(`day cell not marked ill: ${wed}`);
 j.done('plan', 2, 'from any screen');
});
await journey('Rings describe planned and recorded per day', 'U13', async (page) => {
 await ext(page, 'plan');
 const labels = await page.locator('.day-cells .dc').evaluateAll(els => els.map(e => e.getAttribute('aria-label')));
 if (!labels[0].includes('Strength 1 of 1 planned')) throw new Error(`Mon: ${labels[0]}`);
 if (!labels[2].includes('Strength 0 of 1 planned')) throw new Error(`Wed: ${labels[2]}`);
 if (labels[4].includes('Strength')) throw new Error(`Fri draws a ring with nothing planned: ${labels[4]}`);
 if (!labels[3].includes('Garmin session')) throw new Error(`Thu Garmin item missing: ${labels[3]}`);
 if (await page.locator('.ws .nr').count() !== 1) throw new Error('no week ring');
});
await journey('Today: effort is not a control; HR row; nap Body Battery', 'U13', async (page) => {
 if (await page.locator('.effort button').count()) throw new Error('effort scale has buttons');
 if (!(await page.locator('.effort').getAttribute('aria-label')).startsWith('Raring effort: easy')) throw new Error('effort label');
 if (!await page.locator('.day-rows').getByRole('button', {name: /^Heart rate/}).count()) throw new Error('HR row missing');
 await expectText(page, 'Body Battery +', 'nap Body Battery change');
 if (await page.locator('.day-rows').count() !== 1) throw new Error('Your day duplicated');
});
await journey('Send to Garmin is visible but not available', 'U13', async (page) => {
 await ext(page, 'intention/i1');
 if (!await page.getByRole('button', {name: /Send to Garmin/}).isDisabled()) throw new Error('Send to Garmin is enabled');
 await expectText(page, 'Not available yet', 'labelled');
});
await journey('Race goal: countdown, prediction and weeks', 'U13', async (page, j) => {
 await ext(page, 'plan'); await j.tap(/^Winter 10K, 24 days/);
 await expectRoute(page, 'plan/race/event', 'race detail');
 await expectText(page, 'prediction', 'Garmin prediction'); await expectText(page, 'Your weeks toward it', 'weeks');
 j.done('plan/race/event', 1);
});
await journey('U53 Journal consent, open, leave, background, opt out', 'U52', async (page, j) => {
 await j.tap('Allow local journal reads'); await j.tap('Allow journal reads'); await j.tap('Open journal');
 await expectText(page, 'Caffeine', 'U53 open'); j.done('health/journal', 3, 'consent sheet then explicit open');
 await page.locator('.back').click(); await settle(page);
 await expectNoText(page, 'Caffeine', 'U53 hidden after leaving');
 await page.evaluate(() => window.RaringApp.navigate('health/journal')); await settle(page);
 await expectNoText(page, 'Caffeine', 'U53 still hidden on return');
 await page.getByRole('button', {name: 'Open journal'}).click(); await settle(page);
 await expectText(page, 'Caffeine', 'U53 reopened');
 await page.evaluate(() => { Object.defineProperty(document, 'hidden', {configurable: true, value: true}); document.dispatchEvent(new Event('visibilitychange')); }); await settle(page);
 await expectNoText(page, 'Caffeine', 'U53 hidden after background');
 await ext(page, 'today'); await expectNoText(page, 'Caffeine', 'U52 Today never shows journal');
 await ext(page, 'progress'); await expectNoText(page, 'Caffeine', 'U52 Progress never shows journal');
 await ext(page, 'settings');
 await page.locator('#sec-journal input[type=checkbox]').uncheck(); await settle(page);
 await expectText(page, 'Downloaded logs remain on this phone', 'U53 opt-out disclosure in Settings');
 await ext(page, 'health/journal');
 await expectText(page, 'Downloaded journal days stay on this phone', 'U53 retention after opt-out');
 await expectNoText(page, 'Caffeine', 'U53 entries hidden after opt-out');
});
await journey('U56 A choice survives sync and resume; new notice is distinct', 'U56', async (page, j) => {
 await j.tap('Keep');
 const kept = async label => { if ((await page.locator('.hero .keep').innerText()).trim() !== 'Kept') throw new Error(`${label}: Keep not retained`); };
 await kept('U56'); j.done('today', 1);
 await page.evaluate(() => window.RaringApp.sync()); await settle(page);
 await page.reload(); await page.waitForSelector('main h1'); await settle(page);
 await kept('U56 after sync + reload');
 await page.evaluate(() => window.RaringApp.sync('exception')); await settle(page);
 await expectText(page, 'Two overnight readings', 'U56 new exception');
 await kept('U56 still kept');
});
await journey('U64 Settings, export disclosure, sign out', 'U64', async (page, j) => {
 await j.tap('Settings'); await expectText(page, 'circumstances and notes, routine checklist and self-reports', 'U64 export');
 await j.tap('Sign out and erase…');
 const sheet = await page.locator('.sheet').innerText();
 if (!sheet.includes('Rest, pauses, limits and notes')) throw new Error('U64: erase disclosure misses circumstances');
 j.done('settings', 2, 'export disclosure visible on Settings');
});
await journey('U58 Old links and Back', 'U58', async (page) => {
 const map = await page.evaluate(() => Object.keys(window.RaringApp.LEGACY).map(from => [from, window.RaringApp.canonical(from).route, window.RaringApp.defaultTrail(window.RaringApp.canonical(from).route)]));
 for (const [from, to, trail] of map) {
  await page.evaluate(h => { location.hash = h; }, from); await page.waitForTimeout(220);
  const r = await route(page);
  if (r !== to) { fail('U58', `#${from} opened #${r}, expected #${to}`); continue; }
  for (let i = trail.length - 2; i >= 0; i--) {
   await page.goBack(); await page.waitForTimeout(180);
   const b = await route(page);
   if (b !== trail[i]) { fail('U58', `#${from}: Back reached #${b}, expected #${trail[i]}`); break; }
  }
 }
 await page.evaluate(() => { location.hash = 'not-a-page'; }); await page.waitForTimeout(220);
 const note = await page.locator('.case-note').innerText().catch(() => '');
 if (!note.includes('isn’t a Raring page')) fail('U58', 'invalid link opened Today silently');
 await page.evaluate(()=>window.RaringApp.external('workout/a-1203/evidence')); await settle(page);
 await expectRoute(page,'workout/a-1203','v1 workout evidence alias');
 await page.evaluate(()=>window.RaringApp.external('strength/plank')); await settle(page);
 await expectRoute(page,'strength/plank','v1 bodyweight exercise link');
 notes.push(`${map.length} legacy links mapped with Back paths, plus v1 workout evidence/bodyweight exercise links`);
});
await journey('U58 Status surface tap', 'U01', async (page, j) => {
 await set(page, {permission: 'granted'}); await ext(page, 'surfaces');
 await j.tap(/^Morning readiness/); await expectRoute(page, 'readiness', 'U58 surface');
 await page.goBack(); await settle(page); await expectRoute(page, 'today', 'U58 surface Back');
 j.done('readiness', 1, 'cold link; Back → Today');
});
await journey('Interrupted week walkthrough', 'U15', async (page) => {
 const at = async (day, clock, where = 'today') => { await set(page, {day, clock}); await ext(page, where); };
 await at('2026-11-16', 'day'); await expectText(page, '1 of 2 strength this week', 'Mon recognition');
 await at('2026-11-17', 'evening', 'sleep/tonight');
 await page.getByRole('button', {name: 'I wound down', exact: true}).click(); await settle(page);
 await at('2026-11-18', 'morning', 'sleep'); await expectText(page, 'Wound down · self-reported', 'Wed preparation separate'); await expectText(page, '5:10', 'Wed imperfect night');
 await at('2026-11-18', 'day'); await expectText(page, 'An easier day may fit', 'Wed conflict');
 await page.locator('.hero .session-open').click(); await settle(page); await expectRoute(page, 'scheduled/g-int', 'Wed Garmin session');
 await page.getByRole('button', {name: 'Rest today'}).click(); await settle(page);
 await at('2026-11-18', 'day'); await expectText(page, 'Intervals 6 × 3 min', 'Wed rest keeps schedule as context'); await expectText(page, 'Rest day', 'Wed rest');
 await at('2026-11-18', 'late'); if ((await page.locator('.adapt').innerText()).trim() !== 'Rest day') fail('week', 'rest did not carry past midnight');
 await at('2026-11-19', 'day'); if ((await page.locator('.adapt').innerText()).trim() !== 'Adapt today') fail('week', 'rest did not end on Thursday');
 await ext(page, 'adapt/pause'); await page.getByRole('button', {name: 'Pause from today'}).click(); await settle(page);
 await at('2026-11-20', 'day'); await expectText(page, 'Training is paused', 'Fri pause with readiness 84');
 await at('2026-11-22', 'day', 'adapt/pause'); await page.getByRole('button', {name: 'Resume training…'}).click(); await settle(page);
 await page.getByRole('button', {name: 'Resume as they are'}).click(); await settle(page);
 await at('2026-11-22', 'day', 'review/2026-W47'); await expectNoText(page, 'Reached', 'no retroactive award');
 await at('2026-11-22', 'day', 'review/2026-W46'); await expectText(page, 'targets then: 3 strength · 150 min', 'old version honest');
 notes.push('interrupted week walked Mon→Sun');
});
await journey('Morning rest owns actual calendar day', 'U10', async (page, j) => {
 await j.tap(/^Adapt today/); await j.tap('Rest today');
 const day = await page.evaluate(() => window.RaringApp.state.rest.day);
 if (day !== '2026-11-18') throw new Error(`rest saved for ${day}, not the actual day`);
 await set(page, {clock: 'morning'});
 await expectText(page, 'Rest day', 'rest survives synced-night handover');
});
await journey('Movement review suppresses every training prompt', 'U31', async (page) => {
 await set(page, {day: '2026-11-21', permission: 'granted'}); await ext(page, 'today');
 await expectText(page, 'Review needed', 'unknown leg routine');
 await ext(page, 'surfaces');
 await expectText(page, 'Training suggestions limited', 'limited surfaces');
 await expectNoText(page, 'Training is an option', 'no high-readiness override');
 await set(page, {permission: 'declined'});
 if (await page.locator('.surf').filter({hasText: 'Morning readiness'}).count()) throw new Error('notification shown after permission denied');
});
await journey('Historical revisions do not time travel', 'U26', async (page) => {
 await set(page, {day: '2026-11-16'}); await ext(page, 'plan');
 await expectNoText(page, 'Targets changed Thu', 'Monday before revision');
 await ext(page, 'workout/a-1107'); await expectText(page, 'Strength session · no sets', 'old timer evidence');
 await ext(page, 'workout/a-1102'); await expectText(page, '1 of 3', 'older workout uses week 46 targets');
});
await journey('Permission denial does not repeat prompt', 'U57', async (page, j) => {
 await j.tap('Remind me to wind down');
 if (await page.locator('.sheet').count()) throw new Error('permission requested again');
 await ext(page, 'surfaces'); await expectText(page, 'Persistent status off', 'status defaults off');
});
await journey('Intention edit, timing migration and routine self-report', 'U22', async (page, j) => {
 await j.tap('Edit'); await j.tap('Tue 17'); await j.tap('Save changes');
 await expectText(page, 'Changed to Strength', 'edit history');
 await ext(page, 'plan');
 await j.tap('Goals'); for (let i = 0; i < 3; i++) await page.getByRole('button', {name: 'More Sleep timing'}).click(); await j.tap('Save weekly goals');
 const st = await page.evaluate(() => window.RaringApp.state);
 if (st.goalVersions?.at(-1)?.targets.sleep !== 3 || Object.values(st.winddown).some(Boolean)) throw new Error('sleep timing goal inferred completion');
 await expectText(page, '3 sleep-timing nights', 'sleep timing nights start in the backlog');
 await ext(page, 'adapt/routines');
 await page.getByLabel('Exercise name').fill('Wall sit 3 × 30 s'); await j.tap('Add to checklist'); await j.tap('Mark done today');
 await expectText(page, 'Done today · Self-reported', 'routine separate from Garmin');
});
await journey('Unknown timing never reuses old-night training advice', 'U38', async (page) => {
 await set(page, {clock: 'late'});
 await expectText(page, 'Your recorded day', 'neutral fixed presentation after midnight');
 await expectNoText(page, 'Training is an option', 'no old-night training advice');
 await ext(page, 'surfaces'); await expectText(page, 'Recorded context only', 'same fixed presentation on surfaces');
});
await journey('First commitment does not adopt old targets', 'U01', async (page) => {
 await ext(page, 'plan/commitments');
 await page.getByRole('button', {name: '2 strength sessions a week'}).click(); await settle(page);
 const st = await page.evaluate(() => window.RaringApp.state.ownCommitments);
 if (!st || st.im !== 0) throw new Error('intensity commitment silently adopted');
 await ext(page, 'progress'); await expectNoText(page, 'targets then: 3 strength', 'no pre-setup achievements');
});
await journey('Movement review covers a local Run intention', 'U01', async (page, j) => {
 await set(page, {limits: {exclude: [], note: 'No deep knee bends', routines: {}}});
 await page.locator('.tabs').getByRole('button', {name: 'Plan', exact: true}).click(); await settle(page);
 await j.tap('Add a session'); await j.tap(/^Walk/, '.sheet');
 if (!(await page.locator('.sheet').innerText()).includes('Review needed')) throw new Error('movement note missing in non-strength planning');
 await j.tap('Today', '.sheet'); await j.tap('Place in my week');
 await page.locator('.tabs').getByRole('button', {name: 'Today', exact: true}).click(); await settle(page);
 await expectText(page, 'Your limits apply', 'movement review before readiness'); await expectText(page, 'Review needed', 'session marked for review');
 await page.locator('.hero .ctx-row').filter({hasText: 'Walk'}).click(); await settle(page);
 await expectText(page, 'No deep knee bends', 'note visible on the local intention');
});
await journey('Fixed layout preserves tonight and self-report identity', 'U37', async (page) => {
 const before = await page.evaluate(() => window.RaringRules.derive(window.RaringApp.state).tonight.nightOf);
 await set(page, {follow: false}); await ext(page, 'sleep/tonight');
 await expectText(page, 'Wed night → Thu morning', 'presentation independent night');
 await page.getByRole('button', {name: 'I wound down', exact: true}).click(); await settle(page);
 const marks = await page.evaluate(() => window.RaringApp.state.winddown);
 if (!marks[before] || marks['2026-11-19']) throw new Error('preparation assigned to a different night');
});
await journey('Time-only edit retains activity and original day', 'U19', async (page, j) => {
 await ext(page, 'intention/i0');
 await j.tap('Edit'); await page.locator('.sheet input[type=time]').fill('08:30'); await settle(page);
 await j.tap('Save changes');
 const i = await page.evaluate(() => window.RaringApp.state.intentions.find(x => x.id === 'i0'));
 if (i.sport !== 'strength' || i.date !== '2026-11-16' || i.time !== '08:30') throw new Error(`time edit corrupted draft: ${JSON.stringify(i)}`);
});
await journey('Changing date/type clears a confirmed link', 'U22', async (page, j) => {
 await j.tap('This was my session'); await j.tap('Edit'); await j.tap('Run'); await j.tap('Tue 17'); await j.tap('Save changes');
 const i = await page.evaluate(() => window.RaringApp.state.intentions.find(x => x.id === 'i0'));
 if (i.link) throw new Error('incompatible activity link survived');
 await expectText(page, 'old link cleared', 'association history retained');
});
await journey('Progress periods move and compare to date', 'U27', async (page, j) => {
 const tab = async name => { await page.getByRole('tab', {name, exact: true}).click(); j.path.push(name); await settle(page); };
 await tab('Year'); await expectText(page, 'Dec 2025–Nov 2026', 'year label');
 await tab('Week'); await expectText(page, '16–22 Nov', 'week label');
 await j.tap('Previous period'); await expectText(page, '9–15 Nov', 'previous week');
 await tab('Month'); await expectText(page, 'November 2026', 'month');
 await set(page, {profile: 'simple'});
 await expectText(page, 'vs 1–18 Oct', 'to-date comparison');
 await set(page, {profile: 'sparse'});
 await expectText(page, 'Known activities · partial history', 'partial totals labelled');
 if (await page.locator('.report .vs').count()) throw new Error('partial history has an unqualified comparison');
 const future = await page.locator('.sn .nt').nth(25).locator('.gap').count();
 if (future) throw new Error('a future night is drawn as a missing night');
 await expectText(page, 'Active on', 'active days denominator');
 await expectNoText(page, 'streak', 'no streak counter');
 j.done('progress', 'n/a');
});

await journey('Mobile Chrome-sized touch screen at DPR 3', 'U13', async (page, j) => {
 await page.locator('.hero .session-open').tap();
 await settle(page); await expectRoute(page,'scheduled/g-int','touch review');
 await page.locator('.back').tap(); await settle(page);
 await page.getByRole('button',{name:'Adapt today',exact:true}).tap(); await settle(page);
 await page.locator('.sheet').getByRole('button',{name:'Rest today',exact:true}).tap(); await settle(page);
 await expectText(page,'Rest day','touch rest');
 notes.push('390×844 mobile emulation, touch, DPR 3: review and rest flows passed');
}, {width:390,height:844}, {dpr:3,touch:true});
await journey('Metric controls survive caller Back', 'U51', async (page) => {
 await page.getByRole('button',{name:'Custom dates',exact:true}).click(); await settle(page);
 await page.getByLabel('Block start',{exact:true}).fill('2026-10-25');
 await page.getByLabel('Block end',{exact:true}).fill('2026-11-10');
 await page.getByRole('button',{name:'Metric',exact:true}).click(); await settle(page);
 await page.getByLabel('Metric to overlay').selectOption('sleepTime'); await settle(page);
 await page.evaluate(()=>window.RaringApp.navigate('help')); await settle(page);
 await page.goBack(); await settle(page);
 if(await page.getByLabel('Block start',{exact:true}).inputValue()!=='2026-10-25') throw new Error('custom start lost');
 if(await page.getByLabel('Block end',{exact:true}).inputValue()!=='2026-11-10') throw new Error('custom end lost');
 if(await page.getByLabel('Metric to overlay').inputValue()!=='sleepTime') throw new Error('overlay lost');
});

// ---------- 3. Layout budgets ----------
const VIEWS = [
 ['phone-390', {width: 390, height: 844}, 1], ['phone-360', {width: 360, height: 780}, 1],
 ['phone-390-text1.3', {width: 390, height: 844}, 1.3], ['phone-390-text2', {width: 390, height: 844}, 2],
 ['unfolded-720', {width: 720, height: 900}, 1], ['desktop-1280', {width: 1280, height: 820}, 1],
];
const TODAY_CASES = ['U01', 'U03', 'U05', 'U08', 'U10', 'U13', 'U23', 'U24', 'U29', 'U30', 'U36', 'U37', 'U43'];
for (const [name, viewport, scale] of VIEWS) {
 const {context, page} = await fresh(viewport);
 for (const id of TODAY_CASES) {
  await open(page, `case=${id}&text=${scale}`);
  await page.locator('.case-note button').click().catch(() => {}); await settle(page);
  const m = await page.evaluate(() => {
   const z = parseFloat(getComputedStyle(document.querySelector('.scroll')).zoom) || 1;
   const r = el => el ? el.getBoundingClientRect() : null;
   const scroll = document.querySelector('.scroll');
   const tab = document.querySelector('.tabs').getBoundingClientRect();
   const tabTop = tab.width < innerWidth / 2 ? innerHeight : tab.top;
   const lead = r(document.querySelector('.hero h2')), act = r(document.querySelector('.hero .session-actions button, .hero .atile, .hero .calm-row button, .hero .tonight-line, .hero > .secondary')), notice = r(document.querySelector('main .notice-strip'));
   const ribbon = r(document.querySelector('.day-rows'));
   const wide = [...document.querySelectorAll('main *')].filter(el => { const b = el.getBoundingClientRect(); return el instanceof HTMLElement && b.width && el.closest('.miles, .ch-row, .wstrip') == null && (b.right > innerWidth + 1 || b.left < -1); }).map(el => el.className || el.tagName).slice(0, 3);
   const clipped = [...document.querySelectorAll('.tile, .tab, .hbtn, .adapt, .chipb, .primary, .secondary, .hero h2, .rowlink b')].filter(el => el.offsetParent && el.scrollWidth > el.clientWidth + 1).map(el => (el.className || el.tagName) + ':' + el.textContent.trim().slice(0, 20)).slice(0, 3);
   return {height: scroll.scrollHeight * z, vh: innerHeight, tabTop, actionBottom: act ? act.bottom : null, noticeBottom: notice ? notice.bottom : null, leadBeforeRibbon: lead && ribbon ? lead.top < ribbon.top : null,
    overflowX: scroll.scrollWidth - scroll.clientWidth, wide, clipped};
  });
  const aboveFold = m.actionBottom != null && m.actionBottom <= m.tabTop && (m.noticeBottom == null || m.noticeBottom <= m.tabTop);
  const viewports = +(m.height / m.vh).toFixed(2);
  layout.push({view: name, id, aboveFold, viewports, overflowX: m.overflowX});
  if (m.overflowX > 1 || m.wide.length) fail(`${name} ${id}`, `horizontal overflow ${m.overflowX}px ${m.wide.join(',')}`);
  if (m.clipped.length) fail(`${name} ${id}`, `text wider than its box: ${m.clipped.join(' | ')}`);
  if (scale === 1 && name.startsWith('phone') && !aboveFold) fail(`${name} ${id}`, `hero action or notice below the fold (${Math.round(m.actionBottom)} > ${Math.round(m.tabTop)})`);
  if (name === 'phone-390' && viewports > 2.6) fail(`${name} ${id}`, `Today is ${viewports} viewports tall`);
  if (m.leadBeforeRibbon === false) fail(`${name} ${id}`, 'hero is not before the day ribbon in reading order');
 }
 await context.close();
}
for (const scale of [1, 1.3, 2]) {
 const {context, page} = await fresh({width: 360, height: 780});
 await open(page, `case=U19&text=${scale}`);
 for (const r of ['plan', 'progress', 'sleep', 'sleep/patterns', 'readiness', 'training', 'strength', 'activities', 'health', 'settings', 'adapt', 'metric/hrv', 'workout/a-1203', 'workout/a-1201', 'review/2026-W46', 'plan/commitments']) {
  await ext(page, r);
  const extra = await page.evaluate(() => { const s = document.querySelector('.scroll'); return s.scrollWidth - s.clientWidth; });
  if (extra > 1) fail(`detail ${r} text${scale}`, `horizontal overflow ${extra}px`);
 }
 await context.close();
}

// ---------- 4. Accessibility heuristics ----------
{
 const {context, page} = await fresh({width: 390, height: 844}, {reducedMotion: 'reduce'});
 await open(page, 'case=U24');
 const running = await page.evaluate(() => document.getAnimations().filter(a => a.playState === 'running').length);
 if (running) fail('reduced motion', `${running} animations run with reduced motion`);
 if (!(await page.locator('.hero h2').innerText()).includes('Strength target reached')) fail('reduced motion', 'information lost without animation');
 const pages = ['today', 'plan', 'progress', 'sleep', 'readiness', 'training', 'strength', 'activities', 'health', 'settings', 'adapt', 'workout/a-1201', 'workout/a-1203', 'metric/hrv', 'review/2026-W46', 'surfaces', 'plan/commitments'];
 for (const p of pages) {
  await ext(page, p);
  const unnamed = await page.evaluate(() => [...document.querySelectorAll('button, a, input, select, textarea')].filter(el => {
   const name = (el.getAttribute('aria-label') || el.textContent || el.labels?.[0]?.textContent || el.getAttribute('placeholder') || '').trim();
   return !name && el.offsetParent !== null;
  }).map(el => el.outerHTML.slice(0, 80)));
  if (unnamed.length) fail(`a11y ${p}`, `unnamed controls: ${unnamed.join(' | ')}`);
  const small = await page.evaluate(() => [...document.querySelectorAll('main button, .tabs button')].filter(el => el.offsetParent && !el.classList.contains('inline') && !el.closest('.rb-wrap, .wstrip, .cal, .ch-row') && el.getBoundingClientRect().height < 32).map(el => (el.getAttribute('aria-label') || el.textContent).trim().slice(0, 30)));
  if (small.length) fail(`a11y ${p}`, `targets under 32px tall: ${small.slice(0, 4).join(' | ')}`);
 }
 await open(page, 'case=U13');
 await page.locator('.case-note button').click(); await settle(page);
 await page.evaluate(() => document.activeElement.blur());
 const order = [];
 for (let i = 0; i < 8; i++) { await page.keyboard.press('Tab'); order.push(await page.evaluate(() => (document.activeElement.getAttribute('aria-label') || document.activeElement.textContent).trim().slice(0, 24))); }
 notes.push(`keyboard order on Today: ${order.join(' → ')}`);
 if (!order.some(o => o.startsWith('Adapt today')) || !order.includes('Settings')) fail('keyboard', `Settings/Adapt not early in tab order: ${order.join(' → ')}`);
 await page.getByRole('button', {name: /^Adapt today/}).focus(); await page.keyboard.press('Enter'); await settle(page);
 if (!await page.evaluate(() => document.activeElement.closest('.sheet') != null)) fail('keyboard', 'focus did not move into the Adapt sheet');
 await page.keyboard.press('Escape'); await settle(page);
 if (await page.locator('.sheet').count()) fail('keyboard', 'Escape did not close the sheet');
 await context.close();
}

}
// ---------- 5. Screenshots ----------
if (shots) {
 await mkdir(outDir, {recursive: true});
 const list = [
  ['phone', 'U19', 'tab:training'], ['phone', 'U19', 'tab:health'], ['phone', 'U19', 'full-tab:training'], ['phone', 'U19', 'full-tab:health'], ['phone', 'U41', 'full-tab:plan'],
  ['phone', 'U11', ''], ['phone', 'U01', ''], ['phone', 'U03', ''], ['phone', 'U05', ''], ['phone', 'U08', ''], ['phone', 'U10', ''], ['phone', 'U13', ''], ['phone', 'U24', ''], ['phone', 'U29', ''], ['phone', 'U30', ''], ['phone', 'U36', ''], ['phone', 'U37', ''], ['phone', 'U12', ''],
  ['phone', 'U11', 'full'], ['phone', 'U23', 'full'], ['phone', 'U19', 'full'], ['phone', 'U26', 'full'], ['phone', 'U27', 'full'], ['phone', 'U47', 'full'], ['phone', 'U50', 'full'], ['phone', 'U42', 'full'], ['phone', 'U15', 'full'], ['phone', 'U16', 'full'],
  ['phone', 'U15', 'run'], ['phone', 'U31', 'full'], ['phone', 'U53', ''], ['phone', 'U53', 'journal-open'], ['phone', 'U44', ''], ['phone', 'U40', 'full'], ['phone', 'U18', 'full'], ['phone', 'U17', 'full'],
  ['phone', 'U13', 'sheet:adapt'], ['phone', 'U11', 'sheet:add'], ['phone', 'U13', 'sheet:why'], ['phone', 'U13', 'sheet:legend'], ['phone', 'U28', 'surfaces'], ['phone', 'U27', 'year'],
  ['phone-text1.3', 'U13', ''], ['phone-text2', 'U13', 'full'], ['phone-360', 'U30', ''], ['phone-360', 'U27', 'full'], ['unfolded', 'U13', ''], ['desktop', 'U13', ''], ['desktop', 'U27', 'full'],
 ];
 const sizes = {phone: [390, 844, 1], 'phone-text1.3': [390, 844, 1.3], 'phone-text2': [390, 844, 2], 'phone-360': [360, 780, 1], unfolded: [720, 900, 1], desktop: [1280, 820, 1]};
 for (const [size, id, mode] of list) {
  const [w, hgt, scale] = sizes[size];
  const {context, page} = await fresh({width: w, height: hgt}, {dpr: 2});
  await open(page, `case=${id}&text=${scale}`);
  await page.locator('.case-note button').click().catch(() => {}); await settle(page);
  if (mode === 'run') await ext(page, 'workout/a-1203');
  if (mode.includes('tab:')) await ext(page, mode.split('tab:')[1]);
  if (mode === 'year') { await page.getByRole('tab', {name: 'Year'}).click(); await settle(page); }
  if (mode === 'journal-open') { await page.getByRole('button', {name: 'Open journal', exact: true}).click(); await settle(page); }
  if (mode.startsWith('sheet:')) { await page.evaluate(n => window.RaringApp.openSheet(n), mode.slice(6)); await settle(page); }
  if (mode === 'surfaces') { await set(page, {rest: {day: '2026-11-20'}, permission: 'granted', channels: {morning: true, weekly: true, status: true, metrics: true, vitals: true, training: false}}); await ext(page, 'surfaces'); }
  await page.waitForTimeout(1300);
  if (['full', 'surfaces', 'run', 'year'].includes(mode) || mode.startsWith('full-')) { const full = await page.evaluate(() => { const s = document.querySelector('.scroll'); return s.scrollHeight * (parseFloat(getComputedStyle(s).zoom) || 1) + 20; }); if (full > hgt) { await page.setViewportSize({width: w, height: Math.min(9000, Math.ceil(full))}); await page.waitForTimeout(400); } }
  const r = (await route(page)).replace(/\//g, '-');
  await page.screenshot({path: resolve(outDir, `${size}-${id}-${mode ? mode.replace(/:.*/, '') + '-' : ''}${r}.png`)});
  await context.close();
 }
 // Compare captures: synthetic v2 alongside the shipped app and retired v1.
 for (const [cs, r, file] of [['U24', '', 'v2-today.jpg'], ['U15', 'workout/a-1203', 'v2-workout.jpg'], ['U27', '', 'v2-progress.jpg']]) {
  const {context, page} = await fresh({width: 390, height: 844}, {dpr: 1.5});
  await open(page, `case=${cs}`); await page.locator('.case-note button').click().catch(() => {});
  if (r) await ext(page, r);
  await page.waitForTimeout(1300);
  const full = await page.evaluate(() => document.querySelector('.scroll').scrollHeight + 20);
  await page.setViewportSize({width: 390, height: Math.min(6000, full)}); await page.waitForTimeout(400);
  await page.screenshot({path: resolve(compareDir, file), type: 'jpeg', quality: 72});
  await context.close();
 }
 notes.push(`screenshots written to ${outDir}`);
}

await browser.close();
await server?.close();
const fold = layout.filter(l => l.view === 'phone-390');
await mkdir(resolve('../.data'), {recursive: true});
if (!shotsOnly) await writeFile(resolve('../.data/experience-check.json'), JSON.stringify({failures, notes, journeys, layout}, null, 2));
console.log('Journeys (deliberate taps):');
for (const j of journeys) console.log(`  ${j.name}: ${j.taps} [budget ${j.budget}] ${j.path}${j.extra ? ` (${j.extra})` : ''}`);
console.log('Today at 390×844: ' + fold.map(l => `${l.id} ${l.viewports}vp${l.aboveFold ? '' : ' BELOW-FOLD'}`).join(', '));
console.log('Large text (hero action above fold?): ' + layout.filter(l => l.view.includes('text')).map(l => `${l.view.replace('phone-390-', '')}/${l.id}:${l.aboveFold ? 'yes' : 'no'}`).join(' '));
for (const n of notes) console.log(`Note: ${n}`);
if (failures.length) { console.error(`\n${failures.length} failure(s):\n` + failures.map(f => `  ${f}`).join('\n')); process.exit(1); }
console.log(shotsOnly ? '\nSynthetic captures refreshed; checks skipped.' : '\nAll prototype checks passed. This is not usability, device or real-data evidence.');
