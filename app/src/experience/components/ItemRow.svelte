<script lang="ts">
 import Icon from '../../lib/components/Icon.svelte';
 import Glyph from './Glyph.svelte';
 import {store} from '../lib/store.svelte';
 import {A} from '../lib/actions';
 import {itemMatch, STATE_WORD, type PlanItem} from '../lib/match';
 import {itemRoute} from '../lib/allocation';
 import {SPORTS} from '../lib/sports';
 import {dayLabel} from '../lib/dates';
 // One plan item, the same everywhere: done = filled with a check and the
 // recorded numbers; planned = outlined with the plan; passed and not
 // recorded = muted with Reschedule and Let it go.
 let {p, showDay = false, event = null, actions = true}: {p: PlanItem, showDay?: boolean, event?: string | null, actions?: boolean} = $props();
 let ctx = $derived(store.ctx);
 let m = $derived(itemMatch(ctx, p));
 let a = $derived(m.activity);
 let name = $derived(p.name ?? SPORTS[p.sport].name);
 const LABELS: Record<string, string> = {strength: 'strength', zone2: 'Zone 2 min', hi: 'Zone 4–5 min', im: 'intensity min', sleep: 'night'};
 let plan = $derived([p.time ?? (p.kind === 'sleep' ? '' : 'Any time'), p.min ? `${p.min} min` : '', Object.entries(p.allocation).filter(([, n]) => n).map(([k, n]) => `${n} ${LABELS[k]}`).join(' · ')].filter(Boolean).join(' · '));
</script>
<div class="ir s-{m.state}" class:ev={!!event} style:--ac={p.kind === 'sleep' ? '#A9C8FF' : SPORTS[p.sport].color}>
 <button class="ir-main" onclick={() => store.navigate(itemRoute(p))} aria-label={`${STATE_WORD[m.state]}: ${name}${showDay ? `, ${dayLabel(p.date)}` : ''}${a ? `, recorded ${a.min} min` : ''}${p.source === 'garmin' ? ', Garmin schedule' : ', your plan'}`}>
  <span class="gl"><Glyph sport={p.kind === 'sleep' ? 'other' : p.sport} mark={m.state === 'done' ? 'recorded' : p.source === 'garmin' ? 'garmin' : 'plan'} size={30}/>{#if m.state === 'done'}<i class="ck"><Icon name="check" size={11}/></i>{/if}</span>
  <span class="tx">
   <small><b class="st">{m.state === 'done' ? '✓ ' : ''}{STATE_WORD[m.state]}</b> · {p.source === 'garmin' ? 'Garmin' : 'Your plan'}{showDay ? ` · ${dayLabel(p.date).slice(0, 3)}` : ''}{event ? ` · ${event}` : ''}</small>
   <b>{name}</b>
   <em>{#if m.state === 'planned' && store.state.limits.exclude.includes(p.sport)}Conflicts with your limits · {:else if m.state === 'planned' && store.state.limits.note}Review needed · {/if}{#if a}{a.start} · {a.min} min{a.km ? ` · ${a.km} km` : ''} · avg HR {a.avgHr}{:else}{plan}{/if}</em>
  </span>
  <Icon name="chevron" size={15}/>
 </button>
 {#if actions && p.source === 'local' && p.intention}
  {#if m.state === 'planned' && p.date >= ctx.calendarDate && !p.intention.garminId}<button class="act" aria-label={`Move ${name} to another day`} onclick={() => store.openSheet('move', p.id)}>Move</button>
  {:else if m.state === 'not-recorded'}<span class="pass"><button class="act" onclick={() => store.openSheet('reschedule', p.id)}>Reschedule</button><button class="act" onclick={() => A.finishIntention(p.id, 'released')}>Let it go</button></span>{/if}
 {/if}
</div>
<style>
 .ir{display:flex;flex-wrap:wrap;gap:0 6px;align-items:stretch;margin-top:8px}
 .ir-main{flex:1 1 200px;min-width:0;display:grid;grid-template-columns:32px 1fr 15px;gap:10px;align-items:center;padding:9px 10px;border-radius:15px;border:1px solid var(--line);border-left:3px solid var(--ac);background:rgba(255,255,255,.03);color:var(--text);text-align:left}
 .gl{position:relative;display:grid}.ck{position:absolute;right:-3px;bottom:-3px;width:15px;height:15px;border-radius:50%;background:#3DDC97;color:#0B0F1A;display:grid;place-items:center}
 .tx{min-width:0}.tx small{font:500 10.5px Inter;color:var(--text3)}.tx b{display:block;font:600 15px Outfit;margin:2px 0}.tx em{font-style:normal;font-size:11.5px;color:var(--text2)}
 .st{display:inline!important;font:700 10.5px Inter!important;margin:0!important}
 .s-done .ir-main{border-color:rgba(61,220,151,.35);background:linear-gradient(110deg,rgba(61,220,151,.08),rgba(255,255,255,.02))}.s-done .st{color:#3DDC97}
 .s-planned .ir-main{border-style:dashed;border-left-style:solid;background:none}.s-planned .st{color:var(--text)}
 .s-not-recorded .ir-main,.s-released .ir-main,.s-missed .ir-main{opacity:.6;background:none}.s-not-recorded .st{color:var(--text2)}
 .s-review .ir-main{border-color:rgba(255,181,71,.45)}.s-review .st{color:#FFB547}
 .s-awaiting .ir-main{border-style:dotted}
 .ev .ir-main{margin-left:12px}
 .act{margin-top:8px;padding:0 10px;min-height:40px;border-radius:12px;border:1px solid var(--line);background:rgba(255,255,255,.04);color:var(--text);font:600 11.5px Inter;min-width:58px}
 .pass{display:flex;gap:6px}
</style>
