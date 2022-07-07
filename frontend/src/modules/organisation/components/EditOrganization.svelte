<script lang="ts">
  import GroupTitle from 'components/GroupTitle/GroupTitle.svelte';
  import IconUser from 'icons/person-12.svg';
  import IconEdit from 'icons/pencil-12.svg';
  import Tabs from 'modules/app/components/Tabs/Tabs.svelte';
  import Billing from 'modules/admin-console/components/EditUser/Billing.svelte';
  import Invoices from 'modules/admin-console/components/EditUser/Invoices.svelte';

  import OrganisationDetails from './OrganisationDetails.svelte';
  import { selectedOrganization } from '../store/organisationStore';
  import OrganizationNotifications from './OrganizationNotifications.svelte';

  let items = [
    {
      label: 'Organisation Details',
      value: 1,
      component: OrganisationDetails,
    },
    { label: 'Billing', value: 2, component: Billing },
    { label: 'Invoices', value: 3, component: Invoices },
    {
      label: 'Notifications',
      value: 4,
      component: OrganizationNotifications,
    },
  ];

  let memberCount = $selectedOrganization?.member_count ?? 0;
</script>

<GroupTitle>
  <svelte:fragment slot="title">{$selectedOrganization?.name}</svelte:fragment>
  <svelte:fragment slot="stats">
    <IconUser />
    {#if memberCount > 1}
      {memberCount} members
    {:else}
      {memberCount} member
    {/if}
  </svelte:fragment>
</GroupTitle>
<article class="edit-organization__content">
  <Tabs {items} />
</article>

<style>
  .edit-organization__content {
    margin-top: 40px;
    width: 100%;
  }
</style>
