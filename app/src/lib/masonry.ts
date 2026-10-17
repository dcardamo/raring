// A grid row is as tall as its tallest card, which leaves a hole under the
// shorter one. With 1px grid rows, spanning each card's own height packs it
// under whichever column ends higher, keeping the reading order.
export function masonry(grid: HTMLElement) {
 const items=()=>grid.querySelectorAll<HTMLElement>(':scope>*,:scope>.row>*');
 const layout=()=>{
  const style=getComputedStyle(grid),gap=parseFloat(style.columnGap)||0,on=style.display==='grid';
  for(const item of items()){
   const end=on?`span ${Math.ceil(item.getBoundingClientRect().height+gap)}`:'';
   if(item.style.gridRowEnd!==end)item.style.gridRowEnd=end;
  }
 };
 const resize=new ResizeObserver(layout);
 const observe=()=>{resize.disconnect();items().forEach(item=>resize.observe(item))};
 const children=new MutationObserver(observe);
 children.observe(grid,{childList:true,subtree:true});
 observe();
 return ()=>{resize.disconnect();children.disconnect()};
}
