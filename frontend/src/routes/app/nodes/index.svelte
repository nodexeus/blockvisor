<script lang="ts">
  import ActionTitleHeader from 'components/ActionTitleHeader/ActionTitleHeader.svelte';
  import ButtonWithDropdown from 'modules/app/components/ButtonWithDropdown/ButtonWithDropdown.svelte';
  import { app } from 'modules/app/store';

  import DropdownLinkList from 'components/Dropdown/DropdownList.svelte';
  import DropdownItem from 'components/Dropdown/DropdownItem.svelte';

  import IconAccount from 'icons/person-12.svg';
  import IconDocument from 'icons/document-12.svg';
  import IconCog from 'icons/cog-12.svg';
  import IconPlus from 'icons/plus-12.svg';

  import { onMount } from 'svelte';
  import { ROUTES } from 'consts/routes';
  import NodeGroup from 'modules/nodes/components/NodeGroup/NodeGroup.svelte';
  import Button from 'components/Button/Button.svelte';

  onMount(() => {
    app.setBreadcrumbs([
      {
        title: 'Nodes',
        url: '/nodes',
      },
      {
        title: 'All',
        url: '/nodes',
      },
    ]);
  });
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
  <li class="nodes__group-item">
    <NodeGroup id="group-1" nodes={52}>
      <svelte:fragment slot="label">Group earnings (USD)</svelte:fragment>
      <svelte:fragment slot="title">Node group 1</svelte:fragment>
      <Button
        asLink
        href={ROUTES.NODE_GROUP('group-1')}
        style="outline"
        size="small"
        slot="action">View</Button
      >
    </NodeGroup>
  </li>
  <li class="nodes__group-item">
    <NodeGroup id="group-2" nodes={42}>
      <svelte:fragment slot="label">Group earnings (USD)</svelte:fragment>
      <svelte:fragment slot="title">Node group 2</svelte:fragment>
      <Button
        asLink
        href={ROUTES.NODE_GROUP('group-2')}
        style="outline"
        size="small"
        slot="action">View</Button
      >
    </NodeGroup>
  </li>
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
