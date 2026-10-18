<script lang="ts">
 import {tick} from 'svelte';
 import Icon from '../../lib/components/Icon.svelte';
 import GoalEditor from './GoalEditor.svelte';
 import PlanWeek from './PlanWeek.svelte';
 import AllocationEditor from './AllocationEditor.svelte';
 import SessionChoices from './SessionChoices.svelte';
 import {dayEvidence} from '../lib/allocation';
 import {dayRings} from '../lib/week';
 import NestedRings from './NestedRings.svelte';
 import {search} from '../lib/searchIndex';
 import Glyph, {MARK_NAMES, type Mark} from './Glyph.svelte';
 import XIcon from './XIcon.svelte';
 import {store} from '../lib/store.svelte';
 import {A} from '../lib/actions';
 import {addDays, dayLabel, weekMonday} from '../lib/dates';
 import {versionFor} from '../lib/rules';
 import {PLAN_SPORTS, SPORTS, type Sport} from '../lib/sports';
 let sheet = $derived(store.sheet);
 let ctx = $derived(store.ctx);
 let box = $state<HTMLElement>();
 // Add-to-plan draft: activity and day are enough; time is optional.
 let draft = $state<{sport: Sport | null, date: string, time: string}>({sport: null, date: '', time: ''});
 let query = $state(''), ambName = $state(''), ambDate = $state(''), rescheduleDate = $state('');
 $effect(() => {
  if (sheet?.name === 'reschedule') rescheduleDate=store.ctx.calendarDate;
  if (sheet?.name === 'add') {
   const editing = sheet.arg ? store.state.intentions.find(i => i.id === sheet.arg) : null;
   draft = {sport: editing?.sport ?? null, date: editing?.date ?? store.ctx.calendarDate, time: editing?.time ?? ''};
  }
  if (sheet) void tick().then(() => (box?.querySelector('[data-autofocus]') as HTMLElement ?? box?.querySelector('h2'))?.focus());
 });
 function keys(e: KeyboardEvent) {
  if (!sheet) return;
  if (e.key === 'Escape') store.closeSheet();
  if (e.key === 'Tab' && box) {
   const nodes = [...box.querySelectorAll<HTMLElement>('button:not(:disabled), input, select, textarea')];
   const first = nodes[0], last = nodes.at(-1);
   if (e.shiftKey && (document.activeElement === first || document.activeElement?.tagName === 'H2')) { e.preventDefault(); last?.focus(); }
   else if (!e.shiftKey && document.activeElement === last) { e.preventDefault(); first?.focus(); }
  }
 }
 const days = $derived(Array.from({length: 8}, (_, i) => addDays(ctx.calendarDate, i)));
 const TITLES: Record<string, string> = {adapt: 'Adapt today', add: 'Add to my plan', why: 'Why this?', notices: 'Notices', permission: 'Allow notifications?', winddown: 'A wind-down intention', consent: 'Allow Garmin journal reads?', resume: 'Resume training', ambition: 'Add an ambition', signout: 'Sign out and erase?', legend: 'Key', versions: 'Commitment targets', search: 'Search', planweek: 'Plan your week', move: 'Move to another day', goals:'Your weekly goals', allocate:'Place in your week', choose:'Choose another', reschedule:'Reschedule locally', day:'Your day'};
 const LEGEND: [Mark, Sport, string][] = [['recorded', 'running', 'Recorded by your watch'], ['plan', 'running', 'Your plan, on this phone'], ['garmin', 'running', 'Garmin schedule, read-only'], ['self', 'yoga', 'Self-reported'], ['rest', 'other', 'Rest day'], ['paused', 'other', 'Paused'], ['unknown', 'other', 'Awaiting evidence']];
</script>
<svelte:window onkeydown={keys}/>
{#if sheet}
 <div class="scrim" role="presentation" onclick={() => store.closeSheet()}></div>
 <div class="sheet" role="dialog" aria-modal="true" aria-labelledby="sheet-t" bind:this={box}>
  <div class="sheet-h"><h2 id="sheet-t" tabindex="-1">{sheet.name === 'add' && sheet.arg ? 'Edit your plan' : TITLES[sheet.name] ?? ''}</h2><button class="iconbtn" aria-label="Close" onclick={() => store.closeSheet()}><Icon name="close" size={18}/></button></div>

  {#if sheet.name === 'search'}
   <label class="field col"><span class="sr">Search metrics and details</span><input type="search" data-autofocus placeholder="Search metrics, sleep, exercises…" bind:value={query} autocomplete="off"/></label>
   {#each search(query) as e}<button class="rowlink" onclick={() => { const r = e.route; query = ''; store.navigate(r); }}><Icon name="chevron" size={14}/><span><b>{e.name}</b><small>{e.area}</small></span><Icon name="arrow-right" size={14}/></button>{:else}{#if query.trim()}<p class="small">Nothing by that name. Training and Health hold every metric.</p>{/if}{/each}
  {:else if sheet.name === 'planweek'}<PlanWeek/>
  {:else if sheet.name === 'move'}
   {@const mi=store.state.intentions.find(x=>x.id===sheet!.arg)}
   {#if mi}<p class="small">{mi.name??SPORTS[mi.sport].name} · now {dayLabel(mi.date)}. Only your plan moves.</p>
    <div class="chiprow">{#each days.filter(d=>d!==mi.date) as d}<button class="chipb" onclick={()=>A.reschedule(mi.id,d)}>{d===ctx.calendarDate?'Today':dayLabel(d).replace(/ \w+$/,'')}</button>{/each}</div>{/if}
  {:else if sheet.name === 'goals'}<GoalEditor/>
  {:else if sheet.name === 'allocate'}<AllocationEditor arg={sheet.arg}/>
  {:else if sheet.name === 'choose'}<SessionChoices/>
  {:else if sheet.name === 'reschedule'}
   <label class="field"><span>New date</span><input aria-label="New date" type="date" min={ctx.calendarDate} bind:value={rescheduleDate}/></label>
   <button class="primary" disabled={!rescheduleDate || rescheduleDate<ctx.calendarDate} onclick={()=>A.reschedule(sheet!.arg!,rescheduleDate)}>Move my session</button>
  {:else if sheet.name === 'day'}
   {@const day=sheet.arg??ctx.calendarDate}{@const d=dayRings(ctx,day)}{@const ev=dayEvidence(ctx,day)}
   <div class="day-head"><NestedRings rings={d.rings} size={64} stroke={5} gap={2} mark={d.mark} future={d.future}/><div><b>{dayLabel(day)}</b><small>{d.mark==='rest'?'Rest day':d.mark==='ill'?'Ill · goals paused':d.mark==='paused'?'Paused':d.mark==='unknown'?'Awaiting Garmin data':d.future?'Ahead':day===ctx.calendarDate?'Today':''}{ev.night?` · Sleep Score ${ev.night.score} (Garmin)`:''}</small></div></div>
   {#if d.rings.some(r=>r.outline)}<table class="dgoals"><thead><tr><th></th><th>Planned</th><th>Recorded</th><th>Left</th></tr></thead><tbody>
    {#each d.rings.filter(r=>r.outline) as r}<tr style:--c={r.color}><th><i></i>{r.name}</th><td>{r.planned||'—'}</td><td>{d.future?'—':d.mark==='unknown'?'?':r.recorded}</td><td>{d.future||d.mark==='unknown'?'—':Math.max(0,r.planned-r.recorded)||'—'}</td></tr>{/each}
   </tbody></table>{:else}<p class="small">Nothing planned{d.future?'':' or recorded toward your goals'}.</p>{/if}
   {#each ev.plans as i}<button class="rowlink" onclick={()=>store.navigate(`intention/${i.id}`)}><Glyph sport={i.kind==='sleep'?'other':i.sport} mark="plan" size={26}/><span><b>{i.name??SPORTS[i.sport].name}</b><small>Your plan{i.time?' · '+i.time:''}</small></span><Icon name="chevron" size={16}/></button>{/each}
   {#each ctx.world.schedule.filter(g=>g.date===day) as g}<button class="rowlink" onclick={()=>store.navigate(`scheduled/${g.id}`)}><Glyph sport={g.sport} mark="garmin" size={26}/><span><b>{g.name}</b><small>Garmin schedule · {g.min} min</small></span><Icon name="chevron" size={16}/></button>{/each}
   {#each ev.acts as a}<button class="rowlink" onclick={()=>store.navigate(`workout/${a.id}`)}><Glyph sport={a.sport} size={26}/><span><b>{a.name}</b><small>Recorded · {a.start} · {a.min} min</small></span><Icon name="chevron" size={16}/></button>{/each}
  {:else if sheet.name === 'adapt'}
   {#if ctx.restActive}<div class="state-row"><Glyph mark="rest" size={34}/><span><b>Rest day is on</b><small>Through tonight</small></span></div><button class="secondary" onclick={A.unrest}>Undo rest</button>
   {:else}<button class="primary big" data-autofocus onclick={A.rest}><Icon name="moon" size={18}/>Rest today</button><p class="small">No exercise prompts today or tonight.</p>{/if}
   {#if !ctx.paused}<button class="secondary" onclick={A.ill}><Icon name="pause" size={16}/>I’m ill</button><p class="small">Pauses goals and the backlog until you resume. No catch-up.</p>{/if}
   <div class="list">
    <button class="rowlink" onclick={() => store.navigate('adapt/pause')}><Icon name="pause" size={20}/><span><b>Pause training</b><small>Until you resume</small></span>{#if ctx.paused}<span class="val">On</span>{:else}<Icon name="chevron" size={16}/>{/if}</button>
    <button class="rowlink" onclick={() => store.navigate('adapt/limits')}><XIcon name="adapt" size={20}/><span><b>Avoid activities or movements</b><small>Exclusions and your own note</small></span>{#if ctx.hasLimits}<span class="val">On</span>{:else}<Icon name="chevron" size={16}/>{/if}</button>
    <button class="rowlink" onclick={() => store.navigate('adapt/routines')}><Icon name="check" size={20}/><span><b>My routines</b><small>A checklist you supply</small></span><Icon name="chevron" size={16}/></button>
    <button class="rowlink" onclick={() => store.navigate('adapt/lighter')}><Icon name="clock" size={20}/><span><b>Busy or travelling</b><small>A lighter week</small></span><Icon name="chevron" size={16}/></button>
   </div>

  {:else if sheet.name === 'add'}
   <fieldset><legend>Activity</legend><div class="sport-grid">
    {#each PLAN_SPORTS as s}{@const ex = store.state.limits.exclude.includes(s)}<button class="sport" aria-pressed={draft.sport === s} disabled={ex} onclick={() => draft.sport = s}><Glyph sport={s} mark={draft.sport === s ? 'recorded' : 'plan'} size={34} label={SPORTS[s].name}/><span>{SPORTS[s].name}{ex ? ' · excluded' : ''}</span></button>{/each}
   </div>{#if draft.sport && store.state.limits.note}<p class="small">Review needed: your movement note applies.</p>{/if}</fieldset>
   <fieldset><legend>Day</legend><div class="chiprow">{#each days as d}<button class="chipb" aria-pressed={draft.date === d} onclick={() => draft.date = d}>{d === ctx.calendarDate ? 'Today' : dayLabel(d).replace(/ \w+$/, '')}</button>{/each}</div></fieldset>
   <label class="field"><span>Time (optional)</span><input type="time" bind:value={draft.time}/></label>
   <button class="primary" disabled={!draft.sport} onclick={() => draft.sport && A.saveIntention({sport: draft.sport, date: draft.date, time: draft.time || null}, sheet?.arg)}>{sheet.arg ? 'Save changes' : 'Add to plan'}</button>

  {:else if sheet.name === 'why'}
   <ol class="trace">{#each ctx.lead.trace as t, i}<li><span class="step n">{i + 1}</span><span><b>{t.gate}</b><small>{t.result}</small></span></li>{/each}</ol>

  {:else if sheet.name === 'notices'}
   {#each ctx.notices as n}<div class="notice-strip"><Icon name="alert" size={18}/><p>{n.text}</p><button class="link inline" onclick={() => store.navigate(n.action.go)}>{n.action.label}</button></div>{/each}

  {:else if sheet.name === 'permission'}
   <p>The app stays fully useful if you decline.</p>
   <button class="primary" onclick={() => A.permit('granted', sheet!.arg ?? '')}>Allow</button><button class="secondary" onclick={() => A.permit('declined', sheet!.arg ?? '')}>Decline</button>

  {:else if sheet.name === 'winddown'}
   <p>Make room to wind down this evening. Marking it done is optional and kept apart from recorded sleep.</p>
   <button class="primary" onclick={A.winddownIntention}>Add wind-down intention</button>

  {:else if sheet.name === 'consent'}
   <ul class="plain"><li>Last 7 days of Garmin lifestyle logs, read-only.</li><li>Entries stay closed until you open them, and hide when you leave or switch apps.</li><li>Never in Today, Progress, search or notifications.</li><li>Turning this off stops new reads. Downloaded logs stay on this phone and in exports until you erase local data.</li></ul>
   <button class="primary" data-autofocus onclick={A.journalAllow}>Allow journal reads</button><button class="secondary" onclick={() => store.closeSheet()}>Not now</button>

  {:else if sheet.name === 'resume'}
   <p>Your commitments as they were:</p>
   {#if ctx.plan}<div class="state-row"><Glyph sport="strength" mark="plan" size={30}/><span><b>{ctx.plan.target.strength} strength · {ctx.plan.target.im} min</b><small>A week</small></span></div>{:else}<p class="small">No commitments.</p>{/if}
   <button class="primary" onclick={A.resume}>Resume as they are</button><button class="secondary" onclick={A.resumeChange}>Resume and change commitments</button>

  {:else if sheet.name === 'ambition'}
   <label class="field col"><span>Name</span><input bind:value={ambName} placeholder="For example: Hike in May" data-autofocus/></label>
   <label class="field col"><span>Date (optional)</span><input type="date" bind:value={ambDate}/></label>
   <button class="primary" disabled={!ambName.trim()} onclick={() => { A.saveAmbition(ambName, ambDate); ambName = ''; ambDate = ''; }}>Add</button>
   <p class="small center">A local entry · not a Garmin race</p>

  {:else if sheet.name === 'signout'}
   <p>This erases from this phone:</p>
   <ul class="plain"><li>Downloaded Garmin history, including any journal logs</li><li>Your plans, intentions, commitments and ambitions</li><li>Rest, pauses, limits and notes, routine checklists and self-reports</li><li>Dismissals and settings</li></ul>
   <p class="small">Sign-in tokens are removed. Nothing in Garmin changes.</p>
   <button class="secondary" onclick={() => store.navigate('settings/export')}>Export first</button><button class="danger" onclick={A.signout}>Sign out and erase</button>

  {:else if sheet.name === 'versions'}
   {@const vs = versionFor(ctx)}
   <p class="small">Each week is measured against the targets it had. Changes never rewrite earlier weeks.</p>
   <ol class="steps">
    {#each vs.versions.filter(v => v.from <= vs.wk) as v}<li><span class="when">From {dayLabel(weekMonday(v.from))}</span><span class="dots" aria-hidden="true">{#each Array(v.strength) as _}<i></i>{/each}</span><span class="what">{v.strength} strength · {v.im} min<small>{v.note}</small></span></li>{/each}
    {#each vs.revisions as r}<li><span class="when">From {dayLabel(r.from)}</span><span class="dots" aria-hidden="true">{#each Array(r.strength) as _}<i></i>{/each}</span><span class="what">{r.strength} strength · {r.im} min<small>Changed this week</small></span></li>{/each}
   </ol>
  {:else if sheet.name === 'legend'}
   {@const R=(f:number,o:number,k:'strength'|'zone2'='strength')=>[{key:k,name:'',unit:'',color:k==='strength'?'#FF7DB8':'#5CA8FF',recorded:0,planned:0,filled:f,outline:o}]}
   <h3 class="lg-h">Day rings · one per goal, same order everywhere</h3>
   <ul class="legend">
    <li><NestedRings rings={R(1,1)} size={30}/><span><b>Filled</b><small>Recorded toward that day’s planned share</small></span></li>
    <li><NestedRings rings={R(.4,1)} size={30}/><span><b>Thin line</b><small>Planned, not recorded (yet)</small></span></li>
    <li><NestedRings rings={[]} size={30}/><span><b>No ring</b><small>Nothing planned or recorded for that goal</small></span></li>
    <li><NestedRings rings={[]} size={30} mark="rest"/><span><b>Moon</b><small>Rest day you chose</small></span></li>
    <li><NestedRings rings={[]} size={30} mark="ill"/><span><b>Hatched</b><small>Ill or paused · no catch-up</small></span></li>
    <li><NestedRings rings={[]} size={30} mark="unknown"/><span><b>Dotted</b><small>Awaiting Garmin data · not zero</small></span></li>
    <li><span class="lg-g">◇</span><span><b>Diamond by the date</b><small>A Garmin schedule session that day</small></span></li>
   </ul>
   <h3 class="lg-h">Sessions</h3>
   <ul class="legend">{#each LEGEND as [m, s, t]}<li><Glyph sport={s} mark={m} size={30} label={MARK_NAMES[m]}/><span><b>{MARK_NAMES[m]}</b><small>{t}</small></span></li>{/each}</ul>
   <p class="small">Activity colours: {#each ['running', 'cycling', 'strength', 'walking', 'hiit', 'swimming', 'yoga'] as s}<span class="sw" style:background={SPORTS[s as Sport].color}></span>{SPORTS[s as Sport].name} {/each}</p>
  {/if}
 </div>
{/if}
<style>
 .sr{position:absolute;width:1px;height:1px;overflow:hidden;clip:rect(0 0 0 0)}
 .lg-h{font:600 11px Inter;letter-spacing:.08em;text-transform:uppercase;color:var(--text3);margin:10px 0 4px}.lg-g{width:30px;text-align:center;color:#BDAAFF;font-size:18px}
 .day-head{display:flex;gap:14px;align-items:center;margin:4px 0 12px}.day-head b{display:block;font:600 18px Outfit}.day-head small{font-size:12px;color:var(--text2)}
 .dgoals{width:100%;border-collapse:collapse;font-size:13px;margin-bottom:10px}.dgoals th,.dgoals td{padding:7px 4px;border-bottom:1px solid var(--line);text-align:right}.dgoals thead th{font:600 10.5px Inter;color:var(--text3)}.dgoals tbody th{text-align:left;font-weight:600}.dgoals i{display:inline-block;width:8px;height:8px;border-radius:50%;background:var(--c);margin-right:7px}.dgoals td{font-family:Outfit}
 .big{font-size:16px}
 .list{margin-top:10px}
 .state-row{display:flex;align-items:center;gap:12px;margin:10px 0;padding:10px 12px;border-radius:14px;background:rgba(255,255,255,.05)}
 .state-row b{display:block}.state-row small{color:var(--text2);font-size:12px}
 .sport-grid{display:grid;grid-template-columns:repeat(4,1fr);gap:8px}
 .sport{display:flex;flex-direction:column;align-items:center;gap:6px;padding:10px 4px;border-radius:14px;border:1px solid var(--line);background:rgba(255,255,255,.03);color:var(--text2);font:600 12px Inter}
 .sport[aria-pressed="true"]{border-color:var(--ink);color:var(--text);background:rgba(255,255,255,.08)}
 .sport:disabled{opacity:.4}
 .center{text-align:center}
 .trace{list-style:none;padding:0;margin:0;display:grid;gap:12px}
 .trace li{display:grid;grid-template-columns:28px 1fr;gap:10px;align-items:start}
 .trace .step{width:26px;height:26px;border-radius:50%;display:grid;place-items:center;background:rgba(255,255,255,.08);font-size:13px}
 .trace b{display:block}.trace small{display:block;color:var(--text2);font-size:13px;margin-top:2px}
 .plain{padding-left:18px;color:var(--text2);line-height:1.5;font-size:14px}
 .legend{list-style:none;padding:0;margin:0;display:grid;gap:10px}
 .legend li{display:flex;gap:12px;align-items:center}
 .legend b{display:block}.legend small{color:var(--text2);font-size:12px}
 .steps{list-style:none;padding:0;margin:12px 0 0;display:grid;gap:10px}
 .steps li{display:grid;grid-template-columns:120px 44px 1fr;gap:10px;align-items:center;font-size:14px}
 .when{color:var(--text2);font-size:13px}
 .dots{display:flex;gap:3px}.dots i{width:9px;height:9px;border-radius:50%;background:#FF7DB8}
 .what small{display:block;color:var(--text3);font-size:12px}
 .sw{display:inline-block;width:10px;height:10px;border-radius:3px;margin:0 4px 0 8px}
</style>
