<script lang="ts">
  import { fade } from 'svelte/transition';
  import { pageTransition } from 'consts/animations';
  import ActionTitleHeader from 'components/ActionTitleHeader/ActionTitleHeader.svelte';
  import GroupTitle from 'components/GroupTitle/GroupTitle.svelte';
  import IconUser from 'icons/person-12.svg';
  import HeaderControls from 'modules/admin-console/components/HeaderControls/HeaderControls.svelte';
  import Button from 'components/Button/Button.svelte';
  import { useForm } from 'svelte-use-form';
  import { ROUTES } from 'consts/routes';
  import { onMount } from 'svelte';
  import { app } from 'modules/app/store';
  import AllOrganisationsManagement from 'modules/organisation/components/AllOrganisationsManagement.svelte';
  import CreateNewOrganisation from 'modules/organisation/components/CreateNewOrganisation.svelte';
  import LoadingSpinner from 'components/Spinner/LoadingSpinner.svelte';
  import {
    isLoading,
    organisations,
  } from '../../../../modules/organisation/store/organisationStore';

  const form = useForm();

  let createNewActive: boolean = false;

  let numberOfOrgs = $organisations?.length ?? 0;
  let numberOfMembers =
    $organisations?.reduce((acc, obj) => acc + obj.member_count, 0) ?? 0;

  function handleClickOutsideCreateNew() {
    createNewActive = false;
  }

  onMount(() => {
    setTimeout(() => {
      app.setBreadcrumbs([
        {
          title: 'Admin Console',
          url: ROUTES.ADMIN_CONSOLE_DASHBOARD,
        },
        {
          title: 'Organisations',
          url: ROUTES.ADMIN_CONSOLE_ORGANISATIONS,
        },
      ]);
    }, 200);
  });
</script>

<section in:fade|local={pageTransition}>
  <ActionTitleHeader className="container--pull-back">
    <Button
      on:click={() => (createNewActive = true)}
      style="primary"
      size="small">Create New</Button
    >
    <h2 class="t-large" slot="title">Organization Management</h2>
  </ActionTitleHeader>
</section>

<section in:fade|local={pageTransition} class="container--medium-large">
  <GroupTitle>
    <svelte:fragment slot="title">Organizations</svelte:fragment>

    <svelte:fragment slot="stats">
      <IconUser />
      {numberOfMembers} users in {numberOfOrgs} organizations
    </svelte:fragment>
  </GroupTitle>

  <HeaderControls />
  <AllOrganisationsManagement />
</section>

<CreateNewOrganisation
  isModalOpen={createNewActive}
  handleModalClose={handleClickOutsideCreateNew}
/>
