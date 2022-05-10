<script lang="ts">
  import DataState from 'components/DataState/DataState.svelte';

  import DropdownLinkList from 'components/Dropdown/DropdownList.svelte';
  import DropdownItem from 'components/Dropdown/DropdownItem.svelte';

  import IconAccount from 'icons/person-12.svg';
  import IconDocument from 'icons/document-12.svg';
  import IconDots from 'icons/dots-12.svg';
  import ButtonWithDropdown from 'modules/app/components/ButtonWithDropdown/ButtonWithDropdown.svelte';

  export let address_name;
  export let status;
  export let ip_addr;
  export let url;
  export let location;

  const classes = ['table__row host-data-row', `host-data-row--${status}`].join(
    ' ',
  );
</script>

<tr class={classes}>
  <td class="host-data-row__col">
    <a class="u-link-reset" href={url}>
      {address_name}
    </a>
    <small class="host-data-row__info t-small">
      <span>
        {ip_addr}
      </span>
      <address>
        {location}
      </address>
    </small>
  </td>
  <td class="host-data-row__col t-uppercase t-microlabel host-data-row__state">
    <DataState state={status} />
  </td>
  <td class="host-data-row__col t-right">
    <ButtonWithDropdown
      position="right"
      iconButton
      buttonProps={{ style: 'ghost', size: 'tiny' }}
      slot="action"
    >
      <svelte:fragment slot="label">
        <IconDots />
      </svelte:fragment>
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
      </DropdownLinkList>
    </ButtonWithDropdown>
  </td>
</tr>

<style>
  .host-data-row {
    position: relative;

    & :global(.dropdown) {
      margin-top: 0;
    }

    &--issue {
      color: theme(--color-utility-warning);

      & :global(.data-state) {
        color: theme(--color-utility-warning);

        & :global(svg) {
          animation: blink 1s var(--transition-easing-cubic) infinite alternate;
        }
      }
    }

    @media (--screen-medium-larger-max) {
      padding-top: 32px;
      padding-bottom: 18px;
      display: block;
      width: 100%;
      position: relative;
    }

    &__col {
      padding: 24px 0 12px;
      vertical-align: top;

      @media (--screen-medium-larger-max) {
        padding: 0;
        display: block;
      }
    }

    &__info {
      display: flex;
      flex-wrap: wrap;
      gap: 12px;
      color: theme(--color-text-2);
      padding-top: 8px;
    }

    &__state {
      padding-top: 28px;
      color: theme(--color-text-3);
    }

    &__link {
      display: inline-block;
      padding: 6px 12px;

      @media (--screen-medium-larger-max) {
        position: absolute;
        bottom: 16px;
        right: 0;
      }
    }
  }
</style>
