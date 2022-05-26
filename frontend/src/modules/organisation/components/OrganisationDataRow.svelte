<script lang="ts">
  import DropdownItem from 'components/Dropdown/DropdownItem.svelte';
  import DropdownLinkList from 'components/Dropdown/DropdownList.svelte';
  import IconDots from 'icons/dots-12.svg';
  import IconDelete from 'icons/close-12.svg';
  import IconEdit from 'icons/pencil-12.svg';
  import ButtonWithDropdown from 'modules/app/components/ButtonWithDropdown/ButtonWithDropdown.svelte';
  import type { Organisation } from '../models/Organisation';

  export let item: Organisation;
  export let index: number;
</script>

<tr
  class="table__row organisation-data-row"
  in:fade={{ duration: 250, delay: index * 100 }}
>
  <td class="organisation-data-row__col">{item.name}</td>
  <td class="organisation-data-row__col">Number of members</td>
  <td class="user-data-row__col user-data-row__col--action t-right">
    members button
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

<style>
  .organisation-data-row {
    @media (--screen-large-max) {
      display: flex;
      flex-direction: column;
      gap: 18px;
      padding: 32px 0 18px;
      position: relative;
    }

    & :global(.dropdown) {
      margin-top: 0;
    }
  }

  .organisation-data-row__col {
    padding: 30px 28px 18px 0;

    &:last-child {
      padding-right: 0;
    }

    @media (--screen-large-max) {
      padding: 0 40px 0 0;
      display: block;
    }
  }

  .organisation-data-row__col--action {
    @media (--screen-large-max) {
      padding-right: 0;
      position: absolute;
      top: 32px;
      right: 0;
    }
  }
</style>
