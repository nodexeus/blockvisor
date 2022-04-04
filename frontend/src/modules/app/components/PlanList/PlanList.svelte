<script>
  import Button from 'components/Button/Button.svelte';
  import PlanCard from 'components/PlanCard/PlanCard.svelte';

  const FEATURES_BASIC = ['All BlockVisor features'];

  const FEATURES_PREMIUM = [...FEATURES_BASIC, 'Premium support'];

  const FEATURES_ENTERPRISE = [
    ...FEATURES_PREMIUM,
    'Team management',
    'Advanced Analytics & Reporting',
  ];
</script>

<ul class="u-list-reset plan-list">
  <li>
    <PlanCard plan={FEATURES_BASIC}>
      <svelte:fragment slot="title">Basic Plan</svelte:fragment>
      <svelte:fragment slot="fee">3%</svelte:fragment>
      <svelte:fragment slot="description">Fixed Fee</svelte:fragment>
      <Button display="block" size="small" slot="action" style="secondary"
        >Select Plan</Button
      >
    </PlanCard>
  </li>
  <li>
    <PlanCard plan={FEATURES_PREMIUM}>
      <svelte:fragment slot="title">Basic Plan</svelte:fragment>
      <svelte:fragment slot="fee">3%</svelte:fragment>
      <svelte:fragment slot="description">Fixed Fee</svelte:fragment>
      <Button display="block" size="small" slot="action" style="secondary"
        >Select Plan</Button
      >
    </PlanCard>
  </li>
  <li>
    <PlanCard plan={FEATURES_ENTERPRISE}>
      <svelte:fragment slot="title">Basic Plan</svelte:fragment>
      <svelte:fragment slot="fee">3%</svelte:fragment>
      <svelte:fragment slot="description">Fixed Fee</svelte:fragment>
      <Button display="block" size="small" slot="action" style="secondary"
        >Select Plan</Button
      >
    </PlanCard>
  </li>
</ul>

<style>
  .plan-list {
    display: grid;
    padding-bottom: 16px;
    grid-template-columns: repeat(3, minMax(250px, 1fr));
    gap: 28px;
    overflow: auto;
  }
</style>
