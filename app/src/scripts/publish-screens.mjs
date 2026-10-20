// Publish the website images from `npm run screenshots`: synthetic fixtures only, never emulator or personal data.
// Run from app/: node src/scripts/publish-screens.mjs
import {copyFile,readFile} from 'node:fs/promises';
import {resolve} from 'node:path';
import {chromium} from 'playwright';

// Website screens, named as in .data/screens and site/assets.
const site=['today','plan','sleep','training','strength-workout','health','review'];
// The README montage shows the five tabs.
const tabs=[['Today','today'],['Plan','plan'],['Training','training'],['Health','health'],['Progress','progress-cardio']];
const root=resolve('..'),screens=resolve(root,'.data/screens'),assets=resolve(root,'site/assets');
for(const name of site)await copyFile(resolve(screens,`${name}.png`),resolve(assets,`${name}.png`));
const data=async(path,type)=>`data:${type};base64,${(await readFile(path)).toString('base64')}`;
const shot=name=>data(resolve(screens,`${name}.png`),'image/png');
const inter=await data(resolve(assets,'inter.woff2'),'font/woff2'),outfit=await data(resolve(assets,'outfit.woff2'),'font/woff2');
const icon=await data(resolve(assets,'icon.svg'),'image/svg+xml');
// The site's palette (site/style.css), which follows the app's tokens.
const base=`@font-face{font-family:Inter;src:url('${inter}')}@font-face{font-family:Outfit;src:url('${outfit}')}*{box-sizing:border-box}body{margin:0;background:#111317;color:#F4F1EA;font-family:Inter,sans-serif}.phone{padding:7px;border-radius:34px;background:#1B1E24;border:1px solid #343A43;box-shadow:0 24px 60px #0009}.phone img{display:block;width:100%;border-radius:27px}`;
const ready=async page=>{
 await page.evaluate(()=>document.fonts.ready);
 await page.locator('img').first().evaluate(async()=>await Promise.all([...document.images].map(i=>i.decode())));
};
const browser=await chromium.launch({executablePath:process.env.CHROMIUM??'/run/current-system/sw/bin/chromium',args:['--no-sandbox']});
try{
 const phones=await Promise.all(tabs.map(async([label,name])=>({label,src:await shot(name)})));
 const collage=await browser.newPage({viewport:{width:1680,height:738},deviceScaleFactor:2});
 await collage.setContent(`<style>${base}.gallery{height:738px;display:flex;align-items:center;justify-content:center;gap:24px}.item{width:296px;text-align:center}.item span{display:block;margin-bottom:18px;color:#A1ABB9;font:600 12px Inter;letter-spacing:.16em;text-transform:uppercase}</style><div class="gallery">${phones.map(({label,src})=>`<div class="item"><span>${label}</span><div class="phone"><img src="${src}" alt=""></div></div>`).join('')}</div>`);
 await ready(collage);
 await collage.screenshot({path:resolve(assets,'screens.png')});
 await collage.close();

 const social=await browser.newPage({viewport:{width:1200,height:630}});
 await social.setContent(`<style>${base}.brand{position:absolute;left:58px;top:40px;display:flex;align-items:center;gap:12px;font:600 27px Inter;letter-spacing:-.03em}.brand img{width:40px;height:40px}.copy{position:absolute;left:58px;top:132px;width:600px}.eyebrow{color:#A1ABB9;font:600 12px Inter;letter-spacing:.14em}h1{font:600 70px/1.02 Inter;letter-spacing:-.05em;margin:22px 0 22px}p{color:#C2C8D1;font-size:20px;line-height:1.5;margin:0;max-width:540px}.free{margin-top:18px;color:#A1ABB9}.free b{color:#F4F1EA;font-weight:600}.button{display:inline-block;margin-top:26px;border-radius:12px;padding:14px 24px;background:#F4F1EA;color:#111317;font:600 17px Inter}.phone{position:absolute;width:236px}.back{left:752px;top:118px;opacity:.9}.front{left:900px;top:70px}</style><div class="brand"><img src="${icon}" alt="">Raring</div><div class="copy"><div class="eyebrow">ANDROID · FREE · OPEN SOURCE</div><h1>A clearer view<br>of your day.</h1><p>Your Garmin sleep, readiness and training at a glance, with weekly goals your workouts fill in.</p><p class="free"><b>Free to use.</b> No subscription, ads or tracking.</p><div class="button">Get Raring for Android ↗</div></div><div class="phone back"><img src="${await shot('plan')}" alt=""></div><div class="phone front"><img src="${await shot('today')}" alt=""></div>`);
 await ready(social);
 await social.screenshot({path:resolve(assets,'social.png')});
 await social.close();
}finally{await browser.close()}
console.log(`Published ${site.length} synthetic app screens, the five-tab README montage and the social preview.`);
