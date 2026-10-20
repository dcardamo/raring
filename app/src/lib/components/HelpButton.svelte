<script lang="ts">
 import {HELP} from '../quickHelpTopics';
 import {quickHelp} from '../quickHelp.svelte';
 let {topic}: {topic: string} = $props();
</script>
<button class="qh" aria-label={`Help: ${HELP[topic]?.title ?? topic}`} onclick={e => { e.stopPropagation(); quickHelp.topic = topic; }}>?</button>
<style>.qh::after{content:'';position:absolute;inset:-6px}.qh{position:relative;flex:none;width:32px;height:32px;min-height:32px;border-radius:50%;border:1px solid var(--line);background:none;color:var(--text2);font:500 13px Inter;display:inline-grid;place-items:center;padding:0}</style>
