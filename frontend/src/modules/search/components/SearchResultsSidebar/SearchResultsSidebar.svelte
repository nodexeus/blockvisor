<script>
  import { onDestroy, onMount } from 'svelte';
  import SwipeListener from 'swipe-listener';
  import { fade } from 'svelte/transition';
  import NoResults from '../NoResults/NoResults.svelte';
  import ResultsGroup from '../ResultsGroup/ResultsGroup.svelte';

  import IconBox from 'icons/box-12.svg';

  export let isActive = true;
  export let handleClose;

  let swipeListener;
  let element;

  onMount(() => {
    swipeListener = SwipeListener(element, { preventScroll: true });
    const isFixedRight = window.matchMedia('screen and (min-width: 55rem)');

    if (isFixedRight.matches) {
      element.addEventListener('swipe', ({ detail: { directions } }) => {
        if (directions.right) {
          handleClose?.();
        }
      });
    } else {
      element.addEventListener('swipe', ({ detail: { directions } }) => {
        if (directions.top) {
          handleClose?.();
        }
      });
    }
  });

  onDestroy(() => swipeListener && swipeListener.off());

  $: classes = [
    'search-results',
    `search-results--${isActive ? 'active' : 'inactive'}`,
  ].join(' ');

  let hasResults = true;
</script>

<aside bind:this={element} class={classes}>
  {#if isActive}
    <div
      out:fade|local={{ duration: 300, delay: 0 }}
      in:fade|local={{ duration: 300, delay: 200 }}
    >
      {#if hasResults}
        <ul class="u-list-reset search-results__list">
          <li class="search-results__item">
            <ResultsGroup>
              <svelte:fragment slot="title">Nodes</svelte:fragment>
              <IconBox slot="icon" />
            </ResultsGroup>
          </li>
        </ul>
      {:else}
        <NoResults />
      {/if}
    </div>
  {/if}
</aside>

<style>
  .search-results {
    padding: 20px 20px 40px;
    position: absolute;
    top: 65px;
    max-height: calc(80vh - 65px);
    z-index: var(--level-2);
    background-color: theme(--color-overlay-background-1);
    right: -65px;
    overflow-y: auto;
    transition: opacity 0.25s 0.1s ease;
    width: 100vw;
    opacity: 0;

    &__list {
      display: flex;
      flex-direction: column;
      gap: 40px;
    }

    @media (--screen-medium-large) {
      opacity: 1;
      transition: transform 0.3s 0.1s ease-out;
      transform: translate3d(100%, 0, 0);
      max-height: none;
      height: calc(100vh - 65px);
      width: 300px;
      position: fixed;
      left: auto;
      right: 0;
    }

    &--active {
      opacity: 1;

      @media (--screen-medium-large) {
        transform: translate3d(0, 0, 0);
        transition-delay: 0;
      }
    }
  }
</style>
