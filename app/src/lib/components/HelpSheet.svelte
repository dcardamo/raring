<script lang="ts">
 import Icon from './Icon.svelte';
 let {close,more}:{close:()=>void,more:()=>void}=$props();
 const steps=['Your watch counts reps and guesses the exercise.','Pressing lap ends a set. Rests are separate.','Raring flags 0-rep sets, doubled sets and unnamed exercises.','Fix sets in Garmin Connect. Raring only rechecks them during the first 48 hours after the workout starts; later edits won’t appear, even after Refresh.'];
</script>
<svelte:window onkeydown={(e)=>{if(e.key==='Escape')close()}}/>
<button class="scrim" aria-label="Close help" onclick={close}></button>
<div class="sheet" role="dialog" aria-modal="true" aria-labelledby="sheet-title">
 <span class="grab"></span>
 <div class="sheet-h"><h2 id="sheet-title">How sets are counted</h2><button class="close" aria-label="Close" onclick={close}><Icon name="close" size={20}/></button></div>
 <ol>{#each steps as step,i}<li><span class="n">{i+1}</span>{step}</li>{/each}</ol>
 <div class="actions"><button class="primary" onclick={close}>Got it</button><button class="secondary" onclick={more}>More help</button></div>
</div>
<style>
 .scrim{position:fixed;inset:0;border:0;background:rgba(5,8,14,.62);z-index:20;padding:0}
 .sheet{position:fixed;left:0;right:0;bottom:0;z-index:21;max-width:560px;margin:0 auto;background:var(--card);border:1px solid var(--line);border-bottom:0;border-radius:26px 26px 0 0;padding:12px 20px calc(28px + env(safe-area-inset-bottom));max-height:90dvh;overflow:auto}
 .grab{display:block;width:40px;height:4px;border-radius:2px;background:rgba(255,255,255,.18);margin:0 auto 16px}
 .sheet-h{display:flex;justify-content:space-between;align-items:center;gap:12px}.sheet-h h2{min-width:0;overflow-wrap:anywhere}
 .close{flex:none;width:44px;height:44px;border-radius:50%;border:1px solid var(--line);background:none;display:grid;place-items:center;color:var(--text2)}
 ol{list-style:none;padding:0;margin:16px 0 20px;display:grid;gap:14px}
 li{display:grid;grid-template-columns:24px minmax(0,1fr);gap:12px;color:var(--text2);font-size:15px;line-height:1.4}
 li span{width:24px;height:24px;border-radius:50%;background:rgba(255,255,255,.07);display:grid;place-items:center;font-size:12px;color:var(--text)}
 .actions{display:grid;grid-template-columns:repeat(auto-fit,minmax(min(100%,9rem),1fr));gap:10px}
 .actions button{margin-top:0}
</style>
