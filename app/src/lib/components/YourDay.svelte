<script lang="ts">
 import Icon from './Icon.svelte';
 import HelpButton from './HelpButton.svelte';
 import type {DayTimeline} from '../generated/DayTimeline';
 import {stressColors,stressNames} from '../charts';
 import {duration,minuteClock as hm} from '../format';
 import {declutter} from '../declutter';
 // Sleep, Body Battery, stress, heart rate and recorded events share one
 // midnight-to-midnight axis, so a moment lines up across rows. Values come
 // from the core; nothing is drawn after the watch's last upload.
 // `compact` follows tonight's plan from the evening on: sleep, Body Battery and recordings only.
 let {day,navigate,compact=false}:{day:DayTimeline,navigate:(route:string)=>void,compact?:boolean}=$props();
 const pct=(m:number)=>Math.max(0,Math.min(100,m/1440*100));
 const STAGE:Record<string,string>={deep:'#2F6FE0',light:'#5C9BFF',rem:'#A9C8FF',awake:'#E4E8F0'};
 // The core already blanks slots past the upload or now; this only places the last reading.
 let limit=$derived(Math.min(day.uploadMin??day.nowMin,day.nowMin));
 let now=$derived(day.nowMin<1440?day.nowMin:null);
 const path=(slots:(number|null)[],step:number,lo:number,hi:number)=>{let d='',pen=false;slots.forEach((v,i)=>{const m=i*step;if(v==null||m>limit){pen=false;return}d+=`${pen?'L':'M'}${m},${(50-(Math.min(hi,Math.max(lo,v))-lo)/(hi-lo)*48).toFixed(1)}`;pen=true});return d};
 let bb=$derived(path(day.bodyBattery,3,0,100));
 let bbNow=$derived(day.bodyBatteryNow);
 // Heart rate is scaled to the day's own readings (padded to tens), so no peak or low is clipped.
 let hrRange=$derived.by(()=>{const v=day.heartRate.filter((x,i):x is number=>x!=null&&i*2<=limit);return v.length?[Math.min(...v),Math.max(...v)]:null});
 let hr=$derived(hrRange?path(day.heartRate,2,Math.floor((hrRange[0]-5)/10)*10,Math.ceil((hrRange[1]+5)/10)*10):'');
 // Fifteen-minute means of Garmin's three-minute stress readings; empty stays empty.
 let bars=$derived.by(()=>{const out:{m:number,v:number}[]=[];for(let i=0;i<day.stress.length;i+=5){const vals=day.stress.slice(i,i+5).filter((v):v is number=>v!=null);if(vals.length&&i*3<=limit)out.push({m:i*3,v:vals.reduce((a,b)=>a+b,0)/vals.length})}return out});
 const level=(v:number)=>v<=25?'rest':v<=50?'low':v<=75?'medium':'high';
 let naps=$derived(day.events.filter(e=>e.kind==='nap'&&e.startMin!=null&&e.endMin!=null));
 let placed=$derived(day.events.filter(e=>e.startMin!=null));
 let napChange=$derived(naps.findLast(n=>n.bodyBatteryChange!=null)?.bodyBatteryChange??null);
</script>
<section class="your-day" class:compact aria-label="Your day on one timeline">
 <div class="yd-h"><h2>Your day <HelpButton topic="today"/></h2></div>
 {#if day.clockChange}<p class="clock" role="note">Clocks changed by an hour today, so Body Battery, stress and heart rate are in their details.</p>{/if}
 <div data-chart="your-day" {@attach declutter}>
 <button class="row" onclick={()=>navigate('sleep/last')} aria-label={day.sleep.length?`Sleep until ${hm(Math.max(...day.sleep.map(s=>s.endMin)))}, ${duration(day.asleepMin)} asleep`:'Sleep: no recorded night yet'}>
  <span class="lab"><Icon name="moon" size={15}/><b>Sleep</b></span>
  <span class="track sleep">{#each day.sleep as s}<i style:left={pct(s.startMin)+'%'} style:width={pct(s.endMin)-pct(s.startMin)+'%'} style:background={STAGE[s.stage]}></i>{/each}{#if !day.sleep.length}<em class="none">No recorded night yet</em>{:else if now!=null}<u data-mark="now" style:left={pct(now)+'%'}></u>{/if}</span>
  <span class="val n">{day.asleepMin!=null?duration(day.asleepMin):'—'}</span>
 </button>
 <button class="row" onclick={()=>navigate('metric/bodyBattery')} aria-label={`Body Battery ${bbNow??'not recorded'}${day.uploadMin!=null?' at '+hm(day.uploadMin):''}${naps.filter(n=>n.bodyBatteryChange!=null).map(n=>`, nap ${n.bodyBatteryChange!>=0?'+':''}${n.bodyBatteryChange}`).join('')}`}>
  <span class="lab"><Icon name="bolt" size={15}/><b>Body Battery</b></span>
  <span class="track chart">{#if bb}<svg viewBox="0 0 1440 50" preserveAspectRatio="none" aria-hidden="true">{#each naps as n}<rect x={n.startMin} y="0" width={n.endMin!-n.startMin!} height="50" fill="rgba(169,200,255,.22)"/>{/each}<path d={bb} fill="none" stroke="#3DDC97" stroke-width="2" vector-effect="non-scaling-stroke"/></svg>{#if now!=null}<u data-mark="now" style:left={pct(now)+'%'}></u>{/if}{:else}<em class="none">{day.clockChange?'Clock change today · open the detail':bbNow!=null?'Latest level only':'Awaiting upload'}</em>{/if}</span>
  <span class="val n">{bbNow??'—'}{#if napChange!=null}<small>nap {napChange>=0?'+':''}{napChange}</small>{/if}</span>
 </button>
 {#if !compact}
 <button class="row" onclick={()=>navigate('metric/stress')} aria-label={`Stress, average ${day.stressAverage??'not recorded'}`}>
  <span class="lab"><Icon name="pulse" size={15}/><b>Stress</b></span>
  <span class="track chart">{#if bars.length}<svg viewBox="0 0 1440 50" preserveAspectRatio="none" aria-hidden="true">{#each bars as b}<rect x={b.m+1} y={50-Math.max(2,b.v/100*48)} width="13" height={Math.max(2,b.v/100*48)} fill={stressColors[level(b.v)]}/>{/each}</svg>{#if now!=null}<u data-mark="now" style:left={pct(now)+'%'}></u>{/if}{:else}<em class="none">{day.clockChange?'Clock change today · open the detail':'Awaiting upload'}</em>{/if}</span>
  <span class="val n">{day.stressAverage??'—'}<small>avg</small></span>
 </button>
 <button class="row" onclick={()=>navigate('metric/heartRate')} aria-label={day.heartRate.length?`Heart rate through the day, resting ${day.restingHr??'not recorded'} bpm${hrRange?`, readings ${hrRange[0]}–${hrRange[1]}`:''}`:'Heart rate: see the heart-rate detail'}>
  <span class="lab"><Icon name="heart" size={15}/><b>Heart rate</b></span>
  <span class="track chart">{#if hr}<svg viewBox="0 0 1440 50" preserveAspectRatio="none" aria-hidden="true"><path d={hr} fill="none" stroke="#FF6B8A" stroke-width="1.6" vector-effect="non-scaling-stroke"/></svg>{#if now!=null}<u data-mark="now" style:left={pct(now)+'%'}></u>{/if}{:else}<em class="none">{day.clockChange?'Clock change today · open the detail':'Awaiting upload'}</em>{/if}</span>
  <span class="val n">{day.restingHr??'—'}<small>rest bpm</small></span>
 </button>
 {/if}
 <div class="row ev" role="group" aria-label="Recorded activities and naps">
  <span class="lab"><Icon name="run" size={15}/><b>Recorded</b></span>
  <span class="track events">{#each placed as e}<button class="evb" class:nap={e.kind==='nap'} style:left={pct(e.startMin!)+'%'} style:width={`max(26px, ${pct(e.endMin??e.startMin!)-pct(e.startMin!)}%)`} aria-label={`${e.name} ${hm(e.startMin!)}${e.durationMin!=null?`, ${Math.round(e.durationMin)} min`:''}${e.bodyBatteryChange!=null?`, Body Battery ${e.bodyBatteryChange>=0?'+':''}${e.bodyBatteryChange}`:''}`} onclick={()=>navigate(e.id?'workout/'+e.id:'sleep')}><Icon name={e.icon} size={12}/></button>{/each}{#if !placed.length}<em class="none">Nothing recorded yet</em>{:else if now!=null}<u data-mark="now" style:left={pct(now)+'%'}></u>{/if}</span>
  <span class="val"></span>
 </div>
 <div class="row axis" aria-hidden="true"><span></span><span class="track ticks">{#each [0,6,12,18,24] as h}<i data-thin style:left={h/24*100+'%'}>{String(h).padStart(2,'0')}</i>{/each}{#if now!=null}<b style:left={pct(now)+'%'} style:translate={pct(now)<12?'0':pct(now)>88?'-100%':'-50%'}>Now {hm(now)}</b>{/if}</span><span></span></div>
 {#if !compact && bars.length}<p class="key">{#each ['rest','low','medium','high'] as l}<span><i style:background={stressColors[l as 'rest']}></i>{stressNames[l as 'rest']}</span>{/each}</p>{/if}
 </div>
</section>
<style>
 .your-day.compact{margin-top:0}
 .your-day{margin-top:12px;border:1px solid var(--line);border-radius:22px;background:var(--card);padding:14px 14px 10px}
 .yd-h{display:flex;justify-content:space-between;align-items:baseline;margin-bottom:6px}.yd-h h2{display:flex;align-items:center;gap:8px;font:600 11px Inter;letter-spacing:.13em;text-transform:uppercase;color:var(--text2)}
 .row{display:grid;grid-template-columns:96px minmax(0,1fr) 56px;gap:8px;align-items:center;width:100%;padding:6px 0;background:none;border:0;color:var(--text);text-align:left;min-height:44px}
 .lab{display:flex;align-items:center;gap:6px;min-width:0}.lab :global(svg){color:var(--text2);flex:none}.lab b{font:600 12.5px/1.15 Inter;overflow-wrap:anywhere}
 .track{position:relative;display:block;height:34px;border-radius:6px;background:rgba(255,255,255,.035)}
 .track.sleep{height:16px}.track.sleep i{position:absolute;top:0;bottom:0}
 .track.chart svg{position:absolute;inset:0;width:100%;height:100%}
 .track.events{background:none;border-bottom:1px solid var(--line);border-radius:0}
 .evb::after{content:'';position:absolute;inset:-6px -11px}
 .evb{position:absolute;top:1px;height:32px;border:0;border-radius:6px;display:grid;place-items:center;color:#0B0F1A;background:#3DDC97;padding:0;min-width:22px}.evb.nap{background:#A9C8FF}
 .track u{position:absolute;top:-4px;bottom:-4px;width:2px;margin-left:-1px;background:var(--ink);border-radius:1px;opacity:.85;pointer-events:none}
 .none{position:absolute;inset:0;display:grid;place-items:center;font:500 12px Inter;text-align:center;line-height:1.1;color:var(--text2);font-style:normal}
 .val{text-align:right;font-size:17px;font-weight:600}.val small{display:block;font:500 12px Inter;color:var(--text2)}
 .axis{padding:0;min-height:22px}.ticks{height:1.7em;background:none;font:500 11px Inter}.ticks i{position:absolute;top:2px;transform:translateX(-50%);color:var(--text2);font-style:normal}.ticks i:first-child{transform:none}.ticks i:nth-child(5){transform:translateX(-100%)}.ticks b{position:absolute;top:0;font:600 12px/1.2 Inter;color:var(--text);background:#141a27;padding:1px 4px;border-radius:4px;white-space:nowrap}
 .clock{margin:0 0 6px;font-size:12px;color:var(--text2)}
 .key{display:flex;flex-wrap:wrap;gap:4px 12px;margin:6px 0 0;font-size:12px;color:var(--text2)}.key span{display:inline-flex;gap:5px;align-items:center}.key i{width:8px;height:8px;border-radius:2px}
 @container(max-width:300px){.row{grid-template-columns:72px minmax(0,1fr) 52px}.lab b{font-size:12px}}
</style>
