<script lang="ts">
  import { goto } from '$app/navigation';

  import Button from 'components/Button/Button.svelte';
  import { ROUTES } from 'consts/routes';
  import { format } from 'date-fns';
  import DataRow from 'modules/nodes/components/DetailsTable/DataRow.svelte';
  import { installedNode } from 'modules/nodes/store/nodesStore';

  function handleViewNode() {
    goto(ROUTES.NODES);
  }
</script>

<section>
  <p>Your node has been successfully created</p>

  <section class="review">
    <table class="table">
      <colspan>
        <col width="80px" />
        <col />
      </colspan>
      <tbody>
        <DataRow>
          <svelte:fragment slot="label">Address</svelte:fragment>
          {$installedNode?.address || 'n/a'}
        </DataRow>
        <DataRow>
          <svelte:fragment slot="label">Block height</svelte:fragment>
          {$installedNode?.block_height || 'n/a'}
        </DataRow>
        <DataRow>
          <svelte:fragment slot="label">Chain type</svelte:fragment>
          <span class="title-case">{$installedNode?.chain_type || 'n/a'}</span>
        </DataRow>
        <DataRow>
          <svelte:fragment slot="label">Created at</svelte:fragment>
          {$installedNode.created_at
            ? format(+new Date($installedNode.created_at), 'dd MMM yyyy')
            : 'n/a'}
        </DataRow>
        <DataRow>
          <svelte:fragment slot="label">Host id</svelte:fragment>
          {$installedNode?.host_id || 'n/a'}
        </DataRow>
        <DataRow>
          <svelte:fragment slot="label">Node Type</svelte:fragment>
          <span class="title-case">{$installedNode?.node_type || 'n/a'}</span>
        </DataRow>
        <DataRow>
          <svelte:fragment slot="label">Status</svelte:fragment>
          <span class="title-case">{$installedNode?.status || 'n/a'}</span>
        </DataRow>
      </tbody>
    </table>

    <Button
      size="medium"
      display="block"
      style="primary"
      on:click={handleViewNode}
    >
      Done
    </Button>
  </section>
</section>

<style>
  .title-case {
    text-transform: capitalize;
  }

  .review {
    padding-bottom: 100px;
    & :global(button) {
      position: relative;
    }
  }

  .table {
    margin-bottom: 20px;
  }
</style>
