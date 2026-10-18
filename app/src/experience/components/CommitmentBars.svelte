<script lang="ts">
 import Ring from './Ring.svelte';
 import Icon from '../../lib/components/Icon.svelte';
 import type {WeekBar} from '../lib/progress';
 // Weekly bars against each week's own target (the line steps where a version
 // changed). Paused weeks are hatched, unknown dotted; weeks before a plan carry
 // no target and no verdict. The current week is a ring.
 let {weeks, name, unit = '', color, onWeek}: {weeks: WeekBar[], name: string, unit?: string, color: string, onWeek: (id: string) => void} = $props();
 let past = $derived(weeks.filter(w => !w.current));
 let cur = $derived(weeks.find(w => w.current));
 let max = $derived(Math.max(1, ...weeks.map(w => Math.max(w.value ?? 0, w.target ?? 0))) * 1.15);
 const H = 84;
 const y = (v: number) => H - v / max * H;
</script>
<div class="cb">
 <div class="cb-h"><b>{name}</b>{#if cur}<span class="small">This week · {cur.value}{unit} of {cur.target}{unit}</span>{/if}</div>
 <div class="cb-body">
  <div class="bars" style:--h={H + 'px'}>
   {#each past as w, i}
    {@const prev = past[i - 1]}
    <button class="wk s-{w.state}" onclick={() => onWeek(w.id)} aria-label={`Week of ${w.label}: ${w.state === 'none' ? 'no plan yet' : w.state === 'paused' ? 'paused' : w.state === 'unknown' ? 'awaiting evidence' : `${w.value}${unit} of ${w.target}${unit}${w.state === 'reached' ? ', reached' : ''}`}`}>
     <span class="plot">
      {#if w.target != null}<u style:bottom={(H - y(w.target)) + 'px'} class:step={prev?.target != null && prev.target !== w.target}></u>{/if}
      <i style:height={(w.state === 'paused' ? H * .9 : Math.max(3, (w.value ?? 0) / max * H)) + 'px'} style:--c={color}></i>
      {#if w.state === 'reached'}<em><Icon name="check" size={11}/></em>{/if}
     </span>
     <span class="lbl">{w.label.split(' ')[0]}</span>
    </button>
   {/each}
  </div>
  {#if cur}<button class="now" onclick={() => onWeek(cur!.id)}><Ring value={cur.value ?? 0} target={cur.target ?? 1} size={60} stroke={6} {color} state={cur.state} label={`This week: ${cur.value}${unit} of ${cur.target}${unit}`}/><span class="lbl">Now</span></button>{/if}
 </div>
</div>
<style>
 .cb:not(:first-child){margin-top:18px;padding-top:14px;border-top:1px solid var(--line)}
 .cb-h{display:flex;justify-content:space-between;align-items:baseline;gap:8px;margin-bottom:10px}
 .cb-h b{font-weight:600}
 .cb-h .small{margin:0}
 .cb-body{display:flex;align-items:flex-end;gap:10px}
 .bars{flex:1;display:flex;gap:4px;align-items:flex-end;min-width:0}
 .wk{flex:1;min-width:0;display:flex;flex-direction:column;align-items:center;background:none;border:0;padding:0;color:var(--text3)}
 .plot{position:relative;width:100%;max-width:24px;height:var(--h);display:flex;align-items:flex-end}
 .plot i{display:block;width:100%;border-radius:5px 5px 3px 3px;background:var(--c);opacity:.85}
 .plot u{position:absolute;left:-3px;right:-3px;height:0;border-top:2px solid rgba(238,241,247,.7);text-decoration:none;z-index:1}
 .plot u.step{border-top-style:dashed}
 .plot em{position:absolute;left:50%;transform:translateX(-50%);bottom:4px;color:#0B0F1A;z-index:2;line-height:0}
 .s-none .plot i{background:rgba(255,255,255,.14);opacity:1}
 .s-open .plot i{opacity:.55}
 .s-paused .plot i{background:repeating-linear-gradient(45deg,rgba(169,200,255,.35) 0 3px,transparent 3px 7px);border:1px solid rgba(169,200,255,.3)}
 .s-unknown .plot i{background:none;border:1.5px dotted rgba(142,152,172,.7)}
 .lbl{font:600 10px Inter;margin-top:5px;white-space:nowrap}
 .now{display:flex;flex-direction:column;align-items:center;background:none;border:0;color:var(--text);padding:0}
</style>
