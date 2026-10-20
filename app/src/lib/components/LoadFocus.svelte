<script lang="ts">
 import type {LoadFocusBar} from '../generated/LoadFocusBar';
 import {colors} from '../charts';
 import {num} from '../format';
 import AmountRows from './AmountRows.svelte';
 let {bars}:{bars:LoadFocusBar[]}=$props();
</script>
<AmountRows label="Garmin load focus, last four weeks" referenceLabel="Outlined band · Garmin's target" rows={bars.map(b=>({
 label:b.name,value:b.value,text:num(b.value),detail:b.shortage?'Shortage':undefined,
 band:b.target,color:b.shortage?colors.fair:colors.ink,
 comparisonText:b.target?'Target '+num(b.target[0])+'–'+num(b.target[1]):'Target unavailable',
}))}/>
