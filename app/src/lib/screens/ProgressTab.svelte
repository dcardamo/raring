<script lang="ts">
 import type {Snippet} from 'svelte';
 import Icon from '../components/Icon.svelte';
 import type {HealthView} from '../generated/HealthView';
 import {num} from '../format';
 // Progress: what is adding up or changing. The weekly review and Garmin's
 // trends, each opening its canonical detail.
 let {health, navigate, header}: {health: HealthView, navigate: (route: string) => void, header: Snippet} = $props();
 const vital = (kind: string) => health.vitals.readings.find(r => r.kind === kind);
 let tiles = $derived([
  {route: 'metric/vo2max', name: 'VO₂ max', value: null as number | null, unit: ''},
  {route: 'health', name: 'Fitness Age', value: health.fitnessAge.fitnessAge, unit: 'yrs'},
  {route: 'metric/restingHr', name: 'Resting HR', value: vital('restingHr')?.value ?? null, unit: 'bpm'},
  {route: 'metric/hrv', name: 'HRV', value: vital('hrv')?.value ?? null, unit: 'ms'},
  {route: 'metric/bodyBattery', name: 'Body Battery', value: null, unit: ''},
  {route: 'metric/stress', name: 'Stress', value: health.stress.average, unit: 'avg'},
 ]);
</script>
{@render header()}
<button class="card review" onclick={() => navigate('review/latest')}><span><b>Last week</b><small>What was met, what changed and next week’s targets</small></span><Icon name="chevron" size={16}/></button>
<div class="sec-h"><h2>Trends</h2></div>
<div class="tiles2">
 {#each tiles as t}<button class="card tile2" onclick={() => navigate(t.route)}><span>{t.name}</span>{#if t.value != null}<b class="n">{num(t.value)}<small>{t.unit}</small></b>{:else}<em>Open chart</em>{/if}</button>{/each}
</div>
<p class="small foot">Every chart reads your downloaded Garmin history, with ranges from 7 days to all.</p>
<style>
 .review{display:flex;justify-content:space-between;align-items:center;width:100%;text-align:left;color:var(--text)}.review b{display:block;font:600 17px Outfit}.review small{display:block;font-size:12px;color:var(--text2);margin-top:3px}
 .sec-h{margin:22px 4px 0}.sec-h h2{font:600 19px Outfit}
 .tiles2{display:grid;grid-template-columns:repeat(auto-fill,minmax(150px,1fr));gap:10px}
 .tile2{display:flex;flex-direction:column;align-items:flex-start;gap:6px;text-align:left;color:var(--text);margin-top:10px}.tile2 span{font:500 12px Inter;color:var(--text3)}.tile2 b{font:600 26px Outfit}.tile2 small{font:500 12px Inter;color:var(--text3);margin-left:3px}.tile2 em{font-style:normal;font-size:13px;color:var(--text2)}
 .foot{margin:14px 4px}
</style>
