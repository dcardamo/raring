<script lang="ts">
 import type {GoalKind} from '../generated/GoalKind';
 import {GOALS} from '../goals';
 // The colour key for a grid of goal rings: only the goals drawn, in ring order.
 let {kinds}: {kinds: GoalKind[]} = $props();
 const ORDER: GoalKind[] = ['strength', 'zone2', 'highIntensity', 'intensity', 'sleepTiming'];
 let shown = $derived(ORDER.filter(k => kinds.includes(k)));
</script>
{#if shown.length}<p class="ring-legend">{#each shown as k}<span><i style:background={GOALS[k].color}></i>{GOALS[k].name}</span>{/each}</p>{/if}
<style>
 .ring-legend{display:flex;flex-wrap:wrap;gap:4px 12px;margin:8px 2px 0;font-size:12px;color:var(--text2)}
 .ring-legend span{display:flex;align-items:center;gap:5px}.ring-legend i{width:9px;height:9px;border-radius:50%;display:inline-block}
</style>
