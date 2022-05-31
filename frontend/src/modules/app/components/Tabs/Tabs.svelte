<script>
  import { page } from '$app/stores';
  import { onMount } from 'svelte';

  import { fade } from 'svelte/transition';
  import { updateQueryParam } from 'utils/updateQueryParam';

  export let items;
  export let activeTabValue = 1;

  let container;

  const handleClick = (tabValue) => (e) => {
    activeTabValue = tabValue;
    updateQueryParam({ tab: tabValue.toString() });
    container?.scrollTo({ left: e.target.offsetLeft, behavior: 'smooth' });
  };

  onMount(() => {
    const tabValue = parseInt($page.url.searchParams.get('tab'), 10);
    if (!tabValue || tabValue === NaN || tabValue > items.length) {
      return;
    }
    activeTabValue = tabValue;
  });
</script>

{#if Boolean(items.length)}
  <nav class="tabs">
    <ul bind:this={container} class="u-list-reset tabs__list">
      {#each items as { value, label }}
        <li>
          <button
            class:tabs__button--active={activeTabValue === value}
            class="u-button-reset tabs__button t-medium"
            on:click={handleClick(value)}>{label}</button
          >
        </li>
      {/each}
    </ul>
  </nav>

  {#each items as { value, component }}
    {#if activeTabValue == value}
      <div in:fade|local={{ duration: 250 }}>
        <svelte:component this={component} />
      </div>
    {/if}
  {/each}
{/if}

<style>
  .tabs {
    box-shadow: inset 0 -1px 0 0 theme(--color-text-5-o10);
    position: relative;
    overflow: hidden;
    max-width: 100wv;
  }
  .tabs__list {
    overflow: auto;
    display: flex;
    gap: 32px;
    max-width: 100%;

    &::-webkit-scrollbar-track {
      border-top: 1px solid theme(--color-text-5-o10);
    }
  }

  .tabs__button {
    white-space: nowrap;
    padding: 20px 0;
    color: theme(--color-text-2);
    border-bottom: 1px solid transparent;
    transition: color 0.15s var(--transition-easing-cubic),
      border-color 0.15s var(--transition-easing-cubic);
  }

  .tabs__button--active {
    color: theme(--color-text-5);
    border-bottom: 1px solid theme(--color-text-5);
  }
</style>
