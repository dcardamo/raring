<script lang="ts">
 import type {Effort} from '../generated/Effort';
 // One shared rail keeps the chevron seams aligned when the selected level rises.
 // Labels use their own equal-width grid, so text zoom cannot distort the marks.
 let {effort}: {effort: Effort} = $props();
 const EFFORTS = ['rest', 'easy', 'steady', 'push'] as const;
 const id = $props.id();
 let selected = $derived(EFFORTS.indexOf(effort));
 const segment = (i: number) => {
  const left = i * 100, right = left + 100;
  return `M${left} 6H${right}${i < 3 ? `L${right + 10} 14L${right} 22` : 'V22'}H${left}${i > 0 ? `L${left + 10} 14` : ''}Z`;
 };
</script>
<div class="effort" role="img" aria-label={`Suggested effort from Garmin readiness: ${effort}`}>
 <small>Suggested effort · from Garmin readiness</small>
 <div class="ebar">
  <svg viewBox="0 0 400 24" preserveAspectRatio="none" aria-hidden="true">
   <defs><clipPath id="rail-{id}"><rect y="6" width="400" height="16" rx="4"/>
    <!-- A raised end cap supplies its own top corner; don't round the join beneath it. -->
    {#if selected === 0 || selected === 3}<rect x={selected === 0 ? 0 : 396} y="6" width="4" height="4"/>{/if}
   </clipPath></defs>
   <g clip-path={`url(#rail-${id})`}>
    {#each EFFORTS as e, i}<path class="seg s{i}" class:on={e === effort} d={segment(i)}/>{/each}
   </g>
   <path class="cap s{selected}" d={`M${selected * 100} 6Q${selected * 100} 2 ${selected * 100 + 4} 2H${selected * 100 + 96}Q${selected * 100 + 100} 2 ${selected * 100 + 100} 6Z`}/>
  </svg>
  <div class="labels">{#each EFFORTS as e}<span class="sw" class:on={e === effort}><em>{e}</em></span>{/each}</div>
 </div>
</div>
<style>
 .effort{display:grid;gap:4px;margin:8px 4px 6px}.effort>small{font:500 11px Inter;color:var(--text2)}
 .ebar{display:grid;gap:8px}svg{display:block;width:100%;height:24px;overflow:visible}
 .seg{opacity:.55}.seg.on{opacity:1}
 .s0{fill:var(--text3)}.s1{fill:var(--amount)}.s2{fill:var(--fair)}.s3{fill:var(--poor)}
 .labels{display:grid;grid-template-columns:repeat(4,minmax(0,1fr))}
 .sw{position:relative;min-width:0;text-align:center;overflow-wrap:anywhere}
 .sw em{font:500 12px/1.2 Inter;font-style:normal;text-transform:capitalize;color:var(--text2)}
 .sw.on em{color:var(--text);font-weight:700}
 .sw.on::before{content:'';position:absolute;top:-7px;left:50%;transform:translateX(-50%);border-left:4px solid transparent;border-right:4px solid transparent;border-bottom:5px solid var(--ink)}
</style>
