<script lang="ts">
  import GroupTitle from 'components/GroupTitle/GroupTitle.svelte';
  import MinimalLineGraph from 'modules/charts/components/MinimalLineGraph/MinimalLineGraph.svelte';
  import { CONFIG_EARNINGS } from 'modules/charts/configs/chart-earnings';
  import IconBox from 'icons/box-12.svg';
  import GroupTable from 'modules/nodes/components/GroupTable/GroupTable.svelte';
  import EmptyColumn from 'modules/dashboard/components/EmptyStates/EmptyColumn.svelte';

  export let nodes: [];
  export let id = 'js-group-graphic';

  /* Temp - generates random data for graphs */
  var getDaysArray = function (s, e) {
    for (var a = [], d = new Date(s); d <= e; d.setDate(d.getDate() + 1)) {
      const randomDate = new Date(d);
      a.push({
        x: randomDate,
        y: Math.floor(Math.random() * (100 - 60 + 1)) + 60,
      });
    }
    return a;
  };

  const PLACEHOLDER_DATA = getDaysArray(
    new Date('2022-06-01'),
    new Date('2022-07-01'),
  );

  $: hasItems = true;
</script>

<div class="s-bottom--large">
  <div class="node-group__section">
    <GroupTitle>
      <slot name="title" slot="title" />
      <slot name="action" slot="action" />
      <svelte:fragment slot="stats">
        <IconBox />
        {nodes.length || 0} nodes
      </svelte:fragment>
    </GroupTitle>
  </div>
  {#if hasItems}
    <div class="container--medium-large">
      <MinimalLineGraph
        height="200"
        config={CONFIG_EARNINGS}
        data={[PLACEHOLDER_DATA]}
      >
        <slot name="label" slot="label" />
      </MinimalLineGraph>
      <GroupTable {nodes} />
    </div>
  {:else}
    <div class="node-group--empty">
      <EmptyColumn id={`graphic-${id}`}>
        <svelte:fragment slot="title">No Nodes In Group</svelte:fragment>
        <svelte:fragment slot="description"
          >Assign a node to this group to easily track multiple node
          performance.</svelte:fragment
        >
      </EmptyColumn>
    </div>
  {/if}
</div>

<style>
  .node-group {
    &--empty {
      max-width: 484px;
    }

    &__section {
      margin-bottom: 40px;
    }
  }
</style>
