// A chart's tick labels are optional: on a narrow screen or with large system
// text, a tick that would touch another label is hidden rather than stacked or
// shrunk. Labels marked `data-thin` give way, in order of their value (higher
// first, then document order); every other text in the chart and every
// `data-mark` marker always stays.
type Box={l:number,r:number,t:number,b:number};
const GAP=3;
const hit=(a:Box,b:Box)=>a.l<b.r+GAP && b.l<a.r+GAP && a.t<b.b && b.t<a.b;

function textBoxes(node:Text):Box[] {
 const range=document.createRange();
 range.selectNodeContents(node);
 const size=parseFloat(getComputedStyle(node.parentElement!).fontSize)||0;
 return [...range.getClientRects()].filter(r=>r.width>0&&r.height>0).map(r=>{
  // Line boxes include leading; the glyphs fill about the em box.
  const pad=Math.max(0,(r.height-size*scale(node.parentElement!))/2);
  return {l:r.left,r:r.right,t:r.top+pad,b:r.bottom-pad};
 });
}
const scale=(el:Element)=>{if(!(el instanceof SVGGraphicsElement))return 1;const m=el.getScreenCTM();return m?Math.min(Math.hypot(m.a,m.b),Math.hypot(m.c,m.d)):1};

export function thinLabels(root:Element, labels:Element[]) {
 const fixed:Box[]=[];
 const walker=document.createTreeWalker(root,NodeFilter.SHOW_TEXT);
 for(let n=walker.nextNode() as Text|null;n;n=walker.nextNode() as Text|null){
  const el=n.parentElement;
  if(!n.data.trim()||!el||el.closest('[data-thin]')||el.closest('title,desc,.sr-only')||!el.checkVisibility({visibilityProperty:true}))continue;
  fixed.push(...textBoxes(n));
 }
 for(const m of root.querySelectorAll('[data-mark]')){const r=m.getBoundingClientRect();if(r.width||r.height)fixed.push({l:r.left,r:r.right,t:r.top,b:r.bottom})}
 const order=labels.map((el,i)=>({el,i,p:Number((el as HTMLElement).dataset.thin)||0})).sort((a,b)=>b.p-a.p||a.i-b.i);
 for(const {el} of order){
  const boxes:Box[]=[];
  const walk=document.createTreeWalker(el,NodeFilter.SHOW_TEXT);
  for(let n=walk.nextNode() as Text|null;n;n=walk.nextNode() as Text|null)if(n.data.trim())boxes.push(...textBoxes(n));
  const hide=boxes.some(b=>fixed.some(f=>hit(b,f)));
  (el as HTMLElement).style.visibility=hide?'hidden':'';
  if(!hide)fixed.push(...boxes);
 }
}

export function declutter(root:HTMLElement|SVGElement) {
 let frame=0;
 const labels=()=>[...root.querySelectorAll('[data-thin]')];
 const run=()=>{frame=0;thinLabels(root,labels())};
 const schedule=()=>{if(!frame)frame=requestAnimationFrame(run)};
 const resize=new ResizeObserver(schedule);
 const observe=()=>{
  resize.disconnect();
  resize.observe(root);
  // Text that grows without the chart resizing (system font size) still re-runs.
  for(const el of root.querySelectorAll('*'))if([...el.childNodes].some(n=>n.nodeType===3))resize.observe(el);
  schedule();
 };
 const changes=new MutationObserver(observe);
 changes.observe(root,{childList:true,subtree:true,characterData:true});
 observe();
 void document.fonts?.ready.then(schedule);
 return ()=>{resize.disconnect();changes.disconnect();cancelAnimationFrame(frame)};
}
