<script lang="ts">
 import Icon from './Icon.svelte';
 import BodyMap from './BodyMap.svelte';
 import Sparkline from './Sparkline.svelte';
 import ActivityRow from './ActivityRow.svelte';
 import {colors} from '../charts';
 import {dateLabel,shortMonth} from '../format';
 import {amountColor} from '../bodymap';
 import {muscleNames,weight,unitLabel} from '../strength';
 import type {StrengthView} from '../generated/StrengthView';
 import type {LiftSeries} from '../generated/LiftSeries';
 import type {TrendRange} from '../generated/TrendRange';
import type {FigureChoice} from '../generated/FigureChoice';
 import type {MuscleRange} from '../generated/MuscleRange';
 import RangePicker from './RangePicker.svelte';
 import {declutter} from '../declutter';
 // `liftSeries` reads one exercise over a range and `muscles` sets per muscle over a range, both from the core.
 // The chosen exercise, measure and ranges are bound from the app so Back keeps them.
 let {view,navigate,liftSeries,muscles,figureChoice,setFigure,picked=$bindable(null),metric=$bindable('max'),range=$bindable('days90'),muscleRange=$bindable('days30')}:{view:StrengthView,navigate:(route:string)=>void,liftSeries:(key:string,range:TrendRange)=>Promise<LiftSeries|null>,muscles:(range:TrendRange)=>Promise<MuscleRange|null>,figureChoice:FigureChoice,setFigure:(figure:FigureChoice)=>void,
  picked?:string|null,metric?:'max'|'volume',range?:TrendRange,muscleRange?:TrendRange}=$props();
const figures:[FigureChoice,string][]=[['profile','Profile'],['female','Female'],['male','Male']];
 let muscleView=$state<MuscleRange|null>(null),muscleRequest=0;
 $effect(()=>{const r=muscleRange,id=++muscleRequest;void muscles(r).then(v=>{if(id===muscleRequest)muscleView=v}).catch(()=>{if(id===muscleRequest)muscleView=null})});
 // Long lists start short; the full 12 weeks are a tap away.
 const SESSIONS=6,WORKOUTS=8;
 let allSessions=$state(false),allWorkouts=$state(false);
 function pick(key:string){picked=key;allSessions=false}
 let exercise=$derived(view.exercises.find(e=>e.key===picked)??view.exercises[0]??null);
 let latest=$derived(exercise?.sessions[0]??null);
 let unit=$derived(unitLabel(view.unit));
 // Max weight or volume over the chosen range, from the core; one choice for every lift.
 let series=$state<LiftSeries|null>(null);
 let request=0;
 $effect(()=>{const key=exercise?.key,r=range,id=++request;if(!key){series=null;return}void liftSeries(key,r).then(v=>{if(id===request)series=v}).catch(()=>{if(id===request)series=null})});
 let shown=$derived(series && series.key===exercise?.key?series:null);
 let excluded=$derived(shown?shown.points.reduce((t,p)=>t+p.excluded,0):0);
 // Without a weighted set in its history an exercise charts its most reps instead.
 let weighted=$derived(!!exercise?.bests.some(b=>b.weight!=null));
 let measure=$derived(weighted?metric:'reps');
 // Oldest first for the chart; sessions without a value are gaps.
 let points=$derived(shown?shown.points.map(p=>{const s=exercise?.sessions.find(x=>x.activityId===p.activityId);return {date:p.date,v:measure==='max'?p.max:measure==='volume'?p.volume:p.mostReps,reps:measure==='max'?s?.top?.reps??null:null,record:measure==='max'&&!!s?.record}}):[]);
 let average=$derived(Object.fromEntries(view.average4w.map(m=>[m.muscle,m.sets])) as Record<string,number>);
 let scale=$derived(Math.max(14,...view.thisWeek.map(m=>m.sets),...view.average4w.map(m=>m.sets)));
 // Drawn at the measured width so axis text keeps its size.
 const H=190,pl=34,pb=22,DAY=864e5,DENSE=40;
 // A best's date: "20 Aug" this year, "Aug 2024" before.
 const when=(date:string)=>date.slice(0,4)===view.date.slice(0,4)?dateLabel(date).replace(/^\S+ /,''):`${shortMonth(date)} ${date.slice(0,4)}`;
 const day=(date:string)=>Date.parse(`${date.slice(0,10)}T12:00:00Z`);
 // Date ticks: weekly for a month or so, month starts beyond; the year goes on the first label and on January.
 const dateTicks=(from:string,to:string)=>{
  const t0=day(from),t1=day(to),out:{t:number,label:string,year:string,jan:boolean}[]=[];
  if(t1-t0<=45*DAY){for(let t=t1;t>=t0;t-=7*DAY)out.unshift({t,label:dateLabel(new Date(t).toISOString().slice(0,10)).replace(/^\S+ /,''),year:'',jan:false});return out}
  const d=new Date(t0);d.setUTCDate(1);d.setUTCMonth(d.getUTCMonth()+1);
  for(;d.getTime()<=t1;d.setUTCMonth(d.getUTCMonth()+1)){const iso=d.toISOString().slice(0,10);out.push({t:d.getTime(),label:shortMonth(iso),year:String(d.getUTCFullYear()),jan:d.getUTCMonth()===0})}
  return out;
 };
 const build=(W:number)=>{
  const ws=points.map(p=>p.v),present=ws.filter((w):w is number=>w!=null);
  if(present.length<2||!shown)return null;
  const min=Math.min(...present),max=Math.max(...present);
  // Two or three round gridlines around the range.
  const steps=measure==='reps'?[1,2,5,10,20,50,100]:[1,2.5,5,10,20,25,50,100,250,500,1000,2500,5000];
  const step=steps.find(s=>(max-min)/s<=2.2)??10000;
  const lo=Math.max(0,Math.floor(min/step)*step-(min%step===0?step/2:0)),hi=Math.ceil(max/step)*step+(max%step===0?step/2:0);
  const t0=day(shown.from),t1=day(shown.to);
  const X=(date:string)=>pl+6+(day(date)-t0)/Math.max(t1-t0,DAY)*(W-pl-14),Y=(v:number)=>8+(hi-v)/(hi-lo||1)*(H-pb-14);
  const ticks:number[]=[];for(let t=Math.ceil(lo/step)*step;t<=hi;t+=step)ticks.push(t);
  // Thin date labels so none collide; the first keeps its year.
  const dates:{x:number,label:string}[]=[];
  for(const tick of dateTicks(shown.from,shown.to)){const x=pl+6+(tick.t-t0)/Math.max(t1-t0,DAY)*(W-pl-14),label=tick.year&&(!dates.length||tick.jan)?`${tick.label} ${tick.year}`:tick.label,half=label.length*3.3;if(x-half<pl-4||x+half>W)continue;if(dates.length&&x-half<dates.at(-1)!.x+dates.at(-1)!.label.length*3.3+8)continue;dates.push({x,label})}
  const dense=points.length>DENSE;
  const pts=points.map((p,i)=>p.v!=null?{x:X(p.date),y:Y(p.v),reps:dense?null:p.reps,record:p.record,last:i===points.length-1,dot:!dense||p.record||i===points.length-1}:null);
  let d='';pts.forEach((p,i)=>{if(p)d+=`${d&&pts[i-1]?'L':'M'}${p.x.toFixed(1)},${p.y.toFixed(1)}`});
  return {W,ticks,Y,dates,pts,d,base:H-pb};
 };
 let chartWidth=$state(330);
 let chart=$derived(build(Math.max(240,Math.round(chartWidth))));
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
  <section class="card"><h3><span>Muscles</span></h3><RangePicker value={muscleRange} label="Muscle range" onchange={r=>muscleRange=r}/><div class="fig-row"><span class="fig-label">Figure</span><span class="segc" role="group" aria-label="Body map figure">{#each figures as [value,label]}<button class:on={figureChoice===value} aria-pressed={figureChoice===value} onclick={()=>setFigure(value)}>{label}</button>{/each}</span></div>
   {#if muscleView}<BodyMap figure={muscleView.figure} muscles={muscleView.muscles} max={Math.max(1,...muscleView.muscles.map(m=>m.sets))}/>{#if muscleView.gaps.length}<p class="dim">Few or no sets since {dateLabel(muscleView.from)}: {muscleView.gaps.map(m=>muscleNames[m]).join(', ')}</p>{/if}
    <ul class="mlist" aria-label="Sets per muscle group">{#each [...muscleView.muscles].sort((a,b)=>a.sets-b.sets) as m}<li class:gap={muscleView.gaps.includes(m.muscle)}><span>{muscleNames[m.muscle]}</span><b class="n">{Math.round(m.sets)}</b><small>sets</small></li>{/each}</ul>{/if}</section>
  {#if exercise}
   <div class="exchips" role="group" aria-label="Exercise">{#each view.exercises as e}<button class:on={e.key===exercise.key} aria-pressed={e.key===exercise.key} onclick={()=>pick(e.key)}>{e.name}</button>{/each}</div>
   <section class="card"><h3><span>{exercise.name}</span>{#if measure==='max' && latest?.record}<span class="pill ok"><Icon name="star" size={12}/>New record</span>{/if}</h3>
    {#if measure==='max' && latest?.top}<div class="tr-top"><div class="big n">{weight(latest.top.weight)}<span class="unit">{unit} × {latest.top.reps}</span></div><span class="dim">Latest heaviest set</span></div>{/if}
    <div class="lift-ctl">{#if weighted}<span class="segc" role="group" aria-label="Lift measure"><button class:on={metric==='max'} aria-pressed={metric==='max'} onclick={()=>metric='max'}>Max weight</button><button class:on={metric==='volume'} aria-pressed={metric==='volume'} onclick={()=>metric='volume'}>Volume</button></span>{/if}<RangePicker value={range} label="Lift range" onchange={r=>range=r}/></div>
    <p class="dim">{measure==='max'?`Heaviest set each session, ${unit}`:measure==='volume'?`Volume each session: weight × reps over weighted sets, ${unit}`:'Most reps in one set each session'}{weighted && excluded?` · ${excluded} bodyweight, unweighted or flagged set${excluded===1?'':'s'} left out`:''}</p>
    {#if shown && !chart}<p class="dim">{shown.points.length?`Fewer than two sessions with ${weighted?'weighted sets':'reps'} in this range.`:'No sessions in this range.'}</p>{/if}
    <div bind:clientWidth={chartWidth} data-chart="lift" {@attach declutter}>{#if chart}<svg class="trend" viewBox={`0 0 ${chart.W} ${H}`} role="img" aria-label={`${measure==='max'?'Heaviest set':measure==='volume'?'Volume':'Most reps'} each session, ${points.length} sessions: ${points.map(p=>`${dateLabel(p.date)} ${p.v??'none'}`).join(', ')}`}>
     {#each chart.ticks as t}<line x1={pl} x2={chart.W} y1={chart.Y(t)} y2={chart.Y(t)} stroke="rgba(255,255,255,.06)"/><text data-thin="2" x="0" y={chart.Y(t)+4} class="axis n">{t}</text>{/each}
     <path d={chart.d} fill="none" stroke={colors.ink} stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"/>
     {#each chart.pts as p}{#if p?.dot}{#if p.last}<circle cx={p.x} cy={p.y} r="6.5" fill={p.record?colors.good:colors.ink} stroke="var(--bg)" stroke-width="3"/>{:else}<circle cx={p.x} cy={p.y} r="3" fill="var(--bg)" stroke={colors.ink} stroke-width="2"/>{/if}{#if p.reps!=null && latest?.top && p.reps!==latest.top.reps}<text data-thin x={p.x} y={p.y+18} text-anchor="middle" class="axis n">×{p.reps}</text>{/if}{/if}{/each}
     {#each chart.dates as m}<text data-thin="1" x={m.x} y={H-4} text-anchor="middle" class="axis">{m.label}</text>{/each}
    </svg>{/if}</div>
   </section>
   <section class="card"><h3>{weighted?'Best at each rep count':'Most reps in a set'} <small class="h-note">in available history</small></h3><div class="recs">{#if weighted}{#each exercise.bests as b}{@const fresh=b.date!=null && b.date===latest?.date && latest.record}<div class="rec" class:new={fresh}><span>{b.reps} rep{b.reps>1?'s':''}</span><b class="n">{weight(b.weight)}</b><small>{b.date?when(b.date):''}</small></div>{/each}{/if}{#if exercise.repsBest}<div class="rec" aria-label={`Most reps in one set: ${exercise.repsBest.reps}, ${dateLabel(exercise.repsBest.date)}`}><span>Most reps</span><b class="n">{exercise.repsBest.reps}</b><small>{when(exercise.repsBest.date)}</small></div>{/if}</div><p class="note">{weighted?`${unit} · at least that many reps`:'Reps · one set, no weight'}{#if weighted && exercise.repsBest} · bodyweight sets count reps, not weight{/if}</p></section>
   <section class="card"><h3><span>Sessions</span><span class="h-note">{exercise.sessions.length} in 12 weeks</span></h3>{#each allSessions?exercise.sessions:exercise.sessions.slice(0,SESSIONS) as s}<button class="ses" onclick={()=>navigate('workout/'+s.activityId)}><span>{dateLabel(s.date)}</span><b class="n">{weight(s.top?.weight??null)}<small>{s.top?.weight!=null?unit:''}</small></b><span class="n rr">{s.reps.map(r=>r??'—').join(' · ')}</span><span class="pr">{#if s.record}<Icon name="star" size={13}/>{/if}</span></button>{/each}{#if exercise.sessions.length>SESSIONS}<button class="more" aria-expanded={allSessions} onclick={()=>allSessions=!allSessions}>{allSessions?'Show fewer':`Show all ${exercise.sessions.length} sessions`}</button>{/if}</section>
  {/if}
 </div>
 <div class="col">
  <section class="card"><h3><span>This week vs your 4-week average</span><span class="h-note">Since {dateLabel(view.weekStart)}</span></h3>
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
 h3{flex-wrap:wrap;gap:6px}
 .segc{display:inline-flex;flex-wrap:wrap;background:rgba(255,255,255,.05);border-radius:10px;padding:2px;letter-spacing:0;text-transform:none}
 .segc button{border:0;background:none;color:var(--text2);font:600 12px Inter;padding:6px 10px;border-radius:8px;min-height:32px}
 .segc button.on{background:rgba(255,255,255,.12);color:var(--text)}
 .fig-row{display:flex;flex-wrap:wrap;align-items:center;justify-content:space-between;gap:10px;margin:10px 0 4px}
 .fig-label{font:600 12px Inter;color:var(--text3)}
 .exchips{display:flex;gap:8px;overflow-x:auto;scrollbar-width:none;margin:12px 0 0}
 .exchips::-webkit-scrollbar{display:none}
 .exchips button{flex:none;padding:10px 14px;border-radius:999px;background:rgba(255,255,255,.05);border:1px solid var(--line);font:600 13px Inter;color:var(--text2);min-height:44px}
 .exchips button.on{background:var(--ink);border-color:var(--ink);color:var(--bg)}
 .tr-top{display:flex;align-items:baseline;justify-content:space-between;gap:10px;margin:-4px 0 12px}
 .trend{display:block;width:100%;height:auto}
 .axis{fill:var(--text3);font:500 11px Inter}
 .recs{display:grid;grid-template-columns:repeat(auto-fit,minmax(52px,1fr));gap:6px}
 .rec{background:rgba(255,255,255,.03);border-radius:13px;padding:10px 2px;text-align:center}
 .rec span{font:500 11px Inter;color:var(--text3)}
 .rec b{display:block;font:600 20px Outfit;margin:2px 0}
 .rec small{display:block;font:500 11px Inter;color:var(--text3);min-height:14px}
 .rec.new b,.rec.new small{color:var(--good)}
 .ses{display:grid;grid-template-columns:1fr minmax(74px,auto) minmax(86px,auto) 18px;gap:0 6px;align-items:center;width:100%;padding:12px 0;border:0;border-top:1px solid var(--line);background:none;text-align:left;font-size:13px;color:var(--text2);min-height:44px}
 .ses:first-of-type{border-top:0;padding-top:0}
 .ses b{font:600 16px Outfit;color:var(--text);text-align:right}
 .ses b small{font:500 11px Inter;color:var(--text3);margin-left:2px}
 .ses .rr{text-align:right;font:500 14px Outfit}
 .ses .pr{display:flex;justify-content:flex-end;color:var(--good)}
 .more{display:block;width:100%;min-height:44px;margin-top:8px;border:0;border-top:1px solid var(--line);background:none;color:var(--text);font:600 13px Inter;padding:12px 0 0}
 .nosets{margin:2px 0 0 56px;font-size:12px;color:var(--text2)}
 .mus{display:grid;gap:10px}
 .mlist{list-style:none;margin:10px 0 0;padding:0;display:grid;grid-template-columns:repeat(auto-fill,minmax(130px,1fr));gap:4px 16px;font-size:13px}
 .mlist li{display:grid;grid-template-columns:1fr auto 30px;gap:6px;align-items:baseline;color:var(--text2)}.mlist b{color:var(--text)}.mlist small{font-size:11px}.mlist li.gap span{color:var(--fair);font-weight:600}
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
  .exchips{flex-wrap:wrap;overflow:visible}
 }
 @media(min-width:1000px){
  .strength-history{grid-template-columns:minmax(0,.8fr) minmax(0,1.2fr) minmax(0,1fr)}
  .exercises-col{display:block}
  .exchips{display:none}
 }
</style>
