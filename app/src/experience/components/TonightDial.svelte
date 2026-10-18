<script lang="ts">
 import {toMin, hm} from '../lib/dates';
 // A 24-hour dial for tonight: now, wind-down, Garmin's window and the expected
 // wake, with last night's sleep as a faint arc for reference. Midnight at the top.
 let {now, windDown, windowStart, windowEnd, wake, lastBed, lastWake, need, size = 176, estimate = false}: {
  now: string, windDown: string, windowStart: string, windowEnd: string, wake: string, lastBed?: string | null, lastWake?: string | null, need: string, size?: number, estimate?: boolean,
 } = $props();
 const c = $derived(size / 2), r = $derived(size / 2 - 14);
 const ang = (t: string) => toMin(t) / 1440 * Math.PI * 2 - Math.PI / 2;
 const pt = (t: string, rr: number) => [c + Math.cos(ang(t)) * rr, c + Math.sin(ang(t)) * rr];
 function arc(a: string, b: string, rr: number) {
  const [x1, y1] = pt(a, rr), [x2, y2] = pt(b, rr);
  let span = toMin(b) - toMin(a); if (span < 0) span += 1440;
  return `M${x1.toFixed(1)},${y1.toFixed(1)} A${rr},${rr} 0 ${span > 720 ? 1 : 0} 1 ${x2.toFixed(1)},${y2.toFixed(1)}`;
 }
 let nowP = $derived(pt(now, r));
 let nowIn = $derived(pt(now, r - 16)), nowOut = $derived(pt(now, r + 2));
 let nowLabel = $derived(pt(now, r + 11));
 let wakeP = $derived(pt(wake, r));
</script>
<svg class="dial" width={size} height={size} viewBox={`0 0 ${size} ${size}`} role="img" aria-label={`Tonight: now ${now}, wind down from ${windDown}, ${estimate ? 'asleep by' : "Garmin's window"} ${windowStart}${estimate ? '' : ` to ${windowEnd}`}, expected wake ${wake}`}>
 <defs><radialGradient id="dialglow"><stop offset=".55" stop-color="#2F6FE0" stop-opacity=".16"/><stop offset="1" stop-color="#2F6FE0" stop-opacity="0"/></radialGradient></defs>
 <circle cx={c} cy={c} r={r + 12} fill="url(#dialglow)"/>
 <circle cx={c} cy={c} r={r} fill="none" stroke="rgba(255,255,255,.07)" stroke-width="10"/>
 {#each Array(24) as _, i}{@const a = i / 24 * Math.PI * 2 - Math.PI / 2}<line x1={c + Math.cos(a) * (r - 13)} y1={c + Math.sin(a) * (r - 13)} x2={c + Math.cos(a) * (r - (i % 6 ? 16 : 20))} y2={c + Math.sin(a) * (r - (i % 6 ? 16 : 20))} stroke="rgba(255,255,255,.2)" stroke-width={i % 6 ? 1 : 1.6}/>{/each}
 {#if lastBed && lastWake}<path d={arc(lastBed, lastWake, r - 26)} stroke="#5C9BFF" stroke-opacity=".28" stroke-width="5" fill="none" stroke-linecap="round"/>{/if}
 <path d={arc(windowEnd, wake, r)} stroke="#2F6FE0" stroke-opacity=".55" stroke-width="10" fill="none"/>
 <path d={arc(windDown, windowStart, r)} stroke="#FFB547" stroke-opacity=".55" stroke-width="10" fill="none"/>
 <path d={arc(windowStart, estimate ? hm(toMin(windowStart) + 6) : windowEnd, r)} stroke="#A9C8FF" stroke-width="10" fill="none"/>
 <circle cx={wakeP[0]} cy={wakeP[1]} r="5" fill="#EEF1F7" stroke="#0B0F1A" stroke-width="2"/>
 <line x1={nowIn[0]} y1={nowIn[1]} x2={nowOut[0]} y2={nowOut[1]} stroke="#EEF1F7" stroke-width="2.5" stroke-linecap="round"/>
 <circle cx={nowP[0]} cy={nowP[1]} r="6" fill="#EEF1F7" stroke="#0B0F1A" stroke-width="2"/>
 <text x={nowLabel[0]} y={nowLabel[1] + 3} text-anchor="middle" font-size="8.5" fill="#EEF1F7" font-weight="700">NOW</text>
 {#each [['00:00', '0'], ['06:00', '6'], ['12:00', '12'], ['18:00', '18']] as [t, l]}{@const p = pt(t, r - 25)}<text x={p[0]} y={p[1] + 4} text-anchor="middle" font-size="10" fill="#8E98AC" font-weight="600">{l}</text>{/each}
 <text x={c} y={c - 4} text-anchor="middle" font-size="9" fill="#8E98AC" font-weight="600" letter-spacing="1.2">NEED</text>
 <text x={c} y={c + 16} text-anchor="middle" font-size="17" fill="#EEF1F7" font-family="Outfit" font-weight="600">{need}</text>
</svg>
<style>.dial{display:block;flex:none}</style>
