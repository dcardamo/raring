<script lang="ts">
 import Icon from './Icon.svelte';
 import type {GarminSession} from '../generated/GarminSession';
 import type {Limits} from '../generated/Limits';
 import type {LocalAction} from '../generated/LocalAction';
 import {SPORT_NAMES} from '../goals';
 import {sheetOpened} from '../sheet';
 import {dateLabel} from '../format';
 // One of Garmin's scheduled workouts: read-only. Raring knows its name and day
 // only, so it can't check it against your limits or count it.
 let {session, today, limits, close, act, navigate}: {session: GarminSession, today: string, limits: Limits | null, close: () => void, act: (a: LocalAction) => void, navigate: (route: string) => void} = $props();
 $effect(() => sheetOpened(close));
 let limited = $derived(!!limits && (limits.excluded.length > 0 || !!limits.note));
</script>
<svelte:window onkeydown={e => { if (e.key === 'Escape') close(); }}/>
<button class="scrim" aria-label="Close" onclick={close}></button>
<div class="sheet" role="dialog" aria-modal="true" aria-labelledby="garmin-title">
 <span class="grab"></span>
 <div class="sheet-h"><h2 id="garmin-title">{session.name}</h2><button class="close" aria-label="Close" onclick={close}><Icon name="close" size={20}/></button></div>
 <p class="small">◇ Garmin schedule · read-only · {dateLabel(session.date)}</p>
 {#if limited}<p class="warn" role="note"><b>Review needed.</b> Contents unknown to Raring: Garmin doesn’t tell Raring this workout’s sport or movements, so it can’t be checked against {limits!.excluded.length ? `your excluded ${limits!.excluded.map(s => SPORT_NAMES[s].toLowerCase()).join(', ')}` : ''}{limits!.excluded.length && limits!.note ? ' or ' : ''}{limits!.note ? 'your movement note' : ''}. You decide.</p>{/if}
 <p class="small">Garmin’s completion flag isn’t read, and this workout isn’t counted toward your goals or matched to a recording. Change or remove it in Garmin Connect.</p>
 {#if session.date === today}<button class="opt" onclick={() => { act({action: 'dismiss', id: `garmin:${session.date}:${session.name}`}); close(); }}><Icon name="close" size={18}/><span><b>Not today</b><small>Today leads with something else</small></span></button>{/if}
 <button class="opt" onclick={() => { close(); navigate('plan'); }}><Icon name="arrow-right" size={18}/><span><b>Your plan</b><small>Your own sessions this week</small></span></button>
</div>
<style>
 .scrim{position:fixed;inset:0;border:0;background:rgba(5,8,14,.62);z-index:20;padding:0}
 .sheet{position:fixed;left:0;right:0;bottom:0;z-index:21;max-width:560px;margin:0 auto;background:#151B28;border:1px solid var(--line);border-bottom:0;border-radius:26px 26px 0 0;padding:12px 20px 28px;max-height:90vh;overflow:auto}
 .grab{display:block;width:40px;height:4px;border-radius:2px;background:rgba(255,255,255,.18);margin:0 auto 16px}
 .sheet-h{display:flex;justify-content:space-between;align-items:center;gap:12px}
 .close{width:44px;height:44px;border-radius:50%;border:1px solid var(--line);background:none;display:grid;place-items:center;color:var(--text2)}
 .warn{font-size:14px;line-height:1.45;margin:10px 0 0;padding:10px 12px;border-radius:12px;background:rgba(255,181,71,.08);border:1px solid rgba(255,181,71,.3)}
 .opt{display:grid;grid-template-columns:24px 1fr;gap:12px;align-items:center;width:100%;text-align:left;margin-top:8px;padding:12px 14px;border:1px solid var(--line);border-radius:16px;background:rgba(255,255,255,.04);color:var(--text);min-height:52px}
 .opt b{display:block;font:600 15px Inter}.opt small{display:block;font-size:12px;color:var(--text2);margin-top:2px}
</style>
