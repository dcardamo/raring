<script lang="ts">
 import type {Icon as GarminIcon} from '../generated/Icon';
 export type IconName = GarminIcon | 'pulse' | 'check' | 'lift' | 'gauge' | 'vo2' | 'back' | 'arrow-right' | 'arrow-left' | 'trend-up' | 'trend-down' | 'sparkle' | 'scale' | 'info' | 'close' | 'chevron-down' | 'pause' | 'pencil';
 let {name,size=20}:{name:IconName,size?:number}=$props();
</script>
<svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
 {#if name==='moon'}<path d="M20.5 13.5A8.5 8.5 0 1 1 10.5 3.5a6.6 6.6 0 0 0 10 10z"/>
 {:else if name==='pulse'}<path d="M3 12h4l2.5-5 4 10 2.5-5H21"/>
 {:else if name==='hrv'}<path d="M3 15c2 0 2-6 4-6s2 9 4 9 2-12 4-12 2 9 4 9h2"/>
 {:else if name==='flame'}<path d="M12 3c.8 3.2 5 5.3 5 10a5 5 0 0 1-10 0c0-2.3 1.1-3.7 2.2-4.8.2 1.8 1.2 2.8 1.2 2.8C10.4 8 11.4 5.6 12 3z"/>
 {:else if name==='check'}<path d="M5 12.5l4.5 4.5L19 7.5"/>
 {:else if name==='heart'}<path d="M12 20s-7.5-4.6-7.5-10.2A4.3 4.3 0 0 1 12 7.2a4.3 4.3 0 0 1 7.5 2.6C19.5 15.4 12 20 12 20z"/>
 {:else if name==='lungs'}<path d="M4 9h10.5a2.8 2.8 0 1 0-2.8-2.8M4 13h14a3 3 0 1 1-3 3M4 17h6"/>
 {:else if name==='drop'}<path d="M12 3.5s6 6.4 6 10.8a6 6 0 0 1-12 0c0-4.4 6-10.8 6-10.8z"/>
 {:else if name==='temp'}<path d="M14 14.6V5.5a2 2 0 0 0-4 0v9.1a4 4 0 1 0 4 0z"/><path d="M12 11v6"/>
 {:else if name==='run'}<circle cx="14.5" cy="4.5" r="1.8"/><path d="M8 21l3-6 3 2.5V21M6.5 12.5l2-3.5 4-1 3 4 3 .5M11 15l-1.5-3"/>
 {:else if name==='strength' || name==='lift'}<path d="M6 8v8M18 8v8M3.5 10v4M20.5 10v4M6 12h12"/>
 {:else if name==='bike'}<circle cx="6" cy="16" r="3.5"/><circle cx="18" cy="16" r="3.5"/><path d="M6 16l4-7h5l3 7M10 9l2.5 7M13.5 6h2"/>
 {:else if name==='bolt'}<path d="M13.5 2.5L5 13.5h6.5l-1 8 8.5-11h-6.5z"/>
 {:else if name==='gauge'}<path d="M4 16a8 8 0 1 1 16 0"/><path d="M12 16l4.5-4.5"/>
 {:else if name==='clock'}<circle cx="12" cy="12" r="8.5"/><path d="M12 7.5V12l3 2"/>
 {:else if name==='scale'}<rect x="4" y="4" width="16" height="16" rx="4"/><path d="M9 10a3 3 0 0 1 6 0M12 10l1.2-1.6"/>
 {:else if name==='vo2' || name==='trend-up'}<path d="M3 17l5-5 4 3 8-8"/><path d="M15 7h5v5"/>
 {:else if name==='trend-down'}<path d="M3 7l5 5 4-3 8 8"/><path d="M15 17h5v-5"/>
 {:else if name==='back'}<path d="M15 5l-7 7 7 7"/>
 {:else if name==='arrow-right'}<path d="M4 12h16m-6-6 6 6-6 6"/>
 {:else if name==='arrow-left'}<path d="M20 12H4m6-6-6 6 6 6"/>
 {:else if name==='sparkle'}<path d="m12 2 1.8 6.2L20 10l-6.2 1.8L12 18l-1.8-6.2L4 10l6.2-1.8L12 2zM19 17l.6 1.4L21 19l-1.4.6L19 21l-.6-1.4L17 19l1.4-.6L19 17z"/>
 {:else if name==='swim'}<path d="m6 10 4-3 3 2 4 1m-5-7a2 2 0 1 0 0 4 2 2 0 0 0 0-4ZM2 15c2 0 2-1 4-1s2 1 4 1 2-1 4-1 2 1 4 1 2-1 4-1M2 19c2 0 2-1 4-1s2 1 4 1 2-1 4-1 2 1 4 1 2-1 4-1"/>
 {:else if name==='walk'}<circle cx="13" cy="4" r="2"/><path d="m12 7-2 5 4 2 2 7M10 12l-4 9m5-13 5 2 2 3"/>
 {:else if name==='hike'}<circle cx="12" cy="4" r="2"/><path d="m11 7-2 6 4 2 2 6M9 13l-4 8m8-12 4 3m2-6v15M5 11l-2-3-2 3"/>
 {:else if name==='pause'}<path d="M9 6.5v11M15 6.5v11"/>
 {:else if name==='pencil'}<path d="M4 20h4L19 9l-4-4L4 16v4zM13.5 6.5l4 4"/>
 {:else if name==='info'}<circle cx="12" cy="12" r="8.5"/><path d="M12 11v5M12 8h.01"/>
 {:else if name==='close'}<path d="M6 6l12 12M18 6L6 18"/>
 {:else if name==='chevron-down'}<path d="M7 10l5 5 5-5"/>
 {:else if name==='other'}<circle cx="12" cy="12" r="8"/><path d="M12 8v4m0 4h.01"/>
 {/if}
</svg>
