<script lang="ts">
  import { browser } from '$app/env';
  import { page } from '$app/stores';
  import Logo from 'components/Logo/Logo.svelte';
  import Menu from 'components/Menu/Menu.svelte';
  import { fadeDefault, pageTransition } from 'consts/animations';
  import { FEATURE_FLAGS } from 'consts/featureFlags';
  import { APPS } from 'models/App';
  import SettingsNav from 'modules/admin-console/components/SettingsNav/SettingsNav.svelte';
  import Breadcrumbs from 'modules/app/components/Breadcrumbs/Breadcrumbs.svelte';
  import SideNav from 'modules/app/components/SideNav/SideNav.svelte';
  import { app } from 'modules/app/store';
  import BlockVisorFeature from 'modules/feature-flags/components/BlockVisorFeature/BlockVisorFeature.svelte';
  import HostsHierarchy from 'modules/hosts/components/HostsHierarchy/HostsHierarchy.svelte';
  import NodeHierarchy from 'modules/nodes/components/NodeHierarchy/NodeHierarchy.svelte';
  import HeaderSearch from 'modules/search/components/HeaderSearch/HeaderSearch.svelte';
  import { onMount } from 'svelte';
  import { fade } from 'svelte/transition';
  import BlockVisorLinks from '../Navigation/BlockVisorLinks.svelte';
  import BroadcastLinks from '../Navigation/BroadcastLinks.svelte';
  import ParentMenu from '../Navigation/ParentMenu.svelte';
  import SubMenu from '../Navigation/SubMenu.svelte';
  import OrganisationSelector from '../OrganisationSelector/OrganisationSelector.svelte';
  import ProfileDropdown from '../ProfileDropdown/ProfileDropdown.svelte';

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
          <HostsHierarchy on:click={handleClickOutside} />
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
      <div class="layout__menu">
        <Menu handleClick={onMenuOpen} {menuClass} />
        <div class="layout__organisation"><OrganisationSelector /></div>
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
  }

  .layout__content {
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

  .layout__util {
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

  .layout__breadcrumbs {
    padding-top: 26px;
    padding-bottom: 22px;
  }

  .layout__logo {
    text-align: center;

    @media (--screen-smaller-custom-max) {
      transition: opacity 0.15s var(--transition-easing-cubic);

      &--hidden {
        opacity: 0;
      }
    }
  }

  .layout__overlay {
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

  .layout__logo :global(a) {
    display: inline-block;
  }

  .layout__header {
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

  .layout > :global(.sidenav) {
    flex-grow: 1;
  }

  .layout__menu {
    display: flex;
    align-items: center;

    @media (--screen-medium-large) {
      display: none;
    }
  }

  .layout__organisation {
    margin-left: 22px;

    :global(.organisation-selector__name) {
      display: none;
    }
  }
</style>
