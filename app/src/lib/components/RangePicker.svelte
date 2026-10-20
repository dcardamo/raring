<script lang="ts">
 import {ranges} from '../trends';
 import type {TrendRange} from '../generated/TrendRange';
 // Shared metric range control. Full evidence details may additionally offer Dates.
 let {value,label,onchange,custom=false,local=false,nights=false,disabled=false}:{value:TrendRange,label:string,onchange:(r:TrendRange)=>void,custom?:boolean,local?:boolean,nights?:boolean,disabled?:boolean}=$props();
 let choices=$derived(ranges.filter(r=>local ? r.days!=null && r.days<=30 : custom || r.range!=='custom'));
 const name=(r:typeof ranges[number])=>r.days?`${r.days} ${nights?'nights':'days'}`:r.range==='year'?'1 year':r.range==='custom'?'Custom dates':'All history';
</script>
<div class="rp" role="group" aria-label={label}>
 {#each choices as r}<button {disabled} class:active={value===r.range} aria-pressed={value===r.range} aria-label={name(r)} onclick={()=>onchange(r.range)}>{local?name(r):r.label}</button>{/each}
</div>
<style>
 .rp{display:flex;flex-wrap:wrap;gap:3px;padding:3px;border:1px solid var(--line);border-radius:13px;max-width:100%;min-width:0}
 button{flex:1 0 auto;min-width:34px;min-height:44px;border:0;border-radius:10px;background:none;color:var(--text2);font:600 12px Inter;padding:0 7px}
 button.active{background:var(--ink);color:var(--bg)}
 button:disabled{cursor:default}
</style>
