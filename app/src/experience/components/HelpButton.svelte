<script lang="ts">
 import {store} from '../lib/store.svelte';
 import {HELP} from '../lib/help';
 let {topic}: {topic: string} = $props();
</script>
<button class="qh" aria-label={`Help: ${HELP[topic]?.title ?? topic}`} onclick={() => store.openSheet('qhelp', topic)}>?</button>
<style>.qh{flex:none;width:32px;height:32px;min-height:32px;border-radius:50%;border:1px solid var(--line);background:rgba(255,255,255,.05);color:var(--text2);font:700 13px Inter;display:inline-grid;place-items:center;padding:0}</style>
