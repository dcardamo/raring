<script lang="ts">
 import Icon from './Icon.svelte';
 import BodyMap from './BodyMap.svelte';
 import Sparkline from './Sparkline.svelte';
 import ActivityRow from './ActivityRow.svelte';
 import {colors} from '../charts';
 import {dateLabel} from '../format';
 import {amountColor,textOn} from '../bodymap';
 import {muscleNames,weight,unitLabel} from '../strength';
 import type {StrengthView} from '../generated/StrengthView';
 let {view,navigate}:{view:StrengthView,navigate:(route:string)=>void}=$props();
 let span=$state<'week'|'4w'>('week');
 let picked=$state<string|null>(null);
 // Long lists start short; the full 12 weeks are a tap away.
 const SESSIONS=6,WORKOUTS=8;
 let allSessions=$state(false),allWorkouts=$state(false);
 function pick(key:string){picked=key;allSessions=false}
 let exercise=$derived(view.exercises.find(e=>e.key===picked)??view.exercises[0]??null);
 let latest=$derived(exercise?.sessions[0]??null);
 let unit=$derived(unitLabel(view.unit));
 // Oldest first for the chart; sessions without a weighted set are gaps.
 let points=$derived(exercise?[...exercise.sessions].reverse():[]);
 let average=$derived(Object.fromEntries(view.average4w.map(m=>[m.muscle,m.sets])) as Record<string,number>);
 let scale=$derived(Math.max(14,...view.thisWeek.map(m=>m.sets),...view.average4w.map(m=>m.sets)));
 const short=(date:string)=>{const d=new Date(`${date}T12:00:00`);return d.toLocaleDateString('en-GB',{month:'short'})};
 // Phone and wider layouts get their own chart width so axis text keeps its size.
 const H=190,pl=34,pb=22;
 const build=(W:number)=>{
  const ws=points.map(p=>p.top?.weight??null),present=ws.filter((w):w is number=>w!=null);
  if(present.length<2)return null;
  const min=Math.min(...present),max=Math.max(...present);
  // Two or three round gridlines around the range.
  const step=[1,2.5,5,10,20,25,50,100].find(s=>(max-min)/s<=2.2)??200;
  const lo=Math.floor(min/step)*step-(min%step===0?step/2:0),hi=Math.ceil(max/step)*step+(max%step===0?step/2:0);
  const X=(i:number)=>pl+6+i*(W-pl-14)/Math.max(points.length-1,1),Y=(v:number)=>8+(hi-v)/(hi-lo||1)*(H-pb-14);
  const ticks:number[]=[];for(let t=Math.ceil(lo/step)*step;t<=hi;t+=step)ticks.push(t);
  const months:{x:number,label:string}[]=[];
  points.forEach((p,i)=>{const m=short(p.date);if((!months.length||months.at(-1)!.label!==m) && (!months.length||X(i)-months.at(-1)!.x>=44))months.push({x:X(i),label:m})});
  const pts=points.map((p,i)=>p.top?.weight!=null?{x:X(i),y:Y(p.top.weight),reps:p.top.reps,record:p.record,last:i===points.length-1}:null);
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
  <section class="card"><h3><span>{span==='week'?'This week':'Last 4 weeks'}</span><span class="segc" role="group" aria-label="Body map period"><button class:on={span==='week'} aria-pressed={span==='week'} onclick={()=>span='week'}>Week</button><button class:on={span==='4w'} aria-pressed={span==='4w'} onclick={()=>span='4w'}>4 weeks</button></span></h3>
   <BodyMap figure={view.figure} muscles={span==='week'?view.thisWeek:view.last4w} max={span==='week'?12:40}/></section>
  {#if exercise}
   <div class="exchips" role="group" aria-label="Exercise">{#each view.exercises as e}<button class:on={e.key===exercise.key} aria-pressed={e.key===exercise.key} onclick={()=>pick(e.key)}>{e.name}</button>{/each}</div>
   <section class="card"><h3><span>{exercise.name}</span>{#if latest?.record}<span class="pill ok"><Icon name="star" size={12}/>New record</span>{/if}</h3>
    {#if latest?.top}<div class="tr-top"><div class="big n">{weight(latest.top.weight)}<span class="unit">{unit} × {latest.top.reps}</span></div><span class="dim">Heaviest set each session</span></div>{:else}<p class="dim">No weighted sets yet.</p>{/if}
    {#each charts as chart}{#if chart}<svg class="trend" class:narrow={chart.W<400} class:wide={chart.W>=400} viewBox={`0 0 ${chart.W} ${H}`} role="img" aria-label={`Heaviest set each session, ${points.length} sessions`}>
     {#each chart.ticks as t}<line x1={pl} x2={chart.W} y1={chart.Y(t)} y2={chart.Y(t)} stroke="rgba(255,255,255,.06)"/><text x="0" y={chart.Y(t)+4} class="axis n">{t}</text>{/each}
     <path d={chart.d} fill="none" stroke={colors.ink} stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"/>
     {#each chart.pts as p}{#if p}{#if p.last}<circle cx={p.x} cy={p.y} r="6.5" fill={p.record?colors.good:colors.ink} stroke="#0B0F1A" stroke-width="3"/>{:else}<circle cx={p.x} cy={p.y} r="3" fill="#0B0F1A" stroke={colors.ink} stroke-width="2"/>{/if}{#if latest?.top && p.reps!==latest.top.reps}<text x={p.x} y={p.y+18} text-anchor="middle" class="axis n">×{p.reps}</text>{/if}{/if}{/each}
     {#each chart.months as m}<text x={m.x} y={H-3} text-anchor="middle" class="axis">{m.label}</text>{/each}
    </svg>{/if}{/each}
   </section>
   <section class="card"><h3>Best at each rep count</h3><div class="recs">{#each exercise.bests as b}{@const fresh=b.date!=null && b.date===latest?.date && latest.record}<div class="rec" class:new={fresh}><span>{b.reps} rep{b.reps>1?'s':''}</span><b class="n">{weight(b.weight)}</b><small>{b.date?(b.date>=view.weekStart?dateLabel(b.date).split(' ')[0]:short(b.date)):''}</small></div>{/each}</div><p class="note">{unit} · at least that many reps</p></section>
   <section class="card"><h3><span>Sessions</span><span class="h-note">{exercise.sessions.length} in 12 weeks</span></h3>{#each allSessions?exercise.sessions:exercise.sessions.slice(0,SESSIONS) as s}<button class="ses" onclick={()=>navigate('workout/'+s.activityId)}><span>{dateLabel(s.date)}</span><b class="n">{weight(s.top?.weight??null)}<small>{s.top?.weight!=null?unit:''}</small></b><span class="n rr">{s.reps.map(r=>r??'—').join(' · ')}</span><span class="pr">{#if s.record}<Icon name="star" size={13}/>{/if}</span></button>{/each}{#if exercise.sessions.length>SESSIONS}<button class="more" aria-expanded={allSessions} onclick={()=>allSessions=!allSessions}>{allSessions?'Show fewer':`Show all ${exercise.sessions.length} sessions`}</button>{/if}</section>
  {/if}
 </div>
 <div class="col">
  <section class="card"><h3><span>Sets per muscle</span><span class="h-note">Since {dateLabel(view.weekStart)}</span></h3>
   <div class="mus">{#each view.thisWeek as m}<div class="mu"><span>{muscleNames[m.muscle]}</span><div class="mu-b"><i style:width={m.sets/scale*100+'%'} style:background={amountColor(m.sets,12)}></i><u style:left={(average[m.muscle]??0)/scale*100+'%'}></u></div><b class="n">{m.sets}</b></div>{/each}</div>
   <p class="note"><i class="tick"></i>Weekly average, previous 4 weeks</p></section>
  <section class="card heat-card"><h3>Weekly sets · 12 weeks</h3>
   <div class="heat" role="table" aria-label="Sets per muscle each week">
    <span></span>{#each view.weeks as w,i}<small>{i===view.weeks.length-1?'Now':i%4===0?dateLabel(w.weekStart).split(' ').slice(1).join(' '):''}</small>{/each}
    {#each view.thisWeek as row,j}<span>{muscleNames[row.muscle]}</span>{#each view.weeks as w}{@const v=w.muscles[j].sets}{@const fill=amountColor(v,12)}<i class="n" style:background={fill} style:color={textOn(fill)}>{v}</i>{/each}{/each}
   </div></section>
  {#if view.workouts.length}<section class="card"><h3><span>Strength workouts</span><span class="h-note">{view.workouts.length} in 12 weeks</span></h3>{#each allWorkouts?view.workouts:view.workouts.slice(0,WORKOUTS) as w}<div class="workout-item"><ActivityRow activity={w.activity} open={()=>navigate('workout/'+w.activity.id)}/>{#if !w.setsRecorded}<p class="nosets">No sets recorded</p>{/if}</div>{/each}{#if view.workouts.length>WORKOUTS}<button class="more" aria-expanded={allWorkouts} onclick={()=>allWorkouts=!allWorkouts}>{allWorkouts?'Show fewer':`Show all ${view.workouts.length} workouts`}</button>{/if}</section>{/if}
 </div>
</div>
<style>
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
 .recs{display:grid;grid-template-columns:repeat(5,minmax(0,1fr));gap:6px}
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
 .heat-card{display:none}
 .heat{display:grid;grid-template-columns:76px repeat(12,minmax(0,1fr));gap:3px;align-items:center;font-size:12px;color:var(--text2)}
 .heat small{font:500 9px Inter;color:var(--text3);white-space:nowrap;overflow:visible}
 .heat i{height:20px;border-radius:5px;display:grid;place-items:center;font:600 10px Outfit;font-style:normal}
 .exl{display:flex;align-items:center;gap:10px;width:calc(100% + 16px);padding:10px 8px;margin:0 -8px;border:0;border-radius:12px;background:none;text-align:left;min-height:44px}
 .exl.on{background:rgba(255,255,255,.07)}
 .exl>span{flex:1;min-width:0}
 .exl b{display:block;font:600 13px Inter;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
 .exl span span{font-size:12px;color:var(--text3)}
 @media(min-width:660px){
  .strength-history{grid-template-columns:minmax(0,1fr) minmax(0,1fr)}
  .col>:global(.card:first-child){margin-top:0}
  .heat-card{display:block}
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
