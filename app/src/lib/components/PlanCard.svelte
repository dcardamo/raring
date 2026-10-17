<script lang="ts">
 import type {PlanWeek} from '../generated/PlanWeek';
 import Card from './Card.svelte';
 import PlanGoals from './PlanGoals.svelte';
 let {week,open}:{week:PlanWeek;open:()=>void}=$props();
</script>

<Card title="This week" onclick={open} extra="plan-card">
 {#snippet aside()}<span class="left">{week.paused?'Paused':week.daysLeft===0?'Last day':week.daysLeft===1?'1 day left':`${week.daysLeft} days left`}</span>{/snippet}
 <PlanGoals items={week.items} compact/>
</Card>

<style>
 .left{font-size:12px;font-weight:500;letter-spacing:0;text-transform:none;color:var(--text2)}
</style>
