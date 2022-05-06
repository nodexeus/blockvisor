<script lang="ts">
  import Sorter from 'components/Sorter/Sorter.svelte';
  import NodeDataRow from 'modules/nodes/components/GroupTable/NodeDataRow.svelte';

  export let nodes;

  const PLACEHOLDER_DATA = [
    {
      token: 'eth',
      id: '9b74c98979b74c9897',
      ip: '212.213.214.2',
      name: 'YellowBeaver',
      added: '5 weeks ago',
      state: 'unstaked',
    },
    {
      token: 'sol',
      id: '9b74c98979b74c9897',
      ip: '212.213.214.2',
      name: 'YellowBeaver',
      added: '5 weeks ago',
      state: 'consensus',
    },
    {
      token: 'btc',
      id: '9b74c98979b74c9897',
      ip: '212.213.214.2',
      name: 'YellowBeaver',
      added: '5 weeks ago',
      state: 'disabled',
    },
  ];

  let sortActive;

  console.log(nodes);

  const handleSort = (id: string, value: SorterValues) => {
    sortActive = { id, value };
  };
</script>

<table class="table node-table">
  <colgroup>
    <col width="40px" />
    <col width="280px" />
    <col width="160px" />
    <col />
    <col />
  </colgroup>
  <thead>
    <tr>
      <th class="table__heading">
        <Sorter id="token" active={sortActive} callback={handleSort}>
          <span class="visually-hidden">Token</span>
        </Sorter>
      </th>
      <th class="table__heading">
        <Sorter id="name" active={sortActive} callback={handleSort}>Name</Sorter
        >
      </th>
      <th class="table__heading">
        <Sorter id="added" active={sortActive} callback={handleSort}
          >Added</Sorter
        >
      </th>
      <th class="table__heading">
        <Sorter id="status" active={sortActive} callback={handleSort}
          >Status</Sorter
        >
      </th>
    </tr>
  </thead>
  <tbody>
    {#each nodes as node}
      <NodeDataRow {nodes} {...node} />
    {/each}
  </tbody>
</table>

<style>
  .node-table {
    margin-top: 60px;

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
