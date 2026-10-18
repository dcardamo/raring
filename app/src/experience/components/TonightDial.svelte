<script lang="ts">
 import {hm, toMin} from '../lib/dates';
 // Tonight on a 24-hour dial, midnight at the top: wind-down, Garmin's window
 // and the expected sleep to waking, each labelled where it sits, and now.
 // Last night's sleep is a faint inner arc for reference.
 let {now, windDown, windowStart, windowEnd, wake, lastBed, lastWake, need, size = 176, estimate = false}: {
  now: string, windDown: string, windowStart: string, windowEnd: string, wake: string, lastBed?: string | null, lastWake?: string | null, need: string, size?: number, estimate?: boolean,
 } = $props();
 const c = $derived(size / 2), r = $derived(size / 2 - 30);
 const ang = (t: string) => toMin(t) / 1440 * Math.PI * 2 - Math.PI / 2;
 const pt = (t: string, rr: number) => [c + Math.cos(ang(t)) * rr, c + Math.sin(ang(t)) * rr];
 function arc(a: string, b: string, rr: number) {
  const [x1, y1] = pt(a, rr), [x2, y2] = pt(b, rr);
  let span = toMin(b) - toMin(a); if (span < 0) span += 1440;
  return `M${x1.toFixed(1)},${y1.toFixed(1)} A${rr},${rr} 0 ${span > 720 ? 1 : 0} 1 ${x2.toFixed(1)},${y2.toFixed(1)}`;
 }
 let nowP = $derived(pt(now, r));
 let nowL = $derived(pt(now, r - 19));
 let wdL = $derived(pt(windDown, r + 13));
 let winL = $derived(pt(windowEnd, r + 13));
 let wakeL = $derived(pt(wake, r + 13));
 const anchor = (t: string) => { const a = Math.cos(ang(t)); return a > .25 ? 'start' : a < -.25 ? 'end' : 'middle'; };
</script>
<svg class="dial" width={size} height={size} viewBox={`0 0 ${size} ${size}`} role="img" aria-label={`Tonight: now ${now}; wind down from ${windDown}; ${estimate ? 'asleep by' : "Garmin's window"} ${windowStart}${estimate ? '' : ` to ${windowEnd}`}; expected wake ${wake}; need ${need}`}>
 <defs><radialGradient id="dialglow"><stop offset=".6" stop-color="#2F6FE0" stop-opacity=".14"/><stop offset="1" stop-color="#2F6FE0" stop-opacity="0"/></radialGradient></defs>
 <circle cx={c} cy={c} r={r + 16} fill="url(#dialglow)"/>
 <circle cx={c} cy={c} r={r} fill="none" stroke="rgba(255,255,255,.07)" stroke-width="10"/>
 {#each [['00:00', '0'], ['06:00', '6'], ['12:00', '12'], ['18:00', '18']].filter(([t]) => Math.abs(((toMin(t) - toMin(now) + 720) % 1440 + 1440) % 1440 - 720) > 60) as [t, l]}{@const p = pt(t, r - 17)}<text x={p[0]} y={p[1] + 3.5} text-anchor="middle" font-size="9.5" fill="#6F7A90" font-weight="600">{l}</text>{/each}
 {#if lastBed && lastWake}<path d={arc(lastBed, lastWake, r - 9)} stroke="#5C9BFF" stroke-opacity=".3" stroke-width="3" fill="none" stroke-linecap="round"/>{/if}
 <path d={arc(windowEnd, wake, r)} stroke="#2F6FE0" stroke-opacity=".6" stroke-width="10" fill="none"/>
 <path d={arc(windDown, windowStart, r)} stroke="#FFB547" stroke-width="10" fill="none"/>
 <path d={arc(windowStart, estimate ? hm(toMin(windowStart) + 8) : windowEnd, r)} stroke="#A9C8FF" stroke-width="10" fill="none"/>
 <!-- direct labels -->
 <text x={wdL[0]} y={wdL[1]} text-anchor="end" font-size="10" fill="#FFB547" font-weight="700">{windDown}<tspan x={wdL[0]} dy="11" font-weight="500" fill="#C9A464">wind down</tspan></text>
 <text x={winL[0] + 2} y={winL[1] - 8} text-anchor="start" font-size="10" fill="#A9C8FF" font-weight="700">{windowStart}<tspan x={winL[0] + 2} dy="11" font-weight="500" fill="#8FA8D6">{estimate ? 'asleep by' : 'window'}</tspan></text>
 <text x={wakeL[0]} y={wakeL[1]} text-anchor={anchor(wake)} font-size="10" fill="#EEF1F7" font-weight="700">{wake}<tspan x={wakeL[0]} dy="11" font-weight="500" fill="#B4BCCB">wake</tspan></text>
 <!-- now -->
 <circle class="halo" cx={nowP[0]} cy={nowP[1]} r="11" fill="#EEF1F7" opacity=".16"/>
 <circle cx={nowP[0]} cy={nowP[1]} r="6.5" fill="#EEF1F7" stroke="#0B0F1A" stroke-width="2"/>
 <text x={nowL[0]} y={nowL[1] + 3} text-anchor="middle" font-size="8.5" fill="#EEF1F7" font-weight="700" letter-spacing=".6">NOW</text>
 <text x={c} y={c - 3} text-anchor="middle" font-size="9" fill="#8E98AC" font-weight="600" letter-spacing="1.2">NEED</text>
 <text x={c} y={c + 15} text-anchor="middle" font-size="17" fill="#EEF1F7" font-family="Outfit" font-weight="600">{need}</text>
</svg>
<style>
 .dial{display:block;flex:none;overflow:visible}
 .halo{animation:breathe 3s ease-in-out infinite;transform-box:fill-box;transform-origin:center}
 @keyframes breathe{50%{opacity:.05;transform:scale(1.25)}}
 @media (prefers-reduced-motion:reduce){.halo{animation:none}}
</style>
