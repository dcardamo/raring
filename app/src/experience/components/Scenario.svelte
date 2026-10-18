<script lang="ts">
 import Icon from '../../lib/components/Icon.svelte';
 import {store} from '../lib/store.svelte';
 import {A} from '../lib/actions';
 import {CLOCKS, PROFILES, type ClockKey, type Profile} from '../lib/rules';
 import {CASES} from '../lib/cases';
 import {DAYS} from '../lib/world';
 import {dayLabel} from '../lib/dates';
 // Prototype tooling for moderators and reviewers; not part of the app.
 let {open = $bindable(false)}: {open?: boolean} = $props();
 let dlg = $state<HTMLDialogElement>();
 $effect(() => { if (!dlg) return; if (open && !dlg.open) dlg.showModal(); if (!open && dlg.open) dlg.close(); });
 const close = () => open = false;
 const s = $derived(store.state);
</script>
<dialog class="scenario" bind:this={dlg} onclose={close} aria-labelledby="scn-t">
 <div class="sheet-h"><h2 id="scn-t">Prototype controls</h2><button class="iconbtn" aria-label="Close prototype controls" onclick={close}><Icon name="close" size={18}/></button></div>
 <p class="small">For moderators and reviewers. Not part of the app.</p>
 <label class="field"><span>Profile</span><select value={s.profile} onchange={e => { close(); store.reset(e.currentTarget.value as Profile); }}>{#each Object.entries(PROFILES) as [k, v]}<option value={k}>{v.label}</option>{/each}</select></label>
 <label class="field"><span>Day</span><select value={s.day} onchange={e => store.set({day: e.currentTarget.value})}>{#each DAYS as d}<option value={d}>{dayLabel(d)}</option>{/each}</select></label>
 <label class="field"><span>Time</span><select value={s.clock} onchange={e => store.set({clock: e.currentTarget.value as ClockKey})}>{#each Object.entries(CLOCKS) as [k, v]}<option value={k}>{v.time} · {v.label}</option>{/each}</select></label>
 <label class="field"><span>Explore entry</span><select value={s.navVariant} onchange={e => { store.set({navVariant: e.currentTarget.value as 'tabs' | 'header'}); store.external('today'); }}><option value="tabs">Fourth tab (default)</option><option value="header">Header button (compare)</option></select></label>
 <label class="toggle"><span>Offline</span><input type="checkbox" checked={s.offline} onchange={e => store.set({offline: e.currentTarget.checked})}/></label>
 <label class="toggle"><span>Irregular wake times</span><input type="checkbox" checked={s.irregular} onchange={e => store.set({irregular: e.currentTarget.checked})}/></label>
 <label class="field"><span>Naps</span><select value={s.napCase} onchange={e => store.set({napCase: e.currentTarget.value as typeof s.napCase})}>{#each [['same', 'Same-day nap'], ['cross', 'Across midnight + missing end'], ['empty', 'Empty list'], ['unavailable', 'Unavailable'], ['stale', 'Stale']] as [k, l]}<option value={k}>{l}</option>{/each}</select></label>
 <label class="field"><span>Garmin calendar</span><select value={s.calendar} onchange={e => store.set({calendar: e.currentTarget.value as typeof s.calendar})}>{#each [['complete', 'Complete'], ['partial', 'Full page (partial)'], ['stale', 'Stale']] as [k, l]}<option value={k}>{l}</option>{/each}</select></label>
 <div class="btns"><button class="secondary" onclick={() => A.sync()}>Sync</button><button class="secondary" onclick={() => A.sync('exception')}>Sync: new notice</button><button class="secondary" onclick={() => { close(); A.background(); }}>Background + resume</button><button class="secondary" onclick={() => location.reload()}>Reload (resume)</button></div>
 <div class="btns"><button class="secondary" onclick={() => { close(); store.external('surfaces'); }}>Status surfaces</button><button class="secondary" onclick={() => { close(); store.external('legacy'); }}>Old links</button><button class="secondary" onclick={() => { close(); store.external('compare'); }}>Compare</button><button class="secondary" onclick={() => { close(); store.reset(s.profile); }}>Reset choices</button></div>
 <label class="field col"><span>Journey case</span><select value="" onchange={e => { const id = e.currentTarget.value; if (id) { close(); A.applyCase(id); } }}><option value="">Choose U01–U64…</option>{#each CASES as c}<option value={c.id}>{c.id} · {c.title}</option>{/each}</select></label>
</dialog>
