// Screenshots of the mockup's screens at phone, unfolded and desktop widths, into design/mockup/shots/.
// Run from anywhere: node design/mockup/shoot.mjs. Needs Playwright from app/node_modules; set
// RARING_NODE_MODULES to use another checkout's.
import {createRequire} from 'node:module';
import {mkdir} from 'node:fs/promises';
import {dirname, resolve} from 'node:path';
import {fileURLToPath, pathToFileURL} from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const modules = process.env.RARING_NODE_MODULES ?? resolve(here, '../../app/node_modules');
const {chromium} = createRequire(resolve(modules, 'noop.js'))('playwright');
const url = pathToFileURL(resolve(here, 'index.html')).href, out = resolve(here, 'shots');
const sizes = {phone: [390, 844, 2], fold: [720, 900, 1], desktop: [1280, 820, 1]};
const screens = {
  language: ['phone', 'desktop'],
  today: ['phone'], sleep: ['phone'], training: ['phone'], workout: ['phone'], health: ['phone'],
  plan: ['phone', 'fold', 'desktop'], 'plan-setup': ['phone', 'fold', 'desktop'], checkin: ['phone'],
  'intro-1': ['phone'], 'intro-2': ['phone'], 'intro-3': ['phone'], 'intro-4': ['phone'],
  'strength-workout': ['phone', 'fold', 'desktop'], fix: ['phone', 'fold', 'desktop'], 'strength-history': ['phone', 'fold', 'desktop'],
  builder: ['phone', 'fold', 'desktop'], catalog: ['phone'], help: ['phone', 'desktop'], 'help-sheet': ['phone'], widget: ['phone'],
  'tod-rules': ['phone'], 'tod-wake': ['phone'], 'tod-day': ['phone', 'desktop'], 'tod-evening': ['phone', 'fold'], 'tod-winddown': ['phone'],
  'tod-late': ['phone'], 'tod-missing': ['phone'], 'tod-overview': ['phone'],
  reconcile: ['phone'], 'reconcile-closed': ['phone'], 'why-readiness': ['phone', 'desktop'], 'why-battery': ['phone'], 'why-training': ['phone'],
};
// Overlays are shot at the window's size, as they'd appear.
const fixedHeight = new Set(['help-sheet', 'widget', 'why-readiness', 'why-battery', 'why-training']);

await mkdir(out, {recursive: true});
const browser = await chromium.launch({executablePath: process.env.CHROMIUM ?? '/run/current-system/sw/bin/chromium', args: ['--no-sandbox']});
const errors = [];
try {
  for (const [kind, [w, h, scale]] of Object.entries(sizes)) {
    const page = await browser.newPage({viewport: {width: w, height: h}, deviceScaleFactor: scale});
    page.on('pageerror', e => errors.push(e.message));
    for (const [key, kinds] of Object.entries(screens)) {
      if (!kinds.includes(kind)) continue;
      await page.setViewportSize({width: w, height: h});
      await page.goto(`${url}?app#${key}`, {waitUntil: 'networkidle'});
      await page.evaluate(() => document.fonts.ready);
      // Grow the window to the content so each shot shows the whole screen, not just the first view.
      const full = await page.evaluate(() => document.querySelector('.scroll')?.scrollHeight ?? 0);
      if (full > h && !fixedHeight.has(key)) await page.setViewportSize({width: w, height: full});
      const overflow = await page.evaluate(() => document.querySelector('.scroll') ? document.querySelector('.scroll').scrollWidth - document.querySelector('.scroll').clientWidth : 0);
      if (overflow > 1) errors.push(`${kind}/${key}: ${overflow}px horizontal overflow`);
      await page.screenshot({path: resolve(out, `${kind}-${key}.png`)});
    }
    await page.close();
  }
  // The compatibility page lives on the static site, which runs without JavaScript or third-party requests.
  const site = await browser.newContext({javaScriptEnabled: false});
  const sitePage = await site.newPage();
  const siteUrl = pathToFileURL(resolve(here, '../../site/compatibility.html')).href;
  await site.route('**/*', route => route.request().url().startsWith('file:') ? route.continue() : (errors.push(`site requested ${route.request().url()}`), route.abort()));
  for (const [w, name] of [[320], [390, 'phone'], [768], [1280, 'desktop']]) {
    await sitePage.setViewportSize({width: w, height: 900});
    await sitePage.goto(siteUrl);
    await sitePage.evaluate(() => document.fonts.ready);
    const wide = await sitePage.evaluate(() => document.documentElement.scrollWidth);
    if (wide > w) errors.push(`site ${w}px: ${wide}px wide`);
    if (name) await sitePage.screenshot({path: resolve(out, `${name}-site-compatibility.png`), fullPage: true});
  }
  await site.close();
  const gallery = await browser.newPage({viewport: {width: 2240, height: 900}, deviceScaleFactor: 1.5});
  await gallery.goto(`${url}?group=Core`, {waitUntil: 'networkidle'});
  await gallery.evaluate(() => document.fonts.ready);
  await gallery.screenshot({path: resolve(here, 'screens.png'), fullPage: true});
} finally {
  await browser.close();
}
if (errors.length) { console.error(errors.join('\n')); process.exit(1); }
console.log(`Screenshots in ${out}`);
