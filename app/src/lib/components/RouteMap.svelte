<script lang="ts">
 import type {RouteMapView} from '../generated/RouteMapView';
 import * as api from '../api';
 import {fit,mercator} from '../routemap';
 let {route,map=null,id=''}:{route:[number,number][],map?:RouteMapView|null,id?:string}=$props();
 // A card whose area isn't drawn yet asks once while it shows; the core downloads each area at most once.
 let drawn=$state<{id:string,map:RouteMapView|null}|null>(null);
 $effect(()=>{
  if(map?.state!=='missing' || !id || drawn?.id===id)return;
  const asked=id;
  api.route_map(asked).then(next=>{if(asked===id)drawn={id:asked,map:next}},()=>{if(asked===id)drawn={id:asked,map}});
 });
 // The core's answer wins only while the page still has a map; none means street maps are off.
 let shown=$derived(map&&drawn?.id===id?drawn.map:map);
 let layers=$derived(shown?.state==='ready'?shown.layers:[]);
 let projected=$derived(route.map(mercator).filter((p):p is [number,number]=>p!==null));
 let frame=$derived(shown?.frame??fit(projected));
 let points=$derived(frame?projected.map(([x,y]):[number,number]=>[(x-frame.x0)*frame.scale,(y-frame.y0)*frame.scale]):[]);
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
<div class="route-map"><svg class="map" viewBox="0 0 358 176" role="img" aria-label={layers.length?'Workout route on a street map':'Workout route'}>{#each layers as layer}<path class="layer {layer.kind}" d={layer.path}/>{/each}{#if points.length>1}<path d={path} stroke="var(--ink)" stroke-opacity=".12" stroke-width="13" fill="none" stroke-linecap="round" stroke-linejoin="round"/><path d={path} stroke="var(--ink)" stroke-width="4.5" fill="none" stroke-linecap="round" stroke-linejoin="round"/><circle cx={points[0][0]} cy={points[0][1]} r="7" fill="var(--ink)" stroke="var(--card)" stroke-width="3"/>{:else}<text x="179" y="91" fill="var(--text3)" text-anchor="middle">No route recorded</text>{/if}</svg>{#if layers.length}<span class="credit">© OpenMapTiles © OpenStreetMap</span>{/if}</div>
<style>
 .route-map{position:relative}
 .map{width:100%;display:block;background:var(--card);border-radius:16px}
 .layer{fill:none;stroke-linecap:round;stroke-linejoin:round}
 .green{fill:var(--good);fill-opacity:.07}
 .water{fill:var(--amount);fill-opacity:.2}
 .waterway{stroke:var(--amount);stroke-opacity:.3;stroke-width:1}
 .path{stroke:var(--ink);stroke-opacity:.09;stroke-width:.6}
 .minor{stroke:var(--ink);stroke-opacity:.1;stroke-width:.9}
 .road{stroke:var(--ink);stroke-opacity:.13;stroke-width:1.5}
 .major{stroke:var(--ink);stroke-opacity:.16;stroke-width:2.2}
 .credit{position:absolute;right:8px;bottom:6px;padding:1px 6px;border-radius:6px;background:color-mix(in srgb,var(--card) 80%,transparent);font:500 11px/1.3 var(--font-body);color:var(--text3)}
</style>
