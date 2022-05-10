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
  import HostGroup from 'modules/hosts/components/HostGroup/HostGroup.svelte';
  import Button from 'components/Button/Button.svelte';
  import { ROUTES } from 'consts/routes';

  onMount(() => {
    app.setBreadcrumbs([
      {
        title: 'Hosts',
        url: ROUTES.HOSTS,
      },
      {
        title: 'All',
        url: '',
      },
    ]);
  });
</script>

<ActionTitleHeader className="container--pull-back">
  <Button asLink style="primary" size="small" href={ROUTES.HOST_ADD}
    >Add host</Button
  >
  <h2 class="t-large" slot="title">All Hosts</h2>
  <ButtonWithDropdown slot="action">
    <svelte:fragment slot="label">Filter <IconPlus /></svelte:fragment>
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
</ActionTitleHeader>

<section class="container--medium-large ">
  <p>Select host group.</p>
</section>
