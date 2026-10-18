<script lang="ts">
 // A trend line that never bridges a gap: each run of readings is drawn and
 // filled on its own. Sparse estimates (VO₂ max, Fitness Age) are dots with
 // their values, since nothing is known between two estimates.
 let {values, w = 140, h = 40, color = '#E4E8F0', sparse = false, decimals = 0}: {values: (number | null)[], w?: number, h?: number, color?: string, sparse?: boolean, decimals?: number} = $props();
 const id = $props.id();
 let known = $derived(values.map((v, i) => [i, v] as const).filter((p): p is readonly [number, number] => p[1] != null && Number.isFinite(p[1])));
 let lo = $derived(Math.min(...known.map(p => p[1]))), hi = $derived(Math.max(...known.map(p => p[1])));
 const pad = $derived(sparse ? 12 : 4);
 const X = (i: number) => 4 + i * (w - 8) / Math.max(values.length - 1, 1);
 const Y = (v: number) => h - 4 - (v - lo) / (hi - lo || 1) * (h - 4 - pad);
 let runs = $derived.by(() => {
  const out: (readonly [number, number])[][] = [];
  let cur: (readonly [number, number])[] = [];
  values.forEach((v, i) => { if (v == null || !Number.isFinite(v)) { if (cur.length) out.push(cur); cur = []; } else cur.push([i, v]); });
  if (cur.length) out.push(cur);
  return out;
 });
 // Label only where the estimate changed, plus the latest.
 let labelled = $derived(known.filter((p, k) => k === known.length - 1 || k === 0 || p[1] !== known[k - 1][1]));
</script>
<svg width={w} height={h} viewBox={`0 0 ${w} ${h}`} class="tsp" aria-hidden="true">
 <defs><linearGradient id="f{id}" x1="0" y1="0" x2="0" y2="1"><stop stop-color={color} stop-opacity=".28"/><stop offset="1" stop-color={color} stop-opacity="0"/></linearGradient></defs>
 {#if sparse}
  <line x1="4" x2={w - 4} y1={h - 3} y2={h - 3} stroke="rgba(255,255,255,.08)"/>
  {#each known as [i, v]}<circle cx={X(i)} cy={Y(v)} r="3" fill={color}/>{/each}
  {#each labelled as [i, v]}<text x={Math.min(w - 10, Math.max(10, X(i)))} y={Y(v) - 6} font-size="9.5" fill="#B4BCCB" text-anchor="middle" font-weight="600">{v.toFixed(decimals)}</text>{/each}
 {:else}
  {#each runs as run}
   {#if run.length === 1}<circle cx={X(run[0][0])} cy={Y(run[0][1])} r="2" fill={color}/>
   {:else}
    {@const d = run.map(([i, v], k) => `${k ? 'L' : 'M'}${X(i).toFixed(1)},${Y(v).toFixed(1)}`).join('')}
    <path d={`${d}L${X(run.at(-1)![0]).toFixed(1)},${h}L${X(run[0][0]).toFixed(1)},${h}Z`} fill={`url(#f${id})`}/>
    <path {d} fill="none" stroke={color} stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
   {/if}
  {/each}
  {#if known.length}<circle cx={X(known.at(-1)![0])} cy={Y(known.at(-1)![1])} r="3.4" fill={color} stroke="#0B0F1A" stroke-width="2"/>{/if}
 {/if}
</svg>
<style>.tsp{display:block;max-width:100%;overflow:visible}</style>
