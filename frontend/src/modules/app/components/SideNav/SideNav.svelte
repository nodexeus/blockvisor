<script lang="ts">
  import ChatIcon from 'icons/chat-12.svg';
  import Logo from 'components/Logo/Logo.svelte';
  import NavItem from 'components/NavItem/NavItem.svelte';
  import SwipeListener from 'swipe-listener';
  import { onDestroy, onMount } from 'svelte';
  import AppNav from '../AppNav/AppNav.svelte';
  import { APPS } from 'models/App';
  import { goto } from '$app/navigation';
  import { ROUTES } from 'consts/routes';

  export let handleClickOutside;
  export let callback;

  let element;
  let swipeListener;

  const handleAppNav = (newValue: APPS) => {
    callback(newValue);
    if (newValue === APPS.BLOCKVISOR) {
      goto(ROUTES.DASHBOARD, { replaceState: true });
    } else {
      goto(ROUTES.BROADCASTS, { replaceState: true });
    }
  };

  onMount(() => {
    swipeListener = SwipeListener(element, { preventScroll: true });
    element.addEventListener('swipe', ({ detail: { directions } }) => {
      if (directions.left) {
        handleClickOutside();
      }
    });
  });

  onDestroy(() => swipeListener && swipeListener.off());
</script>

<aside bind:this={element} class="sidenav__wrapper">
  <div class="sidenav">
    <header class="sidenav__header">
      <Logo />
    </header>
    <div class="sidenav__nav-wrapper">
      <AppNav callback={handleAppNav} />
      <nav class="sidenav__nav">
        <div class="sidenav__links">
          <slot />
        </div>
        <footer class="sidenav__footer">
          <NavItem>
            <ChatIcon slot="icon" />
            <svelte:fragment slot="label">Get Support</svelte:fragment>
          </NavItem>
          <copy class="sidenav__copy u-o-30 t-micro">
            BlockVisor is a Blockjoy product. All rights reserved.
          </copy>
        </footer>
      </nav>
    </div>
  </div>
</aside>

<style>
  .sidenav__wrapper {
    flex: 1 0 300px;
    max-width: 300px;

    @media (--screen-medium-max) {
      max-height: 100vh;
      overflow: auto;
    }

    @media (--screen-medium-large) {
      flex: 1 0 300px;
      max-width: 300px;
      border-right: 1px solid theme(--color-text-5-o10);
      max-height: 100vh;
    }
  }

  .sidenav {
    background-color: theme(--color-foreground-secondary);
    display: flex;
    flex-direction: column;
    height: 100%;
    transition: box-shadow 1s ease;

    &__links {
      flex-grow: 1;
      padding: 16px 40px 0 20px;
    }
  }

  .sidenav__header {
    display: none;
    padding: 20px;
    border-bottom: 1px solid theme(--color-text-5-o10);
    flex-shrink: 0;

    @media (--screen-medium-large) {
      display: block;
    }
  }

  .sidenav__nav {
    display: flex;
    flex-direction: column;
    overflow: auto;
    user-select: none;
    min-height: 100%;
    flex-grow: 1;

    &-wrapper {
      height: 100%;
      display: grid;
      grid-template-columns: 60px auto;
    }
  }

  .sidenav__copy {
    display: block;
    padding-top: 20px;
  }

  .sidenav__footer {
    flex-shrink: 0;
    color: var(--color-text-5);
    padding: 24px 32px;
  }

  .sidenav__footer :global(a) {
    padding: 0;
  }
</style>
