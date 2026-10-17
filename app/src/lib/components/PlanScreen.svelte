<script lang="ts">
 import type {PlanView} from '../generated/PlanView';
 import {masonry} from '../masonry';
 import {weekRange} from '../plan';
 import Card from './Card.svelte';
 import Icon from './Icon.svelte';
 import PlanGoals from './PlanGoals.svelte';
 import PlanHistory from './PlanHistory.svelte';
 import PlanSummary from './PlanSummary.svelte';
 let {view,back,edit}:{view:PlanView;back:()=>void;edit:()=>void}=$props();
</script>

<button class="back" onclick={back}><Icon name="back" size={20}/> <span>Today</span></button>
<header class="top"><div><div class="eyebrow">{weekRange(view.week.start,view.week.end)}</div><h1>This week</h1></div>{#if view.configured}<button class="edit" aria-label="Edit targets" onclick={edit}><Icon name="pencil" size={20}/></button>{/if}</header>
{#if view.configured}
 <div class="dashboard" {@attach masonry}>
  <Card extra="plan-week"><PlanSummary week={view.week} count={view.week.paused?view.week.reached:view.week.onTrack}/><PlanGoals items={view.week.items}/></Card>
  <Card title="Last 8 weeks" extra="plan-history"><PlanHistory weeks={view.history}/></Card>
 </div>
{:else}
 <Card title="Weekly plan" extra="plan-intro">
  <p>Pick a few targets for the week: strength sessions, Garmin's intensity minutes, Zone 2 minutes and nights asleep on time. Raring counts them from your Garmin data; nothing is sent anywhere.</p>
  <button class="primary" onclick={edit}>Choose targets <Icon name="arrow-right" size={18}/></button>
 </Card>
{/if}

<style>
 .edit{width:44px;height:44px;border-radius:50%;display:grid;place-items:center;background:rgba(255,255,255,.06);border:1px solid var(--line);color:var(--text)}
 p{color:var(--text2);line-height:1.5}
</style>
