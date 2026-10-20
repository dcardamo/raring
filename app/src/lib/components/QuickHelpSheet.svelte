<script lang="ts">
 import Icon from './Icon.svelte';
 import {HELP} from '../quickHelpTopics';
 import {quickHelp} from '../quickHelp.svelte';
 import {sheetOpened} from '../sheet';
 let {navigate}: {navigate: (route: string) => void} = $props();
 const close = () => { quickHelp.topic = null; };
 $effect(() => sheetOpened(close));
 let h = $derived(quickHelp.topic ? HELP[quickHelp.topic] : null);
</script>
<svelte:window onkeydown={e => { if (e.key === 'Escape') close(); }}/>
{#if h}
 <button class="scrim" aria-label="Close" onclick={close}></button>
 <div class="sheet" role="dialog" aria-modal="true" aria-labelledby="qh-title">
  <span class="grab"></span>
  <div class="sheet-h"><h2 id="qh-title">{h.title}</h2><button class="close" aria-label="Close" onclick={close}><Icon name="close" size={20}/></button></div>
  {#each h.body as p}<p>{p}</p>{/each}
  {#if h.more}<button class="link" onclick={() => { close(); navigate(h.more!.route); }}>{h.more.label}</button>{/if}
 </div>
{/if}
<style>
 .scrim{position:fixed;inset:0;border:0;background:rgba(5,8,14,.62);z-index:20;padding:0}
 .sheet{position:fixed;left:0;right:0;bottom:0;z-index:21;max-width:560px;margin:0 auto;background:var(--card);border:1px solid var(--line);border-bottom:0;border-radius:26px 26px 0 0;padding:12px 20px 28px;max-height:80vh;overflow:auto}
 .grab{display:block;width:40px;height:4px;border-radius:2px;background:rgba(255,255,255,.18);margin:0 auto 16px}
 .sheet-h{display:flex;justify-content:space-between;align-items:center;gap:12px}
 .close{width:44px;height:44px;border-radius:50%;border:1px solid var(--line);background:none;display:grid;place-items:center;color:var(--text2)}
 p{font-size:14.5px;line-height:1.5;color:var(--text);margin:10px 0 0}
 .link{background:none;border:0;color:var(--ink);font:600 13px Inter;min-height:44px;padding:0;margin-top:8px}
</style>
