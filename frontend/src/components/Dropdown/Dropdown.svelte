<script lang="ts">
  import Portal from 'svelte-portal/src/Portal.svelte';
  import { fadeDefault, flyDefault } from 'consts/animations';
  import { fly, fade } from 'svelte/transition';
  import { browser } from '$app/env';
  import SwipeListener from 'swipe-listener';
  import { afterUpdate, onDestroy, onMount } from 'svelte';

  export let onSwipeDown;

  let mobileElement;
  let swipeListener;

  afterUpdate(() => {
    if (!onSwipeDown || !mobileElement) {
      return;
    }

    swipeListener = SwipeListener(mobileElement, { preventScroll: true });
    mobileElement.addEventListener('swipe', ({ detail: { directions } }) => {
      if (directions.bottom) {
        onSwipeDown();
      }
    });
  });

  onDestroy(() => swipeListener && swipeListener.off());

  export let isActive = false;
</script>

{#if isActive && browser}
  <aside
    transition:fly|local={flyDefault}
    class="dropdown dropdown--desktop"
    {...$$restProps}
  >
    <slot />
  </aside>

  <Portal hidden={true} target="body">
    <aside
      bind:this={mobileElement}
      transition:fade|local={fadeDefault}
      class="dropdown-wrapper"
    >
      <div transition:fly|local={flyDefault} class="dropdown" {...$$restProps}>
        <slot />
      </div>
    </aside>
  </Portal>
{/if}

<style>
  .dropdown {
    position: absolute;
    top: 100%;
    background-color: theme(--color-overlay-background-1);
    z-index: var(--level-2);
    border-radius: 4px;
    min-width: max-content;

    &-wrapper {
      display: none;
    }

    @media (--screen-medium-max) {
      position: relative;
      max-width: 320px;
      width: 100%;
      border-radius: 12px;

      & :global(.dropdown-item) {
        padding: 18px 20px;
      }

      &-wrapper {
        background-color: theme(--color-shadow-o10);
        backdrop-filter: blur(8px);
        display: block;
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: flex-end;
        position: fixed;
        top: 0;
        left: 0;
        right: 0;
        min-height: 100%;
        z-index: var(--level-9);
        padding: 20px;
      }

      &--desktop {
        display: none;
      }
    }
  }
</style>
