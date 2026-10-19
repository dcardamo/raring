<script lang="ts">
 // Navigation glyphs beside the metric icon set.
 let {name, size = 22}: {name: 'today' | 'plan' | 'progress' | 'training' | 'health' | 'settings' | 'adapt' | 'search' | 'lock' | 'offline' | 'calendar' | 'flag' | 'diamond', size?: number} = $props();
</script>
<svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
 {#if name === 'training'}<path d="M13 2L4 14h7l-1 8 10-12h-8l1-8z"/>
 {:else if name === 'health'}<path d="M12 21S3 15 3 8.5a4.5 4.5 0 0 1 9-1 4.5 4.5 0 0 1 9 1C21 15 12 21 12 21z"/>
 {:else if name === 'today'}<circle cx="12" cy="12" r="4"/><path d="M12 2.5v2M12 19.5v2M4.6 4.6l1.4 1.4M18 18l1.4 1.4M2.5 12h2M19.5 12h2M4.6 19.4L6 18M18 6l1.4-1.4"/>
 {:else if name === 'plan' || name === 'calendar'}<rect x="3.5" y="5" width="17" height="15.5" rx="3"/><path d="M3.5 10h17M8 3v4M16 3v4"/>
 {:else if name === 'progress'}<path d="M5 19v-7M10 19V6M15 19v-9M20 19V9M3 21h18"/>
 {:else if name === 'settings'}<circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.7 1.7 0 0 0 .3 1.8l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1.7 1.7 0 0 0-1.8-.3 1.7 1.7 0 0 0-1 1.5V21a2 2 0 1 1-4 0v-.1a1.7 1.7 0 0 0-1.1-1.5 1.7 1.7 0 0 0-1.8.3l-.1.1a2 2 0 1 1-2.8-2.8l.1-.1a1.7 1.7 0 0 0 .3-1.8 1.7 1.7 0 0 0-1.5-1H3a2 2 0 1 1 0-4h.1a1.7 1.7 0 0 0 1.5-1.1 1.7 1.7 0 0 0-.3-1.8l-.1-.1a2 2 0 1 1 2.8-2.8l.1.1a1.7 1.7 0 0 0 1.8.3H9a1.7 1.7 0 0 0 1-1.5V3a2 2 0 1 1 4 0v.1a1.7 1.7 0 0 0 1 1.5 1.7 1.7 0 0 0 1.8-.3l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1a1.7 1.7 0 0 0-.3 1.8V9a1.7 1.7 0 0 0 1.5 1H21a2 2 0 1 1 0 4h-.1a1.7 1.7 0 0 0-1.5 1z"/>
 {:else if name === 'adapt'}<path d="M4 6h9M17 6h3M4 12h3M11 12h9M4 18h11M19 18h1"/><circle cx="15" cy="6" r="2"/><circle cx="9" cy="12" r="2"/><circle cx="17" cy="18" r="2"/>
 {:else if name === 'search'}<circle cx="11" cy="11" r="7"/><path d="M21 21l-4.3-4.3"/>
 {:else if name === 'lock'}<rect x="4.5" y="11" width="15" height="10" rx="2"/><path d="M8 11V7a4 4 0 0 1 8 0v4"/>
 {:else if name === 'offline'}<path d="M2 2l20 20M8.5 16.5a5 5 0 0 1 7 0M5 12.6a10 10 0 0 1 5-2.6M19 12.6a10 10 0 0 0-2-1.5M12 20h.01"/>
 {:else if name === 'flag'}<path d="M5 21V4M5 4h11l-2 4 2 4H5"/>
 {:else if name === 'diamond'}<rect x="6.5" y="6.5" width="11" height="11" rx="2" transform="rotate(45 12 12)"/>{/if}
</svg>
