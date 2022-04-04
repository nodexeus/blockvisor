<script lang="ts">
  import NavItem from 'components/NavItem/NavItem.svelte';
  import { ROUTES } from 'consts/routes';

  import IconAutomation from 'icons/refresh-12.svg';

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
  <dt class="main-links-title t-uppercase t-microlabel">Broadcasts</dt>
  <dd class="main-links-wrapper">
    <ul class="main-links u-list-reset">
      <li>
        <NavItem href={ROUTES.BROADCASTS}>
          <IconAutomation slot="icon" />
          <svelte:fragment slot="label">Dashboard</svelte:fragment>
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
