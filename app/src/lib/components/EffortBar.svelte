<script lang="ts">
 import type {Effort} from '../generated/Effort';
 // Garmin's effort call as one continuous chevron bar: a display, never buttons.
 let {effort}: {effort: Effort} = $props();
 const EFFORTS = ['rest', 'easy', 'steady', 'push'] as const;
</script>
<div class="effort" role="img" aria-label={`Suggested effort from Garmin readiness: ${effort}`}><small>Suggested effort · from Garmin readiness</small>
 <div class="ebar">{#each EFFORTS as e, i}<span class="sw" class:on={e === effort}><i class="seg s{i}"></i><em>{e}</em></span>{/each}</div></div>
<style>
 .effort{display:grid;gap:4px;margin:8px 4px 6px}.effort>small{font:500 11px Inter;color:var(--text2)}
 .ebar{display:flex;align-items:flex-end}.sw{flex:1;display:grid;gap:5px;justify-items:center;margin-left:-6px}.sw:first-child{margin-left:0}
 .seg{display:block;width:100%;height:16px;clip-path:polygon(0 0,calc(100% - 9px) 0,100% 50%,calc(100% - 9px) 100%,0 100%,9px 50%);opacity:.5}
 .sw:first-child .seg{clip-path:polygon(0 0,calc(100% - 9px) 0,100% 50%,calc(100% - 9px) 100%,0 100%);border-radius:6px 0 0 6px}.sw:last-child .seg{clip-path:polygon(0 0,100% 0,100% 100%,0 100%,9px 50%);border-radius:0 6px 6px 0}
 .s0{background:linear-gradient(90deg,#6F95FF,#5FB8D8)}.s1{background:linear-gradient(90deg,#5FB8D8,#3DDC97)}.s2{background:linear-gradient(90deg,#3DDC97,#FFB547)}.s3{background:linear-gradient(90deg,#FFB547,#FF6B6B)}
 .sw em{font:600 12px Inter;font-style:normal;text-transform:capitalize;color:var(--text2)}.sw.on .seg{opacity:1;height:21px;box-shadow:0 0 0 2px var(--ink)}.sw.on em{color:var(--text);font-weight:700}.sw.on em::before{content:'▲ ';font-size:9px;color:var(--ink)}
</style>
