<script lang="ts">
  import IconBox from 'icons/box-12.svg';
  import GroupTitle from 'components/GroupTitle/GroupTitle.svelte';
  import GroupTable from 'modules/hosts/components/GroupTable/GroupTable.svelte';
  import EmptyColumn from 'modules/dashboard/components/EmptyStates/EmptyColumn.svelte';

  export let hosts;
  export let id;

  $: hasItems = false;
</script>

<article class="host-group">
  <div class="host-group__section">
    <GroupTitle>
      <slot name="title" slot="title" />
      <slot name="action" slot="action" />
      <svelte:fragment slot="stats">
        <IconBox />
        {hosts} hosts
      </svelte:fragment>
    </GroupTitle>
  </div>

  {#if hasItems}
    <GroupTable />
  {:else}
    <div class="host-group--empty">
      <EmptyColumn id={`graphic-${id}`}>
        <svelte:fragment slot="title">No Nodes In Group</svelte:fragment>
        <svelte:fragment slot="description"
          >Assign a node to this group to easily track multiple node
          performance.</svelte:fragment
        >
      </EmptyColumn>
    </div>
  {/if}
</article>

<style>
  .host-group {
    margin-bottom: 40px;

    &--empty {
      max-width: 484px;
    }

    &__section {
      margin-bottom: 40px;
    }

    @media (--screen-medium) {
      margin-bottom: 120px;
    }
  }
</style>
