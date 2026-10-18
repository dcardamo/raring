<script lang="ts">
 import {onMount, setContext} from 'svelte';
 import {store} from './lib/store.svelte';
 import './lib/actions';
 import XIcon from './components/XIcon.svelte';
 import Icon from '../lib/components/Icon.svelte';
 import Sheets from './components/Sheets.svelte';
 import Scenario from './components/Scenario.svelte';
 import Today from './screens/Today.svelte';
 import Plan from './screens/Plan.svelte';
 import Progress from './screens/Progress.svelte';
 import Explore from './screens/Explore.svelte';
 import Workout from './screens/Workout.svelte';
 import Details from './screens/Details.svelte';
 import Compare from './screens/Compare.svelte';
 import {toneColor} from '../lib/charts';
 let scroller: HTMLElement;
 let scenario = $state(false);
 // Trend charts re-read when the scenario changes the synthetic "today".
 setContext('trend-generation', () => store.state.day + store.state.clock + store.state.profile);
 onMount(() => { store.scroller = scroller; store.start(); });
 let route = $derived(store.route);
 let ctx = $derived(store.ctx);
 let key = $derived(route.split('/')[0]);
 let isRoot = $derived(store.roots.includes(route));
 let glow = $derived(route === 'today' ? (ctx.lead.kind === 'sleep' && ['evening', 'winddown', 'late'].includes(ctx.phase) ? '#5C7CFF' : ctx.readiness ? toneColor(ctx.readiness.value >= 75 ? 'good' : ctx.readiness.value >= 50 ? 'fair' : 'poor') : '#5C9BFF') : route === 'progress' ? '#A48BFF' : route === 'plan' ? '#36C9C6' : null);
 const TABS = [{route: 'today', label: 'Today', icon: 'today'}, {route: 'plan', label: 'Plan', icon: 'plan'}, {route: 'progress', label: 'Progress', icon: 'progress'}, {route: 'explore', label: 'Explore', icon: 'explore'}] as const;
 let tabs = $derived(TABS.filter(t => store.roots.includes(t.route)));
 let textScale = $derived(Number(new URLSearchParams(location.search).get('text') || 1));
</script>

<div class="phone xp" class:quiet={store.state.quiet} class:zoomed={textScale !== 1} style:--text-scale={textScale}>
 <div class="glow" style:--glow={glow ? `color-mix(in srgb, ${glow} 13%, transparent)` : 'transparent'}></div>
 <main class="scroll" bind:this={scroller} inert={!!store.sheet || scenario} data-route={route}>
  <div class="page" class:wide={key === 'compare'}>
   <div class="proto-strip" role="note"><span><b>Prototype</b> · synthetic data</span><button onclick={() => scenario = true} aria-haspopup="dialog">Scenario</button></div>
   {#if store.notice}<div class="case-note" role="status"><span>{store.notice}</span><button aria-label="Dismiss this message" onclick={() => store.notice = null}><Icon name="close" size={16}/></button></div>{/if}
   {#key route}
    {#if route === 'today'}<Today/>
    {:else if route === 'plan'}<Plan/>
    {:else if route === 'progress'}<Progress/>
    {:else if route === 'explore'}<Explore/>
    {:else if key === 'workout'}<Workout id={route.slice(8)}/>
    {:else if key === 'compare'}<Compare/>
    {:else}<Details {route}/>{/if}
   {/key}
  </div>
 </main>
 <nav class="tabs" aria-label="Main navigation" inert={!!store.sheet || scenario}>
  {#each tabs as t}<button class="tab" aria-current={store.root === t.route && (isRoot || true) ? 'page' : undefined} onclick={() => store.navigate(t.route)}><span class="tab-icon"><XIcon name={t.icon} size={23}/></span>{t.label}</button>{/each}
 </nav>
 <Sheets/>
 <div class="toast" class:on={store.toastOn} role="status" aria-live="polite">{store.toastText}</div>
 <Scenario bind:open={scenario}/>
</div>
