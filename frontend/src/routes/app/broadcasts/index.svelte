<script lang="ts">
  import ActionTitleHeader from 'components/ActionTitleHeader/ActionTitleHeader.svelte';
  import Button from 'components/Button/Button.svelte';
  import DropdownItem from 'components/Dropdown/DropdownItem.svelte';
  import DropdownLinkList from 'components/Dropdown/DropdownList.svelte';
  import { fadeDefault } from 'consts/animations';
  import { ROUTES } from 'consts/routes';
  import IconCog from 'icons/cog-12.svg';
  import IconDocument from 'icons/document-12.svg';
  import IconAccount from 'icons/person-12.svg';
  import IconPlus from 'icons/plus-12.svg';
  import ActiveFilters from 'modules/app/components/ActiveFilters/ActiveFilters.svelte';
  import ButtonWithDropdown from 'modules/app/components/ButtonWithDropdown/ButtonWithDropdown.svelte';
  import { user } from 'modules/authentication/store';
  import BroadcastsTable from 'modules/broadcasts/components/BroadcastsTable/BroadcastsTable.svelte';
  import {
    broadcasts,
    getAllBroadcasts,
    getOrganisationId,
    organisationId,
  } from 'modules/broadcasts/store/broadcastStore';
  import EmptyColumn from 'modules/dashboard/components/EmptyStates/EmptyColumn.svelte';
  import { onMount } from 'svelte';
  import { fade } from 'svelte/transition';

  onMount(() => {
    getOrganisationId($user.id);
  });

  $: {
    if ($organisationId) {
      getAllBroadcasts($organisationId);
    }
  }
</script>

<!-- svelte-ignore missing-declaration -->
<ActionTitleHeader className="container--pull-back">
  <ActiveFilters slot="util" />
  <Button style="primary" asLink size="small" href={ROUTES.BROADCAST_CREATE}>
    Create a Broadcast
  </Button>
  <h2 class="t-large" slot="title">All Broadcasts</h2>
  <ButtonWithDropdown slot="action">
    <svelte:fragment slot="label">Filter <IconPlus /></svelte:fragment>
    <DropdownLinkList slot="content">
      <li>
        <DropdownItem as="button" href="#">
          <IconAccount />
          Profile</DropdownItem
        >
      </li>
      <li>
        <DropdownItem as="button" href="#">
          <IconDocument />
          Billing</DropdownItem
        >
      </li>
      <li>
        <DropdownItem as="button" href={ROUTES.PROFILE_SETTINGS}>
          <IconCog />
          Settings</DropdownItem
        >
      </li>
    </DropdownLinkList>
  </ButtonWithDropdown>
</ActionTitleHeader>
{#if $broadcasts.length}
  <section in:fade={fadeDefault} class="container--medium-large">
    <BroadcastsTable />
  </section>
{:else}
  <section in:fade={fadeDefault} class="automation-wrapper">
    <EmptyColumn id="js-empty-broadcasts">
      <svelte:fragment slot="title">No Broadcasts</svelte:fragment>
      <svelte:fragment slot="description"
        >Create a Broadcast to start recieving information from the blockchain.</svelte:fragment
      >
    </EmptyColumn>
  </section>
{/if}

<style>
  .automation {
    &-wrapper {
      max-width: 484px;
    }
  }
</style>
