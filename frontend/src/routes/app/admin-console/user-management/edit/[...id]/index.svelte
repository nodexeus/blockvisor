<script lang="ts">
  import { fade } from 'svelte/transition';
  import { pageTransition } from 'consts/animations';
  import ActionTitleHeader from 'components/ActionTitleHeader/ActionTitleHeader.svelte';
  import GroupTitle from 'components/GroupTitle/GroupTitle.svelte';
  import IconUser from 'icons/person-12.svg';
  import UserDetails from 'modules/admin-console/components/UserDetails/UserDetails.svelte';
  import { onMount } from 'svelte';
  import { app } from 'modules/app/store';
  import { ROUTES } from 'consts/routes';
  import EditUser from 'modules/admin-console/components/EditUser/EditUser.svelte';

  onMount(() => {
    setTimeout(() => {
      app.setBreadcrumbs([
        {
          title: 'Admin Console',
          url: ROUTES.ADMIN_CONSOLE_DASHBOARD,
        },
        {
          title: 'User Management',
          url: ROUTES.ADMIN_CONSOLE_USER_MANAGEMENT,
        },
        {
          title: 'Edit User',
          url: '',
        },
      ]);
    }, 200);
  });

  const hasEditAccess = true;
</script>

<section in:fade|local={pageTransition}>
  <ActionTitleHeader className="container--pull-back">
    <h2 class="t-large" slot="title">Edit User</h2>
  </ActionTitleHeader>
</section>

<section in:fade|local={pageTransition} class="edit container--medium-large">
  {#if hasEditAccess}
    <EditUser />
  {:else}
    <GroupTitle>
      <svelte:fragment slot="title">Mark Bryant</svelte:fragment>
      <svelte:fragment slot="stats">
        <IconUser />
        Basic user
      </svelte:fragment>
    </GroupTitle>
    <UserDetails />
  {/if}
</section>

<style>
  .edit {
    & :global(.dropdown) {
      right: 0;
    }

    & :global(.group-title) {
      justify-content: space-between;
    }
  }
</style>
