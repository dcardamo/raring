<script lang="ts">
 import Icon from './Icon.svelte';
 import type {LocalAction} from '../generated/LocalAction';
 import type {Sport} from '../generated/Sport';
 import {SPORT_NAMES} from '../goals';
 import {sheetOpened} from '../sheet';
 // A local event is a date to count down to, not a Garmin registration or a plan.
 let {today, close, act}: {today: string, close: () => void, act: (a: LocalAction) => void} = $props();
 let name = $state('');
 let date = $state('');
 let sport = $state<Sport | null>(null);
 const SPORTS: Sport[] = ['running', 'cycling', 'swimming', 'hiking', 'walking'];
 $effect(() => sheetOpened(close));
</script>
<svelte:window onkeydown={e => { if (e.key === 'Escape') close(); }}/>
<button class="scrim" aria-label="Close" onclick={close}></button>
<div class="sheet" role="dialog" aria-modal="true" aria-labelledby="event-title">
 <span class="grab"></span>
 <div class="sheet-h"><h2 id="event-title">Add a race or event</h2><button class="close" aria-label="Close" onclick={close}><Icon name="close" size={20}/></button></div>
 <label class="field"><span>Name</span><input maxlength="60" bind:value={name} aria-label="Event name"/></label>
 <label class="field"><span>Date</span><input type="date" min={today} bind:value={date} aria-label="Event date"/></label>
 <div class="chips2" role="group" aria-label="Sport, optional">{#each SPORTS as s}<button aria-pressed={sport === s} onclick={() => sport = sport === s ? null : s}>{SPORT_NAMES[s]}</button>{/each}</div>
 <p class="small">Saved on this phone. It counts down; it isn’t added to Garmin.</p>
 <button class="primary" disabled={!name.trim() || !date || date < today} onclick={() => { act({action: 'addEvent', name, date, sport}); close(); }}>Add event</button>
</div>
<style>
 .scrim{position:fixed;inset:0;border:0;background:rgba(5,8,14,.62);z-index:20;padding:0}
 .sheet{position:fixed;left:0;right:0;bottom:0;z-index:21;max-width:560px;margin:0 auto;background:#151B28;border:1px solid var(--line);border-bottom:0;border-radius:26px 26px 0 0;padding:12px 20px 28px}
 .grab{display:block;width:40px;height:4px;border-radius:2px;background:rgba(255,255,255,.18);margin:0 auto 16px}
 .sheet-h{display:flex;justify-content:space-between;align-items:center;gap:12px}
 .close{width:44px;height:44px;border-radius:50%;border:1px solid var(--line);background:none;display:grid;place-items:center;color:var(--text2)}
 .field{display:grid;gap:6px;margin-top:12px;font-size:13px;color:var(--text2)}.field input{background:rgba(255,255,255,.06);border:1px solid var(--line);border-radius:10px;color:var(--text);padding:10px;font:500 15px Inter}
 .chips2{display:flex;flex-wrap:wrap;gap:8px;margin:12px 0 0}.chips2 button{min-height:40px;padding:8px 12px;border-radius:999px;border:1px solid var(--line);background:none;color:var(--text2);font:600 13px Inter}.chips2 button[aria-pressed=true]{background:var(--ink);color:var(--bg)}
</style>
