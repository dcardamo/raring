import type {MapFrame} from './generated/MapFrame';

/** Web Mercator, 0–1 across the world with y down; null off the map. */
export const mercator=([lat,lon]:[number,number]):[number,number]|null=>Number.isFinite(lat)&&Number.isFinite(lon)&&Math.abs(lat)<=85.05112878&&Math.abs(lon)<=180?[(lon+180)/360,(1-Math.asinh(Math.tan(lat*Math.PI/180))/Math.PI)/2]:null;

/** The core's street-map fit: the route fills at most 310×128 of the 358×176 card, zooming in no closer than a
 * street map can. Cards without a map use it too, so a map arriving never moves the route. */
export function fit(points:[number,number][]):MapFrame|null {
 if(points.length<2)return null;
 const xs=points.map(p=>p[0]),ys=points.map(p=>p[1]);
 const [minX,maxX,minY,maxY]=[Math.min(...xs),Math.max(...xs),Math.min(...ys),Math.max(...ys)];
 const room=(span:number,size:number)=>span>0?size/span:Infinity;
 const scale=Math.min(room(maxX-minX,310),room(maxY-minY,128),256*65536);
 return {x0:(minX+maxX)/2-179/scale,y0:(minY+maxY)/2-88/scale,scale};
}
