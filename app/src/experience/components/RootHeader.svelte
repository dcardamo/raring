<script lang="ts">
 import Icon from '../../lib/components/Icon.svelte';
 import XIcon from './XIcon.svelte';
 import PlanPrompt from './PlanPrompt.svelte';
 import {store} from '../lib/store.svelte';
 import {dayLabel, monthYear} from '../lib/dates';
 // Compact root header: date and upload freshness, the title with a stable,
 // labelled Adapt control, and quiet one-line status notes.
 let {title, adapt = true}: {title: string, adapt?: boolean} = $props();
 let ctx = $derived(store.ctx);
 let history = $derived(!ctx.world.historyComplete && title === 'Today' ? `History since ${monthYear(ctx.world.historyStart).replace(/^(\w{3})\w*/, '$1')} · older history downloading` : null);
</script>
<header class="rhead">
 <p class="eyebrow"><span>{dayLabel(ctx.calendarDate)}</span><span aria-hidden="true">·</span><span class="fresh" class:stale={ctx.stale}>Watch upload {ctx.upload.label}</span></p>
 <div class="rhead-row">
  <h1 tabindex="-1">{title}</h1>
  <div class="hctl">
   {#if adapt}<button class="adapt" class:on={ctx.adaptLabel !== 'Adapt today'} aria-haspopup="dialog" onclick={() => store.openSheet('adapt')}>{#if ctx.paused}<Icon name="pause" size={15}/>{:else if ctx.restActive}<Icon name="moon" size={15}/>{:else}<XIcon name="adapt" size={15}/>{/if}{ctx.adaptLabel}</button>{/if}
   
   <button class="hbtn icon" aria-label="Search metrics" onclick={() => store.openSheet('search')}><XIcon name="search" size={18}/></button>
   <button class="hbtn icon" aria-label="Settings" onclick={() => store.navigate('settings')}><XIcon name="settings" size={19}/></button>
  </div>
 </div>
 {#if ctx.offline || history}
  <p class="hnote">{#if ctx.offline}<span><XIcon name="offline" size={13}/>Offline · saved data</span>{/if}{#if history}<span>{history}</span>{/if}</p>
 {/if}
</header>
<PlanPrompt/>
<style>
 .hctl{flex-wrap:wrap;justify-content:flex-end}
 .hnote{display:flex;flex-wrap:wrap;gap:4px 12px;margin:4px 2px 0;font-size:12px;color:var(--text3)}
 .hnote span{display:inline-flex;align-items:center;gap:5px}
</style>
