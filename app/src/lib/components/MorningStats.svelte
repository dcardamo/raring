<script lang="ts">
 import Gauge from './Gauge.svelte';
 import type {TodayView} from '../generated/TodayView';
 import type {NightReading} from '../generated/NightReading';
 import {toneColor} from '../charts';
 import {duration} from '../format';
 // Four compact stats at the top of Today: last night's sleep score,
 // readiness, Sleep HR and HRV. Each opens its detail. Each rating is a word
 // in its tone colour: Garmin's level, or in or outside the usual range.
 let {today, waiting, navigate}: {today: TodayView, waiting: boolean, navigate: (route: string) => void} = $props();
 let night = $derived(today.focus.lastNight);
 // A night that hasn't synced shows nothing of an earlier one.
 let unsynced = $derived(waiting || (night?.asleepMin == null && today.focus.sleepMissing));
 let asleep = $derived(unsynced ? null : night?.asleepMin ?? null);
 let score = $derived(unsynced ? null : today.sleep);
 let readiness = $derived(waiting ? null : today.readiness);
 // From the evening on tonight leads; last night's vitals stay off Today.
 let nightPhase = $derived(['evening', 'windDown', 'lateNight'].includes(today.focus.phase));
 let vitals = $derived(nightPhase ? null : today.nightVitals);
 let showVitals = $derived(!nightPhase);
 // The score's level and the time asleep; a duration without a score says so.
 let sleepLine = $derived(unsynced ? 'Not synced yet' : score ? [score.label, asleep != null ? duration(asleep) : null].filter(Boolean).join(' · ') : asleep != null ? 'No score' : 'Not recorded');
 let sleepLabel = $derived(unsynced ? 'not synced yet' : [score ? `score ${score.value}, ${score.label}` : 'no score', asleep != null ? `${duration(asleep)} asleep` : null].filter(Boolean).join(', '));
 const tint = (tone: string | null | undefined) => tone && tone !== 'neutral' ? toneColor(tone) : null;
 const fmt = (v: number) => String(Math.round(v));
 function usual(r: NightReading | null | undefined): {text: string, mark: string, tone: string | null} {
  if (waiting) return {text: 'Waiting', mark: '', tone: null};
  if (!r || r.value == null) return {text: 'Not recorded', mark: '', tone: null};
  const tone = tint(r.tone);
  switch (r.state) {
   case 'above': return {text: `${fmt(Math.abs(r.delta ?? 0))} above usual`, mark: '▲', tone};
   case 'below': return {text: `${fmt(Math.abs(r.delta ?? 0))} below usual`, mark: '▼', tone};
   case 'within': return {text: 'Within usual', mark: '', tone};
   case 'calibrating': return {text: 'Garmin calibrating', mark: '', tone: null};
   default: return {text: 'Usual not set yet', mark: '', tone: null};
  }
 }
 let hr = $derived(usual(vitals?.sleepHr));
 let hrv = $derived(usual(vitals?.hrv));
 const range = (r: NightReading | null | undefined) => r?.usual ? `, usual ${fmt(r.usual[0])}–${fmt(r.usual[1])} ${r.unit}` : '';
</script>
<section class="morning" aria-label="Last night and readiness" data-chart="morning stats">
 <div class="stats" class:two={!showVitals}>
  <button class="stat sleep-fact" onclick={() => navigate('sleep/last')} aria-label={`Last night’s sleep, ${sleepLabel}`}>
   <span class="dial">{#if score}<Gauge value={score.value} max={score.max} size={64} stroke={6} color={toneColor(score.tone)}/>{:else}<b class="n big-n">{asleep != null ? duration(asleep) : '—'}</b>{/if}</span>
   <b class="label">Sleep</b><small style:color={tint(score?.tone)}>{sleepLine}</small>
  </button>
  <button class="stat ready-fact" onclick={() => navigate('readiness')} aria-label={`Readiness ${readiness ? `${readiness.value}, ${readiness.label}` : waiting ? 'waiting for last night' : 'unavailable'}`}>
   <span class="dial">{#if readiness}<Gauge value={readiness.value} max={readiness.max} size={64} stroke={6} color={toneColor(readiness.tone)}/>{:else}<b class="n big-n">—</b>{/if}</span>
   <b class="label">Readiness</b><small style:color={tint(readiness?.tone)}>{readiness?.label ?? (today.missingMetricReasons.readiness && !waiting ? 'Not on this watch' : 'Waiting')}</small>
  </button>
  {#if showVitals}
   <button class="stat hr-fact" onclick={() => navigate('metric/sleepHr')} aria-label={`Sleep HR ${vitals?.sleepHr.value != null ? fmt(vitals.sleepHr.value) + ' bpm' : 'not recorded'}, ${hr.text}${range(vitals?.sleepHr)}`}>
    <span class="dial"><b class="n big-n">{vitals?.sleepHr.value != null ? fmt(vitals.sleepHr.value) : '—'}</b>{#if vitals?.sleepHr.value != null}<span class="unit">bpm</span>{/if}</span>
    <b class="label">Sleep HR</b><small style:color={hr.tone}>{#if hr.mark}<i aria-hidden="true">{hr.mark + '\u00a0'}</i>{/if}{hr.text}</small>
   </button>
   <button class="stat hrv-fact" onclick={() => navigate('metric/hrv')} aria-label={`HRV ${vitals?.hrv.value != null ? fmt(vitals.hrv.value) + ' ms' : 'not recorded'}, ${hrv.text}${range(vitals?.hrv)}`}>
    <span class="dial"><b class="n big-n">{vitals?.hrv.value != null ? fmt(vitals.hrv.value) : '—'}</b>{#if vitals?.hrv.value != null}<span class="unit">ms</span>{/if}</span>
    <b class="label">HRV</b><small style:color={hrv.tone}>{#if hrv.mark}<i aria-hidden="true">{hrv.mark + '\u00a0'}</i>{/if}{hrv.text}</small>
   </button>
  {/if}
 </div>
</section>
<style>
 .morning{margin:8px 0 20px;container-type:inline-size}
 .stats{display:grid;grid-template-columns:repeat(4,minmax(0,1fr));gap:8px}
 .stats.two{grid-template-columns:repeat(2,minmax(0,1fr))}
 .stat{display:grid;justify-items:center;align-content:start;gap:2px;min-width:0;min-height:44px;padding:10px 4px 9px;border-radius:var(--radius-control);background:var(--card);border:1px solid var(--line);color:var(--text);text-align:center}
 .dial{display:grid;place-items:center;align-content:center;color:var(--text);width:4.6em;height:4.6em;max-width:100%;margin-bottom:4px}
 .big-n{font-size:22px;font-weight:500;line-height:1;letter-spacing:-.035em}.unit{font:500 12px Inter;color:var(--text2);margin-top:3px}
 .label{font:600 12px Inter;color:var(--text)}
 small{display:block;font:500 12px/1.25 Inter;color:var(--text2);overflow-wrap:anywhere}
 small i{font-style:normal}
 /* In em, so larger system text wraps the row into two by two before it overflows. */
 @container(max-width:21em){.stats{grid-template-columns:repeat(2,minmax(0,1fr))}}
</style>
