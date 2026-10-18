<script lang="ts" module>
 export type Mark = 'recorded' | 'plan' | 'garmin' | 'rest' | 'paused' | 'unknown' | 'self';
 export const MARK_NAMES: Record<Mark, string> = {recorded: 'Recorded', plan: 'Your plan', garmin: 'Garmin schedule', rest: 'Rest', paused: 'Paused', unknown: 'Awaiting evidence', self: 'Self-reported'};
</script>
<script lang="ts">
 import SportIcon from './SportIcon.svelte';
 import Icon from '../../lib/components/Icon.svelte';
 import {SPORTS, type Sport} from '../lib/sports';
 // One visual language for state: filled = recorded, outlined = your plan,
 // diamond = Garmin schedule, moon = rest, hatched = paused, dotted = unknown.
 let {sport = 'other', mark = 'recorded', size = 28, label, named = false}: {sport?: Sport, mark?: Mark, size?: number, label?: string, named?: boolean} = $props();
 let color = $derived(mark === 'rest' || mark === 'unknown' ? '#8E98AC' : SPORTS[sport].color);
 let name = $derived(label ?? (mark === 'rest' ? 'Rest day' : mark === 'paused' ? 'Paused' : mark === 'unknown' ? 'Awaiting evidence' : `${MARK_NAMES[mark]}: ${SPORTS[sport].name}`));
 const id = $props.id();
</script>
<span class="glyph m-{mark}" style:--c={color} style:width={size + 'px'} style:height={size + 'px'} role={named ? 'img' : undefined} aria-label={named ? name : undefined} aria-hidden={named ? undefined : 'true'}>
 <svg viewBox="0 0 32 32" width={size} height={size} aria-hidden="true">
  <defs><pattern id="h{id}" width="4" height="4" patternUnits="userSpaceOnUse" patternTransform="rotate(45)"><rect width="1.6" height="4" fill={color} opacity=".55"/></pattern></defs>
  {#if mark === 'garmin'}<rect x="6" y="6" width="20" height="20" rx="3.5" transform="rotate(45 16 16)" fill="rgba(11,15,26,.6)" stroke={color} stroke-width="2"/>
  {:else if mark === 'recorded'}<circle cx="16" cy="16" r="15" fill={color}/>
  {:else if mark === 'plan'}<circle cx="16" cy="16" r="14" fill="rgba(11,15,26,.5)" stroke={color} stroke-width="2"/>
  {:else if mark === 'self'}<circle cx="16" cy="16" r="14" fill="rgba(11,15,26,.5)" stroke={color} stroke-width="2" stroke-dasharray="6 3"/>
  {:else if mark === 'paused'}<circle cx="16" cy="16" r="14" fill="url(#h{id})" stroke={color} stroke-width="1.5" stroke-opacity=".7"/>
  {:else if mark === 'unknown'}<circle cx="16" cy="16" r="14" fill="none" stroke={color} stroke-width="2" stroke-dasharray="1.5 3.5" stroke-linecap="round"/>
  {:else}<circle cx="16" cy="16" r="14" fill="rgba(142,152,172,.14)"/>{/if}
 </svg>
 {#if mark === 'rest'}<span class="gi" style:color="#B4BCCB"><Icon name="moon" size={Math.round(size * .5)}/></span>
 {:else if mark !== 'unknown'}<span class="gi" style:color={mark === 'recorded' ? '#0B0F1A' : color}><SportIcon {sport} size={Math.round(size * (mark === 'garmin' ? .44 : .54))}/></span>{/if}
</span>
<style>
 .glyph{position:relative;display:inline-grid;place-items:center;flex:none}
 .glyph svg{position:absolute;inset:0}
 .gi{position:relative;display:grid;place-items:center}
</style>
