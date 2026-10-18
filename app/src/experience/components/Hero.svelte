<script lang="ts">
 import Gauge from '../../lib/components/Gauge.svelte';
 import Icon from '../../lib/components/Icon.svelte';
 import Hypnogram from '../../lib/components/Hypnogram.svelte';
 import BodyMap from '../../lib/components/BodyMap.svelte';
 import {colors, toneColor} from '../../lib/charts';
 import RouteHero from './RouteHero.svelte';
 import Ring from './Ring.svelte';
 import Glyph from './Glyph.svelte';
 import TonightDial from './TonightDial.svelte';
 import {SPORTS} from '../lib/sports';
 import {contribution, effortFor, type Ctx} from '../lib/rules';
 import {routeFor, hrTrace, stagesFor} from '../lib/world';
 import {strengthSession, todayView} from '../lib/views';
 import {dur, dayLabel, hm, toMin} from '../lib/dates';
 import {elapsed} from '../../lib/format';
 // Today's one adaptive hero: its picture matches what matters now.
 let {ctx, navigate, openSheet, dismiss}: {ctx: Ctx, navigate: (r: string) => void, openSheet: (n: string, a?: string) => void, dismiss: (id: string) => void} = $props();
 let l = $derived(ctx.lead);
 let r = $derived(ctx.readiness);
 let n = $derived(ctx.night);
 let rTone = $derived(r ? (r.value >= 75 ? 'good' : r.value >= 50 ? 'fair' : 'poor') : 'neutral');
 let variant = $derived(
  l.kind === 'done' ? 'workout' : l.kind === 'waiting' || l.kind === 'stale' ? 'sync' : l.kind === 'rest' || l.kind === 'paused' ? 'rest'
  : l.kind === 'sleep' && ['evening', 'winddown', 'late'].includes(ctx.phase) ? 'tonight' : l.kind === 'sleep' ? 'sleep' : 'readiness');
 let tint = $derived(variant === 'tonight' ? '#5C7CFF' : variant === 'rest' ? '#A9C8FF' : variant === 'sync' ? '#8E98AC' : variant === 'workout' ? (l.activity ? SPORTS[l.activity.sport].color : colors.good) : variant === 'sleep' ? colors.light : toneColor(rTone));
 let factors = $derived(r ? todayView(ctx).factors.filter(f => f.percent != null).sort((a, b) => (a.percent ?? 0) - (b.percent ?? 0)) : []);
 let effort = $derived(effortFor(r));
 const act = () => l.action.sheet ? openSheet(l.action.sheet) : navigate(l.action.go!);
 let a = $derived(l.activity);
 let session = $derived(a?.sport === 'strength' ? strengthSession(a) : null);
 let contrib = $derived(a ? contribution(ctx, a) : null);
 let pending = $derived(ctx.intentionsToday.find(i => !i.link));
</script>

<section class="hero v-{variant}" style:--tint={tint} aria-labelledby="hero-h">
 {#if variant === 'workout' && a}
  <div class="hero-pic" class:strength={!!session}>
   {#if routeFor(a).length}<RouteHero route={routeFor(a)} samples={hrTrace(a)} floors={[118, 136, 150, 163]} color={SPORTS[a.sport].color} h={176} label={`${a.name} route, coloured by heart-rate zone`}/>
   {:else if session?.setsRecorded}<div class="bm-hero"><BodyMap figure="female" muscles={session.muscles} width={86} max={6} legend={false}/></div>
   {:else}<div class="art-hero" style:color={SPORTS[a.sport].color}><Icon name={SPORTS[a.sport].icon} size={72}/></div>{/if}
   <span class="pic-kicker"><Glyph sport={a.sport} size={22}/>Recorded today · {a.start}</span>
  </div>
  <div class="hero-body">
   <div class="hero-title">
    <div><h2 id="hero-h">{#if l.celebrate && !ctx.state.quiet}<span class="spark" class:seen={!!ctx.state.celebrated[l.celebrate]} aria-hidden="true"><Icon name="check" size={18}/></span>{/if}{l.headline}</h2>{#if l.line}<p class="line">{l.line}</p>{/if}</div>
    {#if contrib?.ring}<Ring value={contrib.ring[0]} target={contrib.ring[1]} size={58} stroke={6} color={SPORTS[a.sport].color} state={contrib.ring[0] >= contrib.ring[1] ? 'reached' : 'open'} label={contrib.short}><b class="n ring-n">{contrib.ring[0]}<small>/{contrib.ring[1]}</small></b></Ring>{/if}
   </div>
   <div class="bigstats">
    {#if a.km}<div><b class="n">{a.km}<small>km</small></b><span>Distance</span></div>{/if}
    <div><b class="n">{elapsed(a.min * 60)}</b><span>Time</span></div>
    {#if session?.setsRecorded}<div><b class="n">{session.sets}</b><span>Sets</span></div>{/if}
    {#if a.km && a.sport !== 'cycling'}<div><b class="n">{elapsed(a.min * 60 / a.km)}<small>/km</small></b><span>Pace</span></div>{/if}
    <div><b class="n">{a.avgHr}<small>bpm</small></b><span>Avg HR</span></div>
   </div>
   <div class="effects-row">
    <div class="te"><Gauge size={62} stroke={6} value={a.aerobic} max={5} color={colors.ink}/><span>Aerobic<br><b>{a.benefit}</b></span></div>
    <div class="te"><Gauge size={62} stroke={6} value={a.anaerobic} max={5} color={colors.secondary}/><span>Anaerobic</span></div>
    {#if r}<button class="chip-btn" onclick={() => navigate('readiness')} aria-label={`Readiness ${r.value}, ${r.level}`}><i style:background={toneColor(rTone)}></i>Readiness {r.value}</button>{/if}
   </div>
   <div class="actions"><button class="primary" onclick={act}>{l.action.label}</button><button class="link" onclick={() => openSheet('why')}>Why this?</button></div>
  </div>

 {:else if variant === 'tonight'}
  <div class="hero-body tn-body">
   <div class="tonight-top">
    <TonightDial now={ctx.clock.time} windDown={ctx.tonight.windDown} windowStart={ctx.tonight.window ? ctx.tonight.windowStart : '23:20'} windowEnd={ctx.tonight.window ? ctx.tonight.windowEnd : '23:20'} estimate={!ctx.tonight.window} wake={ctx.tonight.wake} lastBed={n?.bed} lastWake={n?.wake} need={dur(ctx.tonight.need)} size={164}/>
    <div class="tonight-text">
     <span class="kicker"><Icon name="moon" size={14}/>{ctx.tonight.nightLabel}</span>
     <h2 id="hero-h">{l.headline}</h2>
     {#if l.line}<p class="line">{l.line}</p>{/if}
     <dl class="tkeys">
      <div><dt><i class="k wd"></i>Wind down</dt><dd class="n">{ctx.tonight.windDown}</dd></div>
      <div><dt><i class="k win"></i>{ctx.tonight.window ? 'Garmin’s window' : 'Asleep by'}</dt><dd class="n">{ctx.tonight.window ?? '23:20'}</dd></div>
      <div><dt><i class="k wake"></i>Wake</dt><dd class="n">{ctx.tonight.wake}</dd></div>
     </dl>
    </div>
   </div>
   {#if n}<button class="lastnight" onclick={() => navigate('sleep')} aria-label={`Last night ${dur(n.asleep)}, open sleep`}><span class="ln-l"><small>Last night</small><b class="n">{dur(n.asleep)}</b></span><span class="ln-h"><Hypnogram segments={stagesFor(ctx.displayedDay, n.asleep)} startLocal={`${ctx.displayedDay}T${n.bed}:00`}/></span></button>{/if}
   {#if pending && ctx.phase !== 'late'}<button class="ctx-row" onclick={() => navigate(`intention/${pending.id}`)}><Glyph sport={pending.sport} mark="plan" size={26}/><span><b>{SPORTS[pending.sport].name}{pending.time ? ` · ${pending.time}` : ''}</b><small>Still in your plan</small></span><Icon name="chevron" size={16}/></button>{/if}
   <div class="actions"><button class="primary" onclick={act}>{l.action.label}</button><button class="link" onclick={() => openSheet('why')}>Why this?</button></div>
  </div>

 {:else if variant === 'rest'}
  <div class="hero-body rest-body">
   <div class="rest-top">
    <div class="rest-vis" aria-hidden="true">
     <svg viewBox="0 0 140 140" width="132" height="132">
      <defs><pattern id="rhatch" width="6" height="6" patternUnits="userSpaceOnUse" patternTransform="rotate(45)"><rect width="2" height="6" fill="#A9C8FF" opacity=".4"/></pattern>
       <radialGradient id="restglow"><stop offset=".4" stop-color="#A9C8FF" stop-opacity=".22"/><stop offset="1" stop-color="#A9C8FF" stop-opacity="0"/></radialGradient></defs>
      <circle cx="70" cy="70" r="68" fill="url(#restglow)"/>
      {#if l.kind === 'paused'}<circle cx="70" cy="70" r="50" fill="none" stroke="url(#rhatch)" stroke-width="12"/>
      {:else}<circle cx="70" cy="70" r="50" fill="none" stroke="rgba(169,200,255,.16)" stroke-width="12"/><path class="rest-arc" d={`M70,20 A50,50 0 1 1 ${70 - 50 * Math.sin(0.6)},${70 - 50 * Math.cos(0.6)}`} fill="none" stroke="#A9C8FF" stroke-width="12" stroke-linecap="round"/>{/if}
     </svg>
     <span class="rest-ico"><Icon name={l.kind === 'paused' ? 'pause' : 'moon'} size={40}/></span>
    </div>
    <div>
     <span class="kicker">{l.kind === 'paused' ? 'Adapt · paused' : 'Adapt · rest'}</span>
     <h2 id="hero-h">{l.headline}</h2>
     {#if l.line}<p class="line">{l.line}</p>{/if}
     {#if r}<button class="chip-btn" onclick={() => navigate('readiness')} aria-label={`Readiness ${r.value}, ${r.level}`}><i style:background={toneColor(rTone)}></i>Readiness {r.value} · {r.level}</button>{/if}
    </div>
   </div>
   {#if l.schedule}<div class="ctx-row static"><Glyph sport={l.schedule.sport} mark="garmin" size={26}/><span><b>{l.schedule.name}</b><small>Garmin schedule</small></span></div>{/if}
   <div class="actions"><button class="primary" onclick={act}>{l.action.label}</button><button class="link" onclick={() => openSheet('why')}>Why this?</button></div>
  </div>

 {:else if variant === 'sync'}
  <div class="hero-body sync-body">
   <div class="sync-vis" aria-hidden="true">
    <span class="dev"><svg viewBox="0 0 24 24" width="30" height="30" fill="none" stroke="currentColor" stroke-width="1.8"><rect x="6" y="6" width="12" height="12" rx="3.5"/><path d="M9 6l1-3h4l1 3M9 18l1 3h4l1-3"/></svg><small>Watch</small></span>
    <span class="wire"><i></i><b class="n">{ctx.upload.label}</b></span>
    <span class="dev"><svg viewBox="0 0 24 24" width="30" height="30" fill="none" stroke="currentColor" stroke-width="1.8"><rect x="7" y="2.5" width="10" height="19" rx="2.5"/><path d="M11 18.5h2"/></svg><small>Garmin Connect</small></span>
    <span class="wire short"><i></i></span>
    <span class="dev"><svg viewBox="0 0 24 24" width="30" height="30" fill="none" stroke="currentColor" stroke-width="1.8"><path d="M12 20s-7-4.4-7-10a4 4 0 0 1 7-2.6A4 4 0 0 1 19 10c0 5.6-7 10-7 10z"/></svg><small>Raring</small></span>
   </div>
   <span class="kicker"><Icon name="clock" size={14}/>Waiting for data</span>
   <h2 id="hero-h">{l.headline}</h2>
   {#if l.line}<p class="line">{l.line}</p>{/if}
   <div class="actions"><button class="primary" onclick={act}>{l.action.label}</button><button class="link" onclick={() => openSheet('why')}>Why this?</button></div>
  </div>

 {:else if variant === 'sleep'}
  <div class="hero-body sleep-body">
   <span class="kicker"><Icon name="moon" size={14}/>Last night · Garmin</span>
   <div class="sleep-top">
    <div><h2 id="hero-h">{l.headline}</h2>{#if l.line}<p class="line">{l.line}</p>{/if}</div>
    {#if n}<Gauge size={86} stroke={8} value={n.score} caption="Score" color={toneColor(n.score >= 80 ? 'good' : n.score >= 60 ? 'fair' : 'poor')}/>{/if}
   </div>
   {#if n}
    <div class="needbar2" role="img" aria-label={`Asleep ${dur(n.asleep)} against a need of ${dur(n.need)}`}><i style:width={Math.min(100, n.asleep / Math.max(540, n.need) * 100) + '%'}></i><u style:left={n.need / Math.max(540, n.need) * 100 + '%'}><span>Need {dur(n.need)}</span></u></div>
    <div class="hyp"><Hypnogram segments={stagesFor(ctx.displayedDay, n.asleep)} startLocal={`${ctx.displayedDay}T${n.bed}:00`}/></div>
   {/if}
   <div class="actions"><button class="primary" onclick={act}>{l.action.label}</button><button class="link" onclick={() => openSheet('why')}>Why this?</button></div>
  </div>

 {:else}
  <div class="hero-body r-body">
   <div class="r-top">
    <button class="gauge-btn" onclick={() => navigate('readiness')} aria-label={r ? `Readiness ${r.value}, ${r.level}. Open readiness` : 'Readiness unavailable'}>
     <Gauge size={136} stroke={12} value={r?.value ?? null} caption="Readiness" label={r?.level ?? ''} color={toneColor(rTone)}/>
    </button>
    <div class="r-text">
     <span class="kicker">{l.kind === 'schedule' || l.kind === 'conflict' || l.kind === 'review' ? 'Garmin schedule' : l.kind === 'intention' ? 'Your plan' : 'Readiness · Garmin'}</span>
     <h2 id="hero-h">{l.headline}</h2>
     {#if l.line}<p class="line">{l.line}</p>{/if}
    </div>
   </div>
   {#if l.schedule}<button class="ctx-row" class:warn={l.kind === 'conflict' || l.kind === 'review'} onclick={() => navigate(`scheduled/${l.schedule!.id}`)}><Glyph sport={l.schedule.sport} mark="garmin" size={28}/><span><b>{l.schedule.name}</b><small>{l.kind === 'conflict' ? 'Conflicts with your limits' : l.kind === 'review' ? 'Review needed' : `Garmin · ${l.schedule.min} min`}</small></span><Icon name="chevron" size={16}/></button>
   {:else if l.intention}<button class="ctx-row" class:warn={l.kind === 'review'} onclick={() => navigate(`intention/${l.intention!.id}`)}><Glyph sport={l.intention.sport} mark="plan" size={28}/><span><b>{SPORTS[l.intention.sport].name}{l.intention.time ? ` · ${l.intention.time}` : ''}</b><small>{l.kind === 'review' ? 'Review needed' : 'Your plan · this phone'}</small></span><Icon name="chevron" size={16}/></button>
   {:else if effort && (l.kind === 'suggest' || l.kind === 'easy')}
    <div class="effort" role="img" aria-label={`Garmin's effort suggestion: ${effort}`}>
     {#each ['rest', 'easy', 'steady', 'push'] as e}<span class:on={effort === e}>{e}</span>{/each}
    </div>
   {/if}
   {#if factors.length && (l.kind === 'suggest' || l.kind === 'easy' || l.kind === 'quiet')}
    <div class="factors2">
     {#each [factors.at(-1)!, factors[0]] as f, i}<button onclick={() => navigate('readiness')} aria-label={`${i ? 'Holding back' : 'Helping most'}: ${f.name} ${f.percent}`}><small>{i ? 'Holding back' : 'Helping most'}</small><span>{f.name}</span><i class="fb"><i style:width={f.percent + '%'} style:background={toneColor(f.tone)}></i></i></button>{/each}
    </div>
   {/if}
   {#if l.kind === 'fixed'}<p class="small fixed-note"><button class="link inline" onclick={() => navigate('settings')}>Settings</button></p>{/if}
   <div class="actions">
    <button class="primary" onclick={act}>{l.action.label}</button>
    <button class="link" onclick={() => openSheet('why')}>Why this?</button>
    {#if l.dismissable}<button class="link" onclick={() => dismiss(l.id)}>Not today</button>{/if}
   </div>
  </div>
 {/if}
</section>

<style>
 .hero{position:relative;margin-top:12px;border-radius:26px;overflow:hidden;background:linear-gradient(180deg,color-mix(in srgb,var(--tint) 13%,rgba(255,255,255,.05)),rgba(255,255,255,.04) 70%);border:1px solid color-mix(in srgb,var(--tint) 26%,rgba(255,255,255,.08));box-shadow:0 18px 50px -24px color-mix(in srgb,var(--tint) 60%,transparent)}
 .hero-body{padding:18px 18px 16px}
 .kicker{display:inline-flex;align-items:center;gap:6px;font:600 11px Inter;letter-spacing:.12em;text-transform:uppercase;color:color-mix(in srgb,var(--tint) 60%,#B4BCCB)}
 h2{font:600 26px/1.12 Outfit,Inter;margin-top:6px;color:var(--text);display:flex;align-items:center;gap:8px}
 .line{font-size:14px;color:var(--text2);margin-top:6px;line-height:1.35}
 .actions{display:flex;align-items:center;gap:16px;margin-top:16px;flex-wrap:wrap}
 .actions .primary{width:auto;margin:0;padding:13px 22px;flex:none}
 .link{background:none;border:0;color:var(--text);font-weight:600;text-decoration:underline;text-underline-offset:3px;padding:10px 0;min-height:44px}
 .link.inline{min-height:0;padding:0}
 .chip-btn{display:inline-flex;align-items:center;gap:7px;margin-top:10px;padding:7px 12px;border-radius:999px;border:1px solid var(--line);background:rgba(255,255,255,.06);font:600 12px Inter;color:var(--text)}
 .chip-btn i{width:8px;height:8px;border-radius:50%}
 .ctx-row{display:grid;grid-template-columns:auto 1fr auto;gap:12px;align-items:center;width:100%;text-align:left;margin-top:14px;padding:10px 12px;border-radius:16px;background:rgba(11,15,26,.35);border:1px solid var(--line);color:var(--text)}
 .ctx-row.static{grid-template-columns:auto 1fr}
 .ctx-row.warn{border-color:rgba(255,181,71,.45);background:rgba(255,181,71,.08)}
 .ctx-row b{display:block;font-weight:600}
 .ctx-row small{color:var(--text2);font-size:12px}
 /* readiness */
 .r-top{display:flex;align-items:center;gap:14px}
 .gauge-btn{background:none;border:0;padding:0;flex:none}
 .r-text{min-width:0}
 .effort{display:grid;grid-template-columns:repeat(4,1fr);gap:6px;margin-top:16px}
 .effort span{height:34px;border-radius:12px;display:grid;place-items:center;background:rgba(255,255,255,.05);color:var(--text3);font:600 13px Inter;text-transform:capitalize;position:relative}
 .effort span.on{background:var(--tint);color:#0B0F1A}
 .effort span.on::after{content:'Garmin';position:absolute;top:-8px;right:6px;font:700 8.5px Inter;letter-spacing:.08em;text-transform:uppercase;background:#0B0F1A;color:var(--tint);padding:1px 5px;border-radius:6px}
 .factors2{display:grid;grid-template-columns:1fr 1fr;gap:10px;margin-top:12px}
 .factors2 button{background:rgba(11,15,26,.3);border:1px solid var(--line);border-radius:14px;padding:9px 11px;text-align:left;color:var(--text)}
 .factors2 small{display:block;font-size:11px;color:var(--text3)}
 .factors2 span{display:block;font-weight:600;font-size:13px;margin-top:2px}
 .fb{display:block;height:5px;border-radius:3px;background:rgba(255,255,255,.07);margin-top:7px;overflow:hidden}
 .fb i{display:block;height:100%;border-radius:3px}
 .fixed-note{margin-top:8px}
 /* workout */
 .hero-pic{position:relative;height:176px;background:#0E1424;overflow:hidden}
 .hero-pic.strength{height:168px}
 .bm-hero,.art-hero{height:100%;display:grid;place-items:center;background:radial-gradient(80% 90% at 50% 40%,color-mix(in srgb,var(--tint) 24%,#121829),#0E1424)}
 .pic-kicker{position:absolute;left:12px;top:12px;display:inline-flex;align-items:center;gap:8px;padding:5px 10px 5px 5px;border-radius:999px;background:rgba(11,15,26,.72);backdrop-filter:blur(4px);font:600 12px Inter;color:var(--text)}
 .hero-title{display:flex;justify-content:space-between;gap:12px;align-items:flex-start}
 .hero-title h2{margin-top:0}
 .ring-n{font-size:17px}.ring-n small{font-size:11px;color:var(--text3)}
 .spark{display:inline-grid;place-items:center;width:28px;height:28px;border-radius:50%;background:#3DDC97;color:#0B0F1A;flex:none;animation:pop .7s cubic-bezier(.3,1.6,.4,1) both}
 .spark.seen{animation:none}
 @keyframes pop{from{transform:scale(.2);opacity:0}}
 .bigstats{display:grid;grid-template-columns:repeat(auto-fit,minmax(62px,1fr));gap:6px;margin-top:14px}
 .bigstats b{display:block;font-size:21px;font-weight:600;line-height:1.1;white-space:nowrap}
 .bigstats small{font:500 10.5px Inter;color:var(--text3);margin-left:1px}
 .bigstats span{font-size:12px;color:var(--text3)}
 .effects-row{display:flex;align-items:center;gap:14px;margin-top:12px;flex-wrap:wrap}
 .te{display:flex;align-items:center;gap:8px}
 .te span{font-size:12px;color:var(--text3);line-height:1.3}
 .te b{color:var(--text);font-weight:600}
 .effects-row .chip-btn{margin-top:0;margin-left:auto}
 /* tonight */
 .tonight-top{display:flex;gap:14px;align-items:center}
 .tonight-text{min-width:0}
 .tkeys{margin:10px 0 0;display:grid;gap:5px}
 .tkeys div{display:flex;justify-content:space-between;gap:10px;font-size:13px}
 .tkeys dt{color:var(--text2);display:flex;align-items:center;gap:6px}
 .tkeys dd{margin:0;color:var(--text);font-weight:600}
 .k{width:9px;height:9px;border-radius:2px;display:inline-block}
 .k.wd{background:#FFB547}.k.win{background:#A9C8FF}.k.wake{background:#EEF1F7;border-radius:50%}
 .lastnight{display:grid;grid-template-columns:auto 1fr;gap:12px;align-items:center;width:100%;margin-top:14px;padding:8px 12px;border-radius:16px;background:rgba(11,15,26,.35);border:1px solid var(--line);color:var(--text);text-align:left}
 .ln-l small{display:block;font-size:11px;color:var(--text3)}
 .ln-l b{font-size:18px}
 .ln-h{display:block;opacity:.9}
 .ln-h :global(.plot){max-height:64px}
 /* rest */
 .rest-top{display:flex;gap:16px;align-items:center}
 .rest-vis{position:relative;flex:none;width:132px;height:132px}
 .rest-ico{position:absolute;inset:0;display:grid;place-items:center;color:#A9C8FF}
 .rest-arc{animation:restsweep 1.4s ease-out both}
 @keyframes restsweep{from{stroke-dasharray:0 400}to{stroke-dasharray:400 400}}
 /* sync */
 .sync-vis{display:flex;align-items:center;gap:6px;margin:4px 0 16px}
 .dev{display:flex;flex-direction:column;align-items:center;gap:4px;color:var(--text2);flex:none}
 .dev small{font-size:10px;color:var(--text3)}
 .wire{flex:1;position:relative;display:grid;place-items:center;min-width:40px}
 .wire i{position:absolute;left:0;right:0;top:50%;border-top:2px dashed rgba(255,181,71,.5)}
 .wire b{position:relative;background:#141a2a;padding:2px 8px;border-radius:999px;font-size:12px;color:#FFB547;border:1px solid rgba(255,181,71,.35)}
 .wire.short i{border-top-color:rgba(255,255,255,.2)}
 /* sleep */
 .sleep-top{display:flex;justify-content:space-between;gap:12px;align-items:center}
 .needbar2{position:relative;height:12px;border-radius:6px;background:rgba(255,255,255,.07);margin:16px 0 22px}
 .needbar2 i{display:block;height:100%;border-radius:6px;background:linear-gradient(90deg,#2F6FE0,#5C9BFF)}
 .needbar2 u{position:absolute;top:-4px;bottom:-4px;width:2px;background:#EEF1F7;text-decoration:none}
 .needbar2 u span{position:absolute;top:22px;left:50%;transform:translateX(-60%);white-space:nowrap;font-size:11px;color:var(--text2)}
 .hyp{margin-top:4px}
 @media (prefers-reduced-motion:reduce){.spark,.rest-arc{animation:none}}
 @media(max-width:370px){.r-top{gap:8px}.tonight-top{flex-direction:column;align-items:flex-start}.rest-top{gap:10px}}
</style>
