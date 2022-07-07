<script>
  import NavItem from 'components/NavItem/NavItem.svelte';
  import { ROUTES } from 'consts/routes';

  import IconGrid from 'icons/grid-12.svg';
  import IconUser from 'icons/person-12.svg';
  import IconUnion from 'icons/union-16.svg';
</script>

<ul class="u-list-reset settings-nav">
  <li class="settings-nav__item">
    <NavItem
      on:click
      class="nav-link__mobile-only"
      checkIfParent
      href={ROUTES.ADMIN_CONSOLE_DASHBOARD}
    >
      <IconGrid slot="icon" />
      <svelte:fragment slot="label">Dashboard</svelte:fragment>
    </NavItem>
  </li>

  <li class="settings-nav__item">
    <NavItem
      on:click
      class="nav-link__mobile-only"
      href={ROUTES.ADMIN_CONSOLE_USER_MANAGEMENT}
    >
      <IconUser slot="icon" />
      <svelte:fragment slot="label">User Management</svelte:fragment>
    </NavItem>
  </li>
  <li class="settings-nav__item">
    <NavItem
      on:click
      class="nav-link__mobile-only"
      href={ROUTES.ADMIN_CONSOLE_ORGANISATIONS}
    >
      <IconUnion slot="icon" />
      <svelte:fragment slot="label">Organisations</svelte:fragment>
    </NavItem>
  </li>
</ul>

<style>
  .settings-nav {
    padding-bottom: 32px;
    position: sticky;
    top: 0;
    left: 0;
    max-height: calc(100vh - 65px);
    overflow-y: auto;

    &__item {
      margin-bottom: 4px;
    }

    @media (--screen-medium-large) {
      padding-top: 32px;
    }
  }
</style>
