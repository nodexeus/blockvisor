<script lang="ts">
  import Button from 'components/Button/Button.svelte';
  import DropdownItem from 'components/Dropdown/DropdownItem.svelte';
  import DropdownLinkList from 'components/Dropdown/DropdownList.svelte';
  import IconDelete from 'icons/close-12.svg';
  import IconDots from 'icons/dots-12.svg';
  import IconEdit from 'icons/pencil-12.svg';
  import ButtonWithDropdown from 'modules/app/components/ButtonWithDropdown/ButtonWithDropdown.svelte';
  import { fade } from 'svelte/transition';
  import type { Organisation } from '../models/Organisation';
  import OrganisationMembersManagement from './OrganisationMembersManagement.svelte';

  export let item: Organisation;
  export let index: number;
</script>

<tr
  class="table__row organisation-data-row"
  in:fade={{ duration: 250, delay: index * 100 }}
>
  <td class="organisation-data-row__col">{item.name}</td>
  <td class="organisation-data-row__col">Number of members</td>
  <td
    class="organisation-data-row__col organisation-data-row__col--action t-right"
  >
    <Button size="small" style="outline">Members</Button>
    <ButtonWithDropdown
      iconButton
      position="right"
      buttonProps={{ style: 'ghost', size: 'tiny' }}
    >
      <svelte:fragment slot="label">
        <span class="visually-hidden">Open action dropdown</span>
        <span
          class="organisation-data-row__action-icon t-color-text-2"
          aria-hidden="true"
        >
          <IconDots />
        </span>
      </svelte:fragment>
      <DropdownLinkList slot="content">
        <li>
          <DropdownItem href="#">
            <IconEdit />
            Rename</DropdownItem
          >
        </li>
        <li>
          <DropdownItem href="#">
            <IconDelete />
            Delete</DropdownItem
          >
        </li>
      </DropdownLinkList>
    </ButtonWithDropdown>
  </td>
</tr>
<OrganisationMembersManagement />

<style>
  .organisation-data-row {
    @media (--screen-smaller-max) {
      display: block;
      position: relative;
    }
  }
  .organisation-data-row__col {
    padding-bottom: 10px;
  }

  .organisation-data-row__col--action {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .organisation-data-row {
    padding-top: 32px;
    padding-bottom: 18px;
    padding-right: 28px;

    @media (--screen-smaller-max) {
      padding: 16px 72px 16px 0;
      display: block;
    }

    &:last-child {
      padding-right: 0;
    }
  }

  .organisation-data-row__link {
    display: inline-block;
    padding: 2px 8px;
  }
</style>
