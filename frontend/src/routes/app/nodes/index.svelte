<script lang="ts">
  import ActionTitleHeader from 'components/ActionTitleHeader/ActionTitleHeader.svelte';
  import Button from 'components/Button/Button.svelte';
  import DropdownItem from 'components/Dropdown/DropdownItem.svelte';
  import DropdownLinkList from 'components/Dropdown/DropdownList.svelte';
  import { ROUTES } from 'consts/routes';
  import IconCog from 'icons/cog-12.svg';
  import IconDocument from 'icons/document-12.svg';
  import IconAccount from 'icons/person-12.svg';
  import IconPlus from 'icons/plus-12.svg';
  import ButtonWithDropdown from 'modules/app/components/ButtonWithDropdown/ButtonWithDropdown.svelte';
  import { app } from 'modules/app/store';
  import { fetchAllNodes } from 'modules/nodes/store/nodesStore';
  import { onMount } from 'svelte';
  import { session } from '$app/stores';

  onMount(() => {
    app.setBreadcrumbs([
      {
        title: 'Nodes',
        url: ROUTES.NODES,
      },
      {
        title: 'All',
        url: '',
      },
    ]);
  });

  fetchAllNodes($session.token);
</script>

<ActionTitleHeader className="container--pull-back">
  <Button asLink style="primary" size="small" href={ROUTES.NODE_ADD}
    >Add node</Button
  >
  <h2 class="t-large" slot="title">All Nodes</h2>
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

<ul class="u-list-reset nodes__group">
  <p>No nodes selected.</p>
</ul>

<style>
  .nodes {
    &__group {
      &-item {
        margin-bottom: 60px;

        @media (--screen-medium-large) {
          margin-bottom: 120px;
        }
      }
    }
  }
</style>
