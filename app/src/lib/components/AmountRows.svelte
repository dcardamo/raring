<script lang="ts" module>
 export type AmountRow={label:string;value:number|null;text:string;detail?:string;color?:string;comparison?:number|null;comparisonText?:string;accessibleLabel?:string;band?:[number,number]|null};
</script>
<script lang="ts">
 import {plotAxis} from '../plot';
 import {clamp} from '../charts';
 // Comparators are supplied, never calculated here. All rows share a true zero.
 let {rows,label,unit='',format=(n:number)=>String(Math.round(n)),referenceLabel=''}:{
  rows:AmountRow[];label:string;unit?:string;format?:(n:number)=>string;referenceLabel?:string;
 }=$props();
 let axis=$derived(plotAxis(rows.flatMap(r=>[r.value,r.comparison??null,...(r.band??[])]),{zero:true,integer:unit==='sets',unit:unit==='h:mm'||unit==='min:sec'?'min':''}));
 let top=$derived(axis.scale[1]);
 const position=(v:number)=>clamp(v/top)*100+'%';
</script>
<div class="amount-rows" data-chart="amount-rows" role="group" aria-label={label}>
 <div class="axis"><span>{format(0)}</span>{#if unit}<span class="axis-unit">{unit}</span>{/if}<span>{format(top)}</span></div>
 {#each rows as row}
  <div class="amount-row" role={row.accessibleLabel?'group':undefined} aria-label={row.accessibleLabel}>
   <div class="heading"><span>{row.label}{#if row.detail}<small>{row.detail}</small>{/if}</span><b class="n">{row.value==null?'Not recorded':row.text}</b></div>
   <div class="rail" aria-hidden="true">
    {#if row.band}<span class="band" style:left={position(row.band[0])} style:width={position(row.band[1]-row.band[0])}></span>{/if}
    {#if row.value!=null && row.value>0}<span data-mark="amount" class="amount" style:width={position(row.value)} style:background={row.color??'var(--amount)'}></span>{/if}
    {#if row.comparison!=null}<span data-mark="comparison" class="comparison" style:left={position(row.comparison)}></span>{/if}
   </div>
   {#if row.comparisonText}<p class="comparison-text">{row.comparisonText}</p>{/if}
  </div>
 {/each}
 {#if referenceLabel}<p class="key">{referenceLabel}</p>{/if}
</div>
<style>
 .amount-rows{min-width:0}
 .axis{display:flex;justify-content:space-between;gap:12px;color:var(--text2);font:400 11px/1.5 Inter;margin:0 0 8px}
 .axis-unit{font-size:12px}
 .amount-row+.amount-row{margin-top:16px}
 .heading{display:flex;justify-content:space-between;align-items:baseline;gap:4px 12px;flex-wrap:wrap;font:500 13px/1.5 Inter;margin-bottom:8px}
 .heading>span{min-width:0;overflow-wrap:anywhere}
 .heading small{display:block;font:400 12px/1.5 Inter;color:var(--text2)}
 .heading b{font-size:18px;font-weight:500;line-height:1.4;overflow-wrap:anywhere}
 .rail{position:relative;height:10px;background:var(--surface2);border-radius:3px}
 .amount{position:absolute;inset:2px auto 2px 0;border-radius:2px}
 .band{position:absolute;top:0;bottom:0;background:var(--chart-band);border:1px solid var(--text3);box-sizing:border-box}
 .comparison{position:absolute;top:-3px;bottom:-3px;border-left:2px dashed var(--ink);transform:translateX(-1px)}
 .comparison-text,.key{margin:8px 0 0;font:400 12px/1.5 Inter;color:var(--text2);overflow-wrap:anywhere}
 .key{margin-top:16px}
</style>
