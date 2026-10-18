<script lang="ts">
 import Header from '../../lib/components/Header.svelte';
 import Card from '../../lib/components/Card.svelte';
 import AboutRaring from '../../lib/components/AboutRaring.svelte';
 import {store} from '../lib/store.svelte';
 import {A} from '../lib/actions';
 import type {State} from '../lib/rules';
 let {}: {route: string} = $props();
 let s = $derived(store.state);
 const CHANNELS: [keyof State['channels'], string][] = [['morning', 'Morning readiness'], ['weekly', 'Weekly check-in'], ['metrics', 'VO₂ max or Fitness Age changed'], ['vitals', 'Vitals off'], ['training', 'Training status changed'], ['status', 'Persistent status']];
</script>
<Header title="Settings" eyebrow="RARING" back={() => store.back()} backLabel={store.parentLabel}/>
<Card title="Notifications">
 <p class="small">Permission: {s.permission === 'declined' ? 'declined. Raring won’t ask again; you can allow it in Android settings.' : s.permission === 'granted' ? 'allowed.' : 'not asked yet. Raring asks when you turn on a reminder.'}</p>
 {#each CHANNELS as [key, name]}<label class="toggle"><span>{name}</span><input type="checkbox" checked={s.channels[key]} onchange={e => { const el = e.currentTarget; if (!A.channel(key, el.checked)) el.checked = s.channels[key]; }}/></label>{/each}
 <button class="secondary" onclick={A.bedtime}>{s.notifyBedtime ? 'Wind-down reminder on' : 'Remind me to wind down'}</button>
 <p class="small">Reminders follow the same rest, pause and limit rules as Today. Lock screens never show restriction notes, routine names, journal or weight. Widgets need no permission.</p>
</Card>
<Card title="Today">
 <label class="toggle"><span>Weekly planning reminder<small>A Plan your week prompt at the start of each week. Never during wind-down.</small></span><input type="checkbox" checked={s.planReminder !== false} onchange={e => store.set({planReminder: e.currentTarget.checked})}/></label>
 <label class="toggle"><span>Follow my day<small>Lead with this morning, the day or tonight. Turn off for shift work.</small></span><input type="checkbox" checked={s.follow} onchange={e => store.set({follow: e.currentTarget.checked})}/></label>
 <label class="toggle"><span>Quiet celebrations<small>Same information, no animation.</small></span><input type="checkbox" checked={s.quiet} onchange={e => store.set({quiet: e.currentTarget.checked})}/></label>
</Card>
<Card title="Adapt" onclick={() => store.navigate('adapt')}><p class="small">Rest, pauses, limits and routines</p></Card>
<Card title="Personalisation"><p class="small">This version doesn’t learn from how you use the app. Navigation and controls stay where they are.</p></Card>
<section class="card" id="sec-journal"><h3>Journal privacy</h3>
 <label class="toggle"><span>Allow Garmin journal reads<small>Off by default. Turning off stops new reads and hides entries. Downloaded logs remain on this phone and in future exports until you erase local data.</small></span><input type="checkbox" checked={s.journal.consent} onchange={e => A.journalConsent(e.currentTarget.checked)}/></label>
</section>
<section class="card" id="sec-export"><h3>Export</h3>
 <p class="small">A ZIP of saved Garmin responses, daily CSVs and Raring settings, including plans, intentions, circumstances and notes, routine checklist and self-reports, dismissals and any downloaded journal logs. Sign-in tokens are never included. The file leaves the phone only when you share it.</p>
 <button class="secondary" onclick={A.exportData}>Export your data</button>
</section>
<Card title="Account"><p class="small">Signing out erases all local Garmin history and everything you added here.</p><button class="danger" onclick={() => store.openSheet('signout')}>Sign out and erase…</button></Card>
<Card title="Garmin history"><div class="big n">Since {store.ctx.world.historyStart.slice(0, 7)}</div><p class="small">{store.ctx.world.historyComplete ? 'All of your Garmin history is on this phone.' : 'Downloading older history in the background.'}</p></Card>
<Card title="Help" onclick={() => store.navigate('help')}><p class="small">Strength sets, the body map, watches, syncing</p></Card>
<Card title="How Raring works"><AboutRaring/><p class="small">Settings import/restore isn’t shipped; export isn’t a restore workflow. This prototype connects to no account.</p></Card>
