<script lang="ts">
 import Icon from '../../lib/components/Icon.svelte';
 import {store} from '../lib/store.svelte';
 import {allocations, goalsFor} from '../lib/allocation';
 let {compact=false}:{compact?:boolean}=$props();
 let rows=$derived(allocations(store.ctx)), goals=$derived(goalsFor(store.ctx));
</script>
<section class="goal-ledger" aria-label="Weekly goals">
 {#each rows as g}
  <div class="goal" style:--gc={g.color}>
   <div class="gh"><span class="gi"><Icon name={g.key==='strength'?'strength':g.key==='sleep'?'moon':g.key==='zone2'?'pulse':'flame'} size={18}/></span><b>{g.name}</b><span class="target n">{g.target}<small> {g.unit}/week</small></span></div>
   {#if g.key==='sleep'}
    <div class="timing"><b class="n">{goals.sleepTime}</b><span>{goals.sleepKind==='winddown'?'Wind down':'Bedtime'} · your choice</span></div>
    <div class="allocation-bar" aria-label={`${g.planned} nights planned`}><i class="planned" style:width={Math.min(100,g.planned/g.target*100)+'%'}></i></div>
    <div class="amounts"><span><b class="n">{g.planned}</b><small>planned ahead</small></span><span><b class="n">{g.remaining ?? '—'}</b><small>not yet placed</small></span></div>
   {:else}
    <div class="allocation-bar" role="img" aria-label={`${g.recorded} recorded, ${g.planned} planned; target ${g.target}`}><i class="recorded" style:width={Math.min(100,(g.recorded??0)/g.target*100)+'%'}></i><i class="planned" style:width={Math.max(0,Math.min(100-(g.recorded??0)/g.target*100,g.planned/g.target*100))+'%'}></i></div>
    <div class="amounts"><span><b class="n">{g.recorded}</b><small>{g.complete?'recorded':'known'}</small></span><span><b class="n">{g.planned}</b><small>planned</small></span><span class="left"><b class="n">{g.remaining ?? '—'}</b><small>{!g.complete?'awaiting data':g.paused?'paused':g.ambiguous?'review link':'not yet placed'}</small></span></div>
   {/if}
   {#if !compact}<button class="place" onclick={()=>store.openSheet('allocate',g.key)} disabled={g.paused}><span>{g.key==='sleep'?'Choose nights':g.remaining ? `Place ${g.key==='strength'?'a session':g.key==='zone2'?'Zone 2 time':'intensity time'}`:'Place a session'}</span><Icon name="arrow-right" size={14}/></button>{/if}
  </div>
 {:else}
  <div class="no-goals"><b>Make this week yours</b><p>Choose a goal, then give it a place in your week.</p><button class="primary" onclick={()=>store.openSheet('goals')}>Choose weekly goals</button></div>
 {/each}
</section>
<style>
 .goal-ledger{display:grid;gap:10px}.goal{border:1px solid var(--line);background:linear-gradient(125deg,color-mix(in srgb,var(--gc) 7%,transparent),var(--card));border-radius:20px;padding:14px}
 .gh{display:flex;align-items:center;gap:9px}.gi{display:grid;place-items:center;width:30px;height:30px;background:color-mix(in srgb,var(--gc) 14%,transparent);color:var(--gc);border-radius:10px}.gh b{font:600 17px Outfit}.target{margin-left:auto;font-size:22px;font-weight:600}.target small{font:500 11px Inter;color:var(--text3)}
 .allocation-bar{height:8px;margin:12px 0 9px;border-radius:4px;display:flex;overflow:hidden;background:rgba(255,255,255,.07)}.allocation-bar i{display:block;height:100%;flex:none}.recorded{background:var(--gc)}.planned{background:repeating-linear-gradient(110deg,color-mix(in srgb,var(--gc) 50%,transparent) 0 3px,transparent 3px 6px);border:1px solid color-mix(in srgb,var(--gc) 50%,transparent)}
 .amounts{display:flex;gap:10px;justify-content:space-between}.amounts span{display:flex;align-items:baseline;gap:5px;flex-wrap:wrap}.amounts b{font-size:18px;font-weight:600}.amounts small{font:500 11px Inter;color:var(--text2)}.left b{color:var(--gc)}
 .place{display:flex;align-items:center;justify-content:space-between;border:0;border-top:1px solid var(--line);margin-top:12px;padding:10px 0 0;width:100%;background:none;color:var(--text);font:600 13px Inter;min-height:34px}.place:disabled{opacity:.4}.timing{display:flex;align-items:baseline;gap:10px;margin-top:8px}.timing b{font-size:24px}.timing span{font-size:12px;color:var(--text2)}
 .no-goals{padding:20px;border:1px dashed var(--line);border-radius:22px;background:var(--card)}.no-goals b{font:600 22px Outfit}.no-goals p{color:var(--text2);margin-top:6px}
</style>
