<script lang="ts">
  import DataRow from 'modules/nodes/components/DetailsTable/DataRow.svelte';

  export let data: NodeDetails;

  const {
    ownerAddress,
    version,
    blockHeight,
    migrateValidator,
    penalties,
    user,
  } = data;
</script>

<table class="table">
  <colspan>
    <col width="80px" />
    <col />
  </colspan>
  <tbody>
    <DataRow>
      <svelte:fragment slot="label">Owner address</svelte:fragment>
      {ownerAddress}
    </DataRow>
    <DataRow>
      <svelte:fragment slot="label">Version</svelte:fragment>
      {version}
    </DataRow>
    <DataRow>
      <svelte:fragment slot="label">Block height</svelte:fragment>
      {blockHeight}
    </DataRow>
    <DataRow>
      <svelte:fragment slot="label">Migrate validator</svelte:fragment>
      {migrateValidator}
    </DataRow>
    <DataRow>
      <svelte:fragment slot="label">Penalties</svelte:fragment>
      {penalties}
    </DataRow>
    <DataRow>
      <svelte:fragment slot="label">User</svelte:fragment>
      {user}
    </DataRow>
  </tbody>
</table>
