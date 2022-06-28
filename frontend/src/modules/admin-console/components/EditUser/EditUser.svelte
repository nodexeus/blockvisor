<script lang="ts">
  import GroupTitle from 'components/GroupTitle/GroupTitle.svelte';
  import IconUser from 'icons/person-12.svg';
  import ButtonWithDropdown from 'modules/app/components/ButtonWithDropdown/ButtonWithDropdown.svelte';
  import OrganisationManagement from 'modules/organisation/components/OrganisationManagement.svelte';
  import DropdownLinkList from 'components/Dropdown/DropdownList.svelte';
  import DropdownItem from 'components/Dropdown/DropdownItem.svelte';

  import IconEdit from 'icons/pencil-12.svg';

  import IconDots from 'icons/dots-12.svg';
  import IconTrash from 'icons/trash-12.svg';
  import IconLock from 'icons/lock-12.svg';
  import PersonalInformation from './PersonalInformation.svelte';
  import Tabs from 'modules/app/components/Tabs/Tabs.svelte';
  import Invoices from './Invoices.svelte';
  import Billing from './Billing.svelte';
  import Notifications from './Notifications.svelte';
  import ChangePassword from './ChangePassword.svelte';

  let items = [
    { label: 'Personal Information', value: 1, component: PersonalInformation },
    { label: 'Organisations', value: 2, component: OrganisationManagement },
    { label: 'Billing', value: 3, component: Billing },
    { label: 'Invoices', value: 4, component: Invoices },
    { label: 'Notifications', value: 5, component: Notifications },
  ];

  let changePasswordModalVisible = false;

  function handleClose() {
    changePasswordModalVisible = false;
  }
</script>

<GroupTitle>
  <svelte:fragment slot="title">Mark Bryant</svelte:fragment>
  <svelte:fragment slot="stats">
    <IconUser />
    Basic user
  </svelte:fragment>
  <ButtonWithDropdown
    slot="action"
    buttonProps={{ style: 'ghost', size: 'small' }}
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
        <DropdownItem
          as="button"
          size="large"
          on:click={() => (changePasswordModalVisible = true)}
        >
          <IconLock />
          Change Password</DropdownItem
        >
      </li>
      <li>
        <DropdownItem as="button" size="large">
          <IconTrash />
          Remove User</DropdownItem
        >
      </li>
    </DropdownLinkList>
  </ButtonWithDropdown>
</GroupTitle>
<article class="edit-user__content">
  <Tabs {items} />
</article>
<ChangePassword
  handleModalClose={handleClose}
  isModalOpen={changePasswordModalVisible}
/>

<style>
  .edit-user {
    &__content {
      margin-top: 40px;
      width: 100%;
    }
  }
</style>
