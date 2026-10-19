<script lang="ts">
 import type {Snippet} from 'svelte';
 import Icon from '../components/Icon.svelte';
 import GearCard from '../components/GearCard.svelte';
 import type {AppStatus} from '../generated/AppStatus';
 import type {TrainingView} from '../generated/TrainingView';
 import {monthLabel} from '../format';
 // Profile: what Raring holds from Garmin, gear, and the way to Settings and help.
 let {status, training, navigate, header}: {status: AppStatus | null, training: TrainingView, navigate: (route: string) => void, header: Snippet} = $props();
 const rows = $derived([
  {route: 'settings', title: 'Settings', sub: 'Notifications, privacy, export and sign out'},
  {route: 'help', title: 'Help', sub: 'Lifting with your watch, sets, the body map, syncing'},
 ]);
</script>
{@render header()}
<section class="card">
 <h3>Garmin history on this phone</h3>
 <p class="big n">{status?.historyFrom ? `Since ${monthLabel(status.historyFrom)}` : '—'}</p>
 <p class="small">{status?.historyComplete ? 'All of your Garmin history is on this phone.' : 'Downloading older history in the background.'}</p>
</section>
{#if training.gear}<div class="gear"><GearCard gear={training.gear}/></div>{/if}
{#each rows as r}<button class="card row-link" onclick={() => navigate(r.route)}><span><b>{r.title}</b><small>{r.sub}</small></span><Icon name="chevron" size={16}/></button>{/each}
<section class="card"><h3>Add weight</h3><p class="small">Not yet available: recording a weigh-in would write to Garmin.</p><button class="secondary" disabled>Add weight</button></section>
<style>
 .row-link{display:flex;justify-content:space-between;align-items:center;width:100%;text-align:left;color:var(--text)}.row-link b{display:block;font:600 17px Outfit}.row-link small{display:block;font-size:12px;color:var(--text2);margin-top:3px}
 .secondary:disabled{color:var(--text2);background:none;border:1px dashed var(--line);cursor:default}
</style>
