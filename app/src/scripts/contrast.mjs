// Contrast of rendered text, including translucent cards, tiles and chips over the page ink.
// Return every failing label so the UI regression can identify its screen and width.
// The site regression passes its own selector; the app's text is inside .phone.
export async function contrastFailures(page, selector = '.phone *') {
 return page.evaluate(selector => {
  const parse = c => {
   const m = c.match(/rgba?\(([^)]+)\)/);
   if (!m) return null;
   const [r,g,b,a=1] = m[1].split(/[ ,/]+/).filter(Boolean).map(Number);
   return [r,g,b,a];
  };
  const over = (top,base) => [0,1,2].map(i => top[i]*top[3]+base[i]*(1-top[3])).concat(1);
  const lum = c => {
   const v = c.slice(0,3).map(x => {x/=255;return x<=.03928?x/12.92:((x+.055)/1.055)**2.4});
   return .2126*v[0]+.7152*v[1]+.0722*v[2];
  };
  const ratio = (a,b) => {const [high,low]=[lum(a),lum(b)].sort((x,y)=>y-x);return (high+.05)/(low+.05)};
  const root=parse(getComputedStyle(document.documentElement).backgroundColor),ground=root?.[3]===1?root:[11,15,26,1];
  const background = el => {
   const layers=[],box=el.getBoundingClientRect(),cx=box.x+box.width/2,cy=box.y+box.height/2;
   for(let e=el;e && e!==document.documentElement;e=e.parentElement){
    const r=e.getBoundingClientRect();
    if(e!==el && (cx<r.left||cx>r.right||cy<r.top||cy>r.bottom))continue;
    const bg=parse(getComputedStyle(e).backgroundColor);
    if(bg && bg[3]>0){layers.push(bg);if(bg[3]===1)break}
   }
   return layers.reverse().reduce((base,top)=>over(top,base),ground);
  };
  const opacity = el => {let o=1;for(let e=el;e;e=e.parentElement)o*=+getComputedStyle(e).opacity;return o};
  const failures=[];
  for(const el of document.querySelectorAll(selector)){
   if(![...el.childNodes].some(n=>n.nodeType===3 && n.textContent.trim()))continue;
   const r=el.getBoundingClientRect(),cs=getComputedStyle(el);
   if(!r.width||!r.height||cs.visibility==='hidden'||cs.display==='none'||el.closest('.sr-only, [hidden]'))continue;
   const svg=el instanceof SVGElement;
   let fg=parse(svg?cs.fill:cs.color);
   if(!fg)continue;
   const bg=background(svg?el.closest('div')??el:el);
   fg=[...fg.slice(0,3),fg[3]*opacity(el)];
   const actual=ratio(over(fg,bg),bg),size=parseFloat(cs.fontSize),bold=+cs.fontWeight>=700;
   const needed=size>=24||(bold&&size>=18.66)?3:4.5;
   if(actual<needed)failures.push(`${actual.toFixed(2)} < ${needed} (${size}px) ${JSON.stringify(el.textContent.trim().slice(0,35))}`);
  }
  return [...new Set(failures)];
 }, selector);
}
