<script lang="ts">
 import type {SelfEvaluation} from '../generated/SelfEvaluation';
 import {FEELS,FEEL_WORD} from '../workoutFields';
 // The person's own answers on the watch: Garmin's five feel answers as a
 // choice, perceived effort as an amount out of 10. Neutral blue, no verdict.
 let {rating}:{rating:SelfEvaluation}=$props();
 let step=$derived(rating.feel?FEELS.indexOf(rating.feel):-1);
</script>
<section class="card felt" aria-labelledby="felt-h">
 <h3 id="felt-h">How it felt</h3>
 {#if rating.feel}
  <div class="rating" role="img" aria-label={`Feel: ${FEEL_WORD[rating.feel]}, ${step+1} of 5`}>
   <div class="rating-head"><span>Feel</span><b>{FEEL_WORD[rating.feel]}</b></div>
   <div class="steps feel">{#each FEELS as f,i (f)}<i class:on={i===step}></i>{/each}</div>
  </div>
 {/if}
 {#if rating.effort!=null}
  <div class="rating" role="img" aria-label={`Perceived effort: ${rating.effort} of 10`}>
   <div class="rating-head"><span>Perceived effort</span><b class="n">{rating.effort}<small>/10</small></b></div>
   <div class="steps effort">{#each Array.from({length:10},(_,i)=>i) as i (i)}<i class:on={i<rating.effort}></i>{/each}</div>
  </div>
 {/if}
 <p class="small">Your own rating on the watch</p>
</section>
<style>
 .rating+.rating{margin-top:14px}
 .rating-head{display:flex;justify-content:space-between;align-items:baseline;gap:8px;flex-wrap:wrap;margin-bottom:7px}
 .rating-head span{font-size:13px;color:var(--text2)}.rating-head b{font:600 16px Outfit,Inter}.rating-head small{font:500 12px Inter;color:var(--text3)}
 .steps{display:grid;gap:4px}.steps i{display:block;height:10px;border-radius:5px;background:rgba(255,255,255,.08)}
 .steps.feel{grid-template-columns:repeat(5,1fr)}.steps.effort{grid-template-columns:repeat(10,1fr)}
 .steps i.on{background:var(--amount)}
 .small{margin-top:12px}
</style>
