<script>
  import ButtonWithDropdown from 'modules/app/components/ButtonWithDropdown/ButtonWithDropdown.svelte';
  import DropdownLinkList from 'components/Dropdown/DropdownList.svelte';
  import DropdownItem from 'components/Dropdown/DropdownItem.svelte';

  import IconEdit from 'icons/pencil-12.svg';

  import IconDots from 'icons/dots-12.svg';
  import { ROUTES } from 'consts/routes';
</script>

<tr class="table__row user-data-row">
  <td class="user-data-row__col">Mark Bryant</td>
  <td class="user-data-row__col">markbryant123@hi.com</td>
  <td class="user-data-row__col">Grounded Skynet LTD.</td>
  <td class="user-data-row__col t-right">&nbsp;</td>
  <td class="user-data-row__col user-data-row__col--action t-right">
    <ButtonWithDropdown
      iconButton
      position="right"
      buttonProps={{ style: 'ghost', size: 'tiny' }}
    >
      <svelte:fragment slot="label">
        <span class="visually-hidden">Open action dropdown</span>
        <span
          class="user-data-row__action-icon t-color-text-2"
          aria-hidden="true"
        >
          <IconDots />
        </span>
      </svelte:fragment>
      <DropdownLinkList slot="content">
        <li>
          <DropdownItem href={ROUTES.ADMIN_CONSOLE_USER_EDIT('example-id')}>
            <IconEdit />
            Edit user</DropdownItem
          >
        </li>
      </DropdownLinkList>
    </ButtonWithDropdown>
  </td>
</tr>

<style>
  .user-data-row {
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

    &__col {
      padding: 30px 28px 18px 0;

      &:last-child {
        padding-right: 0;
      }

      @media (--screen-large-max) {
        padding: 0 40px 0 0;
        display: block;
      }

      &--action {
        @media (--screen-large-max) {
          padding-right: 0;
          position: absolute;
          top: 32px;
          right: 0;
        }
      }
    }
  }
</style>
