<script lang="ts">
 import {SPORTS, type Sport} from '../lib/sports';
 // Time by activity type per day, week or month. Days still to come are blank,
 // days without evidence are dotted, and a known day with nothing recorded is
 // an empty track.
 type Bucket = {label: string, short: string, total: number, future: boolean, known: boolean, sports: [Sport, number][]};
 let {buckets, h = 120}: {buckets: Bucket[], h?: number} = $props();
 let max = $derived(Math.max(30, ...buckets.map(b => b.total)));
 let step = $derived(buckets.length > 20 ? Math.ceil(buckets.length / 8) : 1);
 const fmt = (m: number) => m >= 60 ? `${Math.floor(m / 60)}h ${String(m % 60).padStart(2, '0')}` : `${m} min`;
</script>
<div class="pbars" style:--h={h + 'px'} role="img" aria-label={`Active time: ${buckets.filter(b => b.total).map(b => `${b.label} ${fmt(b.total)}`).join(', ') || 'none recorded'}`}>
 {#each buckets as b, i}
  <div class="col" class:future={b.future}>
   <div class="stack" class:unknown={!b.known && !b.future && !b.total}>
    {#each b.sports as [s, m]}<i style:height={m / max * 100 + '%'} style:background={SPORTS[s].color}></i>{/each}
   </div>
   <span class="lbl">{i % step === 0 ? b.short : ''}</span>
  </div>
 {/each}
</div>
<style>
 .pbars{display:flex;align-items:stretch;gap:2px;height:calc(var(--h) + 18px)}
 .col{flex:1;min-width:0;display:flex;flex-direction:column;align-items:center}
 .stack{width:100%;max-width:26px;height:var(--h);display:flex;flex-direction:column-reverse;gap:1px;border-radius:5px;overflow:hidden;background:rgba(255,255,255,.035)}
 .stack i{display:block;width:100%;animation:grow .6s ease-out both;transform-origin:bottom}
 .stack.unknown{background:none;border:1.5px dotted rgba(142,152,172,.5)}
 .future .stack{background:none}
 .lbl{font:600 10px Inter;color:var(--text3);margin-top:5px;height:12px;white-space:nowrap}
 @keyframes grow{from{transform:scaleY(0)}}
 @media (prefers-reduced-motion:reduce){.stack i{animation:none}}
</style>
