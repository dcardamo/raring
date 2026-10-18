<script lang="ts">
 import Header from '../../lib/components/Header.svelte';
 import {store} from '../lib/store.svelte';
 // Prototype-only review aid: synthetic captures of the shipped app, the
 // retired v1 prototype and this v2 for the same three screens.
 const ROWS = [
  {name: 'Today', shipped: 'shipped-today.jpg', v1: 'v1-today.jpg', v2: 'v2-today.jpg'},
  {name: 'Workout', shipped: 'shipped-workout.jpg', v1: 'v1-workout.jpg', v2: 'v2-workout.jpg'},
  {name: 'Progress', shipped: 'shipped-training.jpg', v1: 'v1-progress.jpg', v2: 'v2-progress.jpg'},
 ];
 const base = import.meta.env.BASE_URL + 'compare/';
</script>
<Header title="Compare" eyebrow="Synthetic captures · shipped | v1 | v2" back={() => store.back()} backLabel={store.parentLabel}/>
<p class="small">The shipped app has no Progress tab; its Training screen is the closest progress view.</p>
{#each ROWS as r}
 <section class="cmp">
  <h2>{r.name}</h2>
  <div class="cols">
   {#each [['Shipped', r.shipped], ['v1 (retired)', r.v1], ['v2', r.v2]] as [label, file]}<figure><figcaption>{label}</figcaption><img src={base + file} alt={`${label} ${r.name}, synthetic data`} loading="lazy"/></figure>{/each}
  </div>
 </section>
{/each}
<style>
 .cmp{margin-top:20px}
 .cmp h2{font:600 18px Outfit;margin:0 4px 10px}
 .cols{display:grid;grid-template-columns:repeat(3,minmax(220px,1fr));gap:12px;overflow-x:auto}
 figure{margin:0;background:var(--card);border:1px solid var(--line);border-radius:16px;padding:8px}
 figcaption{font:600 12px Inter;color:var(--text2);margin:2px 4px 8px}
 img{display:block;width:100%;height:auto;border-radius:10px}
</style>
