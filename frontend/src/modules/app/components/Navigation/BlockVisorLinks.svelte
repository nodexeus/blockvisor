<script lang="ts">
  import NavItem from 'components/NavItem/NavItem.svelte';
  import { ROUTES } from 'consts/routes';

  import IconBox from 'icons/box-12.svg';
  import IconGrid from 'icons/grid-12.svg';
  import IconHost from 'icons/host-12.svg';
  import IconSliders from 'icons/sliders-12.svg';

  export let changeSubNav;

  const handleSubmenu = (e: MouseEvent) => {
    e.preventDefault();
    const { value } = e.currentTarget.dataset;

    if (!value) {
      return;
    }

    changeSubNav(value);
  };
</script>

<dl>
  <dt class="main-links-title t-uppercase t-microlabel">Nodes & Hosts</dt>
  <dd class="main-links-wrapper">
    <ul class="main-links u-list-reset">
      <li>
        <NavItem href={ROUTES.DASHBOARD}>
          <IconGrid slot="icon" />
          <svelte:fragment slot="label">Dashboard</svelte:fragment>
        </NavItem>
      </li>

      <li>
        <NavItem
          urlAlias="node"
          class="nav-link__desktop-only"
          href={ROUTES.NODES}
        >
          <IconBox slot="icon" />
          <svelte:fragment slot="label">Nodes</svelte:fragment>
        </NavItem>
      </li>

      <li>
        <NavItem
          urlAlias="node"
          class="nav-link__mobile-only"
          href={ROUTES.NODES}
          on:click={handleSubmenu}
          data-value="nodes"
        >
          <IconBox slot="icon" />
          <svelte:fragment slot="label">Nodes</svelte:fragment>
        </NavItem>
      </li>

      <li>
        <NavItem
          urlAlias="host"
          class="nav-link__desktop-only"
          href={ROUTES.HOSTS}
        >
          <IconHost slot="icon" />
          <svelte:fragment slot="label">Hosts</svelte:fragment>
        </NavItem>
      </li>

      <li>
        <NavItem
          class="nav-link__mobile-only"
          href={ROUTES.HOSTS}
          urlAlias="host"
          on:click={handleSubmenu}
          data-value="hosts"
        >
          <IconHost slot="icon" />
          <svelte:fragment slot="label">Hosts</svelte:fragment>
        </NavItem>
      </li>
    </ul>
  </dd>

  <dt class="main-links-title t-uppercase t-microlabel">Admin</dt>
  <dd class="main-links-wrapper">
    <ul class="main-links u-list-reset">
      <li>
        <NavItem
          class="nav-link__desktop-only"
          href={ROUTES.ADMIN_CONSOLE_DASHBOARD}
        >
          <IconSliders slot="icon" />
          <svelte:fragment slot="label">Admin Console</svelte:fragment>
        </NavItem>
      </li>
      <li>
        <NavItem
          class="nav-link__mobile-only"
          href={ROUTES.ADMIN_CONSOLE_DASHBOARD}
          on:click={handleSubmenu}
          data-value="admin-console"
        >
          <IconSliders slot="icon" />
          <svelte:fragment slot="label">Admin Console</svelte:fragment>
        </NavItem>
      </li>
    </ul>
  </dd>
</dl>

<style>
  .main-links {
    & :global(li + li) {
      margin-top: 8px;
    }

    &-title {
      color: theme(--color-text-3);
      margin-bottom: 20px;
    }

    &-wrapper :global(+ .main-links-title) {
      margin-top: 32px;
    }
  }
  :global(.sidenav .nav-item.nav-link__desktop-only) {
    @media (--screen-medium-max) {
      display: none;
    }
  }
  :global(.sidenav .nav-item.nav-link__mobile-only) {
    @media (--screen-medium-large) {
      display: none;
    }
  }
</style>
