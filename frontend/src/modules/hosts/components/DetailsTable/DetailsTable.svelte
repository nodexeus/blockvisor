<script>
  import HostDataRow from './HostDataRow.svelte';
</script>

<table class="table details-table">
  <colgroup>
    <col width="180px" />
    <col />
  </colgroup>

  <tbody>
    <HostDataRow>
      <svelte:fragment slot="title">Created</svelte:fragment>
      05/23/2022
    </HostDataRow>
    <HostDataRow>
      <svelte:fragment slot="title">Created</svelte:fragment>
      05/23/2022
    </HostDataRow>
    <HostDataRow>
      <svelte:fragment slot="title">Created</svelte:fragment>
      05/23/2022
    </HostDataRow>
  </tbody>
</table>

<style>
  .details-table {
    margin-bottom: 40px;

    @media (--screen-smaller-max) {
      display: block;

      & tbody {
        display: block;
      }
    }
  }
</style>
