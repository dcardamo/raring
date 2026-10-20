<script lang="ts">
 import type {Effort} from '../generated/Effort';
 // Four joined chevrons share one rail; the suggested one is the same chevron drawn
 // taller above and below it, so selection never moves a seam or adds a tab.
 // Labels use their own equal-width grid, so text zoom cannot distort the marks.
 let {effort}: {effort: Effort} = $props();
 const EFFORTS = ['rest', 'easy', 'steady', 'push'] as const;
 const id = $props.id();
 let selected = $derived(EFFORTS.indexOf(effort));
 // Rail y 8–24; the suggested chevron spans y 3–29. Points and notches keep one depth.
 const DEPTH = 10;
 // The bar's outer ends are rounded like the rail, including on the taller chevron.
 const R = 4;
 const chevron = (i: number, top: number, bottom: number) => {
  const left = i * 100, right = left + 100, mid = (top + bottom) / 2;
  const start = i > 0 ? `M${left} ${top}` : `M${left + R} ${top}`;
  const tail = i < 3
   ? `H${right}L${right + DEPTH} ${mid}L${right} ${bottom}`
   : `H${right - R}Q${right} ${top} ${right} ${top + R}V${bottom - R}Q${right} ${bottom} ${right - R} ${bottom}`;
  const head = i > 0
   ? `H${left}L${left + DEPTH} ${mid}`
   : `H${left + R}Q${left} ${bottom} ${left} ${bottom - R}V${top + R}Q${left} ${top} ${left + R} ${top}`;
  return `${start}${tail}${head}Z`;
 };
</script>
<div class="effort" role="img" aria-label={`Suggested effort from Garmin readiness: ${effort}`}>
 <small>Suggested effort · from Garmin readiness</small>
 <div class="ebar">
  <svg viewBox="0 0 400 32" preserveAspectRatio="none" aria-hidden="true">
   <defs><clipPath id="rail-{id}"><rect y="8" width="400" height="16" rx="4"/></clipPath></defs>
   <g clip-path={`url(#rail-${id})`}>
    {#each EFFORTS as e, i}<path class="seg s{i}" d={chevron(i, 8, 24)}/>{/each}
   </g>
   <path class="pick s{selected}" d={chevron(selected, 3, 29)}/>
  </svg>
  <div class="labels">{#each EFFORTS as e}<span class="sw" class:on={e === effort}><em>{e}</em></span>{/each}</div>
 </div>
</div>
<style>
 .effort{display:grid;gap:4px;margin:8px 4px 6px}.effort>small{font:500 11px Inter;color:var(--text2)}
 .ebar{display:grid;gap:6px}svg{display:block;width:100%;height:32px;overflow:visible}
 .seg{opacity:.38}
 .pick{stroke:var(--bg);stroke-width:2;stroke-linejoin:round;vector-effect:non-scaling-stroke;paint-order:stroke}
 .s0{fill:var(--text3)}.s1{fill:var(--amount)}.s2{fill:var(--fair)}.s3{fill:var(--poor)}
 .labels{display:grid;grid-template-columns:repeat(4,minmax(0,1fr))}
 .sw{min-width:0;text-align:center;overflow-wrap:anywhere}
 .sw em{font:500 12px/1.2 Inter;font-style:normal;text-transform:capitalize;color:var(--text2)}
 .sw.on em{color:var(--text);font-weight:700}
</style>
