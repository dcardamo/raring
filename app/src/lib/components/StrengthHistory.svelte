<script lang="ts">
 import Icon from './Icon.svelte';
 import BodyMap from './BodyMap.svelte';
 import Sparkline from './Sparkline.svelte';
 import ActivityRow from './ActivityRow.svelte';
 import {colors} from '../charts';
 import {dateLabel} from '../format';
 import {amountColor} from '../bodymap';
 import {muscleNames,weight,unitLabel} from '../strength';
 import type {StrengthView} from '../generated/StrengthView';
 import type {LiftSeries} from '../generated/LiftSeries';
 import type {TrendRange} from '../generated/TrendRange';
 import type {MuscleRange} from '../generated/MuscleRange';
 import RangePicker from './RangePicker.svelte';
 // `liftSeries` reads one exercise over a range and `muscles` sets per muscle over a range, both from the core.
 let {view,navigate,liftSeries,muscles}:{view:StrengthView,navigate:(route:string)=>void,liftSeries:(key:string,range:TrendRange)=>Promise<LiftSeries|null>,muscles:(range:TrendRange)=>Promise<MuscleRange|null>}=$props();
 let muscleRange=$state<TrendRange>('days30'),muscleView=$state<MuscleRange|null>(null),muscleRequest=0;
 $effect(()=>{const r=muscleRange,id=++muscleRequest;void muscles(r).then(v=>{if(id===muscleRequest)muscleView=v}).catch(()=>{if(id===muscleRequest)muscleView=null})});
 let picked=$state<string|null>(null);
 // Long lists start short; the full 12 weeks are a tap away.
 const SESSIONS=6,WORKOUTS=8;
 let allSessions=$state(false),allWorkouts=$state(false);
 function pick(key:string){picked=key;allSessions=false}
 let exercise=$derived(view.exercises.find(e=>e.key===picked)??view.exercises[0]??null);
 let latest=$derived(exercise?.sessions[0]??null);
 let unit=$derived(unitLabel(view.unit));
 // Max weight or volume over the chosen range, from the core; one choice for every lift.
 let metric=$state<'max'|'volume'>('max'),range=$state<TrendRange>('days90'),series=$state<LiftSeries|null>(null);
 let request=0;
 $effect(()=>{const key=exercise?.key,r=range,id=++request;if(!key){series=null;return}void liftSeries(key,r).then(v=>{if(id===request)series=v}).catch(()=>{if(id===request)series=null})});
 let shown=$derived(series && series.key===exercise?.key?series:null);
 let excluded=$derived(shown?shown.points.reduce((t,p)=>t+p.excluded,0):0);
 // Oldest first for the chart; sessions without a weighted set are gaps.
 let points=$derived(shown?shown.points.map(p=>{const s=exercise?.sessions.find(x=>x.activityId===p.activityId);return {date:p.date,v:metric==='max'?p.max:p.volume,reps:metric==='max'?s?.top?.reps??null:null,record:metric==='max'&&!!s?.record}}):[]);
 let average=$derived(Object.fromEntries(view.average4w.map(m=>[m.muscle,m.sets])) as Record<string,number>);
 let scale=$derived(Math.max(14,...view.thisWeek.map(m=>m.sets),...view.average4w.map(m=>m.sets)));
 const short=(date:string)=>{const d=new Date(`${date}T12:00:00`);return d.toLocaleDateString('en-GB',{month:'short'})};
 // Phone and wider layouts get their own chart width so axis text keeps its size.
 const H=190,pl=34,pb=22;
 const build=(W:number)=>{
  const ws=points.map(p=>p.v),present=ws.filter((w):w is number=>w!=null);
  if(present.length<2)return null;
  const min=Math.min(...present),max=Math.max(...present);
  // Two or three round gridlines around the range.
  const step=[1,2.5,5,10,20,25,50,100,250,500,1000,2500,5000].find(s=>(max-min)/s<=2.2)??10000;
  const lo=Math.floor(min/step)*step-(min%step===0?step/2:0),hi=Math.ceil(max/step)*step+(max%step===0?step/2:0);
  const X=(i:number)=>pl+6+i*(W-pl-14)/Math.max(points.length-1,1),Y=(v:number)=>8+(hi-v)/(hi-lo||1)*(H-pb-14);
  const ticks:number[]=[];for(let t=Math.ceil(lo/step)*step;t<=hi;t+=step)ticks.push(t);
  const months:{x:number,label:string}[]=[];
  points.forEach((p,i)=>{const m=short(p.date);if((!months.length||months.at(-1)!.label!==m) && (!months.length||X(i)-months.at(-1)!.x>=44))months.push({x:X(i),label:m})});
  const pts=points.map((p,i)=>p.v!=null?{x:X(i),y:Y(p.v),reps:p.reps,record:p.record,last:i===points.length-1}:null);
  let d='';pts.forEach((p,i)=>{if(p)d+=`${d&&pts[i-1]?'L':'M'}${p.x.toFixed(1)},${p.y.toFixed(1)}`});
  return {W,ticks,Y,months,pts,d,base:H-pb};
 };
 let charts=$derived([build(330),build(440)]);
</script>
<div class="strength-history">
 <div class="col exercises-col">
  <section class="card"><h3>Exercises</h3>
   {#each view.exercises as e}
    {@const top=e.sessions[0]?.top}
    <button class="exl" class:on={e.key===exercise?.key} onclick={()=>pick(e.key)}><span><b>{e.name}</b><span class="n">{top?`${weight(top.weight)} × ${e.sessions[0].reps.map(r=>r??'—').join(' · ')}`:e.sessions[0]?.reps.map(r=>r??'—').join(' · ')}</span></span><Sparkline values={[...e.sessions].reverse().map(s=>s.top?.weight??null)} w={64} h={26} color={colors.secondary} fill={false}/></button>
   {/each}
  </section>
 </div>
 <div class="col">
  <section class="card"><h3><span>Muscles</span></h3><RangePicker value={muscleRange} label="Muscle range" onchange={r=>muscleRange=r}/>
   {#if muscleView}<BodyMap figure={muscleView.figure} muscles={muscleView.muscles} max={Math.max(8,Math.round(muscleView.days/2))}/>{#if muscleView.gaps.length}<p class="dim">Few or no sets since {dateLabel(muscleView.from)}: {muscleView.gaps.map(m=>muscleNames[m]).join(', ')}</p>{/if}{/if}</section>
  {#if exercise}
   <div class="exchips" role="group" aria-label="Exercise">{#each view.exercises as e}<button class:on={e.key===exercise.key} aria-pressed={e.key===exercise.key} onclick={()=>pick(e.key)}>{e.name}</button>{/each}</div>
   <section class="card"><h3><span>{exercise.name}</span>{#if latest?.record}<span class="pill ok"><Icon name="star" size={12}/>New record</span>{/if}</h3>
    {#if latest?.top}<div class="tr-top"><div class="big n">{weight(latest.top.weight)}<span class="unit">{unit} × {latest.top.reps}</span></div><span class="dim">Heaviest set each session</span></div>{:else}<p class="dim">No weighted sets yet.</p>{/if}
    <div class="lift-ctl"><span class="segc" role="group" aria-label="Lift measure"><button class:on={metric==='max'} aria-pressed={metric==='max'} onclick={()=>metric='max'}>Max weight</button><button class:on={metric==='volume'} aria-pressed={metric==='volume'} onclick={()=>metric='volume'}>Volume</button></span><RangePicker value={range} label="Lift range" onchange={r=>range=r}/></div>
    <p class="dim">{metric==='max'?`Heaviest set each session, ${unit}`:`Volume each session: weight × reps over weighted sets, ${unit}`}{excluded?` · ${excluded} bodyweight, unweighted or flagged set${excluded===1?'':'s'} left out`:''}. No estimated one-rep max.</p>
    {#if shown && charts.every(c=>!c)}<p class="dim">{shown.points.length?'Fewer than two sessions with weighted sets in this range.':'No sessions in this range.'}</p>{/if}
    {#each charts as chart}{#if chart}<svg class="trend" class:narrow={chart.W<400} class:wide={chart.W>=400} viewBox={`0 0 ${chart.W} ${H}`} role="img" aria-label={`${metric==='max'?'Heaviest set':'Volume'} each session, ${points.length} sessions: ${points.map(p=>`${dateLabel(p.date)} ${p.v??'none'}`).join(', ')}`}>
     {#each chart.ticks as t}<line x1={pl} x2={chart.W} y1={chart.Y(t)} y2={chart.Y(t)} stroke="rgba(255,255,255,.06)"/><text x="0" y={chart.Y(t)+4} class="axis n">{t}</text>{/each}
     <path d={chart.d} fill="none" stroke={colors.ink} stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"/>
     {#each chart.pts as p}{#if p}{#if p.last}<circle cx={p.x} cy={p.y} r="6.5" fill={p.record?colors.good:colors.ink} stroke="#0B0F1A" stroke-width="3"/>{:else}<circle cx={p.x} cy={p.y} r="3" fill="#0B0F1A" stroke={colors.ink} stroke-width="2"/>{/if}{#if p.reps!=null && latest?.top && p.reps!==latest.top.reps}<text x={p.x} y={p.y+18} text-anchor="middle" class="axis n">×{p.reps}</text>{/if}{/if}{/each}
     {#each chart.months as m}<text x={m.x} y={H-3} text-anchor="middle" class="axis">{m.label}</text>{/each}
    </svg>{/if}{/each}
   </section>
   {@const weighted=exercise.bests.some(b=>b.weight!=null)}
   <section class="card"><h3>{weighted?'Best at each rep count':'Most reps in a set'} <small class="h-note">in available history</small></h3><div class="recs">{#if weighted}{#each exercise.bests as b}{@const fresh=b.date!=null && b.date===latest?.date && latest.record}<div class="rec" class:new={fresh}><span>{b.reps} rep{b.reps>1?'s':''}</span><b class="n">{weight(b.weight)}</b><small>{b.date?(b.date>=view.weekStart?dateLabel(b.date).split(' ')[0]:short(b.date)):''}</small></div>{/each}{/if}{#if exercise.repsBest}<div class="rec" aria-label={`Most reps in one set: ${exercise.repsBest.reps}, ${dateLabel(exercise.repsBest.date)}`}><span>Most reps</span><b class="n">{exercise.repsBest.reps}</b><small>{exercise.repsBest.date>=view.weekStart?dateLabel(exercise.repsBest.date).split(' ')[0]:short(exercise.repsBest.date)}</small></div>{/if}</div><p class="note">{weighted?`${unit} · at least that many reps`:'Reps · one set, no weight'}{#if weighted && exercise.repsBest} · bodyweight sets count reps, not weight{/if}</p></section>
   <section class="card"><h3><span>Sessions</span><span class="h-note">{exercise.sessions.length} in 12 weeks</span></h3>{#each allSessions?exercise.sessions:exercise.sessions.slice(0,SESSIONS) as s}<button class="ses" onclick={()=>navigate('workout/'+s.activityId)}><span>{dateLabel(s.date)}</span><b class="n">{weight(s.top?.weight??null)}<small>{s.top?.weight!=null?unit:''}</small></b><span class="n rr">{s.reps.map(r=>r??'—').join(' · ')}</span><span class="pr">{#if s.record}<Icon name="star" size={13}/>{/if}</span></button>{/each}{#if exercise.sessions.length>SESSIONS}<button class="more" aria-expanded={allSessions} onclick={()=>allSessions=!allSessions}>{allSessions?'Show fewer':`Show all ${exercise.sessions.length} sessions`}</button>{/if}</section>
  {/if}
 </div>
 <div class="col">
  <section class="card"><h3><span>Sets per muscle</span><span class="h-note">Since {dateLabel(view.weekStart)}</span></h3>
   <div class="mus">{#each view.thisWeek as m}<div class="mu"><span>{muscleNames[m.muscle]}</span><div class="mu-b"><i style:width={m.sets/scale*100+'%'} style:background={amountColor(m.sets,12)}></i><u style:left={(average[m.muscle]??0)/scale*100+'%'}></u></div><b class="n">{m.sets}</b></div>{/each}</div>
   <p class="note"><i class="tick"></i>Weekly average, previous 4 weeks</p></section>
  {#if view.workouts.length}<section class="card"><h3><span>Strength workouts</span><span class="h-note">{view.workouts.length} in 12 weeks</span></h3>{#each allWorkouts?view.workouts:view.workouts.slice(0,WORKOUTS) as w}<div class="workout-item"><ActivityRow activity={w.activity} open={()=>navigate('workout/'+w.activity.id)}/>{#if !w.setsRecorded}<p class="nosets">No sets recorded</p>{/if}</div>{/each}{#if view.workouts.length>WORKOUTS}<button class="more" aria-expanded={allWorkouts} onclick={()=>allWorkouts=!allWorkouts}>{allWorkouts?'Show fewer':`Show all ${view.workouts.length} workouts`}</button>{/if}</section>{/if}
 </div>
</div>
<style>
 .lift-ctl{display:flex;flex-wrap:wrap;gap:8px;align-items:center;justify-content:space-between;margin:8px 0 4px}
 .strength-history{display:grid;gap:0 16px}
 .col{min-width:0}
 .exercises-col{display:none}
 .dim,.note{font-size:12px;color:var(--text2)}
 .note{margin-top:12px;display:flex;align-items:center;gap:8px}
 .h-note{font:500 12px Inter;letter-spacing:0;text-transform:none;color:var(--text2)}
 .segc{display:inline-flex;background:rgba(255,255,255,.05);border-radius:10px;padding:2px;letter-spacing:0;text-transform:none}
 .segc button{border:0;background:none;color:var(--text2);font:600 12px Inter;padding:6px 10px;border-radius:8px;min-height:32px}
 .segc button.on{background:rgba(255,255,255,.12);color:var(--text)}
 .exchips{display:flex;gap:8px;overflow-x:auto;scrollbar-width:none;margin:12px 0 0}
 .exchips::-webkit-scrollbar{display:none}
 .exchips button{flex:none;padding:10px 14px;border-radius:999px;background:rgba(255,255,255,.05);border:1px solid var(--line);font:600 13px Inter;color:var(--text2);min-height:44px}
 .exchips button.on{background:var(--ink);border-color:var(--ink);color:#0B0F1A}
 .tr-top{display:flex;align-items:baseline;justify-content:space-between;gap:10px;margin:-4px 0 12px}
 .trend{display:block;width:100%;height:auto}
 .trend.wide{display:none}
 .axis{fill:var(--text3);font:500 11px Inter}
 .recs{display:grid;grid-template-columns:repeat(auto-fit,minmax(52px,1fr));gap:6px}
 .rec{background:rgba(255,255,255,.03);border-radius:13px;padding:10px 2px;text-align:center}
 .rec span{font:500 11px Inter;color:var(--text3)}
 .rec b{display:block;font:600 20px Outfit;margin:2px 0}
 .rec small{display:block;font:500 11px Inter;color:var(--text3);min-height:14px}
 .rec.new b,.rec.new small{color:var(--good)}
 .ses{display:grid;grid-template-columns:1fr 74px 86px 18px;align-items:center;width:100%;padding:12px 0;border:0;border-top:1px solid var(--line);background:none;text-align:left;font-size:13px;color:var(--text2);min-height:44px}
 .ses:first-of-type{border-top:0;padding-top:0}
 .ses b{font:600 16px Outfit;color:var(--text);text-align:right}
 .ses b small{font:500 11px Inter;color:var(--text3);margin-left:2px}
 .ses .rr{text-align:right;font:500 14px Outfit}
 .ses .pr{display:flex;justify-content:flex-end;color:var(--good)}
 .more{display:block;width:100%;min-height:44px;margin-top:8px;border:0;border-top:1px solid var(--line);background:none;color:var(--text);font:600 13px Inter;padding:12px 0 0}
 .nosets{margin:-4px 0 0 56px;font-size:12px;color:var(--text2)}
 .mus{display:grid;gap:10px}
 .mu{display:grid;grid-template-columns:84px 1fr 26px;align-items:center;gap:10px;font-size:13px;color:var(--text2)}
 .mu b{font:600 14px Outfit;color:var(--text);text-align:right}
 .mu-b{position:relative;height:8px;border-radius:4px;background:rgba(255,255,255,.06)}
 .mu-b i{position:absolute;left:0;top:0;bottom:0;border-radius:4px}
 .mu-b u{position:absolute;top:-4px;bottom:-4px;width:2px;border-radius:1px;background:var(--ink);opacity:.7}
 .tick{display:inline-block;width:2px;height:12px;border-radius:1px;background:var(--ink);opacity:.7}
 .exl{display:flex;align-items:center;gap:10px;width:calc(100% + 16px);padding:10px 8px;margin:0 -8px;border:0;border-radius:12px;background:none;text-align:left;min-height:44px}
 .exl.on{background:rgba(255,255,255,.07)}
 .exl>span{flex:1;min-width:0}
 .exl b{display:block;font:600 13px Inter;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
 .exl span span{font-size:12px;color:var(--text3)}
 @media(min-width:660px){
  .strength-history{grid-template-columns:minmax(0,1fr) minmax(0,1fr)}
  .col>:global(.card:first-child){margin-top:0}
  .trend.wide{display:block}
  .trend.narrow{display:none}
  .exchips{flex-wrap:wrap;overflow:visible}
 }
 @media(min-width:1000px){
  .strength-history{grid-template-columns:minmax(0,.8fr) minmax(0,1.2fr) minmax(0,1fr)}
  .exercises-col{display:block}
  .exchips{display:none}
 }
</style>
