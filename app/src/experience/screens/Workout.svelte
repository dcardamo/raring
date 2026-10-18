<script lang="ts">
 import SportIcon from '../components/SportIcon.svelte';
 import Icon from '../../lib/components/Icon.svelte';
 import HelpSheet from '../../lib/components/HelpSheet.svelte';
 import WorkoutDetail from '../../lib/screens/WorkoutDetail.svelte';
 import {colors} from '../../lib/charts';
 import RouteHero from '../components/RouteHero.svelte';
 import Ring from '../components/Ring.svelte';
 import Glyph from '../components/Glyph.svelte';
 import {store} from '../lib/store.svelte';
 import {contribution, derive} from '../lib/rules';
 import {activityById} from '../lib/world';
 import {workoutView} from '../lib/views';
 import {dayLabel} from '../lib/dates';
 import {SPORTS} from '../lib/sports';
 // The one canonical workout detail, from every entrance: map first, then how it
 // counted, then the shipped cards in full.
 let {id}: {id: string} = $props();
 let a = $derived(activityById(id)!);
 let view = $derived(workoutView(a));
 let help = $state(false);
 let ctx = $derived(store.ctx);
 // Contribution is judged with the plan as it stood on the activity's own day.
 let then = $derived(derive({...store.state, day: a.date, clock: 'evening'}));
 let c = $derived(contribution(then, a));
 let linked = $derived(store.state.intentions.find(i => i.link === id));
</script>

<button class="back" onclick={() => store.back()}><Icon name="back" size={20}/> <span>{store.parentLabel}</span></button>
<header class="wo-head" style:--c={SPORTS[a.sport].color}>
 <div class="wo-pic" class:flat={!view.route.length}>
  {#if view.route.length}<RouteHero route={view.route} color={SPORTS[a.sport].color} h={210} label={`${a.name} recorded route`}/>
  {:else}<div class="wo-art" aria-hidden="true"><SportIcon sport={a.sport} size={110}/></div>{/if}
  <div class="wo-title">
   <Glyph sport={a.sport} size={34}/>
   <div><p class="eyebrow">{SPORTS[a.sport].name} · {dayLabel(a.date)} · {a.start}</p><h1 tabindex="-1">{a.name}</h1></div>
   {#if a.sport === 'strength'}<button class="avatar help-button" aria-label="Help" onclick={() => help = true}><Icon name="help" size={20}/></button>{/if}
  </div>
 </div>
</header>

<section class="counted" aria-labelledby="counted-t">
 <h2 id="counted-t" class="sr">How it counted</h2>
 <div class="ch-row">
  {#if c.ring}<div class="hc"><Ring value={c.ring[0]} target={c.ring[1]} size={40} stroke={4.5} color={SPORTS[a.sport].color} state={c.ring[0] >= c.ring[1] ? 'reached' : 'open'} label={c.short}/><span><small>Your week</small><b>{c.short}</b></span></div>
  {:else}<div class="hc"><Glyph sport={a.sport} size={36}/><span><small>Recorded</small><b>{a.timerOnly ? 'Strength session · no sets' : `${SPORTS[a.sport].name} session`}</b></span></div>{/if}
  {#if !a.timerOnly}<div class="hc"><Ring value={a.aerobic} target={5} size={40} stroke={4.5} color={colors.ink} label={`Garmin aerobic effect ${a.aerobic.toFixed(1)} of 5`}><b class="n te">{a.aerobic.toFixed(1)}</b></Ring><span><small>Garmin</small><b>{a.benefit} benefit</b></span></div>{/if}
  {#if a.record}<div class="hc best"><span class="star"><Icon name="star" size={18}/></span><span><small>Best in available history</small><b>{a.record}</b></span></div>{/if}
  {#if linked}<button class="hc" onclick={() => store.navigate(`intention/${linked.id}`)}><Glyph sport={linked.sport} mark="plan" size={36}/><span><small>Linked by you</small><b>Your plan · {dayLabel(linked.date)}</b></span></button>{/if}
 </div>
 {#if a.lateUpload}<p class="small late">Uploaded {dayLabel(a.lateUpload)} · counts on {dayLabel(a.date)}</p>{/if}
</section>

<WorkoutDetail workoutData={view} navigate={(r) => store.navigate(r)} help={() => help = true} showMap={false}/>
{#if help && view.strength}<HelpSheet close={() => help = false} more={() => { help = false; store.navigate('help/strength-sets'); }}/>{/if}

<style>
 .wo-head{margin-top:4px}
 .wo-pic{position:relative;height:210px;border-radius:24px;overflow:hidden;border:1px solid var(--line);background:#0E1424}
 .wo-pic.flat{height:150px;background:radial-gradient(90% 120% at 80% 30%,color-mix(in srgb,var(--c) 30%,#141b2e),#0E1424)}
 .wo-art{position:absolute;right:18px;top:12px;color:var(--c);opacity:.35}
 .wo-title{position:absolute;left:14px;right:14px;bottom:12px;display:flex;align-items:flex-end;gap:12px}
 .wo-title>div{flex:1;min-width:0}
 .wo-title h1{margin:2px 0 0;font-size:28px;text-shadow:0 2px 12px rgba(0,0,0,.6)}
 .wo-title .eyebrow{color:var(--text2);letter-spacing:.08em}
 .counted{margin-top:14px}
 .sr{position:absolute;width:1px;height:1px;overflow:hidden;clip:rect(0 0 0 0)}
 .ch-row{display:flex;gap:8px;overflow-x:auto;scrollbar-width:none;padding-bottom:2px}
 .hc{display:flex;align-items:center;gap:10px;flex:none;padding:8px 14px 8px 8px;border-radius:999px;background:var(--card);border:1px solid var(--line);color:var(--text);text-align:left}
 .hc small{display:block;font-size:10.5px;color:var(--text3);letter-spacing:.04em}
 .hc b{display:block;font-size:13px;font-weight:600;white-space:nowrap}
 .hc.best{border-color:rgba(255,214,102,.35);background:rgba(255,214,102,.07)}
 .star{width:36px;height:36px;border-radius:50%;display:grid;place-items:center;background:rgba(255,214,102,.16);color:#FFD666}
 .late{margin:8px 4px 0}
 .te{font-size:12px}
</style>
