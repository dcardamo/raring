<script lang="ts">
 import {figureArt,amountColor} from '../bodymap';
 import {muscleNames} from '../strength';
 import type {Figure} from '../generated/Figure';
 import type {MuscleSets} from '../generated/MuscleSets';
 let {figure,muscles,max=12,width=170,label='sets',legend=true}:{figure:Figure,muscles:MuscleSets[],max?:number,width?:number,label?:string,legend?:boolean}=$props();
 let art=$derived(figureArt(figure));
 let sets=$derived(Object.fromEntries(muscles.map(m=>[m.muscle,m.sets])) as Record<string,number>);
 let summary=$derived(muscles.filter(m=>m.sets>0).map(m=>`${muscleNames[m.muscle]} ${Math.round(m.sets*10)/10}`).join(', ')||'No sets');
</script>
<figure class="bodymap">
 <svg {width} height={Math.round(width*220/128)} viewBox="0 0 128 220" role="img" aria-label={`Muscles worked, front and back: ${summary}`}>
  {#each [['front',-18],['back',46]] as [side,dx]}
   {@const shapes=side==='front'?art.front:art.back}
   {#snippet half()}<path d={art.outline} fill="var(--surface3)"/>{#each shapes as s}<path d={s.d} fill={amountColor(sets[s.muscle]??0,max)}/>{/each}<path d={side==='front'?art.frontLines:art.backLines} fill="none" stroke="var(--bg)" stroke-width=".7" stroke-linecap="round" opacity=".55"/>{/snippet}
   <g transform={`translate(${dx} 0)`}><ellipse cx="50" cy="14" rx={art.head.rx} ry={art.head.ry} fill="var(--surface3)"/>{@render half()}<g transform="matrix(-1 0 0 1 100 0)">{@render half()}</g></g>
  {/each}
 </svg>
 {#if legend}<figcaption class="akey"><span class="n">0</span><i style:background={`linear-gradient(90deg,${amountColor(0,1)},${amountColor(0.01,1)} 12%,${amountColor(1,1)})`}></i><span class="n">{max}+ {label}</span></figcaption>{/if}
</figure>
<style>
 .bodymap{margin:0;display:flex;flex-direction:column;align-items:center}
 svg{display:block;max-width:100%;height:auto}
 .akey{display:flex;align-items:center;gap:10px;margin-top:12px;width:100%;font-size:11px;color:var(--text3)}
 .akey i{flex:1;height:6px;border-radius:3px}
</style>
