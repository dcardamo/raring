<script lang="ts">
 import Icon from './Icon.svelte';
 import {searchIndex,searchHits,type SearchEntry} from '../search';
 import {sheetOpened} from '../sheet';
 // Find a page, card or Garmin metric by name. Browsing works without it.
 let {close, navigate, sleepHr}: {close: () => void, navigate: (route: string, anchor?: string) => void, sleepHr?: () => Promise<boolean>} = $props();
 let showsSleepHr = $state<boolean | null>(null);
 $effect(() => { sleepHr?.().then(v => showsSleepHr = v).catch(() => {}); });
 let q = $state('');
 let hits = $derived(searchHits(q, searchIndex(showsSleepHr)));
 $effect(() => sheetOpened(close));
 const go = (entry: SearchEntry) => { close(); navigate(entry.route, entry.anchor); };
</script>
<svelte:window onkeydown={e => { if (e.key === 'Escape') close(); }}/>
<button class="scrim" aria-label="Close search" onclick={close}></button>
<div class="sheet" role="dialog" aria-modal="true" aria-labelledby="search-title">
 <span class="grab"></span>
 <div class="sheet-h"><h2 id="search-title">Search</h2><button class="close" aria-label="Close" onclick={close}><Icon name="close" size={20}/></button></div>
 <!-- svelte-ignore a11y_autofocus -->
 <input class="q" type="search" placeholder="Metric or page" aria-label="Search metrics and pages" bind:value={q} autofocus/>
 <ul>{#each hits as entry (entry.name + entry.route)}<li><button onclick={() => go(entry)}><span>{entry.name}</span><Icon name="chevron" size={15}/></button></li>{:else}<li class="none">Nothing matches “{q}”</li>{/each}</ul>
</div>
<style>
 .scrim{position:fixed;inset:0;border:0;background:rgba(5,8,14,.62);z-index:20;padding:0}
 .sheet{position:fixed;left:0;right:0;bottom:0;z-index:21;max-width:560px;margin:0 auto;background:#151B28;border:1px solid var(--line);border-bottom:0;border-radius:26px 26px 0 0;padding:12px 20px 28px}
 .grab{display:block;width:40px;height:4px;border-radius:2px;background:rgba(255,255,255,.18);margin:0 auto 16px}
 .sheet-h{display:flex;justify-content:space-between;align-items:center;gap:12px}
 .close{width:44px;height:44px;border-radius:50%;border:1px solid var(--line);background:none;display:grid;place-items:center;color:var(--text2)}
 .q{width:100%;margin:6px 0 10px;padding:12px 14px;border-radius:14px;border:1px solid var(--line);background:rgba(255,255,255,.06);color:var(--text);font:500 15px Inter}
 ul{list-style:none;margin:0;padding:0;max-height:52vh;overflow:auto}
 li button{display:flex;justify-content:space-between;align-items:center;width:100%;min-height:44px;padding:8px 4px;border:0;border-bottom:1px solid var(--line);background:none;color:var(--text);text-align:left;font:500 15px Inter}
 .none{color:var(--text3);padding:12px 4px}
</style>
