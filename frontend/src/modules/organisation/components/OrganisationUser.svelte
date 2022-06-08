<script lang="ts">
  import Button from 'components/Button/Button.svelte';
  import DropdownItem from 'components/Dropdown/DropdownItem.svelte';
  import DropdownLinkList from 'components/Dropdown/DropdownList.svelte';
  import { ENDPOINTS } from 'consts/endpoints';
  import IconCaret from 'icons/caret-micro.svg';
  import ButtonWithDropdown from 'modules/app/components/ButtonWithDropdown/ButtonWithDropdown.svelte';
  import ConfirmDeleteModal from 'modules/app/components/ConfirmDeleteModal/ConfirmDeleteModal.svelte';
  import { useForm } from 'svelte-use-form';
  import { httpClient } from 'utils/httpClient';
  import type { OrgUser } from '../models/OrgUser';

  export let pending: boolean = false;
  export let item: OrgUser;
</script>

<article
  class={`organisation-user ${pending ? 'organisation-user--pending' : ''}`}
>
  <p>
    <span class="organisation-user__initials">JD</span>{item.user_id} (You)
    <span class="pending">Pending</span>
  </p>
  <div class="organisation-user__action">
    <div class="s-right--small">
      <ButtonWithDropdown
        slot="action"
        buttonProps={{ style: 'ghost', size: 'small' }}
      >
        <svelte:fragment slot="label">
          <span class="visually-hidden">Open action dropdown</span>
          <span class="organisation-user__action" aria-hidden="true">
            <span class="s-right--medium-small t-capitalize">{item.role}</span>
            <IconCaret />
          </span>
        </svelte:fragment>
        <DropdownLinkList slot="content">
          <li>
            <DropdownItem
              size="large"
              as="button"
              on:click={() => {
                handleChange('owner');
              }}>Owner</DropdownItem
            >
          </li>
          <li>
            <DropdownItem
              size="large"
              as="button"
              on:click={() => {
                handleChange('user');
              }}>User</DropdownItem
            >
          </li>
        </DropdownLinkList>
      </ButtonWithDropdown>
    </div>
    <Button size="small" style="outline"
      >{pending ? 'Revoke Invitation' : 'Revoke'}</Button
    >
  </div>
</article>

<style>
  .organisation-user {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 18px 0px;
    border-bottom: 1px solid theme(--color-border-2);

    & :global(.dropdown) {
      right: 0;
      min-width: 160px;
      margin-top: 8px;
    }
  }

  .pending {
    display: none;
  }

  .organisation-user--pending {
    .organisation-user__initials {
      background: transparent;
      border: 1px solid theme(--color-border-2);
      color: theme(--color-text-4);
    }

    .pending {
      display: inline;
      color: var(--color-utility-note);
      text-transform: uppercase;
      font-size: 10px;
      font-weight: 400;
    }
  }

  .organisation-user__action {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .organisation-user__initials {
    display: inline-flex;
    justify-content: center;
    align-items: center;
    background: var(--color-primary);
    width: 24px;
    height: 24px;
    border-radius: 50%;
    color: var(--color-text-1);
    font-weight: var(--font-weight-normal);
    font-size: 10px;
    margin-right: 12px;
  }
</style>
