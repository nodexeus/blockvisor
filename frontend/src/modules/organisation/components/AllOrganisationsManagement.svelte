<script lang="ts">
  import Button from 'components/Button/Button.svelte';
  import Sorter from 'components/Sorter/Sorter.svelte';
  import { organisations } from '../store/organisationStore';
  import AllOrganisationsDataRow from './AllOrganizationsDataRow.svelte';

  let sortActive;

  const handleSort = (id: string, value: SorterValues) => {
    sortActive = { id, value };
  };
</script>

<section class="organisation-management">
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
      {#if $organisations}
        {#each $organisations as item, i}
          <AllOrganisationsDataRow {item} index={i} />
        {/each}
      {/if}
    </tbody>
  </table>
</section>

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
