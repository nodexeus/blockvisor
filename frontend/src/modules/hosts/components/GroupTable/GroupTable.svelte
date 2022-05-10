<script lang="ts">
  import Sorter from 'components/Sorter/Sorter.svelte';
  import { ROUTES } from 'consts/routes';
  import HostDataRow from './HostDataRow.svelte';

  export let hosts;

  const PLACEHOLDER_DATA = [
    {
      name: 'HostFox',
      ip: '212.213.214.2',
      location: 'Zagreb, Croatia',
      status: 'pending',
      url: ROUTES.HOST_DETAILS('id-1'),
    },
    {
      name: 'HostFox',
      ip: '212.213.214.2',
      location: 'Zagreb, Croatia',
      status: 'normal',
      url: ROUTES.HOST_DETAILS('id-2'),
    },
    {
      name: 'HostFox',
      ip: '212.213.214.2',
      location: 'Zagreb, Croatia',
      status: 'issue',
      url: ROUTES.HOST_DETAILS('id-3'),
    },
    {
      name: 'HostFox',
      ip: '212.213.214.2',
      location: 'Zagreb, Croatia',
      status: 'loaded',
      url: ROUTES.HOST_DETAILS('id-4'),
    },
  ];

  let sortActive;

  const handleSort = (id: string, value: SorterValues) => {
    sortActive = { id, value };
  };
</script>

<table class="table hosts-table">
  <colgroup>
    <col width="320px" />
    <col width="100px" />
    <col />
  </colgroup>
  <thead>
    <tr>
      <th class="table__heading">
        <Sorter callback={handleSort} active={sortActive} id="name">Name</Sorter
        >
      </th>
      <th class="table__heading">
        <Sorter callback={handleSort} active={sortActive} id="status"
          >Status</Sorter
        >
      </th>
      <th class="table__heading">
        <span class="visually-hidden">Actions</span>
      </th>
    </tr>
  </thead>
  <tbody>
    {#each hosts as host}
      <HostDataRow {...host} />
    {/each}
  </tbody>
</table>

<style>
  .hosts-table {
    @media (--screen-medium-larger-max) {
      thead {
        display: none;
      }

      &,
      & :global(tbody) {
        display: block;
        width: 100%;
      }
    }
  }
</style>
