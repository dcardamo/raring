// Publish only fixture screenshots from `npm run screenshots`, never emulator or personal data.
// Run from app/: node src/scripts/publish-screens.mjs
import {copyFile,readFile} from 'node:fs/promises';
import {resolve} from 'node:path';
import {chromium} from 'playwright';

const names=['today','sleep','training','workout','health','stress'];
const root=resolve('..'),screens=resolve(root,'.data/screens'),assets=resolve(root,'site/assets');
for(const name of names)await copyFile(resolve(screens,`${name}.png`),resolve(assets,`${name}.png`));
const image=async name=>`data:image/png;base64,${(await readFile(resolve(assets,`${name}.png`))).toString('base64')}`;
const font=`data:font/woff2;base64,${(await readFile(resolve(assets,'outfit.woff2'))).toString('base64')}`;
const today=await image('today');
const brandIcon=`data:image/svg+xml;base64,${(await readFile(resolve(assets,'icon.svg'))).toString('base64')}`;
const phones=await Promise.all(names.slice(0,5).map(async name=>({name,src:await image(name)})));
const browser=await chromium.launch({executablePath:process.env.CHROMIUM??'/run/current-system/sw/bin/chromium',args:['--no-sandbox']});
try{
 const collage=await browser.newPage({viewport:{width:1680,height:738},deviceScaleFactor:2});
 await collage.setContent(`<style>@font-face{font-family:Outfit;src:url('${font}')}*{box-sizing:border-box}body{margin:0;background:#070b13;color:#b4bccb;font:600 12px Outfit,system-ui,sans-serif;letter-spacing:.18em;text-transform:uppercase}.gallery{height:738px;display:flex;align-items:center;justify-content:center;gap:24px}.item{width:296px;text-align:center}.item span{display:block;margin-bottom:18px}.phone{padding:8px;border-radius:37px;background:#141824;box-shadow:0 0 0 1px #303847,0 20px 50px #0008}.phone img{width:280px;display:block;border-radius:29px}</style><div class="gallery">${phones.map(({name,src})=>`<div class="item"><span>${name}</span><div class="phone"><img src="${src}" alt=""></div></div>`).join('')}</div>`);
 await collage.evaluate(()=>document.fonts.ready);
 await collage.locator('img').first().evaluate(async()=>await Promise.all([...document.images].map(i=>i.decode())));
 await collage.screenshot({path:resolve(assets,'screens.png')});
 await collage.close();

 const social=await browser.newPage({viewport:{width:1200,height:630}});
 await social.setContent(`<style>@font-face{font-family:Outfit;src:url('${font}')}*{box-sizing:border-box}body{margin:0;background:radial-gradient(ellipse at 80% 12%,#17323977,transparent 38%),#0b0f1a;color:#eef1f7;font:16px/1.65 system-ui,sans-serif}.header{display:flex;align-items:center;justify-content:space-between;padding:18px 58px}.brand{font:600 27px Outfit;display:flex;align-items:center;gap:12px}.brand-icon{width:40px;height:40px}nav{color:#b4bccb;font-size:14px;display:flex;gap:26px}.copy{position:absolute;left:58px;top:128px;width:600px}.eyebrow{color:#3ddc97;font:700 11px system-ui;letter-spacing:.15em}h1{font:600 68px/1.05 Outfit;letter-spacing:-.03em;margin:24px 0 20px}p{color:#b4bccb;font-size:18px;line-height:1.6;margin:0;max-width:565px}.free{margin-top:16px}.free b{color:#3ddc97}.button{display:inline-block;margin-top:16px;border-radius:14px;padding:13px 22px;background:#e4e8f0;color:#0b0f1a;font:700 15px system-ui}.phone{position:absolute;left:912px;top:98px;width:230px;padding:7px;background:#19212d;border:1px solid #354255;border-radius:32px}.phone img{display:block;width:100%;border-radius:25px}.rule{position:absolute;left:58px;right:58px;bottom:34px;border-top:1px solid #273140}</style><div class="header"><div class="brand"><img class="brand-icon" src="${brandIcon}" alt="">Raring</div><nav><span>How it works</span><span>Privacy</span><span>Source</span></nav></div><div class="copy"><div class="eyebrow">ANDROID · EARLY RELEASE</div><h1>A clearer view<br>of your day.</h1><p>Your Garmin readiness, sleep and training, at a glance. See how you’re doing, what’s behind it, and when to take it easy.</p><p class="free"><b>Free to use.</b> No subscription.</p><div class="button">Get Raring for Android　↗</div></div><div class="phone"><img src="${today}" alt=""></div><div class="rule"></div>`);
 await social.evaluate(()=>document.fonts.ready);
 await social.locator('img').first().evaluate(async()=>await Promise.all([...document.images].map(i=>i.decode())));
 await social.screenshot({path:resolve(assets,'social.png')});
 await social.close();
}finally{await browser.close()}
console.log('Published six synthetic app screenshots, five-screen README composite and social preview.');
