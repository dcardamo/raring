<script lang="ts">
 import Icon from '../../lib/components/Icon.svelte';
 import {SPORTS, type Sport} from '../lib/sports';
 // The shipped icon set draws "other" as an info mark, which reads as a warning
 // on a sport. Yoga and other activities get their own neutral figures here.
 let {sport, size = 20}: {sport: Sport, size?: number} = $props();
</script>
{#if sport === 'yoga'}<svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><circle cx="12" cy="4.5" r="2"/><path d="M12 7.5v6M5 11l7 2.5 7-2.5M8 20l4-6.5 4 6.5M6 20h12"/></svg>
{:else if sport === 'other'}<svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M3 12h4l2.5-6 5 12 2.5-6H21"/></svg>
{:else}<Icon name={SPORTS[sport].icon} {size}/>{/if}
