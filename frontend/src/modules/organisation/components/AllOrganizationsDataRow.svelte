<script lang="ts">
  import { goto } from '$app/navigation';
  import IconPerson from 'icons/person-12.svg';
  import Button from 'components/Button/Button.svelte';
  import DropdownItem from 'components/Dropdown/DropdownItem.svelte';
  import DropdownLinkList from 'components/Dropdown/DropdownList.svelte';
  import { toast } from 'components/Toast/Toast';
  import { ENDPOINTS } from 'consts/endpoints';
  import { ROUTES } from 'consts/routes';
  import IconDelete from 'icons/close-12.svg';
  import IconDots from 'icons/dots-12.svg';
  import IconEdit from 'icons/pencil-12.svg';
  import ButtonWithDropdown from 'modules/app/components/ButtonWithDropdown/ButtonWithDropdown.svelte';
  import ConfirmDeleteModal from 'modules/app/components/ConfirmDeleteModal/ConfirmDeleteModal.svelte';
  import { useForm } from 'svelte-use-form';
  import { fade } from 'svelte/transition';
  import { getUserInfo } from 'utils';
  import { httpClient } from 'utils/httpClient';
  import type { Organisation } from '../models/Organisation';
  import { getOrganisationsByUserId } from '../store/organisationStore';
  import OrganisationMembersManagement from './OrganisationMembersManagement.svelte';
  import RenameOrganisation from './RenameOrganisation.svelte';

  export let item: Organisation;
  export let index: number;

  let isModalOpen: boolean = false;
  let renameModalOpen: boolean = false;
  let deleteModalOpen = false;
  let deleting: boolean = false;

  const deleteForm = useForm({
    targetValue: {
      initial: 'DELETE',
    },
  });

  function handleConfirm() {
    deleting = true;

    httpClient
      .delete(ENDPOINTS.ORGANISATIONS.DELETE_ORGANISATION(item.id))
      .then((res) => {
        deleting = false;
        getOrganisationsByUserId(getUserInfo().id);
        deleteModalOpen = false;
      })
      .catch((err) => {
        toast.warning(err);
        deleting = false;
      });
  }
</script>

<tr
  class="table__row organisation-data-row"
  in:fade={{ duration: 250, delay: index * 100 }}
>
  <td class="organisation-data-row__col"
    ><a
      class="u-link-reset"
      href={ROUTES.ADMIN_CONSOLE_ORGANISATIONS_EDIT(item.id)}>{item.name}</a
    ></td
  >
  <td class="organisation-data-row__col">{item.member_count}</td>
  <td class="organisation-data-row__col  t-right">
    <div class="organisation-data-row__col--action">
      <div class="s-right--small">
        <Button
          on:click={() => {
            isModalOpen = true;
          }}
          size="small"
          style="outline"
        >
          <IconPerson />
          Members
        </Button>
      </div>
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
            <DropdownItem
              size="large"
              as="button"
              on:click={() => (renameModalOpen = true)}
            >
              <IconEdit />
              Rename</DropdownItem
            >
          </li>
          <li>
            <DropdownItem
              size="large"
              as="button"
              on:click={() =>
                goto(ROUTES.ADMIN_CONSOLE_ORGANISATIONS_EDIT(item.id))}
            >
              <IconEdit />
              Edit</DropdownItem
            >
          </li>
          <li>
            <DropdownItem
              size="large"
              as="button"
              on:click={() => (deleteModalOpen = true)}
            >
              <IconDelete />
              Delete</DropdownItem
            >
          </li>
        </DropdownLinkList>
      </ButtonWithDropdown>
    </div>
  </td>
</tr>
<OrganisationMembersManagement
  handleModalClose={() => (isModalOpen = false)}
  {isModalOpen}
  organisationId={item.id}
  organisationName={item.name}
/>
{#if deleteModalOpen}
  <ConfirmDeleteModal
    id="delete-organisation-modal"
    form={deleteForm}
    on:submit={(e) => {
      e.preventDefault();
      handleConfirm();
    }}
    isLoading={deleting}
    isModalOpen={deleteModalOpen}
    handleModalClose={() => (deleteModalOpen = false)}
  >
    <svelte:fragment slot="label">
      Type “DELETE” to confirm deletion of organisation <strong
        >{item.name}</strong
      >. All users will be removed and all data will be lost.
    </svelte:fragment>
  </ConfirmDeleteModal>
{/if}
{#if renameModalOpen}
  <RenameOrganisation
    isModalOpen={renameModalOpen}
    organisationId={item.id}
    organisationName={item.name}
    handleModalClose={() => (renameModalOpen = false)}
  />
{/if}

<style>
  .organisation-data-row {
    @media (--screen-smaller-max) {
      display: block;
      position: relative;
    }
  }
  .organisation-data-row__col {
    padding-bottom: 18px;
    padding-top: 12px;
  }

  .organisation-data-row__col--action {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .organisation-data-row {
    @media (--screen-smaller-max) {
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
