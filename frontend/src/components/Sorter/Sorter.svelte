<script lang="ts">
  import IconSort from 'icons/sort-12.svg';
  import type { SorterProps } from './Sorter.svelte';

  const SORTER_VALUES: SorterValues[] = ['none', 'asc', 'desc'];

  export let id;
  export let active = { id: null, value: SORTER_VALUES[0] };

  let activeDirection = 0;

  $: isActive = active.id === id;

  $: {
    const shouldReset = !isActive;

    if (shouldReset) {
      activeDirection = 0;
    }
  }

  export let callback: SorterProps['callback'];

  const handleClick = () => {
    let newValue = activeDirection + 1;

    if (newValue > SORTER_VALUES.length - 1) {
      newValue = 0;
    }

    activeDirection = newValue;

    callback(id, SORTER_VALUES[newValue]);
  };

  $: classes = [
    'u-button-reset t-microlabel t-uppercase sorter',
    `sorter--${isActive ? active.value : SORTER_VALUES[0]}`,
  ].join(' ');
</script>

<button on:click={handleClick} class={classes}>
  <span class="visually-hidden">Sort by </span>
  <slot />
  <IconSort class="sorter__arrow" />
</button>

<style>
  .sorter {
    padding: 6px 0;
    display: inline-flex;
    color: theme(--color-text-4);
    align-items: center;
    gap: 6px;
    line-height: 1;
    transition: color 0.15s var(--transition-easing-cubic);

    & :global(.sorter__arrow) {
      color: theme(--color-text-2);

      & :global(path) {
        transition: color 0.15s var(--transition-easing-cubic);
      }
    }

    &:hover,
    &:active {
      color: theme(--color-text-5);

      & :global(.sorter__arrow) {
        color: theme(--color-text-3);
      }
    }

    &--asc {
      & :global(.sorter__arrow--up) {
        color: theme(--color-text-5);
      }
    }

    &--desc {
      & :global(.sorter__arrow--down) {
        color: theme(--color-text-5);
      }
    }
  }
</style>
