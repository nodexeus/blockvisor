<script lang="ts">
  import Sorter from 'components/Sorter/Sorter.svelte';
  import BroadcastDataRow from './BroadcastDataRow.svelte';

  let sortActive;

  export let broadcasts: Broadcast[];

  const handleSort = (id: string, value: SorterValues) => {
    sortActive = { id, value };
  };
</script>

<table class="table broadcast-table">
  <colgroup>
    <col width="260px" />
    <col width="200px" />
    <col width="140px" />
    <col width="140px" />
  </colgroup>
  <thead>
    <tr>
      <th class="table__heading">
        <Sorter id="name" active={sortActive} callback={handleSort}>
          Name
        </Sorter>
      </th>
      <th class="table__heading">
        <Sorter id="added" active={sortActive} callback={handleSort}>
          Added
        </Sorter>
      </th>
      <th class="table__heading">
        <Sorter id="addresses" active={sortActive} callback={handleSort}>
          Addresses
        </Sorter>
      </th>
      <th class="table__heading">
        <Sorter id="filters" active={sortActive} callback={handleSort}>
          Filters
        </Sorter>
      </th>
      <th>
        <span class="visually-hidden">Actions</span>
      </th>
    </tr>
  </thead>
  <tbody>
    {#each broadcasts as item}
      <BroadcastDataRow {item} />
    {/each}
  </tbody>
</table>

<style>
  .broadcast-table {
    @media (--screen-smaller-max) {
      &,
      & tbody {
        display: block;
      }

      & thead {
        display: none;
      }
    }
  }
</style>
