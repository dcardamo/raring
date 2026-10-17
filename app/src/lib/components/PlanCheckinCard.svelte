<script lang="ts">
 import type {PlanCheckin} from '../generated/PlanCheckin';
 import {colors} from '../charts';
 import {num} from '../format';
 import {delta,planIcons,planNames} from '../plan';
 import Card from './Card.svelte';
 import Icon from './Icon.svelte';
 import PlanSummary from './PlanSummary.svelte';
 let {plan,keep,change}:{plan:PlanCheckin;keep:()=>void;change:()=>void}=$props();
</script>

<Card title="Your plan" extra="plan-checkin">
 <PlanSummary week={plan.week} count={plan.week.reached}/>
 <div class="rows">
  {#each plan.week.items as item,i (item.kind)}
   <div class="row-item">
    <span class="icon"><Icon name={planIcons[item.kind]} size={18}/></span>
    <b>{planNames[item.kind]}</b>
    {#if item.unavailable}<span class="value muted">{item.unavailable}</span>
    {:else}
     <span class="value n">{num(item.value)}<small>&nbsp;of {num(item.target)}</small></span>
     <span class="mark" style:color={item.state==='done'?colors.good:colors.fair}>{#if item.state==='done'}<Icon name="check" size={18}/>{:else}<i aria-label="Short of target"></i>{/if}</span>
     <span class="delta">{delta(item.value,plan.previous[i]??null)}</span>
    {/if}
   </div>
  {/each}
 </div>
 <h3>This week</h3>
 <p>{plan.next}</p>
 <div class="actions"><button class="primary" onclick={keep}>Keep</button><button class="secondary" onclick={change}>Change</button></div>
</Card>

<style>
 .rows{border-top:1px solid var(--line)}
 .row-item{display:grid;grid-template-columns:34px 1fr auto 22px 44px;align-items:center;gap:10px;padding:12px 0;border-bottom:1px solid var(--line)}
 .icon{width:34px;height:34px;border-radius:11px;display:grid;place-items:center;background:rgba(255,255,255,.06);color:var(--text2)}
 .row-item b{font:600 15px Inter;min-width:0}
 .value{font-size:20px;font-weight:600;white-space:nowrap}
 .value.muted{font:500 12px Inter;color:var(--text2);white-space:normal;grid-column:span 3;text-align:right}
 .value small{font:500 13px Inter;color:var(--text2)}
 .mark{display:grid;place-items:center}
 .mark i{width:9px;height:9px;border-radius:50%;background:currentColor}
 .delta{font-size:13px;color:var(--text2);text-align:right;white-space:nowrap}
 h3{font:600 11px Inter;letter-spacing:.14em;text-transform:uppercase;color:var(--text2);margin-top:20px}
 p{color:var(--text2);margin-top:6px}
 .actions{display:grid;grid-template-columns:1.4fr 1fr;gap:12px}
</style>
