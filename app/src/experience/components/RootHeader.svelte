<script lang="ts">
 import Icon from '../../lib/components/Icon.svelte';
 import XIcon from './XIcon.svelte';
 import {store} from '../lib/store.svelte';
 import {dayLabel, monthYear} from '../lib/dates';
 let {title, adapt = false}: {title: string, adapt?: boolean} = $props();
 let ctx = $derived(store.ctx);
</script>
<header class="rhead">
 <p class="eyebrow"><span>{dayLabel(ctx.calendarDate)}</span><span aria-hidden="true">·</span><span class="fresh" class:stale={ctx.stale}>Watch upload {ctx.upload.label}</span></p>
 <div class="rhead-row">
  <h1 tabindex="-1">{title}</h1>
  <div class="hctl">
   {#if store.state.navVariant === 'header'}<button class="hbtn" onclick={() => store.navigate('explore')}><XIcon name="explore" size={18}/>Explore</button>{/if}
   <button class="hbtn icon" aria-label="Settings" onclick={() => store.navigate('settings')}><XIcon name="settings" size={19}/></button>
  </div>
 </div>
 {#if adapt || ctx.offline || (!ctx.world.historyComplete && title === 'Today')}
  <div class="rhead-row2">
   {#if adapt}<button class="adapt" class:on={ctx.adaptLabel !== 'Adapt today'} aria-haspopup="dialog" onclick={() => store.openSheet('adapt')}>{#if ctx.paused}<Icon name="pause" size={16}/>{:else if ctx.restActive}<Icon name="moon" size={16}/>{:else}<XIcon name="adapt" size={16}/>{/if}{ctx.adaptLabel}</button>{/if}
   {#if ctx.offline}<span class="qpill"><XIcon name="offline" size={14}/>Offline · saved data</span>{/if}
   {#if !ctx.world.historyComplete && title === 'Today'}<span class="qpill">History since {monthYear(ctx.world.historyStart).replace(/^(\w{3})\w*/, '$1')} · older downloading</span>{/if}
  </div>
 {/if}
</header>
