<script lang="ts">
  import Sorter from 'components/Sorter/Sorter.svelte';
  import BroadcastDataRow from './BroadcastDataRow.svelte';

  let sortActive;

  const handleSort = (id: string, value: SorterValues) => {
    sortActive = { id, value };
  };
</script>

<table class="table broadcast-table">
  <colgroup>
    <col width="160px" />
    <col width="140px" />
    <col width="140px" />
    <col />
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
        <Sorter id="interval" active={sortActive} callback={handleSort}>
          Interval
        </Sorter>
      </th>
      <th>
        <span class="visually-hidden">Actions</span>
      </th>
    </tr>
  </thead>
  <tbody>
    <BroadcastDataRow />
    <BroadcastDataRow />
    <BroadcastDataRow />
    <BroadcastDataRow />
    <BroadcastDataRow />
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
