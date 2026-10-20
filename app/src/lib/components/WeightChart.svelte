<script lang="ts">
 import {smoothPath} from '../curve';
 import type {WeightPoint} from '../generated/WeightPoint';
 import {dateLabel} from '../format';
 import {declutter} from '../declutter';
 let {points,unit,color='#8FA8FF'}:{points:WeightPoint[],unit:string,color?:string}=$props();
 let width=$state(320),index=$state(0);
 $effect(()=>{index=Math.max(0,points.length-1)});
 const H=170,L=44,R=10,T=12,B=25;
 let values=$derived(points.flatMap(p=>[p.scale,p.trend]).filter((v):v is number=>v!=null));
 let lo=$derived(values.length?Math.floor((Math.min(...values)-.3)*2)/2:0);
 let hi=$derived(values.length?Math.ceil((Math.max(...values)+.3)*2)/2:1);
 const stamp=(d:string)=>Date.parse(d+'T12:00:00Z');
 const x=(d:string)=>L+(width-L-R)*(points.length>1?(stamp(d)-stamp(points[0].date))/Math.max(1,stamp(points.at(-1)!.date)-stamp(points[0].date)):.5);
 const y=(v:number)=>T+(1-(v-lo)/Math.max(.001,hi-lo))*(H-T-B);
 let scalePath=$derived.by(()=>{
  const pts:[number,number|null][]=[];
  for(const p of points){if(p.scale==null)continue;if(p.breakScale)pts.push([0,null]);pts.push([x(p.date),y(p.scale)]);}
  return smoothPath(pts);
 });
 let trendPath=$derived(smoothPath(points.map(p=>[x(p.date),p.trend==null?null:y(p.trend)])));
 let selected=$derived(points[index]);
 let every=$derived(Math.max(1,Math.ceil(points.length/4)));
 const label=(p:WeightPoint)=>`${dateLabel(p.date)} ${p.date.slice(0,4)}: scale ${p.scale==null?'not recorded':p.scale.toFixed(1)+' '+unit}, trend ${p.trend==null?'unavailable':p.trend.toFixed(1)+' '+unit}`;
</script>
<div class="weight-chart" bind:clientWidth={width} data-chart="weight" {@attach declutter}>
 <span class="unit">{unit}</span>
 <svg width={width} height={H} role="img" aria-label={`Scale weight and Raring trend, ${unit}. Scale gaps are not weigh-ins.`}>
  {#each [lo,(lo+hi)/2,hi] as tick}<line x1={L} x2={width-R} y1={y(tick)} y2={y(tick)} stroke="rgba(255,255,255,.1)" stroke-dasharray="3 4"/><text x={L-5} y={y(tick)+4} text-anchor="end">{tick.toFixed(1)}</text>{/each}
  <path class="scale-line" d={scalePath} fill="none" stroke={color} stroke-opacity=".45" stroke-width="1.8"/>
  <path class="trend-line" d={trendPath} fill="none" stroke={color} stroke-width="2.6"/>
  {#each points as p}{#if p.scale!=null}<circle class="scale-mark" cx={x(p.date)} cy={y(p.scale)} r="2.4" fill={color} opacity=".55"/>{#if p.trend!=null}<circle class="trend-mark" cx={x(p.date)} cy={y(p.trend)} r="3" fill="#121827" stroke={color} stroke-width="1.8"/>{/if}{/if}{/each}
  {#each points as p,i}{#if i%every===0||i===points.length-1}<text data-thin={i===0||i===points.length-1?1:''} x={x(p.date)} y={H-5} text-anchor={i===points.length-1?'end':i===0?'start':'middle'}>{dateLabel(p.date).replace(/^\S+\s*,?\s*/,'')}</text>{/if}{/each}
 </svg>
 {#if selected}<input type="range" min="0" max={Math.max(0,points.length-1)} step="1" bind:value={index} aria-label="Weight readings" aria-valuetext={label(selected)}/><p class="readout" aria-live="polite">{label(selected)}</p>{/if}
</div>
<style>
 .weight-chart{width:100%;min-width:0}svg{display:block;max-width:100%}text{font:11px Inter,sans-serif;fill:var(--text2)}.unit{display:block;font:500 12px Inter;color:var(--text2);margin:0 0 2px}
 input{width:100%;accent-color:#8FA8FF;min-height:28px}.readout{font:500 12px Inter;color:var(--text2);margin:4px 0 8px}
</style>
