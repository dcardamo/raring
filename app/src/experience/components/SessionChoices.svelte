<script lang="ts">
 import Icon from '../../lib/components/Icon.svelte';
 import Glyph from './Glyph.svelte';
 import {store} from '../lib/store.svelte';
 import {A} from '../lib/actions';
 import {SPORTS} from '../lib/sports';
 import {dayLabel} from '../lib/dates';
 let ctx=$derived(store.ctx);
 let saved=$derived(ctx.state.intentions.filter(i=>i.kind!=='sleep'&&i.status!=='released'&&i.status!=='missed'));
 // The sports this person actually records, most frequent first.
 let sports=$derived.by(()=>{const n=new Map<string,number>();for(const a of ctx.world.activities)n.set(a.sport,(n.get(a.sport)??0)+1);return [...n.entries()].sort((a,b)=>b[1]-a[1]).map(([s])=>s as typeof ctx.world.activities[number]['sport']);});
</script>
{#if ctx.paused||ctx.restActive}<p class="small">{ctx.paused?'Training is paused until you resume.':'Your rest day is active through tonight.'}</p><button class="secondary" onclick={()=>store.navigate('plan')}>View your plan</button>{:else}
 {#if ctx.scheduleToday.length}<h3>From Garmin</h3>{#each ctx.scheduleToday as s}<button class="rowlink" disabled={ctx.limits.exclude.includes(s.sport)} onclick={()=>{A.keepGarmin(s.id);store.closeSheet();}}><Glyph sport={s.sport} mark="garmin" size={30}/><span><b>{s.name}</b><small>Garmin schedule · read-only</small></span><Icon name="chevron" size={16}/></button>{/each}{/if}
 {#if saved.length}<h3>Your saved plans</h3>{#each saved.slice(0,5) as i}<button class="rowlink" disabled={ctx.limits.exclude.includes(i.sport)} onclick={()=>i.date===ctx.calendarDate?A.selectToday(i.id):store.openSheet('allocate','template:'+i.id)}><Glyph sport={i.sport} mark="plan" size={30}/><span><b>{i.name??SPORTS[i.sport].name}</b><small>{i.date===ctx.calendarDate?'Today':dayLabel(i.date)}{i.time?` · ${i.time}`:''}{i.date!==ctx.calendarDate?' · use as a template':''}</small></span><Icon name="chevron" size={16}/></button>{/each}{/if}
 <h3>Pick a sport</h3><div class="sports">{#each sports as s}<button disabled={ctx.limits.exclude.includes(s)} onclick={()=>store.openSheet('allocate','sport:'+s)}><Glyph sport={s} mark="plan" size={30}/>{SPORTS[s].name}{ctx.limits.exclude.includes(s)?' · excluded':''}</button>{/each}</div>
 {#if ctx.limits.note}<p class="small">Review needed · {ctx.limits.note}</p>{/if}
 <button class="secondary" onclick={A.rest}><Icon name="moon" size={16}/>Rest today</button>
 <p class="small">Your choices stay local. Garmin’s calendar is unchanged.</p>
{/if}
<style>h3{font:600 11px Inter;letter-spacing:.1em;text-transform:uppercase;color:var(--text3);margin:18px 0 4px}.sports{display:flex;flex-wrap:wrap;gap:8px;margin-top:10px}.sports button{display:flex;gap:8px;align-items:center;padding:8px 12px;border:1px solid var(--line);border-radius:14px;background:var(--card);color:var(--text);font:600 14px Inter}button:disabled{opacity:.4}</style>
