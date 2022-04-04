<script>
  import DropdownLinkList from 'components/Dropdown/DropdownList.svelte';
  import DropdownItem from 'components/Dropdown/DropdownItem.svelte';

  import IconPerson from 'icons/person-12.svg';
  import IconDelete from 'icons/close-12.svg';
  import IconDots from 'icons/dots-12.svg';
  import ButtonWithDropdown from 'modules/app/components/ButtonWithDropdown/ButtonWithDropdown.svelte';
</script>

<tr class="table__row broadcast">
  <td class="broadcast__col">
    <span class="t-line-clamp--2" title="My Broadcast really long name "
      >My Broadcast
    </span>
  </td>
  <td class="t-small t-color-text-2 broadcast__col">1h 33min ago</td>
  <td class="t-small t-color-text-2 broadcast__col">Weekly</td>
  <td class="t-right broadcast__col broadcast__col--controls">
    <ButtonWithDropdown
      position="right"
      slot="action"
      iconButton
      buttonProps={{ style: 'ghost', size: 'small' }}
    >
      <svelte:fragment slot="label">
        <span class="visually-hidden">Open action dropdown</span>
        <span class="user-data-row__action-icon" aria-hidden="true">
          <IconDots />
        </span>
      </svelte:fragment>
      <DropdownLinkList slot="content">
        <li>
          <DropdownItem href="#">
            <IconPerson />
            Permissions</DropdownItem
          >
        </li>
        <li class="node__item-divider">
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
  .broadcast {
    @media (--screen-smaller-max) {
      display: block;
      position: relative;
    }

    &__col {
      vertical-align: middle;
      padding: 24px 40px 20px 0;

      @media (--screen-smaller-max) {
        display: block;
        padding: 12px 0;

        &:first-child {
          padding-right: 60px;
        }

        &--controls {
          position: absolute;
          top: 0;
          right: 0;
        }
      }

      &:last-child {
        padding-right: 0;
      }
    }
  }
</style>
