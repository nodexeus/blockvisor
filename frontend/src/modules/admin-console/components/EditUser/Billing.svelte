<script>
  import ActiveInformation from '../ActiveInformation/ActiveInformation.svelte';
  import ActivePlanEmpty from '../ActivePlanEmpty/ActivePlanEmpty.svelte';

  const hasActivePlan = false;
</script>

{#if hasActivePlan}
  <ActiveInformation />
{:else}
  <ActivePlanEmpty />
{/if}
