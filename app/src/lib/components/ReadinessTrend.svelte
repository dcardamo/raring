<script lang="ts">
 import HistoryChart from './HistoryChart.svelte';
 let {values,date}:{values:(number|null)[];date:string}=$props();
</script>
<HistoryChart {values} endDate={date} label="Training Readiness" unit="/ 100" readiness metric="readiness"/>
