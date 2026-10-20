<script lang="ts">
 // Formatting, dates and comparison arithmetic belong to the caller/core.
 let {label,value,unit='',context='',small=false}:{label:string;value:string;unit?:string;context?:string;small?:boolean}=$props();
</script>
<div class="metric-summary" class:small data-chart="metric-summary">
 <span class="label">{label}</span>
 <div class="reading"><b class="n">{value}</b>{#if unit}<span class="unit">{unit}</span>{/if}</div>
 {#if context}<span class="context">{context}</span>{/if}
</div>
<style>
 .metric-summary{min-width:0}
 .label,.context,.unit{font:400 12px/1.5 Inter;color:var(--text2);overflow-wrap:anywhere}
 .label,.context{display:block}
 .reading{display:flex;align-items:baseline;gap:5px;flex-wrap:wrap}
 b{font-size:32px;font-weight:500;line-height:1.2;letter-spacing:-.025em;color:var(--ink);overflow-wrap:anywhere}
 .small b{font-size:23px}
</style>
