<script>
  import GroupItem from '../GroupItem/GroupItem.svelte';

  export let items = [
    {
      title: 'Node name 1',
      description: 'Node group name',
    },
    {
      title: 'Node name 2',
      description: 'Node group name',
    },
    {
      title: 'Node name 3',
      description: 'Node group name',
    },
  ];
</script>

<article class="results-group">
  <div class="results-group__title t-uppercase t-microlabel">
    <slot name="title" />
  </div>
  {#if Boolean(items.length)}
    <ul class="u-list-reset results-group__list">
      {#each items as item}
        <li class="results-group__item">
          <GroupItem {...item}>
            <slot name="icon" slot="icon" />
          </GroupItem>
        </li>
      {/each}
    </ul>
  {/if}
</article>

<style>
  .results-group {
    &__item + :global(&__item) {
      padding-top: 16px;
      margin-top: 16px;
      border-top: 1px solid theme(--color-text-5-o10);
    }

    &__title {
      margin-bottom: 28px;
      color: theme(--color-text-3);
    }
  }
</style>
