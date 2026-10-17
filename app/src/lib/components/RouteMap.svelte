<script lang="ts">
 let {route}:{route:[number,number][]}=$props();
 let lat=$derived(route.reduce((a,p)=>a+p[0],0)/Math.max(route.length,1));
 let cos=$derived(Math.cos(lat*Math.PI/180));
 let xValues=$derived(route.map(p=>p[1]*cos)),yValues=$derived(route.map(p=>p[0]));
 let minX=$derived(Math.min(...xValues)),maxX=$derived(Math.max(...xValues));
 let minY=$derived(Math.min(...yValues)),maxY=$derived(Math.max(...yValues));
 let scale=$derived(Math.min(310/(maxX-minX||1),128/(maxY-minY||1)));
 let width=$derived((maxX-minX)*scale),height=$derived((maxY-minY)*scale);
 let points=$derived(route.map(([latitude,longitude]):[number,number]=>[179-width/2+(longitude*cos-minX)*scale,88+height/2-(latitude-minY)*scale]));
 function pathFor(points:[number,number][]) {
  if(!points.length)return '';
  const pt=([x,y]:[number,number])=>`${x.toFixed(1)},${y.toFixed(1)}`;
  let path=`M${pt(points[0])}`;
  // A short route is a simplified polygon; soften corners without changing dense GPS traces.
  for(let i=0;i<points.length-1;i++){
   const [x0,y0]=points[i-1]??points[i], [x1,y1]=points[i], [x2,y2]=points[i+1], [x3,y3]=points[i+2]??points[i+1];
   path+=points.length<100?`C${pt([x1+(x2-x0)*.18,y1+(y2-y0)*.18])} ${pt([x2-(x3-x1)*.18,y2-(y3-y1)*.18])} ${pt(points[i+1])}`:`L${pt(points[i+1])}`;
  }
  return path;
 }
 let path=$derived(pathFor(points));
</script>
<svg class="map" viewBox="0 0 358 176" role="img" aria-label="Workout route">{#if route.length>1}<path d={path} stroke="#E4E8F0" stroke-opacity=".12" stroke-width="13" fill="none" stroke-linecap="round" stroke-linejoin="round"/><path d={path} stroke="#E4E8F0" stroke-width="4.5" fill="none" stroke-linecap="round" stroke-linejoin="round"/><circle cx={points[0][0]} cy={points[0][1]} r="7" fill="#E4E8F0" stroke="#0E1424" stroke-width="3"/>{:else}<text x="179" y="91" fill="#8E98AC" text-anchor="middle">No route recorded</text>{/if}</svg>
<style>.map{width:100%;display:block;background:#0E1424;border-radius:16px}</style>
