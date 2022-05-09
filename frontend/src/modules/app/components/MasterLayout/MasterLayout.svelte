<script lang="ts">
  import { fade } from 'svelte/transition';
  import SideNav from 'modules/app/components/SideNav/SideNav.svelte';
  import Breadcrumbs from 'modules/app/components/Breadcrumbs/Breadcrumbs.svelte';
  import HeaderSearch from 'modules/search/components/HeaderSearch/HeaderSearch.svelte';
  import Logo from 'components/Logo/Logo.svelte';
  import Menu from 'components/Menu/Menu.svelte';
  import { browser } from '$app/env';
  import ProfileDropdown from '../ProfileDropdown/ProfileDropdown.svelte';
  import BlockVisorLinks from '../Navigation/BlockVisorLinks.svelte';
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { ROUTES } from 'consts/routes';
  import SubMenu from '../Navigation/SubMenu.svelte';
  import ParentMenu from '../Navigation/ParentMenu.svelte';
  import SettingsNav from 'modules/admin-console/components/SettingsNav/SettingsNav.svelte';
  import { fadeDefault, pageTransition } from 'consts/animations';
  import NodeHierarchy from 'modules/nodes/components/NodeHierarchy/NodeHierarchy.svelte';
  import HostsHierarchy from 'modules/hosts/components/HostsHierarchy/HostsHierarchy.svelte';
  import { app } from 'modules/app/store';
  import BroadcastLinks from '../Navigation/BroadcastLinks.svelte';
  import { APPS } from 'models/App';
  import { FEATURE_FLAGS } from 'consts/featureFlags';
  import BlockVisorFeature from 'modules/feature-flags/components/BlockVisorFeature/BlockVisorFeature.svelte';

  const TEMP_NODES = [
    {
      title: 'All Nodes',
      href: ROUTES.NODES,
      children: [
        {
          title: 'Group 1',
          href: ROUTES.NODE_GROUP('group-1'),
          id: 'id1',
        },
        {
          title: 'Group 2',
          href: ROUTES.NODE_GROUP('group-2'),
          id: 'id2',
        },
        {
          title: 'Group 3',
          href: ROUTES.NODE_GROUP('group-3'),
          id: 'id3',
        },
      ],
    },
  ];

  const TEMP_HOSTS = [
    {
      title: 'All Hosts',
      href: ROUTES.HOSTS,
      children: [
        {
          title: 'Group 1',
          href: '#',
          id: 'host1',
        },
        {
          title: 'Group 2',
          href: '#',
          id: 'host2',
        },
        {
          title: 'Group 2',
          href: '#',
          id: 'host3',
        },
      ],
    },
  ];

  const MENU_CLOSED = 'menu';
  const MENU_OPENED = 'menu menu--active';

  let layoutClass = 'layout master-layout';
  let menuClass = MENU_CLOSED;
  let bodyClass = '';
  let disableClick = false;

  let isSearchOpen = false;
  let isResultsOpen = true;

  const handleOpen = () => {
    isSearchOpen = true;
    document.getElementById('search').focus();
  };

  const handleClose = () => {
    isSearchOpen = false;
  };

  const onMenuOpen = () => {
    if (disableClick) {
      return;
    }

    checkIfSubmenuRoute();

    bodyClass = 'scroll-lock';
    menuClass = MENU_OPENED;
    window.scrollTo({ top: 0, behavior: 'auto' });
    layoutClass = 'layout master-layout master-layout--active';
    disableClick = true;
  };

  const checkIfSubmenuRoute = () => {
    openSubmenuWhenOnPage('nodes');
    openSubmenuWhenOnPage('node', 'nodes');
    openSubmenuWhenOnPage('hosts');
    openSubmenuWhenOnPage('host', 'hosts');
    openSubmenuWhenOnPage('admin-console');
  };

  const openSubmenuWhenOnPage = (submenu: string, menu?: string) => {
    if ($page.url.pathname.includes(submenu)) {
      changeSubNav(menu || submenu);
    }
  };

  const onAppChange = () => {
    changeSubNav('');
  };

  const handleClickOutside = () => {
    if (menuClass === MENU_CLOSED) {
      return;
    }

    layoutClass = 'layout master-layout';
    bodyClass = '';
    menuClass = MENU_CLOSED;

    setTimeout(() => {
      disableClick = false;
    }, 300);
  };

  let activeSubNav = '';

  const changeSubNav = (subNav: string) => {
    activeSubNav = subNav;
  };

  onMount(checkIfSubmenuRoute);

  $: if (browser)
    document.documentElement.classList.toggle(
      'no-scroll',
      menuClass === MENU_OPENED,
    );

  $: activeApp = $app.activeApp;
</script>

{#if menuClass === MENU_OPENED}
  <aside
    in:fade|local={{ duration: 200, delay: 420 }}
    out:fade|local={{ duration: 200, delay: 0 }}
    class="layout__overlay"
    on:click={handleClickOutside}
  />
{/if}

<svelte:body class={bodyClass} />

<div class={layoutClass}>
  <SideNav callback={onAppChange} {handleClickOutside}>
    {#if activeApp === APPS.BLOCKVISOR}
      <div
        out:fade|local={fadeDefault}
        in:fade|local={{ ...fadeDefault, delay: 200 }}
      >
        <SubMenu isActive={activeSubNav === 'nodes'} {changeSubNav}>
          <NodeHierarchy on:click={handleClickOutside} />
        </SubMenu>

        <SubMenu isActive={activeSubNav === 'hosts'} {changeSubNav}>
          <HostsHierarchy on:click={handleClickOutside} nodes={TEMP_HOSTS} />
        </SubMenu>

        <SubMenu isActive={activeSubNav === 'admin-console'} {changeSubNav}>
          <SettingsNav on:click={handleClickOutside} />
        </SubMenu>

        <ParentMenu isActive={activeSubNav === ''}>
          <BlockVisorFeature type="small">
            <BlockVisorLinks {changeSubNav} />
          </BlockVisorFeature>
        </ParentMenu>
      </div>
    {:else}
      <div
        out:fade|local={fadeDefault}
        in:fade|local={{ ...fadeDefault, delay: 200 }}
      >
        <ParentMenu isActive={true}>
          <BroadcastLinks {changeSubNav} />
        </ParentMenu>
      </div>
    {/if}
  </SideNav>

  <main class="layout__content master-layout__content">
    <header class="layout__header">
      <div class="t-tiny layout__breadcrumbs hide--from-medium-large">
        <Breadcrumbs />
      </div>
      <div class="layout__menu display--to-medium-large">
        <Menu handleClick={onMenuOpen} {menuClass} />
      </div>
      <div
        class="layout__logo display--to-medium-large"
        class:layout__logo--hidden={isSearchOpen}
      >
        <Logo />
      </div>
      <div class="layout__util">
        {#if FEATURE_FLAGS.BLOCKVISOR}
          <HeaderSearch
            isOpen={isSearchOpen}
            {isResultsOpen}
            {handleOpen}
            {handleClose}
          />
        {/if}
        <ProfileDropdown />
      </div>
    </header>
    <div
      tabindex="0"
      class="layout__content"
      id="content"
      in:fade={pageTransition}
    >
      <slot />
    </div>
  </main>
</div>

<style>
  .layout {
    display: flex;
    min-height: 100vh;

    @media (--screen-medium-large) {
      overflow: hidden;
    }

    &__content {
      flex-grow: 1;
      display: flex;
      flex-direction: column;
      overflow-x: hidden;

      @media (--screen-medium-large) {
        overflow-x: hidden;
        overflow-y: auto;
        max-height: 100vh;
      }
    }

    &__util {
      min-width: 32px;
      align-items: center;
      display: flex;
      gap: 12px;

      & :global(.avatar) {
        flex-basis: 32px;
        flex-shrink: 0;
        cursor: pointer;
      }

      @media (--screen-medium-large) {
        max-width: 210px;
      }
    }

    &__breadcrumbs {
      padding-top: 26px;
      padding-bottom: 22px;
    }

    &__logo {
      text-align: center;

      @media (--screen-smaller-custom-max) {
        transition: opacity 0.15s var(--transition-easing-cubic);

        &--hidden {
          opacity: 0;
        }
      }
    }

    &__overlay {
      position: fixed;
      top: 0;
      right: 0;
      z-index: var(--level-3);
      background-color: theme(--color-shadow-o10);
      backdrop-filter: blur(8px);
      height: 100vh;
      width: calc(100% - 300px);

      @media (--screen-medium-large) {
        display: none;
      }
    }

    &__logo :global(a) {
      display: inline-block;
    }

    &__header {
      display: flex;
      gap: 16px;
      justify-content: space-between;
      align-items: center;
      padding-top: 20px;
      padding-left: 20px;
      padding-right: 20px;

      @media (--screen-medium-large) {
        padding-top: 0;
        border-bottom: 1px solid theme(--color-text-5-o10);
        padding-left: 28px;
        padding-right: 16px;
      }
    }

    & > :global(.sidenav) {
      flex-grow: 1;
    }
  }
</style>
