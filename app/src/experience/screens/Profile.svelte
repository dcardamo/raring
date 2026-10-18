<script lang="ts">
 import Header from '../../lib/components/Header.svelte';
 import Icon from '../../lib/components/Icon.svelte';
 import GearCard from '../../lib/components/GearCard.svelte';
 import XIcon from '../components/XIcon.svelte';
 import HelpButton from '../components/HelpButton.svelte';
 import {store} from '../lib/store.svelte';
 import {GEAR, CHRONOLOGICAL_AGE, weightFor} from '../lib/world';
 import {SPORTS} from '../lib/sports';
 import {dayLabel} from '../lib/dates';
 // What Garmin knows, gear and limitations. Edits to Garmin data are not built.
 let ctx = $derived(store.ctx);
 const go = (r: string) => store.navigate(r);
 let weight = $derived.by(() => { for (let i = 0; i < 60; i++) { const d = new Date(Date.parse(ctx.calendarDate) - i * 864e5).toISOString().slice(0, 10); const w = weightFor(d); if (w != null) return {w, d}; } return null; });
</script>
<Header title="Profile" eyebrow="From Garmin Connect" back={() => store.back()} backLabel={store.parentLabel}/>
<div class="ph"><HelpButton topic="profile"/></div>
<section class="card"><h3>What Garmin knows</h3>
 <dl class="kv">
  <dt>Age</dt><dd>{CHRONOLOGICAL_AGE}</dd>
  <dt>Weight</dt><dd>{weight ? `${weight.w} kg · ${dayLabel(weight.d)}` : 'Not recorded'}</dd>
  <dt>Units</dt><dd>Metric · km, kg</dd>
  <dt>Heart-rate zones</dt><dd>Garmin’s, max HR 186</dd>
  <dt>Watch</dt><dd>Last upload {ctx.upload.label}</dd>
 </dl>
 <button class="na" disabled aria-describedby="w-na"><Icon name="scale" size={16}/>Add a weight</button><p class="small" id="w-na">Not available yet: Raring can’t write to Garmin.</p>
</section>
<section class="card"><h3>Limitations and injuries</h3>
 {#if ctx.hasLimits}<p class="lim">{[ctx.limits.exclude.length && `Avoiding ${ctx.limits.exclude.map(s => SPORTS[s].name.toLowerCase()).join(', ')}`, ctx.limits.note && `“${ctx.limits.note}”`].filter(Boolean).join(' · ')}</p>{:else}<p class="small">None set.</p>{/if}
 <button class="rowlink" onclick={() => go('adapt/limits')}><XIcon name="adapt" size={18}/><span><b>Manage limitations</b><small>Also in Adapt today</small></span><Icon name="chevron" size={16}/></button>
</section>
<section class="card gear"><h3>Gear</h3><GearCard gear={GEAR as never}/></section>
<section class="card">
 <button class="rowlink" onclick={() => go('settings')}><XIcon name="settings" size={18}/><span><b>Settings</b><small>Notifications, privacy, export, sign out</small></span><Icon name="chevron" size={16}/></button>
 <button class="rowlink" onclick={() => go('help')}><Icon name="help" size={18}/><span><b>Help</b><small>Sync, strength and more</small></span><Icon name="chevron" size={16}/></button>
</section>
<style>
 .ph{display:flex;justify-content:flex-end;margin-top:-34px}
 .na{display:flex;gap:8px;align-items:center;justify-content:center;width:100%;margin-top:12px;min-height:40px;border-radius:12px;border:1px dashed var(--line);background:none;color:var(--text3);font:600 13px Inter}
 .lim{font-size:14px;margin:0 0 6px}
 .gear :global(.card){margin-top:0;border:0;padding:0;background:none}
</style>
