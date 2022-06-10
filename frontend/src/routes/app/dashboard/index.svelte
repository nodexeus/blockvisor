<script lang="ts">
  import { fade, fly } from 'svelte/transition';
  import { MetaTags } from 'svelte-meta-tags';
  import Summary from 'modules/dashboard/components/Summary/Summary.svelte';
  import RecentNodes from 'modules/dashboard/components/RecentNodes/RecentNodes.svelte';
  import HostsList from 'modules/dashboard/components/HostList/HostsList.svelte';
  import Button from 'components/Button/Button.svelte';
  import DashboardEmpty from 'modules/dashboard/components/EmptyStates/DashboardEmpty.svelte';
  import EmptyColumn from 'modules/dashboard/components/EmptyStates/EmptyColumn.svelte';
  import { ROUTES } from 'consts/routes';
  import ButtonWithDropdown from 'modules/app/components/ButtonWithDropdown/ButtonWithDropdown.svelte';
  import DropdownLinkList from 'components/Dropdown/DropdownList.svelte';
  import DropdownItem from 'components/Dropdown/DropdownItem.svelte';

  import IconAccount from 'icons/person-12.svg';
  import IconDocument from 'icons/document-12.svg';
  import IconCog from 'icons/cog-12.svg';
  import IconPlus from 'icons/plus-12.svg';
  import EarningsSummary from 'modules/dashboard/components/EarningsSummary/EarningsSummary.svelte';

  const hasNoNodes = false;
  const hasNoHosts = false;

  const hasNoData = hasNoHosts && hasNoNodes;

  const PLACEHOLDER_HOSTS = [
    {
      name: 'HostFox',
      ip: '212.213.214.2',
      location: 'Zagreb, Croatia',
      state: 'pending',
      url: ROUTES.HOST_DETAILS('id-1'),
    },
    {
      name: 'HostFox',
      ip: '212.213.214.3',
      location: 'Zagreb, Croatia',
      state: 'normal',
      url: ROUTES.HOST_DETAILS('id-2'),
    },
    {
      name: 'HostFox',
      ip: '212.213.214.4',
      location: 'Zagreb, Croatia',
      state: 'loaded',
      url: ROUTES.HOST_DETAILS('id-3'),
    },
    {
      name: 'HostFox',
      ip: '212.213.214.5',
      location: 'Zagreb, Croatia',
      state: 'issue',
      url: ROUTES.HOST_DETAILS('id-4'),
    },
  ];

  const PLACEHOLDER_NODES = [
    {
      icon: 'algo',
      name: 'BlackFox',
      ip: '212.213.214.2',
      node: 'a1234567890123456789012345678901234567890',
      added: '1h 22m',
      state: 'staked',
      url: 'placeholder-node',
    },
    {
      icon: 'avax',
      name: 'BlackFox',
      ip: '212.213.214.2',
      node: 'b1234567890123456789012345678901234567890',
      added: '1h 22m',
      state: 'syncing',
      url: 'placeholder-node',
    },
    {
      icon: 'btc',
      name: 'BlackFox',
      ip: '212.213.214.2',
      node: 'c1234567890123456789012345678901234567890',
      added: '1h 22m',
      state: 'updating',
      url: 'placeholder-node',
    },
    {
      icon: 'eth',
      name: 'BlackFox',
      ip: '212.213.214.2',
      node: 'd1234567890123456789012345678901234567890',
      added: '1h 22m',
      state: 'updating',
      url: 'placeholder-node',
    },
    {
      icon: 'hnt',
      name: 'BlackFox',
      ip: '212.213.214.2',
      node: 'e1234567890123456789012345678901234567890',
      added: '1h 22m',
      state: 'consensus',
      url: 'placeholder-node',
    },
    {
      icon: 'sol',
      name: 'BlackFox',
      ip: '212.213.214.2',
      node: 'f1234567890123456789012345678901234567890',
      added: '1h 22m',
      state: 'disabled',
      url: 'placeholder-node',
    },
  ];
</script>

<MetaTags title="Dashboard | BlockVisor" />
{#if hasNoData}
  <DashboardEmpty />
{:else}
  <div in:fade={{ duration: 100 }} class="dashboard grid-spacing">
    <section>
      <h1 class="t-large">Your Nodes</h1>

      <Summary />
    </section>

    <hr class="dashboard__divider container--pull-back" />

    <section class="dashboard__section dashboard__section--earnings">
      <h2 class="t-large s-bottom--large">Earnings</h2>

      <EarningsSummary />
    </section>

    <hr class="dashboard__divider container--pull-back" />

    <div class="container--large dashboard__container grid">
      <section
        class="dashboard__col dashboard__col--nodes"
        in:fly={{ y: 16, duration: 400, delay: 200 }}
      >
        <header class="dashboard__header">
          <h2 class="t-large">Recent Nodes</h2>
          <div class="container--buttons">
            <ButtonWithDropdown slot="action">
              <svelte:fragment slot="label">Filter <IconPlus /></svelte:fragment
              >
              <DropdownLinkList slot="content">
                <li>
                  <DropdownItem as="button">
                    <IconAccount />
                    Profile</DropdownItem
                  >
                </li>
                <li>
                  <DropdownItem as="button">
                    <IconDocument />
                    Billing</DropdownItem
                  >
                </li>
                <li>
                  <DropdownItem as="button">
                    <IconCog />
                    Settings</DropdownItem
                  >
                </li>
              </DropdownLinkList>
            </ButtonWithDropdown>
            <Button href={ROUTES.NODE_ADD} asLink style="secondary" size="small"
              >Add Node</Button
            >
          </div>
        </header>
        {#if hasNoNodes}
          <EmptyColumn>
            <svelte:fragment slot="title">No Nodes Added Yet.</svelte:fragment>
            <svelte:fragment slot="description"
              >Add a new node to get started.</svelte:fragment
            >
          </EmptyColumn>
        {:else}
          <RecentNodes data={PLACEHOLDER_NODES} />
        {/if}
      </section>

      <hr class="dashboard__col-divider" />

      <section
        class="dashboard__col dashboard__col--hosts"
        in:fly={{ y: 16, duration: 400, delay: 500 }}
      >
        <header class="dashboard__header">
          <h2 class="t-large">Hosts</h2>
          <div class="container--buttons">
            <ButtonWithDropdown slot="action">
              <svelte:fragment slot="label">Filter <IconPlus /></svelte:fragment
              >
              <DropdownLinkList slot="content">
                <li>
                  <DropdownItem as="button">
                    <IconAccount />
                    Profile</DropdownItem
                  >
                </li>
                <li>
                  <DropdownItem as="button">
                    <IconDocument />
                    Billing</DropdownItem
                  >
                </li>
                <li>
                  <DropdownItem as="button">
                    <IconCog />
                    Settings</DropdownItem
                  >
                </li>
              </DropdownLinkList>
            </ButtonWithDropdown>
            <Button href={ROUTES.HOST_ADD} asLink style="secondary" size="small"
              >Add Host</Button
            >
          </div>
        </header>
        {#if hasNoHosts}
          <EmptyColumn>
            <svelte:fragment slot="title">No Host Added Yet.</svelte:fragment>
            <svelte:fragment slot="description"
              >Add a new host to get started.</svelte:fragment
            >
          </EmptyColumn>
        {:else}
          <HostsList data={PLACEHOLDER_HOSTS} />
        {/if}
      </section>
    </div>
  </div>
{/if}

<style>
  .dashboard {
    padding-bottom: 80px;
    margin-top: 36px;
    grid-column: 1/13;
    display: flex;
    flex-direction: column;
    flex-grow: 1;

    &__section {
      grid-column: span 12;

      &--earnings {
        padding-top: 40px;
      }
    }

    &__header {
      margin-bottom: 24px;
      justify-content: space-between;
      gap: 16px 48px;
      display: flex;
      flex-direction: column;

      @media (--screen-medium-small) {
        flex-direction: row;
      }
    }

    &__container {
      grid-row-gap: 48px;
      flex-grow: 1;
    }

    &__col {
      padding-top: 48px;
      grid-column: span 12;

      @media (--screen-large) {
        grid-column: span 5;
      }
    }

    &__col-divider {
      display: none;
      margin: 0 auto;
      border-color: theme(--color-text-5-o10);

      @media (--screen-large) {
        display: block;
        width: 0;
        height: 100%;
      }
    }

    &__divider {
      border-width: 0;
      border-bottom: 1px solid theme(--color-text-5-o10);
    }
  }
</style>
