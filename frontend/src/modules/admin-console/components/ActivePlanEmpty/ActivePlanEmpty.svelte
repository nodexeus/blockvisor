<script>
  import PlanList from 'modules/app/components/PlanList/PlanList.svelte';
  import EmptyColumn from 'modules/dashboard/components/EmptyStates/EmptyColumn.svelte';
</script>

<section class="active-plan-empty">
  <EmptyColumn>
    <svelte:fragment slot="title">You Have No Active Plans.</svelte:fragment>
    <svelte:fragment slot="description">
      You are currently on a free plan. To access BlockVisor features add a paid
      plan.
    </svelte:fragment>
  </EmptyColumn>
</section>

<section class="active-plan-list">
  <PlanList />
</section>

<style>
  .active-plan-empty {
    margin-bottom: 80px;
    max-width: 484px;
  }

  .active-plan-list {
    margin-bottom: 40px;
  }
</style>
