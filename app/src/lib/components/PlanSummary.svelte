<script lang="ts">
 import type {PlanWeek} from '../generated/PlanWeek';
 let {week,count}:{week:PlanWeek;count:number}=$props();
</script>

<div class="summary">
 <div class="n count">{week.total===0?'—':count}<small>of {week.items.length}</small></div>
 <div><b>{week.headline}</b><span>{week.detail}</span></div>
</div>

<style>
 .summary{display:flex;align-items:center;gap:18px;margin-bottom:20px}
 .count{font-size:56px;font-weight:600;line-height:.9;white-space:nowrap}
 .count small{font:500 17px Inter;color:var(--text2);margin-left:6px}
 .summary b{display:block;font:600 19px Outfit}
 .summary span{display:block;font-size:14px;color:var(--text2);margin-top:4px}
</style>
