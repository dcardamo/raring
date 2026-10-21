<script lang="ts">
 import type {Snippet} from 'svelte';
 import Icon from '../components/Icon.svelte';
 import GearCard from '../components/GearCard.svelte';
 import AddWeight from '../components/AddWeight.svelte';
 import HelpButton from '../components/HelpButton.svelte';
 import type {AppStatus} from '../generated/AppStatus';
 import type {ProfileView} from '../generated/ProfileView';
 import {SPORT_NAMES} from '../goals';
 import {dateLabel, monthLabel, num} from '../format';
 // Profile: what Raring holds from Garmin, your limitations, gear, and the way
 // to Settings and help. Adding a weight is the one Garmin write here.
 let {status, profile, navigate, limits, header, onwrite}: {status: AppStatus | null, profile: ProfileView, navigate: (route: string) => void, limits: () => void, header: Snippet, onwrite: () => void} = $props();
 const UNITS: Record<string, string> = {metric: 'Metric · km, kg', statute_us: 'Statute (US) · mi, lb', statute_uk: 'Statute (UK) · mi, st and lb'};
 let lim = $derived([profile.limits.excluded.length ? `Avoiding ${profile.limits.excluded.map(s => SPORT_NAMES[s].toLowerCase()).join(', ')}` : '', profile.limits.note ? `“${profile.limits.note}”` : ''].filter(Boolean));
 let upload = $derived(profile.watchLastUpload ? `${dateLabel(profile.watchLastUpload.slice(0, 10))} · ${new Date(profile.watchLastUpload).toLocaleTimeString('en-GB', {hour: '2-digit', minute: '2-digit'})}` : 'Not recorded');
</script>
{@render header()}
<section class="card" aria-labelledby="garmin-facts">
 <h3 class="hh"><span id="garmin-facts">What Garmin knows</span><HelpButton topic="profile"/></h3>
 <dl class="kv">
  <dt>Age</dt><dd>{profile.age != null ? num(profile.age) : 'Not recorded'}</dd>
  <dt>Weight</dt><dd>{profile.weight != null && profile.weightUnit ? `${num(profile.weight, 1)} ${profile.weightUnit}` : 'Not recorded'}{#if profile.weightFreshness.recordedOn}<small class:stale={profile.weightFreshness.stale}>{profile.weightFreshness.stale ? 'Last weigh-in ' : ''}{dateLabel(profile.weightFreshness.recordedOn)}</small>{/if}</dd>
  <dt>Units</dt><dd>{profile.units ? UNITS[profile.units] ?? profile.units : 'Not recorded'}</dd>
  <dt>Watch</dt><dd>Last upload<small>{upload}</small></dd>
  <dt>History</dt><dd>{status?.historyFrom ? `Since ${monthLabel(status.historyFrom)}` : '—'}<small>{status?.historyComplete ? 'All of it is on this phone' : 'Downloading older history'}</small></dd>
 </dl>
 <AddWeight unit={profile.weightUnit} {onwrite}/>
</section>
<section class="card" aria-labelledby="limits-h">
 <h3 id="limits-h">Limitations and injuries</h3>
 {#if lim.length}<p class="lim">{lim.join(' · ')}</p>{:else}<p class="small">None set.</p>{/if}
 <button class="row-link" onclick={limits}><span><b>Manage limitations</b></span><Icon name="chevron" size={16}/></button>
</section>
{#if profile.gear}<div class="gear"><GearCard gear={profile.gear}/></div>{/if}
<section class="card">
 <button class="row-link" onclick={() => navigate('settings')}><span><b>Settings</b><small>Notifications, privacy, export and sign out</small></span><Icon name="chevron" size={16}/></button>
 <button class="row-link" onclick={() => navigate('help')}><span><b>Help</b><small>Lifting with your watch, sets, the body map and which watches work</small></span><Icon name="chevron" size={16}/></button>
</section>
<style>
 .hh{display:flex;justify-content:space-between;align-items:center;gap:8px}
 .kv{display:grid;grid-template-columns:auto 1fr;gap:10px 16px;margin:8px 0 0;font-size:14px}.kv dt{color:var(--text2)}.kv dd{margin:0;text-align:right;font-weight:600}
 .kv small{display:block;font:500 12px Inter;color:var(--text2)}.kv small.stale{color:var(--fair)}
 .lim{font-size:14px;margin:4px 0}
 .row-link{display:flex;justify-content:space-between;align-items:center;width:100%;min-height:52px;text-align:left;color:var(--text);background:none;border:0;padding:6px 0}
 .row-link+.row-link{border-top:1px solid var(--line)}
 .row-link b{display:block;font:600 16px Outfit}.row-link small{display:block;font-size:12px;color:var(--text2);margin-top:3px}
</style>
