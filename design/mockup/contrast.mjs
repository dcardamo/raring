// Checks text contrast on every mockup screen: 4.5:1 for normal text, 3:1 for large text
// (24px, or 18.7px bold). Backgrounds are composited from translucent layers over the page ink.
// Run: node design/mockup/contrast.mjs (Playwright from app/node_modules, or RARING_NODE_MODULES).
import {createRequire} from 'node:module';
import {dirname, resolve} from 'node:path';
import {fileURLToPath, pathToFileURL} from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const modules = process.env.RARING_NODE_MODULES ?? resolve(here, '../../app/node_modules');
const {chromium} = createRequire(resolve(modules, 'noop.js'))('playwright');
const url = pathToFileURL(resolve(here, 'index.html')).href;
const screens = ['language', 'today', 'sleep', 'training', 'workout', 'health', 'plan', 'plan-setup', 'checkin', 'intro-1', 'intro-4',
  'strength-workout', 'fix', 'strength-history', 'builder', 'catalog', 'help', 'help-sheet', 'widget', 'status-surfaces'];

const browser = await chromium.launch({executablePath: process.env.CHROMIUM ?? '/run/current-system/sw/bin/chromium', args: ['--no-sandbox']});
const failures = [];
try {
  for (const width of [390, 1280]) {
    const page = await browser.newPage({viewport: {width, height: 900}});
    for (const key of screens) {
      await page.goto(`${url}?app#${key}`);
      await page.evaluate(() => document.fonts.ready);
      const found = await page.evaluate(() => {
        const parse = c => { const m = c.match(/rgba?\(([^)]+)\)/); if (!m) return null; const [r, g, b, a = 1] = m[1].split(/[ ,/]+/).filter(Boolean).map(Number); return [r, g, b, a]; };
        const over = (top, base) => { const a = top[3]; return [0, 1, 2].map(i => top[i] * a + base[i] * (1 - a)).concat(1); };
        const lum = c => { const v = c.slice(0, 3).map(x => { x /= 255; return x <= .03928 ? x / 12.92 : ((x + .055) / 1.055) ** 2.4; }); return .2126 * v[0] + .7152 * v[1] + .0722 * v[2]; };
        const ratio = (a, b) => { const [h, l] = [lum(a), lum(b)].sort((x, y) => y - x); return (h + .05) / (l + .05); };
        const background = el => {
          const layers = [];
          const box = el.getBoundingClientRect(), cx = box.x + box.width / 2, cy = box.y + box.height / 2;
          for (let e = el; e && e !== document.documentElement; e = e.parentElement) {
            // A label positioned outside its parent (a marker's caption) doesn't sit on that parent's fill.
            const r = e.getBoundingClientRect();
            if (e !== el && (cx < r.left || cx > r.right || cy < r.top || cy > r.bottom)) continue;
            const bg = parse(getComputedStyle(e).backgroundColor);
            if (bg && bg[3] > 0) { layers.push(bg); if (bg[3] === 1) break; }
          }
          return layers.reverse().reduce((base, top) => over(top, base), [11, 15, 26, 1]);
        };
        const opacity = el => { let o = 1; for (let e = el; e; e = e.parentElement) o *= +getComputedStyle(e).opacity; return o; };
        const sheet = document.querySelector('.sheet'), out = [];
        for (const el of document.querySelectorAll('.cq *, .cq text')) {
          if (sheet && !sheet.contains(el)) continue;
          const own = [...el.childNodes].some(n => n.nodeType === 3 && n.textContent.trim());
          if (!own) continue;
          const r = el.getBoundingClientRect(), cs = getComputedStyle(el);
          if (!r.width || cs.visibility === 'hidden' || el.closest('.tabs.off, [style*="display:none"]') || cs.textDecorationLine.includes('line-through')) continue;
          const svg = el instanceof SVGElement;
          let fg = parse(svg ? cs.fill : cs.color);
          if (!fg) continue;
          const bg = background(svg ? el.closest('div') : el);
          fg = [...fg.slice(0, 3), fg[3] * opacity(el)];
          const c = ratio(over(fg, bg), bg), size = parseFloat(cs.fontSize), bold = +cs.fontWeight >= 700;
          const need = size >= 24 || (bold && size >= 18.66) ? 3 : 4.5;
          if (c < need) out.push(`${c.toFixed(2)} < ${need}  ${size}px  "${el.textContent.trim().slice(0, 30)}"`);
        }
        return [...new Set(out)];
      });
      for (const f of found) failures.push(`${width}/${key}: ${f}`);
    }
    await page.close();
  }
} finally {
  await browser.close();
}
console.log(failures.length ? failures.join('\n') : 'All text meets contrast minimums.');
process.exit(failures.length ? 1 : 0);
