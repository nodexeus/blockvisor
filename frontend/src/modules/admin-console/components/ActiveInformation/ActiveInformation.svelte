<script>
  import Button from 'components/Button/Button.svelte';
  import InformationItem from './InformationItem.svelte';
</script>

<section class="active-info">
  <InformationItem
    items={[
      {
        title: 'Active plan',
        info: 'Plan name',
      },
      {
        title: 'Price',
        info: '$100,00 / month',
      },
    ]}
  >
    <svelte:fragment slot="title">Plan</svelte:fragment>
    <Button size="small" style="secondary">Change Plan</Button>
    <Button size="small" style="outline">Cancel Plan</Button>
  </InformationItem>

  <InformationItem
    items={[
      {
        title: 'Credit Card',
        info: 'VISA Business',
      },
      {
        title: 'Credit Card',
        info: 'Ends with 3462',
      },
    ]}
  >
    <svelte:fragment slot="title">Credit Card Information</svelte:fragment>
    <Button size="small" style="secondary">Change</Button>
    <Button size="small" style="outline">Remove Card</Button>
  </InformationItem>
</section>

<style>
  .active-info {
    margin-top: 60px;

    & > :global(* + *) {
      margin-top: 40px;
      padding-top: 40px;
      border-top: 1px solid theme(--color-text-5-o10);
    }
  }
</style>
