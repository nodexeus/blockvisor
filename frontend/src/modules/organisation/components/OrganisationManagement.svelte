<script lang="ts">
  import Button from 'components/Button/Button.svelte';

  import Sorter from 'components/Sorter/Sorter.svelte';
  import LoadingSpinner from 'components/Spinner/LoadingSpinner.svelte';
  import { organisations } from '../store/organisationStore';
  import CreateNewOrganisation from './CreateNewOrganisation.svelte';
  import OrganisationDataRow from './OrganisationDataRow.svelte';

  let sortActive;

  const handleSort = (id: string, value: SorterValues) => {
    sortActive = { id, value };
  };
  let createNewActive: boolean = false;

  function handleClickOutsideCreateNew() {
    createNewActive = false;
  }
</script>

<section class="organisation-management">
  <header class="organisation-management__header">
    <Button on:click={() => createNewActive = true} style="secondary" size="small">Create New Organisation</Button>
  </header>
  <table class="table">
    <colgroup>
      <col width="200" />
      <col />
      <col width="68" />
    </colgroup>
    <thead class="table__head">
      <tr>
        <th class="table__heading">
          <Sorter callback={handleSort} active={sortActive} id="name"
            >Org. name</Sorter
          >
        </th>
        <th class="table__heading">
          <Sorter callback={handleSort} active={sortActive} id="members"
            >Members</Sorter
          >
        </th>
        <th class="table__heading" />
      </tr>
    </thead>
    <tbody>
      {#if !$organisations}
        <LoadingSpinner size="medium" id="organisations" />
      {:else}
        {#each $organisations as item, i}
          <OrganisationDataRow {item} index={i} />
        {/each}
      {/if}
    </tbody>
  </table>
</section>
<CreateNewOrganisation
  isModalOpen={createNewActive}
  handleModalClose={handleClickOutsideCreateNew}
/>

<style>
  .organisation-management {
    padding-bottom: 100px;
    & :global(button) {
      position: relative;
    }
  }

  .organisation-management__header {
    text-align: right;
    margin-top: 40px;
  }

  .table {
    margin-bottom: 20px;
    margin-top: 24px;
  }
</style>
