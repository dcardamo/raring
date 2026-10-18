<script lang="ts">
 import Header from '../../lib/components/Header.svelte';
 import Icon from '../../lib/components/Icon.svelte';
 import Card from '../../lib/components/Card.svelte';
 import AboutRaring from '../../lib/components/AboutRaring.svelte';
 import BrandMark from '../../lib/components/BrandMark.svelte';
 import {help as shippedHelp, GITHUB_QUESTIONS} from '../../lib/help';
 import Glyph, {MARK_NAMES, type Mark} from '../components/Glyph.svelte';
 import XIcon from '../components/XIcon.svelte';
 import {store, LEGACY, titleOf} from '../lib/store.svelte';
 import {A} from '../lib/actions';
 import {surfaces} from '../lib/rules';
 import {SPORTS, type Sport} from '../lib/sports';
 let {route}: {route: string} = $props();
 let ctx = $derived(store.ctx);
 let [key, sub] = $derived(route.split('/'));
 const PAGES: Record<string, {title: string, intro: string, steps: string[]}> = {
  sync: {title: 'How syncing works', intro: 'Your watch uploads to Garmin Connect; Raring then reads Garmin Connect.', steps: ['If the watch hasn’t uploaded, open Garmin Connect on the phone to sync it.', 'Raring shows the watch’s upload time, never its own fetch time.', 'Recorded activities keep their own dates when they arrive late.']},
  adapt: {title: 'Adapt today and circumstances', intro: 'Rest today covers today and the following night.', steps: ['Pauses and limits last until you resume them; a review date only reminds you.', 'Excluded activities are enforced in suggestions. A movement note stays visible; routines Raring can’t check need your review.', 'Raring doesn’t diagnose, clear exercise or suggest substitutes.']},
  'acute-load': {title: 'What acute load is', intro: 'Garmin weights your recent exercise loads into one number and compares it with a range based on your own history.', steps: ['Strength can contribute through physiology, but acute load doesn’t measure muscle recovery or strength progress.', 'It’s context, not a budget to fill today.', 'Garmin’s status and load focus explain what kind of load you’ve done.']},
 };
 const s = $derived(surfaces(ctx));
 const LEGEND: [Mark, Sport, string][] = [['recorded', 'running', 'Recorded by your watch'], ['plan', 'running', 'Your plan, on this phone'], ['garmin', 'running', 'Garmin schedule, read-only'], ['self', 'yoga', 'Self-reported'], ['rest', 'other', 'Rest day'], ['paused', 'other', 'Paused'], ['unknown', 'other', 'Awaiting evidence']];
 let legacyRows = $derived([...Object.keys(LEGACY), 'workout/a-1201', 'metric/hrv', 'metric/vo2max', 'help/strength-sets', 'nonexistent-page']);
</script>
{#snippet header(title: string, eyebrow = '')}<Header {title} {eyebrow} back={() => store.back()} backLabel={store.parentLabel}/>{/snippet}

{#if key === 'help' && sub === 'legend'}
 {@render header('Key', 'Shapes and colours')}
 <Card><ul class="legend">{#each LEGEND as [m, sp, t]}<li><Glyph sport={sp} mark={m} size={32} label={MARK_NAMES[m]}/><span><b>{MARK_NAMES[m]}</b><small>{t}</small></span></li>{/each}</ul></Card>
 <Card title="Activities"><div class="sports">{#each Object.entries(SPORTS) as [k, v]}<span><Glyph sport={k as Sport} size={26}/>{v.name}</span>{/each}</div></Card>
 <Card title="Your day"><p class="small">Stage colours for sleep, stress shading (rest, low, medium, high), heart rate in red, activity blocks, a dashed box for tonight’s window and a hatched future.</p></Card>
{:else if key === 'help' && sub && sub in shippedHelp}
 {@const p = shippedHelp[sub as keyof typeof shippedHelp]}
 {@render header(p.title, 'HELP')}
 <Card><p class="help-intro">{p.intro}</p><ol class="help-steps">{#each p.steps as st, i}<li><span class="n">{i + 1}</span>{st}</li>{/each}</ol></Card>
 <p class="small help-foot">Still stuck? Ask at {GITHUB_QUESTIONS}</p>
{:else if key === 'help' && sub && PAGES[sub]}
 {@const p = PAGES[sub]}
 {@render header(p.title, 'HELP')}
 <Card><p class="help-intro">{p.intro}</p><ol class="help-steps">{#each p.steps as st, i}<li><span class="n">{i + 1}</span>{st}</li>{/each}</ol>{#if sub === 'sync'}<button class="secondary" onclick={A.connect}>Open Garmin Connect</button>{/if}</Card>
{:else if key === 'help'}
 {@render header('Help', 'Offline')}
 <section class="card">
  {#each [...Object.entries(shippedHelp).map(([k, v]) => [k, v.title]), ...Object.entries(PAGES).map(([k, v]) => [k, v.title]), ['legend', 'Key: shapes and colours']] as [k, t]}<button class="rowlink" onclick={() => store.navigate(`help/${k}`)}><Icon name="help" size={18}/><span><b>{t}</b></span><Icon name="chevron" size={16}/></button>{/each}
 </section>

{:else if key === 'surfaces'}
 {@render header('Status surfaces', 'Illustrative · not native rendering')}
 <p class="small">Same suppression rules as Today. Taps open the app as a cold link.</p>
 <Card title="Lock screen">{#if s.morning}<button class="surf" onclick={() => store.external(s.morning!.go, 'Opened from a status surface')}><span class="nt"><XIcon name="lock" size={14}/>{s.morning.lock}</span><span class="nb">Contents hidden on the lock screen</span></button>{:else}<p class="small">No morning readiness notification: permission, channel or current evidence is missing.</p>{/if}</Card>
 <Card title="Notification shade">
  {#if s.morning}<button class="surf" onclick={() => store.external(s.morning!.go, 'Opened from a status surface')}><span class="nt">{s.morning.title}</span><span class="nb">{s.morning.body}</span><span class="ns">{s.morning.sub}</span></button>{/if}
  {#if s.statusEnabled}<button class="surf" onclick={() => store.external('today', 'Opened from a status surface')}><span class="nt">Raring · As of {s.as}</span><span class="nb">Body Battery {s.status.bb ?? '—'} · Stress {s.status.stress ?? '—'} · HR {s.status.hr ?? '—'}</span><span class="ns">{s.focus}</span></button>{:else}<p class="small">Persistent status off. Widgets remain independent.</p>{/if}
  {#if s.bedtime}<button class="surf" onclick={() => store.external(s.bedtime!.go, 'Opened from a status surface')}><span class="nt">{s.bedtime.title}</span><span class="nb">{s.bedtime.body}</span></button>{:else}<p class="small">Bedtime reminder is off.</p>{/if}
  {#if s.weekly}<button class="surf" onclick={() => store.external(s.weekly!.go, 'Opened from a status surface')}><span class="nt">{s.weekly.title}</span><span class="nb">{s.weekly.body}</span></button>{:else}<p class="small">Weekly notification not sent: permission or channel off.</p>{/if}
 </Card>
 <Card title="Widgets"><p class="small">As of {s.as}. Android may defer updates.</p><div class="widgets">
  <button class="wdg" onclick={() => store.external(ctx.readiness ? 'readiness' : 'today', 'Opened from a widget')}><b class="n">{ctx.readiness && ctx.phase !== 'waiting' ? ctx.readiness.value : '—'}</b><small>{s.focus}</small></button>
  <button class="wdg" onclick={() => store.external('sleep', 'Opened from a widget')}><b class="n">{ctx.night && ctx.phase !== 'waiting' ? ctx.night.score : '—'}</b><small>Sleep score</small></button>
  <button class="wdg" onclick={() => store.external('metric/bodyBattery', 'Opened from a widget')}><b class="n">{s.status.bb ?? '—'}</b><small>Body Battery</small></button>
  <button class="wdg" onclick={() => store.external('metric/stress', 'Opened from a widget')}><b class="n">{s.status.stress ?? '—'}</b><small>Stress</small></button>
 </div></Card>

{:else if key === 'legacy'}
 {@render header('Old links', 'Prototype check of the legacy-link map')}
 <section class="card">{#each legacyRows as from}{@const c = store.canonical(from)}{@const t = store.defaultTrail(c.route)}<button class="rowlink" onclick={() => store.external(from)}><XIcon name="flag" size={16}/><span><b>#{from}</b><small>→ {c.route} · Back: {t.slice(0, -1).map(x => titleOf(x)).reverse().join(' → ') || '—'}</small></span><Icon name="chevron" size={16}/></button>{/each}</section>

{:else if key === 'onboarding'}
 <section class="welcome intro">
  <div class="welcome-brand"><BrandMark/><h1>Raring</h1></div>
  <p class="welcome-tagline">A clearer view of your readiness, sleep and training.</p>
  <AboutRaring/>
  <button class="primary" onclick={() => store.navigate('today')}>Continue with synthetic data <Icon name="arrow-right" size={18}/></button>
  <p class="small">This prototype accepts no credentials and opens no account.</p>
 </section>
{:else}
 {@render header('Not found')}<Card><p>That page isn’t part of the prototype.</p></Card>
{/if}
<style>
 .legend{list-style:none;padding:0;margin:0;display:grid;gap:12px}
 .legend li{display:flex;gap:12px;align-items:center}
 .legend b{display:block}.legend small{color:var(--text2);font-size:12px}
 .sports{display:flex;flex-wrap:wrap;gap:10px 16px}
 .sports span{display:inline-flex;gap:8px;align-items:center;font-size:13px}
 .surf{display:flex;flex-direction:column;gap:2px;width:100%;margin-top:8px;padding:12px;border-radius:16px;border:1px solid var(--line);background:rgba(255,255,255,.06);color:var(--text);text-align:left}
 .nt{display:flex;gap:6px;align-items:center;font-weight:600;font-size:13px}
 .nb{font-size:14px}.ns{font-size:12px;color:var(--text2)}
 .widgets{display:grid;grid-template-columns:repeat(2,1fr);gap:10px;margin-top:10px}
 .wdg{display:flex;flex-direction:column;align-items:flex-start;padding:14px;border-radius:20px;background:#161d30;border:1px solid var(--line);color:var(--text)}
 .wdg b{font-size:30px;font-weight:600}.wdg small{font-size:12px;color:var(--text2)}
</style>
