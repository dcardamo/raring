<script lang="ts">
 import {untrack} from 'svelte';
 import Glyph from './Glyph.svelte';
 import {store} from '../lib/store.svelte';
 import {A} from '../lib/actions';
 import {goalsFor} from '../lib/allocation';
 import {SPORTS, type Sport} from '../lib/sports';
 import {SCHEDULE} from '../lib/world';
 import {addDays,dayLabel} from '../lib/dates';
 let {arg=''}:{arg?:string}=$props();
 const initial=untrack(()=>arg);
 const garmin=initial.startsWith('garmin:')?SCHEDULE.find(s=>s.id===initial.slice(7)):null;
 const template=initial.startsWith('template:')?store.state.intentions.find(i=>i.id===initial.slice(9)):null;
 const kind=initial==='sleep'?'sleep':'workout';
 const goals=untrack(()=>goalsFor(store.ctx));
 // The sports this person records, most frequent first, as on Today.
 const sports=untrack(()=>{const n=new Map<Sport,number>();for(const a of store.ctx.world.activities)n.set(a.sport,(n.get(a.sport)??0)+1);return [...n.entries()].sort((a,b)=>b[1]-a[1]).map(([s])=>s);});
 if(!sports.includes('walking'))sports.push('walking');
 if(!sports.includes('strength'))sports.unshift('strength');
 let sport=$state<Sport>(garmin?.sport??template?.sport??(initial==='strength'?'strength':initial.startsWith('sport:')?initial.slice(6) as Sport:sports.find(s=>s!=='strength')??'walking'));
 let dates=$state<string[]>([garmin?.date??store.ctx.calendarDate]);
 let time=$state(kind==='sleep'?goals.sleepTime:template?.time??'');
 let name=$state(garmin?.name??template?.name??'');
 let duration=$state(garmin?.min??template?.durationMin??0);
 let zone2=$state(0),im=$state(0);
 const days=Array.from({length:8},(_,i)=>addDays(store.ctx.calendarDate,i));
 const toggle=(d:string)=>dates=kind==='sleep'?(dates.includes(d)?dates.filter(x=>x!==d):[...dates,d]):[d];
 let excluded=$derived(kind!=='sleep' && store.state.limits.exclude.includes(sport));
 function save(){
  if(!dates.length||excluded||!Number.isFinite(zone2)||zone2<0||!Number.isFinite(im)||im<0||!Number.isFinite(duration)||duration<0)return;
  for(const date of dates)A.allocate({sport:kind==='sleep'?'other':sport,date,time:time||null,kind,name:kind==='sleep'?`${goals.sleepKind==='winddown'?'Wind down':'Bedtime'} ${time}`:name||SPORTS[sport].name,durationMin:duration||undefined,garminId:garmin?.id,allocation:kind==='sleep'?{sleep:1}:{...(sport==='strength'?{strength:1}:{}),...(zone2>0?{zone2}:{}),...(im>0?{im}:{})}},false);
  store.closeSheet();
 }
</script>
{#if garmin}<div class="from-garmin"><Glyph sport={garmin.sport} mark="garmin" size={26}/><span><b>{garmin.name}</b><small>Garmin schedule · {dayLabel(garmin.date)}</small></span></div>{/if}
{#if kind==='workout'}<div class="sports" role="group" aria-label="Sport">{#each sports as s}<button class="sport" aria-pressed={sport===s} disabled={store.state.limits.exclude.includes(s)} onclick={()=>sport=s}><Glyph sport={s} mark={sport===s?'recorded':'plan'} size={30}/>{SPORTS[s].name}{store.state.limits.exclude.includes(s)?' · excluded':''}</button>{/each}</div>{/if}
{#if excluded || kind==='workout'&&store.state.limits.note}<p class="small">{excluded?'Excluded by your limits.':'Review needed: your movement note applies.'}</p>{/if}
<fieldset><legend>{kind==='sleep'?'Choose nights':'Choose a day'}</legend><div class="chiprow">{#each days as d}<button class="chipb" aria-pressed={dates.includes(d)} onclick={()=>toggle(d)}>{d===store.ctx.calendarDate?'Today':dayLabel(d)}</button>{/each}</div></fieldset>
<label class="field"><span>{kind==='sleep'?'Chosen time':'Time (optional)'}</span><input type="time" bind:value={time}/></label>
{#if kind==='workout'}
 <label class="field col"><span>Name (optional)</span><input bind:value={name} placeholder={SPORTS[sport].name}/></label>
 <label class="field"><span>Duration (min, optional)</span><input class="qty" type="number" min="0" max="600" bind:value={duration}/></label>
 {#if goals.zone2>0 || initial==='zone2'}<label class="field"><span>Zone 2 minutes you intend</span><input class="qty" aria-label="Intended Zone 2 minutes" type="number" min="0" max="600" bind:value={zone2}/></label>{/if}
 {#if goals.im>0 || initial==='im'}<label class="field"><span>Intensity minutes you intend</span><input class="qty" aria-label="Planned intensity minutes" type="number" min="0" max="600" bind:value={im}/></label>{/if}
 {#if sport==='strength'}<p class="small">1 strength session</p>{/if}
{/if}
<button class="primary" disabled={excluded||!dates.length} onclick={save}>{kind==='sleep'?`Place ${dates.length} night${dates.length===1?'':'s'}`:'Place in my week'}</button>
<style>.from-garmin{display:flex;gap:10px;align-items:center;padding:10px 12px;border-radius:14px;border:1px solid rgba(164,139,255,.3);background:rgba(164,139,255,.07);margin-bottom:6px}.from-garmin b{display:block;font-weight:600}.from-garmin small{display:block;font-size:11px;color:#C9BEFF}.sports{display:flex;flex-wrap:wrap;gap:8px;margin:12px 0}.sport{display:flex;gap:7px;align-items:center;background:var(--card);border:1px solid var(--line);border-radius:14px;padding:8px 10px;color:var(--text);font:600 13px Inter}.sport[aria-pressed=true]{border-color:var(--ink)}.sport:disabled{opacity:.4}.qty{width:76px}</style>
