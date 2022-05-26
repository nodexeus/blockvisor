<script lang="ts">
  import { browser } from '$app/env';
  import DropdownItem from 'components/Dropdown/DropdownItem.svelte';
  import DropdownLinkList from 'components/Dropdown/DropdownList.svelte';
  import { ROUTES } from 'consts/routes';
  import { formatDistanceToNow } from 'date-fns';
  import IconDelete from 'icons/close-12.svg';
  import IconDots from 'icons/dots-12.svg';
  import IconEdit from 'icons/pencil-12.svg';
  import ButtonWithDropdown from 'modules/app/components/ButtonWithDropdown/ButtonWithDropdown.svelte';
  import SimpleConfirmDeleteModal from 'modules/app/components/SimpleConfirmDeleteModal/SimpleConfirmDeleteModal.svelte';
  import { deleteBroadcastById } from 'modules/broadcasts/store/broadcastStore';
  import { fade } from 'svelte/transition';

  let isModalOpen: boolean = false;
  let deleting: boolean = false;
  export let item: Broadcast;
  export let index: number;

  function handleModalClose() {
    isModalOpen = false;
  }
  function handleConfirm() {
    deleting = true;

    deleteBroadcastById(item.id, item.org_id).then((res) => {
      deleting = false;
      handleModalClose();
    });
  }
</script>

<tr
  in:fade={{ duration: 250, delay: index * 200 }}
  class="table__row broadcast"
>
  <td class="broadcast__col">
    <span class="t-line-clamp--2" title="My Broadcast really long name "
      >{item.name}
    </span>
  </td>
  <td class="t-small t-color-text-2 broadcast__col"
    >{item.created_at && formatDistanceToNow(+new Date(item.created_at))} ago</td
  >
  <td class="t-small t-color-text-2 broadcast__col broadcast__col--trunc"
    >{item.addresses}</td
  >
  <td class="t-small t-color-text-2 broadcast__col">{item.txn_types}</td>
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
          <DropdownItem href={ROUTES.BROADCAST_EDIT(item?.id)}>
            <IconEdit />
            Edit</DropdownItem
          >
        </li>
        <li class="node__item-divider">
          <DropdownItem
            as="button"
            on:click={() => {
              isModalOpen = true;
            }}
          >
            <IconDelete />
            Delete</DropdownItem
          >
        </li>
      </DropdownLinkList>
    </ButtonWithDropdown>
  </td>
</tr>

{#if browser && isModalOpen}
  <SimpleConfirmDeleteModal
    isModalOpen={true}
    {handleModalClose}
    id="delete-broadcast"
    on:cancel={handleModalClose}
    on:confirm={handleConfirm}
    loading={deleting}
  >
    <p slot="content">This action will delete {item.name} broadcast.</p>
  </SimpleConfirmDeleteModal>
{/if}

<style>
  .broadcast {
    padding-bottom: 60px;

    @media (--screen-smaller-max) {
      display: block;
      position: relative;
    }
  }
  .broadcast__col {
    position: relative;
    vertical-align: middle;
    padding: 24px 40px 20px 0;

    @media (--screen-smaller-max) {
      display: block;
      padding: 12px 0;

      &:first-child {
        padding-right: 60px;
      }
    }

    &:last-child {
      padding-right: 0;
    }
  }

  .broadcast__col--trunc {
    text-overflow: ellipsis;
    overflow: hidden;
    max-width: 180px;
  }
</style>
