<script lang="ts">
  import Sorter from 'components/Sorter/Sorter.svelte';
  import LoadingSpinner from 'components/Spinner/LoadingSpinner.svelte';
  import { organisations } from '../store/organisationStore';
  import OrganisationDataRow from './OrganisationDataRow.svelte';

  let sortActive;

  const handleSort = (id: string, value: SorterValues) => {
    sortActive = { id, value };
  };
</script>

<section class="review">
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
            >Full name</Sorter
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

<style>
  .review {
    padding-bottom: 100px;
    & :global(button) {
      position: relative;
    }
  }

  .table {
    margin-bottom: 20px;
    margin-top: 64px;
  }
</style>
