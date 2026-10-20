<script lang="ts">
 import {colors} from '../charts';
 import {elapsed} from '../format';
 let {seconds}:{seconds:(number|null)[]}=$props();
 let complete=$derived(seconds.every(s=>s!=null));
 // A partial set of zones must not be normalized to look like a complete workout.
 let total=$derived(complete?seconds.reduce<number>((a,b)=>a+(b??0),0):0);
 let label=$derived(seconds.map((s,i)=>`Zone ${i+1}: ${s==null?'not recorded':elapsed(s)}`).join('; '));
</script>
{#if complete}<div class="zbar" role="img" aria-label={label}>{#each seconds as s,i}<i style:width={(total?(s??0)/total*100:0)+'%'} style:background={colors.z[i]??colors.secondary}></i>{/each}</div>
{:else}<p class="incomplete">Zone times incomplete</p>{/if}
<style>
 .zbar{display:flex;height:8px;border-radius:4px;overflow:hidden;gap:2px;margin-top:14px}
 i{display:block;height:100%}
 .incomplete{font:400 12px/1.5 Inter;color:var(--text2);margin:8px 0 0}
</style>
