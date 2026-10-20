<script lang="ts">
 import type {TodaySession} from '../generated/TodaySession';
 import type {Snippet} from 'svelte';
 // Today's session below the day's readiness context: what it is in plain
 // words, with the actions that act on it. Garmin's items carry ◇.
 let {session, children}: {session: TodaySession, children?: Snippet} = $props();
 let garmin = $derived(session.source !== 'own');
</script>
<section class="session" aria-labelledby="session-h">
 <p class="kicker">Today’s session</p>
 <h3 id="session-h">{session.name}{#if session.time}<span class="time"> · {session.time}</span>{/if}</h3>
 <p class="src">{garmin ? `◇ ${session.sourceLabel} · read-only` : session.sourceLabel}</p>
 {#if session.what}<p class="what">{session.what}</p>{/if}
 {#if session.effect}<p class="effect">{session.effect}{#if session.garminEffect}<small>{` · Garmin: ${session.garminEffect}`}</small>{/if}</p>{/if}
 {#if session.note}<p class="note">{session.note}</p>{/if}
 {#if session.fit}<p class="fit" role="note">{`▲ ${session.fit}`}</p>{/if}
 {#if children}<div class="acts">{@render children()}</div>{/if}
</section>
<style>
 .session{margin:14px 0 4px;padding:14px 16px;border-radius:22px;background:var(--card);border:1px solid var(--line);max-width:620px}
 .kicker{font:600 11px Inter;letter-spacing:.14em;text-transform:uppercase;color:var(--text3)}
 h3{font:600 19px/1.2 Outfit,Inter;margin:6px 0 2px;overflow-wrap:anywhere}.time{color:var(--text2);font-weight:500}
 .src{font-size:12px;color:var(--text2)}
 .what{margin-top:8px;font:500 14px Inter;color:var(--text)}
 .effect{margin-top:4px;font-size:13px;color:var(--text)}.effect small{font-size:12px;color:var(--text2)}
 .note{margin-top:4px;font-size:13px;color:var(--text2)}
 .fit{margin-top:6px;font:600 13px Inter;color:var(--fair)}
 .acts{display:flex;gap:8px;flex-wrap:wrap;margin:12px 0 0}.acts :global(button){width:auto;flex:1 1 140px;margin-top:0}
</style>
